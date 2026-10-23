# ESP-IDF adapter review

Reviewed 2026-10-22 against [the coding standard](../coding-standard.md), its
pinned sources, and [the 54-entry inventory](../rust-api-checklist.md).
Scope: `firmware/esp-idf` binary, build script, manifest, configuration and build
documentation. The shared workload has its own review. This record covers
source review and the checks in [the build record](../builds/esp-idf.md), not a
hardware pass.

## Context and safety

Target: `riscv32imac-esp-espidf`, ESP32-C6 application processor, Rust 2024 with
formatting edition 2024. Selected compiler is Rust 1.96.0, scoped `build-std`
bootstrap because no ESP-IDF Rust sysroot is distributed. The binary uses
`std` and IDF's heap outside measured intervals. One FreeRTOS main task owns both
sample arrays and stdout. No application-created task, interrupt handler,
callback, DMA transfer, shared mutable static, atomics, async/cancellation, or
resource handle is introduced. FreeRTOS's own timer/idle tasks and interrupts
continue to run; the scheduler is not suspended.

The sole authored unsafe operation calls generated `esp_timer_get_time` bindings
inside `TimestampMicros::now`. The C function has no parameters, ownership,
pointers, allocation contract, or teardown obligation; IDF initializes it before
the main task. It returns a signed 64-bit microsecond timestamp. The call site's
safety comment records task context and ABI. No handwritten FFI declarations,
representation casts, or unsafe trait impls exist. Bindgen input and generation
versions are captured by `esp-idf-sys` and the dependency lockfile/build record.
The [upstream timer contract](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/esp_timer.html#obtaining-current-time)
permits the call without a separate application timer object.

Elapsed intervals use checked subtraction, signed-to-unsigned conversion and
checked multiplication. The timestamp newtype prevents passing an elapsed
nanosecond value as an absolute microsecond timestamp. Failure before completion
leaves an incomplete run; the collector validates the actual received bytes even
when final flush fails after completion was queued. `Sample` is private working storage whose
numeric wire fields are explicitly named. Sample count casts are safe because
the shared constant is 64, representable on both the 32-bit target and host.

The timed code has no format, I/O, lock acquisition, heap growth, scheduler yield,
or `await`. Stdout's lock is acquired after timing and released after output;
there are no competing application writers. All protocol writes follow both
sample buffers, preventing queued UART traffic from overlapping timing. IDF console I/O may block outside
timing. No authored `Drop` implementation can fail or block. Panics abort, so
Rust cannot unwind across IDF's C entry point. Reset loses partial runs and must
remain visible in host evidence. UART failure diagnostics are best effort.

## Complete checklist applicability

Each ID below occurs once. “Not applicable” gives the absent construct; it does
not exempt private code from applicable safety, naming, or correctness rules.

| IDs | Outcome | Evidence / reason |
| --- | --- | --- |
| C-CASE, C-WORD-ORDER | Applies | Rust identifiers use UpperCamelCase types, snake_case functions/fields and SCREAMING_SNAKE_CASE imported constants; related elapsed fields retain their unit suffix. Target formatting/checks are recorded below. |
| C-CONV | Applies | `elapsed_ns` names a measurement operation, not an `as_`/`into_` conversion; no misleading conversion method is introduced. |
| C-GETTER | Not applicable | No stored-field getter API is introduced. |
| C-ITER, C-ITER-TY | Not applicable | No collection or iterator API is defined; loops use standard array iteration. |
| C-FEATURE | Not applicable | This binary defines no Cargo features. |
| C-COMMON-TRAITS | Applies | Both private value types derive Copy, Clone, Debug, equality and hashing. Timestamp has meaningful ordering; Sample has Default for bounded buffer initialization. Sample ordering, display, and timestamp default have no useful contract and are omitted. Neither type owns hardware. |
| C-CONV-TRAITS | Applies | Checked primitive `u64::try_from` is used; no bespoke conversion trait impl is needed for the closed binary. |
| C-COLLECT | Not applicable | No collection type is authored; fixed arrays retain their standard traits. |
| C-SERDE | Applies | Sample data records derive Serialize and Deserialize using Serde without its std/default features. Explicit JSON protocol emission includes metadata outside the private record; timestamp is a runtime measurement marker, not an interchange record. |
| C-SEND-SYNC | Applies | The two pointer-free value types receive correct automatic Send/Sync; no unsafe impl or sharing occurs. Stdout's standard lock stays task-local. |
| C-GOOD-ERR | Applies | Private fallible functions return meaningful `io::Error` values (Error + Send + Sync), preserve underlying UART failures via `?`, and distinguish clock failures in text. No unit error is used. |
| C-NUM-FMT | Not applicable | Timestamp is a numeric duration-related quantity without bitwise operations or binary formatting needs; Sample is a record. |
| C-RW-VALUE | Not applicable | No generic reader/writer function is defined. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable | No macro interface is authored; existing writeln!/eprintln! and derives are consumed. |
| C-CRATE-DOC | Applies | Binary crate-level description and README document target, wiring, build, execution and measurement scope; this is not a reusable library. |
| C-EXAMPLE | Not applicable | No public Rust items are exported. The README gives the realistic hardware build/run contract. |
| C-QUESTION-MARK | Applies | Fallible operations propagate with `?`; no unwrap/expect appears in code or build examples. |
| C-FAILURE | Applies | `run` and `elapsed_ns` document errors; unsafe call invariant is inline. README records panic/reset behavior and incomplete-run rejection. |
| C-LINK | Applies | README and this review link build evidence, shared standard, binding source and actual timer contract. |
| C-METADATA | Exception | Package is private (`publish = false`), includes actual description, keywords and categories. No authors, repository URL, or license is invented; project authorship/license have not been assigned. No distribution is authorized here. Revisit before publication/distribution and when repository ownership is established. |
| C-RELNOTES | Not applicable | No package release or publication occurs; this is the initial evaluation source. |
| C-HIDDEN | Applies | All adapter types/functions are private; there is no undocumented exported implementation surface. |
| C-SMART-PTR | Not applicable | No smart pointer is authored. |
| C-CONV-SPECIFIC | Not applicable | No conversion interface between authored types is introduced. |
| C-METHOD | Applies | `TimestampMicros::now` creates timestamps and `elapsed_ns` is a method on its starting timestamp. |
| C-NO-OUT | Applies | Timer operations return values and Result; no out-parameter API is authored. |
| C-OVERLOAD, C-DEREF | Not applicable | No operator or Deref implementation is authored. |
| C-CTOR | Applies | Timestamp uses the conventional clock constructor `now`, no empty constructor would convey a valid reading. Sample uses derive Default solely for overwritten buffer storage, not a public construction API. |
| C-INTERMEDIATE | Applies | Timestamp readings and each raw sample remain available; overhead is emitted separately rather than silently subtracted. Shared workload is separate from OS transport. |
| C-CALLER-CONTROL | Applies | Fixed local arrays avoid heap ownership transfers; the workload receives values and returns checksum. No reusable caller-facing buffer ownership contract is needed. |
| C-GENERIC | Applies | Fixed hardware timer, workload and UART transport need no generic abstraction; standard array/Write APIs preserve useful reuse without firmware code expansion. |
| C-OBJECT | Not applicable | No trait or dynamic-dispatch interface is authored. |
| C-NEWTYPE, C-CUSTOM-TYPE | Applies | Absolute microsecond timestamps have a dedicated private type; outputs explicitly name elapsed_ns. No ambiguous bool/Option configuration parameters are used. |
| C-BITFLAG, C-BUILDER | Not applicable | No flags type or complex construction API exists. |
| C-VALIDATE | Applies | Checked interval conversion rejects backwards/overflowed readings, array iteration enforces sample capacity, and the host validates protocol/checksum/completeness from bytes actually received (including final flush failures). Firmware has no runtime user-input boundary. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Applies | No authored destructor or device lifetime exists. Explicit Write::flush reports output errors before successful completion; standard stdout lock release does not flush or wait for hardware teardown. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies | Both private types derive meaningful nonempty Debug representations with scalar fields and no hardware reads or credentials. |
| C-SEALED | Not applicable | No trait extension or deliberately closed implementation set exists. |
| C-STRUCT-PRIVATE | Applies | Sample fields and timestamp representation stay private. |
| C-NEWTYPE-HIDE | Applies | Timestamp's underlying integer is private to the binary. |
| C-STRUCT-BOUNDS | Not applicable | No generic data structure is defined. |
| C-STABLE | Applies | No nightly feature is authored, no public stable library API is claimed; build-std's explicitly scoped tool exception is documented below. |
| C-PERMISSIVE | Exception | Dependencies retain their upstream license terms (notably SDK Apache-2.0 plus component/blob notices). Repository licensing is undecided; this build is local evaluation, not distribution. Revisit license choice and transitive notices before distributing binaries/source. |

## Tooling exception and checks

The official guide's stable formatting conventions and Rust 2024 apply. No
nightly rustfmt option or Google product-specific restriction is adopted.
`RUSTC_BOOTSTRAP=1` is the narrow build-std exception: ESP-IDF distributes no
target std, and the upstream template normally uses nightly. Stable 1.96.0 is
selected to share a compiler with first-round no_std firmware; 1.99.1 fails in
unmodified upstream std. No std/libc patch is authored. Revisit when build-std is
stabilized, upstream espidf std is fixed, or a maintained prebuilt sysroot exists.
The consequence is reliance on a pinned unstable Cargo build facility, not a
claim of stable ESP-IDF sysroot support.

Commands, results, final SDK configuration and unrun hardware checks are recorded
in [the build record](../builds/esp-idf.md). Pure workload vectors and host protocol
tests belong to their own crates. This adapter's hardware timer, UART flush and
reset behavior require actual board execution; host tests cannot establish them.
