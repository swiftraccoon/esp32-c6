# identityctl

Capture controlled runtime probes and summarize private identifier records
without publishing identifier values. Summary/comparison/replay commands open
regular saved files only. `runtime-capture` opens the selected UART bridge and
requests reversible firmware MAC-table changes, saving raw values privately.
It explicitly deasserts DTR then RTS, which may release reset or start the
application. It performs no flashing or eFuse programming.
`usb-control-capture` instead submits a fixed read-only USB control plan without
opening a serial port, releasing control lines or configuring a device.
`wifi-capture` uses a prequalified remote Rust worker to read a bounded stream
from an existing Kismet Wi-Fi source through trusted SSH. It does not retune or
control that source. Packets, receiver selection and diagnostics remain private.

The host workspace pins Rust 1.96.0 with rustfmt and Clippy, Rust edition 2024
and formatting edition 2024. The initial build target is
`aarch64-apple-darwin`; offline parsing also builds on ordinary Rust host targets.
No ESP-IDF SDK or embedded target is required. Dependency versions and the
project lockfile pin the parser and command-line inputs. This internal tool is
unpublished; project licensing must be selected before distribution.
On macOS, building the native USB observer requires SDK 27.0 and libclang
21.1.8, with bindgen 0.72.1 pinned in the workspace lock. Set
`LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib` for that build. Portable saved-record
replay on Linux does not compile the macOS backend.

Build from the project directory:

```sh
cargo +1.96.0 build --locked -p identityctl
```

Summarize from the workspace root:

```sh
projects/device-identity/target/debug/identityctl usb-summary --input private/device-identity/RUN/usb-host.json
projects/device-identity/target/debug/identityctl efuse-summary --input private/device-identity/RUN/efuse.json
projects/device-identity/target/debug/identityctl usb-compare --before private/device-identity/RUN/usb-before.json --after private/device-identity/RUN/usb-after.json
```

Qualify an existing remote Wi-Fi source using the
[packet method](../../experiments/wifi-observer.md) and
[remote worker](../kismet-worker/README.md):

```sh
projects/device-identity/target/debug/identityctl wifi-capture --profile private/device-identity/RUN/receiver-profile.json --output private/device-identity/RUN/capture-01
projects/device-identity/target/debug/identityctl wifi-replay --capture private/device-identity/RUN/capture-01
projects/device-identity/target/debug/identityctl wifi-summary --input private/device-identity/RUN/capture-01/packets.pcapng
```

The private profile selects a trusted SSH alias, approved remote worker/source,
expected running-process and credential-file metadata, and worker provenance.
It contains no credential values. Resolve these inputs using read-only receiver
inventory; profiles are not supplied by untrusted packet streams. Capture stops
at 32 complete packet records or fails on its time/byte limits. Seven saved files
and a successful receipt are required for replay. Counts do not establish target
ESP32 reception, RF timing, silence or lossless capture.

Capture macOS USB data with `system_profiler SPUSBHostDataType -json`, keeping
both output and diagnostics in ignored private storage. Successful collection
and the capture's source provenance must be recorded before interpreting the
summary. The obsolete `SPUSBDataType` schema is deliberately rejected; an empty
object is not evidence that devices are absent.

The USB summary counts exact `303a:1001` native Serial/JTAG and `10c4:ea60`
CP2102N matches in the enumeration. Serial fields are classified as present,
empty (including whitespace only) or absent. Repeated VID/PID matches remain
separate counts: enumeration alone does not establish that they are one board.
Product names, serial values, topology, location IDs and unrelated properties
are never copied to the result. A present serial field does not establish
uniqueness, stability, provenance from eFuse or bridge configuration lock state.

`usb-compare` compares the actual serial-property strings for each domain when
both inputs have exactly one matching device and a nonempty serial property.
It emits `unchanged` or `changed`, with counts and no value. Missing/empty
properties yield `unavailable`; repeated matches yield `ambiguous`, because
VID/PID alone cannot pair boards. These are current host properties, which may
be cached. The comparison does not establish fresh USB descriptor transfers.

The eFuse summary accepts only the selected `MAC`, `MAC_EXT` and
`OPTIONAL_UNIQUE_ID` JSON fields emitted by the pinned espefuse 4.12.0
implementation. It validates names, widths, source positions, byte encodings
and agreement between raw and formatted readable values. Both MAC fields use
colon-separated bytes and the upstream successful integrity annotation;
their byte order is reversed relative to the JSON raw value. Other eFuse
coding warnings can remain in capture diagnostics; this parser cannot qualify
every block's integrity. Readable bytes become only `all_zero` or `nonzero`;
protected fields become `unknown`, including when their raw representation is
zero. Readable nonzero bits do not establish uniqueness or provisioning purpose.

