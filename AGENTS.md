# Repository working agreement

Read [docs/coding-standard.md](docs/coding-standard.md) before writing or changing
code. Its pinned source revisions govern this repository; live guide updates do
not silently change the standard.

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
  and keep claims/coverage in [the ledger](quality/evidence.json). Run the
  read-only verifier and archival replay checks; preserve frozen inputs and
  original failed attempts. A check pass supports only its declared scope.
- Commit meaningful completed work as it progresses using Conventional Commits
  1.0.0 and the repository choices in
  [docs/commit-standard.md](docs/commit-standard.md).
- Continue authorized research and reversible work autonomously. These documents
  add no approval step for routine implementation choices or verification.

The initial deliverable is research and reference documentation. No implementation
or benchmark result is established by adopting this agreement.
