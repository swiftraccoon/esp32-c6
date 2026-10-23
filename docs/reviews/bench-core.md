# bench-core initial implementation review

Reviewed 2026-10-22 against the pinned sources in
[the coding standard](../coding-standard.md) and all 54 active entries in
[the API inventory](../rust-api-checklist.md).

Scope: `crates/bench-core/Cargo.toml` and `crates/bench-core/src/lib.rs`.
The crate is internal, version `0.1.0`, and `publish = false`. Its language
edition is 2024. Its dependency-free API needs no language feature newer than
Rust 1.85, the edition's minimum compiler version; that declared minimum is
separate from the toolchain actually used for verification.

## Workload contract

`integer-mix-v1` is one serial chain of 32-bit add, rotate, multiply, and xor
operations. It is a small CPU baseline, not CoreMark or a scheduler, radio,
peripheral, memory-system, or overall OS score. It has no dependencies and no
heap allocation, `alloc`, I/O, unsafe code, FFI, statics, atomics, synchronization,
interrupt manipulation, runtime startup, or cancellation interface.

For seed `s` and nonnegative round count `n`, define `x[0] = s`. For round
`r = 0, ..., n - 1`, with `M = 2^32`:

1. `a = (x[r] + (0x9e3779b9 XOR r)) mod M`.
2. `b = ((a * 2^7) mod M) + floor(a / 2^25)`.
3. `c = (b * 0x85ebca6b) mod M`.
4. `x[r + 1] = c XOR floor(c / 2^16)`.

The result is `x[n]`; zero rounds returns the seed. All `u32` inputs are
mathematically valid. Running `u32::MAX` rounds is finite but may monopolize a
cooperative execution context for a long time, so adapters must choose their
own workload budget. The synchronous function does not yield or call a
blocking API. Calls are reentrant because all state is local. Wrapping arithmetic
and fixed valid shifts avoid overflow panics in both debug and release builds.
The production crate forbids unsafe code. A firmware adapter selects the panic
handler; this crate installs none and contains no explicit panic path.

The public constants are seed `0x12345678`, rounds `4096`, measured samples `64`,
and expected checksum `0xa3944d55`. Adapters own warmup, timers and units, clock
configuration, result validation, output transport, and sample storage.
`#[inline(never)]` intentionally preserves a call boundary; callers also use
`core::hint::black_box` around runtime inputs and the consumed result. Those
choices discourage cross-boundary constant evaluation; they do not prove that
every compiler produces identical instructions or serve as memory barriers.

## Correctness evidence design

Eleven fixed known-answer vectors were calculated separately with arbitrary-width
integer arithmetic, modulo `2^32`, and explicit shifts for the rotation. The
production implementation uses wrapping `u32` operations and `rotate_left`;
the test does not ask production code to generate its own expected values.
Vectors include zero and one round, zero and maximum seeds, a seed crossing the
signed-word boundary, and 64, 257, and 4096 rounds. The default pair gives
`0xa3944d55`; `(0, 4096)` gives `0xeb6d0751`, and
`(0xffffffff, 4096)` gives `0x2a8afcf1`.

A second test uses the inverse mapping of round zero, including the modular
inverse `0xa5cb9243` of the odd multiplier, to recover six boundary seeds.
It checks the inverse identity independently rather than comparing two copies
of the forward Rust loop. Together these checks can detect a wrong operation,
rotation direction, arithmetic width, constant, iteration count, or initial
state. They establish arithmetic behavior for their exercised cases, not timing
or an on-device pass. No timing thresholds are asserted.

## API applicability

Each active entry has a decision below. The exceptions following the table are
scoped to this initial internal crate. No row is pending.

