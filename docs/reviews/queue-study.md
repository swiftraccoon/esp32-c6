# Queue study scheduling and analysis review

Reviewed 2026-10-23 against the [coding standard](../coding-standard.md),
[all 54 API entries](../rust-api-checklist.md) and the existing
[benchctl matrix](benchctl-v2.md). Scope: `queue_study.rs` and its CLI wiring.
This internal Rust 1.96 host binary retains its existing dependency, license
and metadata decisions. The previous matrix's dispositions remain applicable;
there is no new exception or public library API.

Affected applicable entries are C-CASE/C-WORD-ORDER/C-GETTER (named backends,
explicit tick and timestamp fields); C-COMMON-TRAITS/C-SERDE (strict typed
schedule, equality and serialization); C-GOOD-ERR/C-FAILURE/C-VALIDATE
(regenerated schedule, raw replay, manifest/image/provenance validation,
chronological capture order and distinct failed/missing/invalid attempts);
C-NO-OUT/C-CALLER-CONTROL/C-GENERIC (borrowed paths and returned values);
C-INTERMEDIATE (all paired attempts, raw/report hashes and incomplete status);
C-NEWTYPE/C-CUSTOM-TYPE/C-STRUCT-PRIVATE/C-HIDDEN (private schedule/run types);
C-DEBUG/C-DEBUG-NONEMPTY (field-bearing diagnostics); C-CRATE-DOC/C-LINK
(module, CLI and design references); and C-SEND-SYNC (single host context,
no shared mutable state in production).

No unsafe, FFI, async, authored traits, macros, smart pointers, production
destructors or dependency change was added. Standard filesystem operations
block outside device timing. Test-only fixtures exclusively create unique
temporary directories and remove only those owned paths. Project licensing
and the existing serialport dependency exception remain as previously recorded.

The fixed permutation algorithm is reproducible, uses rejection sampling and
balances two AB/two BA blocks per session. It supplies collection order, not a
cryptographic or statistical-independence guarantee. Regeneration rejects
altered paths/counts/order before capture files are read. Schedule creation
refuses overwrite.

Analysis requires `study-manifest.json` to freeze the schedule hash, both
ELF hashes and their provenance hashes. Each admitted report is checked against
raw replay, the frozen image/provenance and increasing scheduled timestamps.
Configuration must match throughout the admitted study. Nonces are checked
across admitted captures; a failed partial boot may not supply a nonce and does
not become an independently validated boot. Fixture invalidations/repeats need
their explicit external attempt ledger, not automatic success replacement.

Only all 12 complete scheduled blocks produce the conditional X(3)..X(10)
median interval. Signed even-sample medians sum through `i128` before conversion;
raw integer contrasts are retained. The output states the IID assumption and
actual 96.142578125% continuous coverage. It does not certify that sessions
are IID, report an event-tail confidence bound or correct calibration error.

Independent integration review checked the estimand/arithmetic and identified
the frozen-manifest and timestamp requirements, which were incorporated before
capture. End-to-end synthetic tests replay all 24 boots, then exercise failed
blocks, time-order corruption and frozen-image corruption. They use the actual
protocol fixture, not an imitation parser. Other tests check balancing, schedule
tampering, overwrite rejection, a fixed permutation vector and signed medians.

Verification: workspace tests passed **35 unit tests and two doctests**;
benchctl formatting, workspace Clippy with `-D warnings`, workspace docs and
release build passed. The log is retained locally at
`.cache/queue-host-checks.log`. These checks establish software behavior, not
physical flashing, timer accuracy or independence of captured trials.

All **35 historical raw transcripts** from the initial, clocked-handoff and
alignment datasets replayed successfully with the new release collector. Their
complete validation JSON was byte-identical to each archived `replay.json`.
