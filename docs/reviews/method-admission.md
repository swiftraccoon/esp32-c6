# Method admission and integration review

Reviewed **2026-10-23** against the [coding standard](../coding-standard.md),
[54-entry API inventory](../rust-api-checklist.md), existing
[collector reviews](collector-boot-guard.md) and [study review](queue-study.md).
Scope: `qualification.rs`, CLI/capture admission metadata, prospective study
checks, archival replay and the configured CI workflow. The independent
[assurance review](assurance-independent.md) separately records executed
adversarial checks and their limits.

## Behavior and evidence boundary

New captures require an explicit purpose. Qualification pilots can be collected
without a comparative method; comparison requires one before opening serial.
Method definitions pin exact ELF/provenance/configuration, source, positive
raw/report controls and semantic/adversarial review files. Re-reading capture
inputs before output creation detects drift after admission. Capture still
does not flash, authenticate hardware or independently identify a board.

The actual protocol validators must accept each positive. The report must
agree with its raw replay, supplied ELF hash and provenance. Every observed
benchmark ID must have exactly one timing contract; missing or extra contracts
fail. Contract text records endpoints, units, aggregation, observer policy and
correctness but cannot be machine-judged as scientifically correct.

ELFs must parse as loadable RISC-V32 executables. Required literals must occur
in the ELF and required output tokens in the positive transcript. Inclusion
alone is not proof a print executes. The passing control and human source/config
trace remain essential. The selected native Zephyr console experiment has no
approved method here and remains failed under its original contract.

Five mandatory counterexamples are applied to each admitted positive and run
through the actual validators: panic, reboot and extra ROM before JSON;
completion removal; and a changed nonzero sample checksum. The rejection must
be for the named category, not an unrelated missing-file/hash error. A definition
cannot simply omit these tests. Other timing/configuration failures remain
covered by the existing protocol tests and need extension when a method changes.

Review attribution is normalized for case/whitespace, and semantic/adversarial
records must have different byte identities. These constraints prevent a simple
label-only substitution. They do not prove reviewer identity, genuine thought
independence or that the documents support the claims. Deliberate fabrication
of an entire record remains outside authenticity guarantees.

The included IDF definition is **retrospective** over the two existing pilots.
It runs new offline controls for future captures of those exact images; it is
not a new hardware run or a claim that this gate preceded the historical study.
All existing report/capture bytes remain preserved.

## Comparative report compatibility

New capture reports use schema **4**, purpose and artifact-bound method proof.
New comparative queue manifests use schema **2** and pin the method definition.
The analyzer reruns the controls, binds both frozen backends and requires each
capture to match its exact method ID/variant/definition, ELF and provenance.
Qualification-purpose records do not enter the comparative estimator. Removing
admission from schema 4 fails; downgrading the schema alone does not make a new
collector historical.

Legacy schema-1 manifests/schema-3 reports require the exact known historical
collector hash `1b5e71a6…fd4277a`; they cannot claim prospective admission.
That preserves original output compatibility for the retained study, not an
arbitrary missing-purpose fallback. An operator rewriting all historical
identities is outside the claimed consistency boundary. Standalone raw protocol
replay remains unchanged.

`replay-archives` traverses regular result files without following symlinks,
reproduces passing protocol outputs and supported study summaries, and compares
complete output bytes. Failed reports must keep a failure and no favorable
summary; they are retained without promotion. This command does not parse
unrelated native-console tests as JSON protocol evidence.

## API applicability delta

The existing all-54 dispositions and metadata/license exceptions were reused;
affected decisions below were reconsidered. There is no new dependency, unsafe
operation, FFI, firmware, async/ISR context or public Rust library. Rust 1.96.0,
edition/style edition 2024 and the Cargo lockfile remain unchanged.

