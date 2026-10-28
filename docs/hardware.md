# ESP32-C6 hardware inventory

Board model: **ESP32-C6-DevKitC-1-N8 with two USB-C ports**. The model is
reported; the PCB revision has not been inspected. Silicon, flash and bridge
observations below were recorded on 2026-10-22 in America/New_York using
esptool 5.1.0.

| Property | Observed value |
| --- | --- |
| Serial device | `/dev/cu.SERIAL_PORT` |
| USB bridge | Silicon Labs CP2102N, VID `10c4`, PID `ea60` |
| SoC | ESP32-C6, QFN40, revision 0.2 |
| Flash | 8 MB; manufacturer `20`, device `4017` as printed by esptool |
| Crystal | 40 MHz |
| Secure boot | Disabled |
| Flash encryption | Disabled; SPI boot crypt count `0x0` |
| Existing firmware | Fresh pre-BLE-probe 8 MiB image restored and separately full-range digest-verified; verification and warning-free host reset completed by 2026-10-26 02:11:46 UTC; postboot health/persistence unmeasured |
| Board model | ESP32-C6-DevKitC-1-N8; reported model, PCB revision unverified |
| Board PCB revision | Not yet recorded; separate from chip revision 0.2 |

The part number matches Espressif's DevKitC-1 N8 variant: ESP32-C6-WROOM-1,
8 MB flash, onboard PCB antenna and no PSRAM. These are documented board
features, not additional hardware qualifications.
[Board guide and ordering table](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html)

## Board ports and onboard fixtures

| Board feature | Routing / use | Local evidence boundary |
| --- | --- | --- |
| USB-C to UART port | CP2102N bridge to UART0 TX GPIO16 / RX GPIO17; power, flashing and console | The bridge/UART route was used by the existing tests |
| Native ESP32-C6 USB-C port | GPIO13 D+ / GPIO12 D-; USB 2.0 full speed, 12 Mb/s signaling; CDC-ACM serial, flashing and JTAG | Mac commands and passive radio capture observed; iPhone data access unqualified |
| RGB LED | Addressable WS2812B on GPIO8 | Available for a future RMT/LED check; not exercised by the retained benchmarks |
| BOOT and RESET buttons | BOOT pulls GPIO9 low; RESET controls chip enable | Documented controls; GPIO9 interrupt/preemption behavior remains untested |
| J1/J3 headers | GPIO, UART, native USB signals, 5 V, 3.3 V and ground | Use the published pin map and account for onboard loads and strapping pins |
| J5 jumper | Module-current measurement point; fitted jumper powers the module normally | Available fixture for later power work; no current measurement established |