The [capture procedure](../../experiments/read-only-inventory.md) specifies the
read-only whitelist command, device restart effects and preservation of failed
captures. The [API and safety record](../../docs/api.md) defines implementation
contracts and guideline applicability.

## Saved identity audits

These commands open saved regular files only. Follow the
[saved-audit method](../../experiments/saved-identity-audits.md) and preserve
complete original inputs; do not substitute table summaries for full replay.

```sh
projects/device-identity/target/debug/identityctl usb-factory-relation --capture private/device-identity/RUN/usb-rom --efuse-input private/device-identity/RUN/efuse.json
projects/device-identity/target/debug/identityctl bridge-config-audit --capture private/device-identity/RUN/usb-rom
projects/device-identity/target/debug/identityctl flash-metadata-audit --input backups/device-identity/RUN/flash-original.bin
```

USB relation/checksum audits first replay all four capture files. They report
equality/layout/integrity, without serial/configuration contents or a bridge
lock claim. The flash audit validates table and physical NVS integrity, reporting
aggregate counts and fixed name heuristics. It does not reconstruct blobs or
establish credential absence.

## Runtime capture and replay

Use the matching [runtime firmware](../../firmware/README.md), with its UART0
bridge at 115200 baud. The caller must verify the intended bridge and preserve
the board's current flash before installing probe firmware. The probe operates
on SDK software address tables; it does not initialize radio services or
measure transmitted addresses. Follow the project's prospective runtime method
and restoration procedure before on-board collection.

Run from the repository root. The output parent must already exist; each output
directory must be new, ignored by Git, under `private/device-identity/`, and have
no symlink or parent-traversal components. Production capture creates directory
mode 0700 and files mode 0600 on Unix.

```sh
projects/device-identity/target/debug/identityctl runtime-capture --port "$serial_port" --output private/device-identity/RUN/capture-01
projects/device-identity/target/debug/identityctl runtime-replay --capture private/device-identity/RUN/capture-01
projects/device-identity/target/debug/identityctl runtime-summary --input private/device-identity/RUN/capture-01/exchanges.jsonl
projects/device-identity/target/debug/identityctl startup-summary --input private/device-identity/RUN/capture-01/startup.raw
```

Live capture requires macOS's fixed `system_profiler` observer. It selects a
nonzero OS-random correlation token, sends four increasing request IDs in the
defaults/base-override/interface-override/restored order, and checks exact reply
correlation. The token is not a device identity or authentication mechanism.
USB host properties are collected during defaults and explicit interface
overrides. All successful phase pairs, intended requests, actual bytes accepted
by serial writes and bytes returned by reads stay in the private directory.

Capture format revision 2 explicitly requests DTR false on open, then checks
separate DTR-false and RTS-false calls in that order. A CP2102N tty open can leave
RTS asserted and hold this board in reset; merely preserving library DTR state
does not release RTS. These transitions may boot/reset the C6, so the method
does not preserve a prior application's RAM state. `connection.json` records
the policy and success of each checked operation.

Before sending `Defaults`, the collector saves a one-second startup window in
`startup.raw`, capped at 8192 bytes and 256 bytes per line. Startup accepts only
empty/whitespace records or one complete C6 ROM boot block with validated header,
build date, reset/boot labels, SPI settings, load entries and final entry point.
Every meaningful line requires LF (optional CR before LF). Truncation, repeated
boots, arbitrary text and case-insensitive panic/abort/assert/error/failed markers
fail before any runtime request is sent. `startup-summary` applies the same
read-only grammar and reports counts/presence only. It does not authenticate the
origin of startup text or establish which control-line transition caused boot.

Serial operations use 100 ms polling timeouts and a five-second per-exchange
deadline. Active response lines are bounded at 2048 bytes and total active
received bytes at 64 KiB. Whitespace is accepted; every other non-JSON line
during the four phases fails capture and remains in raw storage. A 200 ms final
drain retains and rejects unexpected suffixes. Bytes before successful open,
bytes flushed by the library/OS during open, and output after the final drain
are outside this observation boundary. There is no RF-silence or hard real-time
guarantee.

