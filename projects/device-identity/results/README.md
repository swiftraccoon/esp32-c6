# Device identity evidence ledger

The inventory, supported modification methods and direct board observations
are the maintained investigation. The continuous-receiver qualification and
beacon-field comparison below remain complete historical supporting studies;
they add no uniquely identifying hardware source or application deliverable.
Their cohorts, failures and recorded methods remain available below.

| Collection | Observer and admitted observations | Limits |
| --- | --- | --- |
| [Read-only baseline, 2026-10-24 local](baseline-2026-10-24/README.md) | Attached Mac USB host and UART ROM access: two serial properties; three readable nonzero factory fields; explicit C6 reset completion | No runtime override, RF traffic, stored credentials, bridge lock state or uniqueness qualification |
| [Runtime table probe, 2026-10-24 local](runtime-table-2026-10-24/README.md) | Two UART sessions: late-base caching, four explicit interface table controls, saved-table restoration; Mac serial properties unchanged; full original flash restored/verified | Initial zero-RX cohort retained; USB properties can be cached; no active-radio, cold-boot, reconnect or correlation result |
| [USB control-pipe diagnostic, 2026-10-25 UTC](usb-control-2026-10-25/README.md) | One complete ROM-state capture/replay: both nonempty serial descriptors and bridge configuration length/layout; separate host reset sequence completed | Both original-state cohorts failed native LANGID reads; cause unknown. No repeatability, cross-state equality, kernel-cache qualification, bridge checksum/lock or physical application-boot result |
| [Wi-Fi receiver path, 2026-10-25 UTC](wifi-observer-2026-10-25/README.md) | Two complete 32-EPB ambient prefixes from the existing remote Kismet source; strict saved replay and radiotap/FCS/management decoding | Earlier local rejection and quota failure retained. Zero beacons leaves physical beacon decoding unexercised; no C6 reception, delivery-fidelity, timing, silence or rotation result |
| [Controlled AP witness, 2026-10-25 UTC](wifi-ap-2026-10-25/README.md) | Two reset-separated boots: pre-start driver AP-MAC readback and 64 exact nonce/TA/BSSID CRC-valid beacons; checked inactive Stop/saved MAC restoration; full original 8 MiB restored/verified | Presence-only bounded prefixes; driver remains initialized until reset. No complete startup/Stop window, cold boot, hot rotation, silence, correlation reduction or postboot health/persistence result |
| [Retrospective beacon fields, 2026-10-25 UTC](wifi-beacon-profiles-2026-10-25/README.md) | Both complete original bundles replayed; all 64 saved beacons share exact interval/capability and ordered non-SSID/vendor/per-ID TLVs despite distinct nonce SSIDs | Already inspected two-boot data, unchanged AP MAC and configuration; no new RF collection, population uniqueness, tracking accuracy, rotation or correlation-reduction result |
| [AP address change across restart, 2026-10-25 UTC](wifi-restart-2026-10-25/README.md) | Two sessions, each with one retained driver/UART and checked A-to-B stop/restart: 120 exact CRC-valid target beacons, four checked Stops/original restorations, complete replay and full fresh 8 MiB restoration/verification | Same SSID links phases; eight other management frames leave both whole-capture profile comparisons Unqualified. No independent boot attestation, complete transition/startup window, silence, hot change, cold boot, reconnect or correlation-reduction result |
| [Saved USB/flash audits, 2026-10-25 UTC](saved-identity-audits-2026-10-25/README.md) | Whole ROM USB replay supports factory-MAC serial correspondence and CP2102N Fletcher16 integrity; three original full images pass physical partition/NVS metadata checks with independent SDK agreement | Retrospective explored data; no new device access. Original failed USB cohorts remain rejected; bridge lock, cold boot, logical blob contents, credentials and controlled persistence remain unqualified |
| [Continuous receiver qualification, 2026-10-25 UTC](wifi-window-2026-10-25/README.md) | Two 30-host-second windows admit explicit FINISH/cleanup and exact ten-file replay; all 18 included packet CRCs independently agree; receiver guards unchanged | First attempt remains rejected after cleanup-race correction. No C6 serial/reset/write; unknown stream loss and no independent RF reference, target witness, RF timing, startup or silence result |
| [Targeted provisioning metadata, 2026-10-25 UTC](provisioning-metadata-2026-10-25/README.md) | Stock-tool 17-field ROM read matches the factory baseline; all 256 BLOCK3 data bits zero and unprotected; six USER key purposes with read protection clear; secure boot/flash encryption disabled; explicit release status zero | Key contents excluded from output, although stock tool reads all blocks internally. No programming or key-block blankness claim. Broader SPI/XMC release effects; no independent boot, storage-persistence or RF attestation |
| [Flash UID and provisioning storage, 2026-10-26 UTC](flash-identity-2026-10-26/README.md) | Accepted 846-call unchanged-API read: 16-byte UID/five overlaps, bounded SFDP, three 1024-byte FF security regions with locks clear, readable eFuse blocks 3–10 reported zero; explicit reset release | First completed 852-call MOSI schedule remains failed. Exact suffix/population uniqueness, discarded raw-key replay, electrical timing and independent application boot/persistence are unqualified; no nonvolatile programming |
| [BLE identity through HCI, 2026-10-26 UTC](ble-identity-2026-10-26/README.md) | Original application has eight expected public-address reports; one retained-controller/UART A→B session has six checked responses, two exact random-A and one random-B nonce reports; complete replay, unchanged receiver guards and verified fresh full 8 MiB restoration | Earlier pre-Start expiration and preparation/console/discovery failures retained. HCI presence only: no raw RF CRC, silence, full transition/startup, hot change, RPA, cold boot, correlation reduction or independent postrestore boot/persistence result |
| [Saved full-flash coverage, 2026-10-26 UTC](saved-flash-coverage-2026-10-26/README.md) | Five complete 8 MiB originals each have three exact build artifacts, inspected NVS and FF-only gaps; 15 artifact matches, all 122,880 NVS bytes/3,780 slots reproduced, three current PHY values each and only recognized historical PHY head keys | Retrospective overlapping snapshots, no new device access. Compiled/opaque payload semantic absence, historical producer and live volatile keys remain unqualified. Separate vendor-source interpretation resolves the saved bridge lock classifier and records the supported native-USB/key modification paths |
| [Original NimBLE live state, 2026-10-26 UTC](nimble-live-2026-10-26/README.md) | Two reset-separated original-application debug snapshots: nonzero equal default/current IRKs within each boot, different between boots, five null key/store callbacks and clear SC flags; checked resume and final normal reset | Active debug effects; no entropy/population-uniqueness, pairing, SC public-key, link-key, RPA or on-air result. Final recovery boot's keys remain uncaptured |
| [Bridge programming/activation, 2026-10-26 UTC](bridge-config-2026-10-26/README.md) | Five complete captures/50 successful transfer rows across three retained failed trials; the third attempt's candidate capture reads all 678 custom bytes exactly, then all original bytes/serials restored and final normal C6 reset verified | All whole serial-change trials remain failed: fresh active serial stays original after selected Mac SDK reset. No full bridge power-cycle activation/persistence or internal-ID modification result |
| [Bridge delayed-reset timing, 2026-10-27 UTC](bridge-timing-2026-10-27/README.md) | Three complete captures/30 successful transfer rows; all 678 candidate bytes match, serial response remains original; five declared host gaps exceed ten seconds; exact original restoration and normal C6 reset verified | Failed serial criterion for one vendor-guided selected setting. No causal/exhaustive-timing, power-cycle activation/persistence or independent lower-kernel cache result |
| [Factory calibration export, 2026-10-27 UTC](factory-trim-2026-10-27/README.md) | Exact stock-tool 27-row export: all 24 trim fields/153 defined bits plus baseline-matched MAC and supported block versions; raw layouts, access metadata, 77 frozen inputs and final matched normal reset independently replayed | Private raw values; no population uniqueness, entropy, calibration accuracy, application-health or later-persistence claim. Stock constructor reads blocks beyond the selected formatted fields; no programming command requested |

