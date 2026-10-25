# IDF bounded-load method

Source/design review, **2026-10-23**, after the
[accounting qualification](../../results/idf-accounting-qualification-2026-10-23/RESULTS.md).
This proposes the next narrow experiment. It is not a build, physical pass,
sustainable-capacity result or OS-selection recommendation.

## Decision and existing work

The question is how a fixed nominal arrival stream behaves through one bounded
native queue and checked Rust consumer, and how enabling task runtime statistics
changes that complete configured path. This extends the earlier capacity-one
request/reply experiment: arrivals must no longer wait for previous completion.
The [utilization audit](utilization-audit.md), [prior-work review](../../../../docs/research/prior-work.md)
and [reuse inventory](../../../../docs/research/benchmark-reuse.md) were consulted before the design.

Reuse the existing checked `integer-mix-v1` workload, native queue ownership
pattern, clock decoder, failure-preserving capture and explicit input freeze.
The pinned IDF
[esp_timer example](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/esp_timer/main/esp_timer_example_main.c)
already demonstrates task callbacks, timer creation/rearming and cleanup.
Its logging callbacks are not the measured workload here. The IDF runtime-stats
example supplies the accounting concept already qualified locally; it does not
provide an independently offered-load queue test. The new maintained logic is
Rust, with a narrow generated-FFI boundary rather than authored C or assembly.

This is a self-hosted producer on the same C6 CPU, not an external traffic
generator. Generator delay consumes the tested machine's resources and must be
recorded instead of being concealed in a lower actual offered rate.