| Entry | Decision | Evidence or concrete reason |
| --- | --- | --- |
| C-CASE | Applies | `workload`, `rounds`, and locals use snake case; public constants use screaming snake case. |
| C-CONV | Not applicable | No conversion functions or methods. |
| C-GETTER | Not applicable | No accessors or owned record types. |
| C-ITER | Not applicable | No collection or iterator API is exposed; an internal range controls a loop. |
| C-ITER-TY | Not applicable | No authored iterator type. |
| C-FEATURE | Not applicable | No Cargo features. |
| C-WORD-ORDER | Applies | Shared defaults consistently use the `DEFAULT_*` prefix. |
| C-COMMON-TRAITS | Not applicable | No authored public types; the API uses standard integer and string types. |
| C-CONV-TRAITS | Not applicable | No conversions between authored types. |
| C-COLLECT | Not applicable | No collection API. |
| C-SERDE | Not applicable | No authored interchange records or configuration structures; only a pure function and primitive constants. Adapters own serialized result records. |
| C-SEND-SYNC | Applies | Only primitive values and immutable string constants cross the API. All mutable state is local, with no shared memory or hardware handles. |
| C-GOOD-ERR | Not applicable | Every `u32` input has a defined result; no errors or error type. |
| C-NUM-FMT | Not applicable | No authored numeric wrapper type. |
| C-RW-VALUE | Not applicable | No I/O or reader/writer abstraction. |
| C-EVOCATIVE | Not applicable | No authored macros. |
| C-MACRO-ATTR | Not applicable | No authored item macros. |
| C-ANYWHERE | Not applicable | No authored item macros. |
| C-MACRO-VIS | Not applicable | No macro-generated API. |
| C-MACRO-TY | Not applicable | No authored macro inputs. |
| C-CRATE-DOC | Applies | Crate docs describe scope, lack of allocation, measurement boundaries, limitations, and a runnable validation example. |
| C-EXAMPLE | Applies | Crate and function examples validate the default checksum and zero-round contract. Constants link to the documented workload. |
| C-QUESTION-MARK | Not applicable | Examples contain no fallible operations. |
| C-FAILURE | Applies | Function docs state all-input validity, absence of error/panic/unsafe preconditions, synchronous CPU use, and cooperative-task implications. |
| C-LINK | Applies | Rustdoc links workload, defaults, and `core::hint::black_box`; the function links to the crate timing example. |
| C-METADATA | Exception M1 | Accurate description, edition, MSRV, keywords, categories, and internal publication status are recorded. Author, project license, and canonical remote are not invented. |
| C-RELNOTES | Not applicable | Initial unpublished development crate; no versioned release is made. Change the protocol identifier if the recurrence changes. |
| C-HIDDEN | Applies | Tests are private and compiled only for tests; the public surface contains only the workload contract and its defaults. |
| C-SMART-PTR | Not applicable | No smart pointer type. |
| C-CONV-SPECIFIC | Not applicable | No conversion API. |
| C-METHOD | Not applicable | No authored receiver type; this is a standalone mathematical function over primitive inputs. |
| C-NO-OUT | Applies | The final state is returned directly; no output argument. |
| C-OVERLOAD | Not applicable | No operator implementation. |
| C-DEREF | Not applicable | No `Deref` or `DerefMut` implementation. |
| C-CTOR | Not applicable | No constructed public record or resource. |
| C-INTERMEDIATE | Applies | The sole useful outcome is the final state. Intermediate per-round values are not computed as separate query results, and returning them would change the workload's storage/measurement behavior. |
| C-CALLER-CONTROL | Applies | Cheap scalar values pass by value; no implicit borrow/clone, hidden ownership transfer, or internal buffer placement. |
| C-GENERIC | Applies | The protocol specifically requires 32-bit words and round indices. Generalizing numeric width would change behavior and the measured instructions rather than relax an irrelevant input assumption. |
| C-OBJECT | Not applicable | No authored traits or dynamic-dispatch interface. |
| C-NEWTYPE | Exception T1 | Seed and iteration count are distinct concepts exposed as positional `u32` arguments in the agreed minimal arithmetic API. |
| C-CUSTOM-TYPE | Exception T1 | The same primitive parameter distinction is explicitly documented; no bool/option mode interface is introduced. |
| C-BITFLAG | Not applicable | No set of configuration flags. |
| C-BUILDER | Not applicable | Two scalar inputs with no complex construction or optional modes. |
| C-VALIDATE | Applies | Every bit pattern is valid for both parameters; wrapping arithmetic defines the full word domain. Adapters validate their timing budgets outside this algorithm. |
| C-DTOR-FAIL | Not applicable | No authored destructor or owned resource. |
| C-DTOR-BLOCK | Not applicable | No authored destructor or shutdown behavior. |
| C-DEBUG | Not applicable | No authored public type requiring `Debug`. |
| C-DEBUG-NONEMPTY | Not applicable | No authored debug representation. |
| C-SEALED | Not applicable | No authored trait. |
| C-STRUCT-PRIVATE | Not applicable | No public struct or representation. |
| C-NEWTYPE-HIDE | Not applicable | No newtype exposing an implementation-dependent representation. |
| C-STRUCT-BOUNDS | Not applicable | No generic structure. |
| C-STABLE | Not applicable | An unpublished pre-1.0 crate makes no stable-crate commitment and has no external dependencies or exposed dependency types. |
| C-PERMISSIVE | Exception L1 | No project license has been selected; publication is disabled and there are no external crate dependencies. |

