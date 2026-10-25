# Workload instructions, placement and footprint

These linked-code observations explain the limits of the
[clocked handoff](../../results/clocked-handoff-2026-10-23/README.md) and
[alignment](../../results/alignment-probe-2026-10-23/README.md) studies.
Instruction equality does not establish equal placement or execution cost.

## Exact files

| Platform | Final release ELF | Verified SHA-256 |
| --- | --- | --- |
| Bare metal | `firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal` | `be14bac3563b52d5e043485c9f798a4e23d6ed8702b72c9a3028c1cf46f58d0b` |
| Embassy/esp-rtos | `firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy` | `8d06354ca14e9a51550fd3183c9f29750f32fc73880140c6d9ca8151e0a18d44` |
| ESP-IDF/FreeRTOS | `firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline` | `2c29bbbc2c7fe9ebb433907d9a2121c121ad67c460d4f1ead99f50e552991cf9` |

Each corresponding `artifacts/<platform>/firmware.elf` in the experiment directory
has the exact same full-file hash as its release ELF. These identities matter:
source or compiler settings alone do not identify a measured binary.

## Body identity

Symbol sizes from `nm --print-size` are `0x36` in every ELF. HAL uses symbol
`_ZN10bench_core8workload17h2bec9f0ad370606aE`; ESP-IDF uses
`_RNvCs3RkLThg6PCt_10bench_core8workload`.

| Platform | Workload address | Executable section / virtual base | Section file offset | Function file offset |
| --- | --- | --- | --- | --- |
| Bare metal | `0x42011a3e` | `.text` / `0x42010020` | `0x8020` | `0x9a3e` = 39486 |
| Embassy/esp-rtos | `0x42010358` | `.text` / `0x42010020` | `0x9020` | `0x9358` = 37720 |
| ESP-IDF/FreeRTOS | `0x42000cd4` | `.flash.text` / `0x42000020` | `0xf020` | `0xfcd4` = 64724 |

File offset is section file offset + symbol address - section virtual base.
Direct `dd` reads of those 54-byte ranges produced the common hash above.
Pairwise `cmp` of the extracted ranges found no difference between the new
binaries, or between new bare metal and the archived pilot bare-metal body
at file offset 34392. That pilot range was independently rehashed and matches.

The exact common bytes, in file order, are:

```text
95c901473786379eb7d6eb851306969b9386b6a693071700318f3a951357
95011e05598d3305d50213570501398d3e87e392f5fe8280
```

Disassembly confirms nine 16-bit compressed instructions and nine 32-bit
instructions. Relative targets and operands are identical; displayed absolute
addresses differ with placement. The body is:

```text
beqz a1, +0x34
li   a4, 0
lui  a2, 0x9e378
lui  a3, 0x85ebd
addi a2, a2, -0x647
addi a3, a3, -0x595
addi a5, a4, 1       # loop starts at +0x14
xor  a4, a4, a2
add  a0, a0, a4
srli a4, a0, 25
slli a0, a0, 7
or   a0, a0, a4
mul  a0, a0, a3
srli a4, a0, 16
xor  a0, a0, a4
mv   a4, a5
bne  a1, a5, -0x1c   # instruction at +0x30 branches to +0x14
ret
```

The initial zero-round guard, loop counter/branch, wrapping addition, rotate
implemented by shifts/or, multiply, xor and return are present. This is evidence
that the timed compute function has not become a constant return or disappeared
under optimization. It does not prove identical callers, timer paths, caches,
interrupts, task scheduling or surrounding application code. It does not establish
that Rust channels and Embassy channels execute the same machine-code path.

Both firmware directories pin Rust 1.96.0 and use opt-level 3, thin LTO, one
codegen unit, debug information and panic abort. Targets differ: HAL is
`riscv32imac-unknown-none-elf`; IDF is `riscv32imac-esp-espidf` with rebuilt std.
The common source is `bench_core::workload`, marked `inline(never)`. Exact body
identity is a stronger control for this function than matching those settings
alone, while code placement and runtime dependencies remain separate controls.

## Alignment confound: hypothesis under investigation

The first new bare-metal physical run passed protocol validation and reports
CPU 160 MHz / MSPI source 80 MHz / flash 80 MHz / divider 1 in its captured
register readback. Its compute p50 is 308 microseconds; the pilot's five
bare-metal boot p50 values were 283 microseconds. See
[new boot 1](../../results/clocked-handoff-2026-10-23/runs/01-bare-metal/run.json)
and the [pilot review](../../results/initial-2026-10-22/README.md). Preserve the observed values.
Matching workload bytes do not remove a change in their placement.

