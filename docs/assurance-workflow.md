# Evidence and experiment gates

The full-artifact commands below apply to complete original evidence bundles.
For this source and textual-result checkout, run
`target/release/benchctl verify-public-snapshot --root . --manifest quality/publication.json`.
Its [scope](reviews/public-snapshot.md) excludes unavailable original binaries
and historical Git ancestry; it does not replace the gates for new experiments.

Adopted **2026-10-23** following the [feedback audit](reviews/2026-10-23-feedback-audit.md).
These gates apply to new work. They do not retroactively establish that they ran
before historical captures or turn incomplete objectives into completed tests.

## Before comparative collection

1. Record the decision question and closest prior test. Specify every measured
   metric's start/end, units/clock, aggregation, observer policy, correctness
   check, configuration and limits. Trace conditional compilation and actual
   executed branches, including allocation/waiter paths. A source print call
   alone does not establish output from a selected build.
2. Build and preserve exact ELF, provenance and configuration identities. Check
   expected output formats in the actual compiled image and an admitted
   qualification transcript. A required string's presence is necessary for a
   literal-output check, not proof of execution or endpoint correctness.
3. Qualify the method with a passing control and deliberately failing controls.
   Investigate an unexpected qualification failure before repeating collection.
   Do not repair a frozen acceptance rule by admitting its failed attempts.
4. Have a separate reviewer construct contrary tests, reproduce outcomes and
   check the source/configuration/measurement agreement. Retain concrete tests,
   commands, results and unresolved objections. A reviewer name or agreement is
   insufficient. Review artifact identity is machine-checked; independence and
   scientific interpretation still require actual review.
5. Commit the prospective schedule, input digest set and method definition;
   record that freeze commit in the ledger. Preserve board identity, the current
   private full-image backup, flash-transfer evidence and restoration separately.
   The host method gate does not flash or authenticate the board.

For the supported JSON collectors, capture purpose is mandatory:

```sh
target/release/benchctl capture-queue --purpose qualification \
  --port /dev/cu.SERIAL_PORT --output .cache/new-pilot \
  --backend freertos-queue --elf path/to/firmware.elf \
  --provenance path/to/provenance.json --boot-id pilot-01 --run-order 1
```

This command resets the device; it is an example, not an instruction to run
against an unprepared board. It produces a schema-4 report marked
`qualification`. Such pilots are excluded from comparative study analysis.
Do not label a native console smoke test a supported JSON qualification.

The current method definition is deliberately **retrospective**: it reuses
the existing IDF queue pilots to run new offline controls for future captures
of those exact images. It does not claim the gate ran during the old study.

```sh
target/release/benchctl qualify-method \
  --method quality/methods/idf-queue-v1.json
```

This checks both positive controls through the actual protocol validators,
artifact/report consistency and the full metric-ID set. For each variant it
requires rejection of a pre-JSON panic, reboot, additional ROM startup, missing
completion and corrupt checksum, including the expected rejection category.
It also checks required ELF/output text and pinned source/review/configuration
files. The output is `controls_passed`, not a hardware or platform verdict.

A comparison capture additionally needs `--purpose comparison --method <path>`.
It must match the qualified wire/backend, ELF and provenance before serial
access. The inputs are rechecked before capture. A changed artifact or method
requires its own reviewed revision and controls.

New queue studies use manifest schema **2**, retaining the existing balanced
schedule format and image map and adding a frozen definition identity:

```json
{
  "schema_version": 2,
  "experiment": "idf-native-queue-v1",
  "schedule_sha256": "<actual schedule SHA-256>",
  "method_definition": {
    "path": "quality/methods/idf-queue-v1.json",
    "sha256": "<actual method definition SHA-256>"
  },
  "images": "<same std/native image-map structure as the existing study>"
}
```

This shows field structure, not a runnable manifest. The analyzer reruns method
controls and requires schema-4 comparative receipts matching each backend's
frozen method, ELF and provenance. Removing purpose does not make a new pilot
historical. Legacy schema-1 studies/schema-3 reports are accepted only with the
known historical collector identity; their original outputs remain unchanged.
That compatibility is not retrospective qualification. Editing all metadata
to impersonate another collector is outside authenticity guarantees.

