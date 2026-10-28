# Controlled SoftAP address presence witness

Two reset-separated ESP32-C6 probe boots each produced 32 received CRC-valid
beacons whose transmitter/BSSID and private nonce SSID matched the pre-start
AP driver control. Driver readback matched while active; Stop reported inactive
state and restored the original saved driver AP MAC. The fresh complete original
8 MiB image was restored and separately verified before host reset release.

This establishes a pre-start AP address presence witness. It does not establish
full-window/startup/Stop observation, silence, hot rotation, cold boot, privacy
or correlation reduction. The initialized driver remained owned until reset.
Raw addresses, nonce SSIDs, UART/packet data, full images and original-flash
hashes remain private.

## Cohort and inputs

[Source inputs](inputs.sha256), the [prospective method](../../experiments/wifi-ap.md)
and [collector/image identities](artifacts.json) were frozen at `39305c0` before
board access. There were two planned and two admitted trials, with no rejected
hardware trial. An explicit normal reset separates them; they are not cold
power cycles. Each used the same compiled image/control and a distinct fresh
nonzero nonce. [Result JSON](result.json) contains sanitized relationships only.

| Observation | [Trial 1](trial-01.json) | [Trial 2](trial-02.json) |
| --- | --- | --- |
| Initial application state | Idle, driver uninitialized | Idle, driver uninitialized |
| Pre-Start receiver gate | Bound HTTP 200/SHB and live-child check | Bound HTTP 200/SHB and live-child check |
| Active driver AP readback | Requested control, different from original | Requested control, different from original |
| Complete receiver EPBs | 32 | 32 |
| Supported good-FCS beacons | 32 | 32 |
| Exact private SSID and requested TA/BSSID | 32 | 32 |
| Bad / unqualified FCS | 0 / 0 | 0 / 0 |
| Stop readback | Inactive, original driver MAC restored | Inactive, original driver MAC restored |
| Driver initialized after Stop | Yes, retained until reset | Yes, retained until reset |
| Complete trial replay | Passed, saved summary byte-equal | Passed, saved summary byte-equal |

The frozen [Wireshark decoder](decoder-inputs.json), preserving repeated JSON
keys, independently checked all 64 complete ordinary beacons: exact SSID,
transmitter/BSSID, broadcast destination and valid FCS. A separate CRC32
implementation agreed. All reported radiotap frequencies were 2412 MHz; this
is receiver metadata, not a calibrated frequency measurement. Opaque vendor
information elements were present and were not audited for collateral identity.
No packet-identifier equality or population uniqueness comparison was made.

[Before](observer-inputs.json)/[after](observer-after.json) fixed receiver states
and their [continuity check](observer-continuity.json) agree: one selected
running source without error/pause, expected process and ten unchanged config
metadata records, channel 1, and a separate Intel default-route interface/PHY.
No datasource control, retuning, Nordic-port access or service change was made.
The reader was temporary; ordinary Kismet reception/packet tracking continued.

## Device preservation

[Restoration evidence](restoration.json) records the separate boundary. A fresh
8,388,608-byte backup was read and independently verified before writes, with
no application boot between those steps. Security/partition gates passed.
The temporary bootloader, table and application were written as separate aligned
segments; their 4 KiB-rounded erase footprints avoided original stored-data
partitions and kept the app inside the original executable slot. New and original
partition layouts differed; application fit did not imply layout equality.

After both trials, the entire fresh backup was written back and a separate
esptool device-MD5 comparison explicitly verified all 8,388,608 bytes at
`2026-10-25T08:52:37.110925Z`. This is full-range digest verification, not a
second full-file readback or device attestation. The host reset sequence completed
at `2026-10-25T08:52:37.824346Z`, with zero exit and no unexpected reset warning.
Physical postboot health, subsequent persistence changes and radio activity
remain unmeasured. Host reset completion is not a physical boot acknowledgment.

## Qualification and scope

Before collection, 111 workspace tests and five doctests passed, with 11 worker
controls on host/actual remote Rust 1.90. Target build, formatting, Clippy and
documentation checks passed. Separate contrary controls exercised protocol and
autonomous cleanup, exact witness/CRC/header checks, readiness ordering and
cancellation, private sentinels, and the actual production replay consumer.
The discovered cancellation race was corrected before this freeze.

Both complete 19-file trials passed independent receipt, raw/transcript,
nonce/readback, request/worker/build/SHB and saved-summary checks. All 38
original files remained unchanged. Twelve rehashed semantic copies rejected,
including wrong identity, bad FCS, valid-CRC malformed beacon elements,
late records and cleanup/readback conflicts. A valid receiver prefix without a
qualified target cannot become a successful trial. Source/live event records
support the liveness and environment gate; saved bytes do not independently
re-attest process state.

The [firmware snapshot](firmware-inputs.json), [worker snapshot](worker-inputs.json)
and [flash-tool inputs](flash-tool-inputs.json) identify precollection artifacts
and configuration, including the uploaded C6 stub payloads. Snapshot fields
labeled unqualified/precollection are not the final result status. Runtime
libraries are not hermetically pinned. Private proof retains setup/qualification
failures and original attempts; they are not new board observations.

The receiver's 32-packet prefix could finish before the AP lifetime ended. The
requested eight-second automatic deadline and source-enforced host wait are
not a measured RF interval; replay checks event order rather than an elapsed
threshold. No absence or delivery-fidelity claim follows. STA/BLE/IEEE addresses,
active rotation, early default-address exposure, cold boot, reconnect, source
authenticity, application credentials and cross-interface correlation remain
outside this result. Factory/eFuse/USB identities are separate ownership domains.

`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. Input manifests instead use workspace-root paths at their frozen
commit. Public files are sanitized derivatives; exact real replay requires the
private captures. Seals establish retained-file integrity, not authenticity.
