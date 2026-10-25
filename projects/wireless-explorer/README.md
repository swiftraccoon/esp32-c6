# Wireless explorer

[Workspace](../../README.md) · [Project conventions](../README.md)

Capture the wireless traffic that the **ESP32-C6-DevKitC-1-N8** can expose,
retain useful raw data and metadata, and summarize the observations to choose
the next experiments.

Status: passive recorder available, with observed receive windows for all three
radios on UART and Mac native USB. The [results](results/README.md) record
repeated-mode checks, drops, the retained USB startup failure and open gaps.
The [observation method](docs/observation.md) defines capture semantics.

## Capture modes

| Mode | Retained observations | Limits |
| --- | --- | --- |
| Wi-Fi | Promiscuous management, control, data and SDK misc reports; channel, RSSI, PHY/rate codes, receive status and raw bytes | One 2.4 GHz channel at a time; 4,096-byte snapshot cap; SDK filtering and transport loss |
| BLE | Legacy/extended advertising reports, raw advertising data, addresses, RSSI, PHYs and fragmentation status | Passive 1M/coded scan; controller selects channels; no arbitrary existing-connection sniffing |
| IEEE 802.15.4 | Raw MAC frames, channel, RSSI, LQI and SDK timestamp | Channels 11–26; hardware checks/removes FCS; no Thread/Zigbee decryption |

The recorder rotates through explicit receive windows. One shared RF path cannot
continuously cover every protocol and channel. Capture logs retain window timing,
packet sequence numbers, queue drops and truncation. Silence means no reports
during that window, not proof that a channel or protocol was unused.

Raw captures and reconstructed packet JSONL stay under ignored
`private/wireless-explorer/`. Activity summaries report packet types, signal
levels, observed addresses, BLE advertising field types and manufacturer IDs.
Encrypted payloads remain encrypted. See [capture semantics](docs/observation.md).

## Run

Build and flash the [firmware](firmware/README.md), then use the
[desktop client](clients/README.md). Firmware revision 2 uses UART at 921600 baud.
One serial session can rotate through all three modes without reopening the port
between windows. A serial open can still reset this board.

Rust 1.96.0, ESP-IDF 5.5.6 and the pinned esp-rs revision provide the current stack.
The board has 8 MB flash, a PCB antenna, no PSRAM, a CP2102N USB-to-UART port and a
native USB Serial/JTAG port. See the [hardware map](../../docs/hardware.md).

## Priorities

1. Qualify each capture mode and its loss/error reporting on the board.
2. Review recordings to select channels, packet types and protocol features for
   deeper investigation.
3. Establish iPhone USB-C power and bidirectional data through the
   [USB investigation](docs/iphone-usb.md).
4. Improve capture throughput, filtering, export and scheduling where observations
   show a need.

USB power is the initial operating assumption. Battery optimization is deferred.
iPhone operation, reconnect behavior, controlled-peer fidelity and independent
verification of RF silence still require measurements.
