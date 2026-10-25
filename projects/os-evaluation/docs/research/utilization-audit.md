# Audit of utilization and capacity feedback

Reviewed **2026-10-23**, from retained data and pinned source, without a build,
board operation or implementation change. The feedback's central assessment is
supported: our evidence characterizes selected elapsed-time paths much better
than CPU utilization, sustainable capacity, memory peaks or energy efficiency.
It does not identify the platform that uses the C6 most efficiently.

The proposed controlled-load experiment is useful **conditionally**, after
repairing the known source/acceptance/image-qualification problems and defining
the accounting method. It is not an already established next benchmark or a
reason to turn every missing hardware measurement into one large new harness.

## Claims checked against existing evidence

| Feedback | Finding and boundary |
| --- | --- |
| No CPU busy/idle, task-time or interrupt-time breakdown | **Supported.** The archived IDF queue configuration has `CONFIG_FREERTOS_GENERATE_RUN_TIME_STATS` and `CONFIG_FREERTOS_USE_TRACE_FACILITY` disabled. No retained protocol-3 record supplies task runtime, idle residency or IRQ occupancy. Faster RTT is elapsed-path evidence, not evidence of fewer CPU cycles or lower energy. |
| No sustainable-capacity/load sweep | **Supported.** Both queue adapters have capacity one. Main sends one checked request and waits for its reply, with worker-readiness probes before the next sample. There is no independent offered-rate, depth, contender, backlog or application-deadline sweep. Its one-second invalid-duration bound is an admission guard, not a product deadline or a miss-rate experiment. |
| Memory reservations differ from utilization | **Supported.** The queue hello records 32 KiB main / 8 KiB worker stack requests and a 48,000-byte sample slice: 2,000 records at 24 bytes each, including padding. Allocator metadata/capacity can add cost. Neither those sizes nor ELF sections are observed stack/heap peaks, fragmentation or remaining usable application memory. Stack canaries are enabled, but no overflow report is not a high-water measurement. |
| Placement/cache behavior is not matched by clocks | **Supported, with a causal limit.** The alignment probe observes original 308 µs versus aligned 282–283 µs medians, matching clock readbacks and identical 54-byte workload instructions. The compiler option changes placement broadly. This supports a placement-sensitive result; it does not isolate a particular cache event, fetch penalty or cache-miss count. |
| Offload, contention and energy remain missing | **Supported within the retained experiment scope.** The datasets do not measure DMA/accelerator utilization, LP-core work, peripheral/radio contention or joules per completed operation. The WFI check establishes timer progress over its bounded wait, not idle power, sleep residency or energy efficiency. |
| Resolution is not accuracy | **Supported.** Protocol 3 retains raw ticks at a nominal 16 MHz scale, or 62.5 ns per tick. Earlier timing APIs quantize to whole microseconds. Arithmetic conversion and fractional averages do not establish independent oscillator accuracy, timing uncertainty or arbitrary decimal precision. |

Evidence: [archived IDF configuration](../../results/idf-queue-api-2026-10-23/artifacts/sdk/sdkconfig),
[first native run](../../results/idf-queue-api-2026-10-23/runs/s1-b01-02-native/run.json),
[queue sampling source](../../firmware/esp-idf/src/bin/queue-api.rs),
[alignment experiment](../../results/alignment-probe-2026-10-23/README.md),
and [timing contract](timing-and-concurrency-contract.md).

## Reproduced first-native-run calculation

The exact retained [raw transcript](../../results/idf-queue-api-2026-10-23/runs/s1-b01-02-native/serial.raw)
contains 1,000 `message-roundtrip` records, indices 0–999. Integer parsing of
their `start_tick` / `end_tick` values gives:

| Quantity | Exact raw ticks | Nominal milliseconds at 16 MHz |
| --- | ---: | ---: |
| Sum of the 1,000 `end_tick - start_tick` intervals | 318,964 | 19.93525 |
| Last end minus first start: 33,574,723 − 33,182,766 | 391,957 | 24.4973125 |
| Difference: the sum of 999 between-sample gaps | 72,993 | 4.5620625 |

The feedback's **19.94 ms / 24.50 ms** rounded example is correct. Gaps range
from 72 to 245 ticks. The sampling source places checksum/duration checks,
sample storage, loop work and the next readiness probes outside the RTT bracket.
The transcript has no task/ISR attribution or physical sleep trace, so those
gaps cannot be labeled CPU idle time.

