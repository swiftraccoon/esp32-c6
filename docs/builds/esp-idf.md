# ESP-IDF baseline build evidence

Status on 2026-10-22: release target build passed using Rust 1.96.0,
esp-idf-sys 0.37.2 and ESP-IDF 5.5.6. No on-device pass or performance result is established by this file.
The parent hardware operator owns flashing and run capture.

| Component | Selection |
| --- | --- |
| Rust / Cargo | rustc 1.96.0 (`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`), cargo 1.96.0 (`30a34c682`), Rust 2024 / formatting edition 2024 |
| Target | `riscv32imac-esp-espidf` |
| Rust sysroot | Cargo build-std std + panic_abort, scoped RUSTC_BOOTSTRAP=1 |
| ESP-IDF | v5.5.6 tag, commit `880a203ae603d08a94d06aa01e0d6761958e5e9e` |
| Bindings | esp-idf-sys 0.37.2, generated bindings; Cargo.lock commits transitive versions |
| Binding generator | bindgen 0.71.1, libc crate 0.2.191; existing upstream SDK/newlib input headers |
| Build integration | embuild 0.33.5; local ldproxy 0.3.4 |
| Rust optimization | opt-level 3, thin LTO, codegen-units 1, panic abort; debug information retained |
| SDK optimization | IDF performance option, GCC -O2; C/C++/assembly dependencies are upstream code |
| CPU / power | 160 MHz, dynamic power management off |
| Radio | No Wi-Fi/BLE/802.15.4 initialization in this application |
| UART | UART0 GPIO16/GPIO17 at 115200 baud; secondary console disabled |
| Flash | 8 MB, DIO, 40 MHz; default single factory app partition |
| Scheduler | IDF FreeRTOS, 100 Hz tick, main task CPU0, 16 KiB main stack |
| Timer | esp_timer_get_time, integer microseconds converted to ns (1,000 ns resolution) |
| Workload | integer-mix-v1; seed 0x12345678; 4096 rounds; two-second startup settle, 8 warmups; 64 compute + 64 overhead samples |
| FreeRTOS kernel | IDF fork, upstream header reports V10.5.1; exact SDK/submodule revisions retained |
| Host / tools | aarch64 macOS; SDK Python 3.14.8, CMake 3.30.2, Ninja 1.12.1, GCC 14.2.0 (esp-14.2.0_20260121), ESP Clang esp-19.1.2_20250312; host libclang from Homebrew LLVM 23.1.2 |

## Commands and retained evidence

Run from `firmware/esp-idf` using [the README](../../firmware/esp-idf/README.md).
Build logs are retained locally under `firmware/esp-idf/artifacts`; downloaded
SDK/tool trees and full build directories are ignored, not source inputs.

| Command / evidence | Result |
| --- | --- |
| `git ls-remote --tags https://github.com/espressif/esp-idf.git 'v5.5*'` | v5.5.6 tag dereferences to the commit above. |
| `cargo +1.99.1 install ldproxy --version 0.3.4 --locked --root .tools` | Passed; tool installed only in this workspace. Host tool compiler does not compile workload. `artifacts/install-ldproxy.log`. |
| `rustfmt +1.99.1 --edition 2024 src/main.rs build.rs` | Initial formatting passed; final formatting must use selected compiler. |
| Stable 1.99.1 with build-std (`RUSTC_BOOTSTRAP=1`) | Failed in upstream std `sys/fs/unix.rs:1896`: E0425, missing libc::AT_FDCWD for espidf. `artifacts/build-release.log`. No patched std/libc is used. |
| Selected `cargo +1.96.0 build --release` with the documented environment | Passed with sys 0.37.2, 50.98 s. `artifacts/build-1.96-sys0.37-release.log`. |
| Final `cargo +1.96.0 build --release --locked` after comment clarification | Passed, 2.87 s; `artifacts/build-final-locked.log`. No source/config/lock mutations follow this build. |
| `cargo +1.96.0 fmt --check` | Passed. |
| `cargo +1.96.0 clippy --release --locked --no-deps` with documented environment | Passed, no warnings; `artifacts/clippy-release.log`. |
| `cargo +1.96.0 doc --release --locked --no-deps` with documented environment | Passed, no warnings; `artifacts/doc-release.log`. |
| Flash / timer / UART / checksum / reset / performance checks | Not run by this implementation agent; hardware execution is separate evidence. |

Final ELF: `firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline`.
Its SHA-256 is
`793f3cf07d2e23b3623fbe6657d077d078c539f960fe2db97bea3dea055fe1a9`.
Earlier `c21775...` debug ELF was replaced after a comment-only clarification;
do not combine its hash with final source snapshots.

