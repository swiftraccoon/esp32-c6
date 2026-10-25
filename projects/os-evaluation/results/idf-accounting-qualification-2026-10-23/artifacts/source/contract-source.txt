# IDF accounting qualification v1

Prospective contract, **2026-10-23**. This is a diagnostic qualification, not a
comparative load study or an OS score. It asks whether the pinned Rust/IDF
configuration supplies interpretable task-attributed wall-time and allocator
indicators before we use those methods in a controlled-load experiment.

## Reuse and scope

The [utilization audit](../research/utilization-audit.md),
[prior-work review](../research/prior-work.md) and
[reusable assets](../research/benchmark-reuse.md) were consulted first. Closest
assets are IDF 5.5.6's real-time-statistics example and heap metadata tests.
Independently authored Rust reuses their before/after method, adding stable
live-task boundaries, bounded storage, checked payloads and strict raw replay.
No new C/assembly adapter or copied upstream implementation is needed.

Use Rust 1.96.0, `riscv32imac-esp-espidf`, IDF 5.5.6
`880a203ae603d08a94d06aa01e0d6761958e5e9e`, sys 0.37.2, existing release
settings, CPU 160 MHz, flash 80 MHz, PM/radios off, 100 Hz tick and UART0
GPIO16/17. A separate configuration enables runtime statistics, trace facility,
the esp_timer clock and **64-bit** runtime counters. Regenerate bindings and
verify the counter field/total-pointer ABI; old 32-bit bindings are invalid.
Historical image/configuration bytes stay unchanged.

## Procedure and frozen acceptance

One persistent Rust worker has priority 5 and an 8,192-byte stack. Main has
priority 1 and its existing 32,768-byte stack. Create/prime channels and the
worker before observations; settle for two seconds. Store all snapshots in
fixed storage, print only after collection/worker join. Retain numeric handles
and task numbers, never saved `pcTaskName` pointers.

1. With a real allocated status array but declared capacity zero,
   `uxTaskGetSystemState` must return zero and leave a sentinel total unchanged.
   Do not consume that array as a valid snapshot.
2. Demonstrate the current-task blind spot separately: suspend scheduling for
   a bounded 10 ms busy interval; `vTaskGetInfo` with no stack scan before/after
   must show unchanged stored runtime while esp_timer advances. No allocation,
   logging or blocking in that interval; resume before checking/reporting.
   Accept 10,000–50,000 nominal us; the stored counter must not exceed the
   interval's start clock. This is deliberately excluded from phases.
3. Take `blocked-before`, sleep main for 250 ms with worker continuously blocked
   on its command receive, then take `blocked-after`. Require 200,000–500,000
   us between snapshot total clocks and **zero** worker runtime increment.
4. Send a busy command. Worker repeatedly calls the existing checked
   `integer-mix-v1` workload, seed `0x12345678`, 4,096 rounds, until its own
   esp_timer bracket spans at least 250,000 us. For iteration index i, add
   `0xa3944d55 XOR i` into a wrapping u32 checksum. It sends its start/end,
   count/checksum and immediately blocks on the next command. Only after
   proving phase 1 and blocked state take `busy-after`.
   Accept worker span 250,000–500,000 us, count 1–100,000 and exact checksum.
   Its settled runtime increment from `blocked-after` to `busy-after` must be
   80–120% of that span. Accumulate actual checked workload returns, not the
   expected literal alone. This deliberately
   coarse control distinguishes active work from blocking; it is not an
   accuracy bound or an exclusive CPU percentage.
5. Take `heap-control` with no intervening application allocation. Relative
   to `busy-after`, current free heap must remain within 1,024 bytes. Retain
   this background/control drift rather than correcting measurements with it.
6. Directly allocate 16,384 bytes with **MALLOC_CAP_8BIT (4)**, check null,
   write byte i as `i mod 251`, verify all bytes and sum them, then take
   `heap-held`. Free exactly once, take `heap-freed`. Current free must fall
   by at least 16,384 and at most 17,408 bytes while held, then recover within
   1,024 bytes of `heap-control`. The touched byte sum must be exact.
   No heap-minimum monitor, allocation hook or task-allocation tracer is enabled.

Every snapshot requires the same complete task identity set, including worker,
sampler and idle; unique nonzero handles/task numbers; 1–16 tasks; monotonic
u64 counters no greater than the snapshot total clock; a strictly advancing
total clock; and worker blocked at both
boundaries (state 2). Worker/sampler/idle handles are distinct; sampler state
is running (0), idle priority 0, and only live task states 0–3 are accepted.
Initial phases are 0, subsequent phases 1. Worker/main
priorities and stack-fill indicators must fit their declared reservations.
Detect task lifecycle changes rather than silently dropping tasks. Retain each
snapshot's gross task-enumeration, heap-info and empty-timer-bracket endpoints.
Do not subtract these costs or infer a sum-of-task conservation equation.

