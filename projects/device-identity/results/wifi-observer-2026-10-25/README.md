# Existing Wi-Fi receiver qualification

Two complete 32-packet prefixes from the existing Kismet Wi-Fi source passed
saved replay. Wireshark and an independent CRC implementation agree on all
64 probe requests. Neither sample contained beacons. This qualifies bounded
stream/save/replay and the declared radiotap/FCS/non-beacon decoding path;
physical beacon decoding and ESP32-C6 reception remain unexercised.

The [prospective method](../../experiments/wifi-observer.md) defines the sample,
limits, controls and retained failures. No receiver tuning, datasource control,
service change, ESP32 firmware operation or board reset was performed.

| Cohort | Freeze and inputs | Attempts / planned | Outcome |
| --- | --- | --- | --- |
| [Local profile rejection](cohort-01-failure.json) | `ff03a38`; [inputs](cohort-01-inputs.sha256), [collector](cohort-01-artifacts.json) | 1 / 2 | Rejected before directory creation or SSH because Kismet's uppercase UUID was not normalized. No packet stream opened. |
| [Original live deadline](cohort-02-failure.json) | `4600d63`; [inputs](cohort-02-inputs.sha256), [collector](cohort-02-artifacts.json), [tools](cohort-02-tool-inputs.json) | 1 / 2 | HTTP 200 and eight complete EPBs, but the 15-second deadline expired before the quota. Capture remains failed; child cleanup succeeded. |
| [Revised live deadline](cohort-03-result.json) | `44bcc69`; [inputs](inputs.sha256), [collector](artifacts.json), [tools](tool-inputs.json), [decoder](decoder-inputs.json) | 2 / 2 | Both complete 32-EPB prefixes accepted under the separately frozen 120-second ceiling. Replay matches saved summary bytes. |

Each failed cohort stopped on its first rejection. The UUID correction normalizes
profile notation before existing strict worker validation; the private selection
was unchanged. The later deadline revision retained the 32-packet quota and
8 MiB cap. The eight-record failed stream was not reused as a successful sample.
Its standalone packet summary passes, but complete replay still rejects its
failed receipt. Input hashes resolve against each cohort's frozen commit,
not the currently revised files.

## Admitted observations

| Observation | [Capture 1](cohort-03-summary-01.json) | [Capture 2](cohort-03-summary-02.json) |
| --- | --- | --- |
| Actual radiotap IDBs / link type | 1 / 127 | 1 / 127 |
| Complete EPBs | 32 | 32 |
| Management frames / other-frame count | 32 / 32 | 32 / 32 |
| Supported beacons | 0 | 0 |
| Bad / unqualified FCS | 0 / 0 | 0 / 0 |
| Complete seven-file replay | Passed, byte-equal summary | Passed, byte-equal summary |

The frozen Wireshark decoder classified every packet as an 802.11 probe request,
with valid FCS and reported frequency 2412 MHz. Independent CRC32 checks also
passed all 64 frames. These are source-provided radio metadata and captured
bytes; they do not calibrate the receiver, authenticate a transmitter or prove
that every transmitted frame was delivered. No packet-identifier equality or
C6 identity comparison was evaluated. Raw frames, addresses, SSIDs, timestamps,
source UUIDs, interface names, request bindings and decoded payloads are private.

[Before](observer-inputs.json) and [after](observer-after.json) metadata snapshots
and their [continuity check](observer-continuity.json) agree: one matching
running Wi-Fi source without error/pause, the expected Kismet process and ten
unchanged configuration metadata records, monitor channel 1, and a separate
Intel Wi-Fi default route. This bounds saved state snapshots; it is not a
per-packet channel guarantee or proof of complete in-memory configuration.
Credentials remained on the receiver; HTTP headers/cookies were not forwarded.

## Qualification and limits

The initial implementation passed 89 workspace tests and four doctests, plus
nine worker tests on both host and actual remote Rust 1.90. The UUID correction
added its regression, and the final profile/deadline changes passed four capture
and nine worker controls, formatting, Clippy and release builds. A production
worker synthetic test completed its quota after 16 seconds, beyond the former
15-second deadline. These controls do not substitute for physical reception.

Separate review exercised 71 decoder cases, 3729 partial byte cuts, fragmented
inputs, malicious lengths, full CLI bundles, mock HTTP security/cleanup and
actual-profile validation. All 14 admitted original files, receipts and request/
worker/build bindings passed independent checking unchanged. Twelve rehashed
semantic corruptions of the actual saved captures were rejected, including late
beacon/IE errors with valid recomputed FCS. Checksums alone were not the
acceptance test. The partial original stream also matched Wireshark and its
receipt while retaining its failed transport outcome.

The sample is 32 packet records, not a 120-second RF window. No arrival-rate or
performance comparison is made. Earlier filtering, deduplication, stream backlog
loss and RF conditions remain unqualified; well-formed output does not establish
lossless reception. Runtime libraries are not hermetically pinned. Zero beacons
leaves physical beacon extraction unexercised. C6 MAC overrides/startup/rotation,
cold boot, reconnect, RF silence, timing calibration, source authenticity,
correlation reduction and Nordic readiness remain unmeasured.

`SHA256SUMS` covers every retained regular file except itself, with paths relative
to this directory. Frozen input manifests instead use workspace-root paths at
the stated commits. Public files are sanitized derivatives; their seals establish
retained-file integrity, not raw-source authenticity. Exact real replay requires
the private seven-file captures. CI checks synthetic controls and archive seals;
no hosted CI execution is claimed here.
