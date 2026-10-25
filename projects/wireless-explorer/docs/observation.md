# Passive observation recorder

[Project](../README.md) · [Client](../clients/README.md) · [Control protocol](protocol.md)

The recorder collects SDK-visible wireless reports so activity can guide later
experiments. It is a functional exploration tool, not a lossless spectrum
analyzer, general SDR, decryption tool or comparative benchmark.

## Radio behavior

| Radio | Configuration | Bytes and metadata |
| --- | --- | --- |
| Wi-Fi | NULL mode, no association/AP, promiscuous all packet types and all control subtypes, power save disabled | Up to 4,096 payload bytes; original SDK length, packet type, channel, RSSI, receive state, PHY/rate codes, SDK wrapping timestamp |
| BLE | Bluedroid extended scan, passive on 1M and coded PHYs, interval/window 160 × 0.625 ms, duplicate filtering disabled | Each SDK report separately, up to 251 bytes; address/type, advertising flags, primary/secondary PHY, SID, data status, transmit power, periodic interval and directed target |
| IEEE 802.15.4 | Promiscuous receive, receive while idle; promiscuous setup disables automatic ACK transmit/receive and enhanced ACK transmit in pinned IDF | MAC frame excluding length prefix and two replaced FCS positions; RSSI, LQI, channel and raw SDK SFD timestamp |

Wi-Fi supports configured windows on channels 1–13. Channel 14 is not implemented
by this recorder. Wi-Fi misc reports contain metadata only; their reported length
does not authorize reading a payload. Other Wi-Fi reports use the SDK's generated
receive metadata layout and length. FCS semantics follow that SDK; JSONL preserves
its bytes rather than inventing a PCAP link-layer representation.

BLE extended scanning includes controller-reported legacy advertisements. It does
not expose arbitrary BLE connection packets, IQ samples, every rejected packet or
a known receive channel. Channel zero therefore means unknown/controller-managed,
not RF channel zero. The controller schedules both PHYs; a full scan window is
not proof of simultaneous coverage. Extended advertisement fragments remain
separate reports, with their data-status field preserved. Periodic advertisements
are not synchronized by this implementation.

IEEE 802.15.4 only delivers frames accepted by the hardware CRC check; RSSI/LQI
replace the FCS positions in the SDK buffer. The callback copies the MAC bytes
and returns every non-null SDK buffer exactly once, including when a report is
dropped or capture has stopped.

Shutdown first disables receive-while-idle, then puts the 802.15.4 radio to sleep
before disabling its driver. The sleep path explicitly stops the current RX
operation; driver disable alone does not. The original image's repeated-mode
capture produced no Wi-Fi reports after 802.15.4, prompting this correction.

No capture mode intentionally transmits discovery/association/scan/ACK packets.
An independent receiver is needed to qualify actual RF silence.

## Scheduling, buffering and failure behavior

The desktop client opens serial once and sends bounded windows sequentially.
Defaults: Wi-Fi channels 1–13, BLE scan, then IEEE 802.15.4 channels 11–26,
one second per window, one cycle. Channel lists, dwell and cycles are configurable.
No mode switch occurs until the previous window's End has been validated.

Eight static packet slots hold up to 4,096 bytes each. SDK callbacks copy only,
without allocating, formatting, waiting or writing serial. Wi-Fi/BLE use SDK task
callbacks; 802.15.4 normally calls from an ISR and can also call synchronously
inside the SDK's critical section while stopping a pending receive. The same
bounded, nonblocking callback handles both contexts. Atomic producer/consumer gates reject
contention instead of waiting. Full/contended-buffer rejections count as recorder
drops. SDK, RF and controller omissions before callback delivery cannot be counted.

The main task serializes packet metadata and 128-byte hexadecimal chunks.
UART0 is 921600 baud, 8N1; the native USB transport uses the same records.
UART has no TX software buffer; nonblocking writes retry at one millisecond with
a one-second limit per record. FreeRTOS tick rate is 1000 Hz. UART capacity and
hex/JSON overhead can still cause significant drops under load. USB enqueue
success is not delivery acknowledgement.

A requested dwell starts after SDK receive-mode confirmation. Setup, radio
shutdown, queue drain, host validation and serial transfer introduce gaps or
overrun. A large queued packet can extend dwell before the next deadline check.
Record actual timestamps; do not treat requested dwell as measured airtime.

