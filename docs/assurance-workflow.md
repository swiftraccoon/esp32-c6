# Evidence and experiment gates

These requirements apply across the workspace. Each project owns its methods,
qualification controls, claims and results. The
[OS workflow](../projects/os-evaluation/docs/assurance-workflow.md) describes
the historical benchmark schemas and concrete commands; wireless defines its
[observation contract](../projects/wireless-explorer/docs/observation.md) and
[functional evidence](../projects/wireless-explorer/results/README.md).

## Before collection or comparison

1. Record the decision question, closest prior test and reuse decision. Define
   inputs, target/configuration, timed endpoints, units/clock, aggregation,
   observer effects, correctness checks and limits. Trace the selected compiled
   branches rather than inferring execution from source text.
2. Preserve exact source, SDK/toolchain, configuration and artifact identities.
   Qualify the actual capture/derive/save/replay path with passing and deliberately
   failing controls, including fractional/boundary data where relevant.
3. Investigate unexpected qualification failures before repeating collection.
   Preserve failed attempts and their acceptance rules; corrections get their
   own version and evidence.
4. Have a separate reviewer construct contrary controls, reproduce outcomes and
   check agreement among source, configuration, observations and claims. Retain
   concrete commands/results and unresolved objections.
5. Freeze the prospective schedule, method and input digests before comparative
   collection. Record its commit in the project ledger. Preserve the private
   full-flash backup, transfer verification and restoration boundary separately.
   Host controls do not flash, authenticate or qualify the board.

## Claims, packaging and privacy

Distinguish source support, builds, observed output, operator reports, reproduced
results, failures and unmeasured work. State missing coverage explicitly; no
overall score follows from a partial subsystem check. Keep project results and
ledgers under their owning project. The
[OS ledger](../projects/os-evaluation/quality/evidence.json) retains its original scope and bytes.

Use a collecting state while an archive is incomplete. Finalization needs a
complete checksum manifest over every retained regular file except itself and
an explicit path convention. Frozen input manifests must match their recorded
commit; changing a source location does not make current bytes original inputs.
Preserve failed attempts, raw inputs and versioned interpretation corrections.

Apply the [privacy gate](privacy.md) before staging/publication. Raw ambient
captures, device identifiers, personal paths and full flash images remain in
ignored private storage. Publish explicitly sanitized derivatives with their own
identities. Evidence retention never authorizes publishing personal data.

## Verification

Run target-appropriate formatting, builds, lints, documentation and meaningful
tests. Stage complete files before checking inventories and links. Recheck
archive seals and replay saved observations through the actual consumer;
a failed capture must remain failed. Read-only checks must never repair data,
rewrite a freeze, reset hardware or manufacture an observation.

From the repository root, verify the available historical OS data and workspace
source inventory with:

```sh
cargo +1.96.0 build --manifest-path projects/os-evaluation/Cargo.toml --release --locked -p benchctl
projects/os-evaluation/target/release/benchctl verify-public-snapshot \
  --root . --ledger projects/os-evaluation/quality/evidence.json --manifest quality/publication.json
```

This [limited verifier](../projects/os-evaluation/tools/benchctl/docs/api.md) checks source identities,
declared historical lineage, local links and retained raw replays. It cannot
replace omitted executables, original freeze ancestry, full-artifact admission,
physical measurements or each project's semantic validator.
[Relocation mappings](organization.md) apply only to public-snapshot verification;
full method checks still require original inputs.

The OS evidence verifier also accepts project ledgers at
`projects/<project>/quality/<file>.json`, with archives directly under that
project's `results/`. Its schema and qualification rules remain the
[OS contract](../projects/os-evaluation/docs/assurance-workflow.md); other project
schemas require their own validators.

[CI](../.github/workflows/assurance.yml) checks workspace utilities, OS host tools
and wireless host tools independently. Hosted CI configuration is not evidence a
hosted run passed. Record executed commands, outcomes, unrun checks and their
limits. No check proves semantic correctness, anonymity, clock calibration,
reviewer independence or absence of unrecorded failures.
