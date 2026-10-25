# Prior work and reusable experiments

This catalog records sources inspected on 2026-10-23 for C6 radio, power,
runtime and scheduling work. Use the linked audits for exact revisions,
configuration mismatches, reuse permissions and source-access limits.

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

- [C6 OS and timing prior work](../../projects/os-evaluation/docs/research/prior-work-c6-os.md).
- [C6 radios and mesh](prior-work-c6-radio.md).
- [C6 WebAssembly experiments](../../projects/os-evaluation/docs/research/prior-work-wasm.md).
- [Rust scheduling papers and source audit](../../projects/os-evaluation/docs/research/prior-work-rust-scheduling.md).
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
