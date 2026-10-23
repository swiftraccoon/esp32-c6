# IDF accounting firmware review

Source review, **2026-10-23**, against the [coding standard](../coding-standard.md),
[all 54 active API entries](../rust-api-checklist.md) and the existing complete
[IDF](esp-idf.md) / [queue adapter](esp-idf-queue.md) dispositions. This is an
affected-entry delta, not a replacement for those inventories.

Scope: [accounting-probe.rs](../../firmware/esp-idf/src/bin/accounting-probe.rs),
its binary/feature in [Cargo.toml](../../firmware/esp-idf/Cargo.toml), and isolated
[statistics defaults](../../firmware/esp-idf/sdkconfig.accounting.defaults).
The reused timer module retains its existing pinned-source/MMIO review. Source
reasoning below is separate from generated-ABI, linked-binary, stack-frame and
physical qualification evidence, which remain explicit verification items.

## Execution and ownership

The selected target remains Rust 1.96.0, edition/style edition 2024,
`riscv32imac-esp-espidf`, IDF 5.5.6 and exact sys 0.37.2. No new dependency,
authored C/assembly, application interrupt handler, callback, DMA transfer,
async cancellation interface, unsafe trait implementation or mutable static is
added. Standard allocation, pthreads/channels and existing SDK interrupts remain.
The new required feature preserves the historical default binary selection.
The five statistics defaults describe intended configuration; actual generated
configuration was separately inspected below. A `dead_code` allowance applies
only to the reused timer module, with its reason: this binary needs configuration
and public-microsecond helpers while the existing raw-tick helpers stay unused.
It does not suppress unrelated target warnings or alter the historical module.

Main owns the result storage and stdout. One Rust pthread worker owns command
receive/reply send endpoints and a shared Arc-backed atomic phase. Worker
identity is sent once, retained numerically and only passed back to SDK task
queries while the worker remains alive. The only normal worker exit is Stop,
sent after all observations. Main makes no handle queries after Stop. Ordinary
channel failures park the worker alive rather than exposing an already-deleted
handle. A corrupt kernel or disconnected control path can prevent graceful
shutdown; host reset/failed capture is recovery, not successful cleanup.

Prime performs checked work and channel exchanges before the observations.
The worker stores phase 0/1 with Release after sending its result, then waits
for its next command. Main uses Acquire, the actual blocked state and a second
Acquire. Along successful control flow, the only following application wait is
command receive. The higher-priority worker can finish sending, publish phase
and block before lower-priority main resumes; actual priorities are checked.
This establishes the intended boundary in source, not proof from a source
comment that a physical blocked observation occurred.

## Unsafe and FFI boundaries

| Boundary | Invariant and source finding |
| --- | --- |
| Current/idle/worker handles and priority/state queries | Initialized IDF task context; the worker is persistent until authorized Stop and idle persists for scheduler lifetime. Numeric handles are never dereferenced by Rust. Handle-to-u32 conversion is specific to this 32-bit target, not a portable host API. |
| Zeroed generated status/info records | Bindgen records contain integers and raw pointers with valid zero representations. No zeroed Rust reference, owned allocation or invalid Rust enum is constructed. The actual regenerated definitions still require inspection. |
| Capacity control | A live allocated status record and aligned u64 sentinel are supplied with capacity zero. The pinned SDK path must return zero without writing the sentinel. The array is never consumed as a successful empty snapshot. |
| System snapshots | Fixed 16-record array is live and aligned for the call; zero or over-capacity returns fail before slicing/reading results. Only returned records are copied. Task-name and stack pointers are neither retained nor dereferenced. Passing `&mut u64` and assigning the runtime field to u64 intentionally fail compilation against the old u32 ABI. |
| Current-task blind-spot control | Scheduler suspension begins only after record setup. The bounded loop contains timer/task reads and scalar workload, with no allocation, formatting, blocking, fallible return or panic path. Interrupts remain enabled. Resume occurs unconditionally before validation or error construction. |
| Heap info | A writable generated record and the explicit byte-capable mask are supplied in initialized task context. No reference survives the call. The result is copied into private numeric storage. |
| Controlled allocation | Check non-null before access; SDK allocation provides at least 16,384 writable bytes for caps 4. Every pointer offset is in range, every byte is initialized before reading, and no Rust reference aliases it. Volatile ensures actual touches; it supplies no inter-task synchronization. |
| Free and observation errors | Held-snapshot Result is retained, the pointer is freed once, and the freed snapshot is attempted before propagating the held error. No ordinary fallible return occurs while the pointer remains owned. No pointer/reference survives free. Process-aborting panic/reset is not claimed to run cleanup. |
| Reused timer/configuration | Read-only, pinned and aligned C6 MMIO and initialized timer FFI retain their prior invariants. The probe does not take peripheral ownership or change clocks. Actual before/after readback and supported decoding are still capture admission checks. |