## Narrow exceptions

M1, `C-METADATA` in the pinned documentation chapter: the repository has not
established an author declaration, canonical public source URL, or project
license. Omit those fields instead of fabricating metadata. Populating them
from assumptions and adopting a license without a project decision were
rejected. This affects this internal crate's manifest only. Consequence:
publication-quality metadata is incomplete. Revisit before publication or
distribution, or when the repository supplies authoritative metadata.

T1, `C-NEWTYPE` and `C-CUSTOM-TYPE` in the pinned type-safety chapter: retain the
agreed `workload(seed: u32, rounds: u32) -> u32` contract for the first cross-runtime
workload. Seed/round newtypes and a configuration structure were considered;
they would distinguish two operands statically but add API constructs to this
single arithmetic operation. Consequence: callers can transpose parameters
without a compiler error. Named defaults, explicit parameter docs, separate
adapter configuration validation, and checksum validation reduce that risk.
This exception covers only these two parameters, not timer units, priorities,
result statuses, or future runtime configuration. Revisit if the API gains
options or the adapters confuse these parameters.

L1, `C-PERMISSIVE` in the pinned necessities chapter: the project has not chosen
a license, and this implementation does not make that decision for the user.
The crate is internal and `publish = false`; it has no external dependencies to
introduce a dependency-license constraint. Silently assigning MIT/Apache or
pretending unpublished code has no licensing implications were rejected.
Consequence: no project redistribution permission is asserted. Revisit before
distribution or publication, when project licensing must be chosen deliberately.

## Verification

Initial compiler selection was recorded before building: Rust 1.99.1
(`58cf8f9c35b355199727627d07ae34a4e2846d67`, LLVM 23.1.1), host
`aarch64-apple-darwin`, target `riscv32imac-unknown-none-elf`, language and style
edition 2024. The common comparison subsequently selected Rust 1.96.0 for SDK
compatibility, as recorded in [toolchains.md](../builds/toolchains.md). The
unchanged workload was retested with that selected compiler. The minimum
declared compiler, 1.85, has not been tested.

| Command | Actual result |
| --- | --- |
| `rustfmt +1.99.1 --edition 2024 --style-edition 2024 --check crates/bench-core/src/lib.rs` | Passed after formatting the initial multiline assertion. |
| `rustc +1.99.1 --edition 2024 --test --crate-name bench_core crates/bench-core/src/lib.rs -o /tmp/esp32-c6-bench-core.VwTwGM/host-tests` | Passed, no diagnostics. |
| `/tmp/esp32-c6-bench-core.VwTwGM/host-tests --nocapture` | Both host unit tests passed; 11 fixed vectors and six inverse-mapping seeds exercised. |
| `rustc +1.99.1 --edition 2024 --crate-type rlib --crate-name bench_core --target riscv32imac-unknown-none-elf crates/bench-core/src/lib.rs --out-dir /tmp/esp32-c6-bench-core.VwTwGM` | Passed, no diagnostics; target library compiled. |
| `rustdoc +1.99.1 --edition 2024 --crate-name bench_core crates/bench-core/src/lib.rs -D rustdoc::broken_intra_doc_links -o /tmp/esp32-c6-bench-core.VwTwGM/docs` | Passed; no broken intra-doc links. |
| `rustup run 1.99.1 clippy-driver --edition 2024 --crate-type rlib --crate-name bench_core crates/bench-core/src/lib.rs -o /tmp/esp32-c6-bench-core.VwTwGM/libbench_core_host.rlib -W clippy::all -D warnings` | Passed; no diagnostics. Warning denial was scoped to this dependency-free crate and the selected toolchain. |
| `rustdoc +1.99.1 --edition 2024 --test --crate-name bench_core crates/bench-core/src/lib.rs --extern bench_core=/tmp/esp32-c6-bench-core.VwTwGM/libbench_core_host.rlib` | Both doctests passed on the host. |
| `cargo +1.96.0 test -p bench-core --locked` | Both unit tests and both doctests passed on the selected common compiler. |
| `cargo +1.96.0 check -p bench-core --locked --target riscv32imac-unknown-none-elf` | Passed on the selected common compiler. |

Initial `cargo test -p bench-core --locked` attempts stopped before compilation
while the sibling workspace member was being created (first a missing manifest,
then no target). Direct compiler, formatter, Clippy, and rustdoc checks therefore
verified this dependency-free crate without requiring the sibling application.
After the workspace was complete and the comparison compiler selected, the
Cargo tests/target check above passed. These compile/test results are not
on-device measurements. Hardware validation belongs to the firmware adapter.
