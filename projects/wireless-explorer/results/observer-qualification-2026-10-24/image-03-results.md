# Repeated receive windows and native USB

Completed 2026-10-24 on ESP32-C6-DevKitC-1-N8. Image 03 received ambient
Wi-Fi, BLE advertising and IEEE 802.15.4 reports through both Mac serial paths.
Every second-pass Wi-Fi window received reports after the IEEE windows.
This qualifies these observed schedules; it does not isolate the cause of
image 02's interruption or establish lossless reception.

## Frozen inputs and board state

[Source inputs 02](source-inputs-02.sha256) were frozen at
`6237dc4b16fe87f7c5db89f39d40800e9a787558`; all 25 inputs still match.
The [UART plan](plan-image-03.json) was frozen at
`920b03f7cfcaf42b7086d83fe718dca34cad7567`; the
[native USB plan](plan-native-usb.json) at
`4425ccbd685cdfcc6973ef87f6d9f9b9c551aec2` before collection.

| Artifact | SHA-256 |
| --- | --- |
| Image 03 ELF | `0eb8ff583860dd4a12c2017b62914751350be9e931f5c90f84fc6ec59fa33bc2` |
| Image 03 merged flash image | `7e76d200daa1d327353bff37174e8ec6f4d231a66dc37acf97a781c2a87397be` |
| Collector | `f69cfe6aaf681ea029dba05c40472f0e6bab569ee8e31015270167d899e07b17` |
| Fresh original 8 MiB backup | `bc1f2d438df400c6353daf549daf784c546d60c46f80ec8b9293b32445a26f1a` |

The full backup was read and separately verified without booting the application
between those steps. Image 03 was written, separately verified and reset into
the application. After collection, the complete original backup was written,
separately digest-verified in the bootloader and released with a hard reset.
The board ends this investigation running its original firmware.
Both USB-C ports were connected to the Mac during the native tests; this does
not establish phone-only power. Transfer diagnostics and originals remain in
ignored `private/wireless-explorer/continuation-2026-10-24/` and `backups/`.

## Receive outcomes

The short schedule is two passes of Wi-Fi channels 1/6/8/11, one BLE window and
IEEE channels 11/15/20/25/26, at 1,500 ms per window. The full schedule is two
passes of Wi-Fi channels 1–13, one BLE window and IEEE channels 11–26, at
1,000 ms per window. Each pass uses the three receive modes sequentially.

| Session | Transport / schedule | Wi-Fi reports / logged / drops | BLE reports / logged | IEEE reports / logged |
| --- | --- | ---: | ---: | ---: |
| 04 | UART / short | 551 / 304 / 247 | 62 / 62 | 1 / 1 |
| 05 | UART / full | 1102 / 581 / 521 | 47 / 47 | 2 / 2 |
| 06 retry 01 | Native USB / short | 571 / 569 / 2 | 72 / 72 | 1 / 1 |
| 07 | Native USB / full | 506 / 506 / 0 | 36 / 36 | 1 / 1 |

All 160 completed windows passed strict collector replay, independent derivation
from raw serial records, and comparison with the frozen schedules. Terminal
errors, invalid reports and truncations were zero. BLE and IEEE recorder drops
were zero. Counts reconcile SDK reports with logged records and recorder drops;
loss before SDK callbacks is not measured. The lower native USB drop counts
come from different ambient traffic and do not establish a throughput ratio.

Both passes received BLE advertising in every session. IEEE reports were sparse:
one in session 04, two in 05, one in the second pass of the short native run and
one in the first pass of the full native run. Empty windows do not establish
fidelity or channel silence. The native runs observed legacy Wi-Fi formats and
legacy 1M BLE advertising. Coded/secondary BLE PHYs, extended fragments and
Wi-Fi HE reception remain unqualified. Nonzero Wi-Fi SDK receive-status codes
remain recorded rather than being interpreted as clean frames.

## Native USB control and retained startup failure

Native `info` reports transport `native_usb` and firmware revision 2; `ping`
echoes the requested nonce. Both exchanges passed replay and independent
request/reply checks. The native serial opens produced a USB-triggered reset
and 259 startup bytes in the retained attempts.

Original session 06 failed with `Broken pipe` during startup, before a request
or receive window. Its manifest, startup bytes and failure marker remain intact;
replay rejects it. Enumeration and startup/reset evidence were inspected before
repeating the unchanged schedule in `observer-2026-10-24-06-retry-01`.
The completed retry does not qualify automatic reconnect or erase the failure.

## Reproduction and coverage

Completed captures reside in ignored `private/wireless-explorer/captures/`.
For each, run `wxctl replay-observation --capture <directory>` and the retained
[independent checker](capture-checker/src/main.rs):

```sh
cargo +1.96.0 run --offline --locked --manifest-path \
  projects/wireless-explorer/results/observer-qualification-2026-10-24/capture-checker/Cargo.toml \
  -- private/wireless-explorer/captures/observer-2026-10-24-07
```

The checker parses raw JSON directly without importing the collector's consumer.
Its original source SHA-256 is
`590a55bddb21b5670176099bdc166b341c74eb1d7011c6312030997fe96b5e9d`.
The [native review](native-usb-review.md) retains concrete checks and objections.
[Coverage](coverage.json) records the scoped claims and missing measurements.
Public files contain aggregates and method evidence; private raw data is required
to reproduce RF observations. Archive hashes establish retained file identity,
not RF fidelity or independent timestamp calibration.

## Next experiments supported by these observations

1. Use Mac native USB for further exploratory capture while recording drops and
   startup failures. Qualify reconnect behavior with deliberate disconnects.
2. Use controlled transmitters with known sequence numbers and payloads to
   measure missing, duplicated and damaged reports for each receive mode.
   Include coded/extended BLE and Wi-Fi HE traffic explicitly.
3. Verify RF silence with an independent receiver; passive SDK settings alone
   do not establish absence of transmissions.
4. Establish iPhone-only power and a supported phone data path under the
   [USB procedure](../../docs/iphone-usb.md).

The tool captures SDK-visible reports within selected windows. Continuous
coverage of every channel, existing BLE connection sniffing, decryption,
calibrated timing, long-run stability and energy measurements are not established.