| Entries | Evidence / decision |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: named capture-purpose/wire/method types and explicit SHA/path/timing fields retain Rust naming and meanings. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: metadata has field-bearing Debug and useful equality/serde traits. Small purpose/wire enums are Copy; no live device resource is copied. |
| C-SERDE, C-VALIDATE | Applies: method/proof/admission records deny unknown fields; validated file identities, variants, metric coverage and purpose precede success. |
| C-CUSTOM-TYPE, C-NEWTYPE | Applies: enums distinguish purpose and wire contracts; named identity/proof records separate evidence from a capture verdict. No exported representation wrapper is added. |
| C-CTOR, C-METHOD | Applies at internal scope: test-owned fixtures use `new`; `admit`/`qualify` are named fallible domain operations, not a general public constructor API. Input rechecking stays on its admission object. |
| C-GOOD-ERR, C-FAILURE | Applies: contextual Result for stale/missing/escaping artifacts, wrong ELF, rejected controls and invalid promotion. Ordinary failure does not panic. |
| C-SEND-SYNC | Applies: one host context borrows immutable definitions and input bytes; no unsafe sharing or synchronization is authored. Test fixtures use generated exclusive directories. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE, C-QUESTION-MARK | Applies at internal application scope: module/error documentation and CLI/workflow examples state hardware effects and finite proof limits. Examples are operator commands, not claimed physical test executions. |
| C-RW-VALUE, C-CALLER-CONTROL, C-GENERIC | Existing borrowed I/O/path behavior is retained; actual protocol replay is reused instead of a parallel validation framework. |
| C-NO-OUT, C-INTERMEDIATE | Applies: checks return structured reports through Result and never repair evidence. Temporary byte mutations are isolated negative controls. |
| C-STRUCT-PRIVATE, C-HIDDEN | Applies: definition/variant internals remain private; crate-only identities support CLI/adapters without exported library commitments. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No production destructor is added. Test Drop cleanup targets only an exclusively created temporary fixture and ignores cleanup failure. |

Other prior dispositions are unchanged: no authored macros, iterators, collection
traits, conversions/operators, smart pointers, features or generic data-structure
interfaces. Qualification metadata and filenames are operator-controlled host
inputs, not bounded firmware RAM. No source or report schema is silently revised.

## Verification record

Targeted method tests exercise a passing control and all five rejection paths,
a self-consistent ELF/report change with a missing required literal, stale
configuration, post-admission input drift, missing method, forbidden pilot
promotion, absent/downgraded metadata, incomplete metric coverage and relabeled
actor/review identity. Synthetic ELF fixtures are structural host-test files,
never represented as physical firmware evidence.

Final integration checks on the macOS host passed:

| Command | Observed result |
| --- | --- |
| `cargo +1.96.0 fmt --all -- --check` | Clean. |
| `cargo +1.96.0 test --workspace --locked --quiet` | 68 unit tests (66 collector, two shared-core) and two doctests passed. |
| `cargo +1.96.0 clippy --workspace --locked --all-targets -- -D warnings` | Clean after replacing a three-value integer pattern with its equivalent inclusive range. |
| `cargo +1.96.0 doc --workspace --locked --no-deps` | Documentation built. |
| `cargo +1.96.0 build --release --locked -p benchctl` | Release collector built. |
| `target/release/benchctl verify-evidence --root . --ledger quality/evidence.json` | Nine finalized collections, 671 manifest entries, 30 claims and 910 coverage pairs consistent; no warnings. |
| `target/release/benchctl replay-archives --root .` | All 61 protocol transcripts and one study summary reproduced byte-for-byte. |
| `target/release/benchctl qualify-method --root . --method quality/methods/idf-queue-v1.json` | Two archived positives passed; all ten negative controls rejected. |
| `actionlint .github/workflows/assurance.yml` | Clean. |
| `git diff --cached --check` | Clean. |

The independent reviewer additionally challenged temporary copies through CLI,
including comparative study admission. No hardware operations were performed
for this change. These local results do not establish hosted Linux execution.

CI uses official checkout v7.0.0 pinned to verified upstream commit
`9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0`, full history and read-only token
permissions. Tool/dependency provisioning uses network access; the actual
artifact/control checks do not require a board or live-source lookup. No remote
workflow execution or branch-protection update is claimed.
