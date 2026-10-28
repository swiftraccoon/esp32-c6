# Device identity API and safety contract

`identityctl`, `runtime-protocol`, `wifi-protocol`, `usb-observer`,
`wifi-observer`, `kismet-worker`, `device-identity-firmware`,
`device-identity-ble-firmware` and `device-identity-wifi-firmware` are internal,
unpublished Rust 0.1 packages. The project
workspace pins Rust 1.96.0, language and formatting edition 2024, `rustfmt`,
Clippy and dependencies in `Cargo.lock`. The native build target is
`aarch64-apple-darwin`; CI is configured to build offline replay on Linux.
This is the tested baseline, not an inferred minimum for all dependencies.
The host does not compile ESP-IDF bindings or firmware. The macOS USB observer
generates native bindings against SDK 27.0 with bindgen 0.72.1 and libclang
21.1.8, recording header and generated-binding hashes. The firmware has an
independent workspace/lock and target `riscv32imac-esp-espidf`, using scoped
`build-std`/`RUSTC_BOOTSTRAP=1` with the same Rust release. Its sys/HAL wrappers
pin upstream revision `a209052f378be61dd0f2d566b7fbe0f9593b0cd5` and ESP-IDF
5.5.6 commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`. The firmware build
uses ldproxy 0.3.4 and libclang 21.1.8; inspect its README and actual generated
SDK configuration for enabled components. Project licensing remains undecided.

Summary/comparison/replay commands read saved private records and emit fixed
summaries without opening a device. `runtime-capture` opens the specified UART,
sends fixed probe requests and invokes the Mac USB profiler. It does not flash
or program persistent identities. Line release can start/reset the C6. The
[inventory procedure](../experiments/read-only-inventory.md) owns capture,
application reset and evidence boundaries. Its [result ledger](../results/README.md)
records actual observations. The [runtime method](../experiments/runtime-table.md)
defines firmware effects, controls and the full-flash restoration boundary.
Parsing input does not authenticate its origin.

## Execution and privacy

Host commands expose synchronous operations using `std` allocation and blocking
file/stdout I/O. Receiver/UART capture uses scoped reader/watcher threads that
are joined before return. Inputs are capped at 8 MiB, including a read of one
extra byte to detect an oversized stream. Files must be regular, with no
symlink path components or parent traversal. This is a local snapshot check;
concurrent hostile filesystem replacement is outside its contract. No file is
modified by replay. Capture writes new private records; it rejects an existing
destination or a path outside ignored `private/device-identity/`. No background
worker or cancellation API is exposed; `identityctl` and the runtime protocol
forbid unsafe code. The USB observer denies it outside the reviewed native
adapter and generated bindings.
The normal host panic strategy is unwinding. Malformed
inputs use ordinary errors; allocation exhaustion is not a recoverable outcome.

Only fixed field names, enums, booleans and bounded input-derived counts reach
stdout. Serial contents, eFuse bytes, arbitrary metadata and input paths never
reach the report. Parser and argument errors become static diagnostics, without
printing an upstream error or argv. Serialization completes before output begins;
an output device failure can still leave a partial write and returns failure.
Input structures containing raw values have no derived diagnostic formatting.

Reports describe serial-property presence and readable zero/nonzero field
contents. They establish no uniqueness, bridge lock state, RF behavior or
correlation reduction. Unknown input schemas and malformed selected metadata
fail instead of becoming an absence observation. Duplicated recognized JSON
fields fail. The USB schema intentionally ignores unknown properties but never
copies them into output; eFuse input is restricted to the three selected fields.

USB comparison uses property contents only for a single nonempty match in each
snapshot. Duplicate models remain ambiguous. Properties can be cached by macOS;
equality does not establish fresh descriptor bytes or re-enumeration behavior.

Revision 2 checks explicit DTR=false then RTS=false after serial open, releasing
the bridge's auto-programming lines. This can release/reset the application.
Startup bytes have a separate bounded one-second record before requests;
empty startup or one complete recognized ROM boot block is permitted, while
panic/error, malformed/truncated boot records or unexpected text
fails. Replay checks startup bytes/receipt and the requested line policy.
OS/driver open/close can still reset a device; macOS serialport also flushes
buffers internally during open. Raw reception starts after a successful open,
so pre-open/discarded bytes are unavailable. Request/reply IDs
and phases reject stale individual replies. A capture uses a fresh OS-random
correlation token; this is not authentication. Bounded line reads and response
deadlines limit collection, without a hard real-time claim for OS I/O/profiler
execution. A best-effort restore request after failure is separately recorded.

`runtime-summary` validates table relationships in four private request/reply
pairs only. `runtime-replay` additionally requires a completed capture receipt,
complete fixed file inventory, raw/transcript agreement, profiler outcomes and
saved-summary equality. A table-only pass cannot admit a failed capture. Getter
readback establishes software table state, not active-driver state or emitted
addresses. Private values serialize through the protocol; their Debug is redacted.

## Native USB control observer

`usb-control-capture` uses the [fixed USB method](../experiments/usb-control.md).
Its `usb-observer` dependency selects known VID/PID services through numeric
IORegistry properties, creates a plugin and queries `IOUSBDeviceInterface942`.
It does not call USB device open/seize/close, interface claims, detach, reset,
configuration setters or modem controls. The selected SDK permits these
default-control-pipe read requests without opening the device. Plugin/device
COM references use `Release`; `IODestroyPlugInInterface` is excluded because
that operation can stop the associated service.

Generated bindings define the ABI, including COM double-pointer `self`,
`IOUSBDevRequestTO` layout and callback/function pointer types. Every unsafe
call documents pointer validity, reference ownership and synchronous lifetime.
Main alone owns native references. No callback, borrowed output pointer or
unsafe sharing implementation is exposed. Buffers are initialized and fixed
for each synchronous request; returned lengths are checked before admission.
Failed partial responses retain only the reported prefix, never an unreturned
zero-filled suffix. Explicit cleanup reports errors; infallible Drop guards
provide fallback release and may block in system code. A 1 s request timeout
does not establish a hard bound on OS/device completion.

The native allowlist and shared planner emit only standard IN device/string
descriptor requests and the qualified CP2102N part/configuration IN requests.
Fresh descriptor VID/PID, indices, LANGIDs, UTF-16, byte counts, version and
record order are checked. No serial index skips string requests rather than
inventing an empty value. The original capture summary restricts configuration interpretation to actual
678-byte response length, layout byte two and the documented default comparison
at byte four. The separate saved audit below checks size/checksum without
changing that summary or establishing lock state.
No endpoint response authenticates a device or supplies a USB bus trace.

Request plan 2 starts LANGID strings with four bytes and serial strings with two,
then requests the validated declared length (at most 254 bytes), checking full
prefix equality across reads. This replaces a retained failed 255-byte native
LANGID attempt without asserting its cause or silently changing its outcome.

`usb-control-summary` and `usb-control-compare` validate saved records only.
Capture admission requires `usb-control-replay`: exact four-file inventory,
success receipt, supported native provenance, byte hashes and complete semantic
replay with exact saved summary equality. Raw/provenance output storage is
created before native observation. Failure preserves the partial record and
failed receipt without a public summary. Raw USB input is capped at 64 KiB;
control responses at 678 bytes and string descriptors at 254 bytes. Native
capture requires macOS; offline parser/replay controls are portable.

Native SDK provenance contains only relative header identities and hashes,
SDK 27.0, bindgen 0.72.1, libclang 21.1.8 and generated-binding identity. Changing
the selected SDK/compiler requires an explicit method/provenance review.
Native observation has no logging of serial/configuration values; serde raw
records belong in private storage and raw Debug redacts their bytes.

## Saved USB and persistence audits

The [retrospective audit method](../experiments/saved-identity-audits.md) defines
the complete original cohort and qualification. Both USB derivatives use an
owned record returned by whole four-file replay, avoiding a separate raw-file
read after admission. `usb-factory-relation` validates the selected eFuse
metadata and readable nonzero unicast MAC, then compares it with exactly 17
colon-separated ASCII hex code units from the native UTF-16LE serial. Valid
inequality is false; malformed/protected/unsupported input fails.

`bridge-config-audit` supports only the declared 678-byte version-one encoding:
little-endian size and big-endian Fletcher16 over bytes 0..676. The algorithm
uses AN978's initial sums and bounded carry reductions. The observed byte order
is a separately stated format restriction. Neither a checksum nor the byte-four
default comparison qualifies bridge lock or permission to program it. No new
USB request is added by either derivative.

`flash-metadata-audit` borrows exactly 8 MiB, validates the table MD5 and supported
ranges/flags, then all supported physical NVS v1/v2 page/header/payload CRCs,
bitmap states, bounded spans and partition-local namespace references. It
rejects encrypted flags, nonempty NVS-key partitions and unexplained integrity
failure. It uses exact pinned `md-5` and `crc32fast` dependencies; the latter's
initial value is checked against the SDK generator and independent bit oracle.
Input-derived labels/keys remain private borrowed byte strings without Debug;
only fixed aggregates and fixed ASCII name-heuristic counts are reported.

No logical values are reconstructed. Erased entries, opaque blobs and non-NVS
partitions remain unaudited, and name categories do not establish credential
presence/absence. CRC failure cannot distinguish encryption from corruption.
Neither a valid digest nor a saved receipt authenticates a device. These pure
parsers have no device I/O, callbacks, unsafe code or asynchronous cancellation.

## Station probe witness

`station_probe_witness` is a separate consumer over the existing complete
PCAPNG/radiotap/FCS path. The ordinary `analyze`, beacon witnesses and
whole-capture beacon comparisons retain their existing behavior and formats.
Only the station consumer interprets supported broadcast-DA/BSSID probe requests,
checking the unicast transmitter, clear DS/MoreFrag/Protected/Order bits,
unfragmented 24-byte header, bounded IEs and exactly one SSID. All other supported
frame classes stay in the complete capture summary; unsupported variants and
unqualified FCS cannot witness. Actual matching requires the full private SSID
and transmitter plus independently checked included CRC32. A malformed late
supported probe rejects the entire call, including a valid earlier match.

No target bytes, IE contents, timestamps or input hashes enter output/Debug.
The caller owns nonce freshness. This pure parser does not configure a station,
prove association/reconnect/cold boot or establish packet-loss/silence coverage.

## Controlled BLE firmware

The independent [BLE workspace](../firmware-ble/README.md) has a pure no_std
protocol/controller/completion library and an ESP-IDF/std target binary. Status
leaves Bluetooth uninitialized. Domain `ble_advertising`, version 1, bounded
256/1024-byte request/reply frames, increasing IDs, nonzero nonce and ordered
StartFirst/Stop/StartSecond/Stop are checked. Each phase consumes its attempt;
only an explicit successful first Stop authorizes the second Start. Every error
permanently blocks Starts. Raw wire addresses/nonces/readbacks remain private
and have redacted Debug. The image implements only the two bounded advertising controls.

The safe HAL owns one Modem. Pinned SVC's unconditional GATT modules cannot
compile with GATTC/GATTS disabled, so the application instead uses a small Rust
generated-binding initialization adapter. Its 65-field C6 configuration is
transcribed from the pinned SDK macro and compared with pinned BtDriver's C6
branch. No handwritten ABI layout or C shim is added. An atomic singleton
attempt latch is never cleared, including on partial error. The configuration's
one bounded allocation and initialized ownership remain until reset; the binary
controller library may retain its pointer. No later Rust reference, deinit or
fallible destructor is introduced.

Main alone owns UART, configuration and deadlines. An atomic-only GAP callback
borrows the generated union member selected by its event, retaining no pointer
and performing no allocation, lock, logging, blocking or panic. Release/acquire
and a publishing state synchronize exact operation-kind completion. Same-kind
callbacks carry no generation ID: normal nonoverlap relies on the pinned FIFO,
one-completion source path. A delayed earlier Stop can satisfy a failed retry;
the sticky failure still prevents session admission or another Start. No exact
same-kind attribution is claimed.

GAP Stop success is host-stack command acceptance/event state. The pinned SDK
logs a later HCI disable failure without reporting it through that GAP callback.
Used-address queries also report SDK state. Neither proves physical controller
disable, transmitted identity or RF silence. Prestart readback explicitly
distinguishes not-requested/available/unavailable; active readback must report
the configured random-type control. An independent address-and-nonce witness
under the selected observation method is still mandatory. The accepted
[HCI method](../experiments/ble-hci-presence.md) observes receiver reports without
qualifying raw RF CRC. Stop retains random RAM settings until external reset.

The firmware-local start-operation check accepts only StartFirst/StartSecond
before the SDK start call and rejects Status/Stop. The controller still owns
phase ordering, stopped-state admission and the sticky failure latch; this
local check is not host authentication. Advertising is nonconnectable and
nonscannable legacy 1M on all primary advertising channels, with
one custom 128-bit service-data AD containing the nonce. Raw data has permanent
owned storage; pinned SDK queue calls deep-copy their borrowed inputs before
return. Each 8-second requested Stop deadline is armed before setup and checked
before Start; failed cleanup retries. SDK blocking/scheduling/panic prevent a
hard RF bound. GATT, SMP, scanning, BLE 5.0, IEEE and PHY storage are disabled in
the selected build. The hidden Wi-Fi capability remains compiled; application
source initializes no Wi-Fi/network/NVS service. Build evidence establishes no
on-board execution or observer readiness.

E1 scopes the C-GOOD-ERR exception to internal Radio/Mailbox raw i32 statuses
used by private canonical wire records. FrameError implements Display/Error.
A typed ControlError wrapper is the alternative; these seams currently lack
error composition and are not general driver APIs. Revisit E1 when extracting
the shared core for broader host callers. M1/L1/S1 retain their existing scope.

## Existing Wi-Fi receiver

The [Wi-Fi observer method](../experiments/wifi-observer.md) bounds a source-scoped
Kismet packet stream without retuning or controlling the datasource. The host
uses its pinned Rust 1.96.0 workspace. The independent std-only remote worker
pins the already installed Rust 1.90.0 on `x86_64-unknown-linux-gnu`; host tests
also compile its library under 1.96.0. Both use edition/style 2024. Its separate
manifest/lock and recorded remote executable/tool inputs do not change the host
toolchain or imply a general minimum version for dependencies.

`wifi-capture` reads a strict private profile and creates private output before
running trusted SSH with host-key checking. The profile UUID is normalized from
ASCII hex case before strict canonical request validation; arbitrary source text
remains rejected. The remote worker reads pre-resolved
credential paths under process/configuration guards. It supplies Basic auth to
curl through stdin for the fixed loopback source-stream GET only. It consumes
HTTP headers locally and emits fixed/numeric transport status, keeping credentials,
response cookies, raw headers and curl diagnostics out of the SSH output. The
source UUID, selection metadata, packet contents and SSH diagnostics remain
private. Profiles and all errors/Debug must not expose arbitrary values.

The worker stops after 32 complete EPBs, with a 120-second deadline and 8 MiB byte
cap. Intentional boundary completion, premature EOF, malformed framing, deadline,
size and process/cleanup failures are distinct. Process termination is explicit;
success requires HTTP 200, the declared prefix and successful cleanup. Blocking
OS I/O has no hard real-time guarantee. This sample count is not an RF duration.
The same pure framer is compiled in worker and parser, avoiding an independently
interpreted boundary. No cancellation API or shared live handle is exposed.

`wifi-summary` reads saved PCAPNG only and returns fixed counts. `wifi-replay`
additionally checks the exact seven-file inventory, completed transport/receipt,
request/provenance agreement, byte hashes, complete decoding and saved-summary
byte equality. A failed capture cannot be admitted by summarizing its prefix.
Read-only replay does not reconnect to the host or repair saved records.

The parser validates section/block/trailer framing, options, IDB/EPB association,
actual radiotap link type and packet length bounds before packet access.
It supports one version-1.0 section in either byte order, up to 16 radiotap IDBs,
1 MiB blocks, 65,535-byte full packets and 256 options per block within the 8 MiB
capture bound. Source-provided timestamp units/offsets are checked without
requiring ordering or inferring an RF clock.
Supported beacon decoding checks radiotap alignment/flags and management-header,
fixed-field and information-element bounds. It emits no MAC, SSID, packet
timestamp or payload. Other frame classes do not receive invented source-address
interpretations or full payload/conformance validation; malformed supported
contents fail rather than becoming absence. Ordinary infrastructure beacons have
matching unicast transmitter/BSSID fields, broadcast destination, an unfragmented
24-byte management header with both DS bits clear, 12-byte fixed body and one
SSID element of at most 32 raw bytes. Empty/all-zero SSIDs count as hidden.
DS elements have one byte, TIM has at least four, and duplicates of selected
single-valued elements fail. RSN/vendor contents remain opaque; their counters
describe framing/presence. IBSS, Order/protected beacon variants and other frame
classes follow explicit unsupported/other counters. Radiotap TLV field 28 and
unknown standard fields fail. No full IEEE conformance or protocol authentication
is established.
IDB FCS metadata supplies a default, EPB metadata can override it, and radiotap
flags must agree with the selected interpretation. CRC error flags prevent a
validated beacon. Non-beacon DATAPAD variants leave CRC unqualified rather than
inventing a result; unsupported frame variants are counted separately.
Unqualified FCS and unsupported packet features remain explicit counts. Zero
beacons leaves physical beacon extraction unexercised. Packet/source authenticity,
calibrated timing, lossless reception, rotation and RF silence remain unqualified
by this ambient method. C6 presence requires the separate controlled witness
below. Kismet's nonblocking backlog can discard records before collection.

## Controlled AP presence witness

The [AP method](../experiments/wifi-ap.md) owns radio/deployment/restoration gates.
The independent `firmware-wifi` workspace pins the same Rust 1.96.0/ESP-IDF 5.5.6
target integration and sys/HAL/service Git revision. It adds a real Wi-Fi driver;
the older table-only firmware remains radio-uninitialized. Shared `wifi-protocol`
is `no_std`, allocation-free and bounded to 256-byte requests/1024-byte replies.
It validates nonzero IDs/nonces, operation correlation and coherent private
Idle/Active/Stopped snapshots. Snapshot/nonce/MAC Debug is redacted. Raw framing
is versioned JSON/LF; it is correlation, not authentication.

Start is single-use per boot and initializes lazily. The stopped AP is configured
for RAM storage/channel 1/private nonce SSID and a fixed synthetic control MAC.
Safe pinned `WifiDriver` wrappers perform MAC set/get and start/stop. A small
reviewed Rust FFI call selects the generated scalar `WIFI_STORAGE_RAM` after
initialization and before configuration; it passes no pointers or callbacks.
All authored Wi-Fi operations run on the main FreeRTOS task. Upstream event
callbacks update the wrapper's synchronized state; no authored ISR/callback
changes MAC configuration or borrows a temporary buffer across an SDK call.
Driver/loop ownership remains single; no EspNetif/DHCP, STA, BLE, IEEE or NVS
service is initialized. Compiled SDK components do not establish running services.

An independent eight-second software deadline requests automatic Stop even
without UART traffic. Failed cleanup retains its first error and retries at
one-second intervals. Scheduling, blocking SDK calls and panic/reset can exceed
the request; this is not a hard RF shutdown bound. Stop verifies inactive state,
restores the saved original driver MAC and reads it back. A dedicated checked-Stop
flag can avoid a redundant SDK Stop only after prior successful Stop/inactive
confirmation and a fresh inactive observation; MAC restoration is still checked.
Keep the initialized driver/subscription owned until reset, avoiding implicit
vendor Drop stop/deinit/unwrap behavior. Partial constructor failure is terminal
and never becomes a successful uninitialized/clean state. No deinit or SSID
restoration is claimed; the original full image is separately restored/verified.

`wifi-ap-trial` requires the qualified firmware and creates a new private trial.
It opens the verified UART, explicitly releases DTR then RTS and validates a
separate one-second startup record. Status must prove Idle/uninitialized before
receiver launch. The opt-in worker READY requires HTTP 200 and a saved framing-
valid SHB with request/worker/build/prefix bindings. Host semantic SHB validation,
local prefix synchronization and live SSH-child checking precede StartRequested
and UART Start. Pre-gate failures suppress Start; later invalidity fails the trial
and triggers cleanup. Existing passive worker/32-EPB capture behavior is retained.

The matcher reuses complete packet validation, then requires an ordinary beacon
with a checked included CRC, exact expected SSID and matching TA/BSSID. It emits
only `WitnessSummary` counters. Missing/uncheckable CRC cannot witness, and a
good earlier match cannot hide malformed later data. Raw nonce/SSID/address
arguments stay private. `validate_section_prefix` accepts exactly one complete
version-1.0 stream SHB with unknown section length and bounded options, not packets.

Capture attempts Stop after possible Start, including lost ACK/error cases.
Success additionally requires Stopped/inactive, retained initialization, and
Stop original/current MAC equal to the original saved by Start. UART is drained
through receiver completion plus a 200-ms suffix check; noise, stale responses,
partial frames and failed cleanup remain failed. Report fields contain only
receiver/witness counts and fixed success states. The exact root inventory is
12 regular files plus the legacy seven-file `receiver/` capture. Replay checks
all inventories/hashes, raw/transcript/receipt agreement, readiness, causal event
order, Stop, nonzero witness and saved-summary equality without opening devices.
Recorded host elapsed times are monotonic checks only; the live eight-second
wait is source-enforced, not a replay duration threshold or RF interval claim.

A prefix may finish before the AP lifetime ends. Presence is the only on-air
claim: no full-window/startup/Stop coverage, silence, rotation, source authenticity
or correlation reduction follows. Full original-flash verification is mandatory
outside the capture module; a successful trial does not establish board recovery.

## AP address change across restart

The [restart method](../experiments/wifi-restart.md) governs the second
`device-identity-wifi-restart` binary in the same independent target workspace.
The existing version-one wire types and single-start behavior stay separate.
`wifi_protocol::restart` is version two, `no_std` and allocation-free with the
same 256/1024-byte request/reply bounds. Explicit First/Second phases, increasing
nonzero IDs and one nonce correlate a boot-local sequence. Private snapshots
include phase, once-saved original/current MACs, initialized/running state and
the sticky first nonzero failure; Debug redacts private values.

The main FreeRTOS task retains one lazily initialized `WifiDriver`/event loop.
It selects RAM storage and configures SSID/channel once. Each of exactly two
attempts changes only the disabled AP MAC, checks readback before/after start
and invalidates prior checked-stop evidence immediately before the SDK start.
Both controls differ from the once-saved original. An explicit successful StopA
with inactive state/original restoration authorizes B; automatic Stop or Status
alone cannot. Each attempt has a fresh independent eight-second requested
deadline and failed-cleanup retry. No hard real-time/RF-duration guarantee follows.

Every protocol, transport, adapter or failed automatic-cleanup operation latches
the first error and permanently inhibits later starts. Status/Stop may succeed
with checked cleanup and that retained failure; trial admission must still fail.
No second start can reuse a recovered failed phase. The initialized driver stays
owned until reset; no implicit vendor Drop/deinit, SSID restoration, network
service or persistent setter is added. The scalar RAM-storage FFI contract is
shared unchanged with the single-start binary.

`wifi-ap-restart-trial` opens/releases UART once and retains that handle. It
requires five correlated normal commands and two independent receiver captures.
Each Start uses bound READY, locally validated saved SHB and live-child/cancel
gates. A must have checked Stop, original-MAC continuity, complete receiver replay
and a CRC-verified exact target witness before B's receiver is created. UART is
monitored for unexpected/reset output during readiness waits. Failure after a
possible Start attempts recorded Stop and leaves the whole session failed.
Receiver/startup/readiness helpers are shared with the version-one collector;
its existing wire/admission/output behavior is preserved.

The exact version-two success inventory is 13 regular root files plus two
seven-file receiver directories. Replay checks inventories/hashes/receipts,
startup/raw/transcript/event order, both readiness bindings, all five exchanges,
phase/nonce/original continuity, sticky-error absence, both witnesses and exact
saved-summary bytes. Raw UART remains capped at 64 KiB, exchange timeouts at
five seconds and receiver bounds apply independently per phase. Saved elapsed
values check ordering, not an eight-second threshold. One live handle/controller
sequence is software evidence, not independent boot attestation.

The restart collector's private clock uses real monotonic `Instant` in production.
Test-only manual time advances on modeled transport waits/empty reads, so
fragmented successful fixtures do not depend on concurrent filesystem speed.
Separate controls exercise expiry and real-clock preemption; failure cases
assert that the intended injected fault was reached. Response waits check expiry
before and after retaining each received byte, rejecting a known-late final LF.
Simulated elapsed values do not qualify physical UART/RF timing; production
exchange limits remain five seconds.

Presence is required per phase; collateral comparison is optional after both.
Only `Error::ProfileCapture` maps to the fixed Unqualified outcome when entire
captures are not checked targets; other comparison errors fail the session.
Qualified contains the exact profile report with equal expected SSIDs and
different expected MACs, without payload values or hashes. No favorable packet
subset is selected. The same SSID deliberately links phases. Transition traffic,
silence, hot changes, cold boot, client reconnect, uniqueness and correlation
reduction remain outside the presence claim. Full-image recovery remains a
separate required host boundary.

## Retrospective beacon field comparison

The [comparison method](../experiments/wifi-beacon-profiles.md) defines an offline
analysis of complete admitted AP trials. `wifi-ap-compare-profiles` replays both
19-file bundles before loading their bound private targets and packet bytes.
Direct raw PCAPNG input cannot establish that bundle admission. No serial,
receiver, radio or reset operation is invoked.

`wifi-observer` borrows each capture and target through a validated `BeaconInput`
whose Debug representation redacts all private bytes. The comparator reuses the
whole container/radiotap/FCS/beacon validator. Every packet must be a supported
CRC-verified exact-target beacon; wrong targets, mixed frames, unchecked FCS or
late malformed records fail instead of selecting a subset. Existing summary and
presence-witness consumers retain their own acceptance behavior.

Exact private encodings retain interval and capability bytes, the full ordered
non-SSID IE TLV stream, ordered vendor TLVs and repeated ordered occurrences per
element ID. Unknown contents stay opaque. SSID TLVs alone are removed from the IE
comparison; metadata, management headers, TSF and FCS are outside these specific
comparisons. Per-ID absence has its own empty encoding. Output contains distinct
encoding/intersection counts, presence counts, element IDs and equality states;
no raw values, payload hashes or profile labels escape. Equality across all
beacons requires one encoding in the union, not equality of only the first pair.
Different valid encodings remain a successful comparison result.

Private exact-byte sets allocate on the host and are bounded by the admitted
8 MiB input limit per side; they are released after the call. This is offline
host work with no hard real-time or allocation-free claim. Allocation failure
follows standard host allocation behavior. Static errors contain no targets or
input paths. The retrospective result cannot establish population uniqueness,
tracking success, address rotation or correlation reduction.

## Table-probe firmware execution and FFI

The radio-uninitialized table-probe firmware runs in the FreeRTOS main task. It initializes UART0 and
uses bounded command/response storage: 128-byte requests and 2048-byte responses.
The shared protocol is `no_std`, with no allocator. Firmware `std` and SDK/HAL
startup may allocate; no timing or heap-free firmware claim is made. UART I/O
can block or fail; partial writes are handled, and panics abort/reboot.

MAC-table FFI is isolated in `firmware/src/mac.rs`, with generated SDK bindings.
Each local array outlives its synchronous foreign call and has the SDK-required
size: eight bytes for direct factory-default and IEEE reads, six for the other
getters/setters. Immutable setter buffers are copied by the SDK. No pointer is
retained, no callback or shared hardware handle is exposed, and no firmware
thread or radio service concurrently accesses the SDK table. The direct C6
factory-default function can write eight bytes regardless of the factory-table
getter's six-byte length. Ordinary table getters are distinct calls.

The baseline getters populate interface caches. A later base-only setter changes
BASE, preserving cached interface entries. Explicit setters then update four
entries. Restoring attempts all saved entries even if one setter fails; failure
never becomes a complete successful snapshot. No eFuse, flash, NVS, pairing or
bridge-configuration write belongs to the application. The application starts
no radio driver or network service; this does not prove RF silence or qualify
the behavior of another firmware image. Hard reset clears these software-table
changes; full-flash restore/verification is a separate host procedure.

## API guideline decisions

The current CLI supports the inventory and bounded runtime/AP methods documented
above; it does not replay the historical continuous-window format. C-STABLE/S1
remains an internal 0.1 exception. C-CRATE-DOC, C-LINK and C-METADATA apply to
the workspace and tool documentation. C-CUSTOM-TYPE and C-VALIDATE apply to the BLE
operation gate: the existing Operation enum selects a start, with exhaustive
controls rejecting Status/Stop before SDK access. C-EXAMPLE and C-FAILURE apply
to its documented fallible example and fixed error. The remaining decisions
below cover allocation, unsafe, FFI layout, callbacks, deadlines, wire format and
cleanup. Host checks do not transfer historical on-board qualification to a
newly built image.

The [54-entry inventory](../../../docs/rust-api-checklist.md) and pinned
explanatory chapters govern these decisions. The protocol library is internal;
its wire records are explicitly versioned.

| Entry | Decision | Evidence or reason |
| --- | --- | --- |
| C-CASE | Applies | Rust casing and explicit snake_case report enums. |
| C-CONV | Applies | BLE readback uses explicit From/TryFrom for exact bounded wire representation; SDK conversion details remain private. |
| C-GETTER | Applies | `Domain::ids` and framer accessors omit a get prefix; boolean state names describe predicates. |
| C-ITER | Not applicable | No authored collection iteration API. |
| C-ITER-TY | Applies | The private `InformationElements` TLV iterator is named for its contents; tuple construction introduces no iterator-returning collection method. |
| C-FEATURE | Applies | The BLE firmware feature selects only its target binary; host protocol/controller checks remain available without SDK initialization. |
| C-WORD-ORDER | Applies | `usb-summary` and `efuse-summary` share noun/operation order. |
| C-COMMON-TRAITS | Applies | Reports/enums have Debug, Clone and equality; value types have Copy/Hash where meaningful. Errors have Display. Raw USB/protocol/worker and borrowed beacon-target Debug redact private contents; loader/profile metadata and borrowed persistence labels omit value-revealing formatting. Live native references, process ownership and incremental framer state are not duplicated. |
| C-CONV-TRAITS | Applies | BLE address readback uses checked TryFrom conversions for the bounded wire representation. |
| C-COLLECT | Not applicable | No authored collection. |
| C-SERDE | Applies | Typed input/receipt deserialization and report serialization/deserialization; runtime and both Wi-Fi protocol namespaces use bounded serde-json-core interchange. The bounded std-only worker uses an explicit versioned request/status protocol instead of a serialization dependency. |
| C-SEND-SYNC | Applies | Owned reports use automatic traits. Capture shares child/gate state through Mutex/Arc and cancellation atomics with explicit ordering; the AP has one configuration owner and upstream synchronized event state. No authored unsafe Send/Sync implementation. |
| C-GOOD-ERR | Applies; exception E1 | Both protocols, USB, Wi-Fi/framing and worker errors implement Display plus core/std Error with automatic Send/Sync. CLI diagnostics are static; SDK statuses remain in private records. Public errors contain no packet, nonce or credential values. |
| C-NUM-FMT | Not applicable | No numeric or bitmask wrapper. |
| C-RW-VALUE | Applies | Generic host stream operations take the stream by value, allowing owned streams or mutable references for persistent live/fake sessions. Protocol encoding borrows caller storage; summary loader borrows a path. |
| C-EVOCATIVE | Not applicable | No authored macro. |
| C-MACRO-ATTR | Not applicable | No item-producing macro. |
| C-ANYWHERE | Not applicable | No authored macro. |
| C-MACRO-VIS | Not applicable | No authored macro. |
| C-MACRO-TY | Not applicable | No authored macro. |
| C-CRATE-DOC | Applies | Protocol/observer/worker crate docs explain execution, private interchange and bounded parsing; binary docs state command effects. |
| C-EXAMPLE | Applies | Protocol/observer doctests cover records, framing and parsing; the hardware-only USB example is no_run and names SDK/devices. Worker/tool/firmware READMEs define actual prerequisites. |
| C-QUESTION-MARK | Applies | Fallible protocol doctest uses question-mark propagation. |
| C-FAILURE | Applies | Input/output, allocation, unsupported platform/link/frame subsets, transfer, deadlines, process cleanup, selection and parsing limits are documented here and on exported operations. |
| C-LINK | Applies | Procedure, results, API record and tool README link related contracts. |
| C-METADATA | Exception M1 | Accurate package name, description, compiler, edition and publish=false; no invented license, authors or distribution URL. |
| C-RELNOTES | Not applicable | No published release; meaningful changes and checks are recorded in Git. |
| C-HIDDEN | Applies | Protocol exports documented interchange types/controls; host and FFI implementation remain private. |
| C-SMART-PTR | Not applicable | No authored smart pointer. |
| C-CONV-SPECIFIC | Applies | BLE readback conversions stay with specific local readback/wire types. |
| C-METHOD | Applies | Counter construction/recording are inherent operations; parsers operate on input bytes. |
| C-NO-OUT | Applies | Parsing returns a complete value or error; bounded protocol encoding intentionally writes to caller-owned storage and returns used length. |
| C-OVERLOAD | Not applicable | No operator implementation. |
| C-DEREF | Not applicable | No Deref implementation. |
| C-CTOR | Applies | Request/Response constructors initialize the supported version; counter constructors initialize the selected domain; `BlockFramer::new` validates its bound; Beacon target constructors validate their private inputs. Shape validation still precedes admission. |
| C-INTERMEDIATE | Applies | Capture and derived output remain separate; raw failures stay private. |
| C-CALLER-CONTROL | Applies | Parser borrows input bytes; caller owns protocol output buffers and capture destination. |
| C-GENERIC | Applies | Serde helpers use Deserializer bounds; no needless public abstraction. |
| C-OBJECT | Applies | The internal AP Radio trait is object-safe; its controller uses static dispatch for the SDK adapter and host fakes. The private object-safe BeaconConsumer hook shares validated frames with witness/profile consumers. No live peripheral trait object escapes. BLE Radio is an intentionally extensible object-safe seam for its target adapter and independent fakes. |
| C-NEWTYPE | Applies | Separate domain, observation and content-state enums prevent mixed meanings. |
| C-CUSTOM-TYPE | Applies | CLI operations and report states use enums; readability booleans are explicitly named observations. Restart Phase/Operation distinguish ordered attempts; the collateral outcome separates Qualified from fixed Unqualified. `BeaconInput` groups one bounded capture and its private expected identity; typed comparison reports distinguish per-field and per-element counts. |
| C-BITFLAG | Not applicable | No combinable flags. |
| C-BUILDER | Not applicable | No complex object construction contract. |
| C-VALIDATE | Applies | Schemas, widths/caps, private paths, wire transitions, USB request plans, UTF-16, native cleanup, worker/request identities, transport outcomes, PCAPNG/radiotap/beacon bounds and receipt hashes are checked. Saved audits additionally validate complete USB replay, eFuse correspondence, exact CP checksum layout, table MD5 and physical NVS integrity/partition-local namespaces; rehashed semantic controls and late-partition failures reject. Restart also checks increasing IDs, phase authorization, one saved original, sticky failure and two independent READY/witness/Stop admissions. Partial/failed captures never become absence observations. |
| C-DTOR-FAIL | Applies | Native handles explicitly release/check errors; child processes explicitly stop/reap. AP cleanup explicitly stops/checks/restores, retaining the owned driver until reset instead of invoking the vendor's stop/deinit/unwrap Drop path. Fallback native Drop never panics. CF ownership is documented; test cleanup stays in test paths. |
| C-DTOR-BLOCK | Applies | Native references and worker/SSH children explicitly finish before return. OS release/wait calls and filesystem cleanup may block; no hard real-time claim. |
| C-DEBUG | Applies | Report Debug contains sanitized counts/states. Public raw transfer/record, borrowed beacon input and worker request/status Debug redact private values; native handles and loader/profile metadata expose no arbitrary input formatting. |
| C-DEBUG-NONEMPTY | Applies | Sanitized report variants and fields remain visible. |
| C-SEALED | Not applicable | The AP Radio trait is private; BLE Radio intentionally permits target/fake adapters rather than a closed implementation set. |
| C-STRUCT-PRIVATE | Applies | Host/native handles, borrowed beacon-input fields and framer internals are private. Public protocol/observer/worker records are passive interchange data, validated before admission. |
| C-NEWTYPE-HIDE | Applies | The worker's typed static error hides its representation; callers use Display/Error rather than its stored message. |
| C-STRUCT-BOUNDS | Applies | The private generic AP controller stores its adapter without unnecessary struct-level bounds; Radio bounds apply to operations. |
| C-STABLE | Exception S1 | Internal 0.1 protocol, explicitly versioned frames, no stable public release. Firmware requires scoped build-std bootstrap; tested compiler is pinned. |
| C-PERMISSIVE | Exception L1 | Project licensing is undecided and publish=false. Serialport is MPL-2.0; review its distribution constraints before release. SDK/vendor components retain their upstream terms. |

M1/L1/S1 apply to these internal packages only. Revisit metadata, licensing and
compiler/API commitments before distribution or a public stable library. Synthetic
integration/unit controls exercise the real CLI/parser, malformed input,
redaction boundaries, size/path failures and eFuse byte-order cases. They do
not substitute for attached-host or on-board observations.
