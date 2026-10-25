# Collector API and safety contract

This internal host application uses Rust 1.96.0 and `std`. The
[README](../README.md) defines its CLI; the
[coding standard](../../../../../docs/coding-standard.md) and
[54-entry inventory](../../../../../docs/rust-api-checklist.md) govern this record.

## Execution and evidence

Each capture owns its serial port, framer, validator and output files on one host
thread. File/serial I/O, process execution and timeouts may block. No authored
unsafe code, firmware FFI, interrupt handler or async cancellation API is present.
An interrupted process can leave an incomplete capture; it has no passing result.

The common boot-text guard admits one startup and permanently rejects a second
startup, fatal diagnostic or malformed frame. It also checks bytes after a
completion within the same read and a pending partial line completed by a later read.
Raw bytes are saved before interpretation. Ordinary failure never becomes a
zero-valued score or a favorable subset of a failed run.

Protocol 1/2 validates its declared counts, units and checksums; protocol 2 also
checks decoded, matching endpoint clocks. Queue, accounting, load and RTIC
collectors have distinct contracts: diagnostic controls do not become comparison
results. The supplied image/provenance and device output do not authenticate which
firmware ran or calibrate its oscillator.

Capture and replay share validation. Saved floating-point values must round-trip
exactly through `serde_json`; `float_roundtrip` is therefore enabled. The
fractional-mean regression uses retained physical raw data, while corruption
controls verify that changing a report still rejects. This parser correction
does not change original measurements.

Method admission checks its exact source/configuration/image identities and
passing/failing controls before comparative capture. Publication verification
has a narrower contract: it checks current files, declared historical identities
and supported raw replays. Explicit path moves do not supply omitted binaries or
restore unavailable freeze ancestry. Full-artifact commands receive no aliases.
Retired generated files remain declared unavailable; physical cohorts retain
their successful and failed observations.

Relocations require exact tracked-file coverage, matching current hashes and
regular files without symlink traversal. Evidence moves preserve the original
OS archive/ledger name beneath its project directory. An explicit file alias
cannot conceal an omitted file or its descendants under a containing directory
alias. New, labeled derivatives may coexist with honest omission declarations.

Schema 2 distinguishes omitted historical inputs (with their original hashes)
from retired, previously added publications (with both hash fields absent and
an explicit reason). Retirement permits historical Markdown links to identify
unavailable text without inventing an original identity. Both statuses reject
tracked replacements and descendants, and neither permits retiring protected
raw captures or observation receipts. Expected original-hash checks stay strict.

## API applicability

Every active identifier is covered below. The decisions apply to the current
collector modules, with no exported Rust library surface.

| Entries | Decision | Evidence or concrete reason |
| --- | --- | --- |
| C-CASE, C-WORD-ORDER | Applies | Rust casing and explicit tick, ns, us, MHz and byte field names keep protocol units visible. |
| C-CONV, C-CONV-TRAITS, C-CONV-SPECIFIC | Not applicable | No conversion interface or conversion-trait implementation added; clock validation checks evidence rather than exposing a conversion API. |
| C-GETTER | Applies | `Boot::hello` and `Boot::clocks_after` return optional shared borrows using noun names. |
| C-ITER, C-ITER-TY | Not applicable | No authored iterator API or named iterator type. |
| C-FEATURE | Not applicable | No crate features introduced. |
| C-COMMON-TRAITS | Applies | New clock values derive useful `Debug`, `Eq`, `PartialEq`, and serde traits; equality checks complete before/after configuration. Live collector state is not cloned. |
| C-COLLECT | Not applicable | Internal standard-library maps/vectors, no custom collection. |
| C-SERDE | Applies | Versioned wire, report, admission and manifest records have typed serialization/deserialization, including schema-2 retirement records. Archive JSON uses float_roundtrip and exact comparison. Live validator state is not an interchange record. |
| C-SEND-SYNC | Applies | Single host context; exclusive mutable boot state and writers. No manual sharing traits or cross-context assumptions. |
| C-GOOD-ERR | Applies | Missing/readback/configuration/count/checksum errors use contextual `anyhow::Result`; ordinary input failure never panics. No exported error type. |
| C-NUM-FMT | Not applicable | No numeric wrapper API. Wire numbers remain JSON numbers; field names state units. |
| C-RW-VALUE | Applies | Existing owned/borrowed `BufRead` and `Write` abstractions retained. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable | No authored macro interface. |
| C-CRATE-DOC | Applies at application scope | Module docs and CLI help describe each collector, replay, method and evidence command. The README gives invocations. |
| C-EXAMPLE | Not applicable | No exported Rust library API; README gives actual CLI invocations and clearly labels illustrative register values. Tests exercise complete realistic protocol sequences. |
| C-QUESTION-MARK | Not applicable | No fallible Rust documentation example. Production error paths propagate with `?`; assertions/unwraps remain in tests. |
| C-FAILURE | Applies | Malformed, incomplete, rebooted and failed captures retain failure status and raw bytes. File/serial errors return contextual errors. Timeouts and unavailable artifacts have explicit limits. |
| C-LINK | Applies | Module documentation, the tool README and this contract link the source, operational commands, methods and interpretation limits. |
| C-METADATA | Exception M1 | Accurate unpublished package identity, description and compiler requirement; ownership, repository and project-license metadata are not invented. See the exception below. |
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
| C-NEWTYPE, C-CUSTOM-TYPE | Applies | Named clock/protocol records distinguish wire evidence and live state. Publication `Status` distinguishes original omissions from retired additions; validation prevents an unavailable record from asserting current bytes. |
| C-BITFLAG | Not applicable | Raw register snapshots are numbers, not an exposed combinable-flag API; shifts/masks decode the documented fields. |
| C-BUILDER | Not applicable | No incremental builder/configuration API introduced. |
| C-VALIDATE | Applies | Protocol-specific bounds, units, configuration, checksums, ordering and completeness precede scoring. Raw replay checks saved reports; path, inventory and identity checks precede evidence success. Contrary fixtures exercise rejection. |
| C-DTOR-FAIL, C-DTOR-BLOCK | Not applicable to authored destructors | No custom `Drop`; existing explicit stream flushing still surfaces errors outside device timing. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies | All authored records/state provide field-bearing derived `Debug`; no credentials introduced. |
| C-SEALED | Not applicable | No authored trait. |
| C-STRUCT-PRIVATE | Applies | Wire and validation state fields remain private. |
| C-NEWTYPE-HIDE | Not applicable | No representation-dependent newtype. |
| C-STRUCT-BOUNDS | Not applicable | No generic data structure introduced. |
| C-STABLE | Not applicable | Unpublished pre-1.0 internal application, no stable exported Rust API. Protocol 1 archival output is explicitly preserved. |
| C-PERMISSIVE | Exceptions L1/L2 | Project licensing remains undecided; serialport 4.10.1 declares MPL-2.0. Dependency notices and project licensing need review before distribution. |


M1: this application is unpublished. Metadata reflects established facts;
unassigned authorship, repository details and licensing remain unset. Resolve
publication metadata and licensing before distribution. L1/L2 additionally cover
the existing serialport dependency's MPL-2.0 declaration; they do not assign that
license to authored code or assert distribution compliance.

## Regression checks

The workspace tests exercise complete streams, corrupt/reordered/incomplete
records, repeated boots and fatal text, fractional saved values, failed-capture
retention, method input mismatches, archive tampering and relocated evidence.
The maintained commands are in the tool and project READMEs. Host controls cover
these software contracts; device timing, reset behavior and radio operation need
their own observations.
