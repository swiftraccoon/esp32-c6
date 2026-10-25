# IDF task and memory accounting method

Source review, **2026-10-23**, for the prospective
[accounting qualification](../experiments/idf-accounting-qualification.md).
This closes a narrow method-design prerequisite from the
[utilization audit](utilization-audit.md). It establishes neither an on-board
pass nor utilization, capacity, memory peaks, profiling overhead or an OS choice.

## Decision and reuse

The question is whether our pinned Rust/IDF path can distinguish a continuously
blocked worker from checked busy work, expose incomplete task accounting, and
observe a known live allocation before we build a controlled-load experiment.
This is qualification of diagnostic meanings and failure handling. There is no
product arrival rate, backlog/deadline policy or comparative estimator here.

The [prior-work review](../../../../docs/research/prior-work.md) and [reusable assets](../../../../docs/research/benchmark-reuse.md)
were consulted before designing the probe. Closest existing assets:

| Asset at the pinned IDF revision | What is reused | Why the Rust probe adds something |
| --- | --- | --- |
| [Real-time-statistics example](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/freertos/real_time_stats/main/real_time_stats_example_main.c) | Before/after task snapshots and explicit created/deleted-task handling. | Use fixed storage, reject lifecycle changes, match handle plus task number, retain raw counters/endpoints, and deliberately challenge capacity and the open current-task interval. |
| [Heap metadata and minimum tests](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/test_apps/heap_tests/main/test_malloc_caps.c#L143) | Observe allocator state before, during and after a known allocation; warm output before observing. | Check and retain the complete touched payload, capability mask, background drift and distinct requested/observed byte counts with strict replay. |
| [Pthread example](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/pthread/main/pthread_example.c#L62) | Task stack-fill API availability and byte units. | Preserve the indicator with its task identity and reservation, without treating it as a qualified maximum-stack measurement. |

The statistics and pthread examples are C, with public-domain/CC0 notices;
the heap test declares `Unlicense OR CC0-1.0`. Reading their method or evaluating
unmodified upstream C is distinct from authoring an adapter. The new probe is
independently authored Rust using already generated `esp-idf-sys` interfaces;
it does not copy the example's C implementation or add a C/assembly shim.
The [native API note](idf-native-api-options.md) remains the separate wrapper
adoption review. No new general-purpose wrapper dependency is required for
these deliberately narrow raw snapshot operations.

## Exact source and build boundary

Inspected local SDK checkout:
`firmware/esp-idf/.embuild/espressif/esp-idf/v5.5.6`, Git commit
`880a203ae603d08a94d06aa01e0d6761958e5e9e`. Upstream links below identify the
same immutable revision. The firmware baseline is Rust 1.96.0,
`riscv32imac-esp-espidf`, exact `esp-idf-sys 0.37.2`, IDF 5.5.6 and the existing
release settings. Generated bindings were inspected in the existing
`firmware/esp-idf/target/riscv32imac-esp-espidf/release/build/esp-idf-sys-*/out/bindings.rs`
outputs. Those local generated files are inspection context, not a substitute
for archiving the new build's actual ABI and configuration.

The old queue configuration disables runtime statistics/trace. The diagnostic
profile enables them, selects `esp_timer_get_time` and selects **64-bit** runtime
counters through the [non-SMP Kconfig choice](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/Kconfig#L289).
Existing inspected bindings use 32-bit runtime fields and total pointers; they
must be regenerated. Verify the new `TaskStatus_t` runtime field and the
`uxTaskGetSystemState` total-pointer type are both u64 before linking/capture.
An option label or hand-cast pointer cannot establish ABI compatibility.

The actual [RISC-V clock macro](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/portable/riscv/include/freertos/portmacro.h#L609)
casts `esp_timer_get_time()` to the selected counter type. Units are nominal
wall microseconds. The 32-bit variant wraps after 4,294.967296 seconds; the
switch-accounting source explicitly has no wrap protection. Selecting u64
avoids that short horizon, not clock uncertainty or every possible overflow.

## Task accounting semantics

| Operation/source | Meaning for the probe |
| --- | --- |
| [Switch accounting](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L3669) | At a context switch, charge the full wall interval since switch-in to the outgoing task. No ISR duration is subtracted. These are task-attributed inclusive intervals, not exclusive execution cycles. |
| [System snapshot](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L2970) | Requires array capacity for all current tasks. Enumerates ready, delayed, suspended and deleted-but-not-cleaned-up tasks; reads the total clock after enumeration. Insufficient capacity returns zero without assigning the total output. Zero is a failed snapshot, never an empty successful system. |
| [Single-core scheduling guard](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/esp_additions/include/esp_private/freertos_idf_additions_priv.h#L104) | Enumeration suspends scheduling on this single-core configuration. Interrupts remain enabled; it is not an instantaneous stop-the-world hardware trace. |
| [Stored counter copy](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L4685) | `vTaskGetInfo` copies the TCB runtime counter without flushing the currently running task's open interval. The sampler's counter can lag its actual activity. |
| [List enumeration](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L4770) | System snapshots request stack scanning for every enumerated task. Collection cost depends on task/stack state and must remain visible. |
| [Status lifetime](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/include/freertos/task.h#L163) | Saved task-name pointers can dangle after deletion. Retain numeric handle/task-number pairs, detect missing/new/reused identities and keep the controlled worker alive through all snapshots. |
| [Idle hook](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_system/freertos_hooks.c#L42) | Idle may call `esp_cpu_wait_for_intr` even with power management disabled. Idle-task attribution is not physical busy percentage, sleep residency or energy consumption. |

The worker blocks before and after the measured phases. Its priority 5 exceeds
the sampler's priority 1, so a checked return to blocking settles the worker's
interval before the lower-priority sampler records it. This does not settle
the sampler's own open interval or remove interrupt attribution. Keep all-task
deltas and their signed difference from elapsed time descriptive; no exact
conservation equation, exclusive ISR cost or residual called idle is justified.

## Heap and stack meanings

The existing bindings supply `heap_caps_get_info` with `multi_heap_info_t`, free,
largest-block and minimum queries, both local-minimum-monitor operations, and
both stack high-water APIs. The probe needs only the existing raw interfaces.

| Recorded quantity | Valid interpretation and limitation |
| --- | --- |
| Current free / allocated bytes | Aggregate matching allocator regions at observation time. The [aggregation source](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/heap_caps.c#L371) exposes allocator state, not all chip RAM or application headroom. Requested payload and observed consumption differ through rounding and bookkeeping. |
| Largest free block | A single allocatable-block indicator for the same capability mask. It can remain unchanged when allocation uses another region. Total-free/largest-block alone is not a fragmentation workload or allocator-efficiency score. |
| Lifetime minimum free | [API contract](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/include/esp_heap_caps.h#L220): sum of per-region minima, potentially reached at different times; runtime-added heaps have documented limits. It is not an exact global simultaneous peak. Startup may already have established a lower value than this probe reaches. |
| Capability mask | v1 queries and allocates with `MALLOC_CAP_8BIT`, value **4**, and touches 16,384 bytes. Other capability classes overlap; do not sum their totals or silently substitute a different mask. |
| Stack free-fill bytes | [API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/include/freertos/task.h#L1488) returns bytes. [Implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L4805) scans contiguous untouched `0xA5` fill. This is an exercised-path indicator, not minimum stack pointer, unwritten frame reservation, ISR stack or future worst case. v1 contains no dedicated positive stack-touch qualification. |

The historical configuration has `CONFIG_HEAP_TLSF_USE_ROM_IMPL=y`. The
[C6 ROM linker aliases](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_rom/esp32c6/ld/esp32c6.rom.heap.ld#L59)
resolve multi-heap info/free/minimum operations into ROM. The equivalent TLSF
walk in `multi_heap.c` is excluded under that option; reading its C body is not
proof that we inspected the executed ROM implementation. Archive and inspect
the diagnostic image's actual configuration/linkage. Treat collection as
non-free regardless of whether an equivalent C body or ROM routine implements it.

### Why v1 avoids local-minimum monitoring

The [header](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/include/esp_heap_caps.h#L254)
promises failure if monitoring is already active. The pinned
[implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/heap_caps.c#L314)
instead permits another start, resetting minima and returning `ESP_OK`. Its
[fault-case test](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/heap/test_apps/heap_tests/main/test_malloc_caps.c#L246)
explicitly expects repeated start to succeed. This is a verified documentation
mismatch, not a claim that the existing observations suffered an unnoticed reset.

Start also allocates bookkeeping and asserts its allocation succeeded. Stop
restores the historical minimum combined with deeper phase minima and frees
bookkeeping. A future phase-minimum method needs explicit ownership, start/stop
boundaries, collection before stop and its own controls. The first probe omits
this mechanism, allocation hooks and task-allocation tracing entirely.

## Selected controls and remaining qualification

The experiment contract, rather than this research note, defines the frozen
acceptance bounds and exact record order. Its bounded observations are:

1. An undersized-capacity rejection and a separate 10 ms scheduling-suspended
   current-task interval expose missing data. The latter checks advancing wall
   time with an unchanged stored TCB count; it is excluded from useful-work phases.
2. A continuously blocked worker over approximately 250 ms versus approximately
   250 ms of checked `integer-mix-v1` work distinguishes blocking from attributed
   active work. Broad runtime/span bounds are diagnostic admission guards, not
   uncertainty intervals or exclusive CPU percentages.
3. A no-allocation background snapshot precedes the held/free 16,384-byte
   allocation. Every byte is written as `i mod 251`, reread and checked. Fixed
   drift/recovery bounds remain explicit; they are not corrections or a memory
   pressure/fragmentation result.

Warm output and initialize tasks/channels before the baseline: the upstream
heap metadata test explicitly notes the first `printf` allocation. Store all
observations before printing and keep harness storage, worker/main stack
reservations and profiling configuration visible. Retain task-enumeration,
heap-info and empty-bracket endpoints; do not subtract a universal observer cost.
These endpoints expose selected collection latency, not total configuration-on
versus configuration-off overhead. That requires a later separate control.

Before flashing, regenerate and inspect the ABI, review actual source/config
paths and compiled workload, exercise host acceptance and contrary controls,
freeze exact inputs, and preserve failed attempts under the
[assurance workflow](../../../../docs/assurance-workflow.md). Only physical captures and their
review can establish whether these controls pass on the connected board. No
such result is claimed by this source research.
