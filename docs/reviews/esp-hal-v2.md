# Rust review: HAL adapters, protocol 2 and message round trips

Reviewed 2026-10-22 under the [coding standard](../coding-standard.md) and
[54-entry pinned inventory](../rust-api-checklist.md). Scope is the current
[`firmware/esp-hal`](../../firmware/esp-hal/) manifests, private `clocks` and
`measurements` modules, both entry points and README. The
[protocol-1 review](esp-hal.md) remains historical evidence. This review does not
rewrite archived results or imply the initial flash-clock discrepancy was
already resolved on hardware. The [v2 build record](../builds/esp-hal-v2.md)
identifies frozen artifacts and checks; hardware validation is separate.

## Execution, safety and measurement assessment

All authored code remains Rust `no_std`, without `alloc`, FFI, authored unsafe,
assembly or interrupt handlers. Clock telemetry uses the public safe
`PCR::regs()` / `SPI0::regs()` accessors in pinned esp-hal, then the PAC's
read-only volatile register operations. Register specifications show no
read-to-clear behavior. No peripheral handle is stolen/duplicated and no
authored clock write occurs. These are safe upstream interfaces; the underlying
HAL/PAC implementation still contains its own audited obligations rather than
being free of unsafe code.

The decoder explicitly supports the SPLL CPU/MSPI path, valid divider fields,
and force-120 CPU special case. It rejects unsupported/fractional results and
reports supported decoded values as observed. The host separately enforces
the intended 160 MHz CPU / 80 MHz source / 80 MHz flash / divider-1 baseline
from the preserved raw fields. Both snapshots must match exactly. The reads
are sequential and equal endpoints do not prove absence of transient changes;
no dynamic scaling is enabled. This is register configuration evidence, not
an electrical clock measurement. Field sources and formulas are linked from
the [firmware README](../../firmware/esp-hal/README.md).

Bare metal records only compute and timer overhead. Embassy adds an independent
responder task on the same cooperative executor and OS thread as the initiator.
Two static `Channel<CriticalSectionRawMutex, u32, 1>` queues provide bounded,
safe communication. The upstream mutex enters short critical sections within
channel operations; none spans an await. Runtime timer/scheduling interrupts
remain enabled. The LP core, radios, DMA and additional OS threads are unused.
This path measures channel/executor handoff in both directions, not a pure
OS thread-switch cost. FreeRTOS's separate OS threads/channel implementation
must remain visible in comparisons.

Eight compute and eight message warmups finish before the first snapshot and
all measured regions. Samples then run in compute / timer-overhead / message
order. Each request/reply has deterministic sequence/checksum; validation is
immediately after the elapsed read and outside timing, before the next pair.
There is one request in flight. The responder parks on the next receive after
every reply, including the last sample, so task teardown cannot affect only
the final interval. It remains parked after the run.
Fixed sample storage is 2048 bytes bare metal or 18048 bytes Embassy on this
target, excluding channel/runtime/stack overhead. Embassy's cross-await sample
arrays are retained in its statically allocated main-future task pool. No
allocation, formatting or serial output occurs in any timed interval.

The HAL timer has 1 microsecond resolution; integer nanosecond wire units add
no precision and zero observations remain valid. Timer overhead is preserved
and never subtracted. Protocol 2 declares exact per-benchmark counts and
records all raw samples before any UART output. It never substitutes a zero
handoff score for bare metal's unsupported scheduler workload.

No application path cancels the measurement future. Cancellation could leave
one request/reply queued or the responder waiting, yielding an incomplete run;
no recovery or successful completion is synthesized. UART ownership remains
with main, including the mutable borrow held across message awaits. No blocking
lock or interrupt-disabled critical section spans those awaits. Missing replies
suspend indefinitely and require the external collector's bounded timeout.
`RunError` identifies unsupported/changed clocks, spawn, reply and output
failures, with error composition for formatting and concrete reply details.
Main halts on error. Panics abort/spin without unwinding or a success record.
Explicit UART flush is checked after reporting. Post-run spin versus pending
idle behavior is outside the benchmark and cannot establish idle power.

## All 54 checklist decisions

