# ESP-IDF Rust channel versus native queue experiment

Design date: **2026-10-23**. Status: **completed physical qualification and all
24 scheduled comparative boots**, twelve paired blocks across three labeled
sessions, without retries. The [results](../../results/idf-queue-api-2026-10-23/RESULTS.md)
and [independent review](../reviews/queue-api-results.md) record 24,000 checked
round trips and 24,000 observer observations. Original firmware was restored,
digest-verified and reset afterward. The design below records the contract
chosen after the [five research briefs](../research/research-readiness.md);
historical inputs and preregistration remain unchanged.

## Decision and closest work

Does choosing a native FreeRTOS queue instead of Rust `std` bounded channels
materially change the checked request/reply path on the **same** IDF runtime?
The result could justify a small Rust FFI adapter while retaining the existing
kernel and services. It could also show that this API substitution is not a
useful remedy. It will not isolate FFI overhead or rank operating systems.

The closest local result is the
[clocked handoff experiment](../../results/clocked-handoff-2026-10-23/README.md):
the selected IDF Rust channel path had a 62 µs median, compared with 7 µs through
a differently scheduled Embassy path. Neither value establishes the cost of
IDF's native queues. Preserve those images/data; the new raw timer and
readiness checks create a new experiment, including a newly measured std
control. Do not subtract or directly pool historical and new durations.

[Zephyr native object tests and Ariel flags/yield](../research/benchmark-reuse.md)
provide existing synchronization scenarios; they retain their own operation
counts and models. This experiment independently uses a simple published
queue/request-reply contract in Rust. It adds a same-kernel API contrast,
validated per-event records and matched timing that those unmodified tests do
not provide. No restricted paper artifact source will be copied.

## Freeze one proved platform input

Retain Rust **1.96.0**, target **riscv32imac-esp-espidf**, sys **0.37.2**,
IDF **5.5.6** at **880a203ae603d08a94d06aa01e0d6761958e5e9e** and the
existing release configuration. Do not combine this experiment with a wrapper,
SDK, compiler or radio upgrade. Both variants use Rust `std::thread::Builder`
for the worker, the same observed main/worker FreeRTOS priorities **1 / 5**,
stack reservations and **100 Hz** tick. Read back priorities rather than
assuming pthread configuration took effect.

Use the same board, supply, 160 MHz HP CPU and verified flash configuration;
disable radios, DFS and automatic sleep. Retain the watchdog/logging choices.
Allocation of channels/queues, worker creation, startup synchronization,
sample storage and cleanup occur outside timed regions.

Both implementations reserve the same **48,000-byte preallocated heap buffer**
for 1,000 observer and 1,000 message samples. This is sample storage, not a
total-runtime RAM comparison.

## Two variants, one observable contract

| Property | A: Rust channel control | B: native queue choice |
| --- | --- | --- |
| Request/reply objects | Two `std::sync::mpsc::sync_channel::<u32>(1)` objects | Two FreeRTOS queues, each capacity one, item size four bytes |
| Worker | Same Rust pthread/FreeRTOS task arrangement | Same arrangement; only the message backend changes |
| Input/output | Sequential `u32`; reply is request XOR `0xa5a55a5a` | Identical |
| Interval | Immediately before request send through completion of reply receive | Identical endpoints |
| Validation | Every reply checked; no next request until validation | Identical |
| Operation count | Two sends and two receives per round trip | Identical |
| End of measurement | Worker parks for the next request | Identical; no thread exit inside final timing |

Use a reserved stop message outside the measured sequence, after the last reply
has been timed and checked, then join the worker and destroy the backend.
This avoids final-sample worker teardown. All recorded request values must
exclude that sentinel. Report setup/teardown errors separately.

This controls application behavior, not internal implementation. Rust channels
use their own synchronization/wakeup machinery; FreeRTOS queues copy items and
use kernel wait lists. Any difference includes those paths, compiler placement
and their interaction with the retained runtime. A further decomposition needs
its own experiment. Do not divide a round trip by two to label a context switch.

## Rust interfaces and qualification

The [pinned queue header](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/include/freertos/queue.h)
and existing generated bindings establish a Rust-only FFI route:
`xQueueGenericCreate`, `xQueueGenericSend`, `xQueueReceive`,
`uxQueueMessagesWaiting` and `vQueueDelete`. Queue type/base and send-to-back
are typed zero-valued macro arguments in that header; the macros themselves
are absent from the generated bindings. Keep their cited constants inside a
small checked wrapper. Retaining Rust thread creation avoids needing a C macro
shim for `xTaskCreate`. No authored C or assembly is required by this design.

The wrapper must check allocation/status returns and establish the `u32`
size/alignment, handle lifetime, single-owner cleanup and live pointer
invariants. Both backend variants must expose the same application behavior.
Use blocking waits in this idle request/reply case and a separately declared
host/watchdog experiment timeout; a hang remains a failure with retained reset
evidence. If finite kernel waits are introduced, match and disclose timeout
semantics rather than treating a timeout return as a successful duration.

Admission requirements, completed for this study:

1. Prove sequence/checksum correctness, full/empty behavior and cleanup under
   the selected APIs. Allocation failure, wrong reply, timeout and reset are
   explicit outcomes. Inspect the final linked path for each variant.
