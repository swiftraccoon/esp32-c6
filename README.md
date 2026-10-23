# ESP32 C6 operating system evaluation

Research and a reproducible test plan for choosing an ESP32-C6 platform, including Rust applications, existing kernels, and a possible custom OS. Scope includes every hardware subsystem the connected board exposes. Survey checked **2026-10-22**, with prior-work, method and compatibility research updated **2026-10-23**; source snapshots and limitations are recorded in the sheets.

The attached device is confirmed as an **ESP32-C6 revision 0.2 with 8 MB flash and a 40 MHz crystal**, connected through a CP2102N bridge. The carrier model/pinout remains unknown. Physical evidence covers bare metal, Embassy/esp-rtos, ESP-IDF/FreeRTOS, Ariel startup/native outputs, three failed Zephyr native attempts and a failed RTIC monotonic attempt. The pre-session full-flash state was restored, separately digest-verified and reset after testing.

The [matched IDF queue API study](results/idf-queue-api-2026-10-23/RESULTS.md)
completed **24/24 scheduled boots**, twelve paired blocks across three labeled
sessions, with 24,000 checked round trips and 24,000 observer samples. Every
boot's median is **62.875 µs for Rust std channels** or **19.9375 µs for native
FreeRTOS queues**, using nominal 16 MHz raw ticks. Both use the same IDF kernel,
Rust threads and priorities. This supports investigating an API change before
replacing the OS; it measures complete instrumented paths, with no independent
clock calibration or worst-case guarantee. The [independent review](docs/reviews/queue-api-results.md)
checks raw evidence and inference limits. Original firmware is restored and
digest-verified after this study.

The separate [IDF accounting qualification](results/idf-accounting-qualification-2026-10-23/RESULTS.md)
completed **3/3 scheduled boots** under frozen inputs. It distinguishes blocked
from checked busy-worker attribution, demonstrates the current-task counter's
open interval, and observes a touched 16 KiB allocation and recovery. Task
enumeration costs roughly 1 ms in this profile. These are method controls and
allocator/stack-fill indicators; physical CPU utilization, runtime global peaks
and sustainable capacity remain unmeasured. The pre-session full flash is restored,
separately digest-verified and reset.

The [finite-load qualification](results/idf-load-qualification-2026-10-23/RESULTS.md)
completed **2/2 scheduled boots**, retaining **4,032 nominal releases** across
two generator controls and four 250 ms work pulses per profile. The 4,000/s
pulses produce queue-full losses and synthetic 2 ms deadline misses. These
are valid adverse outcomes, not sustainable-capacity estimates or an on/off
instrumentation comparison. Before/after task and memory indicators accompany
the raw jobs. Final replay exposed a JSON float-reading defect, now
[corrected with strict regressions](docs/reviews/idf-load-replay-correction.md)
while preserving every original capture and frozen input. Original flash is
restored, separately verified and reset.

The [RTIC monotonic probe](results/rtic-qualification-2026-10-23/RESULTS.md)
builds unmodified with Rust1.96.0, but its **first board attempt fails**: all
seven markers arrive, followed by another ROM startup during observation.
The two later attempts were not run. Its reset cause is unconfirmed; startup
watchdog inheritance is a source lead. The
[follow-up](docs/research/rtic-startup-followup.md) calls for retaining a complete
second reset reason before changing firmware. Original flash is restored,
separately verified and reset. This does not select or exclude RTIC generally.

[First physical results](results/initial-2026-10-22/README.md): 15 successful measured boots, 960 compute and 960 calibration samples, raw captures, exact firmware and source snapshots. The common workload takes about 283 microseconds on all three stacks; this is a compute/observer baseline rather than an OS performance ranking.

The [clock verification and message experiment](results/clocked-handoff-2026-10-23/README.md) adds 15 passing boots and 10,000 checked request/reply exchanges. All runs read back matching 160 MHz CPU / 80 MHz flash settings. Median round trips are 7 µs through Embassy channels and 62 µs through Rust std channels on FreeRTOS, with the different task models and priorities recorded. These are complete application paths, not isolated kernel-switch measurements.

A separate [alignment probe](results/alignment-probe-2026-10-23/README.md) shows why compute placement matters: unchanged workload bytes take 282–283 µs in the aligned build and 308 µs in the original new bare-metal image. The [initial flash-clock errata](results/initial-2026-10-22/errata.json) remains historical evidence; new register readback resolves that control for the new experiment.

[Ariel qualification](results/ariel-qualification-2026-10-22/README.md) passed both async and stackful-thread startup/UART checks. The qualified development checkout requires Rust 1.98 or newer; these builds used 1.99.1 and are not part of the Rust 1.96 performance comparison. The [released 0.6.0 alternative](docs/research/platform-compatibility.md) declares Rust 1.95 and has different dependencies; it has not been locally built.

