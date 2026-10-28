# Saved full-flash identity coverage

All five retained original-state images have complete **8,388,608-byte** coverage.
Each contains 624,848 bytes exactly matching the known build's bootloader,
partition table and application, a 24,576-byte NVS partition, and 7,739,184
FF bytes in the remaining gaps. All fifteen artifact comparisons match; no
unexplained non-FF gap remains. The five snapshots overlap in history and are
not independent boards or calibration runs.

The original ESP-IDF 5.4.1 NVS tool parsed all five slices with JSON,
integrity/minimal and reconstructed-blob output. Independent comparison of
every header, bitmap and raw slot reproduces all **122,880 NVS bytes and
3,780 slots**. Empty slots are FF. Each slice has three current logical values:
`phy/cal_version`, six-byte `phy/cal_mac`, and 1904-byte `phy/cal_data`.
Recognized erased heads number 16/18/18/20/17, using the same PHY keys.

The known MAC relationships hold in all five: `cal_data` contains the factory
MAC, and `cal_mac` is the first six bytes of the factory-derived EUI-64.
Payload children are raw blob storage; incidental fields decoded from those
bytes are not additional named keys. No additional named persistent identity
store or unexplained programmed region was identified.

Later comparison of the five reconstructed PHY blobs finds four distinct
1894-byte opaque payloads, with unchanged version and known MAC fields. No
additional literal known MAC, reversed MAC or EUI-64 occurs in those opaque
bytes. This is variable stored calibration material; neither an additional
immutable serial nor population-unique fingerprint is established. The public
layout and erase/recalibration APIs document the known fields and supported
modification path; the binary PHY library does not provide a semantic decoder
for every opaque byte. This follow-up uses the same saved images.

This is retrospective inspection of already explored snapshots. The comparison
uses the existing flasher manifest and unmodified upstream tools. Independent
sequential comparisons reproduce the complete cohort; 31 byte-classifier
controls and nine contrary NVS-coverage controls pass. Earlier exploratory
metadata/child interpretations and their corrections remain private. There was
no new device access, radio operation, firmware build, key deletion or programming.

The [summary](summary.json) publishes fixed counts and match relationships;
[provenance](provenance.json) identifies the sources and qualification. Unique
values, snapshot/NVS hashes, binaries and complete tool output remain private.
SHA256SUMS covers every public file except itself.

Byte coverage does not prove arbitrary credential absence inside compiled
artifacts or opaque PHY payloads, authenticate their historical producer, or
measure the current live board. The identified PHY MAC copies are already
identifying. Separate source/ELF review identifies volatile IRK, cached SC key
and negotiated security state; these need not have a persistent partition.
Their generation/change paths and the later targeted live observations are
recorded in the [inventory](../../README.md).