Read-only disassembly of all six old/new ELFs establishes:

| Build | Function entry | Loop target: 32-bit `addi` at +0x14 | Backedge: 32-bit `bne` at +0x30 | Entry/target/backedge modulo 4 |
| --- | --- | --- | --- | --- |
| Pilot bare metal | `0x42011658` | `0x4201166c` | `0x42011688` | 0 / 0 / 0 |
| New bare metal | `0x42011a3e` | `0x42011a52` | `0x42011a6e` | 2 / 2 / 2 |
| Pilot and new Embassy | `0x42010358` | `0x4201036c` | `0x42010388` | 0 / 0 / 0 |
| Pilot ESP-IDF | `0x42000ca8` | `0x42000cbc` | `0x42000cd8` | 0 / 0 / 0 |
| New ESP-IDF | `0x42000cd4` | `0x42000ce8` | `0x42000d04` | 0 / 0 / 0 |

The 32-byte loop has eleven instructions, including five 32-bit instructions
at relative offsets +0x14, +0x1c, +0x24, +0x28 and +0x30. Every one of these
straddles a four-byte word boundary in new bare metal, while none does in the
other inspected placements. In particular, the taken branch redirects to a
32-bit instruction beginning two bytes into a word. Both the old and new
bare-metal loop span two 32-byte cache blocks; neither placement makes one
of those five 32-bit instructions cross a 32-byte cache-block boundary. The
word alignment is a distinct difference even though the body bytes match.

