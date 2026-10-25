# Wireless investigations

[Project](../README.md)

Start with the [passive recorder](../docs/observation.md): collect activity from
Wi-Fi, BLE and IEEE 802.15.4, inspect loss and channel coverage, then select
follow-up experiments. The [control and survey probe](control-bringup.md)
provides the initial UART baseline. Battery optimization is deferred.

Recorder qualification checks SDK startup/shutdown, packet/chunk reconstruction,
per-window count conservation and repeated mode transitions. Empty 802.15.4
windows qualify only initialization and completion; a known transmitter is
needed to qualify its receive callback and payload correctness. Retain private
raw captures and all failed attempts. No comparative performance result follows
from an ambient capture.

## Bring-up areas

| Area | Decision question | First useful check and fixture |
| --- | --- | --- |
| USB/phone transport | Can the chosen iPhone control the board and receive data over its native USB port? | The [power/data feasibility check](../docs/iphone-usb.md), with phone/iOS and cable recorded |
| Wi-Fi | Which channels/frame types merit deeper investigation? | Inspect promiscuous captures, then use controlled traffic to quantify loss; investigate SoftAP, ESP-NOW or CSI where useful |
| BLE | Which advertisement types and PHYs are observed? | Inspect passive reports, then validate legacy/extended/coded cases with a known advertiser; consider GATT after discovery findings |
| IEEE 802.15.4 | Which channels carry observable frames, and are payloads captured correctly? | Scan channels 11–26, then use a known transmitter with declared payload and ACK policy; decode higher layers only as supported by captured evidence |
| Coexistence | What happens to delivery and control responsiveness when supported radio roles share RF time? | Repeat working single-radio cases with declared pairs and all peers recorded; retain packet loss and gaps |

An iPhone control connection is not a substitute for a controlled raw
802.15.4 peer. If Wi-Fi/BLE carries control or results, include that traffic
in the experiment configuration. Capture and host forwarding may drop data;
record both boundaries separately.

## Prior work and permitted reuse

Read the shared [prior-work inventory](../../../docs/research/prior-work.md),
[reusable assets](../../../docs/research/benchmark-reuse.md), and
[C6 radio audit](../../../docs/research/prior-work-c6-radio.md)
before adding a benchmark.

Closest references and initial decisions:

- Wi-Fi/Thread/Zigbee: the reviewed Poli/Johansson study supplies scenario
  ideas and source/configuration pitfalls. Its reviewed repository lacks a
  top-level license; do not copy its code without resolving reuse permission.
- Raw 802.15.4: the reviewed Electric UI artifact supplies MIT-licensed packet,
  ACK and external-timing examples. Preserve notices for permitted reuse and
  reconcile its older IDF/configuration before adaptation.
- BLE and initial connection checks: start from the selected SDK's unmodified
  examples and documented APIs. No matched C6 BLE performance baseline is
  established by the earlier-ESP32 numbers in the radio review.
- Our useful extension is the phone-controlled C6 workflow, bounded capture
  delivery and recovery under radio activity. A new range sweep or OS ranking
  is not required to establish that workflow.

Before comparative collection, specify exact sources, permitted reuse, wire
payloads, units, endpoints, controls, versions and available fixtures under the
[assurance workflow](../../../docs/assurance-workflow.md). Keep the original
failed attempts. New records belong to this project's
[results directory](../results/README.md); the OS study's outcomes remain
separate.
