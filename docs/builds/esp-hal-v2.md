# Build evidence: HAL protocol 2 and Embassy message round trips

Recorded 2026-10-22 for [`firmware/esp-hal`](../../firmware/esp-hal/). This is
compilation/static evidence, separate from physical capture results. The
[54-entry review](../reviews/esp-hal-v2.md) covers execution/safety decisions.
The [protocol-1 build record](esp-hal.md) remains historical and its artifact
hashes do not identify these executables.

## Frozen inputs and settings

- Stable Rust **1.96.0**, `riscv32imac-unknown-none-elf`; language/style editions
  2024; dependency-declared floor Rust 1.95, without a claimed 1.95 build.
- esp-hal **1.2.0**, esp32c6 PAC **0.24.0**, esp-bootloader-esp-idf **0.6.0**,
  esp-rtos **0.4.0**, embassy-executor **0.10.0**, embassy-time **0.5.1**,
  embassy-sync **0.8.0**. Sync is now an optional exact direct pin and was
  already a transitive dependency; the lockfile records the entire graph.
- Local bench-core `integer-mix-v1`, seed `0x12345678`, 4096 rounds, checksum
  `0xa3944d55`; 64 compute / 64 timer-overhead samples in both binaries.
- Embassy adds two capacity-one u32 channels, one main initiator and one
  independent responder task, eight warmup pairs and 1000 measured round trips.
  Both tasks share one cooperative executor/OS thread. No additional OS thread
  stack is configured for the responder. Its infinite receive/reply loop parks
  on another request after every reply, including the last timed sample;
  task teardown cannot distinguish only the final measurement.
- Fixed sample buffers: 2048 bytes bare metal / 18048 Embassy on this target;
  upstream static main-future task storage retains Embassy cross-await arrays.
- Release: opt-level 3, thin LTO, one codegen unit, debug=2, incremental=false,
  panic=abort. No `std`, `alloc`, build-std, vendor compiler, forced frame
  pointer or authored assembly. Upstream `-Tlinkall.x` and app descriptor remain.
- Requested CPU 160 MHz. Controlled image settings are DIO / 80 MHz / 8 MB.
  The hardware owner must preserve actual image/flashing commands. Runtime
  PCR+SPI0 snapshots establish observed configuration; image headers and boot
  logs alone are insufficient. The host enforces the expected clock baseline.

Register decoding was checked against the published esp32c6 0.24.0 PAC,
esp-hal 1.2.0 C6 clock implementation and ESP-IDF **5.5.6**, commit
`880a203ae603d08a94d06aa01e0d6761958e5e9e`, specifically its C6
`clk_tree_ll.h` and `spi_mem_reg.h`. ESP-IDF is a reference for register semantics,
not a HAL binary build dependency. No clock register is written by authored
telemetry code. Supported observed PLL settings are emitted without pretending
an unexpected clock equals the requested baseline.

## Final commands and results

Run from `firmware/esp-hal`:

| Command | Result |
| --- | --- |
| `cargo +1.96.0 fmt --package c6-runtime-bench -- --check` | Pass; final formatting was also applied before final builds. |
| `cargo +1.96.0 build --locked --release --bin bare-metal` | Pass, 1.32 s, no warnings. |
| `cargo +1.96.0 build --locked --release --features embassy --bin embassy` | Pass, 1.11 s, no warnings. |
| `cargo +1.96.0 clippy --locked --release --bin bare-metal` | Pass, 0.35 s, no warnings. |
| `cargo +1.96.0 clippy --locked --release --features embassy --bin embassy` | Pass, 0.28 s, no warnings. |
| `cargo +1.96.0 doc --locked --release --no-deps --features embassy --bin embassy` | Pass, 0.68 s, no warnings. |
| `cargo +1.96.0 doc --locked --release --no-deps --bin bare-metal` | Pass, 0.44 s, no warnings. |

The lockfile update was resolved by the initial build without `--locked`, adding
only the direct application edge to already-locked embassy-sync 0.8.0. All final
build/check commands use `--locked`. An initial Embassy scaffold called the old
spawn API and failed: executor 0.10 makes the task constructor return a fallible
spawn token, while `Spawner::spawn(token)` returns unit. The final code checks
the token and calls that current API; both final builds passed. Review also
changed the responder from a finite loop to an infinite receive/reply loop,
preventing final-reply task teardown from entering only the last timed interval.
Both binaries and all checks were rebuilt after that change. Earlier
intermediate artifacts are not frozen results.

Compiler identity is unchanged from the initial aligned comparison:

```text
rustc 1.96.0 (ac68faa20 2026-05-25)
commit-hash: ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96
host: aarch64-apple-darwin
LLVM version: 22.1.2
```

## Artifact freeze

SHA-256 was read after the final source change, builds, lints and docs. Firmware
source must not change between this freeze and captures without a fresh build,
hash and provenance record. Documentation changes do not alter these binaries.

| Input/artifact | SHA-256 |
| --- | --- |
| `target/riscv32imac-unknown-none-elf/release/bare-metal` | `be14bac3563b52d5e043485c9f798a4e23d6ed8702b72c9a3028c1cf46f58d0b` |
| `target/riscv32imac-unknown-none-elf/release/embassy` | `8d06354ca14e9a51550fd3183c9f29750f32fc73880140c6d9ca8151e0a18d44` |
| firmware `Cargo.lock` | `fe83fea7296603dd9a461ed872fcecab02b277754a18060b0ac0e8cbc48ed89f` |
| `src/clocks.rs` | `809acc669b5d8ca7081b5e361fa62f3f3979578633afa5cb78146650ef213f68` |
| `src/measurements.rs` | `e43ee6c0226cc815237c9fb780e5a7cd1ce8de0f043dbdcac61ddfe802edef6f` |

These ELF paths are within `firmware/esp-hal`. The shared workload source is
unchanged; preserve its own hash and repository revision with each physical run.
Both bin source hashes and the full tree revision should accompany the hardware
capture provenance rather than deriving identity from a filename.

## Static section inspection

Read-only inspection used Rust 1.96.0's `llvm-tools` component:

```text
llvm-size -A target/riscv32imac-unknown-none-elf/release/bare-metal target/riscv32imac-unknown-none-elf/release/embassy
```

| ELF section, bytes | Bare metal | Embassy |
| --- | ---: | ---: |
| `.trap` | 1600 | 1980 |
| `.rwtext` | 2100 | 4876 |
| `.data` | 4932 | 5672 |
| `.bss` | 648 | 19012 |
| `.flash.appdesc` | 256 | 256 |
| `.rodata` | 12152 | 14968 |
| `.text` | 37548 | 44736 |
| `.text_gap` | 53128 | 50312 |
| `.stack` | 442832 | 420572 |

`.stack` reserves remaining RAM and is not a measured high-water mark;
`.text_gap` is linker/MMU alignment reservation. These sections must not be
summed casually into a runtime RAM or flash-use score. Embassy `.bss` now
includes its static async state/sample storage. Debug sections are omitted.
No stack/heap high-water, idle-power, electrical frequency, radio, peripheral,
security, deadline or on-board scheduler claim follows from this build.

The prior upstream configuration remains: 8192-byte main-stack minimum,
stack-pointer range checks, hardware stack guard and HAL critical-section
implementation; esp-rtos default 100 Hz scheduler tick and WFI idle hook, with
automatic light sleep not enabled. Remaining-RAM stack reservation is distinct
from the separately fixed sample buffer bytes and generated async task storage.
No board, serial, flash or simulator operation was performed by this build
agent. The hardware owner's raw captures are the authority for on-board status.
