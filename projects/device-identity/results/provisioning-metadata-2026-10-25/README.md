# Targeted provisioning metadata

All 256 BLOCK3 user-data bits read zero and their write-protection bit is unset.
The custom-MAC field is zero. These meet the documented blank/unprotected
prerequisites for one-time custom-MAC provisioning; no burn was attempted.
Factory BLOCK1 and populated optional-ID BLOCK2 are separate and remain unchanged.

The six key-purpose fields all report `USER`, with no key-block read-protection
bits set. Secure boot and flash encryption report disabled; secure-download
mode and download-mode disabling are unset. Purpose/protection metadata does
not establish that user-purpose key storage is empty or that software credentials
are absent. Key contents were excluded from saved output.

This attached-host ROM read extends the existing
[inventory method](../../experiments/read-only-inventory.md). The exact field
list and effects were recorded privately before collection. An installed-tool
virtual read checked the 14 new metadata field names and widths; the real read
included the three original factory fields for 17 total. All three factory values
match the saved baseline. An exclusive-port check found no owner. Both the
metadata command and the explicit release command completed with status zero.

Independent saved-input review agrees with raw/formatted values, field locations,
protection bits, the complete field list and receipts. Twelve in-memory contrary
cases detect omitted/extra fields, truncated data, nonzero reserved data,
protection/purpose/security changes, a factory mismatch and command failure.
All 16 original cohort files remained byte-identical during review. These
retrospective interpretation controls are not another hardware collection.

Stock espefuse reads all blocks internally even though its whitelist excludes
key contents from output. The existing esptool release performs broader SPI/XMC
startup/reset setup before its host hard reset; it is not a narrow MAC read.
There was no eFuse or bridge programming and no requested flash-array write.
The application restart may change its own storage. This reset receipt is not
independent physical boot, health or persistence attestation, and does not
replace the earlier full-flash restoration boundary.

The [summary](summary.json) contains sanitized booleans/enums only;
[provenance](provenance.json) records the field list, statuses, time bounds and
installed source identities. Raw fields, unique values, concrete port and full
diagnostics remain private. This targeted observation does not qualify a
general metadata collector, factory-ID uniqueness or undocumented programming
behavior. Supported modification paths are maintained in the
[main inventory](../../README.md).

`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. It establishes retained-byte integrity, without authenticating the
hardware or command origin.
