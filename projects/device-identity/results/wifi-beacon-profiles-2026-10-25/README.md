# Retrospective beacon field comparison

All 64 saved beacons from the two [controlled AP boots](../wifi-ap-2026-10-25/README.md)
had identical interval bytes, capability bytes and complete ordered non-SSID
information-element TLVs, including opaque vendor contents. The private nonce
SSIDs differed. Both boots used the same synthetic AP MAC, compiled image,
channel and other requested configuration.

This shows repeated selected bytes in these two boots of one device. It does
not establish uniqueness, tracking accuracy, a device fingerprint, behavior
after an AP MAC change or correlation reduction. The unchanged AP address
already links the boots. No new board, receiver or RF access occurred.

## Cohort and comparison

This is a retrospective analysis of the complete existing two-trial cohort.
Preliminary inspection had already exposed the element shape and apparent
non-SSID equality; these are inspected data, not held-out observations. The
[comparison method](../../experiments/wifi-beacon-profiles.md), 73
[source/input identities](inputs.sha256) and release
[consumer/orchestration identities](artifacts.json) were frozen at `d897fed`
before the recorded derivation. The physical collection retains its earlier
`39305c0` freeze and `cbe71a7` result commit; this analysis freeze does not
retroactively preregister those packets.

Both original 19-file trial bundles passed full replay before analysis. All
32 packets per side were supported CRC-verified exact-target beacons; no
background, wrong-target, bad-FCS or unchecked packet was selected away.
[Comparison JSON](comparison.json) is the exact counts-only release output.

| Compared encoding | Distinct in each boot | Shared | Identical across all 64 |
| --- | --- | --- | --- |
| Two-byte beacon interval | 1 | 1 | Yes |
| Two-byte capability information | 1 | 1 | Yes |
| Complete ordered non-SSID IE stream | 1 | 1 | Yes |
| Complete ordered vendor TLV subsequence | 1 | 1 | Yes |

Nine non-SSID element IDs were present in all 64 beacons. Each ID's complete
ordered occurrence sequence also had one exact encoding shared by both boots.
The consumer retains complete IDs, lengths, values, duplicates and occurrence
order privately. Unknown and vendor contents remain opaque; no semantic audit
or factory-identifier absence claim follows from byte equality.

The comparison removes only the complete SSID TLV from the IE stream. PCAPNG/
radiotap metadata, all management-header fields, TSF timestamp and FCS are
excluded from these field comparisons. Their exclusion provides no conclusion
about equality, independence or identifying information in those fields.

## Qualification and preservation

Before derivation, 122 host tests, six doctests, formatting, workspace Clippy
and documentation checks passed on Rust 1.96.0. Independent contrary controls
passed 128 parser-oracle cases and 136 production full-bundle cases: 102 valid
CLI outputs exactly matched an independent byte-set oracle and 34 rejected.
Controls included order/duplicates/TLV boundaries/absence, per-packet variation,
equal variable sets, partial intersections, excluded fields, malformed late
records, private sentinels and failures on either side's full-bundle admission.
A presence-valid mixed capture cannot become an admitted profile comparison.

Postfreeze independent review checked all 64 exact target/CRC matches and
reconstructed the same private byte-set projections. The pinned
[TShark tool](../wifi-ap-2026-10-25/decoder-inputs.json), preserving repeated JSON
keys, agreed on fixed fields and every tag occurrence boundary. The independent
comparison report exactly matched the production output. These are independent
derivations from the same saved packets, not another physical replication.

Both original trial replays reproduced their saved summaries byte for byte.
Two runs of the frozen release comparison produced identical output bytes.
All 38 original trial files and the complete prior private run seal remained
unchanged. Raw addresses, nonce SSIDs, opaque payloads, field values and their
hashes remain private. The earlier full-flash restoration boundary is unchanged;
an offline analysis does not provide another board-recovery observation.

This result describes bounded prefixes, not the complete AP lifetime or future
traffic. Configuration, software, clients and environment can change the fields.
Population prevalence, tracking success, MAC rotation, RF silence, credentials,
cross-interface correlation and measured privacy improvement remain unqualified.

`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. Frozen input paths are workspace-relative at `d897fed`, including the
pre-derivation README; they do not claim current result text is collection input.
Runtime libraries are not hermetically pinned. Seals establish retained-file
integrity, not source authenticity. Exact real replay requires the private trials.
