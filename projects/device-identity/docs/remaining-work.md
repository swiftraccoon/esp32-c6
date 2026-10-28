# Identity coverage and remaining validation

The objective is a complete practical inventory of this board's identities and
supported modification paths. The [main inventory](../README.md) separates
manufacturer documentation, observed bytes and unverified behavior. Vendor-source
research and the five-image coverage audit now identify the documented hardware
sources and selected original-firmware identity surfaces. Two live snapshots
now resolve the selected volatile IRK/callback/SC-flag state; opaque data
semantics and population uniqueness remain distinct limits.

Four documented factory identity sources are accounted for: the C6 factory MAC,
optional eFuse ID, external flash UID and separate bridge's assigned internal
identity/default serial. Native USB and default radio addresses expose or derive
from these roots. The inventory also covers the identified mutable stored and
application identity surfaces, including the observed volatile IRKs. Exact part
ordering suffix, population uniqueness and untested emitted-address behavior
are evidence limits rather than additional missing factory-identifier readouts.
The later primary-source completeness review additionally catalogs 24 immutable
factory calibration/trim fields as possible correlation material and the
module's printed MAC copy. Neither establishes another documented unit-ID root.
All 24 trim values are now retained privately through the accepted stock-tool
export and independent replay. The physical module marking remains unscanned.

| Question | Current answer | Remaining limit |
| --- | --- | --- |
| Can factory MAC/optional ID be rewritten? | No supported rewrite of populated RS-coded BLOCK1/2, regardless of unset write-protection flags. | No destructive test needed. |
| Is MAC_EXT another unique ID? | No; this board has the standard FF:FE insertion. | Closed. |
| Are factory calibration values accounted for? | Exact 27-row export retains all 24 defined trim fields plus MAC and block versions; layout, raw widths, readability, baseline correspondence and final normal reset independently replay. | They are immutable correlation material on this board, without established population uniqueness or entropy. The module's printed MAC copy remains unscanned. |
| Does native USB reveal factory identity? | Complete ROM serial matches factory MAC. Espressif's hardware-descriptor explanation, register map and SDK source provide no supported software customization route. Custom application MAC selection is not a USB serial-change procedure. | Original-application wire reads failed. Undocumented mechanisms and custom-fuse effects were not physically tested; no destructive test is needed to document supported interfaces. |
| Can the bridge serial change? | Vendor configuration supports a custom descriptor. On this Mac the entire 678-byte custom array reads back exactly, with update-enable FF, and all originals are restored. The serial response remains original after the selected SDK reset, including a separately frozen ten-second-before/after timing setting; all four whole change trials remain failed. Subsequent source/byte review matches both candidates to the existing tool's encoding and provides no formatting correction. | Active custom exposure and persistence through a full bridge power cycle remain unverified. Both USB cables feed bridge power; C6 RESET does not reset it. The underlying assigned ID has no documented setter. |
| Which flash identities exist? | A complete 16-byte UID window passes five overlaps and matches the documented 128-bit D format. Three 1024-byte security windows read FF with locks clear. | The read-only inventory is complete for these documented C/D regions. Exact fitted suffix is unconfirmed; no destructive modification test is needed. |
| Is a custom eFuse MAC provisioned/possible? | All 256 BLOCK3 data bits read zero and write protection is unset. Pinned espefuse supports one-time custom-MAC provisioning for the SDK application path. | No burn was attempted. It is not an established USB descriptor-modification method. |
| Are hardware/provisioned keys another identity source? | Key purposes are USER, protection is clear, and the accepted read reports all data bits in readable blocks 3–10 zero. Secure boot/flash encryption report disabled. | No additional provisioned key/user-block identity is identified. Raw key bits were discarded; software-managed credentials are a separate storage question. |
| Are factory addresses stored elsewhere? | Five full images have exact known firmware artifacts, inspected NVS and FF-only gaps. All NVS bytes/slots are accounted for; each has three current PHY values and only recognized historical PHY head keys. Both stored MAC relationships hold in all five. | Opaque PHY and compiled data are not proved free of identifying information. No additional named persistent identity store was found; byte coverage is not arbitrary semantic absence proof. |
| What does the original application configure? | Matching images/ELF and adjacent source identify public-address BLE advertising with fixed name/services/manufacturer data; fresh HCI reports match them. All five ELF store callback pointers are null; no configured bond-store backend is established. | The source was not rebuilt for complete reproducibility. Actual pairing/history and peer-side state were not observed. |
| Which volatile BLE identities can exist? | Two reset-separated original-firmware snapshots have nonzero equal default/current IRKs that differ between boots, five null key/store callbacks, and clear SC-generation flags. Public IRK generation callback and chip-reset/SC-reinitialization paths are documented. | Two observed values do not qualify entropy or population uniqueness. No pairing was induced; SC public key, link/peer material, RPA behavior and the final recovery boot's keys were not captured. |
| Can runtime addresses be changed? | Supported SDK methods are documented; table overrides, CRC-valid AP changes and receiver HCI witnesses for two BLE random controls are verified. | Station/802.15.4 emitted changes, BLE raw RF CRC/RPA and hot changes remain unverified. These are behavior-validation limits, rather than missing factory-identifier readouts. |

Missing hardware facts stay unknown until a targeted read or supported source
resolves them.
