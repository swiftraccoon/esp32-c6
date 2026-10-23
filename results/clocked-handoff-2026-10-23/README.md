# Clock verification and Rust message handoff results

All **15 measured warm-reset boots passed**, five per platform. They produced
**960 compute samples, 960 timer-observer samples and 10,000 checked message
round trips**. Every boot reported the same CPU and flash configuration before
and after measurement: **160 MHz CPU, 80 MHz MSPI source, 80 MHz SPI0 flash**.

The selected Embassy request/reply path has a 7 microsecond median; Rust
`std::sync::mpsc::sync_channel` on IDF/FreeRTOS has a 62 microsecond median.
These compare the complete selected Rust application paths. They do not isolate
kernel context-switch cost or establish an overall OS winner.

## Measured message paths

Each boot records 1000 sequential exchanges between two independent tasks, using
one capacity-one request queue and one capacity-one reply queue. The request is
a `u32` sequence number; the response is that number XOR `0xa5a55a5a`. Eight
complete warmup exchanges precede measurement. Timing begins before sending the
request and ends after receiving its reply. Every response is checked.

| Stack and primitive | Passing boots | Per-boot median, µs | Per-boot p95, µs | Per-boot p99, µs | Largest observation, µs |
| --- | ---: | ---: | ---: | ---: | ---: |
| Embassy / `embassy-sync` channel | 5 | 7 | 8 | 8 | 20 |
| IDF FreeRTOS / Rust `std` bounded channel | 5 | 62 | 63 | 63 | 69 |
| Bare metal | 5 compute-control boots | Not implemented | Not implemented | Not implemented | Not implemented |

Embassy runs its independent initiator and responder tasks in one cooperative
executor/OS thread. IDF runs the main task at observed FreeRTOS priority **1**
and a separate Rust pthread responder at observed priority **5**, with a 100 Hz
tick. Queue implementation, wakeups, executor dispatch, OS task scheduling,
wrappers, priorities and interrupt interference contribute to the interval.
The IDF result is not a native FreeRTOS queue benchmark. Dividing either result
by two does not establish one-way latency.

The Embassy responder parks waiting for its next request after every reply,
including the last; task destruction is excluded from timing. IDF channel
closure and worker join occur after measurement. No intentional application
logging occurs within any timed region. Arrays retain all samples until output;
both scheduler adapters report 18,048 sample-buffer bytes, which is not total
runtime RAM. IDF requests a 32 KiB main stack and 8 KiB worker stack. The linked
main frame is 18,400 bytes; stack high-water use was not measured.

## Compute control and placement finding

| Stack | Compute samples | Per-boot median, µs | Largest observation, µs | Application image bytes | Merged image bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Bare metal | 320 | 308 | 319 | 103152 | 168688 |
| Embassy / esp-rtos | 320 | 283 | 294 | 110352 | 175888 |
| Rust std / IDF FreeRTOS | 320 | 283 | 290 | 394928 | 460464 |

Each compute call contains 4096 rounds of the unchanged `integer-mix-v1`.
Independent inspection confirms identical **54-byte, 18-instruction** workload
bodies in all three ELFs, also identical to the earlier pilot. However, new
bare metal places its five full-width loop instructions at addresses modulo
four equal to two; the other workloads place them on four-byte boundaries.
Matching bytes alone did not control instruction placement.

The 308 microsecond result remains in this dataset. The separate
[alignment probe](../alignment-probe-2026-10-23/README.md) tests a compiler
function-alignment setting with unchanged source and workload bytes. Read that
result before interpreting the compute difference as runtime overhead. The
[binary review](../../docs/reviews/clocked-workload-binaries.md) records exact
addresses, hashes and the limits of a hardware fetch-penalty explanation.

Timer-observer medians are 1 microsecond in every boot. Zero observer values
reflect quantization. Times retain their 1 microsecond resolution even when
means have fractional values. No overhead subtraction or outlier removal is
performed. Percentiles are nearest rank within each boot; 64-sample compute p99
is the maximum. A 1000-sample message p99 is still a within-boot observation,
not 1000 independent boots or a worst-case bound. Image sizes include the
configured applications; merged sizes also include bootloader, partitions and
padding. Neither is a minimum kernel size or a RAM high-water measurement.

