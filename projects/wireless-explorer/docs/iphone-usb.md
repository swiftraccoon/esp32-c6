# iPhone and USB connection

[Project](../README.md) · [Hardware inventory](../../../docs/hardware.md)

Updated 2026-10-24. Desired outcome: one USB-C cable provides board power and
carries bidirectional commands and radio data between the iPhone and C6.
Neither power delivery nor iPhone data access has been tested for this project.
The target phone is iPhone 17 Pro; the exact iOS version remains unverified.

## Board and port choice

The ESP32-C6-DevKitC-1-N8 has two USB-C ports.
The [shared port map](../../../docs/hardware.md#board-ports-and-onboard-fixtures)
distinguishes the native ESP32-C6 port from the USB-to-UART bridge port.
Start the phone-data investigation with native USB. Keep the bridge as the
established Mac flashing/logging route; a successful Mac serial session does
not prove phone access.

The C6 native controller supplies fixed CDC-ACM serial and JTAG functionality.
It cannot be turned into an arbitrary HID, storage or USB Ethernet device by
changing firmware. Its USB function is distinct from a programmable USB OTG
peripheral. [ESP-IDF 5.5.6 USB guide](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-guides/usb-serial-jtag-console.html)

## What is documented, and what remains open

| Item | Evidence and limit |
| --- | --- |
| iPhone supplies accessory power | Apple documents up to 4.5 W for compatible small USB Power Delivery devices on iPhone 15 and later. This is not a measured power budget or successful connection for our board/cable. |
| iPhone transfers USB data | USB-C supports data for supported device classes. Connector compatibility does not establish that an ordinary iOS app can open CP2102N or CDC-ACM serial. |
| External Accessory framework | Apple's documentation describes communication with supported MFi accessories. It does not establish a generic serial API for this board. |
| Generic USB access from iOS apps | Apple's DTS guidance says third-party iOS code has no general USB-device access mechanism; DriverKit access described there is iPadOS-specific. No supported iPhone serial path has been identified for either board port. |
| Board and phone | ESP32-C6-DevKitC-1-N8 and iPhone 17 Pro. PCB revision and exact iOS version remain unverified. |
| Mac native USB | Info, ping and both repeated capture schedules passed; one pre-command startup failure remains retained. Both board ports were attached to the Mac, so this establishes neither phone data nor phone-only power. |

Sources checked 2026-10-24:
[Apple USB-C guidance](https://support.apple.com/en-us/105099),
[External Accessory documentation](https://developer.apple.com/documentation/externalaccessory/),
[Apple DTS generic USB guidance](https://developer.apple.com/forums/thread/802640)
(October 2025 statement, rechecked 2026-10-24),
[USBDriverKit platform scope](https://developer.apple.com/documentation/usbdriverkit).

The engineering conclusion is that wired power and data are a sensible target,
but iOS access requires a concrete supported path and an actual exchange.
This document does not conclude that all wired approaches are impossible.

## First feasibility check

Record the exact board revision, phone/iOS, cable, connected ports and power
source. Follow the board's documented power arrangements. Both USB ports may
be used according to the board guide; record whether the Mac also supplies
power, since that would not prove phone-only operation.

Establish each outcome separately:

1. Native USB enumeration and bidirectional bytes on the Mac, identifying the
   port independently of the CP2102N route.
2. Board startup from the iPhone with the Mac disconnected; later repeat under
   representative radio activity to observe resets or disconnects.
3. iPhone-side device discovery and a supported app/API that sends and receives
   a checked payload. Enumeration or a power LED alone does not pass this step.
4. Disconnect/reconnect and bounded transfer behavior, including dropped data,
   errors and resets.

Record `not run`, `passed`, `failed` or `unsupported` with the exact
configuration. A stack or OS upgrade requires its own compatibility check.
Step 1 passed on the Mac for the recorded configuration; the
[native USB results](../results/observer-qualification-2026-10-24/image-03-results.md)
include the successful exchanges, radio captures and retained startup failure.
Steps 2–4 remain unqualified. Native serial opens reset the board in these tests;
the successful retry does not establish automatic reconnect.

The [firmware](../firmware/README.md) includes the native Serial/JTAG driver
and the [desktop client](../clients/README.md) can exercise serial commands.
Mac use of that implementation establishes no phone access.

## Alternatives if direct serial access is unavailable

Keep the command/event contract independent of its transport so Wi-Fi or BLE
could carry phone control while USB supplies power. Such a configuration adds
radio traffic and must be recorded as part of a wireless experiment. A Mac
USB client can support initial device development separately from the phone.

Additional USB interface hardware is another possible investigation, not an
assumed capability of this board. Native iOS UI/platform glue may require
Swift or Objective-C; discuss that authored-language boundary before
implementation under the repository's Rust preference.