2. Qualify responder readiness **outside** each timed interval using a
   sequence-specific phase marker, no outstanding request/reply, the worker's
   actual FreeRTOS task handle and observed blocked state. The bindings expose
   `xTaskGetCurrentTaskHandle`, `eTaskGetState` and priority inspection; IDF's
   selected configuration enables `eTaskGetState` without the trace facility.
   For native queues, also verify empty queue state. Audit the worker control
   path so the phase/state pair identifies the intended receive path.
3. A “ready” flag before calling receive does not prove it blocked, and
   `eBlocked` alone does not identify the waiting object. Rust channels may
   block through a semaphore/condition path. If that qualification cannot be
   established, label the experiment **steady-state request/reply**, not a
   proved blocked-waiter or isolated wakeup latency. Preserve the limitation;
   do not invent an equivalent native wait-list state for Rust internals.
4. Preserve naturally aligned typed messages and timer/sample storage. The
   [rev0.2 errata audit](../research/rev02-errata-workarounds.md) accounts for
   IDF's linked copy mitigation; this contract introduces no packed or
   arbitrary unaligned flash/ROM-copy workload.

## Timer choice and admission

Use **raw 52-bit HP SYSTIMER UNIT0 ticks**, nominally **16 MHz / 62.5 ns per
tick** with the verified 40 MHz crystal. Retain raw integers and the rational
scale; do not truncate every event to whole microseconds or imply an integer
nanosecond quantum. The
[timing-method audit](../research/timing-and-concurrency-contract.md) explains
snapshot/read consistency, sleep limits and counter alternatives.

IDF's private, globally defined `esp_timer_impl_get_counter_reg(void)` returns
the raw counter but is absent from our generated sys bindings. A narrowly
version-pinned Rust `extern "C"` declaration and reviewed wrapper call the
existing library; **both final study builds passed linkage/ABI inspection**.
It must run after IDF timer initialization, retain the selected SDK assumption
and be rejected/re-reviewed when that SDK changes. Do not silently replace it
with a different clock or assume a private function is a stable upstream API.

Both A and B use the same timer function and read sequence. Qualify ordered
reads and the conversion against `esp_timer_get_time` over longer known waits;
retain empty-read samples, raw timestamps and monotonicity failures. This
cross-check shares a hardware source, so it checks implementation/scale rather
than independently proving oscillator accuracy. No external timing instrument
has yet been qualified; restrict initial claims to the onboard counter and
its stated nominal scale. External end-to-end timing remains a later check.
Read and retain SYSTIMER source, enable and CPU-stall configuration before
and after each boot, alongside the CPU/flash fields. Verify continuity across
an actual blocked/WFI interval, not just a busy delay.

The counter wraps in about 8.9 years at this rate. Record/reset baseline and
duration guards; an unexpected backwards timestamp during this short study
must be a timer error, not silently normalized into a plausible sample. The
CPU performance counter stops in `WFI`, so it is unsuitable as this study's
sole wall-clock timer across blocking waits.

## Placement, analysis and stopping

Use ordinary release placement for this **whole API-path** comparison. Archive
both ELF/map files, timer/adapter disassembly, symbol addresses and clock
readback. Do not change function alignment or move one variant into IRAM as an
unrecorded improvement. A material result may motivate a separate placement
control before assigning its cause to one implementation layer.

After timer/backend qualification, freeze both images and follow
[the experimental-design brief](../research/experimental-design.md): 12
scheduled paired blocks, balanced AB/BA order across three sessions, two-second
settling, eight warmups and 1,000 checked round trips per boot. Commit the
actual schedule/seed and manifest before capture. App failures consume their
scheduled block; fixture-invalidated attempts remain archived with the stated
repeat limit. No runs are added merely to obtain a favored difference.

Report gross tick durations and per-boot distributions, every failed attempt,
paired boot-p50 differences and the prescribed conditional uncertainty.
Conclusions apply to this pinned pair on this board. A faster native queue
would support investigating a small Rust adapter; it would not establish that
FreeRTOS is intrinsically faster/slower than another scheduler or that a custom
kernel is needed.

Implementation admission details: each completed RTT must be at most one
second, the blocked timer qualification at most 500 ms, and the host capture
deadline is 180 seconds including UART output. Exceeding a bound fails the
attempt; it is not outlier clipping. The worker's next-receive phase marker can
execute before the lower-priority initiator resumes from the preceding reply,
so that common instrumentation contributes to the measured application path.
The initiator's readiness queries remain outside the interval. SDK WFI calls
are qualified against FreeRTOS ticks from SYSTIMER UNIT1; this is another
counter/IRQ path, sharing the peripheral/crystal rather than an independently
calibrated oscillator. See the [integration review](../reviews/queue-experiment-integration.md).

## Completion boundary

Rust implementation/API review, raw-tick collector support, both feature builds,
independent final-binary review, physical timer/backend qualification and the
frozen comparative study are complete. All 24 scheduled boots passed, with no
application failures, fixture invalidations or retries. The original image was
restored, digest-verified and reset.

Every boot's request/reply p50, p95 and p99 is **1,006 ticks / 62.875 µs** for
std channels or **319 ticks / 19.9375 µs** for native queues. All twelve paired
native-minus-std boot-p50 contrasts are **−687 ticks / −42.9375 µs**. The
descriptive median ratio is about **3.15×**; observed maxima are 80 µs and
26.625 µs. These gross values use the nominal 16 MHz counter scale, without
observer subtraction or independent electrical calibration. They characterize
the complete instrumented API paths and ordinary image placement, not kernel
switch costs, WCET or an overall OS ranking. The results/report review linked
above retain raw evidence and statistical limits.
