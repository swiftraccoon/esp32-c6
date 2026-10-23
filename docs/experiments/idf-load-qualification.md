# IDF offered-load qualification v1

Prospective contract, **2026-10-23**. Qualify a finite offered-load generator,
native queue, per-job accounting and two runtime-statistics configurations.
This is separate from a later frozen comparative study. Qualification captures
cannot be promoted into that study or interpreted as sustained capacity.

The [method research](../research/idf-load-method.md),
[prior work](../research/prior-work.md), [reuse inventory](../research/benchmark-reuse.md)
and [qualified accounting](../../results/idf-accounting-qualification-2026-10-23/RESULTS.md)
were consulted before code. Reuse the existing checked integer workload,
nonblocking FreeRTOS queue semantics and SDK timer/task APIs. The Rust
implementation independently adopts nominal-release endpoints and open
arrivals; it does not copy HTTP pacing or a blocking upstream producer.

## Exact workload and release policy

Use the existing Rust 1.96.0 / IDF 5.5.6 / sys 0.37.2 tuple, C6 CPU160/flash80,
100 Hz tick, radios/PM off and existing panic/console/watchdog policy. Both
profiles enable trace/statistics-format support and **two** task-notification
slots. Reserve slot **1** exclusively for timer notifications; pthread uses 0.
`runtime-stats-disabled` leaves statistics disabled and its unavailable counters
are null. `runtime-stats-enabled` enables esp_timer-based U64 runtime counters.
Regenerate and inspect each ABI; never cast a U32 total pointer into U64.
Both images retain the same timestamping, records, queue and workload.

Persistent tasks: coordinator priority1/stack32768, producer priority6/stack8192,
consumer priority5/stack8192; timer TASK dispatch must be observed at priority22.
The native queue has capacity **8**, copying six initialized u32 words: message
kind, global record index, nominal-release low/high and epoch low/high. No Rust
padding, borrowed reference or allocation crosses the queue.

The producer blocks on task-notification slot1 between **one-shot** alarms.
The timer callback only records its atomic diagnostics and notifies the live
producer; no queue offer, payload work, allocation, formatting, blocking or
rearming occurs in that callback. The producer owns rearming and grid policy.
Use `xTaskGenericNotify(...,1,0,eIncrement,NULL)` and
`ulTaskGenericNotifyTake(1,pdTRUE,...)`; every wake must have count exactly one.
There must be no stale notification before a phase or after quiescence.

For a phase with epoch t0, period P, N slots and cutoff T=t0+N*P:

1. Nominal release i is t0+i*P, in `[t0,T)`, regardless of completions.
2. A wake before the next target rearms toward that target without an offer.
3. A wake at/after T marks every remaining slot generator-missed and offers none.
4. Otherwise take one send-entry clock and compute the latest due slot
   `floor((entry-t0)/P)`. Mark older unattempted slots generator-missed. Attempt
   **only that latest slot**, using zero queue wait, recording entry timestamp,
   success/full and immediate queue occupancy. Never replay older offers.
5. Rearm toward the next nominal target. Once all slots are classified, arm an
   additional alarm for T; phase generation cannot finish before its cutoff.
6. In ordinary producer-task context, require `esp_timer_stop_blocking(...,10)`
   to return ESP_OK before reporting generation done. This is the callback
   quiescence boundary, not an assumption that `stop()` waits.

Main records snapshots only while both workload tasks are alive, at their
declared blocked boundaries, queue empty and timer quiescent. It chooses t0 at
least 20 ms after the pre-phase snapshot. Before each phase, perform eight
checked warmup queue transfers; no output occurs until all phases finish.

## Controls and finite pulses

Run exactly this order within each image:

| Phase id | P us | N | Window us | Rounds | First arm relative to t0 |
| --- | ---: | ---: | ---: | ---: | ---: |
| control-no-work | 2000 | 125 | 250000 | 0 | 0 |
| control-delayed | 500 | 16 | 8000 | 0 | 2000 |
| period-2000-us | 2000 | 125 | 250000 | 4096 | 0 |
| period-1000-us | 1000 | 250 | 250000 | 4096 | 0 |
| period-500-us | 500 | 500 | 250000 | 4096 | 0 |
| period-250-us | 250 | 1000 | 250000 | 4096 | 0 |

Before starting the consumer, fill the actual queue with eight known control
messages, require a ninth nonblocking offer to report full, drain/verify FIFO
and require empty. Record this control separately from the 2,016 nominal jobs.
The no-work control requires no missed/full slots, all125 jobs complete and
maximum send-entry lateness <=P/4. The delayed control requires at least its
first four slots missed, at least one accepted/completed job and no queue-full
rejection, with every actual offer still in its own nominal slot.

For each offered job j, compute `workload(0x12345678 XOR global_index, rounds)`
through black_box, recording actual checksum and start/finish clocks. Host
recomputes every accepted checksum independently from the retained index.
The four work phases report **adverse outcomes**, including generator misses,
queue rejection and deadline misses, without replacing the run. A work phase
is generator-qualified only if missed=0 and maximum lateness<=P/4; otherwise it
is explicitly generator-limited. That label does not invalidate its raw data.

