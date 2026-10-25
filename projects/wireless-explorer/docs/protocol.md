# Control protocol, revision 1

[Project](../README.md) · [API and safety contract](api.md)

Control commands support one outstanding operation per port: `info`, `ping`,
and `wifi_scan`. The `observe` extension uses the distinct
[`WC ` packet stream](observation.md) instead of a control reply.
Legacy control replies are a functional probe, not a packet capture stream,
benchmark, authenticated channel, or reliable delivery protocol.

Each frame is ASCII `WX ` followed by one UTF-8 JSON object and LF. The maximum
line length is 512 bytes before LF, including the prefix. CRLF is accepted.
An oversized line is discarded through LF; its valid-looking suffix is never
interpreted as a new request. Empty lines are ignored. Unknown fields, commands,
versions, integer overflow, and trailing non-whitespace are rejected.

```text
WX {"version":1,"id":42,"command":{"ping":{"nonce":123}}}
WX {"version":1,"id":42,"reply":{"pong":{"nonce":123}}}
```

The other command values are the JSON strings `"info"` and `"wifi_scan"`.
Enums use Serde's external tags so the shared parser can run without allocation.
The [shared crate](../crates/wireless-protocol/src/lib.rs) defines all fields.
Request IDs and ping nonces correlate responses; neither authenticates a device.
The desktop ID is derived from host time and can collide across clients or runs.

`info` reports the responding port, firmware contract revision, implemented
commands, rejected-line count, and reply-write failure count for that port.
Firmware revision 1 reports BLE and 802.15.4 false; revision 2 reports both true
for its passive capture commands. These flags indicate implementation, not
hardware qualification. The configured board target is ESP32-C6-DevKitC-1-N8; firmware
does not detect the board's commercial part number or PCB revision.

`wifi_scan` starts a passive scan with a requested 120 ms dwell per channel,
using the SDK's default country/channel policy. It never connects to an AP.
At most 32 AP records are summarized into channel bins 1–14. `total_found` is
the SDK's count before the cap; `summarized` is the number actually decoded by
the wrapper. `unreported` includes both cap omissions and records the upstream
wrapper excludes because their SSID is not UTF-8. Channel bins describe only
summarized records, not every detected AP. Zero APs is a valid observation.
SSID, BSSID, credentials and packet payloads are not returned or saved.

The receiver checks `summarized <= 32`, `total_found = summarized + unreported`,
and `sum(channels) + out_of_range = summarized`, using widened arithmetic.
Unknown channels remain explicit. SDK failures retain their integer status.
Unparseable requests receive a protocol error with ID zero; valid requests
preserve their ID in success and radio-error replies.

## Execution and limits

One main FreeRTOS task polls UART0 and native USB with independent line decoders.
UART0 uses GPIO16/17, 8N1: 115200 baud in revision 1 and 921600 in revision 2.
Native USB uses GPIO12/13. The UART idle
timeout is normal input absence. Both receive buffers are 1024 bytes; USB has a
1024-byte transmit ring. Application diagnostics and the secondary SDK console
are disabled so they cannot share the command stream. ROM boot text may still
appear. The client retains pre-command startup bytes separately; boot output
after the request fails the exchange and is preserved as a failed attempt.

One scan temporarily occupies the command task, including service of the other
port. The scan-completion poll has a five-second deadline; SDK initialization,
start/stop and result-retrieval calls do not acquire a hard real-time deadline
from that poll. The scan path uses SDK callbacks; capture callbacks copy into
bounded buffers as described in the [observation contract](observation.md).
SDK Wi-Fi and PHY calibration persistence are disabled.

Reply writes are retried for at most one second. USB write success means queued
to the SDK, not delivered to a reader. The saturating `tx_failures` counter
counts application-observed write errors/timeouts only. These counters do not
measure UART hardware overruns, RF losses, or bytes lost on disconnection.
There is no promise of lossless streaming or simultaneous radio operation.

Send only one command at a time, including across ports when a scan is active.
An incomplete request remains buffered until LF or overflow; a new client must
finish/discard that line before sending a fresh frame. A partial reply fails
validation. There is no automatic retry, cancellation, or reconnection recovery.
The client deasserts DTR and RTS without generating a reset pulse; port opening
or line changes can still reset this board. Do not use it to carry credentials.

## Capture and replay

`wxctl exchange` requires a new output directory. It saves `request.frame`,
`connection.json`, pre-command `startup.raw`, and exact post-request `serial.raw`
bytes, plus `response.json` on success or `failure.txt` on failure. Both receive
phases have a 64 KiB cap. After deasserting control lines, a nominal 1000 ms
startup capture allows the device to settle; a pending 100 ms read can extend
that window. Startup bytes are retained without applying the command validator.
The response deadline is 1–120 seconds (15 by default). A 300 ms
post-response capture window can detect immediate extra output, not later
resets. Capture limits and deadlines are bounds, not performance measurements.

Live validation and `wxctl replay` use the same parser. Replay requires exactly
one complete matching response and compares it with the saved response. Missing,
duplicate, mismatched, failed, oversized, boot/panic, or truncated output fails.
A retained `failure.txt` prevents promotion to a successful replay. Metadata
and artifact provenance require the separate result record; replay does not
authenticate hardware, validate `connection.json`, prove flash identity, or
establish native USB/iPhone compatibility.

The project-owned `results/evidence.json` is checked by `verify-probes`.
It lists captures and derives only `observed_transcript` coverage for UART
control and passive Wi-Fi summaries. Other domains stay `not_run`; changing
a coverage label alone fails verification. Startup behavior and connection
metadata are not authenticated or validated by replay. This schema cannot authorize
comparisons or promote hardware, phone, power or provenance claims.
