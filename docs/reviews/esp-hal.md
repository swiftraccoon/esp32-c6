# Rust review: ESP32-C6 HAL adapters

Reviewed 2026-10-22 against the [repository standard](../coding-standard.md)
and [54-entry pinned checklist](../rust-api-checklist.md). Scope:
[`firmware/esp-hal`](../../firmware/esp-hal/), including its build script,
two application entry points, and shared private measurement module. The
separate `bench-core` review governs the workload implementation. Compiler,
target, command evidence, and upstream versions are in
[the build record](../builds/esp-hal.md).

## Execution and safety assessment

All authored implementation is Rust. Firmware uses `core`, fixed arrays, safe
HAL ownership, one main execution context per application, and no allocator.
No authored `unsafe`, raw register access, FFI, atomics, interrupt handlers,
shared mutable state, locks, or destructors are introduced. Upstream macros
generate startup, application descriptor placement, and executor lifetime
plumbing. In particular, the published async main macro extends its permanently
running stack executor's lifetime inside its nonreturning entry point; this
is upstream code, and its source/version is recorded rather than duplicated.

The RTOS uses TIMG0 timer0 and FROM_CPU_INTR0 on the ESP32-C6 high-performance
core. The LP core and radio tasks are not started. The workload is synchronous
and does not yield; its samples therefore cannot establish async fairness.
Runtime interrupts remain enabled and may perturb elapsed time. The finite
startup await occurs before samples and UART output; no critical section or
borrowed transfer is held across an await. UART writing/flush can block after
all samples have been buffered, and errors stop the run. Application panic is
abort/spin with no intentional reboot and no success record. The collector must
reject incomplete runs. Hardware behavior needs separate on-board evidence.

## Checklist decisions

Each of the 54 active identifiers appears once below. “Applies” means reviewed
against the implementation and indicated evidence; it does not certify upstream
code. “Not applicable” states the absent construct. The serialization exception
is intentionally narrow, and the authored project license remains pending.