Invalid request settings get a correlated control error. Radio errors produce
a failed End when serial remains usable. A failed stop or uncertain partial
802.15.4 initialization terminates command handling; an unsuccessful final End
write also exits main. Dropping/aborting the desktop process leaves the current
bounded device window to expire; there is no immediate stop command or qualified
automatic reconnect. Resets, incomplete records and missing End invalidate replay.

## Stream contract

Control requests retain revision 1 framing (`WX `, at most 512 bytes before LF):

```json
{"version":1,"id":1,"command":{"observe":{"radio":"wifi","channel":6,"millis":1000}}}
```

The response is a distinct `WC ` stream, revision 1, at most 1,024 bytes per
line before LF. Each record contains version, request ID and one event:

1. Start: settings and firmware uptime before radio setup.
2. Listening: SDK setup/start confirmed.
3. Packet: callback sequence, uptime, radio metadata and lengths.
4. Chunk: packet sequence, decoded offset and lowercase hex bytes; repeated
   until captured length is filled. No packet interleaving.
5. End: stop time, final drain time, callback/emission/drop/invalid/truncation
   counts and error status.

Sequence numbers include dropped and invalid reports. End requires
`seen = emitted + dropped + invalid`. The host checks monotonic time,
radio/channel agreement, exact chunk lengths, sequence ordering, complete windows,
counter conservation and minimum dwell. It rejects extra output, even boot/panic
text. These are consistency checks, not cryptographic authentication or proof
that SDK callbacks reflect all over-air packets.

Firmware timestamps are monotonic microseconds since boot. Wi-Fi SDK receive
time wraps at 32 bits; the 802.15.4 SDK timestamp is retained without conversion.
The manifest records host start UTC as Unix milliseconds. Host and device clocks
are not synchronized; do not directly combine them into precise UTC packet times.

## Saved capture

A new output directory contains:

- `manifest.json`: requested schedule, port, baud and host start time.
- `startup.raw`: bytes captured for two seconds after serial open/control-line
  release. ROM startup at 115200 may be garbled while the application uses 921600.
- `window-NNNN/request.frame` and `serial.raw`: exact request/response bytes.
- Per-window `events.jsonl` and `packets.jsonl`: decoded events and reconstructed
  packet metadata/payload hex.
- Per-window and session `summary.json`: activity, timing and loss counts.
- `failure.txt` on a failed live attempt. Retain it with partial files.

`replay-observation` streams raw bytes through the same consumer and compares all
derived JSONL and summaries. It rejects failure markers, symlinked input files,
changed requests, oversized metadata, unexpected/missing files, malformed frames,
missing completion and inconsistent counts. Startup bytes must be present and
bounded; their contents are not interpreted as a clean-boot assertion.
It is read-only. It does not authenticate the board, the manifest's host metadata,
or externally altered-but-self-consistent captures. A killed process may leave
no failure marker; missing files/End still prevent a successful replay.

Distinct address counts are observations, not device counts: BLE addresses can
rotate and Wi-Fi devices have multiple interfaces. BLE advertising field and
manufacturer counts include only reports marked complete by the SDK; malformed
AD structures are counted. Raw data is retained regardless of interpretation.

## Qualification and sources

Host controls exercise full-size/truncated packets, all three metadata forms,
fragmented serial input, altered JSONL/summaries, retained failures and deliberately
invalid transitions. Buffer controls exercise full queues, index wrap, contention
and producer/consumer publication. The
[API and safety contract](api.md) records callback ownership, admission limits
and guideline decisions.
Target compilation does not establish on-board capture or RF silence.

This maintained document is a derivative of the version named by historical
source-input manifests. Their hashes identify the original text in Git history;
changing a documentation link does not replace that frozen input.

Pinned ESP-IDF 5.5.6 source:
[Wi-Fi sniffer guide](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/docs/en/api-guides/wifi.rst),
[BLE GAP API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/bt/host/bluedroid/api/include/api/esp_gap_ble_api.h),
[802.15.4 API](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/ieee802154/include/esp_ieee802154.h)
and [802.15.4 implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/ieee802154/esp_ieee802154.c).
