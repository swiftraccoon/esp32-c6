# SoftAP identity integration observation

Decision question: does the new library integration emit its generated AP
address and SSID, change both across a checked Stop, and prepare a different
startup identity after reset? This is a prospective presence method. No
anonymization-project board trial has run. The first implementation's external
receiver was unreachable, so collection prerequisites remain unmet.

Reuse the [existing AP method](../../device-identity/experiments/wifi-ap.md),
its bounded receiver/CRC decoder and stock flash tools. The old whole-trial
collector hardcodes synthetic addresses and cannot admit these generated
identities unchanged. Compose existing public worker/decoder APIs and qualify
that integration without changing historical acceptance rules.

Before collection, qualify the actual live/save/replay composition with valid
and contrary controls: wrong/reordered/repeated requests, stale readiness,
wrong target/SSID, invalid FCS, malformed late records, no witness, child timeout,
lost acknowledgment and failed cleanup. Freeze the exact composition, method,
library/example, SDK/build, decoder and input identities. Host unit controls
do not qualify UART execution, SDK fault behavior or receiver delivery.

Require the sole intended USB pair, matched factory baseline, free UART and a
freshly checked receiver profile/channel/provenance. Preserve and separately
verify all 8 MiB of original flash before any segmented write on this board.
Keep complete original backup, write/restore receipts and raw packets private.

Use three separate bounded captures across two boots:

1. Boot one: read the prepared A/B MACs and SSIDs through one retained UART
   handle. Establish receiver custody before requesting StartFirst. Require
   a zero-status/no-failure active reply and exact expected readback, then a
   CRC-valid beacon whose transmitter, BSSID and SSID match A. Request and check Stop.
2. Keep the same UART and initialized driver. Establish a second receiver
   capture, request StartSecond, and require the corresponding B readback and
   independent beacon. Request and check Stop. Both A/B MACs and SSIDs must differ.
3. Perform one explicit C6 reset and obtain boot two's new prepared pair.
   Start its first epoch C with a third receiver capture and the same checks.
   Require A/B/C addresses and SSIDs pairwise distinct and locally administered
   unicast addresses distinct from the saved factory identity. Request Stop.

Do not start when the bounded receiver already ended on ambient traffic.
Any missing, extra, reset/noise, late, failed or incomplete operation fails the
whole affected trial; cleanup does not rehabilitate it. After success or failure,
restore and independently verify the complete original flash, then perform a
matched normal reset. Failed recovery remains explicit and stops further trials.

Replay complete captures, requests/replies, worker provenance, causal receipts,
expected-target bindings, all frame validation and saved summaries independently.
Retain all cohorts including failures and seal each complete result archive.
Publish only sanitized outcomes and counts.

The accepted claim would be generated-identity **presence** in these selected
frames and SDK observations. It would not establish factory-address absence,
continuous startup/transition coverage, RF silence, cold-power persistence,
unlinkability or full anonymity. Those claims require additional qualified
observers and separately defined experiments. Native USB/bridge, immutable
identities, credentials, paired-peer state and other radios remain outside this
first integration's control.
