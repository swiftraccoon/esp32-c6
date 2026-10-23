# Timing and concurrency contract for the next C6 comparison

Reviewed **2026-10-23** against the exact sources used by our physical pilots.
This closes the bounded source-research brief for a radio-off experiment:
compare Rust `std` channels with native FreeRTOS queues on the same IDF kernel.
It also identifies the timing qualification that must precede interpreting its
numbers. No implementation, build or physical qualification was performed in
this review. A viable source path is not an on-board pass.

The decision is whether a maintained native queue API is worth a small Rust
wrapper before attributing our current application-channel cost to the OS or
considering a custom kernel. A new esp-rtos native application-thread adapter
is **not** established here. Unmodified Ariel scheduler tests remain a separate
qualification, with their original semantics and units.

## Sources actually traced

Registry package revisions below come from their `.cargo_vcs_info.json`;
the measured lockfiles and archived sources remain authoritative for the
historical builds. SDK source was inspected in the existing local 5.5.6 checkout;
generated `esp-idf-sys` bindings and the existing SDK archive were read without
rebuilding. Links identify upstream source, not a claim that upstream HEAD was
used.

| Component | Exact inspected source | Relevant path |
| --- | --- | --- |
| esp-hal 1.2.0 | [`dbd951f78c1276461ea3050c2ba71f552a3508ef`](https://github.com/esp-rs/esp-hal/tree/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal) | `src/time.rs`, `src/timer/systimer.rs`, `src/rng/ll.rs` |
| esp-rtos 0.4.0 | [`63e97be553f435f29171f7ae5f37a41ad737a75a`](https://github.com/esp-rs/esp-hal/tree/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos) | `src/lib.rs`, `src/embassy/mod.rs`, `src/task/mod.rs`, `src/timer/embassy.rs`, `src/esp_radio/mod.rs`, `esp_config.yml` |
| Embassy executor 0.10.0 / sync 0.8.0 | [`ca966f8ff4ee81c14bccade56b7a52c1fc154e26`](https://github.com/embassy-rs/embassy/tree/ca966f8ff4ee81c14bccade56b7a52c1fc154e26) | `embassy-executor/src/raw/mod.rs`, `embassy-sync/src/channel.rs`, `embassy-sync/src/blocking_mutex/raw.rs` |
| Embassy time 0.5.1 | [`50c6aac5b9b3b0af18f5a7d61725cc8e4d1f7acb`](https://github.com/embassy-rs/embassy/tree/50c6aac5b9b3b0af18f5a7d61725cc8e4d1f7acb/embassy-time) | `src/instant.rs`; actual driver supplied by esp-rtos |
| IDF 5.5.6 | [`880a203ae603d08a94d06aa01e0d6761958e5e9e`](https://github.com/espressif/esp-idf/tree/880a203ae603d08a94d06aa01e0d6761958e5e9e) | `components/esp_timer`, `hal/systimer_hal.c`, `hal/esp32c6/include/hal/systimer_ll.h`, `esp_hw_support/port/esp32c6/systimer.c`, FreeRTOS and pthread sources |
| Rust 1.96.0 | [`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`](https://github.com/rust-lang/rust/tree/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src) | `sync/mpmc/{array,context,waker}.rs`, `sys/sync/thread_parking/{mod,pthread}.rs`, Unix thread backend |
| esp-idf-sys 0.37.2 | [Versioned package source](https://docs.rs/crate/esp-idf-sys/0.37.2/source/) and the existing generated C6 `bindings.rs` | Declarations available under our actual SDK/configuration; bindings are configuration-dependent |

The local contract and observations are in the [existing experiment](../experiments/clocked-handoff.md),
[implementation review](../reviews/clocked-handoff-implementation.md),
[physical results](../../results/clocked-handoff-2026-10-23/README.md),
[prior-work review](prior-work.md) and [reuse inventory](benchmark-reuse.md).

## Timer method: what the current values mean

| Observation API | Actual backend and units | Width and important limitations |
| --- | --- | --- |
| `esp_hal::time::Instant::now()` / `elapsed()` | Reads SYSTIMER Unit0, then converts raw ticks to whole microseconds. Our `elapsed_ns` multiplies that integer duration by 1000. | The C6 hardware counter is 52 bits. The `u64` software representation does not create a 64-bit hardware timebase or nanosecond precision. |
| `embassy_time::Instant::now()` with our esp-rtos driver | `embassy_time_driver::now()` → esp-rtos `crate::now()` → HAL `Instant` → microseconds. esp-rtos selects `tick-hz-1_000_000`. | Changing to this high-level Embassy type would not improve our resolution. Timer callback scheduling is separate from timestamp reading. |
| `esp_idf_sys::esp_timer_get_time()` inside the existing Rust wrapper | Alias of IDF `esp_timer_impl_get_time()`: read SYSTIMER counter0, then apply C6 `ticks / 16`, returning signed 64-bit microseconds. | It is wall time since timer startup, not the FreeRTOS tick count. The SDK documentation's generic software interface width is not the C6 hardware counter width. |

These facts are traceable to [HAL time conversion](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/time.rs),
[esp-rtos time driver](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/timer/embassy.rs),
[IDF timestamp implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_timer/src/esp_timer_impl_systimer.c)
and [C6 tick conversion](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_hw_support/port/esp32c6/systimer.c).

The default C6 SYSTIMER source is the 40 MHz crystal with a fractional divider
of 2.5: **16 MHz, nominally 62.5 ns per raw tick**. HAL computes the same rate
as `XTAL * 10 / 25`. The low-level clock selector also permits RC_FAST, so
default-source assumptions need configuration evidence rather than extrapolation
from CPU MHz. CPU frequency scaling is not a direct change to an unchanged
XTAL-derived counter. Read the SYSTIMER source/enable/stall configuration before
and after each new run, alongside the existing CPU/flash registers. A nominal
tick duration is not a calibrated oscillator accuracy.
[C6 clock selector](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/esp32c6/include/hal/systimer_ll.h)

Both implementations request a hardware snapshot, wait for `value_valid`, then
read LO/HI/LO until the two low words agree. This handles a snapshot being
refreshed by another context between the separate 32-bit reads. It is not a
single zero-cost CSR read. HAL explicitly permits multiple reader contexts,
with an older snapshot possible. Preserve the upstream handshake; reading live
high/low registers once is not an equivalent replacement.
[HAL snapshot implementation](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/timer/systimer.rs),
[IDF snapshot implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/systimer_hal.c)

At 16 MHz the 52-bit raw counter wraps after approximately **8.9 years**.
For the next seconds-long run, reject a backwards timestamp, reset or changed
source instead of silently interpreting it as a successful enormous duration.
A general long-lived timer adapter would need explicit modular arithmetic and
a maximum unambiguous interval. No wrap behavior has been physically tested here.

Light sleep and deep sleep are outside the next experiment. IDF suspends its
timer during light sleep and advances it using RTC elapsed time on wake; deep
sleep restarts the application/time epoch. esp-rtos automatic light sleep
requires its configured sleep idle hook; our entry point uses ordinary
`esp_rtos::start`, whose default idle hook waits for an interrupt. That WFI
idle is distinct from configured light sleep. Preserve the current PM-disabled
IDF configuration and do not install the esp-rtos sleep hook for this experiment.
[IDF timer/sleep contract](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/esp_timer.html),
[esp-rtos startup and sleep policy](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/lib.rs)

Stall/debug settings also matter: the HAL permits counting or stopping when
the CPU stalls, while IDF initialization sets its CPU-stall policy. Do not
attach a halting debugger during timing. Timer continuity across ordinary
blocking/WFI must be part of the first physical timing qualification; it must
not be inferred from a successful busy loop alone.

## Selected raw timer route and rejected shortcut

The next timing qualification should retain raw **SYSTIMER ticks plus the
16,000,000 Hz scale**, with the existing microsecond reader as a consistency
check. Half-nanosecond tick fractions need rational conversion or a declared
rounding rule. Reporting more decimal digits does not increase accuracy.

| Target | Rust access route | Qualification boundary |
| --- | --- | --- |
| HAL / Embassy | Safe public `esp_hal::timer::systimer::SystemTimer::unit_value(Unit::Unit0)` and `SystemTimer::ticks_per_second()` under the already-enabled unstable API feature. | Reuse the initialized counter; do not construct/reset/reconfigure Unit0 or steal timer ownership just to read it. Safe Rust access does not prove calibration or unchanged clock configuration. [Versioned C6 API](https://docs.espressif.com/projects/rust/esp-hal/1.2.0/esp32c6/esp_hal/timer/systimer/struct.SystemTimer.html) |
| IDF | Existing SDK function `esp_timer_impl_get_counter_reg() -> uint64_t`; a small Rust C-ABI declaration and reviewed safe wrapper can call it after IDF startup. | This is a **private SDK interface**, absent from the current generated sys bindings. Its signature is pinned in `private_include/esp_timer_impl.h`; source uses the same initialized counter/context as `esp_timer_get_time`. A read-only `nm` check confirmed a global definition in the existing SDK `libesp_timer.a`. Next linking and on-board behavior remain unqualified. No authored C or assembly shim is required by this route. [Private declaration](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_timer/private_include/esp_timer_impl.h) |

If the private binding cannot be qualified, retain whole-microsecond RTT and
batched averages with their narrower claims; do not quietly replace it with
ad hoc MMIO or change SDKs within the comparison.

The tempting alternative, generic `riscv::register::mcycle::read64()`, is not
selected. The C6 **HP CPU** uses custom performance CSRs: event selection
`0x7e0`, mode `0x7e1`, counter `0x7e2`. Both pinned HAL RNG code and IDF's cycle
accessor select this custom counter for C6. The TRM says its cycle event stops
during **WFI**; the mode selects saturation versus overflow. A 32-bit wrapping
count would span about 26.84 seconds of counted cycles at 160 MHz, but saturation
would invalidate modular subtraction. There is no inspected public safe HAL
cycle accessor for this path. A generic crate's safe CSR reader does not
establish that an ISA-specific CSR exists or has the desired meaning on this
HP core. The LP core's standard counters are a different subsystem.
[HAL C6 selection](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/rng/ll.rs),
[IDF cycle selection](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/riscv/include/riscv/rv_utils.h),
[C6 TRM v1.2, HP CPU performance registers](https://documentation.espressif.com/esp32-c6_technical_reference_manual_en.pdf)

Even a qualified cycle counter would measure counted CPU activity rather than
reliably accounting for all wall time while a task blocks. It could supplement
a later non-sleeping primitive test, after configuration and ownership review;
it cannot replace the wall clock for this request/reply decision.

## Observer budget and ISR boundaries

| Contributor | Known now | Evidence still needed in the next qualification |
| --- | --- | --- |
| Quantization | Existing API timestamps truncate to 1 µs; raw ticks have a nominal 62.5 ns step. | Confirm observed increments and raw-to-µs bracketing without assuming zero phase error. |
| Reader/serialization cost | Snapshot request, synchronization, retry loop, calls and return instructions take time. Existing empty observations have a 1 µs median. | Gross back-to-back reads and empty-region distributions for each final binary, plus increasing-length batches. Keep raw observations; no automatic observer or percentile subtraction. |
| Frequency accuracy | Default nominal XTAL-derived 16 MHz scale is source-backed. | Record clock fields. No numerical oscillator uncertainty is established without board specifications or an independent reference; CPU-versus-SYSTIMER checks share a crystal and are not independent calibration. |
| Interference | Tick/other interrupts and scheduler work are included while interrupts remain enabled. | Archive actual enabled interrupts, tick/backend and timer placement; distinguish intended load from measurement-only instrumentation. |
| Code placement | A separate physical probe already changed timing with unchanged workload bytes. | Inspect final reader/caller/hot-path addresses and RAM/flash placement; archive maps/disassembly. Declare ordinary-build versus placement-controlled scope. |

Reader qualification should check monotonicity, consistency with the legacy
reader, increasing batch duration and continuity across an actual blocked/WFI
interval before collecting the fixed comparison. This establishes internal
consistency, not electrical traceability. External GPIO/logic-analyzer timing
remains necessary before claims whose useful effect is comparable to unresolved
instrument error or before physical edge-to-response claims.

Do not mask interrupts around a blocking send/receive, call task-only blocking
APIs from an ISR, or use host UART arrival times as microsecond timestamps.
Future ISR tests need separate ISR-safe APIs, priority/masking contracts and
cache-off/IRAM review of the **entire** call path. A function's IRAM attribute
alone does not establish that all code/data it reaches are accessible.
The current IDF configuration has `CONFIG_ESP_TIMER_IN_IRAM=y`; the next study
still measures in ordinary task context with flash available.

## What the selected concurrency paths do

Embassy sync 0.8.0 channels keep a bounded queue and waker registrations.
An empty receive registers the receiving task's waker and returns `Pending`;
a successful send enqueues data and wakes that registration. Our
`CriticalSectionRawMutex` uses `critical_section::with` around short operations.
The executor enqueues a woken task and polls runnable tasks; a wake does not
interrupt the current future's execution. esp-rtos runs this executor in an
OS thread and suspends that thread when it has no runnable async work.
[Channel source](https://github.com/embassy-rs/embassy/blob/ca966f8ff4ee81c14bccade56b7a52c1fc154e26/embassy-sync/src/channel.rs),
[mutex source](https://github.com/embassy-rs/embassy/blob/ca966f8ff4ee81c14bccade56b7a52c1fc154e26/embassy-sync/src/blocking_mutex/raw.rs),
[executor source](https://github.com/embassy-rs/embassy/blob/ca966f8ff4ee81c14bccade56b7a52c1fc154e26/embassy-executor/src/raw/mod.rs),
[esp-rtos executor](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/embassy/mod.rs)

Rust 1.96's bounded std channel uses its MPMC array implementation, atomics,
waiter registration and cached contexts. A blocking wait reaches thread
parking; the ESP-IDF Unix target selects the pthread parker, which uses mutex
and condition-variable operations. IDF implements condition-variable waits
with task-specific FreeRTOS semaphores. This is substantially more than a
single native queue call. Construction/warmup may initialize state before
timing; an arbitrary contention/allocation guarantee still needs measurement.
[Rust bounded channel](https://github.com/rust-lang/rust/blob/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src/sync/mpmc/array.rs),
[parking selection](https://github.com/rust-lang/rust/blob/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src/sys/sync/thread_parking/mod.rs),
[pthread parker](https://github.com/rust-lang/rust/blob/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src/sys/sync/thread_parking/pthread.rs),
[IDF condition variables](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/pthread/pthread_cond_var.c)

Native FreeRTOS queues copy the configured item bytes and maintain blocked
sender/receiver lists. Waking a higher-priority receiver can reschedule before
the send returns. Thus send-call duration, receive-resumption latency and RTT
are distinct endpoints. Our single-core configuration enables preemption and
time slicing; priority must be controlled and recorded, not inferred from tick
frequency. A task notification carries task-local notification state, not the
same capacity-one FIFO contract: it belongs in a separately named experiment.
[Pinned queue behavior](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/queue.c),
[configuration](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/config/include/freertos/FreeRTOSConfig.h)

There is no public general task-spawn API in esp-rtos 0.4.0's inspected
application interface. `CurrentThreadHandle` can delay/set the current thread's
priority. The separate `esp-radio-rtos-driver::task_create` route is unsafe,
allocator-backed in this implementation and intended for radio integration;
its semaphore/queue docs explicitly identify esp-radio as the expected user.
That interface is a candidate to evaluate, not an already-proved ordinary
application threading layer. Do not introduce it solely to fill a comparison
row without documenting feature/allocator/lifetime consequences.
[esp-rtos task interface](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/task/mod.rs),
[driver task API](https://docs.espressif.com/projects/rust/esp-radio-rtos-driver/0.4.0/esp_radio_rtos_driver/fn.task_create.html),
[driver usage boundary](https://docs.espressif.com/projects/rust/esp-radio-rtos-driver/0.4.0/esp_radio_rtos_driver/semaphore/index.html)

## Concrete next comparison and its admission controls

Use two **IDF** variants: the existing `std::sync::mpsc::sync_channel` path and
two native FreeRTOS queues. Keep `std::thread::Builder` for the responder in
both, with the same stack size, main/worker priorities **1/5**, CPU affinity,
SDK, Rust compiler, optimization and common workload. This changes the chosen
communication API on one kernel; it does not isolate FFI instruction cost or
establish that one OS scheduler is superior. Equal-priority **1/1** is a
separate follow-up case, with a separate result population. Async single-task,
separate-task and multi-executor cases are likewise separate scenarios.

The source-backed native route already exists in our generated bindings:
`xQueueGenericCreate`, `xQueueGenericCreateStatic`, `xQueueGenericSend`,
`xQueueReceive`, `uxQueueMessagesWaiting`, `vQueueDelete`,
`xTaskGetCurrentTaskHandle`, `uxTaskPriorityGet` and `eTaskGetState`.
Convenience names such as `xQueueCreate` and `xQueueSend` are macros;
use the exported generic functions through reviewed Rust wrappers. The pinned
header defines queue type BASE and send-to-back position as typed zero; these
macro constants are absent from our bindings. Queue creation/deletion and all
allocation happen outside timing. Preserve ownership of both queue handles and
valid 4-byte payload pointers until both tasks stop accessing them.
[Queue declarations and constants](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/include/freertos/queue.h)

The common timed contract is:

1. Exactly two capacity-one queues, a `u32` request sequence and XOR response
   `sequence ^ 0xa5a55a5a`; at most one request is outstanding.
2. Warmup, task/queue creation, state checks and timer qualification precede
   collection. Each interval begins immediately before the initiator sends
   and ends immediately after its matching reply is received. Record gross
   raw ticks; validate response and buffer samples after the second read.
3. No task destruction, allocation initiated by the harness, UART output or
   formatting occurs within an interval. Tick interrupts and normal kernel
   synchronization remain enabled. Both workers wait for another request after
   replying; closure/join/deletion is outside timing.
4. Preserve failures, timeouts, unexpected resets and changed clocks as failed
   runs. Sample count, order and stopping follow the separate
   [experimental-design brief](experimental-design.md); do not tune them after
   observing the native result.

For a **blocked-receiver-qualified** subcase, an application marker set before
`recv` is insufficient. In a separate qualification/control path, capture the
responder's real task handle while it is alive, identify its receive phase,
verify empty queues where the API exposes that state, and observe `eBlocked`
outside timing before issuing the only request capable of waking it. The
pinned configuration explicitly enables `INCLUDE_eTaskGetState=1` even with
trace facility disabled. Its implementation distinguishes indefinite object
waits from truly suspended tasks. However `eBlocked` alone does not identify
the object: std may be parked in its pthread machinery. Preserve the phase
evidence and do not inspect private std internals to invent a portable waiter
count. If this stronger evidence is absent or cannot be matched, label the
comparison **steady-state request/reply RTT** and defer blocked-wakeup claims.
[Task-state implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c)

The closest existing local test is our validated Rust RTT. Reuse its payload,
correctness rule, raw collector and provenance. The extension resolves the
specific native-versus-wrapper decision using a maintained kernel, instead of
creating another idle cross-OS number. The independent Rust adapter expresses
documented exported APIs; it does not copy an unlicensed benchmark. Separately,
qualify [Ariel's existing flags/yield tests](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks)
without changing their operation counts or reinterpreting tick averages as
per-event tails. [Zephyr's native suite](https://github.com/zephyrproject-rtos/zephyr/blob/76280bce2a050376d57e14240687b5c283dea0e7/tests/benchmarks/latency_measure/README.rst)
already separates primitive cases; that structure informs later qualification,
not direct numerical equivalence to our two-queue RTT.

## Closed research and deferred evidence

The source trace establishes the current timer units/backend, a Rust-only raw
read route, a concrete same-kernel API comparison and its inference boundary.
It rules out an unreviewed generic cycle-counter substitution and a presumed
public esp-rtos thread layer. These are decisions sufficient to prepare the
next implementation brief, not a new requirement to research every kernel
before any startup test.

The next implementation still needs a reviewed Rust private-function binding,
successful linking, final-binary placement checks, timer-source readback,
reader/blocked-idle calibration and actual priority/state evidence. Exact
timer uncertainty, electrical validation and a physical pass remain **not
established**. Source declarations and a global archive symbol reduce the
implementation risk; they do not close those physical checks. No extra boot
or benchmark result was produced by writing this document.
