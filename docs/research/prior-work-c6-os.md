# Prior work: C6 OS comparisons and reusable timing tests

Checked 2026-10-23 against the linked article, upstream source and publisher
records. This is a bounded search, not evidence that no other public, private or
unindexed comparison exists. Nothing in this audit was built or run on our board.

## Closest verified C6 comparison

[Kevin Leon's Qoitech article](https://www.qoitech.com/blog/esp32-c6-power-consumption-comparison/)
was created July 1 and updated July 3, 2026.

| Question | Verified article evidence |
| --- | --- |
| Comparison | Arduino, Zephyr and ESP-IDF on an unspecified C6 development board; Otii Ace supplies 3.6 V and records current, power, voltage and UART. |
| Work | Empty loop, UART reference, LED blink, CRC8 and floating-point multiplication. Active workloads: five 60-second iterations, each followed by 20 seconds without power; startup margins cropped. |
| Builds | Reports IDF “v5.5 Master,” GCC 14.2.0, `-O3`; Arduino 3.3.5 and Zephyr 4.3.99, GCC 12.2.0, `-Os`. Exact commits, board, CPU/flash clocks and complete configuration are absent. |
| Scope | Wi-Fi/BLE uninitialized; light/deep sleep and RTC-only wake excluded. Arduino float runs crashed; their mean includes failure behavior. |
| Reproducibility | Describes NumPy exports but links no study dataset or firmware. The linked GitHub repository is the generic Otii client. |

This establishes meaningful overlap for default-stack power testing. Compiler,
optimization and configuration differences prevent an isolated scheduler-cost
interpretation. Do not adopt its causal battery-life claims as our measurements.

The [author's public repository listing](https://api.github.com/users/JahazielLem/repos?per_page=100)
was also checked. The C6-named
[SBattery snapshot](https://github.com/JahazielLem/SBattery/tree/48d7fbf3ea1d1dcf4dfdde4998f7db4b728bf3d8)
contains one firmware binary, not a retrievable study implementation. This check
does not establish that study materials are unavailable elsewhere.

## Prior art directly relevant to our placement experiment

The [LLTA preprint](https://arxiv.org/abs/2609.40141), submitted September 30,
2026, evaluates an LLVM timing analyzer using physical C6 measurements and
Mälardalen workloads. Its [full text, section 5.1](https://arxiv.org/html/2609.40141v1#S5.SS1)
explains the measured instruction/memory model and explicitly limits the
silicon-model correctness claim. This is execution-time analysis, rather than a
comparison of alternative OS stacks.

Inspected source snapshot:
[`tu-dortmund-ls12-rt/LLTA` `8be6f23ca205857f5226c2848d82bad0318b44f3`](https://github.com/tu-dortmund-ls12-rt/LLTA/tree/8be6f23ca205857f5226c2848d82bad0318b44f3).

| Source finding | Consequence for this repository |
| --- | --- |
| [Timing-model notes, lines 196–198](https://github.com/tu-dortmund-ls12-rt/LLTA/blob/8be6f23ca205857f5226c2848d82bad0318b44f3/docs/targets/esp32c6.md#L196) already report control-flow costs changing by one or two cycles between builds with different code alignment. | Our placement sensitivity is a local reproduction/calibration, not discovery of an unknown C6 effect. |
| [The same document](https://github.com/tu-dortmund-ls12-rt/LLTA/blob/8be6f23ca205857f5226c2848d82bad0318b44f3/docs/targets/esp32c6.md#L314) records an instruction-fetch alignment experiment that fails to link because of the IDF `.flash_rodata_dummy` reservation. | Do not describe every proposed upstream alignment probe as an executed experiment. |
| [Span charging implementation](https://github.com/tu-dortmund-ls12-rt/LLTA/blob/8be6f23ca205857f5226c2848d82bad0318b44f3/lib/Targets/RISCV/FlashCacheAnalysis.cpp#L79) limits word-span charging to loads/stores. | Its data-access span penalty does not establish the cause of our instruction-placement delta. |

Reuse the linked-ELF inspection principle and compare actual addresses/bytes,
branch sites, memory placement and cache conditions before assigning a cause.
The local result still helps prevent an OS ranking driven by link layout. The
specific physical explanation remains an experimental question.

Earlier [cycle-counting work by Timon Van Overveldt, October 30, 2023](https://ctrlsrc.io/posts/2023/counting-cpu-cycles-on-esp32c3-esp32c6/)
demonstrates C3/C6 hardware-counter timing and variable loop/branch costs. It
provides a useful timer cross-check; it does not compare kernels. The related
[onsdagens interrupt benchmark](https://github.com/onsdagens/esp32c3-rt-benchmarks/tree/461b075034b6f7de2cb5e6559c7f0f7da44a46d9)
compares direct vectoring with HAL dispatch on **C3**, explicitly requiring
adaptation for other chips. Its latency figures cannot be relabeled C6 results.

## Reuse Ariel's existing scheduler probes

Inspected upstream snapshot:
[`ariel-os/ariel-os` `13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9).
These are existing tests to qualify and reproduce, not evidence of a completed
upstream C6 OS comparison.

| Existing source | Semantics and reuse |
| --- | --- |
| [`bench-sched-yield`](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks/bench-sched-yield/src/main.rs) | Two threads repeatedly yield at equal priority. One measures 10,000 calls; output divides the mean by two to estimate switch cost. Useful native-thread baseline. |
| [`bench-sched-flags`](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks/bench-sched-flags/src/main.rs) | Two threads exchange IDs, then signal/wait using thread flags; measures 1,000 iterations. Useful native synchronization baseline. It does not transfer and verify our sequence-number payload. |
| [`ariel-os-bench` ESP adapter](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-bench/src/esp.rs) | Uses SYSTIMER unit 0 around the whole loop, checked subtraction, and truncating division by iteration count. Result units are system-timer increments, not automatically CPU cycles or nanoseconds. |

Their Laze definitions select threading and benchmarking; the C6 configuration
provides the benchmark capability. The example READMEs demonstrate an nRF52840
command, so C6 admission still requires a successful C6 build and physical run.
Retain original semantics/results when reproducing these; a separate common
payload test can then answer the Rust application-path question.

## What remains worth measuring

The project can serve an engineering decision without being a novel benchmark
suite. Keep separate evidence for these questions:

1. **Replication:** can known native yield/signaling tests run reliably on our
   C6 with pinned sources, readable raw output and verified clocks?
2. **Application paths:** what latency distribution, deadline behavior and
   memory cost do equivalent validated Rust message exchanges produce through
   each runtime's actual API? Different channel/priority models must remain
   visible; a round trip is not a single context switch.
3. **Confounders:** how sensitive are results to code placement, timer
   resolution, priorities and background work? The existing alignment probe is
   evidence that this matters locally.
4. **Practical fit:** which stacks meet the required radio/peripheral/power
   behaviors, with what implementation and maintenance effort? An unsupported
   feature is a decision constraint, not a zero-valued benchmark score.

For power work, reproduce a simple baseline before expanding it: characterize
the carrier/USB/LED current boundary, isolate measurement synchronization, then
report both default configuration and deliberately configured sleep modes.
Current averaged over a crashed workload is not a successful energy-per-job
measurement. The present repository has no power results.

## Search boundary and verification record

Searches on 2026-10-23 included `"ESP32-C6" "operating systems" benchmark
comparison`, `"ESP32-C6" "Embassy" "benchmark" github`, `"ESP32-C6"
"RTIC" benchmark comparison`, `"ESP32-C6" "context switch" benchmark`,
`"ariel-os" "benchmark" "thread"`, and the exact Qoitech URL. Results were
followed to primary author/publisher pages, source files and GitHub API trees;
secondary aggregators were leads only.

This pass inspected Qoitech's full article and outbound links; the author's
public repository inventory; LLTA's timing article sections, notes and relevant
implementation; Ariel's benchmark implementations and build declarations; and
the earlier counter/interrupt work above. It found no verified same-C6,
shared-Rust-workload comparison covering every stack in our plan. That is a
bounded finding, not an exhaustive novelty claim. Radio, Wasm and cross-chip
Rust studies are covered by the accompanying prior-work audits.