| Identifier | Decision | Evidence or reason |
| --- | --- | --- |
| C-CASE | Applies | Snake-case fields/functions/modules; upper-case constants; `ClockSnapshot`, `RunError`, `Sample`; stable Rust 2024 formatting. |
| C-CONV | Not applicable | No authored conversion-named methods; the standard `From` trait is assessed below. |
| C-GETTER | Not applicable | No property getters; `ClockSnapshot::read` performs hardware I/O rather than exposing a stored property. |
| C-ITER | Not applicable | Standard array/slice iteration; no authored collection iteration interface. |
| C-ITER-TY | Not applicable | No authored named iterator. |
| C-FEATURE | Applies | Additive `embassy` feature selects optional executor/sync/time/runtime dependencies, empty defaults; supported bare-metal comparison builds omit runtime features. |
| C-WORD-ORDER | Applies | `timer_overhead`, `message_roundtrip`, `clocks_after` and clock field families retain consistent ordering across code/wire fields. |
| C-COMMON-TRAITS | Applies | Plain private records/errors derive `Clone`, `Copy`, `Debug`, `Eq`, `PartialEq`; `RunError` also has `Display`/`Error`. No invented order/default/display contract for samples or snapshots; upstream channels/handles are not cloned. |
| C-CONV-TRAITS | Applies | `From<fmt::Error>` preserves the original formatting error for `?` propagation; no ad hoc conversion method. |
| C-COLLECT | Not applicable | Standard fixed arrays/slices and upstream bounded channel; no authored collection or insertion API. Channel backpressure is documented explicitly. |
| C-SERDE | Exception | Private firmware scratch records have an output-only fixed wire format and no public reusable serialization contract; bounded direct formatting avoids an unused secondary representation. Narrow scope and revisit trigger below. |
| C-SEND-SYNC | Applies | Records/errors contain plain data with inferred auto traits; no authored unsafe trait implementation. Shared static channels use the upstream critical-section mutex safe across contexts; UART stays in main. |
| C-GOOD-ERR | Applies | Private `RunError` implements `Display`, `Debug`, `core::error::Error`, formatting source composition and concrete reply details. Clock/spawn failures have distinct variants; entry points halt and host rejects missing completion. |
| C-NUM-FMT | Not applicable | No numeric wrapper; clock snapshots are compound records and raw values are emitted in explicit decimal wire fields. |
| C-RW-VALUE | Applies | `impl core::fmt::Write` parameters are by value; mutable-reference implementations preserve caller UART ownership. |
| C-EVOCATIVE | Not applicable | No authored macro. |
| C-MACRO-ATTR | Not applicable | No authored macro. |
| C-ANYWHERE | Not applicable | No authored macro. |
| C-MACRO-VIS | Not applicable | No authored macro. |
| C-MACRO-TY | Not applicable | No authored macro. |
| C-CRATE-DOC | Applies | Binary docs and README identify target, context, workload/transport/clock/safety/failure behavior; no reusable library crate entry point. |
| C-EXAMPLE | Applies | README gives actual target/wiring/build commands and precise output schema; source is the private application example. Build evidence is distinguished from on-board execution. |
| C-QUESTION-MARK | Applies | Fallible spawn/clock/validation/formatting paths use `?`; no authored `unwrap`, `expect` or `try!`. |
| C-FAILURE | Applies | Entry functions document errors and missing-reply suspension; snapshot read documents unsupported values; README states cancellation, panic/halt, output/timeout behavior and endpoint-read limitations. No authored unsafe precondition. |
| C-LINK | Applies | README connects clock and channel primary sources, pinned macros, protocol, historical records and new build/review evidence. |
| C-METADATA | Applies | Accurate package description/categories/keywords, Rust edition/dependency floor and `publish=false`; no invented author/site/license. Exact stable toolchain and dependency lock are retained. |
| C-RELNOTES | Not applicable | Unpublished evaluation applications; no release or versioned distribution process established. Protocol change is documented explicitly. |
| C-HIDDEN | Applies | Shared helpers/types are private or parent-visible within binaries; no exported reusable API or undocumented library surface. Upstream macro internals remain upstream responsibility. |
| C-SMART-PTR | Not applicable | No authored smart pointer. |
| C-CONV-SPECIFIC | Applies | Formatting conversion is implemented for the specific local `RunError`, keeping the external source error intact. |
| C-METHOD | Applies | Snapshot read/write operations are associated with `ClockSnapshot`; `Sample::EMPTY` belongs to its record; workload orchestration combines clocks/writer/tasks rather than having one natural receiver. |
| C-NO-OUT | Applies | Measurement returns `Result`; baseline collection returns its two arrays; writing mutates an explicit caller-owned writer under the buffer exception. |
| C-OVERLOAD | Not applicable | No authored operator implementation. |
| C-DEREF | Not applicable | No authored dereference implementation. |
| C-CTOR | Not applicable | No public construction API; snapshots arise only from fallible hardware reads, sample buffers from a private initialization constant. |
| C-INTERMEDIATE | Applies | Measurement, validation and reporting remain separate; raw arrays and both snapshots persist unchanged until serialization, without replacing samples with aggregates. |
| C-CALLER-CONTROL | Applies | Borrowed writer and slices preserve caller ownership; no cloning, heap allocation or imposed generic storage framework. |
| C-GENERIC | Applies | Small `impl Write` output boundary supports caller borrowing; concrete fixed arrays/channels keep embedded cost and ownership explicit. |
| C-OBJECT | Not applicable | No authored trait intended for dynamic dispatch; `Error::source` uses the standard trait contract. |
| C-NEWTYPE | Applies | HAL `Instant`/`Duration` distinguish timestamps/durations; final integer conversions use explicit `_ns` fields. Private snapshot construction distinguishes raw register fields and decoded MHz/divider fields at the wire boundary. |
| C-CUSTOM-TYPE | Applies | Distinct `RunError`/`ClockSnapshot`/`Sample` avoid ambiguous status/configuration flags. External HAL types govern clocks/UART; internal benchmark/platform strings are fixed literals. |
| C-BITFLAG | Not applicable | No authored combinable flags API; PAC field reads are upstream typed accessors. |
| C-BUILDER | Not applicable | No complex authored public construction API; upstream configuration builders are used directly. |
| C-VALIDATE | Applies | Source/divider sets and exact integer conversion validated; endpoint snapshots compared; bounded 1000-entry indices fit u32; each reply checked outside timing; counts come from actual slices. Host independently checks clocks/counts/checksums and timer quantization. |
| C-DTOR-FAIL | Applies | Explicit checked UART flush; responder parks after each reply and remains pending after completion; no failure hidden in an authored destructor. |
| C-DTOR-BLOCK | Applies | Blocking writes/flush are explicit and after measurement; no authored destructor blocks. Channel suspension is explicit async behavior. |
| C-DEBUG | Applies | Plain records/errors derive safe `Debug`; inspecting a saved snapshot performs no hardware read and contains no secret. |
| C-DEBUG-NONEMPTY | Applies | Derived output includes record fields and detailed reply/error state. |
| C-SEALED | Not applicable | No authored extension trait or closed implementation set. |
| C-STRUCT-PRIVATE | Applies | Snapshot/sample fields are private and their containing types have binary-internal visibility only; no external layout commitment. |
| C-NEWTYPE-HIDE | Not applicable | No public wrapper exposing an implementation dependency. |
| C-STRUCT-BOUNDS | Not applicable | No authored generic struct. |
| C-STABLE | Applies | Stable Rust 1.96.0, no nightly language features. Pinned HAL `unstable` is an upstream API opt-in, not a compiler feature or adapter stability claim. |
| C-PERMISSIVE | Pending | External runtime/HAL/sync dependencies use MIT OR Apache-2.0. Authored project licensing remains undecided; `publish=false`. Resolve before distribution. |

