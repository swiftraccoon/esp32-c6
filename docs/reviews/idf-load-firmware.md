# IDF offered-load firmware review

Reviewed **2026-10-23** against the [coding standard](../coding-standard.md) and
[all 54 active API entries](../rust-api-checklist.md). Scope:
[load-probe.rs](../../firmware/esp-idf/src/bin/load-probe.rs),
[runtime.rs](../../firmware/esp-idf/src/bin/load_probe/runtime.rs), the additive
[manifest entries](../../firmware/esp-idf/Cargo.toml) and
[common SDK defaults](../../firmware/esp-idf/sdkconfig.load.defaults).
The [observer review](idf-load-observer.md) separately covers the fixed snapshot
helper. Reuse the existing complete [IDF](esp-idf.md) and
[queue adapter](esp-idf-queue.md) dispositions; affected decisions are below.

The [method note](../research/idf-load-method.md),
[qualification contract](../experiments/idf-load-qualification.md) and
[protocol](../experiments/idf-load-protocol.md) define the experiment. This
review establishes source/ABI reasoning, not a physical pass or permission to
promote qualification data into a comparative study.

## Execution, storage and ownership

Rust 1.96.0 / edition 2024 / `riscv32imac-esp-espidf`, exact sys 0.37.2 and
IDF 5.5.6 retain the existing release policy and scoped build-std exception.
The required `load-probe` feature selects the additional binary;
`load-runtime-stats` includes it. No dependency, SDK patch, authored C/assembly,
application ISR, DMA or async/cancellation interface is introduced.

One coordinator owns the harness, explicit timer lifecycle, result channels,
join handles and snapshots. Persistent Rust pthreads implement producer and
consumer, with observed priorities 6/5 and 8 KiB reservations; coordinator is
priority 1 with the existing 32 KiB reservation. A task-dispatch ESP timer
callback at observed priority 22 only reads its SDK identity, updates atomics
and sends a nonblocking notification. It performs no payload computation,
formatting, allocation or application wait.

The 2,016 records are allocated once in an Arc-backed Vec before workers/timing;
each record has six AtomicU32 fields, with a compile-time **24-byte** size check.
The payload is **48,384 bytes**, distinct from Vec/Arc metadata, queue/TCB
allocations, twelve snapshots, raw SDK scratch arrays and stacks. The snapshot
Vec reserves twelve entries before collection and does not grow beyond them.
Some transitive Arc/thread/channel allocations can abort on OOM; the source
does not claim every allocation failure is recoverable.

Native queue items are six initialized u32 words: kind, index and split absolute
times. No borrowed pointer or padded Rust record crosses that queue. Its one
allocation is Arc-owned; every call borrows an Arc-backed owner, keeping the
queue live through a wait. The private unsafe Send/Sync implementations rely
on ordinary FreeRTOS task-safe copied-item queues. No queue reset or ISR API is
exposed. Final deletion cannot occur while a borrowed call/task owner remains.

Queue qualification occurs before either worker can consume it: admit eight,
reject the ninth with wait zero, check occupancy eight, receive exact FIFO
words, and check empty receive/occupancy. Per-phase warmups then exercise the
real consumer and known workload answer before each observation boundary.

## Foreign callback and notification lifetime

The callback argument comes from a retained Arc allocation. SDK creation copies
the argument and a static C-string name; neither points into temporary stack
storage. The callback constructs only an immutable reference to WakeContext,
whose mutable fields are target-supported atomics. It does not change Arc
reference counts in the callback. The producer task handle is kept live until
the timer can no longer invoke the callback; Rust never dereferences the TCB.

The common configuration sets two notification entries. Index **1** belongs
to timer-to-producer signaling, leaving pthread startup/join index 0 intact.
The generated task-notify increment is checked; producer wakes require exactly
one notification, with stale counts checked only when the timer is quiescent.
The callback's expected 22-to-6 priority relationship prevents the notify call
from yielding to its lower-priority target. All use the task-context APIs.

The producer alone arms/rearms its one-shot timer. At each phase end it calls
the pinned blocking-stop API from ordinary task context, then checks notification
state and callback identity/result before publishing phase completion. Success
establishes disarm and no in-flight callback. Final coordinator close repeats
quiescence, accepts deletion, and only then permits producer Stop/exit.
The SDK's deferred delete event frees timer metadata without invoking the old
callback; accepted deletion is not immediate observed heap reclamation.

