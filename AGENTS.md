# Repository working agreement

Read [docs/coding-standard.md](docs/coding-standard.md) before writing or changing
code. Read its [local guide copies](docs/standards/README.md); following the
standard must not require a download. The [source lock](docs/standards/sources.json)
governs the adopted revisions. Use the documented explicit updater for upstream
default-branch refreshes, review the changes, and commit the new pins together.
Live guide updates and toolchain upgrades do not silently change the standard.

- Prefer Rust for firmware, benchmark logic, shared libraries, and host tools.
  Discuss a necessary C, C++, assembly, or other authored implementation with the
  user before implementing it. Evaluating unmodified upstream operating systems
  and recording their existing languages does not require rewriting them.
- Apply the official Rust source/manifest conventions and every applicable Rust
  API Guidelines checklist entry. Use
  [docs/rust-api-checklist.md](docs/rust-api-checklist.md) to record the relevant
  `C-*` identifiers, applicability decisions, and evidence for each code change.
  Formatting alone does not establish API or safety compliance.
- Treat the Google guide as supplemental under the precedence and embedded
  applicability rules in the standard. Chromium, Android, and Fuchsia rules do
  not become ESP32-C6 requirements because they appear in that guide.
- Keep hardware-specific unsafe/FFI code small and document its invariants. Check
  concurrency, interrupt context, allocation, blocking, panic behavior, and
  cancellation against the selected runtime and target.
- Keep benchmark workloads comparable and preserve raw measurements and build
  provenance. Label unsupported features, untested claims, and missing results
  explicitly. Never present a plan, build, or simulator run as an on-board pass.
- Before adding a benchmark, consult [prior work](docs/research/prior-work.md)
  and [reusable assets](docs/research/benchmark-reuse.md). Record its decision
  question, closest existing test, permitted reuse and useful replication or
  extension. Reconcile source, configuration, units and timed endpoints before
  using published numbers. This adds no approval step for authorized work.
- Run checks appropriate to the actual target and change. Record commands,
  results, and checks that could not run; do not invent coverage or write tests
  that merely restate an implementation.
- Apply [the evidence and experiment gates](docs/assurance-workflow.md) before
  new comparisons or finalized evidence commits. Qualify the actual method
  with passing and deliberately failing controls, retain adversarial review,
  and keep claims/coverage in the owning project's ledger. The historical
  [OS ledger](projects/os-evaluation/quality/evidence.json) retains its original declarations. Run the
  read-only verifier and archival replay checks; preserve frozen inputs and
  original failed attempts. A check pass supports only its declared scope.
- Commit meaningful completed work as it progresses using Conventional Commits
  1.0.0 and the repository choices in
  [docs/commit-standard.md](docs/commit-standard.md).
- Enforce [the privacy gate](docs/privacy.md) before commits and pushes. Inspect
  the exact staged bytes, including binary-marked logs; a clean worktree scan
  does not establish that the index is safe. Keep personal paths, names, emails,
  device identifiers, credentials and ambient radio payloads out of Git.
  Keep originals in ignored private storage and publish explicitly sanitized
  derivatives. Evidence-retention rules never authorize publishing personal data.
  Do not disable hooks, use `--no-verify`, silently add privacy exceptions, or
  commit a local personal-identifier list. Fix a rejection before proceeding.
- Continue authorized research and reversible work autonomously. These documents
  add no approval step for routine implementation choices or verification.
- Write documentation about the work: objectives, behavior, design decisions,
  constraints, procedures, evidence and unresolved gaps. Omit conversational
  history and instruction attribution such as "at the user's request."
  Do not narrate user/agent interactions or add gratuitous statements that a
  project is separate from another. Mention other projects only when a concrete
  dependency, shared resource, path or evidence boundary requires it.
  Retain technical provenance and qualification limits that affect interpretation.
- Keep documentation useful to its reader. Tool READMEs explain that tool;
  include other components only for a real dependency or required procedure.
  Maintain current API/safety decisions with their code and results with their
  project. Fold lasting findings into those records and retire resolved one-off
  review narratives. Routine check results belong in commit/CI records, with
  scratch diagnostics in ignored storage, rather than a new review archive.
- Retain primary observations, complete compared cohorts (including failures),
  necessary configuration/provenance and the evidence behind current claims.
  Generated mirrors, reproducible build databases, duplicate source snapshots
  and resolved setup logs need a specific continuing use to remain in Git.
  A checksum listing or a link alone is not that use. Retire an obsolete study
  as a whole with its conclusions scoped honestly; do not select favorable runs.

## Project organization

This is a multi-project ESP32-C6 workspace. Start at [README.md](README.md)
and follow [projects/README.md](projects/README.md) when adding work.
Keep each project's scope, stack choices and results in its project directory.
Before working in a project, read its root `AGENTS.md` and any local
`AGENTS.override.md`, including when the session starts at the workspace root.
Keep project guidance focused on its own commands, contracts and constraints;
inherit shared rules here. Local overrides stay ignored and must explicitly
read the tracked guide in the same directory before adding local instructions.
Project-only crates, host tools, firmware and research belong there too.
Root `tools/` and `docs/` are for workspace policy or actual shared use; a name
such as "core" or possible future reuse does not establish shared ownership.
The root Cargo workspace builds workspace utilities. Each implementation project
owns its Cargo workspace, dependency lock and compatible toolchain; keep firmware
workspaces independent where their target/SDK configuration requires it.
Follow [the ownership map](docs/organization.md) for existing paths. Results,
method definitions and implementation reviews have project/component owners too.

The board is ESP32-C6-DevKitC-1-N8 with two USB-C ports.
Consult [docs/hardware.md](docs/hardware.md) for the documented port/pin map
and actual test coverage. Preserve original observations and input identities
when moving evidence; record its former location without treating it as a new run.
