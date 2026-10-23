# First ESP32 C6 physical benchmark results

All **15 measured warm-reset runs passed**, five each for bare metal, Embassy on esp-rtos, and ESP-IDF/FreeRTOS. Every run produced 64 correct compute samples and 64 timer-observer samples: **960 compute measurements and 960 calibration measurements** in total. The original full-flash image was restored and verified after testing.

The three stacks execute this small CPU workload at essentially the same speed. These results establish a functioning, comparable baseline; they do not establish which OS has better scheduling, networking, power use, or product suitability.

## Measured results

Times cover one call containing 4096 rounds of `integer-mix-v1`. The table reports the range of the five within-boot medians, without subtracting timer overhead.

| Stack | Passing boots | Per boot median in microseconds | Application image bytes | Merged flash image bytes |
| --- | --- | --- | --- | --- |
| Bare metal with esp-hal | 5 | 283 | 102112 | 167648 |
| Embassy on esp-rtos | 5 | 282 to 283 | 107504 | 173040 |
| Rust std on ESP-IDF FreeRTOS | 5 | 283 | 363952 | 429488 |

Application images start at `0x10000` in these merged binaries. Merged sizes include their bootloader, partition metadata and offset padding. These are sizes of the configured applications, not minimum kernel sizes. The FreeRTOS row also includes Rust `std` and the selected SDK integration; the difference cannot be attributed solely to the kernel.

Every boot's compute p95 was 283 microseconds. The maximum observations were 291, 293 and 289 microseconds respectively. With 64 samples, nearest-rank p99 equals the maximum. These maxima are not worst-case execution-time bounds or scheduler-latency measurements.

For audit purposes, the means across the 320 compute samples per stack were 282.675, 282.66875 and 283.0375 microseconds respectively. Those fractional averages do not improve the timer's 1 microsecond resolution. Samples within a boot are correlated; five independent measured boots per stack are too few for a broad reliability or tail-latency claim.

The timer-overhead median was 1 microsecond in every boot. Individual observations included zero and initial outliers. Zero reflects quantization, not a zero-cost clock read. Calibration remains separate from computation.

## Test conditions

- Physical ESP32-C6 QFN40 revision 0.2, 8 MB flash, 40 MHz crystal; carrier model unknown.
- CPU configured at 160 MHz; DIO images request 40 MHz flash. HAL boot logs report 80 MHz, while the IDF SDK is configured for 40 MHz; actual runtime flash clocks were not read back. Flash frequency is therefore an unverified control, not an established common 40 MHz setting. Radio stacks were not initialized, and no intentional serial output occurred during measured regions.
- Rust 1.96.0 and LLVM 22.1.2, edition 2024, optimization level 3, thin LTO, one codegen unit, aborting panics.
- The same shared Rust source, seed `0x12345678`, checksum `0xa3944d55`, eight warmups, and 64 timed calls per boot.
- Two-second startup settling period before timing; internal timers converted from microseconds to nanoseconds.
- UART0 on GPIO16/17, CP2102N bridge, 115200 baud. The host requested warm reset using RTS with DTR deasserted.
- Platform order alternated across the capture sequence; the exact order and boot IDs are in every `run.json`. Captures occurred on 2026-10-22 in America/New_York, corresponding to 2026-10-23 UTC.

Independent binary inspection confirmed the same **54-byte, 18-instruction workload body** in all three final ELFs, SHA-256 `b917090559ff1edf22ad5ac21975f1a998e5594b9ad23db9344d5c71c2b3af49`. Timer paths, layout, caches, background interrupts and runtime services still differ. See the [cross-platform review](../../docs/reviews/cross-platform.md).

The [experiment errata](errata.json) qualifies the captured `configuration.flash_mhz` provenance field: it records the requested image setting, not a measured runtime clock. Original captures and their hashes remain intact. The flash-clock discrepancy must be resolved before a controlled performance ranking.

## Artifacts and checks

[Runs](runs/) retain raw serial bytes, parsed records, complete ELF copies, firmware/collector hashes, build provenance and replayed summaries. [Artifacts](artifacts/) retain source snapshots, exact images, section inventories, IDF bootloader/partition data and configuration. Each source snapshot and dependency lockfile is hashed in the platform provenance.

All 15 raw captures were replayed through the validator and matched the recorded successful results. The [independent results audit](../../docs/reviews/initial-results.md) recomputed counts, checksums, statistics and hashes. The host/workload suite passed 13 unit tests and two doctests, formatting, Clippy and documentation checks. Both `no_std` firmware binaries and the IDF firmware passed their recorded release builds, lint, format and documentation checks.

The IDF baseline uses unmodified `esp-idf-sys` 0.37.2 with ESP-IDF 5.5.6. Rust 1.99.1 and newer binding integration exposed build failures; the [IDF build record](../../docs/builds/esp-idf.md) retains the diagnostics and chosen compatible versions. The SDK embeds build date/time, so rebuilding from source is not promised to reproduce identical ELF bytes. These repeated runs all used the same archived binary for their platform.

## Memory interpretation

Section inventories are archived; **runtime stack and heap high-water usage were not measured**. HAL's `.stack` reserves most unused RAM, `.text_gap` reserves mapping space, and IDF's `.flash_rodata_dummy` overlaps flash mapping rather than consuming ordinary RAM. Summing all allocated or uninitialized sections would produce a misleading RAM comparison. Static RAM-resident code/data and these reservations are distinguished in the independent review.

## Backup and restoration

Before any benchmark flash, esptool read all 8388608 original bytes and verified the backup against the device. After the final run, the full image was written back, verified again, and the board was reset into the original application. No eFuse provisioning was performed.

The original image and transfer logs remain in the Git-ignored `backups/original-2026-10-22/` directory. Its SHA-256 is `13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`. Higher-speed transfer attempts failed; 115200 baud completed the backup, all benchmark flashes, and restoration.

## What to test next

First read back runtime flash sources/dividers and align supported settings. Then move to task/message handoff, priority interference, timer wake jitter, stack/heap peaks and longer stress runs. Add Ariel OS and RTIC, then qualify the Zephyr Rust build. Network and power experiments need their own configurations and peers/instruments from the [hardware coverage plan](../../docs/hardware-coverage.md). Keep custom-OS work tied to a concrete deficit revealed by these tests.

To replay one captured boot without touching hardware:

```sh
cargo build --locked --release -p benchctl
target/release/benchctl validate \
  --input results/initial-2026-10-22/runs/01-bare-metal/serial.raw \
  --platform bare-metal
```