An error after the runtime sequence begins attempts a separate restore request
while the port remains open. Initialization/startup errors send no phase request.
Cleanup is saved separately and cannot complete a failed normal sequence.
`cleanup_success` requires both a validated restore reply and equality with the
captured initial snapshot; without that baseline it remains false. Preserve
failed directories and follow the explicit reset/restoration procedure even
when cleanup succeeds. Opening or profiler errors return static diagnostics.

`runtime-replay` is the complete saved-capture check: it requires the exact
15-file revision-2 inventory, successful capture/profiler/control-line receipts,
the same startup grammar and exact byte counts, all four correlated
phase pairs, agreement between raw transmit/receive bytes and saved pairs,
expected table changes, restored initial values, USB schema validity and exact
saved-summary equality. Its live and replay paths share serial-artifact and
semantic validation. Keep provenance beside the capture directory; adding files
inside it invalidates the exact inventory. Replay never opens or resets devices.
Earlier capture formats are rejected rather than silently upgraded; retain
their original binary, records and failure interpretation.

`runtime-summary` checks only a four-pair JSONL table transcript, capped at
16 KiB, with a final LF. It cannot admit a live capture or inspect profiler
failure receipts. It verifies stable factory bytes, the synthetic base value,
unchanged cached interfaces after changing only BASE, all explicit interface
controls, and exact restoration. Initial values that collide with a control
are rejected so a no-op cannot count as a change. Its fixed boolean findings
establish getter consistency only; use directory replay for evidence admission.

Input is capped at 8 MiB and read with an additional byte to detect growth or a
hidden suffix. The path must have no parent components or symlink components;
directories and other non-regular files are rejected. These checks assume the
local capture directory is not concurrently replaced by another process.
Malformed JSON, metadata, duplicate recognized fields and unsupported schemas
fail without a partial report. Diagnostics use fixed messages and omit input
data and paths. Read-only parsing does not authenticate an input capture.

## USB control capture and replay

The native macOS observer selects exactly one native `303a:1001` device and one
`10c4:ea60` bridge by cached registry model IDs. It then validates those IDs
against a device descriptor returned by the fixed IN-only control plan. It uses
the IOKit device interface without `USBDeviceOpen`, seize, reset, configuration
change or driver claim. Missing/ambiguous devices and setup/cleanup errors fail;
they never trigger an arbitrary request or a fallback that detaches a driver.

The version-2 plan reads the 18-byte device descriptor and, when its serial
index is nonzero, a four-byte language-table prefix. A longer advertised table
is fetched at its exact declared even length (up to 254 bytes), retaining the
complete earlier prefix. The selected serial descriptor is read as a two-byte
header and then at its exact declared even length when longer. Repeated headers
must agree, every complete descriptor must match its declared length/type, and
serial UTF-16 is strictly validated. A failure ends that domain without a
fallback request. This explicit method change does not establish that a larger
request is invalid or guarantee a device response.
For the bridge only, it additionally reads the part code and accepts CP2102N
codes `0x20`–`0x22`, then reads 678 configuration bytes. Configuration metadata
checks only returned length and layout-version byte two; the reported byte-four
default comparison does not establish lock state. Checksum integrity and lock
state remain explicitly unqualified. All strings/configuration bytes stay private.

```sh
projects/device-identity/target/debug/identityctl usb-control-capture --output private/device-identity/RUN/usb-01
projects/device-identity/target/debug/identityctl usb-control-replay --capture private/device-identity/RUN/usb-01
projects/device-identity/target/debug/identityctl usb-control-summary --input private/device-identity/RUN/usb-01/record.json
projects/device-identity/target/debug/identityctl usb-control-compare --before private/device-identity/RUN/usb-01/record.json --after private/device-identity/RUN/usb-02/record.json
```

Capture creates a new ignored private directory using the same path/permission
policy as runtime capture. It creates the raw destination before any observer
call. A complete capture has exactly four files: `record.json`,
`native_provenance.json`, `summary.json` and `capture-status.json`. Raw records
retain every attempted transfer, native statuses, lengths and initialized data;
on a failed transfer, retained backing-buffer bytes are not asserted to be a
valid response. Selection/transfer/shape failures preserve a failed receipt and
available raw data. Failed or incomplete captures remain rejected.

Complete replay requires the successful receipt, strict native input provenance,
exact request order and descriptor shapes, successful setup/transfer/cleanup,
and saved-summary equality. Private receipt hashes also bind the exact raw,
provenance and summary bytes; these detect inconsistent saved artifacts without
authenticating hardware or authorship. Public output contains only fixed enums,
counts, presence and qualified metadata, never raw serials/configuration or their
hashes. The raw-record parser has a separate 64 KiB bound.

