# Qualification of Zephyr's existing native latency suite

**Review correction, interpretation revision 2 (2026-10-23):** the frozen v1
header requirement below is impossible for the selected verbose-off image.
All three captured attempts remain failed under that original contract;
completion banners do not waive it. The timing descriptions below are corrected
to the actual 19/28 row policy. See the [source audit](../research/zephyr-native-audit.md)
and [failed results](../../results/zephyr-native-tests-2026-10-23/RESULTS.md).
No v2 profile has been built or captured; any revised admission needs a new
prospective freeze. Historical inputs remain preserved unchanged.

Design recorded **2026-10-23 before Zephyr test flashing**. Use the unmodified
default suite from Zephyr release **4.4.0**, commit
[`684c9e8f32e4373a21098559f748f06915f950c9`](https://github.com/zephyrproject-rtos/zephyr/tree/684c9e8f32e4373a21098559f748f06915f950c9),
with the exact tool/module/image tuple in the
[build record](../builds/zephyr-native-tests.md). A successful build does not
establish a physical pass.

## Decision and reuse

Can the maintained C6 port boot and complete its existing native kernel latency
suite through our known UART connection? This closes an integration gate before
authoring a Zephyr-specific benchmark or Rust application adapter. It also
provides named native-operation references for future experimental contracts.
It does not compare the complete capabilities or performance of operating
systems and does not establish C6 Rust support.

The [prior-work review](../../../../docs/research/prior-work.md) and
[reuse inventory](../../../../docs/research/benchmark-reuse.md) identify the existing suite
before implementation. Run the upstream code unchanged under its
**Apache-2.0** notices, preserving licenses with source evidence. No C, C++ or
assembly adapter is authored. Radio blobs, SDK components and other dependencies
retain their own terms; this is not a full product licensing audit.

The selected source is
[`tests/benchmarks/latency_measure`](https://github.com/zephyrproject-rtos/zephyr/tree/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure).
Its console harness already names yield, ISR return/reschedule, lifecycle,
FIFO/LIFO, event, semaphore, condition variable, stack, mutex and heap cases.
Preserving those distinctions is more useful than inventing another native
suite. Its default configuration exercises kernel threads only; userspace and
object-statistics variants are separate experiments.

## Fixed physical budget and admission

Schedule **three boots** of the same frozen image, without automatic retries
or replacement. Root alone owns flashing and UART access. Each attempt uses a
fixed **120-second host capture ceiling**, including connection/reset, and
requires at least **two seconds of observation after the success marker**.
Capture the complete rendered UART monitor output; stop through the host
timeout rather than claiming a firmware process exit code. Missing output,
connection failure, timeout before completion, unexpected reset and failed
metric rows remain visible failed attempts.

Before its first Zephyr write, root commits the exact input archive/manifest,
schedule, image/ELF hashes and this contract. Preserve all attempts under the
separate `results/zephyr-native-tests-2026-10-23` collection. Do not change
historical Ariel or IDF measurements. Record the host's capture start, first
observed completion and end, transport/stop method, image and flash verification.

For each accepted boot require:

1. One fresh C6 ROM startup, followed by the Zephyr boot/test output and one
   `Timing results: Clock frequency: 16 MHz` header.
2. All **47 expected default metric names**, each exactly once and in the
   preserved upstream order, with numeric `cycles` and `ns` columns and **empty
   note columns**. Preserve the notes and exact printed values even on failure.
   Zero is not silently changed to a missing result; reject an explicitly failed
   or nonnumeric row and any nonempty note.
3. One final **`PROJECT EXECUTION SUCCESSFUL`** after the complete metric set,
   and no `PROJECT EXECUTION FAILED`, `FAILED` metric, `Error: Tick Occurred`,
   panic, assertion/fatal error, heap-allocation error note or additional ROM
   startup in the observation window. In particular, reject
   `alloc memory @ iteration` and `Memory heap too small` notes.
4. At least two seconds of host observation after the first completed-output
   marker. Late completion without that interval does not pass this contract.

The handoff's `expected-metrics.txt` enumerates the compiled default names.
Require all rows rather than treating the terminal banner alone as sufficient.
The upstream harness accepts that banner, but this local qualification also
checks row coverage, numeric output, startup history and the observation window.
No pass establishes arbitrary runtime progress during quiet output.

The row grammar is `<metric> - <description>: N cycles , M ns : <notes>`,
with upstream padding whitespace and unsigned integer columns. A suitable
inspection expression after removing CR framing is
`^([a-z0-9_.+]+)\s+-\s+(.+?)\s*:\s*(\d+)\s+cycles\s*,\s*(\d+)\s+ns\s*:\s*(.*?)\s*$`.
Apply this expression with a full match to **each separate line**. A global
multiline application lets whitespace consume neighboring rows and is incorrect.
Require the captured names to equal the 47-line expected list exactly, and the
captured notes to be empty after whitespace trimming. Preserve both integer
columns without requiring an exact ratio between their rounded means.

These unmodified programs have no device nonce or host challenge, no checked
sequence-number payload and no individual-event protocol. Transfer logs, reset
output, artifact identities and host chronology support this bounded collection
history without cryptographic execution attestation. Preserve a reliable stdout
mirror if the monitor's auxiliary script file is empty after host termination;
record which file contains the unchanged rendered output and retain both.

## Units, endpoints and configured behavior

The default is **1,000 iterations** per ordinary native-operation batch. Heap
uses **100** allocations/frees of **10 bytes**. The yield case timestamps in the
initiator before `k_yield` and in the peer after switching; it is not a complete
request/reply. Other rows distinguish operations without waiters from blocking
and wake-plus-context paths. Preserve each printed name and its upstream source
endpoint; do not combine different rows into an overall score.

The C6 timing path uses SYSTIMER **counter 1** at nominal **16 MHz**. The console
calls these units “cycles”; they are **not 160 MHz CPU cycles**. The converted
nanoseconds are derived from the configured timer frequency. Integer average
ticks and nanoseconds can have different rounding because the conversion can
precede division. Neither column proves calibrated absolute time.

Exactly **19 of the 47 default rows** subtract an observer estimate; **28 do
not**. Of those 28, eight bracket a whole loop and twenty accumulate individual
spans. The [per-row source map](../research/zephyr-native-audit.md) identifies
endpoints, aggregation and correction for every row. The observer minimum over
ten batches includes loop and outer-read costs, so it is not a structurally
matched two-read control for every operation. Over-subtraction is possible;
its actual bias is not measured here. The suite prints neither the observer
value nor gross totals or per-event distributions. Do not apply another
subtraction or rank these mixed outputs against gross IDF or Ariel results.

Actual source/configuration choices include:

- CPU requested **160 MHz**, kernel tick **1 Hz**, no timeslicing, PM off,
  userspace off and hardware stack protection off.
- A separate test thread at preemptive priority **10**. Main priority **0**
  only joins it. Yield helpers run at preemptive **8** or cooperative **−11**;
  there is no need to change main priority to correct these native classes.
- `CONFIG_FORCE_NO_ASSERT=y` makes `__ASSERT` inactive even though
  `CONFIG_ASSERT=y` also appears. Do not label the final image assertion-enabled
  based on the latter value alone.
- Upstream **`-Os`**, GCC **14.3.0**, ordinary linker placement, no LTO flag,
  Simple Boot, and an effective **4,096-byte** system heap raised from the
  suite's 2,048-byte request by the board minimum.

The terminal report uses `error_count`; inspected source increments it for
heap-allocation failures rather than exhaustively validating every native
operation or payload. Some row-level conditions are reported independently.
Therefore an accepted run establishes completion and the declared console
checks, not full scheduler correctness, all-operation correctness or a stress
qualification. Protection, instrumentation and clock choices remain experimental
factors, not free performance improvements.

## Hardware, restoration and scope

Use the confirmed C6 rev 0.2 / 8 MB / 40 MHz crystal and CP2102N UART0 route
at TX **GPIO16**, RX **GPIO17**, **115200 8N1**. The selected builder does not
identify the unknown carrier. No external pins, LED, radio peer, scope, native
USB cable or irreversible provisioning is needed for this UART-only stage.

Flash the generated **Simple Boot `zephyr.bin` at offset 0**. Its image/runner
request **8 MB / DIO / 80 MHz**. The converter aliases the C6's 40/80 MHz header
encoding, so a decoded header label does not independently distinguish them.
The collected ROM also reports a SHA comparison failure for this layout;
resolve that separate integrity gate before further qualification. Do not wrap its ELF in
espflash's ESP-IDF image/bootloader generator. Runtime flash/CPU/timer register
snapshots and independent oscillator checks are not supplied by this suite;
the header is image evidence, not matched final runtime clock proof.

Verify and preserve the **current pre-session full-image state** before any
write. The preceding session found NVS-only changes after an original-firmware
boot, so do not assume a historical backup's digest still matches. If needed,
preserve a fresh prefix, reconstruct the current full image, and independently
verify its complete device digest before testing. Keep private flash contents
out of Git; retain the verification records with the collection. Restore that
current full image, separately digest-verify it, then hard-reset after collection.

Retain ELF/bin, resolved Kconfig/devicetree, map, compiler and Python inventory,
module/source revisions, build log, source/license evidence, image information,
47-row list, transfer output and each console/report. The
[rev0.2 review](../../../../docs/research/rev02-errata-workarounds.md) remains relevant:
passing this suite does not qualify arbitrary misaligned copies, slow-clock
behavior, radio/sleep timer reliability or every erratum workaround in this
Zephyr/HAL tuple.

This qualification establishes no WCET bound, per-event tail, independent
frequency calibration, C6 Rust viability, radio/driver coverage, recovery path,
power result, product readiness or overall OS winner. A later common Rust
workload needs its own explicitly matched contract and source/API review.