TimerOwner Drop does **not** attempt blocking/fallible stop or deletion. If a
live timer remains on an error path, it intentionally retains an Arc context
reference and leaves the SDK resource for host reset. Producer failures park
the task alive rather than invalidating its callback TCB. This is bounded
failed-boot resource retention, not successful cleanup. The same principle
holds for partially completed worker setup: detached workers retain their own
queue/record Arcs and park on disconnected control paths.

No successful completion is emitted after those failures. Timer/kernel hangs,
OOM/panic abort and unexpected notification failures can still require host
timeout/reset; laboratory recovery is not deployed crash/update recovery.
The panic strategy prevents Rust unwinding across the C callback boundary.

## Arrival, queue and record publication

The fixed `Spec` table bounds every phase/index/period/round count. Nominal
releases stay on `epoch + i × period`; they do not depend on completion. Before
cutoff, the producer attempts only the latest due unattempted slot and marks
older slots missed. At/after cutoff it marks remaining slots missed and offers
no new work. Past-due rearming does not turn those old slots into a burst.
An early wake rearms the existing next target, under a bounded early-wake guard.
The final cutoff wake closes the whole offer interval rather than ending after
the last nominal job.

Entry-before-cutoff bounds the selected index below `slots`; the fixed base
indices bound every direct record write below 2,016. Consumer lookup and phase
membership are checked. Offset conversion uses checked subtraction and u32
conversion. Absolute epochs originate only in the nonnegative signed SDK clock
plus a checked small guard; bounded table additions remain below u64 limits.
Message reconstruction checks its nominal time against phase/index identity.

During offers, producer 6 is the only sender and consumer 5 the only receiver.
Consumer cannot preempt between a successful send, depth query and Release
publication of status 3. Timer-task preemption only notifies the producer; it
does not consume the queue. Thus the immediate post-send query observes the
maximum queued depth caused by that send. It excludes an item already in service;
queue depth is not the complete outstanding-work count.

The consumer first acquires status 3, validates the message identity, timestamps
start, executes the actual workload and timestamps finish. It stores the actual
result/offset fields and publishes status 4 with Release. The coordinator's
Acquire read of final status, followed by relaxed field reads, follows that
publication chain. Skipped/full records similarly publish their final fields
with Release. All shared record fields are atomic; there is no unsynchronized
Rust buffer alias or mutable reference handed to C.

**Per-job checksum correctness is checked by host admission after capture.**
Firmware warmups check the known default answer; offered jobs record the actual
result for their varying seed. The measured path does not run a second payload
to validate the first. Start/end bracket the workload and exclude later atomic
record stores; that bookkeeping still consumes the machine between jobs and
remains part of complete-pipeline behavior. No counter/observer subtraction is
performed.

The coordinator waits for generation completion/quiescence, enqueues a distinct
non-scored end marker and checks the bounded drain receipt. FIFO requires all
prior accepted jobs before that marker. Producer and consumer then block on
their respective control paths; their phase counters and actual SDK blocked
states are checked before snapshots. The lower-priority coordinator cannot
outrun their next blocking transition after a successful reply. No external
task deletion or cancellation is supported.

## Observation, errors and output

All workload actors stay alive from the first before snapshot through the final
after snapshot. System task names/stack pointers are discarded by the observer; only
numeric identities survive. Snapshot scans and heap queries remain outside
offer windows. Enabled counters cover the whole settled offer-plus-drain phase,
including boundary work; the sampler's open interval prevents an exact
sum-of-task conservation equation. Disabled counters are unavailable/null.

The direct source review identified status-loss diagnostics: initial SDK/queue
errors discarded numeric return values and worker branches replaced their
errors with generic messages. Before freezing or flashing, the implementation
was corrected to preserve numeric SDK/unexpected queue results and forward
worker errors into failure diagnostics. The callback retains the actual notify
status atomically for its quiescent check. Failure formatting/allocation remains
outside the successful measured path. This is a resolved preparation defect,
not a failed physical attempt or evidence of a compiler incompatibility.

Successful Stop/join precedes JSON serialization. After stopping actors, no
handle is queried; saved snapshots contain no borrowed SDK pointer. Each final
record must be skipped, full or completed; pending/accepted-but-unfinished data
cannot produce a complete transcript. Main flushes output explicitly; I/O
failure does not manufacture a favorable completion. A partial/error boot can
precede all JSON, so shared raw capture/fatal/reboot/timeout admission remains
essential.

Queue Drop performs SDK delete/free after the final Arc owner, with ordinary
SDK heap/kernel critical sections; it adds no application wait/join. Timer
close and task joins are explicit operations. There is no claim that every SDK
lock has zero cost or a qualified worst-case bound.

