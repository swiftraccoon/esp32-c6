# benchctl initial implementation review

Reviewed 2026-10-22 against the
[coding standard](../coding-standard.md) and all 54 active entries in the
[API inventory](../rust-api-checklist.md). Scope:
`tools/benchctl/src/main.rs`, `tools/benchctl/src/protocol.rs`, their manifest and
the root lockfile. This is an internal host binary, not a public Rust library.
It uses Rust 1.96.0, edition/style edition 2024, target
`aarch64-apple-darwin`; see [toolchain selection](../builds/toolchains.md).

## Correctness review and changes

The collector accepts only protocol 1, the requested platform, 160 MHz declared
CPU configuration, the exact shared seed/round/workload revision, eight warmup
iterations, and a 1,000 ns declared timer resolution. Both known benchmarks
must contain exactly 64 ordered, unique indices before completion. Compute
checksums must match [bench-core](../../crates/bench-core/), compute duration must
be positive, and every elapsed value must be a multiple of 1,000 ns. A zero
timer-observer value is preserved as valid quantization evidence. A missing or
invalid result never becomes a zero-valued substitute score.

Independent review found and corrected these issues before hardware capture:

- `object` 0.40 uses typed ELF flags and an additional section-pattern field;
  inspect now handles that API and checks section-total addition for overflow.
- Validation errors now permanently invalidate a `Boot`. Ignoring one error
  cannot let a subsequent record sequence rehabilitate a failed run.
- A zero-duration compute sample is rejected, while zero timer overhead remains
  supported. Timer units and checksums are validated before samples are stored.
- Live and archived input now share a 4,096-byte maximum line payload, including
  an optional carriage return but excluding the newline. Unterminated overlong
  archived lines no longer receive an extra byte of tolerance.
- A live read processes the remainder of the already-preserved chunk after a
  completion marker. A trailing rejected record in those same raw bytes cannot
  coexist with a passing capture report.
- Build provenance must be a JSON object. The active `rustc -Vv` observation is
  named `host_rustc_at_capture`, and is explicitly distinct from the compiler
  that built the collector. The collector's version and executable SHA-256 are
  recorded instead of inventing embedded build-compiler metadata.

Capture checks supplied artifacts before opening/resetting the device and
refuses to overwrite an existing result directory. It preserves the supplied
ELF, its SHA-256, raw serial bytes, parsed JSON records, supplied build provenance,
and failure reason. Raw and parsed files are explicitly flushed. No final pass
is emitted unless protocol validation completes. Failures leave evidence and a
failed report when output I/O permits; a disk/write failure can itself prevent
finalization, leaving `running` or a partial report, which is not a pass.

`--boot-id` and `--run-order` are operator-supplied labels. The collector requests
reset by deasserting DTR, asserting RTS for 100 ms, clearing buffered input, and
releasing RTS. It cannot independently verify the physical EN waveform or
power-cycle the board. A returned hello establishes the firmware's declarations,
not an independently measured CPU frequency, timer calibration, or proof that
the supplied ELF is the image actually running. Preserve the flash command and
hardware evidence separately. Live capture ends at the first complete boot;
it is not a subsequent stability or reset-survival test. Archived validation
checks every protocol record through EOF and rejects a later reboot/duplicate
record. Unknown extra JSON fields are accepted for additive metadata; unknown
events, missing required fields, and unsupported protocol revisions fail.

## Statistics and footprint limits

Summaries sort all samples, retain outliers, and use nearest-rank percentiles:
rank `ceil(p * n / 100)`, with ranks starting at one. With 64 samples, p50 is
sorted sample 32, p95 sample 61, and p99 sample 64, so p99 equals the observed
maximum. This is per-boot descriptive evidence, not a worst-case bound or 64
independent trials. Mean is a floating-point arithmetic mean and can round large
integer inputs. All times are reported in nanoseconds, with 1,000 ns timer
resolution. Compute includes observer overhead; no subtraction occurs.

ELF inspection enumerates `SHF_ALLOC` sections and their virtual addresses,
classifications, and byte sizes. It does not convert ELF filesystem length into
flash usage or static sections into peak RAM usage. In the inspected bare-metal
artifact, `UninitializedData` includes a 442,844-byte reserved `.stack` and a
53,396-byte flash-address `.text_gap`; summing that category is not measured
stack use or live RAM. File/image padding, bootloader/partitions, stack high-water,
heap peaks and complete flash-image footprint need separate evidence.

## Execution and safety

The collector runs on one host thread. It uses `std` and heap allocation for
bounded protocol lines, at most two 64-sample vectors, JSON metadata, whole ELF
and executable file reads, and statistics. File/serial I/O, process spawning and
the reset sleep can block. These operations happen on the host, outside every
device timing interval. Host file/metadata sizes remain operator-controlled;
this tool is not a general sandbox for hostile ELF or provenance files.

