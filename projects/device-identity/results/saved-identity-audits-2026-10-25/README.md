# Saved USB and flash metadata audits

The single complete ROM-state native USB serial matches the factory MAC's
colon-separated representation. The saved 678-byte version-one CP2102N
configuration validates with little-endian ConfigSize and big-endian Fletcher16
over bytes 0..676. All three retained original 8-MiB backups pass partition-table
and supported physical NVS integrity checks. No new device access belongs to
these audit commands.

These are retrospective results from already explored inputs. The
[method](../../experiments/saved-identity-audits.md), consumers and controls were
frozen at `4eb96eb` before production derivation. The original collection methods
and their limitations remain unchanged. Both failed original-application USB
cohorts remain rejected by both derivative commands; their prefixes were not
substituted for complete replay.

The [native relation](native-factory-relation.json) and
[bridge integrity](bridge-integrity.json) reports require whole four-file USB
replay first. An independent decoding/modulo-checksum check agrees. This one
ROM capture establishes neither native descriptor repeatability across power
cycles nor a supported customization API. Checksum validity establishes neither
bridge lock state nor programming permission. The documented legacy GetLockValue
support list excludes CP2102N, so no such query or write-to-test-lock was used.

| Original backup | Programmed NVS pages | Empty pages | Written heads | Written slots | Erased slots |
| --- | --- | --- | --- | --- | --- |
| [Before runtime-table trial](flash-runtime.json) | 4 | 2 | 6 | 67 | 436 |
| [Before AP witness](flash-ap.json) | 5 | 1 | 6 | 67 | 498 |
| [Before AP restart](flash-restart.json) | 5 | 1 | 6 | 67 | 498 |

Each image has three partitions: one application, one NVS and one other data
partition, with a valid table MD5. Each NVS has one namespace, two integer heads
(including its namespace declaration), two blob-data heads and two blob-index
heads. Physical heads include repeated versions; no logical values or blobs
were reconstructed. Fixed key-name heuristics matched two network/address
heads and zero credential/device-identity names. **Credential absence remains
unqualified:** opaque blobs, erased records and other partitions were not
audited for secrets. No encryption-absence proof follows from these checks.

The later two metadata reports are equal; the earlier page/slot counts differ.
These backups were not collected as a controlled persistence cohort. Metadata
equality is not full-image equality, and differing counts do not identify their
cause or establish application persistence behavior.

Both runs of each of the five release commands produced byte-identical reports;
all 14 original input files remained unchanged. Independent checks against the
pinned SDK parser agree on all 18 written heads, slot/type counts and supported
page/header/payload CRCs, using NVS slices verified against their original images.
One retained check-helper failure used the wrong location for the SDK's payload
CRC pair; correcting the helper changed no production source or original input.
The [derivation](derivation.json), [tool identities](artifacts.json) and
[independent checks](independent-checks.json) record the scopes.

Software qualification includes synthetic SDK v1/v2 fixtures, recomputed-CRC
semantic corruptions, late-partition failures, partition-local namespace checks,
all 5424 one-bit CP configuration mutations and complete-USB receipt controls.
An independent reviewer reproduced contrary controls before the freeze.
Private originals, raw values, failed attempts and full-flash hashes remain
ignored. The previous full-flash restoration boundary is unchanged; these
offline checks provide no new physical boot, health, persistence or RF evidence.

`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. Hashes establish retained-byte integrity, without authenticating
hardware origin or authorship. Exact original replay needs the private inputs.
