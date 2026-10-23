# Build evidence: ESP32-C6 Rust adapters

Recorded 2026-10-22. This document covers compilation and static checks;
on-board captures/results are separate evidence. Both adapters are in
[`firmware/esp-hal`](../../firmware/esp-hal/), with the
[review and applicability decisions](../reviews/esp-hal.md).

## Inputs

- Stable Rust 1.96.0, target `riscv32imac-unknown-none-elf`.
- Rust and formatting editions 2024; dependency-declared Rust floor 1.95
  (1.95 itself was not compiled).
- Direct external pins: esp-hal 1.2.0, esp-bootloader-esp-idf 0.6.0,
  esp-rtos 0.4.0, embassy-executor 0.10.0, embassy-time 0.5.1.
- Local workload dependency: `crates/bench-core`, `integer-mix-v1`, seed
  `0x12345678`, 4096 rounds, checksum `0xa3944d55`.
- Full transitive versions/checksums: firmware `Cargo.lock`.
- Release: opt-level 3, thin LTO, one codegen unit, debug level 2,
  incremental=false, panic=abort. Standard prebuilt target `core`, no `std`,
  no `alloc`, no vendor compiler, no build-std/nightly requirement.
- HAL linker script: `-Tlinkall.x`; upstream application descriptor macro
  retained. No forced frame pointers or authored assembly.
- Planned common image settings for on-board comparison: DIO, 40 MHz flash,
  8 MB physical flash, selected by the hardware owner at image/flash time;
  these are distinct from the fixed 160 MHz application CPU clock. Preserve
  actual flashing commands with each capture; this document asserts no flash.

These released versions were verified with `cargo info` and published package
sources, rather than assuming moving repository-main manifests were released.
esp-rtos 0.4.0 declares `~1.2.0-rc.0` HAL compatibility and Embassy executor 0.10.
The selected released HAL 1.2.0 resolves successfully with that runtime.

An initial scaffold compiled both binaries on installed Rust 1.99.1. The
comparison was then aligned to Rust 1.96.0 after a separate ESP-IDF dependency
failed to compile on 1.99.1. Only the final 1.96.0 artifacts belong in the
cross-runtime measurement set; earlier scaffold artifacts are not results.

## Commands and results

Commands run from `firmware/esp-hal`:

| Command | Result |
| --- | --- |
| `cargo +1.96.0 fmt --package c6-runtime-bench -- --check` | Pass. |
| `cargo +1.96.0 build --locked --release --bin bare-metal` | Pass; 14.37 s, no warnings. |
| `cargo +1.96.0 build --locked --release --features embassy --bin embassy` | Pass; 11.50 s, no warnings. |
| `cargo +1.96.0 clippy --locked --release --bin bare-metal` | Pass; 7.95 s, no warnings. |
| `cargo +1.96.0 clippy --locked --release --features embassy --bin embassy` | Pass; 8.80 s, no warnings. |
| `cargo +1.96.0 doc --locked --release --no-deps --features embassy` | Pass; 0.82 s, no warnings. |

An earlier documentation command omitted `--release`; it succeeded and emitted
the upstream HAL's warning about using its dev profile. Documentation was then
built with the actual release profile as listed above. No dev artifact is used
for measurement.

`cargo fmt --all` was initially attempted while another agent was creating the
host tool in the root workspace; Cargo rejected that temporarily targetless
manifest. Formatting is scoped to `c6-runtime-bench` to avoid formatting
independently owned path dependencies. This was an orchestration condition,
not a firmware compiler failure.

Compiler identity:

```text
rustc 1.96.0 (ac68faa20 2026-05-25)
commit-hash: ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96
host: aarch64-apple-darwin
LLVM version: 22.1.2
```

Final release SHA-256 values before flashing:

| Input/artifact | SHA-256 |
| --- | --- |
| `bare-metal` ELF | `c2c46633ab3d7b9fb1b6ea9541da8746200f27c0c7f0203bdaab53bb8bfe9175` |
| `embassy` ELF | `c6ff29be0b56004b5b9cfe43c2f5e55a30415324abda94fc97a19ae84214894c` |
| firmware `Cargo.lock` | `5f6082fbe117dc7b79a10a7452f79d0407fff370fd2e422d167195d25e4c2687` |
| `crates/bench-core/src/lib.rs` | `27447ee945989a682ee778646f488e7c986d8831a38515e1057eaa143992ab9e` |

Selected upstream configuration was inspected in Cargo build output. Both HAL
builds use an 8192-byte main-stack minimum, stack pointer range checking,
hardware stack-guard monitoring, a 60-byte guard offset, HAL critical-section
implementation, and PMP enabled. Their linker scripts reserve remaining RAM
as stack space; that is not a measured high-water mark. esp-rtos defaults are
100 Hz scheduler tick, stack-pointer range checking and hardware task overflow
detection enabled, software task overflow detection disabled. The default WFI
idle hook is used; the optional automatic light-sleep hook is not configured.
Radio priority settings exist upstream, but no radio task is started here.

Read-only section inspection used the Rust 1.96.0 `llvm-tools` component:

```text
llvm-size -A target/riscv32imac-unknown-none-elf/release/bare-metal target/riscv32imac-unknown-none-elf/release/embassy
```

| ELF section, bytes | Bare metal | Embassy |
| --- | ---: | ---: |
| `.trap` | 1600 | 1980 |
| `.rwtext` | 2100 | 4876 |
| `.data` | 4920 | 5656 |
| `.bss` | 648 | 732 |
| `.flash.appdesc` | 256 | 256 |
| `.rodata` | 11884 | 14188 |
| `.text` | 36494 | 41894 |
| `.text_gap` (alignment reservation) | 53396 | 51092 |
| `.stack` (remaining-RAM reservation) | 442844 | 438868 |

These individual static sections are evidence, not a single claimed RAM or
flash consumption number. `.stack` is not measured stack use; `.text_gap` is
the linker/MMU alignment reservation; debug information is excluded from this
table. Preserve the complete ELF/section inventory for interpretation alongside
the actual bootloader/image format.

## Artifact interpretation and unrun checks

Output paths are
`target/riscv32imac-unknown-none-elf/release/bare-metal` and
`target/riscv32imac-unknown-none-elf/release/embassy` within the firmware directory.
Preserve ELF hashes and compiler identity with the actual run. Their filesystem
sizes contain debug sections and do not measure flash usage or runtime RAM.
Linkable/static section footprints and runtime stack/heap measurements must be
reported separately. No stack high-water, idle-power, scheduler latency,
network/radio, peripheral, security, or on-board timer claim follows from these
builds. No hardware, serial, flash, or simulator operation was performed by
the firmware implementation agent.
