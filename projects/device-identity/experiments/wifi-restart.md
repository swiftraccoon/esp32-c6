# SoftAP address change across stop and restart

Decision question: can one retained ESP32-C6 AP driver emit a CRC-verified beacon
under control A, stop and restore its saved original MAC, then emit under distinct
control B in the same boot? Check driver readback and independent packet presence
at each start, followed by checked Stop/restoration. Hold the private session SSID
and other requested configuration constant while changing the address. Stop and
restart also change AP/client state; any later field difference cannot be
attributed solely to the MAC change.

The [single-start AP witness](wifi-ap.md) established pre-start override presence
in separate boots. The [retrospective field comparison](wifi-beacon-profiles.md)
found repeated non-SSID contents in those saved packets, with unchanged AP MAC.
Reuse their receiver, validator, pinned SDK, privacy rules and full-flash recovery;
the [prior-work](../../../docs/research/prior-work.md) and
[reuse catalog](../../../docs/research/benchmark-reuse.md) do not supply a
same-boot active-driver observation. A new version-two control namespace and
second firmware binary supply that bounded sequence. Version-one firmware,
capture formats and replay remain supported.

This measures an address change across a stopped/restarted AP. Full transition
coverage, early/default-address exposure, RF silence, hot changes while running,
cold power cycles, client reconnect and correlation reduction are separate gaps.
The same SSID deliberately links both phases; no privacy improvement follows.

## Fixed inputs

Use `device-identity-wifi-restart`, the second binary in the independent
`firmware-wifi` workspace, on ESP32-C6-DevKitC-1-N8. Keep Rust 1.96.0,
edition/style 2024, `riscv32imac-esp-espidf`, ldproxy 0.3.4, libclang 21.1.8,
ESP-IDF 5.5.6 commit `880a203ae603d08a94d06aa01e0d6761958e5e9e` and sys/HAL/service
revision `a209052f378be61dd0f2d566b7fbe0f9593b0cd5`. Preserve actual generated
configuration, lockfiles, linked Wi-Fi/PHY libraries, ELF, separate deployment
segments and tool identities before collection. Host Rust remains 1.96.0; the
qualified remote worker retains its explicit Rust 1.90.0 build identity.

UART0 uses GPIO16 TX/GPIO17 RX through the verified bridge at 115200 baud. One
fresh nonzero eight-byte nonce forms the private `di-ap-` SSID for both phases.
Controls A and B are the two fixed synthetic locally administered unicast MACs
in `wifi_protocol::restart`. They are not a uniqueness or collision policy.
The saved driver original must be valid unicast and differ from both controls.
Raw values, port paths, packets, credentials and flash images/digests remain
private. No eFuse, bridge configuration, provisioning or intentional key deletion
belongs to this procedure.

## Firmware state and failure handling

Status before StartFirst initializes only UART/control. Protocol version two
has explicit First/Second phases, increasing nonzero request IDs, one latched
session nonce and phase-tagged snapshots. IDs, phase and nonce correlate requests;
they do not authenticate a device. Preserve the first recorded failure for the
rest of the boot. Every protocol, transport, adapter or automatic-cleanup failure
permanently blocks later Starts, even when subsequent cleanup succeeds.

StartFirst lazily constructs one AP-only `WifiDriver` and system-event ownership,
selects RAM storage, saves the original driver AP MAC once, and configures channel
1, visible nonce SSID, open authentication and at most one client. Initialize no
EspNetif/DHCP, STA, BLE, IEEE 802.15.4, provisioning or NVS service. Actual Wi-Fi
NVS/PHY calibration persistence settings must be inspected. Compiled network
components do not imply application initialization.

While disabled, set A and verify its readback. Immediately before each actual
SDK start, invalidate any prior checked-stop evidence. Wait for the AP-started
state and require active driver readback equal to that phase's control. Setter
success alone cannot establish emitted identity.

Each Start attempt gets a fresh independent 8000 ms software deadline from the
command attempt, not its ACK. Poll without usable UART activity. Request Stop
at expiry; failed cleanup retries at one-second software intervals and retains
the first failure. Scheduling, SDK blocking and panic/reset can exceed requested
deadlines; these are not measured RF intervals or hard shutdown guarantees.

Stop requires inactive state, restoration of the one saved original driver MAC
and exact readback. A dedicated checked-stop flag can skip only a redundant SDK
stop after prior verified stop and a fresh inactive observation. Restoration is
checked on every Stop. Retain the initialized driver/subscription until reset,
with `initialized=true`; claim no deinitialization or SSID restoration.

Only a successful explicit Stop after phase A, with no sticky failure, authorizes
StartSecond. Automatic stop alone does not authorize it. Reuse the same driver,
original MAC, RAM configuration and SSID; change only the disabled AP MAC to B,
check it before and after start, and repeat checked Stop/original restoration.
Consume each phase's single attempt before its SDK work. No third start, retry
of a failed start or return to an earlier phase is admitted. Constructor failure
can leave partial initialization and remains terminal/recovery-required.

## Receiver and host sequence

Use the existing source-scoped Kismet stream without retuning, pausing, probing
interfaces, touching the Nordic port or changing service/configuration. Fresh
pre/post read-only snapshots must establish selected-source state, expected PID,
config metadata, channel 1 and separate default-route interface/PHY. Keep worker,
request and decoder identities frozen.

