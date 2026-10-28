# Saved USB and flash metadata audits

Decision: determine whether the saved ROM USB serial represents the factory
MAC, whether the saved CP2102N configuration validates under its documented
checksum algorithm, and which physical persistence metadata can be accounted
for in the retained original flash images. This is retrospective analysis of
already explored inputs, not a new device capture or a blinded comparison.

Reuse the complete original USB/eFuse captures and full-image backups. Do not
replace the failed original-application USB cohorts, infer successful transfers
from profiler properties, or select a favorable partition/blob subset. This
extends the original inventory's presence/layout checks with separate consumers;
the original summaries, methods and result seals remain unchanged.

Before deriving reportable results, freeze the consumer source, exact input
inventory/digests, qualification artifacts and this method in Git. Record the
freeze in the result ledger. Preserve every original and failed derivation
privately. No serial port, USB API, reset, flash program or erase belongs to
these commands. The inputs are one complete admitted ROM USB bundle, the
baseline's three selected eFuse fields, and all three retained original full
flash images. Private inventory binds their original locations and digests.

`usb-factory-relation` first replays all four USB files, including native
provenance, transfer order, cleanup, hashes and saved-summary equality. It
validates the selected eFuse metadata with the existing strict parser and
requires a readable MAC. The supported native serial is exactly 17 ASCII
hexadecimal characters with colon separators, encoded as UTF-16LE; compare its
six bytes to the factory value and publish only equality. A valid different
address yields false; protected/malformed/unsupported data fails. A match in
this one ROM capture establishes neither cold-boot repeatability nor firmware
descriptor customization.

`bridge-config-audit` also requires whole USB replay. Its supported CP2102N
encoding is exactly 678 bytes, layout byte two equal to one, little-endian
ConfigSize and big-endian stored Fletcher16. AN978 revision 1.5 defines the
algorithm and coverage of bytes 0..676. The stored byte order is the explicitly
tested layout, rather than an endianness claim supplied by that document. It
does not qualify lock state or programming permission. The legacy GetLockValue
support list excludes CP2102N; do not issue that query or attempt a write to
infer a lock.

`flash-metadata-audit` reads exactly 8 MiB. It validates the MD5-protected
partition table at the standard offset, supported app/data types, bounds,
alignment, labels and overlap. It rejects encrypted flags or nonempty NVS-key
partitions. Each supported NVS v1/v2 page and written physical head must have
valid CRCs, bitmap/span/type relationships and partition-local namespace
references; variable payload CRCs are checked. Publish aggregate physical
counts and fixed name-heuristic categories. Do not print names, values, offsets,
raw data or their digests. Repeated versions remain physical records; there is
no reconstruction of logical blob state.

The qualification uses both versions from the pinned SDK NVS generator and
partition generator, independently checks the upstream parser's counts/CRCs,
and rejects corrupted/truncated/unsupported/encrypted inputs. Additional
controls must retain successful prefixes followed by failures, foreign
partition namespace declarations, altered semantic fields with recomputed CRCs,
protected eFuse fields, valid unequal MACs, every one-bit CP configuration
mutation, and failed/inconsistent complete USB receipts. An independent
reviewer must reproduce controls against the actual consumers.

Run from the repository root with ignored private inputs:

```sh
projects/device-identity/target/debug/identityctl usb-factory-relation --capture private/device-identity/RUN/usb-rom --efuse-input private/device-identity/RUN/efuse.json
projects/device-identity/target/debug/identityctl bridge-config-audit --capture private/device-identity/RUN/usb-rom
projects/device-identity/target/debug/identityctl flash-metadata-audit --input backups/device-identity/RUN/flash-original.bin
```

Admission is all-or-error, with static diagnostics and no partial successful
report. CRCs and manifests detect inconsistencies; they authenticate neither
the hardware nor authorship. Adjacent backups were not collected as a controlled
persistence cohort. Opaque blobs, erased records and non-NVS data can contain
credentials; zero name matches cannot establish their absence. No measured
privacy, uniqueness, postboot health or persistence guarantee follows.

Primary sources: [Espressif USB serial reference](https://docs.espressif.com/projects/esp-iot-solution/en/latest/usb/usb_overview/usb_device_const_COM.html),
[AN978 revision 1.5](https://www.silabs.com/documents/public/application-notes/an978-cp210x-usb-to-uart-api-specification.pdf),
and the pinned [NVS parser](https://github.com/espressif/esp-idf/tree/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/nvs_flash/nvs_partition_tool)
and [partition generator](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/partition_table/gen_esp32part.py).
