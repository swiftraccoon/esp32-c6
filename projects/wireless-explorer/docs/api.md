# Wireless API and safety contract

The [coding standard](../../../docs/coding-standard.md) and
[54-entry inventory](../../../docs/rust-api-checklist.md) govern
`wireless-protocol`, `wxctl` and `wireless-explorer-firmware`.
The [protocol](protocol.md) and [observation contract](observation.md) define
wire behavior; [results](../results/README.md) record observed coverage.

All three packages are internal, version 0.1.0, unpublished, edition 2024,
formatted as edition 2024, and use Rust 1.96.0. The declared supported compiler
is the tested baseline, not a claim that earlier compilers cannot build them.
The project workspace contains the protocol and desktop client; firmware has
its own workspace, lockfile, target, and SDK configuration.

The build manifests/toolchain/configuration were digested before collection
with a final blank line. Narrow `.gitattributes` entries preserve those exact
input bytes while disabling only Git's blank-at-EOF diagnostic for those files.
Rust formatting, other whitespace checks, and input digest equality still run;
the frozen inputs are not rewritten for a cosmetic cleanup.

## Applicability

Each active guideline is considered below. Applies means the stated scope and
evidence, not proof of every future use. No applicability decision is pending.

| Entry | Decision | Evidence or reason |
| --- | --- | --- |
| C-CASE | Applies | Rust identifier conventions; wire names use explicit snake_case serialization. |
| C-CONV | Not applicable | No authored conversion-named methods. |
| C-GETTER | Applies | `id`, `command`, `reply`, `is_pending`; no `get_` accessor prefixes. |
| C-ITER | Not applicable | No exposed collection iteration API. |
| C-ITER-TY | Not applicable | No authored iterator type. |
| C-FEATURE | Not applicable | No project-defined feature flags. |
| C-WORD-ORDER | Applies | `decode_request`/`decode_response`; command and response types distinguish directions. |
| C-COMMON-TRAITS | Applies | Control/packet values derive Debug/Clone/Copy/Eq/Hash/Serde; bounded stream events derive Debug/Clone/Eq/Hash/Serde without unnecessary Copy. Decoder has Debug/Default. IDs, packets and live handles have no meaningful universal default. Hardware ownership is not duplicated. |
| C-CONV-TRAITS | Not applicable | No public conversion between authored types. |
| C-COLLECT | Exception B1 | Private fixed packet buffer exposes checked push/pop closures and explicit contention/full results. Infallible collection construction would hide loss or allocation; no collection trait is claimed. |
| C-SERDE | Applies | Strict versioned request/response records derive Serialize/Deserialize; decoder and device handles are not interchange records. |
| C-SEND-SYNC | Applies | Protocol values have automatic traits. Buffer's unsafe Sync requires Copy + Send; atomic producer/consumer gates and acquire/release head/tail publication keep slot access disjoint. Independent concurrent, wrap, contention and panic controls exercise this contract. Drivers remain main-task-owned. |
| C-GOOD-ERR | Applies | `FrameError` implements Display and core::error::Error; host adds anyhow context; radio errors retain SDK codes. |
| C-NUM-FMT | Not applicable | No authored numeric or bitmask wrapper. |
| C-RW-VALUE | Applies | Host Consumer<E: Write, P: Write> and Compare<R: Read> accept owned or borrowed streams by value; live collection and replay share the consumer. |
| C-EVOCATIVE | Not applicable | No authored macro. |
| C-MACRO-ATTR | Not applicable | No item-producing macro. |
| C-ANYWHERE | Not applicable | No authored macro. |
| C-MACRO-VIS | Not applicable | No authored macro. |
| C-MACRO-TY | Not applicable | No authored macro. |
| C-CRATE-DOC | Applies | Crate-level protocol contract, executable docs, and project READMEs. |
| C-EXAMPLE | Applies | Protocol doctest exercises encode/decode; README CLI examples identify required hardware. |
| C-QUESTION-MARK | Applies | Fallible doctest uses `?`; error cases are ordinary Results. |
| C-FAILURE | Applies | Public errors/bounds documented; malformed inputs, capture failures, and radio errors are separate. |
| C-LINK | Applies | Intra-doc references connect decoding, validation and errors; protocol/build/result docs are linked. |
| C-METADATA | Exception M1 | Accurate names/descriptions/edition/compiler/publication status; no invented author, license, or distribution URL. |
| C-RELNOTES | Not applicable | No published release; change is recorded in Git and project status. |
| C-HIDDEN | Applies | Public protocol API is documented; implementation buffers and executable helpers remain private. |
| C-SMART-PTR | Not applicable | No authored smart pointer. |
| C-CONV-SPECIFIC | Not applicable | No authored conversion interface. |
| C-METHOD | Applies | Request/response accessors and validation, decoder operations are associated methods. |
| C-NO-OUT | Applies | Parsing returns typed values; encode uses a caller buffer as required by bounded serialization and returns used length. |
| C-OVERLOAD | Not applicable | No operator overload. |
| C-DEREF | Not applicable | No Deref implementation. |
| C-CTOR | Applies | `new` establishes versioned records and empty decoders; no invalid hardware constructor is exposed. |
| C-INTERMEDIATE | Applies | Decode and semantic validation are distinct; raw bytes and failed attempts remain available. |
| C-CALLER-CONTROL | Applies | Caller owns serialization storage; decoder returns a borrowed line with Rust-enforced lifetime. |
| C-GENERIC | Applies | Serializer accepts Serialize values; no unnecessary generic firmware framework. |
| C-OBJECT | Not applicable | No authored public trait for dynamic dispatch. |
| C-NEWTYPE | Applies | Commands, transports, stages, request/response and errors are distinct types; unit-bearing fields name their units. |
| C-CUSTOM-TYPE | Applies | Command/Reply enums replace ambiguous operation booleans; Info booleans explicitly report command availability. |
| C-BITFLAG | Not applicable | No combinable authored flags. |
| C-BUILDER | Not applicable | Constructors have small fixed contracts; upstream drivers provide configuration builders. |
| C-VALIDATE | Applies | Control validation plus stream transitions, radio/channel identity, monotonic timestamps, exact chunks, byte caps, packet/loss conservation and complete live/save/replay equality. File caps read one extra byte to reject hidden suffixes; exact inventories and non-symlink inputs are checked. |
| C-DTOR-FAIL | Applies with D1 | Buffer gate Drop only releases an atomic flag; no fallible action. SDK handles can fail/panic during teardown. Explicit radio stop precedes final End; uncertain stop terminates command handling. Test directory cleanup ignores only its own removal failure. |
| C-DTOR-BLOCK | Applies with D1 | Gate Drop is one atomic store. SDK and filesystem cleanup may block. No blocking cleanup is performed by callback-owned objects. |
| C-DEBUG | Applies | Public protocol/stream types have Debug. Packet/chunk diagnostics can contain addresses and payloads; raw captures stay in ignored private storage and public summaries omit address values. |
| C-DEBUG-NONEMPTY | Applies | Value/error diagnostics contain variant and field details; decoder Debug exposes buffered bytes, which must not carry secrets. |
| C-SEALED | Not applicable | No authored trait requiring sealing. |
| C-STRUCT-PRIVATE | Applies | Version fields, decoder and buffer internals are private. Packet metadata fields and event variants are intentional interchange data; the consumer validates them before deriving an admitted result. |
| C-NEWTYPE-HIDE | Not applicable | No public implementation-dependent wrapper representation. |
| C-STRUCT-BOUNDS | Not applicable | No public generic structure. |
| C-STABLE | Exception S1 | Internal version 0.1 API; wire version checked; future breaking changes require versioning. Firmware build-std needs scoped RUSTC_BOOTSTRAP. |
| C-PERMISSIVE | Exception L1 | Project license not selected; publish=false. Upstream terms remain intact, including serialport MPL-2.0 and vendor radio blobs. |

