# benchctl protocol 2 review

Reviewed 2026-10-22 against the [coding standard](../coding-standard.md), all
54 active entries in the [API inventory](../rust-api-checklist.md), and the
[initial benchctl review](benchctl.md). Scope: `tools/benchctl/src/protocol.rs`,
`tools/benchctl/src/main.rs`, and the tool README. Existing manifest, dependency
versions, host target, compiler and project-license exceptions are unchanged.
This is an internal Rust 1.96.0, edition/style edition 2024 host binary.

## Result-validation changes

Existing protocol 1 firmware records retain their shared configuration checks
and report schemas. Numeric completion count metadata remains ignored and omitted
when serializing parsed protocol 1 completion records. All 15 initial raw transcripts were replayed
through both the preceding release executable and the changed executable; their
complete validation JSON outputs were byte-identical. This is archival replay,
not a new physical-device trial or retroactive clock-readback evidence.

Compatibility boundary: the shared record structs reserve new optional fields
including `benchmarks`, `clocks`, and completion counts with typed shapes.
Hypothetical protocol 1 diagnostic metadata using those names with other shapes
(for example `samples: "text"`) can now fail deserialization where the old
collector ignored it. Actual protocol 1 firmware uses numeric count metadata;
none of the supplied archives has this collision. Full version-dispatched
deserialization is deferred unless a concrete compatibility need arises. This
does not change the byte-identical replay result for the 15 existing archives.

Protocol 2 retains the shared core seed, rounds, workload, compute/calibration
count of 64, eight compute warmups, declared 160 MHz CPU and 1,000 ns timer
resolution. Its exact benchmark map has compute/calibration only for
`bare-metal`; `embassy` and `esp-idf-freertos` additionally require 1,000
`message-roundtrip` samples. Unknown platforms, unknown benchmarks, different
counts and unknown hello configuration fail. Scheduler metadata requires a
nonempty transport description, capacity one and eight message warmups; other
diagnostic fields are retained as metadata. Bare-metal omits scheduler metadata.

Each message sample completes one exchange, has the checksum
`index XOR 0xa5a55a5a`, and uses a unique sequential index. Elapsed values remain
quantized to 1,000 ns. Zero calibration and message durations are retained as
coarse-timer evidence; compute must remain positive. Completion requires all
actual sample sets and matching total/per-benchmark count fields. Advertising
1,000 samples does not allow an incomplete 999-sample run to pass.

The host independently decodes initial/final register snapshots, checks the
reported clock numbers against those decodes, and requires exact raw-register
equality across the measured region. The wire shape and examples are in the
[tool README](../../tools/benchctl/README.md). Sources for the register fields
are the locally pinned `esp32c6` 0.24.0 PAC `pcr/sysclk_conf.rs`,
`pcr/cpu_freq_conf.rs`, `pcr/mspi_clk_conf.rs`, and `spi0/clock.rs`, checked
against the firmware adapters' corresponding readback. The formulas are:

| Field | Supported decode |
| --- | --- |
| `SYSCLK_CONF[17:16]` | Must select SPLL (`1`). |
| `SYSCLK_CONF[15:8]` | Must be `2`, giving SPLL 480 / 3 = 160 MHz system root. |
| `CPU_FREQ_CONF[15:8]` | Legal divider fields `0`, `1`, `3`; divide system root by field + 1. Field zero with bit 16 set forces 120 MHz. The resulting CPU must be 160 MHz. |
| `MSPI_CLK_CONF[15:8]` | Legal fields `3..=5`; flash source is SPLL 480 / (field + 1), required to equal 80 MHz. |
| `SPI0_CLOCK[31]`, `[23:16]` | Bit 31 gives divide by one; otherwise divider is field + 1. Required divider one and resulting flash 80 MHz. |

These are decoded configuration observations assuming the chip's documented
480 MHz SPLL source, not externally calibrated frequency measurements. Unsupported
source configurations are rejected rather than inferred. Exact before/after
snapshots cannot exclude a transient intermediate change. Device-register
authenticity and the identity of running firmware remain outside this host
protocol's trust boundary; preserve supplied-image hashes and flash evidence.

## Execution, safety and resource review