The [Ariel native tests](results/ariel-native-tests-2026-10-23/RESULTS.md)
produce the expected output in all **12 scheduled boots**. Original operator
pass reports remain preserved, but the first five-second observation guard has
no archived wait receipt. Yield prints **155 ticks** per iteration and flags
**215**; the derived **77** is arithmetic without a peer-progress counter or
validated switch latency. Priority/lock assertion markers support their
particular control flows. These are partial qualification references, not
matched per-event or cross-OS results.

The [Zephyr native attempts](results/zephyr-native-tests-2026-10-23/RESULTS.md)
each emit 47 metric rows and an upstream success banner, but **0/3 qualify**
under frozen v1: the required frequency header is compiled out and every boot
has a ROM image-SHA warning. The separate [Rust build probe](results/zephyr-rust-build-2026-10-23/README.md)
retains three failed stages and produces no Rust image or board result. The
[feedback audit](docs/reviews/2026-10-23-feedback-audit.md) corrects earlier
claims; the [utilization audit](docs/research/utilization-audit.md) records the
broader gaps. Physical CPU utilization, sustainable capacity and global memory
peaks remain unmeasured. No platform or custom runtime is selected.

## Start here

| Reference | Purpose |
| --- | --- |
| [Evidence and experiment gates](docs/assurance-workflow.md) | Required qualification, contrary tests, comparative admission and read-only checks before finalizing new work |
| [Source and result verification](docs/reviews/public-snapshot.md) | Available artifacts, raw-capture replay and the limits of checks requiring complete experiment bundles |
| [Evidence and coverage ledger](docs/evidence-ledger.md) | Typed claims, archive identities and all 910 platform/domain pairs; unknown and partial coverage remain explicit |
| [Independent gate review](docs/reviews/assurance-independent.md) | Executed adversarial checks, retained replay results and limits of automated assurance |
| [IDF accounting qualification](results/idf-accounting-qualification-2026-10-23/RESULTS.md) | Three actual diagnostic boots, raw counters, observer cost and bounded allocation indicators |
| [Accounting method and controls](docs/research/idf-accounting-method.md) | Pinned task/heap API semantics, upstream reuse and the current-task accounting blind spot |
| [IDF finite-load qualification](results/idf-load-qualification-2026-10-23/RESULTS.md) | Two actual boots, complete offered-job outcomes, finite overload, task and memory indicators |
| [Finite-load method and controls](docs/research/idf-load-method.md) | Prior-work reuse, generator checks, explicit losses, synthetic deadlines and measurement limits |
| [RTIC qualification result](results/rtic-qualification-2026-10-23/RESULTS.md) | Unmodified Rust builds, one failed physical attempt, retained raw/timed receipts and two stopped attempts |
| [RTIC startup investigation](docs/research/rtic-startup-followup.md) | Unconfirmed reset cause, forensic follow-up and smallest source-supported remedies |
| [Prior work and experiment value](docs/research/prior-work.md) | Verified studies/artifacts, overlap, reuse choices and questions worth measuring next |
| [Research readiness](docs/research/research-readiness.md) | Five completed method/platform research briefs and remaining hardware/adoption gaps |
| [Matched IDF queue API results](results/idf-queue-api-2026-10-23/RESULTS.md) | Completed 24-boot same-kernel comparison, raw-tick results and limits |
| [Same-kernel experiment contract](docs/experiments/native-queue-comparison.md) | Implemented design, qualification and frozen study controls |
| [Clock verification and message results](results/clocked-handoff-2026-10-23/README.md) | Actual request/reply timings, raw clock registers and repeated boots |
| [Alignment probe](results/alignment-probe-2026-10-23/README.md) | Separate alternating-build experiment and placement limits |
| [Ariel qualification](results/ariel-qualification-2026-10-22/README.md) | Unmodified upstream builds and two physical smoke tests |
| [Ariel native-test results](results/ariel-native-tests-2026-10-23/RESULTS.md) | Twelve expected outputs, operator reports and partial observation-window evidence |
| [Zephyr failed native qualification](results/zephyr-native-tests-2026-10-23/RESULTS.md) | Three completed output sets, zero admitted boots, preserved defects and replay |
| [Zephyr Rust build probe](results/zephyr-rust-build-2026-10-23/README.md) | Actual workspace, layout and generated-code build failures; no image or board pass |
| [Validated feedback](docs/reviews/2026-10-23-feedback-audit.md) | Evidence corrections, per-metric source audit and remaining selection gaps |
| [Collector admission repair](docs/reviews/collector-boot-guard.md) | Reproduced hidden-crash defect, shared Rust guard and archival/regression checks |
| [First benchmark results](results/initial-2026-10-22/README.md) | Actual board results, conditions, artifacts and limitations |
| [Shared workload](crates/bench-core/src/lib.rs) and [Rust collector](tools/benchctl/README.md) | Implemented deterministic workload, capture, replay and ELF inspection |
| [Comparison and evaluation order](docs/comparison.md) | Candidate overview, radio/Rust boundaries, and how to make the decision |
| [Hardware identity](docs/hardware.md) | Actual connected device, host tools, and preparation status |
| [Wireless capabilities](docs/hardware.md#wireless-reference) | C6 radio/PHY limits, software protocols, pinned coexistence matrix and Rust restrictions |
| [Full hardware coverage](docs/hardware-coverage.md) | Peripheral, radio, LP-core, power, storage and security experiments |
| [Benchmark protocol](docs/benchmark-plan.md) | Controlled configurations, measurement methods, fixtures and reproducibility |
| [Coding standard](docs/coding-standard.md) | Adopted pinned Rust guides and rules before implementation |
| [Agent instructions](AGENTS.md) | Repository expectations for anyone writing code |
| [API checklist](docs/rust-api-checklist.md) | Applicable Rust API `C-*` identifiers and review evidence |
| [Results directory](results/README.md) | Archived experiments, raw evidence and an unmeasured planning template |

## Kernel and Rust platform sheets

Each sheet covers actual C6 evidence, project/source links, license, supported features, Rust fit, strengths, costs, and useful experiments.

| Candidate | Classification |
| --- | --- |
| [ESP-IDF FreeRTOS](docs/os/esp-idf-freertos.md) | Vendor RTOS stack; established Rust application route with `std` |
| [Zephyr](docs/os/zephyr.md) | Portable RTOS; native v1 qualification and selected Rust build probe failed |
| [Apache NuttX](docs/os/nuttx.md) | POSIX-oriented RTOS; experimental Rust integration |
| [RIOT](docs/os/riot.md) | IoT/networking RTOS; current RV32 Rust integration blocker |
| [RT-Thread](docs/os/rt-thread.md) | Recent, limited upstream C6 kernel port |
| [esp-rtos](docs/os/esp-rtos.md) | Rust `no_std` runtime with preemptive threads |
| [Embassy with esp-hal](docs/os/embassy.md) | Async executor stack; proposed configuration uses esp-rtos |
| [Ariel OS](docs/os/ariel-os.md) | Integrated Rust library OS with C6 board support |
| [RTIC](docs/os/rtic.md) | Rust interrupt-driven concurrency framework with a C6 backend |
| [Custom Rust firmware or OS](docs/os/custom-rust.md) | Bare-metal baseline and staged custom-development choices |

## Frameworks and experimental systems

| Reference | Included systems |
| --- | --- |
| [Application runtimes](docs/os/idf-runtimes.md) | Arduino-ESP32, MicroPython, CircuitPython, Moddable/XS |
| [Additional language and sandbox routes](docs/os/other-runtimes.md) | AtomVM Erlang/Elixir, Toit/Jaguar, NodeMCU Lua, TinyGo, WAMR, Embedded Swift |
| [Mongoose OS](docs/os/mongoose-os.md) | Newly integrated C6 framework above IDF/FreeRTOS |
| [Contiki-NG C6 port](docs/os/contiki-ng.md) | Community mesh networking port hosted in a FreeRTOS task |
| [MOS-Rust](docs/os/mos-rust.md) | Experimental Rust scheduler, shell and C6 BLE adapter |
| [Research systems](docs/os/research-kernels.md) | project-trobbio, ThistleOS, OpenC6 BIOS |

Embassy, esp-rtos and Ariel share components; Arduino and several other runtimes retain FreeRTOS. The comparison distinguishes execution models and full stacks instead of counting every name as a different kernel.

The [ecosystem audit](docs/research/ecosystem-audit.md) documents excluded or unverified candidates such as ThreadX, RTEMS, Mynewt, AliOS and LiteOS. The [Rust support audit](docs/research/rust-support-audit.md) covers Tock, Hubris, exact targets, radio layers, source snapshots and compatibility traps. These are a survey of public support evidence; new ports can be admitted using the same build/board proof requirements.

## Current recommendation

The same-IDF study establishes a useful API-path difference while retaining the
kernel; review [existing Rust native-API wrappers](docs/research/idf-native-api-options.md)
before adopting a general-purpose adapter. First reconcile acceptance rules
with actual image/output behavior, retain failed attempts, and apply the
[per-metric timing audit](docs/research/zephyr-native-audit.md). A prospective
load method now has a frozen arrival/backlog/deadline contract and two physical
qualification boots. A comparative instrumentation-cost study and longer,
repeated workload qualification remain separate next steps. The accounting and
finite-load probes provide inclusive attribution and bounded allocator
indicators. RTIC now needs reset attribution under a separate diagnostic before
startup changes; Rust build remedies and broader hardware coverage remain separate
gates; driver, radio, sleep/energy, recovery and maintenance evidence is still
needed for platform selection.

Keep results by capability and workload until a real application supplies requirements. Compare the custom Rust baseline against a measured weakness before deciding to own a new OS. Discuss any proposed C/C++ or assembly work before implementation, as required by the language preference and coding standard.
