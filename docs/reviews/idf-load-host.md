# Offered-load host implementation review

Reviewed **2026-10-23** against the [coding standard](../coding-standard.md),
[all 54 API entries](../rust-api-checklist.md), and existing
[accounting host dispositions](idf-accounting-host.md). This is the affected
entry delta for [load.rs](../../tools/benchctl/src/load.rs), CLI integration,
the protocol-5 archival dispatch and operator examples. It does not establish
a firmware build, electrical reset or on-board pass.

## Behavior and evidence boundary

The [qualification contract](../experiments/idf-load-qualification.md) and
[exact schema](../experiments/idf-load-protocol.md) precede collection. Both new
CLI commands require an expected statistics profile. Capture also requires
explicit qualification purpose; comparison rejects before reading artifacts,
creating outputs or opening serial. The default timeout is 120 seconds because
2,016 sample rows are printed after the service phases, at 115200 baud.

The validator consumes exactly 2,043 ordered records: hello/queue control,
six phases with twelve settled snapshots and every nominal row, phase-end
receipts, and complete. It denies unknown fields and checks exact definitions,
global/local indices, statuses, checksums, queue bounds, serial FIFO service,
nominal-slot dispatch, callback/early/terminal-wake counts, quiescence, drain and
deadline chronology. Work checksums are recomputed through the shared checked
Rust workload with the retained global index. Nonzero work requires positive
service duration; zero-round controls may legitimately quantize to zero.

Disabled clock/total/task-counter fields must explicitly be null. A private
nullable wrapper deserializes through `deserialize_any`, so an omitted field
cannot silently acquire Option's missing-field default. Enabled counters remain
u64, including values above u32, with valid absolute clocks and monotonic stable
task identities. Both profiles retain common collection brackets, heap and
stack-fill indicators. No raw task-name pointer or firmware resource is handled
by the host module.

The queue starts empty each phase. In addition to depth 1–8 for accepted offers
and eight for full rejection, necessary history checks remove prior accepted
jobs whose service starts are strictly before the current send-entry: those
jobs were already dequeued. Equal timestamps stay ambiguous. Full requires a
possible-queued upper bound of at least eight; successful depth is bounded by
that upper bound plus the new accepted job. These checks do not reconstruct
exact insertion/dequeue instants or completion-publication backlog. The claimed
immediate queue-query meaning still depends on
the reviewed task priorities and firmware path.

Terminal-status rows reconstruct offered/accepted/full/missed/completed/drain
and synthetic-deadline counts. Nearest-rank distributions are conditional on
observed attempts/completions. Entirely missed work pulses remain adverse,
generator-limited observations with null distributions; they are not replaced
or assigned invented zero latencies. No-work and forced-delay controls have
their own strict acceptance, separate from adverse work-phase outcomes.

Settled task deltas retain signed boundary residuals. They cover generation,
drain and boundary work rather than exact in-window CPU. Observer brackets are
gross and never subtracted. Neither profile is an uninstrumented application,
and no universal overhead, physical CPU percentage, sustained capacity, global
memory peak or comparative estimator is calculated.

## Capture, failure and archival ownership

Capture preserves original ELF/provenance bytes and their admission hashes,
raw serial input, accepted parsed records and a schema-4 report. Inputs are
reread and checked before serial access. A started failure retains its failure
and available raw input when output remains writable; no successful completion
or favorable phase summary is manufactured. Filesystem failures can prevent a
final report, and do not become passing observations.

Actual capture and replay use the existing bounded, permanently rejecting
single-boot/fatal-text guard. The guard consumes the received completion chunk;
it cannot certify diagnostics arriving after capture returns. Reset metadata
does not authenticate flashed hardware or prove statistical independence.

Whole-archive replay recognizes protocol 5, checks qualification-only admission,
expected profile, no benchmark/comparative score, retained ELF/original
provenance identities and parsed provenance equality. It compares every derived
report field against the actual raw replay and requires byte-identical saved
replay output. Failed load records stay failed and cannot retain favorable
phase summaries. Historical protocol 1–4 behavior is preserved; final regression
over their actual archives is recorded by the integration owner.

## API applicability delta

Existing all-54 dispositions and package/license decisions were reused. No
dependency, unsafe operation, FFI, async/ISR context or exported Rust library is
added. Host allocation and blocking filesystem/serial operations are outside
the device's measured service path. The compiler/edition baseline remains
Rust 1.96.0 / 2024.

