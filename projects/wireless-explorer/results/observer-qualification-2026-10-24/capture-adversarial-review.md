# Read-only capture review

Reviewed 2026-10-24. Sessions `observer-2026-10-24-01`, `-02` and `-03` under
`private/wireless-explorer/captures/` all pass the actual collector's
`replay-observation` command and an independent Rust derivation from `serial.raw`.
The review opened no serial device and did not reset or flash hardware. Addresses
and payloads remain private. The sealed [source review artifacts](review/README.md)
are unchanged; their checksum verification still passes.

These sessions use image 02 and the collector identified in the
[qualification record](README.md). The replay executable's SHA-256 was checked:
`f69cfe6aaf681ea029dba05c40472f0e6bab569ee8e31015270167d899e07b17`.
The [input manifest](source-inputs-01.sha256) matched its bytes in source freeze
`88641d09c39f460f758a9037537d96390c8b4e57`; all 25 listed files matched before
the subsequent stop-path correction.

| Session | Radio | Windows | SDK reports | Logged | Recorder drops | Retained bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 01 | Wi-Fi | 13 | 336 | 261 | 75 | 185,161 |
| 01 | BLE | 1 | 22 | 22 | 0 | 554 |
| 01 | IEEE 802.15.4 | 16 | 0 | 0 | 0 | 0 |
| 02 | Wi-Fi | 8 | 411 | 156 | 255 | 86,706 |
| 02 | BLE | 2 | 96 | 96 | 0 | 2,500 |
| 02 | IEEE 802.15.4 | 10 | 4 | 4 | 0 | 180 |
| 03 | Wi-Fi | 6 | 786 | 366 | 420 | 241,620 |

All 56 windows have zero terminal errors, invalid-report counts and truncation
counts. Independent checks agree with the saved summaries on report conservation,
packet counts, retained lengths, metadata radio/channel, sequence order, chunk
offsets and bounds, RSSI extrema, Start/Listening/stop/End ordering and minimum
dwell. Recorded time includes scheduling/serial overrun; it is not calibrated
airtime. Recorder drops are substantial and omit losses before SDK callbacks.

Wi-Fi records include management, control and data types. SDK receive status 65
is retained in 80 of session 01's 261 logged reports and 27 of session 02's 156;
the status is not publicly defined by the header and is not interpreted as a
clean frame. The observed BLE reports use legacy event flags 16/19, primary PHY
1M and complete-data status. These observations do not qualify coded/secondary
PHY capture or extended-fragment reconstruction. Session 02's four 802.15.4
reports occur on channels 11 (two), 15 (one) and 20 (one). This supports receipt
of ambient SDK-reported MAC data, not known-peer delivery or network decoding.

## Mode-transition objection

Session 02's first Wi-Fi pass records 12, 142, 253 and 4 reports on channels
1, 6, 8 and 11. After BLE and five 802.15.4 windows, its second pass records
zero on all four channels despite successful Start/Listening/End records.
This is an unresolved reception anomaly for image 02, not a successful repeated
all-radio receive test. Protocol replay correctly admits zero-report windows;
it cannot diagnose RF silence. Session 03 is a Wi-Fi-only control: all six
windows across three channel-6/8 cycles are nonzero (76, 174, 112, 169, 97, 158).
That supports Wi-Fi-only restart behavior without establishing the anomaly's
cause.

Pinned IDF 5.5.6 source shows a relevant teardown difference:
`esp_ieee802154_disable` frees interrupt/sleep resources and disables RF/clocks
without calling `stop_current_operation`. `esp_ieee802154_sleep` enters the
critical section, stops the active operation (including RX `CMD_STOP`), sets the
sleep state and drops the RF reference. BTBB shutdown is reference-counted and
PHY shutdown uses per-modem flags; the source does not support a claim that
802.15.4 disable simply turns off every radio's PHY.

The reviewed correction sets receive-while-idle false, calls sleep while the
radio is initialized, then attempts disable even if quiescing failed. Either
error propagates and ends command handling. No new ownership or blocking issue
was identified in that small change. A pending RX completion may synchronously
invoke the callback from the calling task inside the SDK critical section;
its bounded copy/atomics/buffer-release path is valid there as well as in the
usual ISR context. Exclusively-ISR wording must not describe this corrected
path. A same-schedule hardware repetition is required before attributing the
anomaly to teardown or claiming the correction restores reception.

## Commands and reproduction scope

For each completed session, from the repository root:

```sh
projects/wireless-explorer/target/release/wxctl replay-observation \
  --capture private/wireless-explorer/captures/observer-2026-10-24-01

cargo +1.96.0 run --manifest-path \
  .cache/wireless-explorer/review/observer-captures/checker/Cargo.toml \
  --offline --locked -- \
  private/wireless-explorer/captures/observer-2026-10-24-01
```

The independent checker parses JSON directly rather than importing the
collector's consumer. Its source and aggregate-only outputs are retained under
`.cache/wireless-explorer/review/observer-captures/`. The private raw captures are
required to reproduce these checks; public host fixtures alone cannot reproduce
RF observations. No claim is made about RF silence, lossless capture, calibrated
timestamps, native USB, iPhone operation, battery use or controlled peer fidelity.