Retrospective inspection of the three original images additionally finds the
factory MAC in CRC-valid PHY calibration payloads and its EUI-64 prefix in
separate `cal_mac` payloads. The saved applications match an existing local
binary and descriptor-matched ELF identifying ESP-IDF 5.4.1; adjacent source,
configuration and selected compiled behavior describe a fixed public-address
BLE advertising application. The [inventory](../README.md) records its fields,
modification methods and source correspondence limits. Original images, exact
private input identities and failed earlier attempts remain preserved. These
existing-byte/source findings add no new board access, RF observation, logical
NVS reconstruction, credential-absence or controlled persistence result.

The later full-array audit extends coverage to all five saved originals. It
reproduces the known MAC-copy relationships in each, with no unexplained
programmed gap. Source/ELF review additionally identifies IRK generation attempts,
the cached SC public key and negotiated connection material as volatile identity
surfaces. These findings do not turn source branches into captured live keys or
the recognized-storage inventory into arbitrary credential-absence proof.

Separate retrospective upstream blob reconstruction of the three saved NVS
slices finds three current PHY values in each and no additional recognized
current or erased identity/credential key. Erased heads are 16/18/18 per snapshot,
all using the known PHY keys; MAC-bearing first chunks match the same factory
MAC. Snapshots overlap and historical chunks were not reconstructed as current
values. This adds logical inspection of the saved PHY blobs, without proving
opaque data or the whole flash credential-free or adding a new board trial.