The allocation checksum cannot overflow u32: at most 16,384 bytes each at most
250 contribute. Busy iteration count is bounded at 100,000; its accumulated
checksum deliberately uses wrapping addition. Elapsed work uses checked
subtraction. The open-interval loop uses saturating subtraction and separately
rejects a negative start or duration outside its frozen guard.

## Blocking, errors and output

Channel creation, Arc/thread creation and any runtime initialization allocate
before the measured snapshots. Some transitive standard-library allocations
can abort on OOM; the code does not make every allocation failure recoverable.
The direct qualification allocation checks null explicitly. Command/reply waits,
Stop and join can block. Join and stop are explicit operations outside the
observations; no authored destructor hides them. The phase-state wait has a
bounded timer guard, while a stuck timer/kernel can still require host recovery.

Ordinary timer, channel, checksum, priority and configuration errors return
`io::Error`. Main prints a diagnostic and emits no successful completion.
Panic remains `abort`, with the SDK print/reboot setting; Rust cannot unwind
across a C boundary. Errors before first JSON can leave no protocol prefix, so
the collector's retained boot text, fatal/reset guard and timeout remain
necessary. A later successful reboot must not rehabilitate the first attempt.

All JSON and stdout locking occur after snapshots, worker Stop and join. IDs
are closed literal strings and data interpolation is numeric; no dynamic string
escaping is assumed. Errors propagate through `Write` and final flush. The
first stdout allocation is therefore outside the observed heap snapshots;
those snapshots exclude later formatting/output memory and are not complete
application memory peaks. Records are manually serialized after observation,
consistent with the existing closed firmware protocol; strict typed parsing is
owned by the host collector.

Fixed snapshot storage is reported by `size_of_val`. That number describes the
six-record array, not all transient raw status arrays, compiler copies, task
stacks, channel/Arc allocations or SDK profiling cost. Compiled frame evidence
and physical stack-fill observations remain necessary; source-level reservations
alone do not establish safe exercised stack margin.

## API applicability delta

The unchanged all-54 decisions, narrow build-std/bootstrap exception and private
package metadata/licensing exceptions are reused. No release, public library
API, trait/macro/iterator/collection implementation, operator/Deref implementation,
smart-pointer implementation or generic data structure is introduced.