No authored unsafe code, FFI, interrupt, async task, callback, cancellation
contract, custom synchronization or destructor is introduced. The binary forbids
unsafe code; serial-port OS integration resides in the upstream dependency.
The host may be interrupted or killed, leaving partial evidence without a final
pass. Release panics abort per workspace settings; ordinary parse, validation,
I/O and timeout failures use `Result` with context. Private statistics indexing
requires a nonempty validated sample set; only completed `Boot` state calls it
outside the dedicated nonempty statistics tests.

## API applicability

Grouped rows enumerate all 54 active identifiers. Every row has a decision;
the exceptions below are scoped and no row is pending.

| Entries | Decision | Evidence or concrete reason |
| --- | --- | --- |
| C-CASE, C-WORD-ORDER | Applies | Conventional Rust casing; related timing fields consistently retain `_ns`, clock `_mhz`, and byte-count `_bytes` units. |
| C-CONV, C-CONV-TRAITS, C-CONV-SPECIFIC | Not applicable | No authored conversion methods or conversion-trait implementations. `from_samples` computes a summary rather than converting a wrapper representation. |
| C-GETTER | Applies | `Boot::hello` uses a noun and returns an optional shared borrow without a `get_` prefix. |
| C-ITER, C-ITER-TY | Not applicable | No authored collection iteration API or named iterator type; internal standard-library iterators do not introduce one. |
| C-FEATURE | Not applicable | No crate features. Dependency features are upstream interfaces. |
| C-COMMON-TRAITS | Applies | Internal records/configuration/state derive useful `Debug`; `Boot::default` represents a valid empty collector. Protocol records derive serialization traits. No resource handle is duplicated through invented `Copy`/`Clone` implementations. No public reusable value type needs ordering/hash/equality traits. |
| C-COLLECT | Not applicable | Standard maps/vectors are used internally; no custom collection type is introduced. |
| C-SERDE | Applies | Wire records implement `Serialize`/`Deserialize`; summaries implement output serialization. `Boot` is live validation state, not a serialized interchange record. |
| C-SEND-SYNC | Applies | Internal values have no manually implemented or suppressed sharing traits. Execution is single-threaded with exclusive mutable validation/writer borrows; no cross-context sharing is assumed. |
| C-GOOD-ERR | Applies | Ordinary failures use contextual `anyhow::Result`; error chains reach CLI diagnostics and failed reports. There is no public library error type to implement `Error` for. |
| C-NUM-FMT | Not applicable | No authored numeric wrapper type. JSON numeric fields remain numbers, and SHA-256 is explicitly hexadecimal. |
| C-RW-VALUE | Applies | `validate` accepts `impl BufRead`, `write_json` accepts `impl Write`, and capture streams accept mutable `impl Write`; callers may supply owned or borrowed I/O. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable | No authored macro interface. Upstream derives, `json!`, and `ensure!` are used through their existing interfaces. |
| C-CRATE-DOC | Applies at application scope | Module docs and CLI help identify the binary's operations and execution context; this review records operational contracts and limitations. |
| C-EXAMPLE | Not applicable | No exported public Rust API. CLI invocation examples below cover the application interface; host tests exercise complete realistic transcripts. |
| C-QUESTION-MARK | Not applicable | No fallible Rust documentation examples. Production fallible calls use `?`; assertions/unwraps are confined to tests. |
| C-FAILURE | Applies | Capture/validation/inspection docs describe failures and blocking effects. `Boot` documents irreversible rejection and incomplete-summary errors. No public unsafe contract exists. |
| C-LINK | Applies | This review links shared workload, toolchain, API inventory and governing standard; CLI field docs identify supplied provenance. |
| C-METADATA | Exception M1 | Accurate package identity, description, compiler requirement and unpublished status are set; publication metadata and project licensing are incomplete. |
| C-RELNOTES | Not applicable | Initial unpublished pre-1.0 application; no versioned release is made. |
| C-HIDDEN | Applies | Rust interfaces are private or `pub(crate)`, and implementation state is not presented as a public reusable library surface. |
| C-SMART-PTR, C-DEREF | Not applicable | No authored smart pointer or dereference implementation. |
| C-METHOD | Applies | Record acceptance, hello access and summaries are methods on `Boot`; summary construction is associated with `Summary`. |
| C-NO-OUT | Applies | Validation/inspection/calculation return their values. Mutable stream/state parameters preserve caller-owned evidence during capture, rather than needless scalar output parameters. |
| C-OVERLOAD | Not applicable | No authored operator implementation. |
| C-CTOR | Applies | `Summary::from_samples` is an associated factory for a complete nonempty sample slice; default `Boot` creates a valid initial state. |
| C-INTERMEDIATE | Applies | Raw bytes, parsed records and partial hello evidence survive failures; invalid/incomplete state deliberately yields no derived numeric summary. ELF reports preserve per-section data alongside totals. |
| C-CALLER-CONTROL | Applies | Configuration borrows paths/strings; I/O callers own their streams; no hidden clone of caller buffers or resource handles occurs. |
| C-GENERIC | Applies | Read/write abstractions require their actual `BufRead`/`Write` operations. Protocol uses fixed-width integers because the wire contract specifies their range. |
| C-OBJECT | Not applicable | No authored trait or dynamic-dispatch interface. |
| C-NEWTYPE, C-CUSTOM-TYPE | Applies | `Duration`, `Instant` and `SystemTime` distinguish host time concepts; `Action`, `Record` and named `Capture` configuration distinguish operations and inputs. Wire timing has explicit `_ns` fields, one enforced unit/resolution and no host unit conversion. Internal completion booleans are documented state flags, not positional mode arguments. |
| C-BITFLAG | Not applicable | No authored set of combinable flags. Typed upstream ELF flags are used through their provided API. |
| C-BUILDER | Not applicable | Private capture configuration is a complete named-field value constructed from parsed required CLI options, without incremental optional construction. |
| C-VALIDATE | Applies | Bounds, protocol/configuration/checksum consistency, complete counts, ordering, timer quantization, positive compute duration and irreversible rejection are enforced and tested. Input provenance must be an object. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Not applicable to authored destructors | No custom `Drop`. Output streams are explicitly flushed and errors surfaced; ordinary host handle closure remains upstream behavior, outside device measurements. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies | Authored records, enum variants and configuration/state derive useful field-bearing `Debug`; no empty dummy representation or credentials field is introduced. |
| C-SEALED | Not applicable | No authored trait implementation set. |
| C-STRUCT-PRIVATE | Applies | Validation-state and protocol-record fields are private to their modules; callers use methods. CLI configuration is private application state. |
| C-NEWTYPE-HIDE | Not applicable | No authored representation-dependent newtype. |
| C-STRUCT-BOUNDS | Not applicable | No generic data structure; the lifetime parameter on borrowed configuration introduces no trait bound. |
| C-STABLE | Not applicable | Pre-1.0 unpublished binary with no stable public Rust library API commitment. |
| C-PERMISSIVE | Exception L1/L2 | Project license is unchosen; `serialport` 4.10.1 declares MPL-2.0. Other external dependencies in the current host tree report permissive licenses. |

