# Prior work and experiment value

Reviewed 2026-10-23. There is substantial existing work on C6 power, radios,
portable runtimes and embedded Rust scheduling. The useful program here is
**controlled replication plus measurements that change our platform decision**.
Published workloads, fixtures and native scheduler tests should provide the
starting points. A claim that this is the first such suite is not established
by this bounded review.

## Earlier research scope

Before this review, the repository contained platform/driver/Rust support
surveys, source audits and vendor measurement guidance. It referenced established
compute benchmarks and Ariel's benchmark example. It did not contain a dedicated
comparative-literature inventory or the source/data reconciliation below.
The early physical work established the local harness and exposed controls that
needed attention; it did not establish a literature gap or justify a custom OS.
No reliable research-hours total was recorded.

## Verified overlap

| Prior work | What is already covered | Useful local question or reuse |
| --- | --- | --- |
| [Leon and Qoitech, 2026](https://www.qoitech.com/blog/esp32-c6-power-consumption-comparison/) | C6 current across Arduino, Zephyr and IDF under common workloads | Reproduce a simple instrument baseline when power equipment exists; add declared sleep policy and energy per successful application operation. |
| [Poli and Johansson, 2026](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6) | C6 Wi-Fi/Thread/Zigbee protocol experiments within IDF | Borrow scenarios and available logs, then measure matched Rust application behavior across viable runtimes. |
| [Electric UI](https://electricui.com/blog/latency-comparison) and [Nobile et al., 2026](https://arxiv.org/html/2603.04221v1) | Raw C6 802.15.4 timing and C6 mesh/application recovery respectively | Reuse external timing and packet-capture practice; focus on local deadline interference, delivery and recovery. |
| [Has et al., 2025](https://arxiv.org/html/2512.00035v1) and [Xiong's earlier thesis](https://www.diva-portal.org/smash/get/diva2:1908617/FULLTEXT01.pdf) | C6 WebAssembly experiments already include Rust guests | Integration references if dynamic modules become useful; validate guest equivalence, host calls, cold/warm costs and failure containment. |
| [Vandervelden et al., 2024](https://www.mdpi.com/1424-8220/24/17/5818) and [Wang, 2026](https://ietresearch.onlinelibrary.wiley.com/doi/10.1049/cdt2/7881841) | Embedded Rust scheduling, response and footprint on other MCUs | Reuse endpoint decomposition and stress questions; audit actual task topology and clocks before comparing numbers. |
| [Ariel's 2025 paper](https://arxiv.org/html/2504.19662v1) and [maintained benchmarks](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks) | Native scheduling experiments, including C3, and existing yield/flag implementations | Qualify the existing tests on C6 before authoring another native-thread suite. |
| [LLTA, 2026](https://arxiv.org/html/2609.40141v1) and [C6 timing notes](https://github.com/tu-dortmund-ls12-rt/LLTA/blob/8be6f23ca205857f5226c2848d82bad0318b44f3/docs/targets/esp32c6.md) | C6 timing analysis already records code-alignment sensitivity | Our alignment experiment is local calibration and reproduction; preserve uncertainty about the particular hardware mechanism. |

The supporting audits inspect full papers, code, configuration and selected raw
artifacts, distinguishing publication claims from inspected source snapshots:

- [C6 OS and timing prior work](prior-work-c6-os.md).
- [C6 radios, mesh and WebAssembly](prior-work-c6-radio-runtimes.md).
- [Rust scheduling papers and source audit](prior-work-rust-scheduling.md).
- [Existing benchmark assets and reuse choices](benchmark-reuse.md).

## Artifact checks that affect our design

The linked audits identify practical reasons to inspect more than an abstract
or results table. Published packet lengths can differ from checked-in send
lengths; a log's timer suffix can disagree with the API's units; source snapshots
can use different workload sizes or clocks. A pair of joined futures inside one
task has different scheduling semantics from two spawned tasks. These findings
describe the inspected artifacts; they do not prove that every published figure
used those exact bytes or configurations.

Each new experiment therefore needs source/configuration/measurement agreement:
actual wire bytes and response checks, generated code and placement, active
clock/timer configuration, timed endpoints, task relationships and licensed
reuse. Public readability alone does not grant permission to copy all source.
Where a publication and public artifact cannot be reconciled, retain the useful
method while labeling the result a new implementation or configuration.

## What our existing data are good for

| Existing evidence | Supported use | Missing decision evidence |
| --- | --- | --- |
| [Initial compute pilot](../../results/initial-2026-10-22/README.md) | Harness correctness and compiler/workload calibration | Initial runtime flash clock was not verified. CPU compute does not select a scheduler. |
| [Clock and message experiment](../../results/clocked-handoff-2026-10-23/README.md) | Local cost of the selected Rust application/channel paths with verified clock configuration | Different primitives, executor/thread models and priorities remain. Native-kernel cost, contention, IRQ response and product behavior are not measured. |
| [Alternating alignment probe](../../results/alignment-probe-2026-10-23/README.md) | A local check that build placement materially changes this workload | Broad function alignment changes several code locations; the exact mechanism and a general cycle-penalty rule remain unproved. |
| [Ariel startup checks](../../results/ariel-qualification-2026-10-22/README.md) | Toolchain, boot, UART and autostart feasibility | No common-workload performance, radio, peripheral, deadline or power qualification follows. |

These results remain useful even where the phenomenon is known. They verify our
board, toolchain, harness and configurations. Additional identical compute runs
or more idle-channel medians would currently add less decision value than the
next comparisons below. Preserve all existing captures; do not relabel them as
newly literature-validated results.

## Next questions that can change the choice

1. **API versus native operation cost.** Qualify maintained Ariel and Zephyr
   tests; compare named native wake/yield/queue operations separately from Rust
   std and async channels. Include blocked-waiter and no-waiter cases, and equal
   versus higher-priority responders where the runtime supports them. Priority
   numbers and async tasks are not interchangeable contracts.
2. **Deadlines under useful load.** Measure external edge to ISR, ISR duration
   and ISR to application response separately, then add bounded computation and
   radio traffic. Record async yield policy, offered load, progress, overflows,
   missed deadlines and watchdog recovery. A fast idle median alone cannot
   decide whether an application remains responsive.
3. **Capability and maintenance cost.** Prove required drivers and Rust APIs,
   measure stack/heap peaks under the same work, and record first-build friction,
   patches and dependency coupling. The user's broad hardware scope remains in
   [the coverage worksheet](../hardware-coverage.md); a missing feature has no
   favorable numeric score.
4. **Delivered work and recovery.** With appropriate peers/instruments, measure
   goodput and losses together, reconnect/rejoin behavior, and energy per
   successfully delivered operation. Match actual payload and protocol role.
   Preserve radio-only and application-plus-radio cases separately.

The [revised benchmark plan](../benchmark-plan.md) applies these changes. Reuse
one reference case to qualify each new fixture, then extend only where the new
data can change a capability assessment, configuration, architecture or practical
cost. Wider range/mesh/Wasm sweeps remain available when a relevant requirement
or unexplained result makes them useful.

For a custom OS, first identify a required behavior that maintained stacks
cannot supply after reasonable configuration/API changes. Then isolate whether
the deficit needs an application change, a small runtime component, a driver
integration or a new kernel. A new kernel also owns driver/radio integration,
power management, recovery and upgrades; a small empty benchmark does not
establish that taking on those costs is worthwhile.

## Review boundary

The follow-up [research-readiness audit](research-readiness.md) identifies
remaining gaps beyond prior work: timing validity, native concurrency contracts,
rev0.2 workarounds, current maintenance/compatibility and experimental design,
plus the evidence needed for later hardware and adoption phases.

The three agents split C6 OS/timing, radio/runtime and Rust scheduling sources;
the primary agent reviewed reusable upstream suites and integrated the decision
consequences. Each audit records queries, inspected revisions and access limits.
The search did not locate a verified public dataset covering the entire proposed
same-C6 Rust and hardware matrix. That is a bounded search finding, not proof
of novelty or an assumption that unpublished work does not exist.

No firmware implementation, build or board experiment was performed for this
review. The next experimental brief should name its closest source, exact
replication or extension, decision question, permitted reuse, controls and
fixtures before code is added. This is a design requirement within the existing
authorized work, not a new approval step.