The matching SDK build output directory is
`firmware/esp-idf/target/riscv32imac-esp-espidf/release/build/esp-idf-sys-2dd3efe9f8c63c2b/out`.
Its `build/bootloader/bootloader.bin` has SHA-256
`863a64949052cd3af8812197034f22a8330c06d49aba6b2b17ad2cc1a2dde5f0`;
`build/partition_table/partition-table.bin` has SHA-256
`7f00b6c042a89b15b0cac534f82ed988caf29278ff5700b0c511eb1b5bb7c820`.
Other SDK output directories are stale failed attempts and must not supply
flashing artifacts. The partition table resides at 0x8000; the factory app is at
0x10000. Use a tool that consumes the actual bootloader/partition table and ELF,
then record its final flash image hash separately.

Generated configuration was copied to
[sdkconfig.esp32c6](../../firmware/esp-idf/artifacts/sdkconfig.esp32c6), SHA-256
`8a832482dd631eb325cf5f897bf5fa4420cb2e743284d82c55e9fd0c7725b061`.
It confirms DIO 40 MHz, 8 MB, CPU 160 MHz, FreeRTOS 100 Hz, GPIO16/17 UART0,
secondary console disabled, log level zero and dynamic power management off.
The [input/artifact hashes](../../firmware/esp-idf/artifacts/build-inputs.sha256),
[recursive SDK submodules](../../firmware/esp-idf/artifacts/idf-submodules.txt),
[SDK tool archive hashes](../../firmware/esp-idf/artifacts/sdk-tool-downloads.sha256),
and [Python package versions](../../firmware/esp-idf/artifacts/idf-python-packages.json)
retain additional provenance. The Rust lockfile pins Rust dependencies. SDK
checkout `git status --short` was empty after the build setup; no SDK source was
modified.

SDK compile-time date embedding remains enabled (`CONFIG_APP_COMPILE_TIME_DATE=y`,
reproducible-build mode disabled). Clean rebuilds can therefore have different
bytes. These hashes identify the exact evaluated snapshot; they do not claim
bit-for-bit rebuild determinism. Wi-Fi driver support is compiled in
(`CONFIG_ESP_WIFI_ENABLED=y`) but no application call initializes it; Bluetooth
is disabled. This first run does not exercise radio support.

## Compatibility findings

ESP-IDF's target integration required actual builds to identify a usable pair.
The selected 0.37.2 bindings and 5.5.6 SDK are unmodified dependencies. The newer
0.38.1 crate failed with E0609 in `src/stdio.rs`: generated `_reent` has only an
opaque `_address` field, while stdio restoration code accesses `_stdin`,
`_stdout` and `_stderr`. Both the full default SDK component build and an
exploratory trimmed build reproduced this. Explicit inclusion of the existing
newlib header through binding arguments did not resolve it; that argument was
removed. Logs are `artifacts/build-1.96-release-all.log` and
`artifacts/build-1.96-release-reent.log`.

The older binding release does not use that restoration glue. The console uses
the upstream configuration alternative `CONFIG_ESP_CONSOLE_SECONDARY_NONE=y`;
its actual UART behavior requires hardware validation. The
[0.38.1 upstream stdio explanation](https://github.com/esp-rs/esp-idf-sys/blob/v0.38.1/src/stdio.rs)
documents why the secondary-console configuration matters to descriptor routing.
No handwritten C shim, std patch, or vendor-source change was introduced.

Exploratory component trimming also showed that the SDK dummy link needs its
generated `main` component, and the binding headers unconditionally require the
legacy `driver/periph_ctrl.h` on IDF 5.x. These were setup failures, not OS runtime
failures; final configuration uses the default SDK component set. Compiling an
available driver does not mean the application initializes or benchmarks it.

This integration friction is evidence about the Rust tooling pair, not a
measured FreeRTOS performance result. The isolated build-std bootstrap facility
and chosen compiler remain part of the comparison's provenance.

## Primary sources and limits

The [`esp-idf-template` target table and build configuration](https://github.com/esp-rs/esp-idf-template)
identify the C6 triple and build-std/native SDK route. The
[`esp-idf-sys` build options](https://github.com/esp-rs/esp-idf-sys/blob/v0.37.2/BUILD-OPTIONS.md)
define native version pinning, `espidf_time64`, SDK configuration and automatic
tool download. The [Rust target documentation](https://doc.rust-lang.org/nightly/rustc/platform-support/esp-idf.html)
describes ESP-IDF std support, and the
[IDF timer documentation](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/esp_timer.html#obtaining-current-time)
defines the microsecond timer. These sources establish an integration route;
the actual build and device results establish what works in this repository.

This is a compute/observer baseline. It does not measure scheduler switches,
queue contention, radio support, peripheral correctness, sleep/power, OTA,
security features, or long-running stability. A common compiler and workload do
not eliminate SDK code-generation, cache, timer, memory-layout and interrupt
differences. Retain raw outliers and timer-overhead data; do not subtract a
quantized zero or turn this result into an OS ranking.