| Identifier | Decision | Evidence or reason |
| --- | --- | --- |
| C-CASE | Applies | Snake-case functions/modules/fields, upper-case constants, `Sample` type; Rust 2024 formatter. |
| C-CONV | Not applicable | No authored conversion methods. |
| C-GETTER | Not applicable | No authored getters. |
| C-ITER | Not applicable | Iterates standard arrays/slices; defines no collection iteration API. |
| C-ITER-TY | Not applicable | No named authored iterator type. |
| C-FEATURE | Applies | Additive `embassy` feature enables optional runtime/time/executor dependencies; default is empty. |
| C-WORD-ORDER | Applies | `timer_overhead` naming is consistent for arrays, count field, and reporting. |
| C-COMMON-TRAITS | Applies | Private plain-data `Sample` derives `Copy`, `Clone`, `Debug`, `Eq`, `PartialEq`; no ordering/display/default contract is needed for a partially filled internal sample slot. Peripheral types remain owned upstream handles. |
| C-CONV-TRAITS | Not applicable | No authored type conversions or conversion implementations. |
| C-COLLECT | Not applicable | Standard fixed arrays/slices; no authored collection type or insertion/overflow API. |
| C-SERDE | Exception | Private `Sample` buffers hold HAL `Duration`; no public reusable record is exported. Fixed JSON formatting supplies a single output protocol; firmware has no deserialize path. See narrow exception below. |
| C-SEND-SYNC | Applies | Authored `Sample` is plain duration/integer data, with inferred auto traits; no unsafe sharing implementation or manual peripheral sharing. |
| C-GOOD-ERR | Applies | No custom error type; formatting functions return `core::fmt::Result`, propagate with `?`, and main handles startup/UART/flush failures by stopping without a success record. Generic transport details remain in the preserved serial capture and host diagnostics. |
| C-NUM-FMT | Not applicable | No numeric wrapper type; HAL `Duration` owns the unit semantics until explicit protocol formatting. |
| C-RW-VALUE | Applies | Generic `core::fmt::Write` is taken by value by `run`/`write_samples`; its standard mutable-reference implementation permits the caller to retain UART ownership. |
| C-EVOCATIVE | Not applicable | No authored macro. |
| C-MACRO-ATTR | Not applicable | No authored macro. |
| C-ANYWHERE | Not applicable | No authored macro. |
| C-MACRO-VIS | Not applicable | No authored macro. |
| C-MACRO-TY | Not applicable | No authored macro. |
| C-CRATE-DOC | Applies | Binary crate docs identify target/context; firmware README documents workload, configuration, transport, failure behavior, commands, and limits. No library entry point is exported. |
| C-EXAMPLE | Applies | README provides target/wiring/build/protocol example for the private entry points; illustrative numbers are labeled and builds are distinguished from board execution. |
| C-QUESTION-MARK | Applies | Fallible formatting uses `?`; entry-point examples are the actual source and handle errors explicitly. No authored `unwrap` or `try!`. |
| C-FAILURE | Applies | `run` documents formatting errors; README and panic handlers state halt/timeout behavior and no implicit success; no authored unsafe precondition. |
| C-LINK | Applies | Firmware README links shared workload, measurement module, upstream pinned sources, review, and build evidence. |
| C-METADATA | Applies | Manifest has accurate description, target keywords/categories, explicit Rust edition/floor, `publish=false`, and no invented author/site/license. Toolchain is separately pinned. |
| C-RELNOTES | Not applicable | Private initial application, no published/versioned release process established. |
| C-HIDDEN | Applies | No exported reusable library surface or undocumented public API; shared measurement function is parent-module-only. Upstream macro-generated internals are upstream responsibility. |
| C-SMART-PTR | Not applicable | No authored smart pointer. |
| C-CONV-SPECIFIC | Not applicable | No authored conversion interface. |
| C-METHOD | Applies | `Sample::EMPTY` belongs to its record; measurement/reporting combine timer, workload, and writer rather than having one natural authored receiver. |
| C-NO-OUT | Applies | Functions return `Result`; mutable writer borrowing intentionally modifies the caller-owned device under the guideline's caller-buffer exception. |
| C-OVERLOAD | Not applicable | No authored operator implementation. |
| C-DEREF | Not applicable | No authored `Deref`/`DerefMut` implementation. |
| C-CTOR | Not applicable | No constructor method; private record values/initialization constant have no public construction contract. Upstream HAL constructors are used directly. |
| C-INTERMEDIATE | Applies | Timing and reporting are separate; fixed raw samples persist unchanged until serialization, and no aggregation replaces the observations. |
| C-CALLER-CONTROL | Applies | Caller retains UART ownership through mutable-reference `Write`; serialization receives a borrowed slice, with no clone/heap requirement. |
| C-GENERIC | Applies | Small formatting boundary uses `impl Write` rather than coupling measurement to a particular UART representation; workload is shared directly, without an abstract OS framework. |
| C-OBJECT | Not applicable | No authored trait intended for dynamic dispatch. |
| C-NEWTYPE | Applies | Timer instants and elapsed durations retain distinct HAL types; only final JSON formatting converts to explicitly named nanosecond integers. |
| C-CUSTOM-TYPE | Applies | No ambiguous boolean/optional configuration API; UART/clock/timer modes use upstream types. Internal platform/benchmark strings are fixed literals at audited call sites, never external input. |
| C-BITFLAG | Not applicable | No authored combinable flag API. |
| C-BUILDER | Not applicable | No complex authored value construction; upstream HAL configuration builders are used. |
| C-VALIDATE | Applies | Fixed parameters match shared constants; arrays have exact sample count; host validates all checksums/counts. Timer use follows HAL initialization; multiplication bound is documented. No untrusted-input parser exists here. |
| C-DTOR-FAIL | Applies | UART flush is explicit and checked before stopping/pending. No authored destructor or failure hidden in cleanup. |
| C-DTOR-BLOCK | Applies | Blocking UART writes/flush occur explicitly after timing; no authored destructor. |
| C-DEBUG | Applies | Private `Sample` has derived `Debug`; no peripheral inspection side effects or secret fields. |
| C-DEBUG-NONEMPTY | Applies | Derived `Sample` debug includes elapsed duration and checksum. |
| C-SEALED | Not applicable | No authored extension trait or closed implementor set. |
| C-STRUCT-PRIVATE | Applies | `Sample` and its fields are private to measurement module; no public representation commitment. |
| C-NEWTYPE-HIDE | Not applicable | No authored wrapper exposing dependency representation. |
| C-STRUCT-BOUNDS | Not applicable | No authored generic struct. |
| C-STABLE | Applies | Stable Rust 1.96.0; no nightly features. HAL `unstable` is an upstream API feature, explicitly pinned to 1.2.0, not a nightly compiler feature or promised stable adapter API. |
| C-PERMISSIVE | Pending | Direct external runtime/HAL dependencies are MIT OR Apache-2.0; authored project license has not been selected by the user. `publish=false`; resolve licensing before distribution. |

## Narrow serialization exception

Source: pinned [C-SERDE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-serde).
Scope: private firmware-only `Sample` in `src/measurements.rs`. It is bounded
scratch storage whose upstream HAL duration has no serialization contract.
Adding optional derives plus an authored duration adapter would create an
unused reusable format beside the explicit measurement protocol. The selected
implementation formats fixed numeric fields directly without allocation. The
consequence is that `Sample` itself cannot be serialized/deserialized through
Serde. Revisit if records become public/reusable, input configuration is added,
or a shared wire-schema crate replaces the current output-only boundary. This
exception does not exempt host records or future public result structures.

## Verification scope

The [build record](../builds/esp-hal.md) gives exact formatting, target build,
Clippy, and documentation commands/results. Host known-answer tests are owned
by `bench-core`. No redundant hardware-free unit test is added for HAL startup
or a fixed serialization implementation. Actual timer resolution, serial pins,
checksum preservation on target, successful completion, resets/watchdogs,
interrupt behavior, memory headroom, and transport failure behavior require
on-board checks by the hardware owner. A successful compile is not one of
those checks.
