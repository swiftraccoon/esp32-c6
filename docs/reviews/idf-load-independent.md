# Independent offered-load preflight review

Reviewed **2026-10-23**, before physical qualification. This agent performed
**no board operations**, source edits or commits. The reviewed implementation
is the new [firmware entry](../../firmware/esp-idf/src/bin/load-probe.rs),
[runtime](../../firmware/esp-idf/src/bin/load_probe/runtime.rs),
[observer](../../firmware/esp-idf/src/bin/load_probe/observer.rs),
[host validator](../../tools/benchctl/src/load.rs) and its CLI/archive integration.
The [qualification contract](../experiments/idf-load-qualification.md) and
[protocol](../experiments/idf-load-protocol.md) are the acceptance references.

The existing [54-entry API inventory](../rust-api-checklist.md) and the affected
[firmware](idf-load-firmware.md), [observer](idf-load-observer.md) and
[host](idf-load-host.md) applicability records were reviewed. These checks
establish a scoped preflight, not a physical pass or completeness assurance.

## Concrete findings resolved before collection

The first host implementation used the cumulative accepted prefix as its
queue-occupancy upper bound. An independently constructed positive contained
all 2,043 records and passed for both profiles. Changing work-phase 2, slot 10
into a queue-full row, with depth 8 and zero service/checksum fields, still
passed the actual CLI. All ten earlier accepted jobs had started and finished
well before that new send-entry; the queue could not still contain them.
The callback count and terminal conservation were unchanged by the mutation.

The corrected guard discounts prior jobs whose start clock is **strictly less
than** the new entry clock: those jobs were already dequeued. This gives a
conservative possible-queued upper bound. A full row needs at least eight
possibly queued predecessors; a successful depth cannot exceed that upper
bound plus the current enqueue. Equal timestamps remain ambiguous. This does
not reconstruct exact insertion times from a pre-call clock. Both the false
full row and a false successful depth-eight row now reject, with maintained
regressions. The original accepted counterexample and result remain under
`.cache/load-independent/`; the earlier executable was not preserved before
replacement, so no old executable identity is invented here.

Source review also identified a new-load capture failure path: successful
framing could be followed by failed checked summary arithmetic, returning
before the final report write and leaving its status `running`. Derivation now
belongs to capture admission. An actual-reader regression supplies a complete
stream whose counter-delta sum overflows; it retains raw bytes, rejects the
boot, and records `failed` with null qualification, phases and score fields.
This was exercised through the maintained test, not through a physical port.

The superficially similar earlier accounting collector was checked rather than
assumed defective. Its completion acceptance already calls `report_fields()`
before returning stream completion. Its new regression confirms rejection at
that point; no production accounting change was needed.

## Firmware ownership and timing trace

The timer callback borrows an immutable Arc-backed context and accesses only
target-supported atomic fields plus initialized task-context SDK queries and
notification. It notifies reserved index **1**. Pthread's index 0 remains
separate. The callback performs no queue offer, workload, formatting,
allocation, blocking or timer rearm.

The sole producer changes its priority to 6 before setup readiness. It owns
one-shot rearming, waits for exactly one notification, chooses at most the
latest due slot and marks earlier slots missed. Consumer 5 cannot preempt the
producer's nonblocking send/depth/status publication sequence. Each copied
queue item is six initialized u32 words; no struct padding or borrowed data
crosses that boundary. Records are fixed atomic storage, with producer Release
publication, consumer Acquire/Release completion and final coordinator Acquire.

The extra cutoff wake prevents generation from finishing at the last release
before the window ends. Per-phase callback count is checked against attempts,
early wakes and that single terminating wake. The producer requires successful
blocking timer stop before reporting Done. Main owns the timer/context through
final quiescence and accepted deletion, then sends Stop and joins both tasks.
Numeric task handles are not queried after exit. No saved SDK name pointer is
retained or dereferenced.

On setup/control/quiescence error, persistent tasks park rather than invalidate
their TCBs. An unclosed TimerOwner retains its bounded context instead of
calling a blocking destructor or freeing a potentially active callback argument.
Arc-backed queue ownership survives task waits. This is failed-boot retention
until reset, not graceful recovery or leak-free successful cleanup. SDK deletion
can be deferred; acceptance does not prove its heap storage is already freed.

Warmups precede pre-snapshots. Observations and output follow the declared
boundaries; no 10 ms blind-spot workload is imported from the earlier accounting
probe. Task deltas span settled generation, drain and boundary work. No exact
offer-window CPU attribution or sum-of-delta conservation is inferred.

## Actual configuration and binary checks

