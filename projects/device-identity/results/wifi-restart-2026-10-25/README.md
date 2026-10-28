# SoftAP address change across restart

Two reset-separated sessions each admitted the same-boot sequence of control A,
checked Stop/original-MAC restoration, control B and checked Stop/restoration.
The independent receiver saved 120 CRC-verified exact-target beacons across
four bounded prefixes. Active driver readbacks matched the distinct controls;
the one saved original MAC was restored after each phase. The fresh complete
original 8 MiB flash image was restored and separately verified before release.

This establishes observed address change across a stopped/restarted AP using
the retained driver. The same private SSID deliberately links both phases.
Transition traffic, startup/default-address exposure, RF silence, hot changes
while running, cold boot, client reconnect and correlation reduction remain
unqualified. One live UART handle and the enforced controller sequence support
same-boot software evidence, not independent boot/reset attestation.

## Cohort and observations

The [prospective method](../../experiments/wifi-restart.md), 73
[source/configuration inputs](inputs.sha256) and [artifact identities](artifacts.json)
were frozen at `9d48f0a` before board access. Exactly two sessions were planned,
attempted and admitted, with no rejected hardware session. An explicit normal
reset separates them; they are not cold power cycles. Each session used one
fresh nonzero nonce, held constant within its two phases and distinct between
sessions. [Result JSON](result.json) publishes relationships and counts only.

| Observation | [Session 1](trial-01.json) | [Session 2](trial-02.json) |
| --- | --- | --- |
| Initial state | Idle, driver uninitialized | Idle, driver uninitialized |
| UART opens / normal commands | 1 / 5 | 1 / 5 |
| A / B received records | 32 / 32 | 32 / 32 |
| A / B exact CRC-verified target beacons | 32 / 30 | 29 / 29 |
| Other management frames | 2 | 6 |
| A / B active driver readback | Own control, distinct addresses | Own control, distinct addresses |
| Stop after each phase | Inactive, same saved original restored | Inactive, same saved original restored |
| Driver initialized after Stop | Yes, retained until reset | Yes, retained until reset |
| Complete replay | Passed, saved summary byte-equal | Passed, saved summary byte-equal |
| Optional collateral comparison | Unqualified | Unqualified |

Each Start had its own bound HTTP 200/SHB READY, locally validated saved prefix
and live-child/cancellation check. A's explicit Stop, full receiver replay and
target witness were admitted before B's receiver was created. All ten normal
UART exchanges had zero status and no sticky failure, with phase/nonce/original
continuity. These saved records agree with the frozen live path; they do not
independently re-attest process liveness or the physical transmitter.

All 128 records remain in the analysis. Eight are other management frames,
so neither session meets the predetermined whole-capture profile admission rule.
Both reports correctly retain Unqualified instead of selecting target beacons
for a favorable comparison. No interval/capability/vendor/non-SSID profile
equality result is claimed for this address-change cohort. The
[earlier field comparison](../wifi-beacon-profiles-2026-10-25/README.md) retains
its separate already-inspected, unchanged-MAC cohort and scope. Vendor semantics
and collateral identifying information remain unaudited.

Independent postcollection review decoded all 128 records with the pinned
TShark tool, preserving repeated JSON keys, and independently checked every
included CRC. It agreed on all 120 exact nonce/TA/BSSID beacon matches and
classified the remaining records as five probe requests and three probe
responses. Both complete 27-file sessions replayed to byte-exact saved reports.
Twenty-four correctly rehashed semantic copies rejected, covering phase,
original-MAC/nonce drift, sticky failures, readiness, receiver/connection records,
missing targets, late malformed bytes, bad CRC and a fabricated Qualified result.
The original captured records were not repaired or filtered.

The [before](observer-inputs.json)/[after](observer-after.json) receiver states
and [continuity check](observer-continuity.json) agree: selected running
linuxwifi source with no error/pause/hopping, the same expected process and ten
config metadata records, capture interface bound to the inspected monitor on
channel 1, and a distinct Intel default-route PHY. Both route-family queries
were valid. No retuning, datasource control, Nordic-port access or service
change occurred; ordinary Kismet reception continued.

## Device preservation

[Restoration evidence](restoration.json) records a fresh 8,388,608-byte backup
and separate full-range verification before writes. Security/partition gates
passed. Actual file lengths matched segment manifests; their rounded 4 KiB
erase ranges avoided every original non-executable partition and kept the
application within its original executable slot. The temporary layout differed
from the original; fitting the app did not establish layout equality. Separate
bootloader/table/app writes introduced no merged padding across stored data.

After both sessions, the complete fresh image was restored and a separate
esptool device-MD5 range comparison verified all 8,388,608 bytes at
`2026-10-25T19:48:14.075490Z`. This is full-range digest verification, not a
second full-file readback or attestation. The host reset sequence completed at
`2026-10-25T19:48:14.794545Z`; all twelve recorded hardware command phases exited
zero and the first restoration attempt passed. A reset notice precedes line
operations; zero exit/no unexpected warning is host-sequence evidence, not a
physical boot acknowledgment. Postboot health, persistence and radio activity
remain unmeasured. No eFuse programming, persistent bridge-configuration
programming or intentional key deletion occurred.

## Qualification and retained limits

Before collection, 142 host tests and seven doctests, formatting, workspace
Clippy/docs and both target-bin build/lint/docs passed. Both older actual v1
AP replays and their prior profile report remained byte-exact. Independent
controls checked controller failures/deadlines, capture/save/replay, queued
noise/reset suppression, exact request/phase/original bindings, optional profile
outcomes and both receipt paths. The final production executable passed 58
independent replay controls: 15 admitted and 43 rejected. Twenty-eight separate
actual-code controls checked deployment sizes, both route-query failure families
and source/interface binding. The corrected response deadline checks retain a
late raw LF and reject its ACK; synthetic success uses modeled transport time
while production retains the real five-second wait limit.

Initial setup/qualification failures, contrary inputs and pre-correction source
remain private; they are not additional hardware sessions. Preserve original
receipts and failed attempts. Physical presence does not qualify whole-window
coverage, delivery fidelity, source authenticity, cross-interface identities,
application credentials, population uniqueness, tracking accuracy or measured
privacy improvement. The same SSID is already a direct correlation marker.

[Firmware](firmware-inputs.json), [flash tools](flash-tool-inputs.json),
[worker](worker-inputs.json) and [decoder](decoder-inputs.json) files are frozen
precollection snapshots. Fields marked precollection/unqualified are not the
final on-board result state. Runtime libraries are not hermetically pinned.
Raw identifiers, SSIDs, payloads, original full-flash images and their hashes stay
private. The full driver configuration is retained in RAM until reset; Stop
claims MAC restoration and inactive state, not deinitialization/SSID restoration.

`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. Input manifests instead use workspace-root paths at `9d48f0a`,
including the precollection README. They do not claim current result text is a
collection input. Seals establish integrity, not origin authenticity; exact real
replay requires the private 27-file session bundles.