Two additional references clarify that extension. Schroeder, Wierman and
Harchol-Balter's [NSDI 2006 model comparison](https://www.usenix.org/legacy/events/nsdi06/tech/schroeder.html)
distinguishes completion-triggered arrivals from independently specified demand;
its server results do not transfer to this C6. At wrk2 commit
`44a94c17d8e6a0bac8559b53da76848e430cb7a7`,
[response timing](https://github.com/giltene/wrk2/blob/44a94c17d8e6a0bac8559b53da76848e430cb7a7/src/wrk.c#L479)
uses the intended release as well as an actual-send reference. Reuse that
endpoint distinction, not its connection/completion-based
[catch-up pacing](https://github.com/giltene/wrk2/blob/44a94c17d8e6a0bac8559b53da76848e430cb7a7/src/wrk.c#L402),
HTTP implementation, calibration duration or advertised percentile precision.
Its [top-level license](https://github.com/giltene/wrk2/blob/44a94c17d8e6a0bac8559b53da76848e430cb7a7/LICENSE)
is Apache-2.0; no source copy is needed here.

The pinned IDF
[queue example](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/freertos/basic_freertos_smp_usage/main/queue_example.c#L37)
has a relevant comment/source mismatch: despite the immediate-full-failure
comment, send uses `portMAX_DELAY`, followed by a relative 250 ms task delay.
That example is not an offered-rate generator. Its Unlicense/CC0 source is useful
for API/lifecycle reference; the existing independently authored Rust queue
adapter, using wait zero for attempts, supplies the chosen semantics.

## Why an ordinary periodic loop is insufficient

Reviewed local SDK: IDF **5.5.6**, commit
`880a203ae603d08a94d06aa01e0d6761958e5e9e`.

| Candidate mechanism | Source consequence |
| --- | --- |
| Relative sleep after sending or completing work | Accumulates phase drift and makes arrivals depend on preceding work. It is a closed-loop or shifted-arrival experiment. |
| FreeRTOS software timer / task delay | The selected tick is 100 Hz: 10 ms steps. Software timers use the configured priority-1 daemon. These do not implement the proposed 250–2,000 us grid. See [timer daemon](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/timers.c#L630) and [delay-until](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L1492). |
| `esp_timer_start_periodic`, skipping disabled | [The implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_timer/src/esp_timer.c#L500) advances the old alarm by one period and processes overdue alarms in a loop. A delayed producer can receive a burst of catch-up callbacks. |
| Periodic `skip_unhandled_events` | With sufficient backlog, the same branch changes the next alarm to `now + period`. Its callback count is not an immutable `T0 + iP` nominal grid. |
| A high-priority busy-wait producer | Can starve the lower-priority consumer while waiting for its next release, creating artificial overload. |

The [official timer guide](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/esp_timer.html)
also explains serialized task-dispatched callbacks and their interference.
Published minimum-timeout examples are for particular configurations; they are
not a qualified C6 latency bound. This proposal needs its own generator controls.

## Proposed producer and nominal-arrival contract

Use a persistent Rust producer at priority **6**, consumer at **5**, main at
**1**, and task-dispatch timer callback at the expected SDK priority **22**.
Verify actual priorities rather than relying solely on source constants.
The callback only notifies the producer; it performs no workload, formatting,
allocation, blocking wait or mutable sample-buffer access.

Use a one-shot timer, with its wakeup rearmed relative to the next point on an
immutable nominal grid. For period `P`, phase epoch `T0` and `N` slots, nominal
release i is `T0 + iP`, and the offer-window end is `Tend = T0 + NP`.

On each wake, the producer reads the monotonic clock. If it is still before
`Tend`, it selects the latest due unattempted slot, marks older unattempted slots
`generator_missed`, and attempts that latest slot **once**, with queue wait zero.
It records actual attempt time and nominal-relative lateness. It never changes
`T0`, retries a full queue, waits for consumer completion or burst-replays old
slots. A wake at/after `Tend` marks any remaining nominal slots missed and
enqueues no new job. A final end alarm ensures that a normally completed last
offer does not terminate the observation window early.

Suggested initial pulses: **250 ms** each, periods **2,000 / 1,000 / 500 /
250 us**, giving **125 / 250 / 500 / 1,000** nominal slots. Queue capacity is
**8**, with one persistent consumer and the existing checked 4,096-round payload.
The consumer may have one item in service in addition to eight queued items.
These short pulses probe underload and overload behavior; they cannot establish
long-duration sustainable capacity, fairness across several consumers or a
rare-failure probability.

The proposed image also contains a no-work cadence control (125 slots at
2,000 us, zero arithmetic rounds) and a delayed-generator control (16 slots at
500 us, with the first alarm deliberately armed 2,000 us after `T0`). The cadence
control requires no skipped/full slots and maximum lateness no greater than
one-quarter period. The delay control requires at least four skipped nominal
slots, at least one accepted job and no full-queue rejection; those old slots
must not become a catch-up burst. The four work pulses retain adverse outcomes;
their `generator_qualified` classification is true only with no skipped slots
and maximum lateness at most one-quarter period.

This totals **2,016 slots**. Six planned U32 atomic record fields imply
**48,384 bytes** of payload storage; actual struct size/capacity and other Arc,
Vec, queue, stack and SDK memory require separate build evidence. A passing
complete requires all accepted jobs checked and drained within the declared
100 ms bound. An exceeded drain bound preserves the failed/partial attempt
without a favorable complete record. That is a laboratory failure boundary,
not a deployed recovery guarantee.

Every nominal slot has one release outcome: accepted, queue-full rejection or
generator-missed. Each accepted item has at most one checked completion, with
its sequence identity and gross start/end retained. Distinguish completion
inside the offer window from completion during the bounded drain afterward.
The proposed constant **2,000 us** release-relative deadline, shared by all four
work rates, is an **exploratory test budget**, not a supplied product requirement.
Keep late accepted completions separate
from offers never admitted; do not double-count causes of unmet nominal work.

A full queue, generator miss or late completion is a measured adverse outcome,
not an excuse to retry or discard a slow profile. Such a result can characterize
the complete pipeline while preventing a narrower pure-consumer-capacity claim.
Wrong payloads, missing records, a reset or unresolved accounting inconsistency
remain failed/incomplete attempts with preserved evidence.

## Notification ownership and timer lifetime

The initial index-0 idea was rejected during source review.
[Pthread startup](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/pthread/pthread.c#L229)
and [join](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/pthread/pthread.c#L467)
use default notification slot 0; FreeRTOS stream/message buffers also use it.
Checking only pthread semaphores was insufficient to establish exclusivity.
Configure **two notification entries in both profiles**, reserving **index 1**
for timer-to-producer signaling. Inspect generated configuration and bindings.

The task-context
[`xTaskGenericNotify`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L5898)
increment operation does not wait. Its wakeup yields only when the notified task
outranks the current task; the expected 22-to-6 relationship avoids that yield.
[`ulTaskGenericNotifyTake`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/freertos/FreeRTOS-Kernel/tasks.c#L5728)
can block and clear the notification count. One-shot generation should produce
one expected notification per wake; unexpected stale or accumulated counts need
explicit handling rather than silent clearing inside a scored phase.

At every end boundary, the producer calls
[`esp_timer_stop_blocking`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_timer/src/esp_timer.c#L286)
from its normal task context with the proposed ten-tick wait budget. Only
successful disarm/quiescence may precede phase-done publication, reading shared
results, reusing state or final teardown. A plain `esp_timer_stop` does not
guarantee that an already-dispatched callback has returned.

The callback's producer task handle and any Arc-backed wake context must remain
valid until quiescence and accepted final deletion. A timeout/error must retain
those resources alive for failed-boot/reset recovery, rather than free an
argument still accessible from C. The
[`esp_timer_delete` implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_timer/src/esp_timer.c#L355)
queues a deferred delete event; the dispatch loop frees it without calling the
old callback. Accepted deletion is not proof that heap bytes have already been
reclaimed. No fallible blocking shutdown should be hidden in an ordinary Drop.

## Statistics-on/off control and observation boundaries

Keep workload, arrival/skip policy, queue capacity, priorities, clocks, payload
checks, sample storage and output policy common. Keep trace/stack scanning and
the same heap queries in both profiles; change actual runtime-statistics
configuration deliberately and encode unavailable off-profile counters as null.
Never turn API-generated zero into a measured zero CPU-cost claim. Off and on
configuration can generate different U32/U64 ABI types; regenerate each, and do
not cast an old total-counter pointer to the other width.

The qualified task scan costs about one millisecond here. Collect only with
producer/consumer settled before a pulse and after accepted work has drained,
while task identities remain alive. Do not insert scans per job or at the
nominal cutoff. Participant deltas describe the **whole offer-plus-drain phase**,
including declared boundary work; they are not exact in-window CPU utilization.
Keep the sampler's unflushed open-interval residual signed and descriptive.

Heap/stack-fill observations retain their existing limits. No authored
per-job allocation is intended, but sparse snapshots do not establish absence
of every transitive allocation or a global instantaneous peak. Record sample
buffers and temporary stack cost separately. Warm any output path that will be
reused, flush/settle outside pulses, and avoid UART output during offered load.

The on/off contrast measures the **profiling-configuration image effect**:
bookkeeping plus changed code/data placement, TCB layout and any necessary
feature gating. Retain exact images, layout and workload instruction evidence.
Matching clocks or unchanged workload bytes alone cannot isolate a universal
counter-only correction; the existing alignment result already demonstrates
that limitation. Do not subtract this contrast from other OS measurements.

Before collection, freeze the exact implementation, configuration, schedule,
window/drain/deadline definitions, positive and deliberately failing controls,
input identities and analysis. In particular qualify an empty/fast consumer,
deliberately full queue, deliberately delayed generator, correct skip/count
conservation, nominal-versus-actual endpoints and safe timer shutdown. Preserve
failed attempts. Physical generator behavior and a comparison result remain
unestablished by this source note.
