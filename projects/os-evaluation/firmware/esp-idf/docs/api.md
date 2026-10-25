# ESP-IDF adapter API and safety contract

This record covers the default compute/handoff binary and the queue variants.
The [README](../README.md) contains build commands, feature selection and serial
settings. [Accounting](accounting.md), [offered load](load.md) and its
[observer](load-observer.md) refine their own FFI and cleanup contracts.
The [coding standard](../../../../../docs/coding-standard.md)
and [API inventory](../../../../../docs/rust-api-checklist.md) govern this record.

## Default compute and handoff

### Execution, ownership and failure paths

Target is `riscv32imac-esp-espidf`, Rust 1.96.0 / Rust and formatting edition 2024,
`std` + ESP-IDF 5.5.6 / IDF FreeRTOS. The main task owns fixed sample arrays
(18,048 bytes), stdout and the initiator endpoints. A Rust thread owns the
opposite endpoints, sends its actual task priority, then replies with the
sequence XOR mask until the request sender closes. Worker stack request is
8,192 bytes, main stack configuration 32,768 bytes, tick rate 100 Hz. No shared
mutable static, application interrupt handler, callback, DMA, raw task handle,
custom atomic protocol, async function or unsafe Send/Sync impl is authored.

Both queues have capacity one, exactly one request is outstanding, and each
reply is checked outside its timed interval. Two seconds of settling and eight
warmups for each workload precede timing. The default higher-priority worker
(priority 5 versus main's 1; actual values emitted) blocks on its next request
before the main task resumes. Thus a request send can wake/preempt into the
worker and its next receive can block back into the initiator. The result
includes Rust synchronization and task scheduling; it is not an isolated kernel
context-switch cost or a native FreeRTOS queue benchmark.

The std 1.96.0 bounded channel uses a preallocated array, a cached per-thread
context and a waiter Vec that retains capacity after removal. Startup/channel
creation and eight handoffs precede timing. One outstanding request ensures no
queue growth and no successful sender waits on a full queue. The worker's
empty-receive path is primed before timing. No authored allocation occurs per
message; no allocator trace was taken, so broad allocator-free claims about std
are unwarranted. Mutex acquisition, wakeups and blocking inside channels are
part of this particular workload. No I/O/formatting/sample allocation occurs
during timing. The worker is blocked during compute/observer samples.

Successful and ordinary error paths explicitly drop both initiator endpoints
and join the worker after timing. Closing the request sender releases a blocked
worker; closing the reply receiver releases a failed send. No authored Drop
implementation blocks. Channel errors preserve operation context, thread
creation errors propagate, and invalid replies or clocks cause incomplete runs.
The firmware has no stalled-task timeout; the host capture deadline/reset is
its recovery boundary. A task that never progresses can also prevent join.
Panic abort prevents Rust unwinding across the SDK entry point and makes partial
runs invalid. A final output flush can fail after completion was queued; host
validation uses the bytes received.

### Unsafe and clock evidence

`TimestampMicros::now` retains its generated, no-pointer timer FFI call and
checked elapsed conversion. `current_task_priority` calls generated
`uxTaskPriorityGet(null)`, whose null task handle means the current task in
normal FreeRTOS task context. It performs no cross-task handle manipulation.
The values are reported, not remapped to an invented common priority scale.

`Clocks::read` has four `read_volatile` operations at aligned, mapped ESP32-C6
clock-configuration addresses: generated `DR_REG_PCR_BASE` plus 0x110, 0x118 and
0x1c, and generated `DR_REG_SPI0_BASE` plus 0x14. No Rust reference is created,
no register is written, and none is FIFO/read-to-clear state. The safe private
wrapper offers no caller-supplied address. PM is disabled and no authored task
changes clocks. Volatile implements MMIO reads; it is not synchronization.

IDF v5.5.6 `pcr_reg.h`, `spi_mem_reg.h` and `clk_tree_ll.h` define these fields.
Only PLL source 1, fixed root divider 3, CPU dividers 1/2/4, and MSPI dividers
4/5/6 are accepted. CPU frequency is 480 MHz divided by root and CPU dividers,
except the documented force-120 path. Flash source is PLL480 / MSPI divider;
SPI0 bit 31 bypasses its bus divider, otherwise divider is N+1 from bits 23:16.
The wrapper rejects fractional-MHz flash values and requires exact raw
before/after equality. Supported decoded values outside the trial baseline are
reported for diagnosis; the host requires CPU 160 MHz / MSPI 80 MHz /
flash 80 MHz / divider 1.
No clock API exposed by the selected generated bindings reports these bus
dividers, so reading the four registers avoids an authored C shim or unsupported
clock writes. Official SDK configuration selects DIO80; independent runtime
readback, not the image header, establishes the observed clocks. The host
independently validates the same raw register fields.


## Queue variants

### Context, ownership and safety

Rust 1.96.0 / IDF 5.5.6 `std`, on `riscv32imac-esp-espidf`, runs a main task and
one Rust pthread worker. No application ISR, callback, DMA, async operation,
packed structure, authored C/assembly, custom allocator or clock write is added.
Existing SDK interrupt/idle/timer contexts remain active. Cancellation is not
supported: no task is externally deleted and no pending foreign call is dropped.
Panic abort retains the existing SDK print/reboot behavior.

Both backends transfer aligned owned `u32` values through two capacity-one
objects. Native FFI uses generated queue bindings, with pinned typed macro
constants for ordinary queues and send-to-back. Null allocation and foreign
status returns are checked. Send/receive pointers refer to aligned initialized
four-byte locals, remain live through the entire call, and are not retained
after it returns. A receive failure cannot expose uninitialized Rust memory.
Indefinite task waits preserve the stated workload; hangs require host reset.

Each native queue's `NonNull` handle is owned by one `Arc<Queue>` allocation.
Every active operation borrows an Arc-backed endpoint, so the final owner cannot
delete a queue while any send/receive is still blocked. The only custom unsafe
`Send` and `Sync` implementations are this private resource handle: FreeRTOS's
ordinary queue APIs provide task-safe synchronization; no mutable Rust view of
queue storage exists. Final `Drop` deletes once without waiting for a task.
There is no queue reset, ISR API, exposed raw handle or authored blocking
destructor. Error/panic paths never unwind into C.

Queue and sample allocations occur before timing. The fixed 2,000-record Vec
reserves capacity fallibly, then initializes it without growth. Rust std/Arc
allocation can still abort on OOM; the wrapper does not claim every transitive
allocation failure is a recoverable Result. Runtime high-water/leak/allocator
tracing was not performed.

The worker publishes an Acquire/Release sequence protocol using target-supported
atomics. Only a Release phase store precedes its intended request receive;
there is no alternative application blocking path between them. Main checks
the exact phase, live task handle's actual `eBlocked` state, native queue
occupancy when available, the unchanged phase and no worker failure. Std does
not expose equivalent public wait-list/occupancy internals; its single-inflight
path is traced and the native-only query count is kept separate. This is the
specified receive-path qualification, not an isolated native waiter-list proof.

The worker normally exits only on the reserved sentinel sent after main's last
query. On an ordinary worker error it publishes `failed` and parks until main
authorizes exit. Thus `eTaskGetState` cannot query a task already deleted by an
error return. Main authorizes exit, unparks, sends stop when applicable, drops
its endpoints and joins explicitly after timing. Lost unpark notifications are
avoided by Rust's token semantics plus the atomic predicate. A corrupt/stuck
kernel can still prevent graceful stop/join; no successful complete is emitted.

The higher-priority worker's next phase store and re-blocking may execute before
the reply's end timestamp. This instrumentation is identical across variants
and remains part of the measured whole API path. The main readiness queries,
correctness probes, formatting and teardown are outside measured intervals.

Raw time uses one narrowly pinned, pointer-free private C declaration. Both
images link its SDK wrapper and the same vendor ROM snapshot routine. Seven
fixed read-only MMIO configuration accesses are aligned/mapped, retain no
references, have no read-clear effect and do not claim peripheral ownership.
Volatile is not the inter-task synchronization mechanism. Actual source,
enable and CPU-stall fields must match before/after; counter backsteps/over-width
or >1 s message intervals are errors. There is no silent wrap normalization.

SDK WFI is called only after startup, with normal enabled interrupts, no
application critical section/lock, worker blocked and no attached debugger.
The active tick interrupt provides progress; bounds and the separate legacy
bracket are checked. This neither installs an idle hook nor enters configured
light sleep. The final binary includes the SDK WFI instruction; on-board
qualification is still required to establish that the admitted code executes
as expected on this device.


## API applicability

The table records the common application decisions and the queue-specific
ownership boundary. The default binary uses typed Serde output and has no
native queue handle. The accounting/load contracts refine their own FFI and
cleanup obligations.

| IDs | Outcome | Evidence / reason |
| --- | --- | --- |
| C-CASE, C-WORD-ORDER | Applies | Consistent Rust casing and explicit tick/byte/phase names; selected rustfmt passes. |
| C-CONV | Applies | `elapsed` is an operation returning ticks; no misleading as/into conversion is introduced. |
| C-GETTER | Not applicable | No stored-field getter API is authored. |
| C-ITER, C-ITER-TY | Not applicable | No collection/iterator interface is defined; standard bounded slices/iterators are consumed. |
| C-FEATURE | Applies | Required `queue-api` feature preserves the default historical binary; `native-queue` includes it and selects the documented private application backend. Both configurations are separately built/linted/documented; no published library feature-unification contract is implied. |
| C-COMMON-TRAITS | Applies | Private Sample is Copy/Clone/Default/Debug; value snapshots derive equality and Debug; Tick derives ordering. Live Queue/endpoint ownership is deliberately not Copy/Clone/Default. |
| C-CONV-TRAITS | Applies | Signed legacy timer and sample index conversions are checked; no bespoke conversion impl is needed. |
| C-COLLECT | Not applicable | No collection implementation is authored; one reserved initialized Vec and upstream capacity-one channels/queues are used. |
| C-SERDE | Applies | Private transient records are explicitly serialized to protocol 3 after timing; the host owns typed strict deserialization. No interchange API or additional firmware serialization dependency is needed. |
| C-SEND-SYNC | Applies | Arc-backed native queue sharing has documented task-safe FFI/lifetime invariants. Std endpoints retain upstream traits. Sample values and atomic phase/status are shared through appropriate safe interfaces; no unsynchronized mutable static exists. |
| C-GOOD-ERR | Applies | `io::Error` preserves operation/status context; ordinary failures print raw partial samples without success. Foreign unexpected statuses retain their integer value. |
| C-NUM-FMT | Not applicable | Tick is a private absolute marker, not a public primitive-like numeric/bit-operation API. Raw result integers retain normal primitive formatting. |
| C-RW-VALUE | Applies | Configuration writer takes `impl Write` by value; callers pass a mutable borrow to retain writer control. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable | No macro interface is authored; formatting/derive macros are consumed. |
| C-CRATE-DOC | Applies | Binary-level documentation links its experiment; build/review records document target, feature and failure contract. |
| C-EXAMPLE | Not applicable | Binary exports no reusable public Rust API; build record gives actual target commands. |
| C-QUESTION-MARK | Applies | Fallible operations use `?`; no unwrap/expect is introduced. |
| C-FAILURE | Applies | FFI invariants, indefinite waits, OOM abort, panic/reset, timeout and incomplete-output behavior are documented. |
| C-LINK | Applies | Experiment, build, reviews, raw artifact hashes and pinned SDK/source audits are linked. |
| C-METADATA | Exception | Existing publish=false private package has real metadata; project ownership/license/repository metadata is not invented. Revisit before distribution, as previously recorded. |
| C-RELNOTES | Not applicable | No versioned release or publication occurs; protocol migration is separately documented. |
| C-HIDDEN | Applies | All application implementation and representations remain private to this binary; crate visibility is not a library export. |
| C-SMART-PTR | Not applicable | Standard Arc is consumed; no smart pointer implementation is authored. |
| C-CONV-SPECIFIC | Not applicable | No authored conversion interface between authored types exists. |
| C-METHOD | Applies | Queue operations and Tick/configuration behavior are associated methods; constructor returns a paired channel. |
| C-NO-OUT | Applies | Safe interfaces return values/Results. Foreign receive out-pointer is encapsulated around a typed initialized local. |
| C-OVERLOAD, C-DEREF | Not applicable | No operator or Deref implementation is authored. |
| C-CTOR | Applies | `channel`, `Tick::now` and `Configuration::read` make construction explicit; Default is only for overwritten sample storage. |
| C-INTERMEDIATE | Applies | Raw timestamps, checksums, configuration and observer records survive; ordinary failed measurements retain their captured raw prefix. No observer subtraction occurs. |
| C-CALLER-CONTROL | Applies | Bounded preallocated storage, owned scalar messages and borrowable writers preserve necessary control. No general-purpose queue API is exposed. |
| C-GENERIC | Applies | Standard writer trait suffices; no generic scheduler framework is introduced for a fixed experiment. |
| C-OBJECT | Not applicable | No authored trait or dynamic-dispatch interface exists. |
| C-NEWTYPE, C-CUSTOM-TYPE | Applies | Tick distinguishes absolute hardware ticks from legacy microseconds and kernel ticks. Named Sample/Configuration/WorkerIdentity/Qualification records identify units and roles. |
| C-BITFLAG | Not applicable | Hardware bitfields are privately decoded from primitive registers; no combinable flag API is defined. |
| C-BUILDER | Not applicable | No builder API is authored; std thread Builder is consumed. |
| C-VALIDATE | Applies | Null/status, empty/full, payload, phase/state, priorities, clock/timer fields, continuity, timestamp width/order and duration are checked. Host validates corresponding emitted fields independently. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Applies | Last-Arc queue deletion is infallible/non-waiting under the SDK contract. Fallible/blocking join, stop and flush are explicit outside measurement; stuck-kernel limitation is stated. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies | Private value types have meaningful side-effect-free Debug. Private live queue handles expose no Debug API that reads hardware or duplicates ownership. |
| C-SEALED | Not applicable | No extensible trait implementation set is authored. |
| C-STRUCT-PRIVATE | Applies | Representations stay private to the application, including native handle and endpoint fields. |
| C-NEWTYPE-HIDE | Applies | Absolute-tick representation is visible only inside the private binary modules. |
| C-STRUCT-BOUNDS | Not applicable | No generic data structure is defined. |
| C-STABLE | Applies | Authored Rust uses stable APIs; existing narrowly scoped build-std/bootstrap exception remains. Private SDK timer ABI stability is explicitly not promised and requires SDK-version review. |
| C-PERMISSIVE | Exception | Existing upstream SDK/dependency licenses remain; project licensing is undecided and this is local evaluation. Revisit transitive notices before distribution. |

## Toolchain and metadata exceptions

ESP-IDF lacks a prebuilt Rust standard-library target here. The pinned build uses
Rust 1.96.0 with scoped RUSTC_BOOTSTRAP=1 and build-std; this exception applies to
this firmware, with unchanged upstream SDK/Rust wrappers. Revisit when a stable
supported target route becomes available. It is not a general workspace setting.

The application is unpublished and its project license remains unassigned.
Metadata reflects established facts; publication and dependency-license details
need deliberate review before distribution. No authorship or license is invented
for checklist completeness. Ordinary firmware failures return contextual errors;
SDK panic/reset, OOM and unrecoverable cleanup retain their documented limits.

Target builds and host controls are complementary. Neither establishes physical
wakeups, UART delivery, register values, WFI continuity or memory margins without
board evidence from the matching image and configuration.