Use a fixed **synthetic 2,000 us completion budget** from nominal release in
every phase. This is an experimental reference, not a supplied product deadline.
Retain accepted-only latency distributions and all offered-work failures.
Do not assign zero latency to rejected/missed work or hide it from offered-work
deadline success. Also retain send-entry-to-start and send-entry-to-completion;
send entry is not an independent observation of the instant of queue insertion.

After generation quiesces at/after T, enqueue a separate end-of-phase marker
outside nominal demand and wait for FIFO drain. Require every accepted job to
finish by T+100000us; failed/hung drain retains partial evidence and cannot emit
a successful completion. Main then verifies both tasks blocked at phase+1,
queue empty and no timer callback, and takes the post-phase snapshot. Accounting
therefore covers **whole generation+drain+boundary work**, not exact in-window CPU.
There is no deliberate 10ms blind-spot control before these baselines.

## Storage, publication and interpretation

Preallocate 2,016 records, each six AtomicU32 values: send-entry offset, start
offset, finish offset, checksum, status, post-send queue depth. Payload is
**48,384 bytes**. All offsets are relative to that phase epoch; zero is valid
only with its status. Producer writes status1=generator-missed,
2=queue-full or 3=accepted using Release after its fields. Consumer acquires
accepted status, writes its fields and publishes 4=completed with Release.
Main reads final status with Acquire. Passing final rows contain only1/2/4;
pending0/accepted3 are incomplete, never favorable zero observations.

Producer6 outranks the sole consumer5, so its immediate post-send occupancy
query occurs before consumer progress. Record this queue peak separately from
in-service work or completion-publication backlog. Sampled snapshots alone
would not establish queue peak. Host reconstructs exact conservation:

`N = generator_missed + queue_full + accepted`

`accepted = completed_at_or_before_T + completed_after_T`

For passing completion, all accepted work finishes within the drain bound.
Record deadline_met, deadline_missed_accepted and missing offered work
separately, with `offered_deadline_failures = N - deadline_met`.

Both profiles collect the same gross empty-timer, task-scan and heap-info
brackets, stack-fill and same-capability heap indicators before/after phases.
Snapshot task sets/handles/numbers remain stable; no saved name pointer is
dereferenced after task exit. Enabled counters are monotonic U64 values on the
same clock; disabled runtime fields/total are null, not measured zeros. Never
divide settled task increments by only the250ms offer window or call residual
idle. No global stack/heap peak or instrumentation subtraction is inferred.

Before producer exit, explicitly stop/quiesce and delete the timer. Successful
delete is an accepted deferred SDK deletion, not proof its heap storage has
already been freed. Context/producer task stay alive through quiescence. On a
failed quiescence or callback/control error, retain live task/context ownership
until host reset; no blocking/failing destructor may silently free the callback
argument. No successful complete is emitted on such a failure.

## Protocol and admission

Protocol5, experiment `idf-load-qualification-v1`. The exact
[field schema](idf-load-protocol.md) is fixed before implementation/input freeze.
It retains hello with
profile, roles/priorities/stacks, notification reservation, queue capacity,
buffer bytes and clock config; the queue control; each phase definition and
two snapshots; **every** nominal sample row (including missed/full); generation/
drain/quiescence endpoints; and a final joined/closed/config-after completion.
No output occurs inside offered/service/drain windows. JSON lines must fit the
shared4,096-byte limit and use the existing fatal/extra-boot guard.
Capture uses a bounded **120-second** timeout: UART output of all2,016 samples
occurs after measurement and exceeds the earlier30-second diagnostic budget.
Preparation must check the complete serialized transcript size before attempts.

Host positive/negative controls must challenge latest-slot/forced-delay policy,
duplicate/missing IDs, hidden misses, wrong phases/clocks/profiles/counter widths,
checksum and conservation errors, impossible causal/deadline/drain timestamps,
record status/publication, occupancy bounds, missing quiescence/cleanup,
counter/lifecycle problems, malformed input, reboot/panic and missing completion.
Synthetic positives remain host-only. Replay must reproduce saved results.

Initial physical attempts are qualification-only: one per profile in order
disabled then enabled. Stop on a control/contract rejection; preserve it and
version a necessary correction. Work-phase losses/generator-limited outcomes
are valid observations rather than reasons to repeat. Exact inputs, source,
ABI, images, schedule and reviewed controls are committed before flashing.
The ordinary current full-flash backup/verification/restoration rules apply.

A later runtime-statistics comparison requires new prospective method admission
and frozen comparative captures. Both profiles still have substantial common
timestamp/harness work; neither is an uninstrumented baseline. Any eventual
difference is a **whole-image statistics-configuration effect**, including code/
TCB/layout changes, rather than a universal per-switch cost. Qualification data
cannot become that comparison. Sustainable capacity, real application deadlines,
physical CPU busy/idle, exclusive ISR time, runtime global peaks, radio, energy
and overall platform selection remain outside this finite-pulse qualification.