M1, pinned documentation `C-METADATA`: do not fabricate authorship, a canonical
remote, or a project license for this unpublished internal binary. The manifest
also has no publication keywords/categories. Filling unestablished metadata or
choosing licensing merely to satisfy a checklist was rejected. Consequence:
publication metadata is incomplete; revisit before publishing/distributing or
when authoritative repository metadata becomes available.

L1/L2, pinned necessities `C-PERMISSIVE`: no project redistribution license has
been selected, and the unchanged upstream `serialport` 4.10.1 dependency is
MPL-2.0 rather than permissively licensed. Mozilla describes MPL as file-level
copyleft and documents source-availability conditions for distributed executable
works in its [official FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).
The narrow scope is this internal collector's existing serial integration; it
does not assign MPL to this project's new files. Retaining the existing maintained
Rust serial interface avoids adding a bespoke OS FFI layer merely for an internal
trial. A permissively licensed alternative would require capability/platform
review and is not claimed evaluated. Consequence: distribution needs deliberate
project licensing and dependency-notice/source review. Revisit before
distribution or if a strict permissive-only dependency requirement is adopted.

## Verification

| Command | Actual result |
| --- | --- |
| `cargo +1.96.0 test -p benchctl --locked` | 11 tests passed: full-transcript parsing, corruption/bounds, irreversible failures/reboots, missing/duplicate samples, checksums, timer quantization/zero overhead, complete sample sets, and four-/64-sample percentile behavior. |
| `cargo +1.96.0 fmt -p benchctl -- --check` | Passed after formatting. |
| `cargo +1.96.0 clippy -p benchctl --locked --all-targets -- -D warnings` | Passed with no diagnostics. Warning denial scoped to this selected application/toolchain, not all vendor code. |
| `cargo +1.96.0 doc -p benchctl --locked --no-deps` | Passed; binary documentation generated. |
| `cargo +1.96.0 run -p benchctl --locked -- inspect --elf firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal --output /tmp/esp32-c6-bench-core.VwTwGM/bare-metal-sections.json` | Passed against the actual linked RISC-V artifact; section/ELF SHA-256 report generated. This is static inspection, not a hardware pass. |
| `cargo +1.96.0 tree -p benchctl --locked --format '{p} {license}'` | Recorded current-host dependency license declarations; identified MPL-2.0 `serialport`. |

For an existing raw transcript, the user-facing validation command is
`cargo run -p benchctl --locked -- validate --input <serial.raw> --platform <platform>`.
Its entire input must contain one valid boot; failure exits nonzero and emits no
success summary. Hardware capture/reset, baud/transport behavior, physical clock
and timer calibration, power and benchmark performance were not tested by this
review agent. The hardware operator records those separately.