No unsafe code, FFI, authored synchronization, interrupt, async, cancellation,
custom destructor, dependency or public Rust API is added. Existing
`#![forbid(unsafe_code)]` remains. The single host thread performs blocking I/O
and heap allocation outside device timing. At most three validated sample sets
contain 1,128 observations in protocol 2; line input retains its 4,096-byte
bound. JSON metadata and input artifact sizes remain operator controlled.

Records must use the hello's protocol throughout. Any validation error
permanently rejects the boot. A later valid record cannot restore it. Missing,
rebooted, corrupt, unsupported, or failed measurements produce no numeric score.
Statistics remain per-boot nearest-rank descriptions with all outliers retained;
no timer subtraction or pooling across independent boots was added. In
particular, a 1,000-observation p99 is a within-boot percentile, not 1,000
independent boots or a worst-case scheduling guarantee.

`run.json` adds `clocks_after` for accepted final readbacks and updates the
limitations for protocol 2. A subsequently rejected trailing record still
produces a failed run; retained readback is intermediate evidence, never a pass.
Raw serial and parsed protocol records retain initial/final clock metadata.

## API applicability

The following enumerates all 54 active identifiers. No decision is pending.
The existing metadata/license exceptions are carried forward with the scope and
revisit triggers documented in the initial review; no additional exception is
introduced.

| Entries | Decision | Evidence or concrete reason |
| --- | --- | --- |
| C-CASE, C-WORD-ORDER | Applies | Rust naming retained; `_mhz`, `_ns`, and sample-count fields consistently state their units/meaning. |
| C-CONV, C-CONV-TRAITS, C-CONV-SPECIFIC | Not applicable | No conversion interface or conversion-trait implementation added; clock validation checks evidence rather than exposing a conversion API. |
| C-GETTER | Applies | `Boot::hello` and `Boot::clocks_after` return optional shared borrows using noun names. |
| C-ITER, C-ITER-TY | Not applicable | No authored iterator API or named iterator type. |
| C-FEATURE | Not applicable | No crate features introduced. |
| C-COMMON-TRAITS | Applies | New clock values derive useful `Debug`, `Eq`, `PartialEq`, and serde traits; equality checks complete before/after configuration. Live collector state is not cloned. |
| C-COLLECT | Not applicable | Internal standard-library maps/vectors, no custom collection. |
| C-SERDE | Applies | Wire records remain typed interchange data; clock structs reject unknown fields. Explicit completion serialization preserves protocol 1's old output while retaining protocol 2 counts/readback. |
| C-SEND-SYNC | Applies | Single host context; exclusive mutable boot state and writers. No manual sharing traits or cross-context assumptions. |
| C-GOOD-ERR | Applies | Missing/readback/configuration/count/checksum errors use contextual `anyhow::Result`; ordinary input failure never panics. No exported error type. |
| C-NUM-FMT | Not applicable | No numeric wrapper API. Wire numbers remain JSON numbers; field names state units. |
| C-RW-VALUE | Applies | Existing owned/borrowed `BufRead` and `Write` abstractions retained. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable | No authored macro interface. |
| C-CRATE-DOC | Applies at application scope | Binary/module docs and README describe both protocols, units, clock trust boundary, errors and operational limitations. |
| C-EXAMPLE | Not applicable | No exported Rust library API; README gives actual CLI invocations and clearly labels illustrative register values. Tests exercise complete realistic protocol sequences. |
| C-QUESTION-MARK | Not applicable | No fallible Rust documentation example. Production error paths propagate with `?`; assertions/unwraps remain in tests. |
| C-FAILURE | Applies | Acceptance documents permanent rejection, incompatible records, incomplete counts and quantization; README/review state clock and measurement limitations. |
| C-LINK | Applies | Review links standard, inventory, original review and application contract. |
| C-METADATA | Exception M1, unchanged | Internal unpublished package; no fabricated authorship, repository or project-license metadata. See initial review for scope/revisit trigger. |
| C-RELNOTES | Not applicable | No publication/versioned release; meaningful changes committed using the repository standard. |
| C-HIDDEN | Applies | All implementation interfaces remain private or `pub(crate)`. |
| C-SMART-PTR, C-DEREF | Not applicable | No authored smart pointer or dereference implementation. |
| C-METHOD | Applies | Clock validation, hello contract checks and boot record acceptance are associated with their owning types. |
| C-NO-OUT | Applies | Clock/configuration checks return `Result`; summaries return values. Existing mutable streams/state intentionally preserve evidence. |
| C-OVERLOAD | Not applicable | No authored operator implementation. |
| C-CTOR | Applies | Existing valid empty `Boot::default` and nonempty `Summary::from_samples` construction remain; no new constructor contract. |
| C-INTERMEDIATE | Applies | Initial/final raw registers, raw records and supplied provenance survive; failed/incomplete sequences produce no derived result. |
| C-CALLER-CONTROL | Applies | Existing borrowed paths/strings/streams retained; snapshots are borrowed when inspected. |
| C-GENERIC | Applies | Read/write bounds match used operations; fixed-width wire/register integers match the device protocol. No generic framework added. |
| C-OBJECT | Not applicable | No authored trait or dynamic-dispatch API. |
| C-NEWTYPE, C-CUSTOM-TYPE | Applies | Named `Clocks`, `ClockRegisters`, `Complete`, `Hello`, `Sample` and `Boot` distinguish wire evidence and live state; explicit clock/time units and validation prevent mixed-unit input. |
| C-BITFLAG | Not applicable | Raw register snapshots are numbers, not an exposed combinable-flag API; shifts/masks decode the documented fields. |
| C-BUILDER | Not applicable | No incremental builder/configuration API introduced. |
| C-VALIDATE | Applies | Exact declared/actual counts, platform-specific capabilities, timer units, checksum/index/protocol ordering, message configuration and independently decoded equal clock snapshots are checked and tested. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Not applicable to authored destructors | No custom `Drop`; existing explicit stream flushing still surfaces errors outside device timing. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies | All authored records/state provide field-bearing derived `Debug`; no credentials introduced. |
| C-SEALED | Not applicable | No authored trait. |
| C-STRUCT-PRIVATE | Applies | Wire and validation state fields remain private. |
| C-NEWTYPE-HIDE | Not applicable | No representation-dependent newtype. |
| C-STRUCT-BOUNDS | Not applicable | No generic data structure introduced. |
| C-STABLE | Not applicable | Unpublished pre-1.0 internal application, no stable exported Rust API. Protocol 1 archival output is explicitly preserved. |
| C-PERMISSIVE | Exceptions L1/L2, unchanged | Project license unchosen; existing upstream `serialport` remains MPL-2.0. No dependency added or license changed. Initial review records consequences and distribution revisit trigger. |