The [inventory method](../experiments/read-only-inventory.md) defines collection
and reset behavior. Public JSON files are sanitized derivatives from private raw
inputs. Their seals establish retained-file integrity, not authenticity or
independent raw replay. Reproducing a real observation requires the private
capture. The inventory CLI's CI checks exercise synthetic passing/failing
controls. The HCI trial uses stock decoders and qualified private one-off
compositions; its public CI check verifies only the sanitized archive seal.

The [original NimBLE live-state method](../experiments/nimble-live-state.md),
frozen before collection, now has two admitted observations through existing
debug tools. The result records transient debug effects, exact selected-byte
admission, reset-separated IRK variation and mandatory normal-reset recovery.

The [bridge serial method](../experiments/bridge-serial.md) now has three retained
failed physical attempts. Exact custom configuration readback and complete
original restoration are observed; active descriptor replacement is not.
The result separates the first CLI argument rejection, the second host-metadata
guard and the third fresh-response mismatch, with prospective corrections and
independent replay. The selected SDK reset does not establish a full bridge reset.
The later [fixed timing distinction](bridge-timing-2026-10-27/README.md), based on
new vendor guidance, also remains failed. Its five measured host gaps and complete
fresh-response/recovery evidence do not establish that another delay or a full
power cycle would succeed.

Qualification retained one failed virtual replay: the initial parser assumed
generic formatting for `MAC_EXT`. Actual SDK output rejected that assumption.
The corrected MAC-class encoding is checked by nonuniform byte-order controls,
an installed-SDK virtual-class check and the unchanged saved virtual capture.
Independent adversarial review also found that help output included an alternate
executable name; fixed command metadata and CLI regression controls now cover
that path. Passing controls, malformed late-field rejection, protected unknown
states, sentinel omission and complete repeated replay were reproduced. Raw
attempts and review controls remain in ignored private storage; maintained
regression tests retain the lasting findings.

Untested coverage remains explicit in the project inventory: active station
and IEEE emitted addresses, BLE raw RF CRC/RPA, native serial cold boot/repeatability,
active custom bridge serial after a full power cycle,
exact flash suffix, application credentials/controlled persistence, passive RF silence,
hot address changes while running and correlation reduction. No anonymity guarantee follows.

The runtime method retained its initial zero-RX cohort at freeze `368d57f`.
The same firmware answered a foreign-request diagnostic after explicit bridge
line release. Transport revision 2, frozen at `c5ff2fc`, checks that release and
validates separately retained startup records before table requests. Two complete
revision-2 sessions and their entire saved directories passed independent replay;
the first attempt remains rejected. The runtime collection records controls,
source identities, measured scope and the verified restoration boundary.
