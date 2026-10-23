# Research gaps before further experiments

Reviewed **2026-10-23**, after the [prior-work audit](prior-work.md) and three
independent reviews of measurement, hardware and platform evidence. The initial
assessment below identified gaps; the [follow-up closure record](#follow-up-research-closure)
links the completed research briefs and distinguishes their remaining physical
qualification. No firmware implementation, build or board experiment was
performed as part of the research assessment itself. The subsequent
[matched IDF queue study](../../results/idf-queue-api-2026-10-23/RESULTS.md)
has now completed the bounded implementation and physical qualification
described below; hardware-phase and adoption gaps remain open.

The earlier work established a broad candidate survey, exact build provenance,
initial physical baselines and several important limitations. It was lighter on
the version-specific contracts needed to turn those baselines into a fair
platform comparison. Recording a limitation is useful, but does not establish
the replacement method.

## Later evidence corrections and current gates

The [validated feedback audit](../reviews/2026-10-23-feedback-audit.md) narrows
the later native-suite claims without changing the matched IDF experiment.
[Ariel](../../results/ariel-native-tests-2026-10-23/RESULTS.md) retains twelve
expected outputs and the operator's pass reports; its first five-second guard
has no archived wait receipt, and derived 77-tick yield output is arithmetic
without a peer-progress counter. [Zephyr native v1](../../results/zephyr-native-tests-2026-10-23/RESULTS.md)
admits **0/3 boots** despite complete row sets and upstream banners: the required
header is compiled out and all ROM logs report image-SHA warnings. Correcting
the parser's 24-row false count to 47 does not change failed admission.

The [Zephyr source audit](zephyr-native-audit.md) maps **19 observer-adjusted
and 28 unadjusted metrics** with distinct endpoints. The separate
[Rust probe](../../results/zephyr-rust-build-2026-10-23/README.md) retains three
failed build stages, no final Rust image and no board result; it does not prove
universal C6 incompatibility. Current flash was restored and independently
verified after the physical attempts; no new firmware repairs these findings.

CPU/ISR utilization, sustainable capacity, stack/heap peaks, external IRQ
response, hardware offload, radios, sleep/energy, recovery and measured update
effort remain unmeasured. The [utilization audit](utilization-audit.md) separates
those questions from elapsed-path latency. Before a prospective load sweep,
close output/image/metric-semantics gates and qualify a matched workload and
profiling method, including attribution, overhead and stopping rules. This is
not an automatic all-OS sweep or evidence selecting a custom runtime.

## Resolve before the next comparative firmware

This table preserves the initial gap assessment. The closure/status record
below identifies what the later research and matched study actually established.

| Research gap | What is missing | Concrete research output |
| --- | --- | --- |
| Timing and observer validity | The timer APIs and observer costs were recorded, but their resolution, clock domains, read semantics, wrap, sleep behavior and suitability for short events were not fully reconciled. | A pinned timer-method sheet, uncertainty budget and calibration design; choose finer timing or external validation where the decision needs it. |
| Comparable concurrency contracts | The two measured application paths were traced, but an equivalent native-primitive experiment has not been specified. | Define the timed endpoints, waiter state, queue capacity, priority, wakeup, preemption, allocation and number of operations for each adapter; retain the existing application-path comparison separately. |
| Revision-specific hardware behavior | The board is rev0.2, but there is no erratum-to-workaround ledger for the selected HAL, SDK, compiler and configuration. | Pin the errata revision, identify applicable triggers, trace actual mitigation and choose checks relevant to the next workload. |
| Current maintenance and compatibility | Measured dependencies are pinned, but canonical repository moves, support policies and the released Rust/compiler/SDK compatibility picture need consolidation. | A dated compatibility and maintenance matrix with canonical sources, supported versions, CI/hardware-test evidence, known patches and an upgrade route. Preserve historical measured pins. |
| Experimental design and inference | Five boots per platform and 1,000 exchanges per scheduling boot are useful pilots, not a general justification for precise tails or broad reliability claims. | Define the decision question, useful effect or characterization range, independent experimental units, blocking/order, uncertainty and stopping rule before choosing the next sample count. |

For example, [esp-hal 1.2.0's time implementation](https://docs.rs/crate/esp-hal/1.2.0/source/src/time.rs)
exposes whole-microsecond instants even though the
[C6 SYSTIMER backend](https://docs.rs/crate/esp-hal/1.2.0/source/src/timer/systimer.rs)
has finer hardware ticks: 16 MHz with our 40 MHz crystal. The
[IDF timer API](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/esp_timer.html)
also distinguishes counter reads, callback dispatch and sleep behavior.
Microsecond quantization matters for our 7 µs median round trip. Changing the
reported unit to nanoseconds would not improve the measurement. Batched averages
also cannot establish individual-event tail latency.

The [official C6 errata summary](https://docs.espressif.com/projects/esp-chip-errata/en/latest/esp32c6/02-errata-summary/index.html)
marks CPU-863, CLK-8588 and RMT-176 as affecting rev0.2. CPU-206 is fixed in
rev0.2. CPU-863 concerns misaligned **data access**, which is a different issue
from our instruction-placement probe. Applicability is not proof that our
workload triggers a defect or that a particular stack lacks a workaround; that
mapping still needs source-level research.

The former [esp-idf-sys repository](https://github.com/esp-rs/esp-idf-sys)
was archived on September 19, 2026 after development moved to the
[esp-rs/esp-idf monorepo](https://github.com/esp-rs/esp-idf). Archiving the old
repository does not mean the Rust wrappers were abandoned. The monorepo also
describes their community maintenance and missing hardware-in-the-loop tests;
vendor support for the C SDK is not evidence of equivalent Rust-wrapper
maintenance. The [esp-hal support policy](https://github.com/esp-rs/esp-hal#support-policy)
is another concrete input to the maintenance comparison.

The [NIST experimental-design guidance](https://www.itl.nist.gov/div898/handbook/pri/section3/pri33.htm)
and [blocking guidance](https://www.itl.nist.gov/div898/handbook/pri/section3/pri332.htm)
provide a starting point for the statistical brief. Samples within a boot,
independent boots, builds, boards and test days are different units. Repeating
one board's warm-reset experiment does not sample manufacturing variation.
There is no universal sufficient sample count. Broad hardware exploration is
already an agreed objective; we should characterize capabilities and tradeoffs
without inventing a product deadline or arbitrary overall score.

## Close before the relevant hardware phase

| Phase | Under-researched contract | Evidence to establish before implementation or measurement |
| --- | --- | --- |
| GPIO, peripheral, native USB, power and radio fixtures | Carrier/module identity, exposed pins, antenna and power path remain unknown despite verified silicon, flash, crystal and UART identity. | Inspect markings and match a schematic; document usable pins, boot/reset interactions, power rails, regulator/LED/USB loads and measurement points. This is not a blocker for UART-only startup checks. |
| Radio and concurrent peripherals | A feature checklist does not establish ownership of timers, interrupts, scheduler, allocator, critical sections, priorities or radio coexistence in the selected Rust release. | A per-stack ownership and protocol-role table, with driver/blob versions and pinned restrictions; identify conflicts before combining workloads. |
| External timing and energy | Equipment requirements are listed, but the actual fixtures and their uncertainty are not qualified. | Inventory available instruments/peers, reuse a reference experiment, then verify timing resolution, lost-event detection, energy bandwidth, burden and the measured rail boundary. |
| RAM and resource comparisons | Static allocations are visible; comparable runtime memory categories and collection methods are not yet defined. | Separate reserved/static memory, stack peaks, heap live/peak usage, fragmentation and sample buffers; establish units, coverage and instrumentation cost for each stack. |
| Sleep, wake and sustained operation | Generic chip/SDK support does not establish the selected stack's idle eligibility, clock locks, retained state or timer continuity. | Trace sleep/clock policy and wake sources, then design tests for retained peripherals, radio reconnect, missed work and long-duration behavior. |

For a concrete radio example, the
[esp-radio 1.0.0-beta.0 C6 API](https://docs.espressif.com/projects/rust/esp-radio/1.0.0-beta.0/esp32c6/esp_radio/index.html)
requires allocation and a preemptive scheduler for Wi-Fi/BLE initialization;
raw 802.15.4 has different requirements. The
[IDF coexistence guide](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-guides/coexist.html)
is role-specific. Chip capability, an SDK's role matrix and a particular Rust
crate's enabled combinations must be recorded separately.

Unresolved reports such as
[IDF issue 19036](https://github.com/espressif/esp-idf/issues/19036)
are test leads, not established silicon errata. A reported timer backstep under
radio/peripheral workloads calls for investigation in that phase; it does not
by itself invalidate our earlier short radio-off experiments.

Our [alignment probe](../../results/alignment-probe-2026-10-23/README.md)
already demonstrates a placement confound. The next comparison must explicitly
choose whether it controls placement or measures each stack's ordinary build
policy. Caller, timer and cache/IRAM placement also need consideration; identical
workload instruction bytes alone do not settle those effects.

## Close before adoption or a custom OS commitment

Research the actual Rust path for firmware updates, rollback, interrupted
storage operations, watchdog/crash capture and recovery, credential/state
migration and dependency upgrades. A full-flash laboratory backup is not
deployed-device recovery. Use existing upstream failure-test designs where
possible, then qualify the shortlisted configurations physically.

Audit licenses for the resolved components and radio blobs, and identify trust
boundaries at unsafe Rust, FFI, DMA and interrupts. Distinguish available
protection mechanisms from protections configured and tested in a candidate.
This requires evidence about the proposed configuration, not a new requirement
to provision irreversible security settings during exploration.

Before a custom kernel, document an observed unmet behavior and evaluate the
smallest maintained remedy: a native API instead of a wrapper, a priority or
executor change, static allocation, code placement, a driver integration or
LP-core offload. Compare a small extension with owning scheduling, drivers,
radios, power, recovery and upgrades. A short empty-loop result cannot establish
that a new kernel is the useful remedy.

## Order and limits

Close the five research briefs in the first table before adding the next
comparative firmware. Close hardware-phase contracts as those experiments
approach. Adoption criteria need deeper evidence for the shortlist, rather than
full product qualification of every surveyed runtime.

The OS discovery, C6-versus-other-chip distinctions, pinned Rust standard,
backup/restore procedure and raw-result provenance already received substantial
attention. They do not need to be redone wholesale. Missing physical results
are not automatically missing literature research; research establishes a
credible route and method, while board tests establish that our configuration
actually works. More general cautions or more identical pilot runs would not
close the specific gaps above.

## Follow-up research closure

Completed later on **2026-10-23**, with pinned source/configuration reads,
archived-binary inspection and independent review. The five research briefs
are complete for the next narrow radio-off scalar queue/timer experiment;
this is not blanket readiness for every hardware or product profile.

| Brief | Research decision | Current qualification and remaining boundary |
| --- | --- | --- |
| [Timing validity](timing-and-concurrency-contract.md#timer-method-what-the-current-values-mean) | Select raw SYSTIMER ticks; high-level APIs truncate to µs and the HP performance counter stops in WFI. | Pinned private-function linkage, configuration readback, ordered reads, observer and blocked/WFI checks passed for both study images. Counter scale is nominal; independent electrical calibration and connected/sleep reliability remain later. |
| [Concurrency equivalence](timing-and-concurrency-contract.md#concrete-next-comparison-and-its-admission-controls) | Compare std channels and native queues on the same IDF kernel with matched thread creation/priorities. Published esp-rtos 0.4.0 lacks the development application-thread API. | All 24 study boots passed checked replies, phase/blocked-state admission and cleanup. Native queues have occupancy queries; std emptiness is inferred from the reviewed single-inflight path. Results remain complete instrumented RTTs, not isolated wakeups. |
| [Rev0.2 workarounds](rev02-errata-workarounds.md) | Account for all ten official entries, IDF's linked copy mitigation and HAL/Ariel ROM uncertainty; aligned scalar queue/timer work can proceed. | Newly linked paths were inspected and scalar correctness passed in the study. Arbitrary memory copies, RMT, sleep/LP and connected timer reliability retain their own prerequisites. |
| [Compatibility and maintenance](platform-compatibility.md) | Preserve proved pins; distinguish published release from development sources and canonical repository/support policies. Ariel's release offers an older-compiler alternative. | Separate upgrade/build/boot probes for unproved tuples; choose a thread API deliberately rather than inferring one from the manifest version. |
| [Experimental design](experimental-design.md) | Characterize one contract with a frozen budget/order, paired boot contrasts, conditional uncertainty and explicit failure handling. | Frozen schedule/seed/manifest and reviewed analyzer were used for twelve complete pairs across three labeled sessions, without retries. All paired median differences agree; session labels and identical quantized medians do not establish IID sampling. |

The [experiment contract](../experiments/native-queue-comparison.md) made these
choices concrete. Its [completed results](../../results/idf-queue-api-2026-10-23/RESULTS.md)
contain 24,000 checked round trips and 24,000 observer observations. Boot medians
are 62.875 µs for std channels and 19.9375 µs for native queues at the nominal
16 MHz scale, with the same IDF kernel/threads/priorities. This supports an API
remedy before a kernel replacement; it does not select an OS. Original firmware
was restored, digest-verified and reset. The
[independent result audit](../reviews/queue-api-results.md) checks the raw data
and states the inference limits.

Before adopting a general-purpose adapter, evaluate the
[existing compatible Rust native-API wrappers](idf-native-api-options.md) and
their ownership/error policies. Unmodified Ariel/Zephyr native suites remain
separate qualification paths, and the hardware-phase/adoption gaps above remain
open.
