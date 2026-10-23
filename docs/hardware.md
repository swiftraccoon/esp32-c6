# Connected ESP32 C6 hardware

Observed on 2026-10-22 in America/New_York, using esptool 5.1.0. The development board model and exposed pinout remain unidentified; the chip and flash were queried directly.

| Property | Observed value |
| --- | --- |
| Serial device | `/dev/cu.SERIAL_PORT` |
| USB bridge | Silicon Labs CP2102N, VID `10c4`, PID `ea60` |
| SoC | ESP32-C6, QFN40, revision 0.2 |
| Flash | 8 MB; manufacturer `20`, device `4017` as printed by esptool |
| Crystal | 40 MHz |
| Secure boot | Disabled |
| Flash encryption | Disabled; SPI boot crypt count `0x0` |
| Existing firmware | Current pre-Zephyr 8 MB state restored, separately digest-verified and reset on 2026-10-23; later persistence may change again |
| Board model | Unknown; do not assume a DevKitC pinout |

These observations identify one physical test device. Upstream DevKitC board definitions are starting points; LED pins, flash configuration, UART routing, and external wiring must match this board before those examples are used.

## Chip capabilities that affect comparisons

The C6 has a 160 MHz maximum HP RISC-V CPU, a separate 20 MHz LP core, 512 KB HP SRAM, and 16 KB LP SRAM. The LP core is a distinct low-power subsystem rather than a second interchangeable application core. The radio supports 2.4 GHz Wi-Fi, BLE, and IEEE 802.15.4. Its USB peripheral is Serial/JTAG, so general USB device or host examples need separate hardware. These are silicon capabilities; an OS must provide the corresponding drivers. [Espressif datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.html)

The 8 MB flash observation does not imply 8 MB of RAM. Do not plan on PSRAM without separate hardware evidence.

## Wireless reference

