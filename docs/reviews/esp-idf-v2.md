# ESP-IDF protocol 2 adapter review

Reviewed 2026-10-22 against [the coding standard](../coding-standard.md), its
pinned sources and [all 54 checklist entries](../rust-api-checklist.md).
Scope: `firmware/esp-idf` protocol 2 runtime clock snapshots, bounded two-task
request/reply measurement, SDK configuration and documentation. This supersedes
the execution-context decisions in the [protocol 1 review](esp-idf.md) for the
current binary; that review remains historical evidence. Build checks are in
[the current build record](../builds/esp-idf-v2.md). Board evidence is separate.

## Execution, ownership and failure paths

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

## Unsafe and clock evidence

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

## Complete checklist applicability

Every active ID appears once below. Prior metadata/licensing and build-std
exceptions remain narrow and unchanged; no new standard exception is adopted.

| IDs | Outcome | Evidence / reason |
| --- | --- | --- |
| C-CASE, C-WORD-ORDER | Applies | Types, fields, constants and related clock/unit names follow conventional Rust casing and consistent order; selected rustfmt passes. |
| C-CONV | Applies | `elapsed_ns` is a timed operation rather than a misleading `as_`/`into_` conversion; clocks use `read`. |
| C-GETTER | Not applicable | No stored-field getter API is authored. |
| C-ITER, C-ITER-TY | Not applicable | No collection/iterator interface is defined; standard arrays, enumerate and channel receive are consumed. |
| C-FEATURE | Not applicable | Binary defines no Cargo features. |
| C-COMMON-TRAITS | Applies | Private pointer-free Sample, Clocks and ClockRegisters derive Copy/Clone/Debug/equality/hash plus Serde. Only timestamps derive ordering; only overwritten Sample storage needs Default. No hardware handle is duplicated. |
| C-CONV-TRAITS | Applies | Checked `u64::try_from` and `u32::try_from` handle signed timer and sequence conversion. No bespoke conversion impl is needed. |
| C-COLLECT | Not applicable | No collection implementation is authored; fixed arrays and standard capacity-one queues supply storage behavior. |
| C-SERDE | Applies | Sample and clock result records derive Serialize/Deserialize. Explicit JSON includes runtime/task metadata; timestamps are transient markers, not interchange records. |
| C-SEND-SYNC | Applies | Only owned u32 messages cross threads through safe std channels. Scalars/records receive automatic correct traits; no unsafe sharing implementation exists. |
| C-GOOD-ERR | Applies | io::Error preserves fallible thread creation and writer errors; channel/timer/clock/reply failures report distinct operation context. All errors implement Error + Send + Sync. |
| C-NUM-FMT | Not applicable | No numeric wrapper exposing primitive bit-oriented operations is authored. ClockRegisters is a named record; its raw u32 fields retain primitive numeric formatting. |
| C-RW-VALUE | Applies | `Clocks::write_json` takes `impl Write` by value and callers pass `&mut output`; standard forwarding preserves caller control over borrowing. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable | No macro interface is authored; existing formatting/derive macros are consumed. |
| C-CRATE-DOC | Applies | Binary crate-level doc and README explain target, wiring, build, measurement and failure contract. |
| C-EXAMPLE | Not applicable | Binary exports no public Rust API. README supplies actual target build/run prerequisites. |
| C-QUESTION-MARK | Applies | Fallible operations use `?`; no unwrap/expect appears in authored code/examples. |
| C-FAILURE | Applies | Fallible private operations document errors; inline unsafe invariants and README describe panic, stalls, channel closure, reset and incomplete-run handling. |
| C-LINK | Applies | README/build/review link related records and primary runtime/clock/channel sources. |
| C-METADATA | Exception | Private package remains publish=false with real description/keywords/categories. No author, repository URL or project license is invented. Revisit before distribution/ownership assignment. |
| C-RELNOTES | Not applicable | No release/publication is performed. Protocol migration is documented in README/build evidence. |
| C-HIDDEN | Applies | All implementation types/functions are private; no hidden exported API exists. |
| C-SMART-PTR | Not applicable | No smart pointer implementation is authored. Standard thread/channel ownership is consumed. |
| C-CONV-SPECIFIC | Not applicable | No authored conversion interface between authored types exists. |
| C-METHOD | Applies | `Clocks::read`, `write_json` and timestamp methods belong to their corresponding value/operation. |
| C-NO-OUT | Applies | Timer/clock APIs return values/Results. Standard channel transfer and writer borrowing avoid authored out-parameter APIs. |
| C-OVERLOAD, C-DEREF | Not applicable | No operator or Deref implementation is authored. |
| C-CTOR | Applies | Timestamp uses `now`, clocks use `read`, and only bounded overwritten Sample storage needs Default. Thread uses standard Builder. |
| C-INTERMEDIATE | Applies | Raw clocks and every sample survive to output; timer overhead is separate and never silently subtracted. |
| C-CALLER-CONTROL | Applies | Fixed arrays bound sample memory; writer can be borrowed; channels transfer owned scalar values. No unnecessary reusable ownership contract is imposed. |
| C-GENERIC | Applies | Writer uses standard Write; fixed timer/register addresses/tasks need no extra generic framework. |
| C-OBJECT | Not applicable | No authored trait or dynamic-dispatch interface exists. |
| C-NEWTYPE, C-CUSTOM-TYPE | Applies | Absolute microsecond timestamps use a private newtype; sample elapsed_ns and clock *_mhz/stack *_bytes distinguish units. Named clock records avoid ambiguous positional arguments. |
| C-BITFLAG | Not applicable | Register bitfields are decoded privately from primitive u32; no combinable flag API exists. |
| C-BUILDER | Not applicable | No builder API is authored; std::thread::Builder is used for name/stack settings. |
| C-VALIDATE | Applies | Clock sources/dividers/frequencies/raw stability, timer conversions and each reply are checked. Fixed arrays bound counts; host validates observed protocol/counts/checksums. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Applies | No authored Drop implementation exists. Explicit endpoint closure, fallible/blocking worker join and output flush occur outside measured intervals; stalled-task limitation is documented. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies | Value types derive meaningful nonempty Debug; formatting does not touch hardware, wait or expose credentials. |
| C-SEALED | Not applicable | No trait extension or closed implementation set exists. |
| C-STRUCT-PRIVATE | Applies | All representation fields are private to the binary. |
| C-NEWTYPE-HIDE | Applies | Timestamp's underlying integer is private. |
| C-STRUCT-BOUNDS | Not applicable | No generic data structure is defined. |
| C-STABLE | Applies | Authored Rust uses stable language/library APIs. Scoped build-std bootstrap exception retains the protocol 1 rationale; no stable ESP-IDF sysroot guarantee is claimed. |
| C-PERMISSIVE | Exception | SDK/dependencies retain upstream notices; repository project license is undecided. This is local evaluation, not distribution. Revisit project licensing and transitive notices before distribution. |

Checks and artifact hashes are in [the build record](../builds/esp-idf-v2.md).
No host test can establish actual task wakeups, timer resolution, stack margin,
MMIO values, UART routing or long-running stability. Those require board evidence.
No test was added merely to repeat this binary's implementation.