## Serialization exception and tooling qualification

The pinned [serialization guideline](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-serde)
is narrowly excepted for private firmware `Sample` and `ClockSnapshot` scratch
records. `Sample` contains upstream HAL `Duration`; neither record is public
or deserialized, and both already have a single fixed numeric output schema.
Adding optional derives and duration adapters would add an unused second
representation. Consequence: these scratch types do not implement Serde.
Revisit on a public/reusable record, input parser/configuration, or shared schema
crate. Host interchange records are not covered by this exception.

A feature-scoped `allow(dead_code)` is attached only to the synchronous `run`
entry helper because the Embassy binary intentionally calls `run_embassy`.
The function remains available when features are enabled; the suppression does
not hide unused algorithms, shared state, or dependency warnings. No global
warning denial or suppression was added. Primary source/manifest conventions
and compatible Google safety/initialization guidance were reviewed under the
existing repository precedence; no new language or authority exception arises.

## Verification limits

Both target release builds, formatting, Clippy and documentation passed without
warnings as recorded in [the build record](../builds/esp-hal-v2.md). The pure
workload tests remain in bench-core, and the host owns independent protocol and
raw-register decoder tests. No test was added that merely restates the fixed
serializer. On-board serial completion, live register readback, real executor
handoffs, correctness across resets, watchdog behavior, timing distribution,
memory headroom and cancellation/fault behavior remain hardware concerns;
compilation does not certify those behaviors. This implementation agent did
not access the board or serial ports.