Both isolated generated SDK configurations contain two notification entries,
trace facility and statistics-formatting support. The disabled build has
runtime statistics disabled and generated `TaskStatus_t.ulRunTimeCounter: u32`
with `uxTaskGetSystemState(...,*mut u32)`. The enabled build selects esp_timer
and U64, with the actual field and total pointer both u64. The observer passes
those distinct types directly and represents unavailable disabled counters as
explicit nulls. No pointer cast hides a mismatched ABI.

Both inspected RISC-V ELFs link the callback, one-shot start, blocking timer
stop, task notification send/take, priority change and system-state APIs. This
inspection is separate from the implementation owner's final build/lint/doc
commands and final selected artifact hashes/input freeze. Changed source or
artifacts require corresponding verification; no compiler success is called a
physical qualification.

The independently built complete fixtures contain 2,043 JSON records each:
six phases, twelve snapshots and all 2,016 nominal rows. Their serial sizes are
408,129 and 408,183 bytes. At 115200/8N1 these complete streams need roughly
35.4 seconds of nominal transport; the 120-second capture default avoids inheriting the
earlier short diagnostic timeout. Actual serial duration remains unmeasured by
this preflight. A conservative sixteen-task snapshot width calculation is
2,742 bytes, below the shared 4,096-byte line limit; this is a size bound, not
a valid physical observation.

## Independent CLI counterexamples

The final copied debug collector used for the transcript and archive reruns has
SHA-256 `257de14cd1e64addd66d8bece76f879c51d0e3571dcd79decca740eb7455811b`.
The fixtures were independently constructed from the documented protocol,
including an independent u32 recurrence for every payload checksum. They are
host-only data, not captures or a second validator implementation.

Both complete profile positives pass. Three additional positives pass:

- Zero-round service with equal start/finish microsecond values.
- A work pulse with every slot generator-missed, retaining adverse counts and
  null conditional latency/peak observations.
- Enabled lifetime counter and absolute clocks above u32 range.

All **45 independently mutated transcripts** reject:

| Fault group | Cases |
| --- | ---: |
| Protocol/profile, notification reservation, role alias or missing/null counter fields | 8 |
| Actor epoch, task state/identity/set or stack reservation | 7 |
| Clock declaration or final readback drift | 2 |
| Sample identity/order/status, invented missed observations or broken control | 7 |
| Early or replayed nominal-slot offer | 2 |
| Wrong checksum, zero-duration compute, overlapping FIFO service or drain overrun | 4 |
| Callback count, stale notification, timer stop or join receipt | 4 |
| Impossible queue-full/depth history | 3 |
| Wrong phase round count | 1 |
| Missing or extra completion | 2 |
| Pre-JSON panic/reboot, extra ROM, trailing panic or overlong line | 5 |

Six CLI preflight faults reject before output creation or serial-open failure:
comparison purpose, absent purpose, absent profile, zero timeout, missing ELF
and invalid provenance. A valid qualification request to a deliberately
nonexistent device creates a failed attempt with its original input bytes,
empty raw file, explicit failure and null favorable fields. No physical device
was opened for these tests.

A derived archive graph built from the synthetic positive passes. Eleven graph
faults reject: wrong profile, comparative purpose, absent admission, score,
benchmark field, altered captured summary, failed status with favorable fields,
changed provenance bytes, changed ELF bytes, missing raw and missing report.
A proper failed counterpart remains one failed attempt and zero successes even
when its retained raw happens to be otherwise valid. These synthesized archive
receipts are not claimed to have been produced by an actual positive capture.

## Verification and limits

Independently executed command families:

```sh
cargo +1.96.0 build --locked -p benchctl
cargo +1.96.0 test --locked -p benchctl load::tests -- --nocapture
cargo +1.96.0 test --workspace --locked --quiet
.cache/load-independent/benchctl-final validate-load --profile <profile> --input <fixture>
.cache/load-independent/benchctl-final capture-load --purpose <purpose> --profile <profile> ...
.cache/load-independent/benchctl-final replay-archives --root <derived-graph>
.cache/load-independent/benchctl-final replay-archives --root .
```

The final workspace run passed **96 unit tests** (94 collector, two shared-core)
and **two doctests**. The fourteen-load-test run preceded the new derived-error
regression; the final workspace run includes all fifteen load tests and the
accounting protection regression. Existing archive replay reproduces all
**64 retained protocol transcripts and one study summary**. Native console
suites retain their separate qualification status. Auxiliary independent
fixtures, mutations and command outputs remain under `.cache/load-independent/`;
the changed guards and executable regression mechanisms are retained in source.

No remaining execution blocker was found within these inspected ownership,
ABI, protocol and retention paths. Physical attempts, exact input freeze,
transfer/restoration verification and actual generator controls remain separate
work. Finite pulses do not establish sustained capacity, a real product
deadline, physical CPU busy/idle, exclusive ISR time, global memory peaks,
instrumentation-free performance, external accuracy, energy or an OS choice.