M1/S1/L1 are narrow initial-project exceptions, consistent with existing
repository practice. Revisit metadata and licensing before distribution, and
API/compiler commitments before any reusable public release. Do not fabricate
a license or call build-std an ordinary prebuilt stable target.

B1 applies only to the internal bounded buffer. Its capacity is a nonzero power
of two; a producer or consumer that cannot acquire its gate returns immediately.
No allocation, waiting, overwrite or unpublished reference is exposed. Revisit
the interface if it becomes a reusable public collection.

D1: SDK handles live for the main loop. Upstream Drop may stop/delete SDK
resources and can panic on failure. The recorder explicitly stops the active
radio before End. A failed stop or uncertain partial IEEE 802.15.4 initialization
terminates command handling; recovery and destructor behavior after such failures
are not qualified.

## Recorder execution and safety

802.15.4 shutdown applies C-VALIDATE, C-SEND-SYNC, C-DTOR-FAIL
and C-DTOR-BLOCK: disable continuous receive, issue sleep/CMD_STOP while the
driver is initialized, then attempt disable even if quiescing failed. Either
error terminates command handling. The SDK can invoke a synchronous
callback under its stop critical section. Original zero-report transition
windows remain preserved; on-board observations are recorded in results.

Protocol and host forbid unsafe code. The protocol remains no_std without heap
allocation: control lines are capped at 512 bytes, capture lines at 1024, chunks
at 128 decoded bytes and snapshots at 4096 bytes. Event has a scoped
large-enum-variant lint allowance because inline bounded chunk storage avoids
heap indirection. New framing does not broaden legacy control decoding.