Reviewed 2026-10-23. These are documented silicon and SDK capabilities, not
radio qualifications of our board. Its antenna/carrier is still unidentified;
our retained benchmark program has not qualified radio operation or range.
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
| Espressif Long Range Wi-Fi | Vendor-specific low-rate links with compatible peers. | Separate from LoRa; no guaranteed range for our unidentified antenna/board. [Pinned LR description](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-guides/wifi.rst#L1692). |
| ESP-WIFI-MESH | Multi-hop networking over Wi-Fi using Espressif's stack. | Distinct from Bluetooth Mesh, Thread and Zigbee. C6 has the SDK capability flag. [C6 capability source](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/include/soc/soc_caps.h#L626), [programming guide](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-reference/network/esp-wifi-mesh.rst). |
| BLE applications and Bluetooth Mesh | Advertising/beacons, central/peripheral connections, GATT services, provisioning and Bluetooth mesh nodes. | Needs a suitable host/profile stack. IDF offers NimBLE/Bluedroid and BLE Mesh; native Rust can use a separate host such as TrouBLE. [Rust layer audit](research/rust-support-audit.md), [IDF BLE overview](https://docs.espressif.com/projects/esp-idf/en/v6.1/esp32c6/api-guides/ble/overview.html). |
| Thread | IPv6 mesh nodes, router/end-device roles as configured, and radio-coprocessor use with a host. | OpenThread supplies the stack; the datasheet's Thread 1.3 label does not fix every software version. [IDF OpenThread](https://docs.espressif.com/projects/esp-idf/en/v6.1/esp32c6/api-guides/openthread.html), [pinned Rust binding evidence](research/rust-support-audit.md). |
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
together. See the [versioned Rust audit](research/rust-support-audit.md#wireless-support-is-layered)
and [compatible release inputs](research/platform-compatibility.md).

For OS selection, check silicon capability, selected driver, protocol role,
Rust API, resource ownership and actual board outcome separately. The
[hardware worksheet](hardware-coverage.md), [stack comparison](comparison.md#radio-evidence),
and [radio prior-work review](research/prior-work-c6-radio-runtimes.md) already
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

The initial research phase only queried identity. During the [first implementation milestone](../results/initial-2026-10-22/README.md), a full backup was captured and digest-verified before any test firmware, then restored and verified after all 15 captures. The backup is Git-ignored at `backups/original-2026-10-22/flash-backup.bin`; retain it for future tests. The initial transfer needed 115200 baud; higher-speed read attempts failed.

Before the next tests, the full backup again matched the original device. After
two Ariel startup checks, 15 clock/message boots and five alignment-probe boots,
the full image was restored, separately digest-verified and hard-reset into the
original application. The [clock/message device state](../results/clocked-handoff-2026-10-23/device-state.json)
links the retained transfer evidence. New benchmarks read back CPU160/MSPI80/
flash80 configuration; this does not add an electrical frequency measurement.

The later [Ariel native tests](../results/ariel-native-tests-2026-10-23/RESULTS.md)
retain twelve expected outputs, with partial observation-window evidence and
the original operator pass reports. Their preflight found 3,979 bytes changed from
the historical backup in NVS sectors `0xa000`, `0xb000`, `0xc000`; the cause was
not established. A fresh 64 KiB prefix overlaid on the historical image produced
a current full-image backup, independently verified against the complete device
digest before writes. Private current and historical backups remain separate.

After those tests, the **current pre-session image**, including the newest
captured NVS state, was restored across all 8,388,608 bytes, independently
digest-verified and hard-reset at **2026-10-23 06:31:18 UTC**. The
[Ariel device-state record](../results/ariel-native-tests-2026-10-23/device-state.json)
and its transfer logs retain that evidence. The application's later boots may
update persistence again; a verified restoration describes the moment before
reset, not permanent equality with a historical flash hash. No eFuse provisioning
was performed.

After the three [failed Zephyr native attempts](../results/zephyr-native-tests-2026-10-23/RESULTS.md),
the newer current pre-session image was restored across all **8,388,608 bytes**,
separately digest-verified and hard-reset at **2026-10-23 06:46:46 UTC**. The
[latest device-state record](../results/zephyr-native-tests-2026-10-23/device-state.json)
retains the logs and private backup identity, SHA-256
`863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
No new image was flashed to repair the acceptance or image-format findings.
The separate Rust probe produced no final firmware and opened no serial port.

After the [IDF accounting qualification](../results/idf-accounting-qualification-2026-10-23/RESULTS.md),
the same current pre-session state was restored across all **8,388,608 bytes**,
separately digest-verified and hard-reset via RTS. Completion was observed at
**2026-10-23 12:43:33 UTC**; the
[device-state record](../results/idf-accounting-qualification-2026-10-23/device-state.json)
retains the logs and private backup identity. No eFuse provisioning occurred.
This verifies the restoration boundary; subsequent application persistence
changes remain possible.

After the [IDF finite-load qualification](../results/idf-load-qualification-2026-10-23/RESULTS.md),
the same existing full-flash backup was restored across all **8,388,608 bytes**.
A separate verification matched its digest and then hard-reset via RTS;
completion was observed at **2026-10-23 15:29:34 UTC**. The
[load device-state record](../results/idf-load-qualification-2026-10-23/device-state.json)
retains the transfer and verification logs. Before these writes, the backup had
been digest-verified against the device rather than freshly read again. The
verification boundary does not establish post-boot persistence constancy or
deployed recovery behavior.

After the [failed RTIC attempt](../results/rtic-qualification-2026-10-23/RESULTS.md),
the complete pre-session image was restored across **8,388,608 bytes**, separately
digest-verified, then hard-reset via RTS. Completion was observed at
**2026-10-23 16:59:16 UTC**. The
[RTIC device-state record](../results/rtic-qualification-2026-10-23/device-state.json)
retains its separate private backup identity and public transfer/verification
logs. The backup was verified against the device before writes. No new RTIC
attempt followed the failure; subsequent original-application persistence
changes remain possible after reset.