Raw USB records, sanitized reports and completion receipts use version 2.
Version-1 inputs are rejected rather than silently upgraded; retain their
original method, executable, records and failed interpretation.

`usb-control-summary` and `usb-control-compare` validate raw record semantics
only; they cannot admit a live capture's completion/provenance receipts. Replay
each directory first when comparing captured records. Comparison emits decoded
serial equality and bridge configuration byte equality without the contents.
These are control-request observations, not RF or anonymity measurements.

```sh
cargo +1.96.0 fmt --all -- --check
cargo +1.96.0 test --workspace --locked
cargo +1.96.0 clippy --workspace --locked --all-targets -- -D warnings
cargo +1.96.0 doc --workspace --locked --no-deps
```

## Controlled AP witness

`wifi-ap-trial` requires the qualified
[AP probe firmware](../../firmware-wifi/README.md), a private verified bridge-port
file and the opt-in READY-capable remote worker profile. Follow the
[AP method](../../experiments/wifi-ap.md), including fresh full-flash backup,
segmented deployment and complete restoration/verification. This command opens
UART and requests temporary radio operation; it does not flash or restore images.

```sh
projects/device-identity/target/debug/identityctl wifi-ap-trial --port-file private/device-identity/RUN/bridge-port.txt --profile private/device-identity/RUN/receiver-profile.json --output private/device-identity/RUN/trial-01
projects/device-identity/target/debug/identityctl wifi-ap-replay --capture private/device-identity/RUN/trial-01
```

Before Start, the controller requires Idle state, bound receiver READY, a locally
validated saved stream header and a live receiver. It then requests one pre-start
AP MAC control using a fresh private nonce SSID. Admission requires correlated
readbacks, checked Stop/MAC restoration, complete receiver replay and at least
one CRC-verified exact nonce/MAC beacon. Failed or missing cleanup stays failed
and requires the method's external reset/full-image restoration. The transmitter
and receiver deadlines do not establish full-window observation or RF silence.
The twelve root records and seven-file receiver directory remain private; replay
checks them without opening the board or reconnecting to Kismet.

## AP address change across restart

`wifi-ap-restart-trial` requires the separate version-two
[restart firmware](../../firmware-wifi/README.md#two-phase-restart-binary) and
[method](../../experiments/wifi-restart.md). It keeps one UART handle and nonce
for two ordered AP starts with distinct fixed synthetic MACs and the same SSID.
Each phase has a separate ready receiver; B is created only after A's checked
Stop/original-MAC restoration, complete replay and target witness pass. A failure
stops the sequence and triggers recorded best-effort cleanup; external full-flash
recovery remains mandatory. UART opening/releasing/closing can reset the device.

```sh
projects/device-identity/target/debug/identityctl wifi-ap-restart-trial --port-file private/device-identity/RUN/bridge-port.txt --profile private/device-identity/RUN/receiver-profile.json --output private/device-identity/RUN/trial-01
projects/device-identity/target/debug/identityctl wifi-ap-restart-replay --capture private/device-identity/RUN/trial-01
```

The private success bundle has 13 root records and two seven-file receiver
directories. Replay checks phase/order/nonce/original continuity, both witnesses,
raw bytes/receipts and saved-summary equality without opening a device or receiver.
The count-only report includes an optional collateral comparison: Qualified only
if every packet in both captures is a checked target beacon; mixed inputs yield
the fixed Unqualified state instead of selecting packets. Readbacks and observed
presence do not establish transition coverage, silence, hot rotation or privacy.

## Saved beacon field comparison

`wifi-ap-compare-profiles` reads two complete saved AP trials offline. Both
bundles must first pass full replay. Every packet must then be a CRC-verified
beacon matching its own bound target; mixed or unchecked records fail. The
command compares interval/capability bytes and ordered non-SSID/vendor/per-ID
information-element TLVs, retaining duplicate occurrences and unknown contents.
It emits counts and equality states, without raw bytes, field hashes or targets.
See the [retrospective comparison method](../../experiments/wifi-beacon-profiles.md)
for exclusions and interpretation; equality does not establish uniqueness,
tracking accuracy or behavior after address rotation.

```sh
projects/device-identity/target/debug/identityctl wifi-ap-compare-profiles --before private/device-identity/RUN/trial-01 --after private/device-identity/RUN/trial-02
```
