# Prior work: C6 radios and mesh

Reviewed 2026-10-23. This is an artifact review, not a replication: no upstream
firmware was built, flashed, or benchmarked for this review. Full text, source,
configuration, and representative measurement files were inspected where
available. Numerical findings from other boards are not measurements of our
attached board. These works establish substantial overlap; this review does not
establish that our proposed combination is unique.

## Verified sources and frozen artifact versions

| Work | Verified evidence | Version inspected |
| --- | --- | --- |
| Benjamin Poli and Anton Johansson, *Performance evaluation of wireless communications protocols on ESP32-C6*, University West bachelor thesis | Full thesis, firmware fragments, Thread instructions, result logs and current workbook | Thesis dated 2026-05-22; [repository `03c1205878af427709426774294abba0a2dbfb1c`](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/tree/03c1205878af427709426774294abba0a2dbfb1c), latest commit dated 2026-08-04 |
| Scott Rapson / Electric UI, *Benchmarking latency across common wireless links for microcontrollers* | Full article and companion firmware, configuration, raw edges, packet captures, analysis | Article metadata dated 2024-02-09; [repository `a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577`](https://github.com/Scottapotamas/embedded-wireless-latency-eval/tree/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577), latest commit dated 2024-02-10 |
| Massimo Nobile, Fabio Palmese, Antonio Boiano, Alessandro E. C. Redondi and Matteo Cesana, *Zigbee vs. Matter over Thread: Understanding IoT Protocol Performance in Practice* | Additional full-text C6 mesh study found during review | [arXiv `2603.04221v1`](https://arxiv.org/html/2603.04221v1), 2026-03-04; no public experiment repository located in the inspected paper |

Locally downloaded PDF SHA-256 values, for identifying the exact reviewed copies:

```text
University West thesis: 88e24942f7b7d73f43a64c8ce4bf5749ff0349a75591c16c317ccdee7cfbd625
```

## Direct C6 protocol comparison: Poli and Johansson

The [thesis, sections 3 and 5.5](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/blob/03c1205878af427709426774294abba0a2dbfb1c/Performance%20evaluation%20of%20wireless%20communications%20protocols%20on%20ESP32-C6.pdf)
compares Wi-Fi 4, Wi-Fi 6, Thread and Zigbee between two C6 devices outdoors at
0, 10, 20 and 30 m. Each communication case has five two-minute runs, with
baseline and added traffic. Current is measured separately for 30 seconds,
five times per role/state, with a Yojock KWS-2303C USB meter. Analysis averages
repeated runs. The text acknowledges manual Thread command synchronization,
antenna/environment uncertainty, lack of mesh evaluation, and excluded sleep/TWT.
It is an active board-current comparison, not energy per successful operation
or a scheduler benchmark.

The inspected repository contains **281 files under `Results/`**, mostly RTF
console transcripts, plus an XLSX current workbook. This is appreciably better
than plots alone. It is not a complete locked project: the inspected tree lacks
`sdkconfig`, dependency locks, CMake projects and a top-level license. Record a
license decision before copying its code into our maintained implementation.
Use [result files](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/tree/03c1205878af427709426774294abba0a2dbfb1c/Results)
as external comparison data with attribution, not our result set.

Source inspection exposes differences that must be resolved before reproducing
the paper's exact cases:

| Artifact | Inspection finding | Consequence for our experiments |
| --- | --- | --- |
| [Wi-Fi latency client](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/blob/03c1205878af427709426774294abba0a2dbfb1c/Wi-Fi%20code/Client/Wi-Fi-Latency-Client.c) | A 64-byte local buffer holds `PING <seq>`; `sendto` uses `strlen(buf)`. Received content/source/sequence is not checked against the request. Delay is one second after receive/timeout. | A buffer capacity does not prove a 64-byte wire payload. Our packet header must declare and validate actual wire length and sequence; record offered rate separately from completion-paced rate. |
| [Wi-Fi throughput client](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/blob/03c1205878af427709426774294abba0a2dbfb1c/Wi-Fi%20code/Client/Wi-Fi-Throughput-Client.c) | Published sender uses a 1,400-byte buffer, versus the thesis's 1,448-byte description. | Freeze and log actual payloads. Do not assume the source snapshot exactly reconstructs every thesis run. |
| [Zigbee latency server](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/blob/03c1205878af427709426774294abba0a2dbfb1c/Zigbee%20code/Server/Latency%20Server.c) | Timing spans toggle submission and a subsequent read-attribute response; each successful callback logs and creates a delay task before the next request. The shown wait state has no explicit response deadline. | This is a complete application interaction with stack/locking effects, not a raw radio RTT. Preserve missing responses and watchdog outcomes; do not calculate latency only for successes without reporting loss. |
| [Thread instructions](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/blob/03c1205878af427709426774294abba0a2dbfb1c/Thread%20configuration%20steps/README.md) | OpenThread CLI ping and UDP/IPv6 iperf commands are provided; target bandwidth includes `-b 100k`. | Resolve that particular iperf implementation's units from its pinned help/source before converting the literal to bytes/s. Automate synchronized start markers rather than copying manual launch timing. |

**Reuse:** scenario structure, distance/traffic axes, observed failure cases and
external logs. **Replicate only if useful:** one short-range, fixed-payload IDF
baseline to validate our radio fixture. **Add for our decision:** cross-runtime
availability and implementation effort, matched application semantics, receiver
goodput and loss, reconnect behavior, and latency under simultaneous local work.
Select distance sweeps only when range or reliability is the decision question.

## Raw C6 802.15.4: Electric UI

The [article](https://electricui.com/blog/latency-comparison) measures one-way
stimulus-to-validated-reception latency for 12, 128 and 1,024-byte messages.
External GPIO timing uses a Saleae at 100 MS/s. C6 coverage is specifically a
pair of ESP32-C6-MINI boards using raw ESP-IDF 802.15.4, comparing requested
acknowledgments and unacknowledged transmission. Its Wi-Fi and Bluetooth ESP32
results are not C6 measurements. Fragmentation and connection/configuration
effects are central to the article; it does not compare C6 operating systems.

The [companion repository README](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/README.md)
describes roughly 1,000 trigger intervals per capture, a signal generator,
spectral checks and a typical one-metre separation. Its
[MIT license](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/LICENSE)
permits code reuse subject to its notice conditions. Especially useful assets:

| Pinned asset | Reuse and caveat |
| --- | --- |
| [C6 firmware and raw edges/PCAPs](https://github.com/Scottapotamas/embedded-wireless-latency-eval/tree/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/firmware/esp-802154) | Existing packetization and acknowledgment cases, timing CSVs, and `1kb-no-ack.pcapng` / `1kb-with-ack.pcapng` support checking airtime and software contributions separately. This is a reference fixture; no need to invent packet formatting from scratch. |
| [C6 `sdkconfig`](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/firmware/esp-802154/sdkconfig) | Header identifies IDF 5.1.2, CPU 160 MHz, flash 80 MHz and performance optimization. The nearby README's Docker `release-v4.4` instruction differs. Select a supported pinned SDK and document migration; do not advertise the old command as a verified C6 build recipe. |
| [`lrpwan_main.c`](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/firmware/esp-802154/main/lrpwan_main.c) | Channel 11, GPIO19 input/GPIO18 output, event queues, callback processing and receive allocation are visible. These software costs belong to its end-to-end measurement. Match wire behavior and disclose adapter differences in Rust ports. |
| [Processed C6 samples](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/analysis/esp32C6-802154-results.csv) and [edge conversion](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/analysis/saleae-latency-log-cleanup.R) | Preserve upstream originals when reanalyzing. The converter pairs each eligible start with a following completion, without an explicit sequence/deadline contract. Add deliberate missing/late-edge fixture cases before adopting its pairing method. |

**Our useful replication:** calibrate a common GPIO trigger/completion fixture
with one payload, then check a fragmentation boundary and ACK/no-ACK behavior.
**Our added decision question:** how much does application latency and delivery
reliability change when the same radio use case runs alongside periodic
peripheral work and CPU load on each viable runtime? Repeating every radio and
buying all listed instruments is outside the current need.

## Additional direct C6 mesh study

[Nobile et al., version 1](https://arxiv.org/html/2603.04221v1) uses six C6 nodes,
mesh/chain topologies, passive PCAP capture, idle/control traffic, latency,
throughput and router failure. The setup includes Home Assistant, different
Zigbee/Thread controller radios, ESP-ZIGBEE-SDK 1.6.5, ESP-MATTER 1.4 and
IDF 5.4.1 for the CLI transport tests. Application-control and lower-layer
ping/iperf experiments are distinct. It is stronger precedent for topology
and recovery tests than a two-node range sweep. No public experiment data/code
link was located in the inspected full text; tables/figures are not raw samples.
Do not attribute a complete ecosystem comparison solely to identical C6 end
nodes when controllers/stacks differ.

Our immediate program should borrow the recovery/traffic questions, then defer
mesh scaling until enough nodes exist and the application needs mesh. For an OS
choice, first establish that the required radio stack works through the proposed
Rust adapter and remains responsive while other application tasks execute.

## Changes this research should make to our test program

These are recommendations derived from the review, not claims that any future
test has passed:

1. Make **functional radio support** a prerequisite before latency rankings.
   Record absent, build-only, on-board functional and measured states separately.
   A fast executor without the needed radio path may fail the product decision.
2. Choose one fixed, checked application interaction across runtimes. Record
   payload bytes on wire, request identity, response identity, deadlines, loss,
   retries, queue depth, task priorities, radio parameters and power policy.
   Match the semantic contract before comparing implementation costs.
3. Preserve an **IDF reference case and external timing cross-check**. Existing
   upstream examples and Electric UI's published artifacts provide independent
   checks; our internal 1 µs timers need their own observer calibration.
4. Report successful deliveries and failures together. Keep per-boot and
   per-environment distributions, rather than treating thousands of adjacent
   packets as thousands of independent environmental repetitions. Predeclare
   delivery/deadline requirements before looking for winning medians.
5. Focus new measurements on contention: periodic deadlines with radio activity,
   bounded message queues, wake/reconnect behavior and memory high-water marks.
   Our existing compute loop is a toolchain/configuration control; current
   channel RTTs measure particular Rust APIs and task arrangements. Neither
   supplies these missing product properties.
## Search scope and remaining limits

The review followed the supplied radio links, their thesis/artifact
references, and targeted searches for C6 protocol comparisons and C6 radio
measurements. It located additional primary full text and repositories above.
This is a bounded search, not a systematic review of every publication, and
absence of a located suite does not prove absence of prior work. Future phase
plans should cite the specific source they replicate or extend and state which
decision the new measurement can change.
