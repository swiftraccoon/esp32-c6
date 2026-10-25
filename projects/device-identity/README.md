# Device identity and privacy

[Workspace](../../README.md) · [Projects](../README.md)

Status: documented investigation;
no identifier changes or anonymity claims have been tested.

Investigate which identifiers the **ESP32-C6-DevKitC-1-N8**, including both
USB-C interfaces, exposes; which can be configured or rotated; and which
observers can still correlate the device. The objective is measurable reduction
of identifying information. “Never identifiable” is not an achievable guarantee
from MAC randomization alone.

## Identifier inventory

| Identifier | Exposure | Configuration and limits |
| --- | --- | --- |
| Factory base MAC in C6 eFuse | Readable by software/debug tools; source for default interface addresses | Runtime base/interface overrides are supported. They do not erase the factory value. Set overrides before initializing interfaces to avoid a brief default-address transmission. |
| Wi-Fi station MAC and SoftAP BSSID | On-air Wi-Fi transmissions, including discovery/association or beacons when enabled | Locally administered unicast addresses can be selected. Address lifetime, collisions, reconnects and information elements need testing. |
| BLE public/identity address | BLE packets when used; pairing relationships can preserve identity | Random address and privacy APIs exist. Resolvable private addresses intentionally remain resolvable by peers with the identity resolving key. Advertising data, names and services can independently identify an application/device. |
| IEEE 802.15.4 extended address/EUI-64 | Frames containing an extended source/destination address | Driver has an extended-address setter. Default derivation uses factory MAC/extension data. Thread/Zigbee may manage their own identities and persistence. |
| IEEE 802.15.4 short address, PAN ID; higher-layer network identities | Depending on transmitted MAC/network fields | These are network/protocol state, not immutable chip serial numbers. Rotation can break network membership and does not independently hide identity. |
| Native USB Serial/JTAG serial string | Attached USB host, including enumeration before application startup | Espressif documents a ROM descriptor serial derived from the MAC. A runtime radio MAC override is not evidence this USB identity changes. No supported customization method has been established here. |
| CP2102N bridge serial string | Host attached to the USB-to-UART port | Independent of C6 eFuses. Silicon Labs documents a default unique 128-bit serial and programmable descriptors, unless configuration is locked. This board's lock state has not been inspected. |
| Optional 128-bit eFuse unique ID | Software/debug access; not automatically an on-air field | Defined by the pinned C6 eFuse table. Population on this board is unknown. Radio address overrides do not modify it. |
| Model/revision identifiers | USB VID/PID/product/version, chip/package revision, potentially protocol capability fields | Usually identify a product class rather than a unique board. Native USB is fixed-function; CP2102N descriptor customization is separate. |
| Application and stored protocol identities | Names, service UUIDs, hostnames, DHCP/mDNS data, certificates, pairing keys, Matter/Thread/Zigbee provisioning and application payloads when enabled | Configuration and persistence must be audited per firmware. There is no finite inventory covering arbitrary future applications. |
| SPI flash identification | Local software/programmer access | Manufacturer/device ID identifies flash type. A separate unique flash ID depends on the fitted part and command support; neither has been qualified for this board. |

No actual per-board identifier values are published here. Factory security keys
are not presumed to be publicly readable or broadcast. Physical-layer traits,
timing, payload continuity and observations from multiple interfaces may still
permit correlation; that is a separate experimental question, not a claim that
any particular observer can always fingerprint this board.

## Investigation order

1. Produce a read-only inventory with unique values kept in ignored `private/`
   storage; distinguish factory values, configured values and observed bytes.
2. Define observers separately: nearby passive receiver, joined-network peer,
   paired BLE peer, attached USB host, and physical/debug access.
3. Test runtime overrides and rotation on each radio, including cold boot,
   reset, reconnect and recovery. Record persistence and collateral identifiers.
4. Observe native USB enumeration before/after runtime changes. Inspect CP2102N
   configuration and lock state without writing it.
5. Test the intended passive recorder with independent receivers. Software
   settings alone do not prove absence of RF transmissions.

No eFuse programming, descriptor changes, bonding reset or identity rotation
has been implemented. Prefer reversible runtime configuration for experiments;
any later irreversible provisioning needs its own concrete plan and authorization.

## Sources and evidence boundaries

Reviewed 2026-10-24. Firmware investigations use the existing pinned ESP-IDF
5.5.6 revision `880a203ae603d08a94d06aa01e0d6761958e5e9e`, not an implicit upgrade
to the current web documentation:

- [Pinned MAC implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_hw_support/mac_addr.c)
  and [system guide](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-reference/system/misc_system_api.rst).
- [Pinned C6 eFuse fields](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/efuse/esp32c6/esp_efuse_table.csv),
  including `MAC`, `MAC_EXT`, and `OPTIONAL_UNIQUE_ID`.
- [Pinned BLE GAP API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/bt/host/bluedroid/api/include/api/esp_gap_ble_api.h)
  and [802.15.4 API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/ieee802154/include/esp_ieee802154.h).
- [Espressif USB descriptor reference](https://docs.espressif.com/projects/esp-iot-solution/en/latest/usb/usb_overview/usb_device_const_COM.html).
- [Silicon Labs CP2102N datasheet](https://www.silabs.com/documents/public/data-sheets/cp2102n-datasheet.pdf).

These sources establish documented fields/APIs, not the exact configuration,
read protection, bridge lock state, or on-air behavior of this individual board.
