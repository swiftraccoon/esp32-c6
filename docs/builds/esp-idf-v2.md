# ESP-IDF protocol 2 build record

Prepared 2026-10-22. This records a compiled Rust two-task handoff adapter with
runtime clock-register snapshots. Hardware operation and measured results belong
to the parent operator's captured trials; this build record does not establish
an on-board pass. Historical protocol 1 setup and compatibility findings remain
in [the original record](esp-idf.md).

| Component | Protocol 2 selection |
| --- | --- |
| Rust / Cargo | rustc 1.96.0 `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`; Cargo 1.96.0; Rust/formatting edition 2024 |
| Target / sysroot | `riscv32imac-esp-espidf`; build-std std + panic_abort; scoped RUSTC_BOOTSTRAP=1 |
| ESP-IDF | v5.5.6, commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`; unmodified SDK |
| Bindings / build integration | esp-idf-sys 0.37.2, bindgen 0.71.1, libc 0.2.191, embuild 0.33.5, local ldproxy 0.3.4; unchanged Cargo.lock |
| Optimization | Rust opt-level 3 / thin LTO / one codegen unit / panic abort / debug 2; IDF performance GCC -O2 |
| Configured clocks / flash | CPU 160 MHz; PM disabled; flash DIO 80 MHz, 8 MB; official SDK configuration |
| Runtime clock evidence | Four active PCR/SPI0 registers read before timed samples and after worker join; raw values emitted, host independently requires CPU 160 MHz / MSPI 80 MHz / flash 80 MHz / divider 1 |
| Scheduler | IDF FreeRTOS, 100 Hz tick; one initiating main task plus one actual Rust std/pthread responder task; existing SDK idle/timer tasks/interrupts remain active |
| Priority | SDK defaults main 1 / pthread worker 5; actual calling-task values emitted in hello |
| Stack and sample memory | Main 32 KiB / worker 8 KiB; three fixed sample arrays 18,048 bytes on C6; compiler main frame 18,400 bytes |
| Message workload | Two capacity-one `sync_channel::<u32>` queues; one in-flight request; reply = sequence XOR 0xa5a55a5a; 8 warmups + 1,000 timed roundtrips |
| Compute / observer | Same integer-mix-v1, seed 0x12345678, 4,096 rounds; 8 compute warmups; 64 compute + 64 timer-overhead samples |
| Timer | esp_timer_get_time; integer microseconds multiplied by 1,000 for nanosecond records; no observer subtraction |
| Protocol / output | Version 2; 1,128 samples; all JSON after timing and join; before/after clocks plus task/channel/storage metadata |
| Radio / transport | No radio initialization; UART0 GPIO16/17, 115200 baud; secondary console disabled |
| SDK tools / host | Same cached GCC 14.2.0 esp-14.2.0_20260121, ESP Clang 19.1.2, CMake 3.30.2, Ninja 1.12.1, Python 3.14.8 and aarch64 macOS as original record |

## Verification

Commands run from `firmware/esp-idf` with the README environment:
`RUSTC_BOOTSTRAP=1 LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib` and workspace-local
`.tools/bin` prepended to PATH. No source/dependency/config changes followed the
final release build; subsequent changes only document the same snapshot.

| Command / evidence | Result |
| --- | --- |
| `cargo +1.96.0 build --release --locked` | First protocol 2 build passed in 58.21 s, including regenerated official SDK config. Local log `artifacts/build-v2-release.log`. |
| Final `cargo +1.96.0 build --release --locked` | Passed in 4.95 s after stack-copy fix and diagnostic clock decoder; `artifacts/build-v2-final-locked.log`. |
| `cargo +1.96.0 fmt --check` | Passed with selected compiler. |
| `cargo +1.96.0 clippy --release --locked --no-deps` | Passed with no warnings in 2.42 s; `artifacts/clippy-v2-release.log`. |
| `cargo +1.96.0 doc --release --locked --no-deps` | Passed with no warnings in 1.90 s; `artifacts/doc-v2-release.log`. |
| SDK configuration inspection | Generated DIO 80 MHz / 8 MB / CPU 160 MHz / main 32,768 bytes / 100 Hz / worker priority 5 / PM disabled confirmed; retained `artifacts/sdkconfig-v2.esp32c6`. |
| ELF disassembly | Final main prologue reserves 256 + 0x46e0 = 18,400 bytes. Retained `artifacts/main-v2-prologue.txt`. |
| API/safety review | [All 54 active entries](../reviews/esp-idf-v2.md) assessed; source review includes channel ownership/cleanup, task context and read-only MMIO. |
| `git diff --check -- firmware/esp-idf docs/reviews/esp-idf-v2.md docs/builds/esp-idf-v2.md` | Passed. |
| Hardware / allocator tracing / radio / long-duration checks | Not performed by this implementation agent. Build and source inspection cannot establish these results. |

A pre-final release binary used 34,384 bytes in its main frame because by-value
sample-array iteration materialized a large iterator copy. This exceeded the
configured 32 KiB main stack and was caught before board operation. Borrowed
sample iteration reduced the final frame to 18,400 bytes. The remaining 14,368
bytes cover nested calls and other frames; this arithmetic is not a measured
stack high-water mark or a complete static worst-case stack proof.

No tests were added solely to restate implementation details. Shared workload
vectors and protocol/clock-decoder negative tests live in the corresponding
host crates. The UART, actual task execution, clock MMIO and checksum flow
require the operator's physical board trial.

## Exact artifacts

Final ELF:
`firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline`

SHA-256:
`2c29bbbc2c7fe9ebb433907d9a2121c121ad67c460d4f1ead99f50e552991cf9`

Matching SDK output directory:
`firmware/esp-idf/target/riscv32imac-esp-espidf/release/build/esp-idf-sys-2dd3efe9f8c63c2b/out`

| Artifact relative to that directory | SHA-256 |
| --- | --- |
| `build/bootloader/bootloader.bin` | `f296275389320ab2c9accdb44813442c8994724a18e90d646189950627ce562b` |
| `build/partition_table/partition-table.bin` | `7f00b6c042a89b15b0cac534f82ed988caf29278ff5700b0c511eb1b5bb7c820` |
| Generated sdkconfig, copied as `firmware/esp-idf/artifacts/sdkconfig-v2.esp32c6` | `5ee0be1128539831db16b63d7008985e16849b216bd4a278076e294a51fe3758` |

The partition table is at 0x8000 and factory app at 0x10000. Use this exact
bootloader/partition/ELF set. The protocol 1 bootloader selects a different flash
clock and must not be paired with this trial. The final flash-image hash belongs
to the hardware operator's provenance.

[Source/config/artifact hashes](../../firmware/esp-idf/artifacts/build-v2-inputs.sha256)
include main.rs, SDK defaults, Cargo manifest/lock, toolchain/config/build script,
the shared workload and final binary set. Original SDK submodule/tool/Python
provenance remains applicable; no new dependency download or SDK source patch
was introduced. SDK compile-time date embedding remains enabled, so hashes
identify this exact snapshot rather than asserting bit-for-bit clean rebuilds.

## Sources and interpretation

The selected
[IDF clock-tree implementation](https://github.com/espressif/esp-idf/blob/v5.5.6/components/hal/esp32c6/include/hal/clk_tree_ll.h),
[PCR register definitions](https://github.com/espressif/esp-idf/blob/v5.5.6/components/soc/esp32c6/register/soc/pcr_reg.h)
and
[SPI-memory register definitions](https://github.com/espressif/esp-idf/blob/v5.5.6/components/soc/esp32c6/register/soc/spi_mem_reg.h)
define the readback decode. ESP32-C6 PLL source 1 feeds CPU via root/CPU dividers,
and MSPI via its high-speed divider. SPI0 bit 31 bypasses its downstream divider;
otherwise N+1 applies. Unsupported sources/dividers or fractional-MHz flash
values fail before a complete record. Supported off-baseline values are emitted
for diagnosis; the host rejects them. No timing register is written by the app.

The selected
[Rust thread implementation](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sys/thread/unix.rs)
uses pthread creation/join and explicit stack attributes for ESP-IDF.
[IDF pthread documentation](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/pthread.html)
explains its FreeRTOS-backed task integration and priority behavior. A channel
roundtrip through these wrappers has a different software path from native
FreeRTOS queue APIs and an Embassy cooperative channel; interpret its measured
latency as the full chosen Rust application path.

Rust 1.96's [bounded channel](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sync/mpmc/array.rs),
[cached context](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sync/mpmc/context.rs)
and [waiter storage](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sync/mpmc/waker.rs)
were inspected locally at the selected compiler source. They support the
narrow expectation that this primed, single-request pattern reuses allocated
storage. This is not an on-device allocator trace. Preserve outliers and
microsecond quantization; do not rank kernels from this one workload.
