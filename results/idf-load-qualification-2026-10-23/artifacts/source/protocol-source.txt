# Offered-load protocol 5

Schema for [qualification v1](idf-load-qualification.md), fixed before
implementation/input freeze. Every object denies unknown fields; all listed
fields are required, including explicit null counter fields in the disabled
profile. Durations/absolute clocks use nominal esp_timer microseconds; offsets
are relative to the phase epoch. Integer fields are unsigned unless noted.

Records occur in this order: hello, queue-control, six repetitions of phase /
before snapshot / all nominal sample rows / phase-end / after snapshot, complete.
All records are printed after collection and cleanup, rather than live markers.

## Fields

`hello`:

`protocol:5`, `experiment:"idf-load-qualification-v1"`,
`profile:"runtime-stats-disabled"|"runtime-stats-enabled"`,
`runtime_stats_enabled` (matching boolean), `counter_bits:32|64`,
`counter_clock:null|"esp_timer_get_time"`, `task_capacity:16`,
`coordinator_handle`, `producer_handle`, `consumer_handle`, `timer_handle`,
`coordinator_priority:1`, `producer_priority:6`, `consumer_priority:5`,
`timer_priority:22`, `coordinator_stack_bytes:32768`,
`producer_stack_bytes:8192`, `consumer_stack_bytes:8192`,
`notification_index:1`, `notification_entries:2`, `queue_capacity:8`,
`record_count:2016`, `record_storage_bytes:48384`, `snapshot_count:12`,
`snapshot_storage_bytes` (positive actual reservation), `warmups_per_phase:8`,
`config_before` (existing validated queue-clock/timer object).
Handles are distinct, nonzero u32 values; `timer_handle` identifies the timer
TASK's TCB, not the esp_timer instance pointer. Disabled profile counters are not
physically measured U32 values; that width identifies its generated ABI only.

`queue-control`:

`capacity:8`, `accepted:8`, `full_rejected:1`, `fifo_pass:true`, `empty_pass:true`.

`phase`:

`index` (0–5), `id`, `period_us`, `slots`, `window_us`, `rounds`,
`initial_delay_us` (the exact contract table), `base_index` (cumulative prior
slot counts), `epoch_us`, `cutoff_us` (=epoch+window), `deadline_us:2000`.
Each pre-snapshot ends at least20,000us before epoch. Indices and ids must be
exactly the prescribed order; phase windows may not overlap.

`snapshot`:

`phase`, `boundary:"before"|"after"`, `producer_phase`, `consumer_phase`,
`bracket_start_us`, `bracket_end_us`, `tasks_start_us`, `tasks_end_us`,
`heap_start_us`, `heap_end_us`, `total_runtime_us` (u64 or null), `tasks`, `heap`.
Before actor phase counters equal phase index; after they equal index+1.
All clock endpoints are ordered as in protocol4. Task total, when enabled,
lies within task enumeration; before heap end <=epoch−20000; after bracket
start >=phase-end drain_done_us. All later snapshots follow earlier endpoints.

Each task has numeric `handle`, `number`, `state` (0–3), `priority`,
`runtime_us` (u64 or null), `stack_free_bytes`. Require the same full1–16
task identity set, including all four roles, across all twelve snapshots;
nonzero unique handles/task numbers, stable(handle,number), matching role
priorities, producer/consumer blocked(state2), coordinator running(state0).
Known role stack indicators cannot exceed their reservations. Enabled counters
are monotonic and no greater than total; disabled counters/total are all null.
Heap fields are `free_bytes`, `allocated_bytes`, `largest_free_block`,
`minimum_free_bytes` for MALLOC_CAP_8BIT only; largest/minimum<=current free.

`sample`:

`phase`, `index` (within phase), `global_index` (=base+index),
`status:"generator-missed"|"queue-full"|"completed"`,
`attempt_offset_us`, `start_offset_us`, `finish_offset_us`, `checksum`,
`queue_depth`. These are the final six atomic field values, with the status
decoded into a name. All offsets/checksum/depth are u32; timestamps converted
to absolute values use checked arithmetic. Exactly one row per nominal slot.

Missed slots have all numeric fields zero. Full rows have attempt in their
own slot `[iP,(i+1)P)`, queue_depth8 and zero start/finish/checksum. Completed
rows have attempt in that same slot, attempt<=start<=finish, queue_depth1–8,
and the exact independently recomputed checksum for seed0x12345678 XOR
global_index and the phase's round count. Nonzero-round work additionally
requires start<finish; zero-round controls may quantize to zero service time.
The consumer is serial/FIFO:
completed rows follow accepted sequence order, without overlapping service
intervals. Every finish<=cutoff+100000. A record after an earlier skipped slot
cannot replay that old release; no missing/pending row is silently dropped.
All entries/finishes must lie within the declared phase/drain chronology.
Because each phase starts with an empty queue, a prior accepted job whose
service start is strictly before the current send-entry was already dequeued.
Subtract those proven starts from prior accepted work to obtain a conservative
possible-queued upper bound; equal timestamps stay ambiguous and are retained.
A full rejection needs that upper bound >=8; a successful post-send depth must
be <=upper+1. These are necessary history bounds, not an inference of exact
insertion/dequeue instants or publication backlog from clocks.

`phase-end`:

`index`, `generation_done_us` (>=cutoff, <=cutoff+100000), `drain_done_us`
(>=generation_done, <=cutoff+100000), `timer_stop_pass:true`, `queue_empty:true`,
`notification_strays:0`, `callback_count` (positive bounded<=2*slots+16),
`early_wakes` (bounded<=slots+8). Each completion finish<=drain_done.
Require `callback_count = attempted_rows + early_wakes + 1`, with the separately
primed callback excluded from each phase's delta. The final one is the wake at
or after cutoff. These counters describe wakeups, not nominal arrivals. All wake counts are
checked one-at-a-time by firmware; no callback-count throughput is inferred.

`complete`:

`phases:6`, `nominal_samples:2016`, `producer_joined:true`,
`consumer_joined:true`, `timer_closed:true`, `config_after` (identical supported
clock/timer readback). Further records/fatal text reject the captured attempt
within the shared bounded observation.

## Derived output and admission

Output retains all raw records and derives each phase's generator-missed,
attempted, full, accepted, completed-in-window, completed-in-drain, offered
deadline successes/failures, accepted deadline misses, latency/late distributions
and post-send queue peak. Use completion<=cutoff for in-window, <=nominal+2000
for deadline success. Nearest-rank summaries remain conditional on the observed
attempted/completed subsets; excluded work stays in explicit terminal counts.
No observation is invented for a missed/full row. All checked conservation,
no-work and forced-delay controls in the contract must pass.

Stats-enabled task deltas use settled before/after boundaries, retaining signed
residual and gross observer duration. Disabled attribution is unavailable/null.
Both retain heap/stack indicators, reservations, actual window and drain spans.
No CPU busy percent, global peak, sustainable rate or universal observer
correction is calculated.

The initial capture command requires explicit qualification purpose, preserves
original ELF/provenance bytes and failures, and uses the existing shared boot
guard. Report schema4 admission is always qualification with no method proof;
benchmark/comparative score fields are null. Standalone replay and whole-archive
replay use the actual same protocol validator. Any later comparison needs a
separate reviewed admission extension and newly captured comparison-purpose data.
