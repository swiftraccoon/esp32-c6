# ESP32 C6 benchmark plan

The aim is to choose a platform for real applications and establish whether a custom Rust runtime would solve a measured problem. Run feature qualification first, controlled performance tests second, and a realistic application last. A platform without the required radio, recovery path, or maintainable Rust integration cannot win by having the smallest empty image.

This is the wider proposed protocol. The [first physical compute and observer pilot](../results/initial-2026-10-22/README.md), [clock/message experiment](../results/clocked-handoff-2026-10-23/README.md), separate [alignment probe](../results/alignment-probe-2026-10-23/README.md), [Ariel startup qualification](../results/ariel-qualification-2026-10-22/README.md) and [matched IDF queue API study](../results/idf-queue-api-2026-10-23/RESULTS.md) are complete with their own narrower contracts. They do not establish the later networking, power or interference measurements. Hardware identity is recorded in [hardware](../../../docs/hardware.md); comparisons and test order are in [comparison](comparison.md). The [hardware coverage worksheet](hardware-coverage.md) expands testing to every subsystem the board exposes. Review the [coding standard](../../../docs/coding-standard.md) before implementing any harness.

Native-suite evidence has narrower admission limits. Ariel retains twelve expected outputs with partial
observation-window evidence; its derived yield value does not validate switch
count. Zephyr emits 47 rows per boot but admits **0/3** under frozen v1 because
the required header is compiled out; all boots also report ROM image-SHA
warnings. Its selected Rust hello failed the build gate. Preserve those attempts
and resolve acceptance/image defects before another native comparison.

## Prior work and experiment admission

The [primary-source review](../../../docs/research/prior-work.md) establishes substantial
overlap with existing C6 and embedded Rust experiments. Consult its detailed
artifact audits and [reuse inventory](../../../docs/research/benchmark-reuse.md) before adding
workloads. Existing native scheduler tests and radio timing artifacts are the
default starting points. A useful replication validates our board or fixture;
an extension must answer an unresolved platform/configuration question.

