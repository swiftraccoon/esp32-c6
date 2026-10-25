# Bare-metal alignment probe build

Prepared 2026-10-23 as a separate exploratory experiment after the main
clocked-handoff captures. No Rust source, Cargo manifest/lockfile, toolchain
file or repository Cargo configuration changed. The [binary review](../research/code-placement.md#alignment-confound-hypothesis-under-investigation)
records why placement warrants investigation. This file establishes a build
and image, not an on-board result; the operator owns capture and restoration.

The probe adds one compiler option, `-C llvm-args=-align-all-functions=2`, to
all target Rust crates through command-scoped `RUSTFLAGS`. LLVM's option uses
log2 alignment, so 2 requests four-byte function alignment. It is an unstable
LLVM implementation option tied to the pinned compiler, not a portable Rust
source/API guarantee. All other selected compiler/profile/runtime/clock settings
remain those of the main bare-metal adapter.

## Reproduction and isolation

From `firmware/esp-hal`:

```sh
RUSTFLAGS='-C llvm-args=-align-all-functions=2' \
  cargo +1.96.0 build --verbose --release --locked --bin bare-metal \
  --target-dir ../../.cache/aligned-hal-target
```

Cargo's repository config still selects `riscv32imac-unknown-none-elf`.
`build.rs` still supplies `-C link-arg=-Tlinkall.x`; the retained verbose log
shows this linker option and the added LLVM option on the final binary, and
the LLVM option on `bench_core`. Normal release controls are opt-level 3,
thin LTO, one codegen unit, panic abort and debug 2. Rust is 1.96.0, commit
`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, LLVM 22.1.2. The separate target
directory is ignored by the existing repository `.cache/` rule. Main release
ELFs were rehashed afterward and remain byte-for-byte unchanged.

The final build passed without warnings in **9.86 seconds**. A preparatory
`cargo rustc -- ...` build had applied the same LLVM option only to the top
crate. Its log is retained separately; it is not the final probe provenance.
The final all-crate flag makes dependency compilation explicit. No routine
formatter/lint test was repeated for unchanged source; prior source/API review
still applies. ELF inspection verifies the actual intended placement.

The authored bare-metal source, measurements, clocks, build script, manifest,
lockfile, Cargo config, toolchain file and shared workload were independently
compared with the main experiment's preserved source archive using `cmp`.
Every comparison passed. The
[source hash manifest](../../results/alignment-probe-2026-10-23/artifacts/source-inputs.sha256)
retains those exact inputs; the shared workload hash is
`27447ee945989a682ee778646f488e7c986d8831a38515e1057eaa143992ab9e`.
No C, assembly, custom linker script or special function body was authored.

From the repository root, create the merged image using the same installed
espflash 4.6.0 and embedded C6 bootloader route as the main HAL image:

```sh
.tools/bin/espflash save-image --skip-update-check --chip esp32c6 \
  --flash-mode dio --flash-freq 80mhz --flash-size 8mb --xtal-freq 40mhz \
  --merge --skip-padding \
  results/alignment-probe-2026-10-23/artifacts/firmware.elf \
  results/alignment-probe-2026-10-23/artifacts/flash.bin
```

Image creation passed. Boot and application header mode/frequency-size bytes
are `02/3f`, nominal DIO 80 MHz / 8 MB. Final runtime clocks still require the
captured before/after register readback. This image creation performs no serial
or hardware access.

## Exact probe artifacts

Artifacts reside under
[results/alignment-probe-2026-10-23/artifacts](../../results/alignment-probe-2026-10-23/artifacts).

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `firmware.elf` | 1514728 | `19184daf8b628b690655fa0ea23453fd5519f630e58450141a7280395d33f741` |
| `flash.bin` | 168784 | `295f5d358f713ce14389a3d769eeeffe106eb73489006ef4d80824198911a8ca` |

`build.log`, `preparatory-build.log`, `save-image.log`, `commands.txt`,
`rustc-version.txt`, `source-inputs.sha256`, `hashes.sha256` and
`workload-disassembly.txt` retain the build, tool and inspection evidence.
`firmware.elf` is copied from the isolated target tree, not rebuilt in the
main firmware output directory.

## Verified machine code

The symbol remains **54 bytes / 18 instructions**, SHA-256
`b917090559ff1edf22ad5ac21975f1a998e5594b9ad23db9344d5c71c2b3af49`.
Direct byte-range hashing at file offset **39496** (`0x9a48`) confirms that it
matches all main/pilot workload bodies. Section `.text` still has virtual base
`0x42010020` and file offset `0x8020`.

| Placement | Main bare metal | Alignment probe |
| --- | --- | --- |
| Function entry | `0x42011a3e` | `0x42011a48` |
| Loop target, `addi a5,a4,1` | `0x42011a52` | `0x42011a5c` |
| Loop `srli a4,a0,25` | `0x42011a5a` | `0x42011a64` |
| Loop `mul a0,a0,a3` | `0x42011a62` | `0x42011a6c` |
| Loop `srli a4,a0,16` | `0x42011a66` | `0x42011a70` |
| Loop backedge `bne` | `0x42011a6e` | `0x42011a78` |
| Every listed address modulo 4 | 2 | 0 |

Every 32-bit loop instruction is now four-byte aligned. Compressed instructions
retain their ordinary two-byte alignment; no padding was inserted inside the
workload. `llvm-objdump` also prints a two-byte `unimp` padding word at
`0x42011a7e`, immediately after the 54-byte symbol. That padding is outside the
function and was excluded from the body hash.

The option affects Rust function placement broadly. The `.text` section grows
from `0x92ac` to `0x930a` (94 bytes) and `.rwtext` from `0x834` to `0x838`
(4 bytes). Caller/timer/helper alignment and code addresses can change too.
This is therefore an exploratory **compiler function-alignment** experiment,
not an isolated change to one loop's address. Even if measured timing returns
to the earlier range, that would support a layout effect; it would not prove
which exact fetch, branch, cache or caller alignment caused it, nor establish
an undocumented one-cycle penalty.

Keep its physical boots separate from the main 15-boot dataset. Preserve the
same counts, seed, rounds, warmups, timer resolution and clock validation, retain
all outliers, identify the compiler flag in provenance, and compare per-boot
statistics. Do not replace the main binary/results with this exploratory variant.
Restore and verify the pre-test firmware after collection.