| Entries | Decision / evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: named identity/phase/snapshot/heap/work records retain Rust casing and explicit microsecond/byte fields. |
| C-FEATURE | Applies: additive required `accounting-probe` binary feature; historical binaries keep their selection and SDK defaults. Isolated actual build evidence is required. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: private scalar records derive useful Copy/Clone/Debug and bounded storage derives Default. No live allocation/thread owner becomes Copy; Debug has no hardware reads. Ordering/equality traits add no required public contract. |
| C-SERDE | Applies: explicit numeric protocol-4 serialization, static record IDs and post-observation writing; typed strict deserialization belongs to the host. No reusable interchange library is exposed. |
| C-SEND-SYNC | Applies: safe owned channel transfers, Arc and target-supported AtomicU32 Release/Acquire phase publication. Raw handles remain numeric with procedural lifetime invariants; no custom sharing implementation is authored. |
| C-GOOD-ERR, C-FAILURE, C-QUESTION-MARK | Applies: contextual io::Error and Result propagate ordinary failures; OOM abort, blocking cleanup, panic/reset and incomplete-output limits are explicit. |
| C-RW-VALUE | Applies: generic writers are accepted by value, permitting callers to pass mutable borrows. Review flagged the initial Snapshot writer's `&mut impl Write`; source now uses `mut output: impl Write`, matching the reused configuration writer. |
| C-CRATE-DOC, C-LINK | Applies at private application scope: module docs, experiment/research/review records describe target and effects. Hardware commands/builds do not establish a physical pass. |
| C-EXAMPLE | Existing not-applicable disposition: no public Rust API is exported; the experiment/build documentation provides the hardware-use contract. |
| C-NEWTYPE, C-CUSTOM-TYPE, C-CTOR, C-METHOD | Applies: enum commands and named record fields distinguish phase, identity, time and bytes; private Default is overwritten observation storage, not a valid public measurement constructor. Snapshot JSON output is associated behavior. No generic scheduler abstraction is needed. |
| C-NO-OUT, C-CALLER-CONTROL, C-GENERIC | Applies: authored operations return Results/value records, fixed buffers remain under main ownership, and unsafe foreign output pointers stay local. Standard writer/channel interfaces suffice. |
| C-INTERMEDIATE | Applies: retain task totals, individual counters, heap metadata and gross operation brackets without an observer subtraction or fabricated utilization total. |
| C-VALIDATE | Applies: live-task boundary, phase/state, width-sensitive ABI, capacity, checked workload, payload, priorities and before/after configuration checks. Full temporal/identity/heap contract admission is additionally a host responsibility. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Applies: no authored destructor; direct heap free, Stop, join and output flush are explicit. SDK/kernel failure can prevent graceful termination without producing success. |
| C-STRUCT-PRIVATE, C-HIDDEN, C-NEWTYPE-HIDE | Applies: implementation and scalar record representations remain private; no public ABI or reusable raw task/allocation handle is exposed. |
| C-STABLE | Applies: authored code uses stable Rust; existing scoped Cargo build-std/bootstrap exception remains. Generated SDK ABI and private timer linkage are pinned and require renewed review on SDK changes. |

## Verification status

This review inspected the actual firmware source, manifest/defaults, reused timer
module, pinned SDK paths and existing all-54 review records. The writer-signature
item was corrected before final checks.

The reviewer additionally inspected the new isolated build's generated bindings
under `firmware/esp-idf/.cache/accounting-target/`: `ulRunTimeCounter` is u64 and
`uxTaskGetSystemState` takes `*mut u64` for the total. Its actual SDK config
enables statistics, trace, formatting, U64 and esp_timer; it records the 32,768
main stack, default pthread priority 5 and ROM heap implementation. These are
build/configuration observations, not a physical priority or memory result.

The locally provided disassembly contains real `uxTaskGetSystemState`,
`vTaskGetInfo` and heap-info calls; `heap_caps_get_info` calls ROM
`multi_heap_get_info` at `0x400004a0`. Inspection of the provided prologues shows
main reserving 256 plus 12,736 bytes, `snapshot` 1,824 bytes,
`allocation_control` 1,248 bytes and `open_interval_control` 144 bytes.
These are per-function allocations in that inspected image, not a whole-chain
bound or proof of safe exercised margin. Final retained binary/disassembly
identities must agree after the writer correction. The owner reports DWARF
Snapshot size 592/alignment 8, so its six-record storage is 3,552 bytes; this is
not total harness memory.

The implementation owner reports successful final isolated formatting,
warning-denied target Clippy, documentation and release build after the writer
change. That report is distinguished from the reviewer's direct source,
generated-ABI/configuration and disassembly inspections. The owner retains exact
commands, logs and the initial isolated-directory setup failure in the build
journal. Neither the completed checks nor the corrected setup establish a board
pass.

Pending physical evidence: prospective input freeze, actual controls/captures,
retained rejection handling, exercised stack-fill observations and separate
verified restoration. Host protocol counterexamples are reviewed separately.
The owner records actual outcomes before finalizing an evidence collection.

The [method research](../research/idf-accounting-method.md) explains why blocked
versus busy attribution is not physical busy/idle, ISR-exclusive time, timer
accuracy or energy. Heap observations and stack-fill indicators do not establish
global runtime peaks, fragmentation, sustainable capacity, profiling-on/off cost
or a platform choice. No broader correctness/completeness assurance follows.
