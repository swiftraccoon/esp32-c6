# Rust scheduling prior work and measurement audit

Reviewed 2026-10-23. This is a focused primary-source review of scheduling and
communication experiments relevant to the project's existing C6 measurements.
It is a bounded search, not a systematic literature review or proof of novelty.
No firmware was built or run for this review. Links below distinguish inspected
papers, inspected source/data, and leads that remain unverified.

## Verified papers

### Vandervelden et al., Sensors, 2024

[Overview of Embedded Rust Operating Systems and Frameworks](https://www.mdpi.com/1424-8220/24/17/5818),
Thibaut Vandervelden, Ruben De Smet, Diana Deac, Kris Steenhaut and An Braeken;
published 7 September 2024, DOI `10.3390/s24175818`. Full published text was
inspected through the [authors' university PDF](https://cris.vub.be/ws/portalfiles/portal/114593743/sensors_24_05818.pdf);
the [RustIEC project report](https://www.rustiec.be/en/reports/m16-os-comparison.html)
also describes the study.

Section 5 evaluates Tock, RTIC and Embassy on nRF52840-DK, Cortex-M4 at 64 MHz.
Hubris is surveyed but excluded from these measurements. All examples use
Rust `1.73.0-nightly-2023-07-30`. External GPIO instrumentation separates input
edge to ISR entry, ISR duration, and ISR exit to task execution; both speed and
size optimization and additional-task configurations are examined. Static
memory and binary sizes are also reported. Data are offered on reasonable
request; this review did not identify a paper-linked public benchmark source
revision. The results support the measurement decomposition, not a C6 latency
prediction. [Full text, sections 5 and data availability](https://cris.vub.be/ws/portalfiles/portal/114593743/sensors_24_05818.pdf)

### Wang, IET Computers & Digital Techniques, 2026

[Concurrency Frameworks for Embedded Systems: A Comparative Evaluation of FreeRTOS, RTIC, and Embassy](https://ietresearch.onlinelibrary.wiley.com/doi/10.1049/cdt2/7881841),
Guoping Wang; first published 12 August 2026, article 7881841, DOI
`10.1049/cdt2/7881841`. Full publisher HTML was accessible and inspected.

The paper compares STM32F3Discovery implementations using Blinky, UART–LED and
a UART/CAN/PWM Crane LPV application. It describes DWT-cycle measurements,
selected external GPIO validation, 1000 measured samples, distributions and
binary memory analysis. Its protocol specifies 72 MHz unless otherwise stated;
FreeRTOS is C/Keil and the Rust frameworks use LLVM. The threats section
recognizes compiler and primitive differences and limited stress coverage.
Its data statement says request-only data, while references 36–39 identify
firmware repositories. Source and some logs are publicly readable, as audited
below; availability does not establish that every published table can be
reconstructed. These are different observable paths from this project's
round-trip experiment. [Full text, sections 6, 9, data availability and references](https://ietresearch.onlinelibrary.wiley.com/doi/10.1049/cdt2/7881841)

### Frank et al., DCOSS-IoT, 2025

[Ariel OS: An Embedded Rust Operating System for Networked Sensors & Multi-Core Microcontrollers](https://arxiv.org/abs/2504.19662),
Elena Frank, Kaspar Schleiser, Romain Fouquet, Koen Zandberg, Christian Amsüss and
Emmanuel Baccelli. Inspected full text is
[arXiv v1, 28 April 2025](https://arxiv.org/html/2504.19662v1);
conference DOI `10.1109/DCOSS-IoT65416.2025.00040`.

Section VII already benchmarks native Ariel scheduling: paired threads wake
one another, and a lower-priority thread sets a flag that wakes a higher-priority
thread. Hardware includes **ESP32-C3 at 160 MHz**, nRF52840, ESP32-S3, RP2040 and
RP2350; it does not report C6 results. C3 uses a 16 MHz system timer; figures
report averages of 1000 runs. The work compares Ariel scheduler variants and
single-core versus multicore-enabled overhead, rather than FreeRTOS versus
Embassy application channels. Benchmark source is linked explicitly. This is
close prior art for our proposed native-thread experiments. [Full text, section VII and reference 18](https://arxiv.org/html/2504.19662v1)

## Inspected artifacts and cautions

The hashes below identify source inspected on the review date. They are not
claims that these exact commits generated every figure in the corresponding
publication. No third-party implementation was copied into this repository.

### Ariel research and maintained upstream benchmarks

The [research artifact](https://github.com/elenaf9/ariel-os-benchmarks/tree/eaa22a553c08646b584abf0a04c84d6da7ddd40f)
was inspected at `eaa22a553c08646b584abf0a04c84d6da7ddd40f`.
Its [manifest](https://github.com/elenaf9/ariel-os-benchmarks/blob/eaa22a553c08646b584abf0a04c84d6da7ddd40f/Cargo.toml)
pins normal Ariel to `9b4e4e0d70ac81bf78d2efcaf2d240ff0097a187`;
its [variant configuration](https://github.com/elenaf9/ariel-os-benchmarks/blob/eaa22a553c08646b584abf0a04c84d6da7ddd40f/benchmarks/laze.yml)
selects reallocation fork `29f21cba6fb574a17d48514ee1448fe365ce1bb5`.
The [preemption test](https://github.com/elenaf9/ariel-os-benchmarks/blob/eaa22a553c08646b584abf0a04c84d6da7ddd40f/benchmarks/bench_thread_flags_preempt/src/main.rs)
uses priority 1 and 2 threads; the lower-priority initiator times 1000 flag-set
operations while the higher-priority responder repeatedly waits. Its
[wrapper](https://github.com/elenaf9/ariel-os-benchmarks/blob/eaa22a553c08646b584abf0a04c84d6da7ddd40f/bench-multicore/src/lib.rs)
also checks migration for configured multicore runs. This path has no message
payload or request/reply checksum.

At this pinned research revision, the complete repository tree contains no
identified license/notice file; an archive-wide text inspection also found no
license grant, SPDX identifier or copyright/license banner. The manifest has no
package license declaration. Public access therefore does not establish
permission to copy this research implementation, and the separate Ariel OS
dependency's license does not automatically license this repository's source.
Use the ideas or separately licensed maintained upstream tests; resolve source
permissions before importing research code.

The maintained Ariel tree at
[`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9)
already contains
[flag handoff](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks/bench-sched-flags/src/main.rs)
and
[equal-priority yield](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks/bench-sched-yield/src/main.rs)
tests. The latter batches 10,000 initiator yields and divides the reported
iteration average by two for its context-switch figure. The
[ESP timing backend](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-bench/src/esp.rs)
reads the system timer around a loop, checks subtraction and returns integer
total ticks divided by iteration count. An average from that API does not
contain per-operation tail or per-boot variation. Prefer qualifying these
existing tests before creating another implementation of their basic behavior.

### Wang firmware: public source, partial logs, unmatched paths

The browser search tool rejected the Purdue domain, but unauthenticated HTTPS
REST requests succeeded. The
[workspace](https://github.itap.purdue.edu/wang32/FreeRTOS_RTIC_EmbassyIDT_Workspace/tree/c70322fa900ad18ee58d46b10e56a235939ef459)
at `c70322fa900ad18ee58d46b10e56a235939ef459` is a wrapper; individual firmware
projects are independent repositories. Relevant inspected revision identities:

| Artifact | Inspected commit |
| --- | --- |
| [FreeRTOS task latency](https://github.itap.purdue.edu/wang32/stm32f3_FreeRTOS_tasklatency_v2/tree/89e8bf173e6ce96b8c95e223523e2dd2a51362fd) | `89e8bf173e6ce96b8c95e223523e2dd2a51362fd` |
| [RTIC task latency](https://github.itap.purdue.edu/wang32/stm32f3_rtic_tasklatency/tree/846c8b44c4d11ee3e10add92977a518719d23767) | `846c8b44c4d11ee3e10add92977a518719d23767` |
| [Embassy task latency](https://github.itap.purdue.edu/wang32/stm32f3-embassy-tasklatency/tree/14eadafb2d6125b0c5c42b12bf3ed2d40fb3c6c4) | `14eadafb2d6125b0c5c42b12bf3ed2d40fb3c6c4` |
| [Crane FreeRTOS](https://github.itap.purdue.edu/wang32/Crane2026_LPV_STM32F3_FreeRTOS_v2/tree/4dbf8a92410456c12685d8149c1450924c3dc879) | `4dbf8a92410456c12685d8149c1450924c3dc879` |
| [Crane RTIC](https://github.itap.purdue.edu/wang32/stm32f3_lpv_rtic/tree/3ff7efdd4e97ea9a0787bcbde1fc7a716b75d5d7) | `3ff7efdd4e97ea9a0787bcbde1fc7a716b75d5d7` |
| [Crane Embassy](https://github.itap.purdue.edu/wang32/stm32f3_lpv_embassy_v2/tree/66c09eb898c40fc3ed2c805c4e2c30201ec2a576) | `66c09eb898c40fc3ed2c805c4e2c30201ec2a576` |

Only the task-latency artifacts below received a source/log audit; listing
Crane revisions is identity verification, not a full audit of that application.

The [Embassy source](https://github.itap.purdue.edu/wang32/stm32f3-embassy-tasklatency/blob/14eadafb2d6125b0c5c42b12bf3ed2d40fb3c6c4/src/main.rs)
joins `sender()` and `receiver()` futures inside **one main executor task**.
There are no two separately spawned Embassy tasks on that path. A Signal
carries a timestamp; the receiver reads DWT before logging, then acknowledges
so the sender can continue. It gathers 1005 iterations and logs between samples.
Thus it measures a particular future/Signal path, not the cost of scheduling
two independent executor tasks or our complete request/reply exchange.

Its [public log](https://github.itap.purdue.edu/wang32/stm32f3-embassy-tasklatency/blob/14eadafb2d6125b0c5c42b12bf3ed2d40fb3c6c4/embassy_task_latency_log.csv)
reports **8 MHz** system/AHB/peripheral clocks at startup, followed by an initial
278-cycle observation and subsequent 322-cycle values. Source initialization
uses the HAL default configuration. This artifact needs reconciliation with
the paper's usual 72 MHz protocol before converting or comparing elapsed times.
It is evidence about these inspected bytes, not proof that every published
measurement used the wrong clock.

The [RTIC README](https://github.itap.purdue.edu/wang32/stm32f3_rtic_tasklatency/blob/846c8b44c4d11ee3e10add92977a518719d23767/README.md)
describes UART-triggered higher-priority work spawning a lower-priority LED
task, acknowledges between-sample RTT effects, and recommends collecting
before printing. A
[latency log](https://github.itap.purdue.edu/wang32/stm32f3_rtic_tasklatency/blob/846c8b44c4d11ee3e10add92977a518719d23767/rtictasklatency_log.csv)
is public. The README reserves rights and describes educational/research use;
inspect permissions before copying source. We can independently apply the
measurement questions without importing that implementation.

## Existing benchmark methods to reuse

| Primary source | Relevant existing behavior | Use here |
| --- | --- | --- |
| [Thread-Metric specification, ThreadX `93387b0a60382a29579bbad5045c68328e3751df`](https://github.com/eclipse-threadx/threadx/blob/93387b0a60382a29579bbad5045c68328e3751df/utility/benchmarks/thread_metric/thread_metric_readme.txt) | Eight tests include equal-priority cooperative scheduling, distinct-priority preemption, interrupt processing, interrupt-to-thread preemption, message operations, semaphores and allocation. Recommends 30-second throughput intervals. | Adopt explicit primitive scenarios and correctness counts. Its message test sends/receives a 16-byte message in one thread; it is not our two-task RTT. Do not label a modified Rust experiment a conforming Thread-Metric result. |
| [Zephyr latency measurement, v4.3.0 commit `3568e1b6d5cdd51a6b964a2a1d6d29200fea2056`](https://github.com/zephyrproject-rtos/zephyr/blob/3568e1b6d5cdd51a6b964a2a1d6d29200fea2056/tests/benchmarks/latency_measure/README.rst) | Names yield, ISR return, ISR-to-different-thread, semaphore, mutex, lifecycle and queue operations; distinguishes userspace configurations. | Qualify existing upstream benchmark on C6 before writing equivalent native-kernel adapters. Keep its results separate from a shared Rust application path. |
| [Zephyr current benchmarking framework documentation](https://docs.zephyrproject.org/latest/develop/test/benchmark.html) | Distinguishes sampled, timed and manually bracketed spans; describes controls, warmup/cold costs and retained-sample percentiles. | Record actual timing endpoints and observer control. Preserve gross raw values when reporting any corrected result. This is a live documentation snapshot, not proof our pinned Zephyr release contains every described feature. |

These upstream implementations contain C. Evaluating unmodified examples fits
the working agreement; any new authored C adapter still needs the user's
language discussion. No native ThreadX-on-C6 support is inferred from its
portable benchmark specification.

## What the existing C6 experiment establishes

The [clocked handoff dataset](../../results/clocked-handoff-2026-10-23/README.md)
is useful **local application-path evidence**: two independent tasks, two
capacity-one `u32` channels, checked request/reply data, 1000 RTT samples per
boot, five warm-reset boots per stack, and before/after clock readback.
It records 7 µs Embassy and 62 µs Rust std/IDF median RTTs. Those values answer
what these selected Rust APIs cost in these configurations.

They do not rank kernel schedulers. Embassy tasks share one cooperative
executor. The IDF configuration reports initiator priority 1 and responder
priority 5; Rust std channels include their selected synchronization machinery.
Four channel operations, two sends and two receives across two queues, plus
their blocking/wakeup paths occur in a complete exchange.
An RTT divided by two would not establish either one-way event latency or a
single context-switch cost. These restrictions follow from our own experiment
and [implementation review](../../firmware/esp-idf/docs/api.md).

## Changes that make the next work valuable

The following are recommendations for this repository, not claims that the
cited papers validated our implementation.

| Question | Next experiment and required record |
| --- | --- |
| Where does the application-channel cost come from? | Keep the existing std/Embassy RTT. Add native FreeRTOS notification/queue and native esp-rtos/Ariel flag or yield paths as separately named tests. Separate an operation with no waiter from an operation that wakes a blocked task. |
| Does priority change the result? | On preemptive threads, explicitly test equal priority and higher-priority responder, recording actual priorities and scheduling policy. On async runtimes, record executor topology; a ready future in one task, a separately spawned task, and a different-priority executor are separate scenarios. |
| What can interrupt the application? | Separate hardware edge→ISR entry, ISR execution, and ISR→task response. Record interrupt priority and masking. Repeat with bounded compute and radio traffic; an idle handoff is insufficient. Use external stimulus and observation when equipment is available. |
| Are deadlines met under contention? | Use absolute periodic deadlines; retain signed error, lateness, missed/coalesced events and overflow. Add task-count and offered-load sweeps with fixed budgets and per-task progress counters. Define async yield points and an explicit timeout for a non-yielding case. |
| Is a 7 µs result resolved adequately? | Keep the 1 µs raw result and observer distribution. Qualify finer hardware timing or batch repeated operations to estimate an average; a batched average cannot recover individual-event tails. Cross-check with GPIO/external measurement where practical. |
| How much memory is required? | Separate static objects, sample buffers, reserved stacks, actual stack high-water and heap peaks. Compare minimum passing configurations under the same stress workload rather than application ELF totals alone. |
| How repeatable is it? | Treat independently reset runs as the main current replication unit, samples within a boot as repeated observations. Vary order, preserve failures/outliers and report per-boot effects. One physical board cannot establish variation across devices. |
| Does an improvement help real applications? | After primitives, run a checked sensor/event→queue→radio/report→actuation workload with declared load, energy and deadline requirements. Measure latency/delivery/recovery together before recommending a runtime or custom kernel. |

There is already substantial overlap in measurement questions and native
primitive tests. Our defensible near-term value is a controlled **C6-specific
replication and extension** for the actual Rust application paths and workloads
we need, with reusable upstream tests where available. It does not depend on
claiming the first cross-runtime benchmark. A new custom OS needs an unmet
requirement or measured bottleneck after simpler configuration/API changes.

## Search boundary and remaining leads

Queries included `ESP32-C6 Embassy FreeRTOS latency benchmark`, `ESP32-C6 RTIC
benchmark`, `esp-rtos benchmark latency`, `Ariel ESP32-C6 benchmark`, the exact
paper titles, and author/artifact follow-ups. Primary full texts, author pages,
repository trees, manifests, sources and selected logs were inspected.

No inspected artifact in this focused search supplies the complete same-board
C6 matrix of our Rust std/IDF, esp-rtos/Embassy, Ariel and RTIC paths with all
planned peripheral/radio/power cases. This is a search result, not an absence
claim about all public or private work. Existing C6 power/radio/runtime studies
belong in the wider prior-work review.

Additional unverified leads from this search:

- [C6 SPSC channel article](https://hadezuka.dev/channel-asyncmu-nganggur-nunggu-heap-ini-cara-bikin-spsc-channel-yang-100-l1-cache-di-rust/):
  claims sub-100 ns latency and data-cache measurements. No reproducible pinned
  measurement artifact was audited; exclude its numbers from comparisons.
- [esp-csi-rs 0.7.0 manifest](https://docs.rs/crate/esp-csi-rs/0.7.0/source/Cargo.toml.orig):
  describes esp-rtos tracing hooks for CPU/ESP-NOW experiments and a C6 feature.
  Candidate for radio-load methodology review, not an audited OS comparison.
- [Kairos/rusty_rtos_backoff 0.1.0](https://docs.rs/crate/rusty_rtos_backoff/0.1.0):
  describes a Rust FreeRTOS-family effort and a C6-class compile target. An
  ISA-compatible target does not establish a working C6 kernel port. Unqualified.

Verification performed: unauthenticated API revision/tree lookup, pinned source
and selected log reads, and comparison against our recorded experiment contract.
Not performed: upstream builds, complete dataset reproduction, author contact,
or an exhaustive publication/repository search.
