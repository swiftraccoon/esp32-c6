# Retrospective beacon field comparison

Decision question: which selected beacon-body bytes were identical within and
between the two existing controlled SoftAP boots when the private nonce SSID
changed? This is a retrospective analysis of the complete two-trial cohort in
[the AP witness](../results/wifi-ap-2026-10-25/README.md). Both boots used the same
compiled image, AP MAC, channel and other requested configuration. No new RF
collection, board access, address rotation or receiver change belongs to this
analysis.

The earlier matcher established 64 exact nonce/TA/BSSID and CRC matches but
treated vendor information elements as opaque. Preliminary inspection has
already exposed the element shape and apparent equality of non-SSID contents;
these are inspected data, not a held-out test. Freeze the exact comparison rule,
consumer and executable before the recorded derivation. Do not recast that
analysis freeze as prospective registration of the earlier physical trials.

The [prior-work catalog](../../../docs/research/prior-work.md) and
[reuse catalog](../../../docs/research/benchmark-reuse.md) motivate checking
collateral fields independently of address setters. Reuse the strict Rust
PCAPNG/radiotap/beacon validator and the complete trial replay, rather than
introducing a second packet decoder or collecting another favorable sample.
No population, published fingerprint or performance comparison is made.

## Inputs and admission

Use both original, immutable 19-file trial bundles from the AP witness, all
32 packets in each. Preserve their original collection commit, tool identities,
private seals and raw data. The new analysis has its own source/artifact inputs
and result seal. Do not add files to the original trial directories or rewrite
their summaries, receipts or checksums.

The production `identityctl wifi-ap-compare-profiles` command must replay both
complete trials before deriving private nonce SSIDs and the synthetic AP MAC.
Require the exact saved-file inventories, raw/transcript/receipt agreement,
receiver readiness, active/Stop readbacks, request and artifact bindings and
saved-summary equality already enforced by `wifi-ap-replay`.

The shared comparator then validates the entire PCAPNG inputs. Every packet in
each nonempty input must be a supported ordinary beacon with an included
verified CRC and exactly its own expected SSID and transmitter/BSSID. A mixed
capture, wrong target, unchecked/bad FCS or late malformed record fails analysis;
there is no selection of favorable packets. For this cohort, require exactly
32 admitted beacons per side. Passing structural checks do not authenticate the
capture origin or physical transmitter.

## Exact comparison rule

Compare bytes privately, without publishing values or hashes:

- The complete two-byte beacon interval and two-byte capability fields,
  independently.
- The complete ordered information-element TLV stream after removing only the
  entire SSID element. Preserve element IDs, lengths, values, duplicate
  occurrences and ordering. Include unknown and vendor elements without
  interpreting their payloads.
- The complete ordered subsequence of vendor TLVs.
- For each element ID present on either side except SSID, its ordered sequence
  of complete occurrences within each beacon. Absence is a distinct empty
  sequence. Separate per-ID equality does not establish global element order;
  the full non-SSID stream checks that order.

Count distinct encodings within each side and their exact intersection.
`identical_across_all_beacons` means the union across both nonempty sides has
exactly one encoding, rather than merely matching the first packet. A difference
is a valid comparison result; malformed or inadmissible data is an error. Count
per-ID presence by beacon, retaining repeated occurrences in the compared bytes.
Targets are compared separately as equality booleans so that the unchanged AP
address and changed SSIDs remain explicit.

Exclude radiotap/PCAPNG metadata, the complete management header (including
addresses, duration, sequence and flags), TSF timestamp and FCS from these
specific field comparisons. Their exclusion does not establish equality,
independence, safety or absence of identifying information in those fields.
Do not decode vendor payloads into a device identity or search for selected
factory values and infer that all other identities are absent.

## Qualification and derivation

Before the recorded derivation, run the pinned Rust 1.96.0 host checks and
qualify the actual consumer with passing and contrary synthetic inputs. Cover
SSID-only changes, excluded TSF/sequence changes, interval/capability changes,
late opaque vendor/unknown-element changes, reordered/repeated occurrences,
different TLV boundaries, absence, mixed packets, wrong targets, invalid or
unchecked FCS, malformed late records and full-bundle receipt failures. Check
both container byte orders and omission of private sentinels from outputs and
errors. A separate reviewer constructs contrary controls and checks the actual
release consumer. Retain unexpected failures before correcting the method.

Freeze source, method, executable and input identities in Git. Hash every file
in both original trial bundles before analysis. Run complete replay on each,
derive through the release comparison command, repeat it and compare all output
bytes. Independently decode all original packets with the pinned TShark tool,
preserving repeated JSON keys, and compare the same byte sequences without
sorting or discarding duplicate elements. Check that all original files and
the existing seals are unchanged afterward. Retain commands, statuses and
contrary inputs privately.

Publish only fixed counts, element IDs, equality states, code/tool identities
and the scoped interpretation, with a complete relative-path checksum seal.
Actual addresses, SSIDs, opaque payloads, raw field values and their hashes
remain private. The CLI performs offline reads and produces no board recovery
claim; the earlier restoration boundary remains the AP witness's record.

## Interpretation

Equality would show repeated bytes in these two boots of one configured device.
It would not establish uniqueness, fingerprint accuracy, tracking success,
population prevalence, behavior after an AP MAC change, RF silence or a measured
privacy improvement. The unchanged synthetic AP address already links the two
boots. Distinct SSIDs alone do not test address rotation. Configuration, software
version, client state and environment can alter the compared fields; two bounded
prefixes do not characterize all future traffic or the entire AP lifetime.