## Affected API checklist decisions

| Entries | Decision / evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: named roles/commands/specs and explicit time/byte/index fields preserve Rust conventions and meanings. |
| C-FEATURE | Applies: additive required binary and runtime-statistics feature; each real SDK configuration needs its own build and provenance. Arbitrary feature/config combinations are not advertised. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies to useful private records: Specs/results/identities are Copy/Debug, atomic records have field-bearing Debug/Default. Live queue/timer/thread owners are not Copy; no public library requires opaque private owners to gain a new trait contract. |
| C-SERDE | Applies: explicit numeric protocol-5 output after cleanup, strict typed host deserialization, actual unavailable fields null; no reusable interchange library is exported. |
| C-SEND-SYNC | Applies: private Queue's two unsafe trait implementations carry the kernel/Arc/copy invariants; wake/sample state uses atomics and ordinary Arc ownership. No fake synchronization is assigned to volatile. |
| C-GOOD-ERR, C-FAILURE | Applies: Results and explicit failed-boot parking; corrected diagnostics retain raw unexpected foreign codes and context. No ordinary success-path panic or unwinding through C. |
| C-VALIDATE | Applies: foreign return/null checks, queue controls, known-answer warmups, role priorities, sequence/phase/release publication, bounded wake/drain and explicit shutdown. All varying offered-job checksums and full protocol consistency additionally require host admission. |
| C-CUSTOM-TYPE, C-NEWTYPE, C-CTOR, C-METHOD | Applies: owned queue/timer types, typed actor commands/results/specs and named units/roles; private defaults reserve empty records, not a valid public result. |
| C-STRUCT-PRIVATE, C-HIDDEN | Applies at private binary scope: only coordinator integration needs parent-visible items; no SDK queue/timer owner is exposed as a public Rust library API. |
| C-RW-VALUE, C-CALLER-CONTROL | Existing generic writers accept values, permitting mutable borrows; output is performed explicitly after collection. Observer output borrows immutable stored records. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Applies: explicit fallible timer close/join, nonblocking failure-retention fallback, final-owner queue deletion with no live waiter. SDK internal free/critical-section cost is acknowledged. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE | Applies at internal application scope: module comments, method/contract/protocol and these reviews distinguish commands, builds, actual captures and limitations. |
| C-NO-OUT, C-INTERMEDIATE | Applies: actual outputs and flags are retained; failed/pending work is not converted into a favorable numeric score or repaired input. |

Other existing all-54 dispositions remain: no new macro API, conversion/operator
traits, iterator/collection abstraction, smart-pointer implementation or async
interface. Reused timer/configuration helpers retain their prior pinned FFI/MMIO
invariants. No metadata/license exception is silently broadened.

## Inspected build evidence and remaining gates

This review independently inspected both local generated configurations and
bindings. Both enable trace, statistics-formatting support and two notification
entries. Off disables runtime statistics with U32 field/total-pointer types;
on enables U64 and the esp_timer counter clock with U64 field/total-pointer types.
No cross-width pointer cast adapts those ABIs.

Initial owner-run target builds completed in 53.47 s off and 53.12 s on. Initial
target Clippy/documentation logs were read (off 4.96/2.98 s, on 3.63/1.88 s).
Final owner-run logs after diagnostic preservation also show clean target
Clippy, documentation and build completion (2.39 / 2.90 / 5.23 s in each log).
Those logs include package-cache waits; these are local command observations,
not an engineering-effort comparison. Exact commands, exit statuses and final
image identities belong to the integration record. This reviewer read the
retained logs and did not independently rerun those builds.

Independent DWARF inspection of both prepared outputs reports common Record
size **24 bytes**, alignment 4, and Snapshot size **744 bytes**, alignment 8.
Thus record payload is 48,384 bytes and twelve stored snapshots 8,928 bytes in
both profiles. These are particular harness reservations, not total RAM cost
or application headroom. Root separately inspects actual linkage/frame/placement
and preserves exact final images/configurations.

No concrete memory-safety or execution blocker remained in this reviewed source
scope after the diagnostics repair. Actual timer cadence/skip behavior, overload
outcomes, deadlines, capture controls and physical restoration still require
their recorded qualifications. On/off results describe complete configuration
images, including placement/TCB changes; they cannot supply a universal counter
correction, sustainable rate, global memory peak, physical busy/idle, exclusive
ISR accounting or an OS ranking. This review made no source changes and
performed no board operation or commit.