The [official C6 datasheet v1.5, sections 4.1.1.1 and 4.1.2.2](https://documentation.espressif.com/esp32-c6_datasheet_en.html)
documents a four-stage HP CPU, a branch target buffer, support for 32-bit
instruction fetch from mapped flash, and a cache with 32-byte blocks. These
make instruction-fetch/branch-target placement a plausible performance
confound. They do not specify an extra-cycle rule for this exact alignment.
The [official TRM v1.2, chapter 1](https://documentation.espressif.com/esp32-c6_technical_reference_manual_en.pdf)
also documents branch/jump-hazard performance-counter events, but its inspected
HP CPU chapter does not establish a numerical halfword-aligned fetch penalty.

At 160 MHz, one additional cycle per 4,096 rounds costs 25.6 microseconds;
one per 4,095 taken backedges costs 25.59375 microseconds. This is close to the
observed shift and is a useful hypothesis, **not verified causality**. Timer
quantization is one microsecond, historical and current binaries differ beyond
placement, and the pilot lacks the new runtime clock snapshots. No alignment
change, counter measurement or paired placement experiment was performed here.
The current evidence therefore cannot attribute the compute gap to OS cost or
claim an exact one-cycle hardware penalty.

A separately versioned follow-up can hold the runtime/clock/configuration
constant while controlling function/loop alignment and, if available through
the selected Rust tooling, collecting cycle/instruction/jump-hazard counters.
Keep this dataset intact; do not pad or relocate a captured binary and reuse
its existing results.

The subsequent 2026-10-23 [prior-work audit](prior-work-c6-os.md)
identified LLTA's already published C6 timing notes on alignment-sensitive
control-flow costs. The local probe is replication/calibration, not discovery
of a new phenomenon. Those notes also distinguish data-access span penalties
from an instruction-fetch probe that failed to link; neither establishes the
specific cause or exact penalty of this workload's placement change.

## Image and section interpretation

The preserved merged images have boot headers at offset 0 and application
headers at `0x10000`. All six inspected headers have magic `e9`, mode byte `02`
and flash size/frequency byte `3f`: nominal DIO, 8 MB, 80 MHz. This interpretation
matches the inspected espflash 4.6.0
[image-header packing](https://github.com/esp-rs/espflash/blob/c1fb39939c1b5880b2aec7f43063f19689d5b00a/espflash/src/image_format/idf.rs)
and [C6 frequency encoding](https://github.com/esp-rs/espflash/blob/c1fb39939c1b5880b2aec7f43063f19689d5b00a/espflash/src/target/mod.rs).
Header agreement does not establish final runtime clocks. The new protocol's
before/after PCR/SPI0 readback and the host's independent decode supply that
separate evidence; physical clock measurement remains a different check.

| Platform | Merged image bytes | Debug ELF file bytes | Merged image SHA-256 |
| --- | ---: | ---: | --- |
| Bare metal | 168688 | 1514724 | `f404a4840ea2c6d700140e37550f627c00b6463e4dee0f962d2ee979f5e80053` |
| Embassy/esp-rtos | 175888 | 2008324 | `334cd1bcef292f906af960598931b988da91c467ddf16b54eebc0987f8534bfa` |
| ESP-IDF/FreeRTOS | 460464 | 8833444 | `2e5a0d693d186d6f3277c5f9180afc8cda1611c2f7752b280d22711f0af65863` |

Merged lengths include the selected bootloader, partitions, address padding and
application image. ELF lengths also include symbols/debug information. Neither
is a minimum OS footprint or a peak RAM measurement. Header inspection here did
not revalidate every image segment/checksum or establish an on-board pass.

The ELF section inventory still requires semantic accounting:

| Section / reservation | Bare metal | Embassy/esp-rtos | ESP-IDF/FreeRTOS | Interpretation |
| --- | ---: | ---: | ---: | --- |
| `.stack` | 442832 B | 420572 B | Absent | HAL linker reservation for remaining RAM, not observed stack use or summed task-stack consumption. |
| `.text_gap` | 53128 B | 50312 B | Absent | SHT_NOBITS flash address-alignment reservation, not file payload or RAM. |
| `.flash_rodata_dummy` | Absent | Absent | 327680 B | SHT_NOBITS virtual flash reservation starting at `0x42000020`, overlapping `.flash.text`; it has SHF_WRITE but is not RAM or extra loaded code. |
| `.rtc_reserved` | Absent | Absent | 24 B | Actual RTC address reservation at `0x50003fe8`; retain its separate meaning. |
| `.iram0.text_end` / `.dram0.heap_start` | Absent | Absent | 2 B / 8 B | Explicit alignment/heap-boundary reservations; not application payload. |

IDF's `.flash.init_array` also has SHF_WRITE, but its virtual address is
`0x4205e620` in the flash mapping; a generic writable-section sum would mislabel
it. Address, type, load contents and linker meaning must all inform footprint
accounting. Removing reservations from a payload total does not make reserved
addresses freely available. No dynamic heap or stack high-water result follows
from this inspection.

LLVM's IDF disassembly labels the workload's internal targets as
`_flash_rodata_dummy_start+0xce8` and `+0xcc8` because that linker symbol shares
the `.flash.text` base address. The actual target addresses remain within the
54-byte executable workload body. Those printed aliases do not indicate a
branch into non-executable rodata or a workload mismatch.

## Reproduction

Commands run from the repository root. LLVM tools are the pinned Rust 1.96.0
`llvm-tools` component, LLVM `22.1.2-rust-1.96.0-stable`, under
`/Users/builduser/.rustup/toolchains/1.96.0-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/`.
GNU nm/objdump from cached Espressif GCC 14.2.0 supplied an independent symbol
and section listing. macOS `dd`, `cmp`, `shasum`, `xxd` and `wc` performed direct
file inspections. Resolve `llvm-*` from the directory above:

```text
llvm-nm --print-size firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal
llvm-nm --print-size firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy
llvm-nm --print-size firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline
llvm-objdump --disassemble-symbols=_ZN10bench_core8workload17h2bec9f0ad370606aE firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy
llvm-objdump --disassemble-symbols=_RNvCs3RkLThg6PCt_10bench_core8workload firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline
llvm-readobj --sections <each-final-ELF>
dd if=firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal bs=1 skip=39486 count=54 2>/dev/null | shasum -a 256
dd if=firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy bs=1 skip=37720 count=54 2>/dev/null | shasum -a 256
dd if=firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline bs=1 skip=64724 count=54 2>/dev/null | shasum -a 256
dd if=results/initial-2026-10-22/artifacts/bare-metal/firmware.elf bs=1 skip=34392 count=54 2>/dev/null | shasum -a 256
xxd -l 24 <each-merged-image>
xxd -s 65536 -l 24 <each-merged-image>
```

All expected body/full-ELF identities passed. Final `git diff --check` for this
review passed. These checks establish binary comparability for compute and
identify footprint/reporting traps; they do not replace the physical run's
clock, checksum, completion and stability evidence.