Firmware denies unsafe code outside the reviewed buffer and radio adapter
modules, and denies implicit unsafe operations in unsafe functions. Generated
bindings come from pinned SDK headers through esp-idf-sys/bindgen and libclang
21.1.8. No authored C/assembly implementation is added. Every foreign call,
pointer dereference, callback ABI and unsafe Sync implementation has an invariant
comment.

Main owns radio start/stop and serial I/O. Wi-Fi callbacks run in the SDK Wi-Fi
task; BLE callbacks run through the Bluedroid service; IEEE 802.15.4 callbacks run
in an ISR or synchronously inside the SDK stop critical section. Callback buffers
are borrowed only until return, copied into fixed
slots, and never stored as foreign pointers. IEEE 802.15.4 returns each non-null
SDK frame once even on drop. Its callback entry is placed in IRAM, but a complete
cache-off call-graph qualification has not been performed. Capture performs no
flash writes and disables persistence.

Buffer slots have one producer and one consumer at a time, enforced by atomic
try-gates. Acquire/release publication prevents reading uninitialized/reused
slots. Tail advances after the consumer closure returns; a callback cannot reuse
a held slot. Gates release on unwinding in host controls; production firmware
uses panic abort. Copy + Send excludes destructors and non-sendable payloads.
Index arithmetic wraps explicitly, with power-of-two slot indexing. Callback
counters use target-supported 32-bit atomics; 60-second windows cannot physically
reach their 2^32 wrap point. Capture gate and in-callback counters use sequential
consistency to establish a closed/drained boundary before reading final counts.

No callback allocates, formats, waits, writes serial or takes a blocking mutex.
Main may wait for SDK setup/shutdown, BLE completion (three-second polling
deadline) or each serial record (one-second application deadline). SDK calls
themselves can block beyond these polls; no hard real-time deadline is claimed.
The requested dwell can overrun while the main task emits a packet. Native USB
write success means queued bytes. A complete host End/replay is required.

Host file/serial I/O and JSON/summary collections allocate and can block.
Raw bytes have a per-window 32 MiB cap; schedule length is capped at 3000 windows;
metadata files have explicit caps. Address sets and JSONL are bounded by admitted
raw input. Parsing and hex conversion return errors, with malformed AD fields
counted without discarding the underlying packet. No authored async cancellation
exists. Process cancellation leaves partial files; the bounded device window
expires independently. Replay rejects missing completion, retained failures,
extra files, symlinked inputs and changed derived output.


## Callback boundaries

- BLE's event discriminator selects the extended-report union member. Its
  reported length is checked against the 251-byte array. Static scan parameters
  remain live through asynchronous completion and timeout.
- IEEE 802.15.4 requires a length of 2–127 before slicing. Buffer-release failure
  terminates command handling; it is not an ordinary invalid-packet count.
  Release occurs after the in-callback counter decrement, so shutdown also
  depends on the pinned single-core interrupt teardown. That counter alone is
  not a portable cross-core reclamation proof.
- `at_us` describes callback delivery. Setup reports may precede Listening;
  admitted packet timestamps remain between Start and stop. These timestamps
  are not calibrated over-air arrival times.

The retained [IEEE callback disassembly](../results/observer-qualification-2026-10-24/review/ieee-rx-disassembly.txt)
comes from inspection ELF
`a7d29327604a63d10ecfd7bb1b1d99e005963e6650393c0c9b8b43c661ebf53f`
and generated SDK configuration
`3196298ec9c99269ace480727adddf712bda14cc33e33fcfeccd022c520c64e5`.
This predates the explicit Bluetooth bond-NVS disable setting; it does not
identify the final deployed image. The callback calls a flash-resident release
helper, so placing its entry in IRAM does not qualify cache-off execution.

## Host admission and coverage

Only `ESP_ERR_TIMEOUT` becomes an empty UART read; other driver errors
propagate. Any `failure.txt` directory entry prevents replay promotion,
including a dangling symlink. Coverage records reject duplicate, missing and
unknown keys. UART coverage combines an info and ping somewhere in the ledger;
it does not bind them to one authenticated device or session.

Replay checks internal consistency and expected files. It does not defend
against concurrent filesystem mutation or coordinated replacement with a
different self-consistent capture. Parser and buffer controls do not establish
RF fidelity, silence, losslessness, phone access or hardware recovery.

## Provenance of this document

This is the maintained contract. Historical observer source-input manifests name
`docs/rust-review.md`: inputs-01 declares
`7898b194a97cf4e8c640a2575f0e7f213feda25a68fef165d3db08f131e68ed2`;
inputs-02 declares
`7b8e3fab8ec944598e905a727114ec29c74ba65b5e0a6b3171d4ebface65c818`.
Those declarations and original documents in Git history remain unchanged.
This revised text is a derivative, not a byte-identical frozen build input.
