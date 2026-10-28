# BLE address override observed through HCI reports

Decision question: does the existing controlled BLE image produce independent
receiver-controller advertising reports for its fixed static-random control A,
then control B after a checked Stop, with the exact same fresh session nonce?
This is a bounded address-and-payload presence check, not a recorder application.

The original application's public-address HCI reports are an already explored
baseline. The controlled A/B observations defined here are prospective. Reuse the
existing image, stock pySerial, BlueZ `bluetoothctl`/`btmon` and TShark; introduce
no firmware change, maintained collector or new packet parser. Earlier BLE
software controls cover the firmware protocol/controller, not this HCI capture
and one-off save/replay path. Qualify that actual path below before collection.

## Fixed inputs and scope

Use ESP32-C6-DevKitC-1-N8 and the existing
`device-identity-ble-firmware` application identified by
[software-inputs.json](../firmware-ble/software-inputs.json): SHA-256
`59740f6165a3e3e97ecdcca7aecc335d62b1deb558fb290b8bd1eaa3ff71fd34`,
871824 bytes, at offset 65536, with a 872448-byte sector-rounded application
extent. This is the previously built image; do not replace it with a rebuild or
the later pure-library extraction's image. Preserve the exact existing ELF,
generated configuration, segments, SDK/tool identities and private deployment
inputs. The older generated three-MiB factory partition size does not authorize
enlarging the original executable slot.

The authoritative wire and radio inputs for this image are
[protocol.rs](../firmware-ble/src/protocol.rs),
[controller.rs](../firmware-ble/src/controller.rs),
[radio.rs](../firmware-ble/src/radio.rs),
[sdk.rs](../firmware-ble/src/sdk.rs) and
[main.rs](../firmware-ble/src/main.rs). UART0 is GPIO16 TX/GPIO17 RX through the
bridge, 115200 baud, 8N1. This probe uses UART0 rather than the original
application's native USB console. Retain one target UART handle across both
phases and the interphase review. Open/close and OS modem-line effects can reset
the target; do not describe the handle as independent reset or boot attestation.

Use the existing Intel HCI0 receiver and BlueZ 5.84, preserving Nordic serial
ownership, Kismet services and Wi-Fi settings. Record actual installed BlueZ,
TShark and pySerial versions. Reuse the existing discovery activity owned by
Kismet's direct-management-socket client. The earlier stock BlueZ discovery
attempt failed Busy; its failed attempt remains preserved, and the original
application's baseline reports were subsequently captured passively. Do not
issue discovery start/stop/filter commands or attempt to take over that scan.
Record Powered and Discovering before and after without changing them; a false
BlueZ Discovering property does not establish absence of another client's
direct-management discovery activity or RF scanning.

`btmon` observes the receiver's existing HCI reports, not raw over-the-air PDUs.
The existing scan may have radio effects; monitoring it does not establish that
the receiver is RF-passive. Do not issue Scan, Pair, Connect, Trust, Remove,
Power, receiver reset, Nordic TTY, source-control or Wi-Fi configuration commands.
Raw reports, addresses, nonce,
port paths, flash contents/digests and ambient payloads remain private.

## Preserve, deploy and unconditionally restore