Heap snapshots require largest block and lifetime minimum no greater than
current total free. Capability classes are not added together. Stack-fill
bytes are exercised-path indicators; this probe does not positively qualify
stack depth with a separate stack-touch workload.

Read the existing raw clock/timer configuration before/after and reject drift
or unsupported values using the existing decoder. No GPIO fixture beyond the
known UART route is needed. No hardware event timing or energy instrument is
used. Timer agreement shares the board clock and is not external calibration.

## Protocol and capture

Protocol **4**, experiment `idf-accounting-qualification-v1`, is separate from
protocols 1/2/3 and comparative estimators. JSON lines, in exact order:

- `hello`: `protocol: 4`, `experiment: "idf-accounting-qualification-v1"`,
  `counter_bits: 64`, `clock: "esp_timer_get_time"`, `task_capacity: 16`,
  `worker_handle`, `sampler_handle`, `idle_handle`, `worker_priority: 5`,
  `sampler_priority: 1`, `worker_stack_bytes: 8192`,
  `sampler_stack_bytes: 32768`, `target_phase_us: 250000`,
  `allocation_bytes: 16384`, `allocation_caps: 4`,
  `snapshot_storage_bytes` and `config_before`.
- `capacity-control`: returned count 0 and `sentinel_unchanged: true`.
- `open-interval-control`: `start_us`, `end_us`, `runtime_before_us`,
  `runtime_after_us`; the last two must be equal.
- Three `snapshot` records: `blocked-before`, `blocked-after`, `busy-after`.
- `busy-work`: worker `start_us`, `end_us`, `iterations`, `checksum`.
- Three `snapshot` records: `heap-control`, `heap-held`, `heap-freed`.
- `allocation`: `bytes`, `caps`, `checksum`.
- `complete`: `snapshots: 6`, `worker_joined: true`, `config_after`.

Each snapshot has `id`, `worker_phase`, `bracket_start_us`, `bracket_end_us`,
`tasks_start_us`, `tasks_end_us`, `heap_start_us`, `heap_end_us`,
`total_runtime_us`, a `tasks` array of numeric `handle`, `number`, `state`,
`priority`, `runtime_us`, `stack_free_bytes`, and a `heap` object containing
`free_bytes`, `allocated_bytes`, `largest_free_block`, `minimum_free_bytes`.
All fields are required; reject unknown JSON fields, extra/missing/duplicate
records and overflowing arithmetic. Endpoints obey bracket end <= task start
<= task end <= heap start <= heap end, and total clock lies inside the task
enumeration bracket. Each snapshot's heap end precedes the next bracket start;
the busy-work bracket lies between `blocked-after` heap end and `busy-after`
bracket start. Use microseconds as measured by the API, not fabricated
nanosecond precision.

The collector marks every report purpose **qualification**, preserves raw input
and failures, uses the existing single-boot/fatal-text guard, and never admits
these records to a comparative estimator. Replay uses the same validator and
must reproduce retained output. Qualification rejection has no favorable summary.

Host controls must challenge wrong width/clock, missing/duplicate/lifecycle task
identities, zero/truncated snapshots, counter decrease, absent blocked phase,
bad phase/order/clock configuration, zero progress/wrong checksum, malformed
heap inequalities and missing completion. Panic, extra ROM and reboot before
the first JSON must reject through capture and replay, even if valid records
follow. A valid synthetic positive is explicitly host-only evidence.

## Attempt budget and evidence limits

Review source, configuration, host positive/negative controls and compiled ABI
before flashing. Commit exact inputs and this contract before physical attempts.
Schedule qualification boots 01, 02, 03, all the same image, with no automatic
replacement. Stop collection after an unexpected rejection and investigate;
preserve the attempt and version a necessary change. These are qualification
replicates, not an estimated performance contrast or a stopping-rule search.

Preserve a fresh full-flash backup/digest before writing, transfer logs, raw
captures, exact ELF/config/bindings/source/build provenance and separate full
restoration verification afterward. Keep private flash contents outside Git.

Successful controls would qualify this API path for further method development.
They would not establish physical CPU busy/idle, exclusive ISR accounting,
sustainable capacity, runtime global heap/stack peaks, allocator fragmentation,
instrumentation-on/off cost, external frequency accuracy, power or an OS choice.
All-task deltas and their signed residual remain raw descriptive evidence:
current-task open intervals and noninstantaneous scans prevent exact accounting
conservation. A later scored load experiment needs independently specified
arrivals, bounded backlog/deadlines and a separate instrumentation-cost control.