The collector opens UART exactly once, deasserts DTR then RTS once and validates
the separate one-second startup record. Opening/releasing/closing may reset the
device; pre-open bytes are unavailable. Retain the handle for the entire sequence.
Require an initial Idle/uninitialized snapshot, then exactly these five correlated
normal requests: Status, StartFirst, Stop, StartSecond, Stop.

Before each Start, create a separate receiver capture. Require its bound HTTP
200/framing-valid saved SHB READY, locally synchronized validated prefix and
live-child/cancellation check immediately before StartRequested and UART write.
This asynchronous check is not atomic with the write; later races fail the
session and trigger cleanup. One 32-packet prefix cannot supply both phases.

After each active ACK, the live path waits eight seconds before normal Stop;
firmware can already have stopped autonomously. Require the phase-tagged inactive
Stop and same original/current MAC relationship. Finish and fully replay that
phase's receiver, requiring at least one supported included-CRC exact SSID and
TA/BSSID beacon for its own control. Only after all A checks succeed may the host
create B's receiver. Monitor/reject UART noise or reset output while waiting for
receiver readiness, including the interphase setup. An observed reset/noise or
lost ACK never authorizes continuing to B.

Receiver limits remain 32 complete EPBs, 120 seconds, 8 MiB and a 132-second SSH
ceiling per phase; readiness waits at most 30 seconds. UART requests/replies
remain 256/1024 bytes, exchanges five seconds and raw storage 64 KiB. Drain UART
through receiver completion and the 200 ms suffix check. These software limits
are not hard real-time guarantees.

After any possible Start, attempt idempotent Stop on every error, including lost
ACK or receiver failure. Preserve failed attempts and cleanup records; later
successful cleanup cannot make a failed session pass. Capture/replay never
automatically resets or restores firmware. The external full-flash recovery
below always follows writes. Replay checks the exact saved inventory, complete
raw/transcript/receipt/hash agreement, phase readiness/order, five normal requests,
original-MAC continuity, both witnesses and byte-exact saved summary.

One live UART handle plus the enforced controller phase sequence supports
same-boot software evidence; it is not independent boot/reset attestation.
Recorded elapsed values establish monotonic event ordering only, not an RF
duration or host-wait threshold. Save the complete session, not two v1 trials
whose separate opens could reset the device.

## Optional collateral comparison

Presence is required independently for A and B. After both captures complete,
run the exact [profile comparator](wifi-beacon-profiles.md) on the same private
bytes and their respective targets. If every packet in both captures is a
CRC-verified exact target, record Qualified with its count-only report; expected
SSIDs must be equal and MACs different. Compare raw interval/capability and full
ordered non-SSID/vendor/per-ID TLVs, retaining duplicates and opaque contents.

Only `Error::ProfileCapture` yields the fixed Unqualified outcome for captures
not entirely composed of checked target beacons. Other errors fail the session.
Do not select target packets or treat Unqualified as equality. Mixed/unchecked
captures can establish scoped presence without qualifying the profile comparison.
Excluded metadata/header/TSF/FCS fields and vendor semantics retain their gaps.
Even identical profiles would describe these controls and packets, not tracking
accuracy, population uniqueness or a measured privacy improvement.

## Qualification, cohort and preservation

Qualify the actual new protocol/controller, single-handle host, receiver gates,
save/replay and cleanup with passing and deliberately failing controls. Cover
wrong/repeated/reordered phases and IDs, nonce/original drift, failed or automatic
Stop without explicit authorization, constructor/start/Stop failures, sticky
failure after recovery, repeated checked-stop invalidation, no-UART deadlines,
lost ACK, observed interphase reset, both receiver readiness/cancellation races,
wrong/unchecked/bad-FCS witnesses, late corruption and optional profile outcomes.
A separate reviewer constructs contrary inputs and traces compiled SDK/lifecycle
branches. Run host and target build/fmt/Clippy/docs checks. Freeze method, source,
actual generated config, binaries/segments, tools and private inputs before any
board write.

Plan exactly two complete A-to-B sessions with the same image, separated by one
explicit normal reset and a fresh session nonce. They are reset-separated boots,
not cold power cycles. Abort the cohort after its first failed session; preserve
it and investigate before changing inputs or repeating collection.

Apply the complete [AP backup/deployment/restoration gates](wifi-ap.md#preserve-and-restore-the-original-device):
fresh full 8,388,608-byte backup and separate entire-range verification before
writes without an intervening app boot; security/encryption and original-table
inspection; separate aligned bootloader/table/app segments with 4 KiB-rounded
erase footprints avoiding original data/key partitions and fitting the original
executable slot. Reject force, erase-all and merged padding across stored data.

Always restore the fresh complete image after either session success or failure,
then separately verify all 8,388,608 bytes before normal host reset release.
Retain both attempts if the first recovery fails; permit exactly one identical
recovery retry. Full-range device-MD5 comparison is not a second full-file
readback or device attestation. Host reset notices precede line operations;
require zero exit and no unexpected warning, allowing only the documented
no-reset preconnection notice. Postboot health, subsequent persistence and radio
activity remain unmeasured. No unverified partial image is released as success.

Replay both immutable sessions, independently decode every saved packet with
the pinned TShark tool preserving repeated JSON keys, verify exact target/CRC
witnesses privately and retain all failed cohorts. Publish only fixed counts,
readback relationships, restoration states and code/tool identities, with a
complete relative-path seal. Raw payloads/identifiers and their hashes stay
private. Seals establish integrity, not origin authenticity. Other radios, USB,
credentials and factory identifiers remain separate ownership domains.
