# ESP32 C6 benchmark results

This checkout includes source and textual experiment records. Compiled images
and host executables are not included. Historical hashes and frozen manifests
identify the complete experiment bundles; the separate snapshot inventory
identifies current checkout files. See the
[verification scope](../docs/reviews/public-snapshot.md) for retained raw replay
and checks requiring the complete bundles.

The [RTIC monotonic qualification](rtic-qualification-2026-10-23/RESULTS.md)
retains **one failed attempt and two not run**. Seven expected markers precede
a second ROM startup within the observation window. The missing second reset
reason leaves cause unconfirmed; successful marker/pacing subsets do not
qualify the attempt. Builds, original/corrected packaging, frozen inputs,
raw/receipt failure, independent review and full restoration are retained.

The [finite-load qualification](idf-load-qualification-2026-10-23/RESULTS.md)
contains two scheduled physical passes, 4,032 nominal release slots and
before/after task/heap/stack indicators. Queue-full losses and synthetic deadline
misses at the highest offered rate remain explicit. These are finite pulses,
without a sustainable-capacity or instrumentation-effect estimate. A subsequent
[host reader correction](../docs/reviews/idf-load-replay-correction.md) preserves
exact saved floating-point values; original captures, reports and frozen
collector remain unchanged. Full flash is restored and separately verified.

The [IDF accounting qualification](idf-accounting-qualification-2026-10-23/RESULTS.md)
contains three scheduled warm-reset diagnostic passes and eighteen snapshots,
under a committed input freeze. It qualifies scoped task-attribution/allocation
controls, with no load-capacity, global-memory-peak or CPU-efficiency result.
Exact inputs, raw captures, rejected packaging candidate and restoration logs
are retained separately from the historical latency studies.

The [Zephyr native collection](zephyr-native-tests-2026-10-23/RESULTS.md)
retains **three failed v1 qualification attempts**. All three contain 47 rows
and an upstream success banner; all lack the compiled-out required frequency
header and report a ROM image-SHA warning. Corrected per-line replay preserves
the original parser reports and failed admission. The separate
[Rust probe](zephyr-rust-build-2026-10-23/README.md) retains three failed build
stages, with no Rust firmware or board result. The current pre-Zephyr full
image was restored, separately digest-verified and reset.

The [Ariel native tests](ariel-native-tests-2026-10-23/RESULTS.md) retain the
expected outputs in **12 scheduled boots**, three per test. The operator's
twelve pass reports remain unchanged; the first five-second guard is
operator-attested without an archived wait receipt. Yield prints a floored
batch mean of **155 ticks**, flags **215**. Derived **77** is arithmetic, with
no peer-progress counter validating switch count or latency. Priority/lock
assertion markers support their specific control flows. This is partial
qualification evidence, with no retained per-event distribution or matched RTT
comparison; see the [corrected review](../docs/reviews/native-suite-qualification.md).

The [matched IDF queue API study](idf-queue-api-2026-10-23/RESULTS.md) contains
24 passing scheduled boots, twelve std/native pairs across three labeled
sessions, and 24,000 checked round trips plus 24,000 observer observations.
Per-boot medians are **1,006 ticks / 62.875 µs** for std channels and
**319 ticks / 19.9375 µs** for native FreeRTOS queues, using the nominal
16 MHz counter scale on the same kernel and Rust thread arrangement. All twelve
paired native-minus-std median differences are −687 ticks / −42.9375 µs.
The [independent audit](../docs/reviews/queue-api-results.md) verifies the raw
records, frozen inputs and limitations. Original firmware was restored,
digest-verified and reset after collection. This dataset remains separate from
the earlier whole-microsecond studies below.

The [first physical benchmark](initial-2026-10-22/README.md) contains 15 passing warm-reset captures across bare metal, Embassy/esp-rtos and ESP-IDF/FreeRTOS, with raw data, exact artifacts and an independent review. The board's original firmware was restored and verified afterward.

The [clock verification and message experiment](clocked-handoff-2026-10-23/README.md)
contains 15 more passing boots, verified runtime clock configuration and 10,000
checked task/thread round trips. The separate
[alignment probe](alignment-probe-2026-10-23/README.md) contains five alternating
build captures. [Ariel qualification](ariel-qualification-2026-10-22/README.md)
contains two unmodified startup/UART passes, with different toolchain inputs and
no performance comparison. Restoration for those three datasets is recorded in
the message experiment's `device-state.json`; the later matched API study
preserves its own restoration record.

`run-template.json` remains an unmeasured planning template, not a sample result. Leave unknown values as `null`; do not fill them with zero. The collector records validated benchmark sets within each boot's `run.json`, whose scope and fields are described in [benchctl](../tools/benchctl/README.md).

For each measured run, copy the template into a directory such as `results/<platform>/<revision>/<run-id>/` and store raw samples, complete configuration, toolchain details, firmware hashes, and the analysis output together. Record units explicitly. Exclude Wi-Fi credentials, MAC addresses when unnecessary, and full-flash backups from Git.

Follow the [benchmark protocol](../docs/benchmark-plan.md). Document unsupported features separately from failed or unperformed tests. A valid report needs correctness results, sample counts, raw-data references, and enough metadata for another run on the same board.

The [feedback audit](../docs/reviews/2026-10-23-feedback-audit.md) records evidence
corrections. Elapsed-path latency alone does not measure CPU/ISR utilization,
sustainable capacity, memory peaks, IRQ response, hardware offload, radios,
sleep/energy or update effort; see the [utilization audit](../docs/research/utilization-audit.md).