## Verification

Actual commands/results are recorded below. These host checks establish parsing,
validation, compatibility and statistics behavior; they are not on-board passes.

| Command | Actual result |
| --- | --- |
| `cargo +1.96.0 test -p benchctl --locked` | All 18 tests passed. Coverage includes complete platform-specific sets and 1,000-sample p99 behavior; missing/false/changed clock evidence; declarations, metadata and completion counts; corruption, ordering, protocol mismatch and reboot; original v1 tests and completion serialization. |
| `cargo +1.96.0 fmt -p benchctl -- --check` | Passed. |
| `cargo +1.96.0 clippy -p benchctl --locked --all-targets -- -D warnings` | Passed after replacing the new manual divisibility check with the stable integer `is_multiple_of` method; no suppression introduced. |
| `cargo +1.96.0 doc -p benchctl --locked --no-deps` | Passed. |
| `cargo +1.96.0 build -p benchctl --locked --release` | Passed. Final collector SHA-256: `6f1a1a38c5079492f957a12ea76193f6b3ce7a8fe5d9dbbbb81c1331b327ac55`. |
| `benchctl validate --input <each initial serial.raw> --platform <run platform>` against prior/current binaries, followed by `diff -u` | All 15 complete protocol 1 validation outputs byte-identical; each replay passed. Prior binary and comparison output retained temporarily under `/tmp/esp32-c6-benchctl-reference.PPzT87`. |

No hardware, reset waveform, absolute-frequency instrument, firmware deployment,
power, radio, IRQ, or scheduler measurements were performed by this review. The
hardware operator records those separately with raw logs/build provenance.