Port/header descriptions come from the
[board guide](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html);
button, bridge and LED wiring are also shown in the
[v1.4 reference schematic, sheet 2](https://dl.espressif.com/dl/schematics/esp32-c6-devkitc-1-schematics_v1.4.pdf).
The schematic is a reference, not an identification of our PCB revision.

The board guide permits power through either or both USB ports. Its other
power arrangements, 5 V headers or 3.3 V headers, are separate mutually
exclusive options. Record the actual supplies for each test. The board has
a regulator, power LED and UART bridge, so whole-board current differs from
module current through J5.

The native controller is fixed to serial/JTAG; two connectors do not add USB
OTG, arbitrary USB classes or a second radio. See the
[iPhone compatibility investigation](../projects/wireless-explorer/docs/iphone-usb.md).
[C6 USB controller](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-guides/usb-serial-jtag-console.html)

For historical reproduction, retain the original port assumptions. In
particular, the [RTIC v1 fixture](../projects/os-evaluation/docs/experiments/rtic-qualification.md) requires
the UART bridge with native USB disconnected because its automatic logger can
change transport. Identifying the second connector does not change that frozen
contract or establish the cause of its failed boot.

Earlier dated build/review/experiment records describe the carrier as unknown
at collection time. This identification supersedes that uncertainty for future
planning; it does not rewrite those records or promote their coverage outcomes.

## Chip capabilities that affect comparisons

The C6 has a 160 MHz maximum HP RISC-V CPU, a separate 20 MHz LP core, 512 KB HP SRAM, and 16 KB LP SRAM. The LP core is a distinct low-power subsystem rather than a second interchangeable application core. The radio supports 2.4 GHz Wi-Fi, BLE, and IEEE 802.15.4. Its USB peripheral is Serial/JTAG, so general USB device or host examples need separate hardware. These are silicon capabilities; an OS must provide the corresponding drivers. [Espressif datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.html)

The 8 MB flash is not RAM. The N8 board has no PSRAM; use the C6's internal
SRAM budget when selecting stacks and capture buffers.

## Wireless reference

Reviewed 2026-10-23. These are documented silicon and SDK capabilities, not
radio qualifications of our board. The 2026-10-24 board identification supplies
the carrier and PCB-antenna specification; our retained benchmark program has
not qualified radio operation or range.
Wireless explorer recorded UART control, passive Wi-Fi surveys and
[Wi-Fi/BLE/IEEE receive windows on UART and Mac native USB](../projects/wireless-explorer/results/observer-qualification-2026-10-24/image-03-results.md)
on 2026-10-24. Reception after repeated mode changes was observed; controlled
RF fidelity, RF silence, iPhone operation and advanced PHY coverage remain
unqualified. The OS study's historical outcomes remain unchanged.
The later [controlled AP witness](../projects/device-identity/results/wifi-ap-2026-10-25/README.md)
observed 64 CRC-valid nonce-tagged beacons matching a pre-start driver AP-MAC
control across two reset-separated boots, using an independent receiver. This
qualifies presence for that fixed control, without range, full-window, silence,
hot-rotation or correlation-reduction coverage.
The [later restart witness](../projects/device-identity/results/wifi-restart-2026-10-25/README.md)
observed 120 exact CRC-valid target beacons across two A-to-B sessions, each
retaining one driver/UART and restoring the original driver MAC after each
checked Stop. The same SSID links phases; transition coverage, RF silence,
hot changes while running and correlation reduction remain unqualified.
The hardware baseline is the [C6 datasheet v1.5, sections 4.3 and 6](https://documentation.espressif.com/esp32-c6_datasheet_en.html).
SDK claims below use the already cached **ESP-IDF 5.5.6**, commit
[`880a203`](https://github.com/espressif/esp-idf/tree/880a203ae603d08a94d06aa01e0d6761958e5e9e),
unless another version is named. A newer live manual does not upgrade our
firmware or establish a Rust API for its features.

### Radio hardware and operating modes

| Radio | Documented capability | Boundary |
| --- | --- | --- |
| Wi-Fi | 2.4 GHz, one transmit/receive chain; 802.11b/g/n/ax. Station, SoftAP, station plus SoftAP, scanning and promiscuous reception. | Wi-Fi 6 uses 20 MHz in non-AP mode. Legacy 802.11n supports 20/40 MHz; the advertised overall PHY ceiling is 150 Mbit/s, not application goodput. No 5/6 GHz operation. |
| Bluetooth LE | Bluetooth LE 5.3 certified; 1M/2M PHY and coded 125/500 kbit/s modes; extended advertising, multiple advertising sets and LE power control. | LE only: no Bluetooth Classic BR/EDR or Classic SPP/A2DP. Application roles/profiles need a host stack. |
| IEEE 802.15.4 | 2.4 GHz O-QPSK, 250 kbit/s, compliant with the 2015 specification. | Raw frames need a driver; Thread and Zigbee add separate networking stacks. |

These rates are PHY signaling rates. Protocol overhead, retries, scheduling,
peer configuration and shared-radio contention reduce useful payload rates.

The [pinned C6 Wi-Fi feature list](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-guides/wifi.rst#L45)
also documents OFDMA, downlink MU-MIMO participation, BSS coloring, individual
and broadcast Target Wake Time, modem sleep, WPA2/WPA3 and enterprise security,
and provisioning through WPS/DPP. These require the appropriate peers and
configuration. Single-chain MU-MIMO participation does not make this a
multi-stream radio. Station and SoftAP do not provide independent radio channels.

For sensing and diagnostics, the SDK exposes promiscuous packets, channel-state
information (CSI), and 802.11mc Fine Timing Measurement (FTM) ranging. The
[pinned Wi-Fi guide](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-guides/wifi.rst#L2104)
describes FTM initiator in station mode and responder in AP mode. The older
initiator limitation is fixed on our revision 0.2, as recorded in the
[errata audit](research/rev02-errata-workarounds.md). Neither that fix nor FTM's
presence establishes distance accuracy on this board.

### Protocols supplied by software

| Protocol or service | What the C6 can be used for | Required distinction/source |
| --- | --- | --- |
| ESP-NOW | Direct Espressif peer messages without joining an access point. | Uses the Wi-Fi radio; it is not a separate transceiver or automatic mesh-routing service. [Pinned guide](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-reference/network/esp_now.rst). |
| Espressif Long Range Wi-Fi | Vendor-specific low-rate links with compatible peers. | Separate from LoRa; range on this board's PCB antenna remains unmeasured. [Pinned LR description](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-guides/wifi.rst#L1692). |
| ESP-WIFI-MESH | Multi-hop networking over Wi-Fi using Espressif's stack. | Distinct from Bluetooth Mesh, Thread and Zigbee. C6 has the SDK capability flag. [C6 capability source](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/include/soc/soc_caps.h#L626), [programming guide](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-reference/network/esp-wifi-mesh.rst). |
| BLE applications and Bluetooth Mesh | Advertising/beacons, central/peripheral connections, GATT services, provisioning and Bluetooth mesh nodes. | Needs a suitable host/profile stack. IDF offers NimBLE/Bluedroid and BLE Mesh; native Rust can use a separate host such as TrouBLE. [Rust layer audit](../projects/os-evaluation/docs/research/rust-support-audit.md), [IDF BLE overview](https://docs.espressif.com/projects/esp-idf/en/v6.1/esp32c6/api-guides/ble/overview.html). |
| Thread | IPv6 mesh nodes, router/end-device roles as configured, and radio-coprocessor use with a host. | OpenThread supplies the stack; the datasheet's Thread 1.3 label does not fix every software version. [IDF OpenThread](https://docs.espressif.com/projects/esp-idf/en/v6.1/esp32c6/api-guides/openthread.html), [pinned Rust binding evidence](../projects/os-evaluation/docs/research/rust-support-audit.md). |
| Zigbee | Coordinator, router and end-device applications through the Zigbee SDK. | A separate SDK and role configuration; raw 802.15.4 support alone is insufficient. [Zigbee introduction](https://docs.espressif.com/projects/esp-zigbee-sdk/en/latest/esp32c6/introduction.html). |
| Matter | Application devices over Wi-Fi or Thread, with BLE used for commissioning where configured. | Matter is an IP application stack, not another radio or a Zigbee transport. [ESP-Matter guide](https://docs.espressif.com/projects/esp-matter/en/latest/esp32c6/introduction.html). |

The IDF 6.1 BLE support table marks C6 **ESP-BLE-ISO and ESP-BLE-AUDIO
unsupported**; a Bluetooth version badge is not proof of LE Audio support.
Wi-Fi Aware/NAN is also unavailable in our pinned C6 SDK configuration:
its [Kconfig requires SOC_WIFI_NAN_SUPPORT](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_wifi/Kconfig#L574),
which the C6 capability header does not define. The chip has no integrated
LoRa/sub-GHz, cellular, NFC, UWB or GNSS radio; those require external hardware.

### Radio sharing and the OS decision

There is **one shared 2.4 GHz RF path**. Wi-Fi, BLE and 802.15.4 take turns;
they cannot independently receive or transmit at the same instant. The
[pinned coexistence matrix](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-guides/coexist.rst)
classifies selected pairs below. Its stability labels are vendor declarations,
not our measurements, and do not qualify every three-protocol combination.

| Combination in IDF 5.5.6 | Vendor classification |
| --- | --- |
| Wi-Fi station with BLE scan/advertising/connection | Supported/stable. |
| Wi-Fi station with Thread/Zigbee end device | Supported/stable. |
| Wi-Fi station with Thread/Zigbee router | Supported, performance unstable. |
| Wi-Fi SoftAP with Thread/Zigbee router | Unsupported. |
| Thread/Zigbee router with BLE scanning | Unsupported; BLE advertising/connection are listed supported. |

Espressif recommends separate SoCs/antennas for Wi-Fi Thread-border-router or
Zigbee-gateway products needing concurrent reception. The inspected Rust
esp-radio source is narrower still: it forbids enabling Wi-Fi and raw 802.15.4
together. See the [versioned Rust audit](../projects/os-evaluation/docs/research/rust-support-audit.md#wireless-support-is-layered)
and [compatible release inputs](../projects/os-evaluation/docs/research/platform-compatibility.md).

For OS selection, check silicon capability, selected driver, protocol role,
Rust API, resource ownership and actual board outcome separately. The
[hardware worksheet](../projects/os-evaluation/docs/hardware-coverage.md), [stack comparison](../projects/os-evaluation/docs/comparison.md#radio-evidence),
and [radio prior-work review](research/prior-work-c6-radio.md) already
cover those different questions; none is an on-board radio pass. Range,
goodput/loss, coexistence, energy per delivery and recovery remain unmeasured.

## Probe record

The following commands succeeded. They reset the chip through the bridge, used the ROM bootloader without uploading a flasher stub, and did not replace flash contents or change eFuses.

```sh
python3 -m serial.tools.list_ports -v
esptool --port /dev/cu.SERIAL_PORT --baud 115200 --no-stub --connect-attempts 2 flash-id
esptool --port /dev/cu.SERIAL_PORT --baud 115200 --no-stub --connect-attempts 2 get-security-info
```

[esptool command reference](https://docs.espressif.com/projects/esptool/en/latest/esp32c6/esptool/basic-commands.html)

## Host preparation status

The host is Apple Silicon macOS. Cargo, rustc, rustup, Python with pyserial,
Homebrew and esptool are available. During initial inspection, espflash/idf.py/
west were absent from PATH and the active compiler lacked a RISC-V target.
Implementation subsequently installed workspace-local espflash 4.6.0 and laze
0.1.39, selected Rust 1.96.0 with the C6 target for benchmarks, and cached IDF
5.5.6 through esp-idf-sys. Ariel's unmodified examples use the separately
installed Rust 1.99.1. No project-wide nightly toolchain is selected; IDF's
scoped build-std bootstrap is recorded in its build notes.

Before the first firmware replacement, save a local full-flash backup and its hash, record the partition layout, identify the board or map the relevant pins, and pin the selected SDK and Rust versions. Store flash backups outside Git: they may contain application credentials. A backup restores ordinary flash contents, not irreversible provisioning changes. Keep secure-boot and flash-encryption provisioning out of comparative benchmark runs.

The initial research phase only queried identity. During the [first implementation milestone](../projects/os-evaluation/results/initial-2026-10-22/README.md), a full backup was captured and digest-verified before any test firmware, then restored and verified after all 15 captures. The backup is Git-ignored at `backups/original-2026-10-22/flash-backup.bin`; retain it for future tests. The initial transfer needed 115200 baud; higher-speed read attempts failed.

Before the next tests, the full backup again matched the original device. After
two Ariel startup checks, 15 clock/message boots and five alignment-probe boots,
the full image was restored, separately digest-verified and hard-reset into the
original application. The [clock/message device state](../projects/os-evaluation/results/clocked-handoff-2026-10-23/device-state.json)
links the retained transfer evidence. New benchmarks read back CPU160/MSPI80/
flash80 configuration; this does not add an electrical frequency measurement.

The later [Ariel native tests](../projects/os-evaluation/results/ariel-native-tests-2026-10-23/RESULTS.md)
retain twelve expected outputs, with partial observation-window evidence and
the original operator pass reports. Their preflight found 3,979 bytes changed from
the historical backup in NVS sectors `0xa000`, `0xb000`, `0xc000`; the cause was
not established. A fresh 64 KiB prefix overlaid on the historical image produced
a current full-image backup, independently verified against the complete device
digest before writes. Private current and historical backups remain separate.

After those tests, the **current pre-session image**, including the newest
captured NVS state, was restored across all 8,388,608 bytes, independently
digest-verified and hard-reset at **2026-10-23 06:31:18 UTC**. The
[Ariel device-state record](../projects/os-evaluation/results/ariel-native-tests-2026-10-23/device-state.json)
and its transfer logs retain that evidence. The application's later boots may
update persistence again; a verified restoration describes the moment before
reset, not permanent equality with a historical flash hash. No eFuse provisioning
was performed.

After the three [failed Zephyr native attempts](../projects/os-evaluation/results/zephyr-native-tests-2026-10-23/RESULTS.md),
the newer current pre-session image was restored across all **8,388,608 bytes**,
separately digest-verified and hard-reset at **2026-10-23 06:46:46 UTC**. The
[latest device-state record](../projects/os-evaluation/results/zephyr-native-tests-2026-10-23/device-state.json)
retains the logs and private backup identity, SHA-256
`863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
No new image was flashed to repair the acceptance or image-format findings.
The separate Rust probe produced no final firmware and opened no serial port.

After the [IDF accounting qualification](../projects/os-evaluation/results/idf-accounting-qualification-2026-10-23/RESULTS.md),
the same current pre-session state was restored across all **8,388,608 bytes**,
separately digest-verified and hard-reset via RTS. Completion was observed at
**2026-10-23 12:43:33 UTC**; the
[device-state record](../projects/os-evaluation/results/idf-accounting-qualification-2026-10-23/device-state.json)
retains the logs and private backup identity. No eFuse provisioning occurred.
This verifies the restoration boundary; subsequent application persistence
changes remain possible.

After the [IDF finite-load qualification](../projects/os-evaluation/results/idf-load-qualification-2026-10-23/RESULTS.md),
the same existing full-flash backup was restored across all **8,388,608 bytes**.
A separate verification matched its digest and then hard-reset via RTS;
completion was observed at **2026-10-23 15:29:34 UTC**. The
[load device-state record](../projects/os-evaluation/results/idf-load-qualification-2026-10-23/device-state.json)
retains the transfer and verification logs. Before these writes, the backup had
been digest-verified against the device rather than freshly read again. The
verification boundary does not establish post-boot persistence constancy or
deployed recovery behavior.

After the [failed RTIC attempt](../projects/os-evaluation/results/rtic-qualification-2026-10-23/RESULTS.md),
the complete pre-session image was restored across **8,388,608 bytes**, separately
digest-verified, then hard-reset via RTS. Completion was observed at
**2026-10-23 16:59:16 UTC**. The
[RTIC device-state record](../projects/os-evaluation/results/rtic-qualification-2026-10-23/device-state.json)
retains its separate private backup identity and public transfer/verification
logs. The backup was verified against the device before writes. No new RTIC
attempt followed the failure; subsequent original-application persistence
changes remain possible after reset.

After the [wireless recorder checks](../projects/wireless-explorer/results/observer-qualification-2026-10-24/image-03-results.md)
on 2026-10-24, the fresh original **8,388,608-byte** backup was restored,
separately digest-verified in the bootloader and released by a hard reset.
Mac native USB commands and repeated radio receive windows were observed with
both USB ports connected. A native startup failure remains recorded; phone
power/data and automatic reconnect remain unqualified. Raw captures, USB
identifiers, full images and transfer diagnostics stay in ignored private storage.

After the [device identity table probe](../projects/device-identity/results/runtime-table-2026-10-24/README.md),
the fresh original **8,388,608-byte** image was restored and independently
digest-verified before reset at **2026-10-25 03:15:53 UTC**. Hard-reset completion
was observed at **03:15:54 UTC**; subsequent application persistence remains
unmeasured. The [device-state record](../projects/device-identity/results/runtime-table-2026-10-24/device-state.json)
retains the scoped boundary. Two SDK table-control sessions passed after a
retained zero-RX startup attempt. Explicit DTR/RTS deassertion permits UART
startup on this bridge; opening can start/reset the C6 and must not be assumed
to preserve prior RAM state. Current USB serial properties stayed unchanged
during table overrides; fresh descriptor transfers and RF identity changes
remain unqualified. Raw identities, originals and diagnostics stay private.

After the [controlled AP witness](../projects/device-identity/results/wifi-ap-2026-10-25/README.md),
the fresh complete **8,388,608-byte** image was restored and a separate device-MD5
range comparison verified it at **2026-10-25 08:52:37 UTC** before release. The
[restoration record](../projects/device-identity/results/wifi-ap-2026-10-25/restoration.json)
separates that full-range digest check from a second full-file readback and from
host reset-sequence completion. Physical postboot health, later persistence and
radio activity remain unmeasured. Temporary segmented writes avoided original
stored-data erase ranges; the complete restore returned all stored data to the
fresh backup bytes before reset. No eFuse programming or persistent
bridge-configuration programming occurred.

After the [two-phase AP restart witness](../projects/device-identity/results/wifi-restart-2026-10-25/README.md),
the fresh complete **8,388,608-byte** image was restored and a separate
device-MD5 range comparison verified it at **2026-10-25 19:48:14 UTC** before
host reset release. The [restoration record](../projects/device-identity/results/wifi-restart-2026-10-25/restoration.json)
retains exact verification/reset times and segmented erase-range evidence.
All twelve hardware command phases exited zero. This full-range digest
verification is not a second full-file readback or device attestation; physical
postboot health, later persistence and radio activity remain unmeasured.

After the [BLE HCI address witness](../projects/device-identity/results/ble-identity-2026-10-26/README.md),
the fresh complete **8,388,608-byte** image was restored and separately verified
against the entire device range, preserving raw flash header bytes. Verification
and warning-free host reset completed by **2026-10-26 02:11:46 UTC**. The
[restoration record](../projects/device-identity/results/ble-identity-2026-10-26/restoration.json)
retains that boundary. Both random address/nonce controls have independent HCI
report witnesses; raw RF CRC, silence, independent application boot and later
persistence remain unmeasured. No eFuse or persistent bridge programming occurred.

The later [original NimBLE live-state check](../projects/device-identity/results/nimble-live-2026-10-26/README.md)
read two reset-separated selected RAM snapshots through the native debug
interface. Both confirmed resume; an independent matched ROM check and
warning-free normal hard-reset sequence completed by **2026-10-26 19:21:21 UTC**.
No firmware replacement, flash-array programming, eFuse burn or bridge
configuration command was issued. Debug access affects execution, watchdogs,
interrupts and registers; the final reset releases the chip for another boot.
Its keys, application health, RF behavior and subsequent PHY persistence were
not captured. The preceding full-flash restoration remains historical.

The later [bridge configuration trials](../projects/device-identity/results/bridge-config-2026-10-26/README.md)
retain three failed serial-change attempts. The third reads back an exact
678-byte custom configuration, while the fresh active USB serial remains
original after the selected Mac SDK reset. Both attempts that programmed the
bridge restored and independently replayed every original configuration byte
and both original serials; update-enable remains `FF`. Every attempt completed
its matched, warning-free normal C6 reset sequence. No restoration remains
pending. A full bridge power-cycle activation/persistence check was not performed.
No C6 firmware programming, eFuse burn or bridge lock was requested; application
health and later PHY persistence remain unmeasured.

The [delayed-reset bridge check](../projects/device-identity/results/bridge-timing-2026-10-27/README.md)
on **2026-10-27 UTC** retains one further failed serial-change trial. Five
declared host gaps each exceed ten seconds. The candidate configuration reads
back exactly, while the serial response remains original. Every original byte
and both serials were restored and independently replayed; the matched,
warning-free normal C6 reset completed. No restoration remains pending. This
setting does not qualify full bridge power-cycle activation, physical reset
effect, later application health or PHY persistence.