| Entries | Evidence / decision |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: explicit profile/boundary/status enums and units in field names distinguish nominal releases, attempts, service and drain. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: field-bearing Debug/serde records and Copy/Eq only for small metadata enums; no live hardware owner is copied. |
| C-SERDE, C-VALIDATE | Applies: strict typed JSON, required explicit nullable fields, checked profile/definitions/identities/causal clocks and irreversible rejection before summaries. |
| C-CUSTOM-TYPE, C-NEWTYPE | Applies: profile/boundary/status enums and the nullable wrapper prevent absence, unavailability and measured zero from becoming interchangeable. |
| C-GOOD-ERR, C-FAILURE | Applies: contextual Results for parse, wrong profile, impossible timestamps, checksum, stale inputs, port and archive failures; ordinary invalid input does not panic. |
| C-RW-VALUE, C-CALLER-CONTROL, C-GENERIC | Applies: replay takes a generic reader by value; the capture-stream helper accepts generic readers/writers by value, allowing callers to pass mutable borrows. Paths remain borrowed. |
| C-SEND-SYNC | Applies: one host validation context owns parsed observations; no authored unsafe sharing or synchronization. Temporary-test names use a relaxed atomic solely for uniqueness. |
| C-NO-OUT, C-INTERMEDIATE | Applies: checks return structured data without repairing/resealing evidence or emitting substituted favorable outcomes. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE, C-QUESTION-MARK | Applies at private application scope: module/CLI docs and realistic operator examples state reset effects and observation limits. Examples are not claimed physical executions. |
| C-STRUCT-PRIVATE, C-HIDDEN | Applies: raw definitions, validator stages and summary internals remain private; only crate-level CLI/profile/replay integration is exposed. |
| C-CTOR, C-METHOD | Existing application decisions apply; test `Temporary::new` creates an exclusive owned directory, while domain validation/capture are fallible named operations. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No production destructor added. Test cleanup attempts removal only of its exclusively created temporary directory and ignores cleanup failure. |

Other dispositions remain unchanged: no authored macro, custom collection,
iterator API, conversion/operator family, smart pointer or representation
commitment is introduced. Private host metadata does not inherit firmware RAM,
interrupt or cancellation guarantees.

## Executed host checks

The final targeted commands passed locally:

```sh
cargo +1.96.0 fmt --all
cargo +1.96.0 test --locked -p benchctl load::tests -- --nocapture
cargo +1.96.0 test --locked -p benchctl accounting::tests -- --nocapture
cargo +1.96.0 clippy --locked -p benchctl --all-targets -- -D warnings
cargo +1.96.0 build --locked -p benchctl
```

All **15 targeted load tests passed**. The full synthetic positive has all 2,016
nominal jobs/2,043 records for each profile, queue saturation and post-window
drain. Further positives preserve an entirely missed work pulse and a negative
signed accounting residual. Fixed percentile vectors retain real zero samples
and null empty subsets.

Counterexamples challenge absent explicit-null fields, wrong profile/width,
missing/duplicate IDs, impossible early-full history, replayed release slots,
payload checksums, FIFO overlap, no-work/forced-delay controls, callbacks and
quiescence, counter/lifecycle/role/stack/heap/observer corruption, malformed
fields, cleanup and missing completion. Fake transports exercise panic, reboot
and additional ROM startup at one-, seven- and 1,024-byte boundaries through
both actual capture and replay. CLI preflights reject missing purpose/profile
and comparison; a deliberately nonexistent port retains a failed attempt and
original inputs without a device operation. A temporary archive additionally
tests profile, score, provenance/report drift and failed-status preservation.

Some initial mutation fixtures hit an earlier valid guard instead of the
intended assertion. They were corrected to keep the other fields consistent
and exercise the intended no-work, forced-delay and post-drain checks. The
necessary empty-start history guards also required the no-work-loss fixture to
exercise a genuine missed release rather than an impossible full queue. No
acceptance rule was weakened and these fixture failures were not physical runs.

Independent CLI mutation found a separate real preflight consistency defect:
the initial cumulative-accepted bound admitted a full queue after ten prior
jobs had all started long before the current offer. The implementation owner
reproduced its successful exit. The conservative proven-start subtraction above
now rejects that case and an analogous inflated successful depth, with dedicated
regression cases. This correction preceded physical collection/input freeze.

A second new-load defect concerned failure recording rather than a passing
result: an arithmetic error during post-stream summary derivation could bypass
the final report write and leave `running`. The actual capture path now derives
its checked result before outcome recording; derivation errors permanently reject
the boot and retain `failed` with no phase/qualification/score. A complete fake
transport with overflowing summed counters exercises that path.

The superficially similar older accounting code was investigated rather than
assumed defective. Its complete-record admission already calls `report_fields`
before acknowledging success. The adversarial arithmetic fixture therefore
rejected inside its capture stream, disproving that analogous gap. A test-only
regression retains that existing protection; all **13 accounting tests passed**.
No production accounting logic,
frozen source copy or historical report was changed. Existing applicability
decisions remain unchanged for that added host-only malformed-input fixture.

Workspace-wide checks, historical/current physical archive replay, release
build, independent review and on-board qualification are separately recorded
by the integration owner. This review makes no hosted Linux or physical pass
claim from these local host tests.

The current debug collector also reproduced all **64 historical protocol
transcripts and the existing study summary** after integration. Final release,
repository packaging and new physical attempts remain separately recorded.

Final whole-archive QA later found a default serde_json reader rounding defect
on an actual fractional mean. The subsequent
[exact replay correction](idf-load-replay-correction.md) records the precise
one-ULP false rejection, pinned feature change, unchanged original artifacts,
affected C-SERDE/C-VALIDATE decisions and two failing-then-passing regressions.
Its 17-test count includes the original 15 tests and those two additions.