Apply the existing [full-flash recovery gates](wifi-ap.md#preserve-and-restore-the-original-device).
Require one intended bridge, no competing target owner, privately matched saved
factory identity, disabled security/encryption for the selected method, and a
fresh 8,388,608-byte backup followed by a separate whole-range device-MD5
verification before any write. No application boot may intervene between that
backup/verification and deployment. Earlier snapshots and successful recoveries
do not replace this fresh gate.

Inspect the fresh original partition table. Verify exact application length and
hash again before writing; its sector-rounded erase footprint must fit the
original executable allowance and avoid all stored-data/key partitions. Write
only the approved application at `0x10000`, retaining the original bootloader
and partition table. No merged-image padding, force, erase-all, eFuse or bridge
programming is permitted. Every subprocess has an outer deadline; preserve
nonzero results, warnings and partial outputs.

After **any write attempt**, restore the fresh complete 8 MiB image on success,
failure, exception or timeout. Recheck its length/hash and intended target before
each restore write. Separately verify the complete restored range before normal
reset/release. Allow at most two identical restore attempts and retain both if
needed. A failed restore/verification remains recovery-required; do not release
an unverified partial image as success. Record reset warnings and require the
selected warning policy. Device-MD5 agreement is not a second full-image readback,
proof of unchanged later persistence or independent boot attestation.

## Six-command schedule and host admission

Choose one fresh nonzero eight-byte nonce. Freeze its private input before the
trial. Requests contain exactly `domain`, `version`, `id`, `operation`, `nonce`:
domain `ble_advertising`, version 1, IDs 1 through 6, and an eight-element byte
array. Send one complete JSON frame plus LF at a time, at most 256 bytes. Retain
every attempted/transferred byte and every response/partial response; bound each
exchange to five seconds and total UART storage to 64 KiB. A response frame is at
most 1024 bytes. Reject duplicate/unknown JSON keys, trailing data, out-of-range
scalars, booleans used as integers, unexpected extra frames and late/reset noise.
Retain the separately bounded startup prefix rather than silently discarding it.

| ID | Operation | Required snapshot |
| --- | --- | --- |
| 1 | `status` | Idle; phase/nonce absent; uninitialized; not advertising; configuration/setter false; both readbacks not requested |
| 2 | `start_first` | Active First; same nonce; initialized/advertising/configured/setter-completed true; active used address exactly random A |
| 3 | `stop` | Stopped First; same nonce; initialized/configured/setter-completed true; advertising false |
| 4 | `status` | Same checked Stopped First state after the interphase receiver review |
| 5 | `start_second` | Active Second; same nonce; initialized/advertising/configured/setter-completed true; active used address exactly random B |
| 6 | `stop` | Stopped Second; same nonce; initialized/configured/setter-completed true; advertising false |

Every response has exactly `domain`, `version`, `id`, `operation`, `status`,
`snapshot`. Require exact request correlation, zero operation status, a snapshot,
`first_failure=null` and `auto_stopped=false` on all six normal responses.
Protocol `validate_for` alone does not reject a latched failure on Status/Stop;
these stricter primary-cohort checks are mandatory.

Snapshots contain exactly eleven fields: `state`, `phase`, `nonce`,
`initialized`, `advertising`, `data_configured`, `address_set_completed`,
`prestart_readback`, `used_readback`, `auto_stopped`, `first_failure`.
Each readback has `availability`, `address`, `address_type`, `status`:

- `not_requested`: all three other fields null.
- `available`: six byte address, `public` or `random` type, status zero.
- `unavailable`: address/type null and nonzero signed 32-bit status.

Active used readback must be available and equal the phase's fixed random control
from `protocol.rs`. Prestart readback must have been requested but may return a
public/previous address or be unavailable. Stopped queries must have been
requested; preserve their actual availability rather than inventing an address.
Stop retains initialized Bluetooth and random RAM state; it does not restore the
public address, deinitialize or prove RF silence.

For each phase, wait a planned four seconds after accepting its active response,
then request Stop. Firmware's independent eight-second software deadline starts
before setup, not at the response. Per-operation completion waits are bounded to
one second, firmware response writes to two seconds, and failed cleanup requests
retry at one-second software intervals. SDK blocking and scheduling prevent a
hard real-time/on-air duration guarantee. Automatic Stop emits no unsolicited
UART response and does not by itself authorize B.

Any primary failure is sticky for the trial. Preserve it, send only bounded Stop
cleanup with a fresh increasing ID when the channel remains usable, and perform
full recovery. Lost ACK, partial write, malformed/late response or receiver loss
never authorizes another Start. A later successful cleanup cannot turn the cohort
into a pass. No retry of a Start, third phase or new nonce is permitted in this boot.

## Two receiver windows and A-before-B gate

Use **two separate closed btsnoop files**, one per phase, to avoid interpreting a
still-open buffered file as complete evidence. Each passive `btmon --index 0
--write ...` process has a 180-second maximum window and an explicit bounded
termination/child-reap allowance. Finish each early after its phase Stop. Start
the monitor before the target Start, without any scan setup; record readiness
and command timestamps on their actual clocks. Preserve all monitor and
transport results. Do not compare uncalibrated remote
HCI timestamps with host monotonic times as precise RF timing.

After A Stop, send INT only to that window's owned `btmon` process, reap it,
retain its cleanup/poststate, and transfer the entire saved file/status.
Keep the target UART handle open, draining/bounding and rejecting unexpected
output while reviewing A. Fully decode A and require its witness below. Only
after successful A replay, request/accept ID4 Status, prepare a fresh ready B
receiver window and send ID5. Failure or an exhausted 180-second bound ends the
cohort without B. B repeats the same capture/Stop/finish/replay process.

## HCI witness and saved replay

Use the installed stock TShark or `btmon --read` on the saved btsnoop files. When
TShark JSON is used, preserve repeated AD entries with `--no-duplicate-keys`;
ordinary last-key-wins JSON parsing can discard service/name fields. Validate
complete capture framing, decoder outcome and retained capture/cleanup statuses;
do not admit a favorable prefix from a failed capture. A normal timed end may
have the predeclared timeout exit status, distinct from premature process failure.

The qualified composition admits exactly one report per HCI advertising event.
Zero or multiple reports reject the complete capture. This strict profile avoids
silently skipping an unsupported late event after an otherwise valid witness.

For each phase require at least one complete advertising report with the exact
phase control, random address type, legacy LE 1M and complete data. Admit either
the legacy LE Advertising Report or a legacy-marked LE Extended Advertising
Report. Require nonconnectable/nonscannable advertising, not a scan response or
connection report. The complete reported advertising data must equal the exact
26-byte `nonce_ad(nonce)` from the committed protocol: length 25, AD type `0x21`,
the committed 16-byte little-endian service UUID and the eight nonce bytes.
Retain all unrelated/mismatching reports and their counts, not only matches.

Replay must reread the immutable raw UART and btsnoop files, reproduce all six
ordered checks and both witnesses, and compare the resulting fixed counts/states
with the saved derivative. Check capture completion, A admission before B, sticky
failure and verified restoration. The earlier Nordic trial replay expects five
commands plus Kismet/DLT256/CRC evidence and does not admit this HCI method.

## Qualification and freeze

Before physical collection, exercise the actual one-off validation, UART
read/write and gate decisions using fake inputs where meaningful, and the stock
HCI decode path using synthetic HCI records. Reuse the qualified existing
backup/recovery and upstream-tool controls; do not create a maintained collector
or another framework. A separate reviewer must construct contrary inputs and
reproduce outcomes. Retain commands, exact script/tool/source identities, all
attempts and static results. At minimum qualify:

| Control | Required outcome |
| --- | --- |
| Complete six-command A/B transcript and both exact HCI witnesses, including available and permitted unavailable prestart/stopped readbacks | Pass and identical result after saved replay |
| Wrong domain/version/ID/operation/nonce; reordered/repeated phase; extra or duplicate field; invalid scalar/type | Reject, preserve offending input, no B |
| Active address/type mismatch or unavailable active used readback; nonzero operation status | Reject, bounded cleanup only |
| Status/Stop with latched failure or automatic-stop flag despite operation status zero | Reject primary cohort |
| Partial/oversize/non-LF response, short write, lost ACK, reply at/after deadline, late UART/reset noise | Preserve partial bytes/failure and prevent later Start |
| Missing receiver readiness, early monitor failure, timeout before complete phase, failed owned-monitor cleanup | Reject, retain failed capture, no B |
| Wrong HCI address/type/nonce/UUID, truncated/fragmented AD, wrong report kind or incomplete data; late malformed record | No witness/reject as appropriate; no favorable-prefix pass |
| Mutated saved transcript, report/status disagreement, omitted/raw file, failed or incomplete flash recovery | Replay rejects; success cannot be reconstructed from a partial archive |

Controls must hit the actual changed condition; malformed fixtures must not fail
only an unrelated earlier guard. Freeze the prospective script, this method,
input/artifact/tool digests and its commit before deployment. Keep a collecting
state until complete recovery and saved replay. Seal every retained regular file
except the manifest itself using relative paths, preserving failed cohorts.

Publish only fixed counts, exact-match relationships, software states and recovery
outcomes. The result can establish scoped A/B address-and-nonce HCI presence,
not raw RF CRC, complete delivery, startup/default-address absence, silence,
controller shutdown, cold-power behavior, authenticated execution, correlation
reduction or population uniqueness. The shared nonce deliberately links A and B.