The [follow-up research briefs](research/research-readiness.md#follow-up-research-closure)
resolve timing, concurrency, rev0.2 exposure, maintained inputs and experimental
design for the [same-IDF API comparison](experiments/native-queue-comparison.md).
That study is now complete: raw SYSTIMER/backend qualification and the matched
std control passed in all 24 scheduled boots, with twelve paired blocks and no
retries. The [results](../results/idf-queue-api-2026-10-23/RESULTS.md) support a
native queue API remedy on this configuration; they remain separate from
native-suite averages and historical whole-microsecond results. A manifest
version alone does not establish the development thread API in published
esp-rtos 0.4.0.

Each future experiment needs a short committed brief before implementation:

| Required item | What it must establish |
| --- | --- |
| Decision question | Which capability, configuration, API, architecture or maintenance choice could change with the result? |
| Closest prior test | Primary source and inspected revision; what is being replicated and what differs? |
| Reuse choice | Run unmodified, port under permitted terms, or independently implement a published contract in Rust; state why new code is needed. |
| Observable contract | Actual inputs/wire bytes, correctness check, timed endpoints, task/executor topology, queue/priority behavior and completion/failure rule. |
| Controls and measurement | Generated configuration plus active readback, timer units/resolution/observer cost, code placement and external cross-check where needed. |
| Repetitions and stopping | Boot/environment units, order, exploratory versus fixed design, failures/outliers, uncertainty needed for the decision and what would justify more runs. |
| Fixtures | Available pins, peers and instruments; distinguish a missing fixture from a missing driver. |

Before freezing acceptance, reconcile required output with the actual source,
print configuration and ELF, and qualify the selected image format/integrity
path. Record what a reviewer can reproduce from artifacts versus operator
observations. A source-derived expectation or terminal success banner does not
override a failed rule. Metric-level endpoints and corrections must be audited;
Zephyr's default has **19 observer-adjusted and 28 unadjusted rows**, as mapped
in the [source audit](research/zephyr-native-audit.md).

Reconcile paper prose with actual source and raw records before treating a
published number as a reference. Unresolved payload, task-topology, clock or unit
differences prevent direct numerical equivalence. Preserve the original artifacts
and identify our changed case; do not silently repair and relabel a replication.
Public source access alone is not a blanket license to copy it.

Two timing categories remain separate. **Native-operation tests** measure a
named yield, notification, queue or preemption path, with and without waiters.
**Rust application-path tests** measure validated complete work through the
chosen libraries/runtime. The existing 7 µs/62 µs round trips are the latter;
the libraries and priority arrangements differ. Neither that ratio nor a
round trip divided by two establishes kernel switch cost.

For native threads, test equal priority and a higher-priority responder where
supported, recording actual policy and priorities. For async work, separately
identify joined futures in one task, separately spawned tasks and distinct
priority executors. Do not manufacture a portable priority number or equate
these scenarios. Reuse native tests before writing replacements.

## Rust implementation approach

Keep the workload, payload generation, checksums, statistics, and host collection in Rust. Design a shared `no_std` workload crate with bounded buffers and deterministic inputs, then use a small adapter per platform for clocks, synchronization, storage, and transport.

The native kernels in ESP-IDF, Zephyr, NuttX, RIOT, and RT-Thread contain C or C++. Using them does not require writing the application in those languages. ESP-IDF already has an established Rust application path; other kernels need individual Rust compile and link probes. If a native C adapter or a C-only baseline becomes necessary, discuss the specific reason before implementing it. Do not quietly replace the common Rust workload with different C algorithms.

Embassy tasks are cooperative within an executor, RTIC uses interrupt priorities, and preemptive kernels schedule threads. Benchmark a common observable behavior such as event-to-response latency; report each scheduler's native operations separately. A task wake and a thread context switch are different measurements.

## Qualification gates

| Gate | Evidence required before moving on |
| --- | --- |
| Reproducible build | Pinned revision, dependency lock, compiler versions, clean build instructions, configuration and ELF archived |
| Physical board boot | Correct flash layout, console, stable monotonic timer, repeated reset and power-cycle recovery |
| Rust application | Actual C6 compile, link, boot, timer and interrupt smoke test; an ISA-compatible Rust target alone is insufficient |
| Peripheral minimum | GPIO plus the application's real bus and storage drivers, with known wiring |
| Connectivity minimum | Required Wi-Fi, BLE, or Thread operation on this board; distinguish raw 802.15.4 from a working Thread stack |
| Recovery | Recover from application crash and watchdog; demonstrate a recoverable update path when updates are required |

Record `unsupported`, `blocked`, `failed`, and `not_run` separately. Unsupported Wi-Fi is neither zero throughput nor a fast network result.

## Controlled configurations

Use the same physical board and supply. Fix the HP CPU at 160 MHz for the first comparison, disable dynamic frequency scaling and automatic sleep, and record the actual clock setup. Later run a separate energy configuration with each platform's supported power policy. ESP-IDF's power manager can change CPU and APB clocks and use automatic light sleep, so defaults would confound a scheduler comparison. [C6 power management](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/system/power_management.html)

Create three profiles per candidate where supported:

| Profile | Purpose | Enabled features |
| --- | --- | --- |
| Minimal | Boot, memory, computation, scheduling | Console buffered or quiet during timing; all radios off |
| Connected | Application and radio stack cost | One specified radio, same payload and security mode |
| Product | Real operating behavior | Recovery, logging, update support, intended sleep policy |

Pin CPU ISA/ABI, Rust and C compilers, optimization, LTO, panic behavior, allocator, logging, timer resolution, tick rate, thread priorities, queue depth, stacks, heap, flash mode/frequency, partitioning, security settings, and whether code executes from flash or RAM. Exact compiler parity across all kernels may be impossible: publish the differing toolchains and describe the result as a whole-platform comparison. Include common-code microbenchmarks only when the Rust code and its generated build settings match.

Use release builds. Measure debug/logging overhead as a separate experiment. Keep safety checks and watchdog choices visible rather than silently removing them to improve scores. Read back runtime CPU and flash sources/dividers; image headers and boot configuration alone do not establish the final clocks. The first pilot's flash-clock discrepancy is recorded in its [errata](../results/initial-2026-10-22/errata.json).

Inspect compute body bytes **and placement**: function entry, loop target,
backedge, full-width instruction alignment and cache-block boundaries. The
alignment probe changed a median by about 25 microseconds without changing the
54-byte workload. Preserve original observations and label exploratory builds
separately. A broad compiler-alignment option changes more than one function;
record its scope instead of claiming an isolated instruction-fetch mechanism.

## Measurement suite

The sample counts below are initial targets, not statistical guarantees. Save raw samples and the sample count for every reported percentile.

| Test | Method | Report |
| --- | --- | --- |
| Boot | At least 30 cold boots and 30 resets; external reset/power reference to GPIO ready marker; network readiness is a second marker | Median, p95, max, failures; separate cold and warm reset |
| Flash footprint | Parse linked ELF/map and final image with identical feature profile | Text/rodata/data/bss, image bytes, bootloader and partition overhead separately |
| RAM | Record static allocation, reserved stacks and heap, runtime free/peak and stack high-water marks under the same load | Bytes by category; methodology for each OS; avoid counting static buffers twice |
| Common compute | Deterministic Rust integer/CRC/copy/serialization workload with verified output; no allocation or I/O in timed region | ns/op or cycles/op, checksum, code size; this primarily checks toolchain and interference |
| Event response | Physical interrupt edge to task-produced GPIO response under idle, compute, and radio load | At least 10,000 events; p50/p95/p99/max, missed deadlines |
| Timer jitter | Periodic deadlines at selected rates, using absolute scheduling to avoid drift; retain missed periods | Distribution of signed error and absolute lateness, drift, misses |
| Task communication | Same producer/consumer message size and queue depth; include ready-to-run timing and saturation | Latency, messages/s, drops, CPU budget; native scheduler primitive named |
| Allocation | Only for an allocating workload; identical allocation trace and lifetime distribution | Peak live bytes, allocator overhead, failures, fragmentation under churn |
| Wi-Fi | Same AP/channel/security/RSSI range, payload, peer, direction, and TCP/UDP setup; 5 or more 60 s trials | Goodput, packet loss, RTT distribution, CPU/RAM, reconnect time |
| BLE | Same PHY, connection interval, MTU, data length, peer and notifications | Application goodput, loss, latency, reconnect and discovery time |
| Thread | Same border router, topology, role, channel, IPv6/CoAP payload and security | Join time, end-to-end latency/loss, recovery, role and sleepy polling interval |
| Energy | Instrument the supply rail at known voltage over a complete observation interval, including sleep, retries, failed sends, reconnects and idle time | Total joules divided by reports confirmed delivered; delivery ratio, active/idle/sleep current, wake time, sample trace; undefined if none delivered |
| Reliability | Sustained mixed load and controlled AP loss, peer loss, malformed application input and watchdog faults | Failures, recovery time, resets, leaks, lost messages over a declared duration |
| Developer effort | Time the first clean build and first functioning application; log blockers and local patches | Person-hours, build times, patch burden, dependency churn |

Use multiple independently booted runs, with randomized or alternating platform order. Treat events within a boot as correlated, not as thousands of independent experiments. Report run-to-run variation as well as pooled latency samples. A maximum is only the largest observed value, never a proof of a worst-case bound. Increase repetitions when uncertainty affects the decision.

Next stress tests use absolute deadlines and record signed error, lateness,
missed/coalesced events, queue overflow, per-task progress and watchdog outcomes.
Declare compute chunks and async yield points; retain a timeout for a
non-yielding case. Sweep offered load and task count before assuming idle
handoff results describe an active device. Example deadline sweeps are
characterization, not product pass/fail requirements until those are supplied.

For repeated boots, use a device identifier generated once at startup or
independent reset/startup evidence to substantiate boot repetitions. A host
challenge can additionally detect stale captures; a new challenge response
alone does not prove a reset. Existing captures have host-requested RTS resets
but no device nonce; their disclosed duplicate quantized transcripts remain
valid under that narrower recorded evidence. Do not deduplicate samples or
retroactively claim a stronger protocol guarantee.

## Timing and radio pitfalls

Calibrate measurement overhead with an empty operation. Batch short operations, preserve results so optimization cannot remove them, and avoid logging within a timed section. Record warm versus cold flash-cache behavior. Cycle counts only convert to time correctly with known clock behavior and counter rollover handling; cross-check short timings using an external logic analyzer. Espressif specifically documents timer overhead, cache effects, and external timing. [Performance measurement guidance](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-guides/performance/speed.html)

A batch average can resolve an average below the single-read timer quantum; it
cannot recover individual-event tails. Qualify higher-resolution timing before
reporting fine scheduler percentiles. Keep raw gross timings and observer
distributions; any corrected estimate is an additional labeled calculation.

Use GPIO timing without assuming an unverified LED pin. USB/UART receive timestamps include buffering and host scheduling, so they are suitable for collecting records rather than proving microsecond interrupt latency. Separate time inside the ISR from ISR-to-task dispatch and from total application response.

Wi-Fi, BLE, and 802.15.4 share one RF path on the C6. Test each radio alone first, then supported combinations with declared traffic. Thread router and sleepy-end-device results are not interchangeable; a border-router workload also has different needs. Espressif documents combinations with unstable or unsupported coexistence, including limits relevant to single-chip gateways. [C6 RF coexistence](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-guides/coexist.html)

For Wi-Fi, an SDK iperf example is useful within that SDK, but ensure peer protocol compatibility before comparing platforms. If iperf versions or implementations differ, use the shared Rust application protocol and payload for the cross-platform result.

For all radio cases, record actual payload length on the wire, sequence and
response validation, offered versus completion-paced traffic, loss, retries,
deadlines and role. Use one known IDF/external-timing reference to qualify a new
fixture before extending it across runtimes. Prior raw-802.15.4, Thread, Zigbee
and Matter studies supply methods; their different packetization, controllers
and environments do not provide interchangeable OS baselines. Introduce matched
periodic application load and reconnect/rejoin tests before a broad distance or
mesh sweep. Keep successful delivery and failure statistics together.

## Tools and equipment

| Available now | Additional equipment or setup |
| --- | --- |
| DevKitC-1-N8, 8 MB flash; published header map, GPIO8 RGB LED, BOOT/RESET; Rust/Python/esptool host tools | Record PCB revision and choose unused pins for each wired fixture |
| CP2102N UART logs and ELF/map inspection; separate native USB-C serial/JTAG connector available | Native USB qualification; logic analyzer or oscilloscope plus external stimulus generator or second MCU for interrupt timing |
| Local development host | Controlled 2.4 GHz AP and wired traffic peer |
| Chip BLE and 802.15.4 capability | BLE peer and Thread border router or second supported radio board |
| USB power and onboard J5 module-current jumper | Calibrated power profiler or shunt measurement; declare module versus whole-board boundary |

The [vendor board map](../../../docs/hardware.md) documents connectors, LEDs, buttons and
measurement points. Their availability does not establish successful use.

A USB current meter generally cannot resolve short wake events or separate regulator, LED, and UART-bridge consumption. Report whole-board energy if that is what the instrument measures.

## Results and reproducibility

Create one directory per platform revision and run under `results/`. Copy [run metadata](../results/run-template.json), fill the required experiment conditions, and retain raw timing/packet/power records next to the firmware ELF, linker map, build log, source revision, lockfile, configuration, and SHA-256 hashes. Keep credentials and full-flash backups outside the repository.

Use the same protocol version for every adapter. Correctness checks, counter rollover, lost samples, resets, and measurement errors must be visible. Preserve all trials, including failures. Repeat only when a stated cause invalidates a run, and retain the invalidation reason.

Publish separate capability and measurement results across the full hardware scope. Once an application supplies requirements, use the optional [decision criteria](decision-criteria.csv) and [scoring method](comparison.md#decision-method) to choose gates and weights before examining performance numbers. Keep hard requirements as gates rather than allowing a high score elsewhere to compensate for a missing capability.

## Optional established compute benchmarks

[CoreMark](https://github.com/eembc/coremark) and [Embench IoT](https://github.com/embench/embench-iot) are useful secondary toolchain and CPU checks. They do not establish OS scheduling quality, radio performance, or product suitability. Their reference implementations introduce C; discuss their value before adding them, retain their validation procedures and licenses, and keep their results separate from the shared Rust workload.

## Next implementation milestones

The initial compute, clock/channel and alignment datasets calibrate the local
harness. The matched IDF study then completed the same-kernel API comparison:
24 passing boots, 24,000 checked round trips, nominal boot medians 62.875 µs
(std) and 19.9375 µs (native). This measures whole instrumented paths with
ordinary release placement, not isolated scheduler cost. Review
[existing Rust wrappers](research/idf-native-api-options.md) before adopting a
general-purpose native API. Further identical compute or idle-channel runs
currently offer less decision value than these gated stages:

1. Resolve the native-suite output/image defects and per-row semantics;
   version any corrected acceptance profile before prospective capture. Preserve
   Ariel's operator/archival distinction and add peer-progress proof where a
   switch claim needs it. Resolve a specific Zephyr Rust generator/build remedy
   before attempting UART or a Rust benchmark; no successful Rust image exists.
2. Define one matched workload and a reviewed profiling method before a load
   sweep. Distinguish task-inclusive wall counters from exclusive CPU/ISR work,
   offered load from completed goodput, and reservations from stack/heap peaks.
   An idle-task share is not physical sleep or energy evidence. Qualify attribution,
   overhead, clock behavior, overflows and stopping rules first; the
   [utilization audit](research/utilization-audit.md) identifies the missing
   evidence. Do not automatically sweep every OS or infer capacity from idle RTT.
3. Extend API/priority, RTIC and hardware cases only where their contracts can
   change a decision. An esp-rtos thread variant needs a deliberate released or
   development API choice. External IRQ, hardware-offload, radio/sleep/energy
   and recovery experiments need appropriate fixtures and separate admission.
4. Expand across [all exposed hardware](hardware-coverage.md) and broader
   kernels/runtimes using the same admission brief. Radio, power, range and
   mesh work wait for their fixtures and decision questions, not for a claim
   of novelty. Evaluate power over delivered work and declared sleep policy.

Bring in NuttX and RIOT for POSIX/networking questions and RT-Thread or hobby
kernels for their supported capabilities. Treat Wasm as an additional runtime
layer when dynamic-module requirements justify it. A custom kernel needs a
documented unmet requirement after existing configuration, API and component
options are tested. Full hardware scope and the Rust language agreement remain
unchanged; discuss a necessary authored C/C++/assembly fallback before coding.