## Verified clock control

All 15 initial/final snapshots contain these same values:

| Register | Raw value | Relevant decode |
| --- | --- | --- |
| `PCR_SYSCLK_CONF` | `0x28010200` | SPLL selected; 480 MHz divided by three to the HP root |
| `PCR_CPU_FREQ_CONF` | `0x00000000` | CPU divider one; 120 MHz force mode off |
| `PCR_MSPI_CLK_CONF` | `0x00000500` | SPLL divided by six, MSPI source 80 MHz |
| `SPI0_CLOCK` | `0x80000000` | Downstream divider bypassed, flash 80 MHz |

Both firmware adapters report raw registers and decoded frequencies. The host
independently decodes the fields and requires exact before/after equality.
This resolves the unverified clock control from the
[first experiment](../initial-2026-10-22/README.md); its original captures and
errata remain intact. Readback establishes configuration assuming the documented
SPLL source. It is not an electrical frequency measurement or proof that no
transient change occurred between snapshots. Dynamic power management is off.

## Inputs, evidence and limits

The same physical C6 revision 0.2 / 8 MB / 40 MHz crystal is used throughout;
carrier model remains unknown. UART0 GPIO16/17 connects through CP2102N at
115200 baud. The collector requests a warm reset for each capture. Platform
order varies, with exact sequence and boot IDs in [runs](runs/); when adjacent
boots use the same platform, the already verified image is reset without an
unnecessary intervening flash. Captures occurred on 2026-10-23 America/New_York.
Some transcripts repeat exactly at this timer resolution. The protocol has no
device boot nonce; separate warm-reset requests and host capture records, rather
than cryptographic/device-generated boot identities, identify these repetitions.

Rust 1.96.0 / LLVM 22.1.2, opt-level 3, thin LTO, one codegen unit and aborting
panics are common controls. HAL uses esp-hal 1.2.0; Embassy uses esp-rtos 0.4.0,
executor 0.10.0 and sync 0.8.0. IDF uses unmodified SDK 5.5.6 with sys 0.37.2 and
the selected Rust std integration. Each boot settles for two seconds, performs
eight compute warmups and, where applicable, eight message warmups. Radios are
not initialized. Networking, peripherals, IRQ response, sleep/power, security,
stack/heap peaks and long-duration reliability are unmeasured here.

The [experiment contract](../../docs/experiments/clocked-handoff.md),
[HAL build](../../docs/builds/esp-hal-v2.md),
[IDF build](../../docs/builds/esp-idf-v2.md),
[independent implementation review](../../docs/reviews/clocked-handoff-implementation.md)
and [results audit](../../docs/reviews/clocked-handoff-results.md) provide details.
[Artifacts](artifacts/) retain source archives, section inventories,
configuration and per-platform provenance identifying historical source commit
`8f4294b`. [summary.json](summary.json) retains per-boot statistics.

Original serial bytes, parsed records, collector hashes and replay outputs are
retained. Compiled artifacts are outside this checkout; their collection-time
identities remain recorded. Protocol transcripts are unchanged.
[SHA256SUMS](SHA256SUMS) records the collection-time archive identities, not a
complete verification of the current checkout. See the
[snapshot review](../../docs/reviews/public-snapshot.md) for available artifacts
and current verification commands.

## Original firmware

The full original 8 MiB backup was verified against the device before any Ariel
qualification or new benchmark flash. Its SHA-256 is
`13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`.
After the main experiment, both Ariel startup tests and the separate alignment
probe, the full 8,388,608 original bytes were written back. A separate device
digest verification passed, then esptool hard-reset the board into the original
application. [device-state.json](device-state.json),
[write log](artifacts/restore.log) and
[verification log](artifacts/restore-verify.log) retain the evidence. No eFuse
provisioning was performed. The backup remains Git-ignored.