The reciprocal mean RTT is about **50,162 operations/s**, while the finite
batch's observed completions divided by its first-start/last-end span are about
**40,821 operations/s**. Both describe this particular closed loop. Neither is
a sustainable-capacity result under independently specified arrivals, competing
work, bounded backlog or a deadline. The displayed decimals are exact scaling
of integer observations under the nominal clock assumption, not accuracy claims.

## Zephyr method and allocator claims

The [separate native source audit](zephyr-native-audit.md) maps all 47 rows.
Only **19** subtract an observer estimate; **28** do not. The uncorrected set
mixes eight whole-loop spans with twenty sums of individual spans. In
`timing_sc.c`, calibration is the minimum of ten intervals around a loop of
timestamp calls. It includes loop/outer-bracket costs that do not structurally
match each individually timed operation interval. That supports **possible
over-subtraction**, not a quantified or universally signed bias. The selected
output omits calibration values and gross endpoints, preventing reconstruction
of the actual bias from console means alone.

The allocator concern is also supported. At the pinned Zephyr commit
`684c9e8f32e4373a21098559f748f06915f950c9`,
[`queue_insert`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/kernel/queue.c#L132)
first checks for a pending receiver. That branch supplies the data pointer,
readies the receiver and exits **before** the allocation branch. A blocked
[`k_queue_get`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/kernel/queue.c#L336)
returns its delivered `swap_data`, without taking/freeing an allocated queue node.
Therefore these four named intended waiting-receiver cases do not establish
allocator performance:

- `fifo.put.alloc.wake+ctx.k_to_k`
- `fifo.get.free.blocking.k_to_k`
- `lifo.put.alloc.wake+ctx.k_to_k`
- `lifo.get.free.blocking.k_to_k`

The source arranges a higher-priority receiver that blocks before the sender
continues. This is the intended path, not a retained allocation trace proving
every iteration's implementation behavior. The corresponding immediate
alloc/free cases do exercise allocation and freeing of the **internal queue
node**, together with queue machinery; they are not standalone payload-allocation
tests. See the pinned
[FIFO](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/fifo.c)
and [LIFO](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/lifo.c)
benchmarks.

The separate
[`heap_malloc_free`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/heap_malloc_free.c)
test performs **100 iterations of `k_malloc(10)` followed immediately by
`k_free`**, with one live allocation. It is a small repeated-allocation control,
not a memory-pressure, concurrent-lifetime or fragmentation workload. Existing
native output also fails its frozen admission contract; these source findings
do not rehabilitate those captures into valid performance comparisons.

## What the existing IDF tools would actually measure

The requested [IDF task guidance](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-guides/performance/speed.html#tasks)
does support runtime statistics after enabling the corresponding configuration.
The [heap guidance](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/heap_debug.html#heap-information)
supplies current free memory, largest allocatable block and low-water tools.
Their availability is useful; it does not make their overhead or meanings
interchangeable across platforms.

Pinned implementation reviewed: IDF **5.5.6**, commit
`880a203ae603d08a94d06aa01e0d6761958e5e9e`.

| Tool / observation | Meaning and constraint to establish before use |
| --- | --- |
| Raw task-runtime counters | The [RISC-V port](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/portable/riscv/include/freertos/portmacro.h#L609) uses `esp_timer_get_time()` cast to the selected runtime-counter type. These are wall-microsecond attribution counters, not executed CPU cycles. |
| Switch-time accounting | [`vTaskSwitchContext`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L3669) charges the entire interval since switch-in to the outgoing task. The inspected code does not subtract ISR spans. Treat ISR attribution as inclusive unless a separate trace establishes exclusive time. Time in WFI can remain charged to whichever task owns the interval; an idle-task fraction is not directly a physical busy/sleep/energy percentage. |
| Counter width and snapshots | [Kconfig](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/Kconfig#L300) defaults to 32-bit counters and offers 64-bit on this non-SMP baseline. At 1 MHz a 32-bit clock wraps after 4,294.967296 s; switch accounting explicitly lacks wrap protection. Raw TCB counts are updated at switches, so an open current-task interval is not automatically flushed by a status snapshot. Keep task identities/lifetimes and phase boundaries explicit. |
| Statistics collection cost | [`uxTaskGetSystemState`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L2970) suspends scheduling and enumerates tasks, including stack-fill scans. The formatted `vTaskGetRunTimeStats` additionally allocates an array and formats text. Prefer preallocated raw collection outside measured operations; enabling switch-time accounting still changes the measured build and must have a control. |
| Task stack high-water | The [IDF task API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/include/freertos/task.h#L1488) reports **bytes**, unlike upstream FreeRTOS word documentation. Its implementation scans the fill pattern. It is an exercised-path indicator, not proof of minimum stack pointer, unwritten reserved space, all future branches or the separate ISR stack. |
| Heap current free / largest block | Query matching capabilities and do not add overlapping capability classes as independent RAM. [`heap_caps_get_info`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/heap_caps.c#L371) gathers allocator metadata; largest-block/info traverses TLSF blocks under a heap lock. Total free versus largest block is a useful fragmentation indicator, not a complete allocator-efficiency score. |
| Heap low-water / phase peak | The [API contract](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/include/esp_heap_caps.h#L220) sums per-region minima that need not occur simultaneously; it is not an exact global instantaneous peak. Added runtime heaps have documented limits. Local-minimum monitoring can restrict the phase, but its start allocates its own bookkeeping array and its stop restores historical minima. Record that cost and collect before stopping. |
| Per-task allocation tracking / hooks | Task tracking and hooks add bookkeeping, RAM and per-allocation processing. A detailed diagnostic profile is valuable but not a free change to the latency baseline. Heap hooks can run in ISR context; they must not allocate or block. Use separate profiles/controls instead of silently enabling comprehensive tracing in a scored image. |

## Direction: a bounded pilot, with prerequisites

A controlled load sweep would answer a new question: **under specified arrival,
queue and execution policies, how much checked work can the selected path
complete while meeting declared service requirements, and at what attributed
CPU/memory cost?** That is more informative about headroom than inverse RTT.
It still would not establish universal hardware efficiency or select an OS
without application/capability requirements.

Before implementation:

1. Close known artifact/contract issues and keep failed captures failed. Pin
   valid images, timing endpoints and one workload contract. A small IDF native
   API pilot is a reasonable starting option because that path already exists;
   it is not evidence that IDF should be the eventual platform.
2. Define release time, offered work, accepted work, completion, rejection,
   backlog limit, drain policy and deadline reference. A closed-loop producer
   changes its offered rate when the system slows. Use independently specified
   arrivals when measuring overload, and record generator lateness as well as
   consumer latency. A product deadline has not been specified.
3. Keep payload correctness and causal sequence IDs. Record all offered,
   completed, dropped and deadline-missed counts in consistent windows; retain
   overload failures and separate in-window work from post-window draining.
4. Define CPU attribution first: task-inclusive wall time versus exclusive
   task/ISR time, idle-task residency versus physical sleep, counter width and
   snapshot lifetime. Use a separate bounded trace when exclusive IRQ occupancy
   matters. Do not name a residual between timestamp brackets “idle.”
5. Distinguish static/reserved memory, workload live memory, stack/heap
   high-water indicators, fragmentation and harness cost. Keep the 48,000-byte
   recording buffer and future profiling buffers visible. Short-lived peaks
   are missed by sparse snapshots; use an audited peak mechanism where needed.
6. Compare profiled and unprofiled controls with identical offered-load policy,
   clocks and payload work; preserve their differing placement/configuration.
   Measure the cost of counters, collection and output rather than subtracting
   a universal correction. Emit diagnostics outside the measured service path.

Reuse is available. The pinned
[IDF runtime-statistics example](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/freertos/real_time_stats/main/real_time_stats_example_main.c)
already demonstrates before/after task-counter snapshots and reports created/
deleted tasks. The
[heap tracking examples](https://github.com/espressif/esp-idf/tree/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/heap_task_tracking)
and [allocator tests](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/test_apps/heap_tests/main/test_malloc_caps.c)
provide existing qualification references. Their example allocations, printing
and sampling policy should be inspected before adapting their method in Rust;
running unmodified upstream examples is distinct from authoring new C.

Radio, sleep/energy and hardware-offload phases need their own peer/instrument,
ownership and power-rail contracts. They need not all be prerequisites to a
radio-off queue pilot, but a queue pilot cannot close them. Wider cross-platform
load sweeps should follow working Rust integration and explicitly reconciled
execution/queue semantics rather than one assumed universal scheduler model.