Native suites without a reviewed adapter remain qualification-only evidence.
No new Zephyr v2 profile is authorized by the IDF method definition. Its failed
v1 contract, image warning and endpoint map retain their separate status.

## Claims and coverage

Update [quality/evidence.json](../quality/evidence.json) and the
[ledger guide](evidence-ledger.md) when evidence changes. Keep source support,
build success, observed output, operator reports, independently reproduced
results, failures and unmeasured work distinct. Pin supporting files and state
scope/limits; do not promote an operator claim merely by changing its label.

All platform/domain pairs expand from explicit unknown/not-run defaults plus
scoped overrides. Partial CPU/UART observations do not qualify an entire
subsystem. Runtime utilization, capacity, memory peaks, radio, sleep/energy,
recovery and maintenance gaps remain visible. No overall score is inferred.

## Packaging and verification before a finalized commit

Exercise the complete derive/serialize/save/read/replay path with fractional
statistics and boundary cases, alongside transcript controls. Uniform fixtures
with integer means missed the [load reader defect](reviews/idf-load-replay-correction.md).
Keep exact integer sums/counts, check saved numeric round trips and require
rejection of a changed report through the same archive path used for final QA.
When that path fails, retain the original failure and artifacts, diagnose the
layer responsible, and version a correction separately from frozen inputs.
Do not weaken equality or rewrite observations to make a replay pass.

Use a `collecting` archive entry during work. Finalization requires a complete
`SHA256SUMS` over every retained regular file except itself, with an explicit
repository-relative or archive-relative path convention. Do not silently omit
unexpected files or substitute paths when resolving a manifest.

`INPUTS.sha256` is immutable once frozen. Its current bytes must match the
recorded Git commit, and listed input files must still match those committed
digests. This binds a previously ignored generated file by its frozen hash;
it does not require pretending that file was tracked in the original tree.
Use full Git history. Interpretation corrections preserve original data/reports,
add a versioned explanation and reconcile the completed archive hashes.

The initial dataset's new packaging is explicitly retrospective. Sealing its
current bytes does not create a pre-collection freeze or fix its initial clock
uncertainty.

Stage complete files before the final checks so an untracked local link cannot
be mistaken for committed evidence:

```sh
cargo +1.96.0 fmt --all -- --check
cargo +1.96.0 test --workspace --locked
cargo +1.96.0 clippy --workspace --locked --all-targets -- -D warnings
cargo +1.96.0 doc --workspace --locked --no-deps
cargo +1.96.0 build --release --locked -p benchctl
target/release/benchctl verify-evidence --root . --ledger quality/evidence.json
target/release/benchctl replay-archives --root .
target/release/benchctl qualify-method --method quality/methods/idf-queue-v1.json
git diff --cached --check
```

The verifier checks archive discovery/inventory, identities, frozen manifests,
typed claim support, coverage structure and its documented local Markdown-link
subset. Replay checks preserve failed-capture status and compare successful
protocol outputs and study summaries. Commands are read-only: they never repair
an archive, update a hash, reset a board or manufacture an observation.

[CI](../.github/workflows/assurance.yml) runs the source checks with Rust 1.96.0
on Ubuntu and the explicit public-snapshot check described above. The original
full-history/artifact commands remain available for complete experiment bundles;
this checkout cannot run their complete checks. No board is
needed for these host commands; fresh dependency/tool provisioning uses the
network. The workflow definition is not evidence a hosted run passed.
Repository branch protection/required-check settings are not configured by
adding this file.

No check establishes semantic correctness, reviewer identity, independent
oscillator accuracy or absence of unrecorded failures. Report exactly what
passed, the contrary tests exercised, and what remains unverified. The
[independent assurance review](reviews/assurance-independent.md) records the
implemented checks and executed environments without a completeness assurance.
