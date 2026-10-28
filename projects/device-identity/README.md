# Device identity investigation

[Workspace](../../README.md) · [Projects](../README.md)

Identify the per-device identities on this ESP32-C6-DevKitC-1-N8, determine which
are exposed to each observer, and document whether and how each can be changed.
Distinguish immutable factory data, derived addresses, configurable descriptors
and stored application identities. A nonzero value or a model identifier does
not by itself establish uniqueness.

The deliverable is the identifier inventory, supported modification procedures
and their evidence. Probe firmware and host tools support those observations.
The [coverage limits](docs/remaining-work.md) distinguish answered identity
questions from unverified behavior.

## Existing answers and tools

Most identifier semantics and supported modification paths are already specified
by the vendors. Use the existing extraction tools and SDK interfaces; local
probe firmware is only needed for a specific observation the sources cannot
establish about this board.

| Question | Existing answer/tool | Board-specific work |
| --- | --- | --- |
| Factory eFuses, protection and custom MAC | [Filtered espefuse summary](https://docs.espressif.com/projects/esptool/en/latest/esp32c6/espefuse/summary-cmd.html), C6 field table/TRM and existing `burn_custom_mac` support. | Read selected fields; do not burn factory data to test documentation. |
| MAC derivation and runtime overrides | [ESP-IDF MAC guide](https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32c6/api-reference/system/misc_system_api.html) and the [existing pinned base-MAC example](https://github.com/espressif/esp-idf/tree/880a203ae603d08a94d06aa01e0d6761958e5e9e/examples/system/base_mac_address), plus radio-specific APIs. | Check selected configuration and only the required emitted addresses. |
| Native USB serial | Espressif documents the MAC-based serial and hardware-defined USB Serial/JTAG descriptors; standard host USB inspection exposes them. | Saved ROM correspondence is established. No supported software descriptor-customization route is identified; original-application control reads failed. |
| Bridge serial and configuration | Silicon Labs Xpress Configurator, CP210x configuration APIs and the existing [CP2102N-aware cp210x-cfg](https://github.com/irrwisch1/cp210x-cfg), also used by [Home Assistant's programmer](https://github.com/home-assistant/addons-development/blob/master/skyconnect_cp2102n_programmer/Dockerfile). Vendor SMT supports full-array restoration for the [targeted serial check](experiments/bridge-serial.md). | The [Mac trials](results/bridge-config-2026-10-26/README.md) read back all 678 custom bytes and restore all originals. A [vendor-guided ten-second reset timing variant](results/bridge-timing-2026-10-27/README.md) also returns the original serial. Full power-cycle activation remains unverified. |
| Flash identity/storage | Existing esptool SPI API, composed as the [qualified address/dummy read schedule](experiments/flash-api-read.md); ordinary ID/SFDP/status commands remain available. | A complete 16-byte candidate UID window and three security regions have been read. The SDK's separate UID API returns only 64 bits; exact fitted suffix remains unconfirmed. |
| Stored NVS values | [ESP-IDF NVS parser](https://docs.espressif.com/projects/esp-idf/en/v5.4.1/esp32c6/api-reference/storage/nvs_partition_parse.html) provides JSON, integrity checks and blob reconstruction. | Five complete original images now have byte coverage and NVS slot/head inspection; only the known PHY values and historical keys were identified. Opaque payload semantics remain distinct from structural coverage. |
| BLE packet inspection | Stock BlueZ `btmon` and TShark inspect receiver HCI reports; [Nordic Sniffer with Wireshark](https://docs.nordicsemi.com/r/bundle/nrfutil/page/nrfutil-ble-sniffer/guides/overview.html) supplies a separate raw-packet path. | The existing Intel receiver now has [original-public-address and controlled A/B witnesses](results/ble-identity-2026-10-26/README.md); raw RF CRC and Nordic capture remain unqualified. |

Current web documentation may use hyphenated esptool command names; the installed
4.12 tool uses underscores. Source reviews and experiments keep their actual
SDK/tool versions pinned rather than silently adopting current web examples.

## Factory and hardware identities

| Item | Identity and exposure | Changeability and evidence |
| --- | --- | --- |
| C6 factory MAC, 48 bits | Espressif factory-assigned base identity; readable by software/debug tools and used to derive default interface addresses. Observed nonzero on this board. | No supported rewrite of this already-programmed BLOCK1. Its write-protection flag is unset, but the populated RS-coded block cannot be programmed again. RAM address overrides do not change it. |
| MAC_EXT, 16 bits | This board contains the standard `FF:FE` insertion. It is not an additional unique serial. The default IEEE 802.15.4 EUI-64 inserts it between the factory MAC's two halves. | In the same already-programmed BLOCK1; no supported rewrite. The derived runtime extended address is separately configurable. |
| Optional 128-bit eFuse ID | Readable and nonzero in BLOCK2. Espressif names it a unique ID and documents RainMaker self-claiming as a use, including software HMAC challenge responses. Its generation algorithm and a C6 population uniqueness guarantee are not specified in the reviewed sources. No automatic on-air exposure by this board is established. | BLOCK2 is already populated and cannot be programmed again through the supported RS-coding path. No reversible change method. |
| Factory eFuse calibration/trim vector | All 24 documented bias, temperature and ADC calibration fields are [captured and independently replayed](results/factory-trim-2026-10-27/README.md): 153 defined bits, comprising 28 in BLOCK1 and 125 in BLOCK2. These may contribute stable correlation material; they are not a documented collision-free serial or 153 bits of entropy. | They share the already-populated RS-coded BLOCK1/2, with no supported rewrite. Existing field-selected `espefuse summary` or `esp_efuse_read_field_blob()` reads them; raw values stay private. Changing application calibration calculations does not rewrite the fuses. |
| Native USB Serial/JTAG serial | The complete saved ROM descriptor contains the factory MAC's 17-character colon representation. This exposes the same underlying identity, not another independent factory ID. | Hardware-defined descriptors; no supported software customization route. Espressif explains that enumeration must work without ROM execution. SDK RAM/custom-base selection provides no USB serial setting. Software overrides left possibly cached host properties unchanged. Avoiding the native connection changes exposure; application disabling can occur after initial enumeration. |
| CP2102N internal identity / default serial | Silicon Labs documents an assigned 128-bit unique ID used as the default USB serial. The observed 32-hex-character serial and internal-serial selector are consistent with that default. This is a separate chip from the C6. | No internal-ID setter is documented in the reviewed sources. A custom USB serial descriptor can replace its exposure without establishing a change to the internal ID. |
| CP2102N custom USB serial | Documented nonvolatile configurable descriptor, up to 63 UTF-16 code units; separate from the assigned internal ID. | The valid V1 configuration is **unlocked**: update-enable byte 4 is `FF`. Use Xpress Configurator or `GetConfig`/`SetConfig`, select the custom string, preserve unrelated fields/update-enable and recompute Fletcher16, then reset/reconnect and read the actual descriptor. Full 678-byte custom configuration readback and exact restoration are observed. The selected Mac SDK reset leaves the active serial original; power-cycle activation/persistence remain unverified. Do not treat configuration readback as active serial replacement. |
| SPI flash UID | A nontrivial 16-byte response is captured with five matching shifted reads. Non-FF data beyond byte 7 contradicts the C datasheet's post-64-bit padding and supports the documented 128-bit D format. The saved SFDP DTR capability also agrees with D; shared JEDEC `20/40/17` and SFDP headers do not encode the exact ordering suffix. | XMC documents the factory UID as read-only; ordinary erase/reflash does not change it. No supported rewrite path. The [accepted read](results/flash-identity-2026-10-26/README.md) covers both documented widths; exact fitted ordering/suffix remains unconfirmed. |
| Flash security/OTP registers | All three 1024-byte read windows returned FF, with all three permanent lock bits clear. Under the documented C/D layouts, no programmed identity was found there. | While unlocked, vendor write-enable plus Program Security Register (`42h`) / Erase Security Register (`44h`) sequences modify this storage. Permanent locking prevents later modification. No such operation was performed. These regions and the factory UID are outside an ordinary 8 MiB backup. |
| Custom C6 eFuse MAC | Optional separate `USER_DATA.MAC_CUSTOM` in BLOCK3 bits 200–247; not a replacement of factory BLOCK1. | All 256 BLOCK3 data bits read zero and write protection is unset. Pinned espefuse provides one-time `burn_custom_mac`; SDK selection uses `CONFIG_ESP_MAC_USE_CUSTOM_MAC_AS_BASE_MAC`. This configures the application MAC path, not a supported USB descriptor-change procedure. No custom-fuse experiment was performed. |

The saved eFuse metadata says `readable=true` and `writeable=true` for MAC,
MAC_EXT and the optional ID. Here, `writeable` only checks write protection.
BLOCK1–10 use whole-block RS(44,32) coding; the SDK and pinned espefuse reject
new data in an already-populated RS block. Do not mistake an unset protection
bit for erase/rewrite capability. BLOCK1/2 also have no per-block read-disable
control. Factory field contents and their hashes remain private.

USB VID/PID, product/release strings, silicon/package revision, JEDEC family,
flash capacity, SFDP capabilities, JTAG part identifiers and calibration-version
numbers describe classes or configurations; they are not documented unique
per-unit serials. Calibration bytes may vary without being collision-free IDs.

The documented module Data Matrix includes the MAC alongside manufacturing
metadata, providing a physical copy of the same identity. Firmware changes do
not rewrite printed markings; covering or relabeling changes only visibility.
This unit's marking was not scanned. CPU vendor/architecture/implementation/hart
CSRs and the standard `esptool chip_id` command do not add a documented C6 unit
serial: the latter uses the MAC fallback on this target.

Bridge activation is a separate reset domain from the C6. The examined Mac
SDK ignores the OS reset result, and its selected re-enumeration path does not
validate serial-string contents. In published board schematics both USB ports
power the shared bridge supply; a full bridge power cycle requires both cables
and any other supply/backpower removed. C6 RESET does not drive bridge RSTb.
No such full cycle was measured in the [programming result](results/bridge-config-2026-10-26/README.md),
and it is not an established fix for that observed activation limit.
The later fixed ten-second waits before/after both resets, with a conservative
restoration pause, also fail the requested-serial criterion. Actual host gaps
and exact original recovery are recorded in the [timing result](results/bridge-timing-2026-10-27/README.md).
This rejects that setting without exhausting timing possibilities.
Subsequent source/byte review reproduces both tested candidate arrays exactly
with the [pinned CP2102N-aware tool's serial encoding](https://github.com/irrwisch1/cp210x-cfg/blob/b417a28b3367a65990ad501c6cb98da2c32b7ea4/src/main.c),
including zero-filled unused serial storage. No formatting correction or
additional programming trial follows from that review. Full power-cycle
activation remains unverified.

The [targeted provisioning metadata read](results/provisioning-metadata-2026-10-25/README.md)
also reports all six key purposes as `USER`, with no key-block read-protection
bits set. Secure boot and flash encryption report disabled. The flash-read
session additionally classifies all 256 data bits in each readable block 3–10
as zero, including all six key blocks and reserved system-data block 10. No
additional provisioned identity is identified in those blocks. Raw key-block
data were discarded; saved booleans cannot independently replay those bits.
The stock tool reads all blocks internally, without formatting key contents.
Both accepted reads completed their explicit host reset releases.

## Runtime addresses and supported modification paths

| Address | Supported method | Scope and board verification |
| --- | --- | --- |
| Application base/interface tables | `esp_base_mac_addr_set()`; `esp_iface_mac_addr_set()` for the selected interface. Apply before interface initialization and cached reads. | RAM state; store/reapply a choice at boot if persistence is required. All four interface table controls were read back in the runtime probe. Factory eFuses remain unchanged. |
| Wi-Fi station / AP MAC | `esp_wifi_set_mac()` for the selected disabled interface, or the pinned safe driver's `set_mac`; check readback before starting it. Use a valid unicast address and avoid network collisions. | Pre-start AP changes and A→B changes across requested stop/restart have independent CRC-valid beacon witnesses. Active station transmission has not been verified. |
| BLE advertising address | `esp_ble_gap_set_rand_addr()` and a matching random own-address type while advertising is inactive. Controller privacy APIs provide separately managed private addressing. | Two distinct static-random controls have independent receiver HCI address/nonce witnesses across a checked Stop in one retained controller/UART session. This qualifies the selected Bluedroid probe; raw RF CRC, hot changes and RPA remain unverified. Public/base tables, bonding/IRKs and payloads are separate identity surfaces. |
| IEEE 802.15.4 extended address | `esp_ieee802154_set_extended_address()` with the driver's documented byte order; short address/PAN ID have separate setters. | Supported SDK method; only pre-initialization address-table readback is observed. No emitted-frame verification yet. Network stacks can manage their own identity/persistence. |

These are supported procedures with separately stated verification, not claims
that every configured value has been observed on air. Changing an address does
not change a USB identity, flash UID, factory eFuse or application credential.
No eFuse burn or key deletion belongs to these experiments. The targeted bridge
descriptor method separately requires exact full-configuration restoration.

## Stored identities

The five saved original flash images contain `phy/cal_mac`, `phy/cal_data` and
`phy/cal_version` records. The original application identifies ESP-IDF 5.4.1;
its 1904-byte calibration structure has four version bytes, six MAC bytes, then
1894 opaque calibration bytes. CRC-valid written payloads contain the complete factory
MAC in that six-byte field in all five images. Their separate six-byte
`cal_mac` equals the first six bytes of the factory-derived EUI-64 in all five.
The matched ELF calls the factory getter, which expands its result to eight
bytes on this target, and stores six bytes. The SDK source also passes a
six-byte buffer to that eight-byte getter. This is consistent with the observed
prefix, without attesting its historical producer. The prefix is not an
additional independent identity. This retrospective interpretation establishes
neither the board's present storage state nor a controlled persistence result.

Upstream NVS blob reconstruction finds exactly three current values
in each saved slice: a PHY format integer, the six-byte MAC blob and the
1904-byte calibration blob. Current blob relationships agree with the physical
payload inspection. Erased heads use the same PHY keys: 16/18/18 heads in the
first three snapshots; the two later pre-BLE images have 20 and 17 erased heads.
These overlapping backup histories are not independent calibration runs;
historical chunks were inspected physically, not reconstructed as live values.

The five current calibration payloads have four distinct opaque byte strings,
while the version and known MAC fields are unchanged. These bytes can correlate
saved copies while retained; they are mutable calibration material, without a
documented additional immutable serial or demonstrated population-unique
fingerprint. Literal searches find no additional known MAC/reversed-MAC/EUI-64
copy in the opaque portion. Encoded copies and other semantics remain unknown;
the low-level PHY implementation is distributed as a binary library.

These fields are writable stored copies, not immutable hardware identifiers.
`esp_phy_erase_cal_data_in_nvs()` erases the entire `phy` namespace, including
calibration keys; subsequent calibration can recreate them. Updating/deleting
a live NVS value does not establish erasure of historical physical records.
Probe builds disable PHY calibration storage; restoring original flash preserves
the original records.
No deletion was performed.

For persistent removal of these stored copies, disable
`CONFIG_ESP_PHY_CALIBRATION_AND_DATA_STORAGE` before PHY initialization and
handle old NVS records explicitly. Merely patching a live MAC blob can cause
the SDK's factory-MAC validation to reject it and recreate calibration records.
NVS setters/erase operations require `nvs_commit()`; logical deletion alone
does not erase old physical bytes. Radio address overrides do not change the
factory getter used by this storage path.

The [whole-flash coverage audit](results/saved-flash-coverage-2026-10-26/README.md)
accounts for all 8 MiB of each image: exact known bootloader/table/application
artifacts, the inspected NVS region, and FF-only gaps. All 756 NVS slots per
image match their raw parser records; empty slots are FF. No additional
persistent identity store was identified. This is structural coverage, not a
claim that compiled artifacts or opaque PHY payloads cannot contain identifying
information. The PHY blob's known factory-MAC copy is already identifying.

## Original application identities

All five compared application images match `ble-spoof-test`, version 1,
ESP-IDF 5.4.1. An existing local application binary matches their application
bytes; its ELF matches the descriptor digest. Inspection of that binary,
adjacent source/configuration and selected compiled behavior identifies these
configuration surfaces. The table distinguishes source facts from the later
targeted live-memory observations:

| Surface | Original configuration | Modification method |
| --- | --- | --- |
| BLE address | Public SDK-derived Bluetooth address; no authored override found. | For this NimBLE application, `ble_hs_id_set_rnd()` configures a random address; pass the matching own-address type to `ble_gap_adv_start()` while advertising is stopped. A base/interface override is a separate pre-initialization option. Neither rewrites the factory MAC. |
| GAP name and scan response | `Echo Pop`, appearance and automatic TX-power field. | Change the name/fields through NimBLE's `ble_svc_gap_device_name_set()` and advertising/scan-response setters; rebuild configuration for a persistent default. |
| Advertised service/manufacturer data | Fixed services `0xFE03`/`0xFE15`, Amazon company ID `0x0171` and placeholder manufacturer payload. These fingerprint the application, not a documented unique unit. | Replace fields with `ble_gap_adv_set_fields()` / `ble_gap_adv_rsp_set_fields()` while advertising is stopped; persistent defaults reside in firmware. |
| Bond-store backend | Requests bonding and encryption/identity-key distribution. NVS bond persistence is disabled; all five original ELF store callbacks are null and no store-config implementation is linked. | No configured RAM or NVS bond-store backend is established. A bonding request does not prove that storage succeeds; peer-side records and active connection state are separate. No keys were erased. |
| Local IRK, 128 bits | Startup attempts a local IRK even with public-address advertising. Two reset-separated [live captures](results/nimble-live-2026-10-26/README.md) contain nonzero default/current buffers, equal within each boot and different between boots. The live key-provider pointer is null. This is observed volatile variation, without an entropy or population-uniqueness result. | Supply an IRK through public `ble_hs_cfg.store_gen_key_cb` for `BLE_STORE_GEN_KEY_IRK` before host startup. Return zero to select supplied bytes; an absent provider or nonzero callback return falls back to the default loader. The private privacy setter is not the recommended public API. Chip reset clears this build's volatile state; the two observed resets produced different buffers. |
| Secure Connections public key | Selected code lazily generates a P-256 pair and caches its 64-byte public key/32-byte private scalar until SC initialization clears the flag. Fixed debug keys are disabled. Both live snapshots have a clear generation flag; no public key was read and no pairing was induced. Pairing can expose the public key and correlate exchanges within that lifetime. | Chip reset, or reinitialization executing `ble_sm_sc_init()`, clears the generation flag; the next eligible pairing attempts fresh generation. An ordinary NimBLE host reset does not clear it. No public arbitrary key-pair setter was identified. Re-pairing without clearing the cache does not necessarily change this public key. |
| Negotiated link/peer material | Pairing may create connection security state and exchange peer identity material. Actual pairing/history was not observed; the configured store callbacks provide no persistence backend. | Fresh pairing creates negotiated material. Persistent retention requires explicit store configuration; changing the advertising address alone does not clear active security state or a peer's records. |

The inspected application starts connectable advertising on host synchronization
and restarts after disconnect. Its GATT writes go to RAM and console logging;
the matched configuration selects native USB Serial/JTAG for that console.
Custom GATT reads return empty data, with no authored Serial Number/System ID
characteristic found. No authored Wi-Fi credential/certificate provisioning path
was found. The adjacent source was not rebuilt to prove complete source-to-binary equivalence.
This review does not establish credential absence in every saved byte or the
board's present physical boot state. Application names, services, credentials
and payloads are changeable independently of immutable factory fields.

Fresh receiver HCI reports match this application's expected public address,
advertised services and scan-response name. The later controlled A/B session
uses the existing Bluedroid probe, with separate successful SDK/readback and
receiver witnesses. These observations do not authenticate every source branch
or qualify the original application's bonding behavior.

Two targeted debug snapshots additionally confirm all five live key/store
callbacks are null, alongside the volatile observations above. Each snapshot
matches the complete original application descriptor and synchronized host
state. Active debug access affects execution; the final normal reset sequence
releases the chip for another boot whose keys were not captured.

## Evidence and limits

The [result ledger](results/README.md) owns actual observations and retained
failures: baseline inventory, runtime tables, ROM USB, AP presence/restart,
beacon fields, saved metadata audits, bounded receiver transport and BLE HCI
presence, plus whole-array coverage and source-backed identity methods.
Station emitted-address changes remain unverified; the BLE result's HCI observer
does not provide raw RF CRC.

The [read-only procedure](experiments/read-only-inventory.md) and
[API contract](docs/api.md) define capture effects and validation. Unique values,
raw enumerations, packets and backups stay in ignored private storage. There is
no claim that address changes make this board unidentifiable to every observer.

## Primary sources

Investigations pin ESP-IDF 5.5.6 revision
`880a203ae603d08a94d06aa01e0d6761958e5e9e` and espefuse 4.12.0:

- [C6 eFuse fields](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/efuse/esp32c6/esp_efuse_table.csv),
  [RS programming checks](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/efuse/esp32c6/esp_efuse_utility.c#L185),
  [C6 TRM eFuse chapter](https://www.espressif.com/sites/default/files/documentation/esp32-c6_technical_reference_manual_en.pdf).
- [Base/custom MAC implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_hw_support/mac_addr.c),
  [Wi-Fi API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_wifi/include/esp_wifi.h),
  [BLE GAP API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/bt/host/bluedroid/api/include/api/esp_gap_ble_api.h),
  [802.15.4 API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/ieee802154/include/esp_ieee802154.h).
- [Native USB descriptors](https://docs.espressif.com/projects/esp-iot-solution/en/latest/usb/usb_overview/usb_device_const_COM.html).
- [Espressif's hardware-descriptor explanation](https://github.com/espressif/esp-idf/issues/18516#issuecomment-4316973953),
  [C6 fixed-function controller guide](https://docs.espressif.com/projects/esp-idf/en/v5.4.1/esp32c6/api-guides/usb-serial-jtag-console.html).
  The vendor reply and register/source review support the absence of a supported
  software route; they do not prove absolute immutability against undocumented mechanisms.
- [Espressif optional-ID explanation](https://github.com/espressif/esp-idf/issues/8819)
  and [RainMaker self-claim implementation](https://github.com/espressif/esp-rainmaker/blob/f57a5ec62d6430a4f517de39a9121fc533f7f17d/components/esp_rainmaker/src/core/esp_rmaker_claim.c#L257).
  The explanation discusses S3 and the shared scheme; it is not a C6 production
  uniqueness test or evidence that this board runs RainMaker.
- [CP2102N datasheet](https://www.silabs.com/documents/public/data-sheets/cp2102n-datasheet.pdf),
  [AN976 configuration API](https://www.silabs.com/documents/public/application-notes/an976-cp2102-3-4-9-to-cp2102n-porting-guide.pdf),
  [AN978 fields/checksum](https://www.silabs.com/documents/public/application-notes/an978-cp210x-usb-to-uart-api-specification.pdf),
  [AN721 configurator](https://www.silabs.com/documents/public/application-notes/AN721.pdf).
- [Vendor-authored CP2102N tool source, pinned third-party SDK archive](https://github.com/mschuldt/_efm8/blob/56d116793833e9be86355a63bc9675489f6c2e38/USBXpressHostSDK/CP210x/srcpkg/cp210xsmt.tar.gz):
  `cp210xsmt.cpp` defines unlocked `FF` and its GetConfig-only lock classifier.
  The inspected archive is a mirror; direct official-package retrieval was not verified.
- [XMC C flash](https://www.xmcwh.com/uploads/800/XM25QH64C_Ver1.7.pdf),
  [XMC D flash](https://www.xmcwh.com/uploads/916/XM25QH64D_Ver1.2.pdf).
- [Board power/reset schematic, sheet 2](https://dl.espressif.com/dl/schematics/esp32-c6-devkitc-1-schematics_v1.4.pdf#page=2),
  [selected Mac reset implementation](https://github.com/libusb/libusb/blob/v1.0.30/libusb/os/darwin_usb.c#L2140).
- [Module marking/Data Matrix format](https://docs.espressif.com/projects/esp-packaging/en/latest/esp32c6/esp-packaging-en-master-esp32c6.pdf#page=12),
  [eFuse read API](https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32c6/api-reference/system/efuse.html).
- Original application ESP-IDF 5.4.1:
  [PHY calibration layout](https://github.com/espressif/esp-idf/blob/v5.4.1/components/esp_phy/include/esp_phy_init.h),
  [PHY NVS store/erase behavior](https://github.com/espressif/esp-idf/blob/v5.4.1/components/esp_phy/src/phy_init.c),
  [factory getter/EUI-64 expansion](https://github.com/espressif/esp-idf/blob/v5.4.1/components/esp_hw_support/mac_addr.c).
- NimBLE [identity API](https://mynewt.apache.org/latest/network/ble_hs/ble_hs_id.html)
  and [GAP advertising API](https://mynewt.apache.org/latest/network/ble_hs/ble_gap.html);
  installed headers remain authoritative for an application build.
- Original NimBLE revision `a9a4c701981b86c91fdc63fce4cb7bc05c69caac`:
  [IRK provider contract](https://github.com/espressif/esp-nimble/blob/a9a4c701981b86c91fdc63fce4cb7bc05c69caac/nimble/host/include/host/ble_store.h#L283-L300),
  [startup/privacy state](https://github.com/espressif/esp-nimble/blob/a9a4c701981b86c91fdc63fce4cb7bc05c69caac/nimble/host/src/ble_hs_pvcy.c#L230-L294),
  [IRK provider dispatch](https://github.com/espressif/esp-nimble/blob/a9a4c701981b86c91fdc63fce4cb7bc05c69caac/nimble/host/src/ble_hs_startup.c#L471-L485),
  [SC key cache](https://github.com/espressif/esp-nimble/blob/a9a4c701981b86c91fdc63fce4cb7bc05c69caac/nimble/host/src/ble_sm_sc.c#L175-L195),
  [ordinary host-reset scope](https://github.com/espressif/esp-nimble/blob/a9a4c701981b86c91fdc63fce4cb7bc05c69caac/nimble/host/src/ble_hs.c#L398-L420).
