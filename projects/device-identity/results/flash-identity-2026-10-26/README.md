# Flash UID and provisioning storage

The accepted existing-tool read captures a nontrivial 16-byte UID window with
all five shifted reads matching. Data above byte 7 are not FF: this contradicts
the C datasheet's post-64-bit padding and supports XMC's documented 128-bit D
format. The exact fitted suffix is not identified by the shared JEDEC family.
The vendor documents this factory UID as read-only, separately from ordinary
flash data and programmable security storage.

All three 1024-byte security-register windows returned FF and their permanent
lock bits were clear. Under the documented C/D mappings, no programmed identity
was found in that storage. The source API also reported all 256 data bits in
each readable eFuse block 3–10 as zero, including all six key blocks. Raw key
bits were deliberately discarded; saved booleans cannot independently replay
those bits. Existing purpose/protection metadata is recorded in the
[targeted metadata result](../provisioning-metadata-2026-10-25/README.md).

The complete accepted schedule contains 846 calls: JEDEC, status before/after,
64 SFDP words, four primary UID words, five UID overlaps and 768 security words.
The 256-byte SFDP response is revision 1.6 with three parameter headers; all
three declared table payloads lie within the saved window. Status bytes before
and after agree. Raw identifiers, all responses and diagnostics stay private.

Later retrospective comparison with the official C/D SFDP tables finds BFPT
DWORD1 bit 19 set, agreeing with D revision 1.2 rather than C revision 1.7.
This independently corroborates the D-compatible capability/UID interpretation.
The three parameter headers and vendor table are shared by those revisions;
they do not supply another documented per-unit identifier. Package, temperature
and exact ordering suffix are not identified by these fields. This comparison
uses the same saved 256 bytes and adds no new device access.

The first 852-call MOSI-padding attempt completed but failed its first UID
overlap: all 13 padded reads returned zero. It remains rejected; no prefix was
substituted for a passing whole capture. Its original freeze and complete failed
cohort remain preserved. Revision two uses address/dummy/MISO phases only,
through the unchanged installed esptool 4.12 API. The
[method](../../experiments/flash-api-read.md) was frozen separately before each
attempt, with synthetic controls and independent source review.

Independent saved replay agrees with the complete revised schedule, all five
UID overlaps, status/region interpretation, source identities and receipts;
35 in-memory contrary cases reject. All 22 original files across the two
cohorts remained unchanged during review, and the failed cohort's seal agrees.
This establishes the selected saved-byte interpretation, not factory uniqueness
across a population, electrically measured SPI timing or authenticated hardware.

Both attempts completed explicit DTR deassertion, a warning-free host hard-reset
sequence and port close. The accepted reset completed at
2026-10-26T00:04:20.176088+00:00. The read sequence issued no firmware download,
eFuse burn, bridge configuration change, security-register program/erase or
flash-array write command. The original application restart may alter its own storage; host reset is not independent
boot, health or persistence attestation. The earlier full-flash restoration
boundary remains historical and is not replaced by these reads.

The [summary](summary.json) contains sanitized states/counts;
[provenance](provenance.json) records both attempts, freezes and validation.
`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. It establishes retained-byte integrity, not hardware authenticity.
