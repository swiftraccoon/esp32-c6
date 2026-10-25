# Native USB read-only evidence review

Scope: completed sessions `observer-2026-10-24-06-retry-01` and
`observer-2026-10-24-07`, native USB info/ping controls, and preservation of the
failed original session 06. No serial device was opened and no board action was
performed. Raw inputs and detailed replay outputs remain private.

The native USB plan matches commit `4425ccb`; its repeated schedules refer to
the short and full schedules frozen in `plan-image-03.json` at `920b03f`.
Both completed manifests match those sequences exactly, including IDs, channel
order, durations and two cycles. Firmware image identity is established by the
transfer and verification records, not authenticated by transcript replay.

Collector SHA-256:
`f69cfe6aaf681ea029dba05c40472f0e6bab569ee8e31015270167d899e07b17`.
Independent checker source SHA-256:
`590a55bddb21b5670176099bdc166b341c74eb1d7011c6312030997fe96b5e9d`.

| Session | Radio | Windows | SDK reports | Logged | Recorder drops | Retained bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 06 retry | Wi-Fi | 8 | 571 | 569 | 2 | 350298 |
| 06 retry | BLE | 2 | 72 | 72 | 0 | 1826 |
| 06 retry | IEEE 802.15.4 | 10 | 1 | 1 | 0 | 45 |
| 07 | Wi-Fi | 26 | 506 | 506 | 0 | 393204 |
| 07 | BLE | 2 | 36 | 36 | 0 | 845 |
| 07 | IEEE 802.15.4 | 32 | 1 | 1 | 0 | 45 |

All 80 windows passed strict collector replay and independent raw derivation.
Terminal error, invalid and truncation counts are zero. Counts, chunk lengths,
metadata radio/channel, RSSI extrema, sequence order and timing agree with saved
summaries; report conservation holds in every window.

Second-pass Wi-Fi SDK report counts in the short run are 24,116,139,8 on channels
1,6,8,11. The full run has reports on every second-pass Wi-Fi channel: counts
17,11,18,5,17,38,37,83,5,1,6,4,3 on channels 1–13. BLE reports occur in both passes
of both runs. The short run's one IEEE report occurs in its second pass on
channel 11; the full run's one IEEE report occurs in its first pass on channel 11.
Empty IEEE windows do not establish fidelity or absence of ambient traffic.

Wi-Fi PHY codes are only 0 and 1 (pinned SDK 11b and 11g/OFDM formats): short-run
counts 364/205 and full-run counts 311/195. No HT/HE-format capture is established
by these native USB sessions. SDK receive-status 65 remains recorded in 216 short
and 200 full-run Wi-Fi reports; it is not interpreted as a clean frame.
All BLE reports have primary PHY 1, secondary PHY 0 and complete-data status 0;
event flags are 16/19. This is legacy 1M advertising evidence, not coded PHY,
secondary PHY or extended-fragment qualification.

Native USB info replay reports transport `native_usb`, firmware revision 2,
and zero protocol/write-error counters. Ping replay passes; an independent
request/reply comparison also confirms numeric nonce equality, request ID and
wire version without publishing their values.

Original session 06 contains only manifest, startup bytes and failure marker.
The startup file remains 259 bytes. Its failure category is Broken pipe; no
request frame or window directory exists. Replay rejects the retained failure
marker. This is a failed pre-command attempt, not a successful zero-window
capture or proof of automatic reconnect. Retry files remain distinct.

These results demonstrate Mac native USB commands and the observed passive
receive schedules. They do not prove lossless RF reception, transport throughput
advantages, RF silence, causal attribution of the earlier teardown anomaly,
iPhone compatibility, phone-only power, automatic reconnect or known-peer
fidelity. Both board ports were connected in the collection setup.

Commands run from the repository root, with successful stdout/stderr retained
under ignored `.cache/wireless-explorer/review/observer-captures/` with the
corresponding replay/independent basenames:

```sh
projects/wireless-explorer/target/release/wxctl replay-observation --capture private/wireless-explorer/captures/observer-2026-10-24-06-retry-01
projects/wireless-explorer/target/release/wxctl replay-observation --capture private/wireless-explorer/captures/observer-2026-10-24-07
.cache/wireless-explorer/review/observer-captures/checker/target/debug/observer-capture-review private/wireless-explorer/captures/observer-2026-10-24-06-retry-01
.cache/wireless-explorer/review/observer-captures/checker/target/debug/observer-capture-review private/wireless-explorer/captures/observer-2026-10-24-07
projects/wireless-explorer/target/release/wxctl replay --capture private/wireless-explorer/captures/native-usb-info-2026-10-24-01
projects/wireless-explorer/target/release/wxctl replay --capture private/wireless-explorer/captures/native-usb-ping-2026-10-24-01
projects/wireless-explorer/target/release/wxctl replay-observation --capture private/wireless-explorer/captures/observer-2026-10-24-06
```

The final command is expected to fail on the retained failure marker. None of
these commands perform collection or access serial hardware.
