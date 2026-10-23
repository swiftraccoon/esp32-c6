# Independent review of assurance checks

Review date: **2026-10-23**. This review covers executable consistency and
comparison-admission checks, with the final source delta checked below. It does
not establish a physical qualification pass or completeness of the OS study.

The reviewer owns only this document. Implementation, data, process and CI are
owned by other agents. Fault injection used a separate copy of the tracked
working files and Git history plus temporary method/study fixtures. Implementation, retained
results and hardware were not changed by the reviewer. Commands ran on
**macOS arm64 with Rust 1.96.0**; no serial device was opened.

## Findings that shaped the controls

The checks now distinguish artifact consistency from scientific meaning. Early
review required actual validator execution for negative controls, an exact
metric-to-contract set, parsed executable ELF validation, distinct normalized
review attribution and distinct semantic/adversarial review identities.
Historical archive-relative checksums required an explicit path-base field;
the verifier does not rewrite or guess the original format. Operator inventory
can support pins/fixtures but cannot alone establish driver/API availability.
Qualified physical coverage additionally requires raw-capture support.

A Git freeze binds the **manifest bytes**, then their listed current input
bytes. It does not require every input's own blob to exist in that old tree:
the historical SDK configuration was pinned by its committed digest before
its previously ignored bytes were separately added to Git. The independent
stale-freeze control changed the input and INPUTS manifest and resealed the
final archive before checking rejection against the old Git blob.

Two deliberate counterexamples remain accepted structurally: unsupported
claim prose with unchanged valid evidence metadata, and an unsupported but
nonempty endpoint description with valid method identities. These demonstrate
the intended limit. A hash, role or reviewer label cannot establish that its
content is true, supports a statement or reflects independent thought.

## Executed baseline and archive faults

The tested CLI snapshot was copied to
`.cache/assurance-independent/benchctl-reviewed`; its SHA-256 was
`53dc32a90ed75f5ec61d7664a346cf98f0c5304884c558e6de57e3453abd067c`.
Subsequent availability/path guard changes receive separate final checks below.
The copied repository retains freeze history at current HEAD
`46e64c2c4e10b5216259d68621e66ab90db34204` plus staged/working additions.

The real repository-data positive control passed: **nine finalized archives,
671 manifest entries, 30 claims, 26 routes × 35 domains = 910 pairs**. Fourteen
pairs are scoped partial observations; 896 remain unknown/not-run and none is
fully qualified. A restored copy passed again after mutations.

Each following mutation was applied separately to that admitted copy. The CLI
returned failure for the indicated mechanism; a nonzero exit alone was not the
acceptance criterion.

| Mutation | Observed rejection |
| --- | --- |
| Change sealed Ariel RESULTS bytes | File checksum mismatch. |
| Remove Zephyr native SHA256SUMS | Missing referenced manifest. |
| Add a hidden unlisted archive file | Archive inventory differs. |
| Change an input, its INPUTS digest and final archive digests together | INPUTS differs from its recorded Git freeze. |
| Delete a review supporting a claim | Missing evidence path. |
| Change that review's bytes | Evidence hash mismatch. |
| Add a broken checked local Markdown link | Missing local target, identifying the source document. |
| Relabel the first Ariel operator guard independent | Missing reproduction attribution/support. |
| Relabel source-only yield evidence observed | Missing required raw-capture role. |
| Remove an evidence SHA | Required support hash absent. |
| Reuse the producer as reproducer with different case/spacing | Normalized actors are equal. |
| Qualify hardware coverage using offline reproduction without raw capture | Qualifying physical claim lacks raw-capture support. |
| Qualify unknown coverage defaults | Defaults must remain unknown/not-run. |
| Supply an unknown evidence role | Typed ledger parsing rejects the enum. |
| Declare the wrong historical checksum path base | Resolved file leaves its archive. |
| Duplicate a checksum row | Duplicate manifest path. |
| Substitute a tracked proof with a symlink to ignored internal bytes | Symlink evidence rejected. |
| Use operator-only evidence to declare driver/Rust API availability | Required source/build/observed/reproduced support absent. |
| Replace a tracked proof's parent directory with an ignored internal symlink | Symlink path component rejected despite cached child tracking. |

One test-harness assertion initially expected the older generic coverage error;
the implementation correctly returned the more specific raw-capture rejection.
The initial diagnostic was retained and the corrected expectation rerun. This
was not a changed acceptance rule or an implementation failure.

## Method and capture admission faults

The retained IDF profile passed **two real positive controls and all ten named
negative controls**, executed through the actual protocol validators. Each
variant rejects panic, reboot, additional ROM startup, missing completion and
corrupt nonzero sample checksum for the named category. These are retrospective
offline controls over the old pilots, not controls that ran before their capture.

Eleven additional independent CLI faults were rejected:

| Fault group | Cases | Rejection |
| --- | ---: | --- |
| Missing required ELF/output token | 2 | Required compiled or observed bytes absent. |
| Missing/extra timing-contract metric | 2 | Exact positive-control metric set differs. |
| Same normalized reviewer or same review bytes | 2 | Attribution/artifact identities are not distinct. |
| Changed configuration bytes | 1 | Pinned configuration hash is stale. |
| Truncated/wrong-architecture ELF, with self-consistently updated hashes and report | 2 | ELF parsing or loadable RISC-V32 executable requirement fails. |
| Rehashed positive raw containing a panic | 1 | Actual positive-control replay rejects it. |
| Rehashed positive report with changed benchmark data | 1 | Positive report disagrees with replay. |

Eight fake-port CLI cases confirmed rejection before output creation or a
serial-open error: missing purpose on both collectors, comparison without a
method on both, qualification with a comparative method on both, wrong wire
variant and a mismatched requested ELF. Source review confirms both real CLI
branches call admission before capture and both captures reread ELF/provenance
before serial access. This is more than a validate-only repair.

A complete derived **schema-2 study/schema-4 report** fixture exercised the new
analyzer path. It used copied historical raw observations, pilot provenance and
explicitly synthesized comparative receipts; it is not a new capture or a
claim that the historical collector produced those receipts. Matching records
admitted twelve pairs and the conditional interval. Changing one receipt's
purpose to qualification, deleting its admission, downgrading it to schema 3
with the new collector identity, or changing its method variant each left
**eleven complete pairs and no interval**. The actual unchanged legacy study
remains accepted only with its historical collector identity.

## Reproduction and source review

The independently executed command families were:

```sh
cargo +1.96.0 test --workspace --locked
cargo +1.96.0 build --locked -p benchctl
target/debug/benchctl verify-evidence --root . --ledger quality/evidence.json
target/debug/benchctl qualify-method --method quality/methods/idf-queue-v1.json
target/debug/benchctl replay-archives --root .
```

For mutation runs the copied CLI and temporary repository/method paths replaced
the corresponding arguments. Local case records are retained under
`.cache/assurance-independent/`; the durable descriptions above identify the
mutations, expected categories and derived-fixture boundary. Production unit
fixtures retain executable regression mechanisms in
[assurance.rs](../../tools/benchctl/src/assurance.rs),
[qualification.rs](../../tools/benchctl/src/qualification.rs) and
[queue_study.rs](../../tools/benchctl/src/queue_study.rs).

`replay-archives` independently reproduced **all 61 retained protocol outputs
and the original study summary byte-for-byte**. This covers protocol 1/2/3
recordings, not the native Ariel/Zephyr console suites. No failed physical
qualification is converted into a passing dataset by this result.

The existing [all-54 API inventory](../rust-api-checklist.md) and the affected
[verifier](assurance.md) / [method integration](method-admission.md) deltas were
reviewed. These private host modules add no dependency, unsafe operation, FFI,
ISR context or firmware. Hashing, blocking filesystem/Git operations and host
allocation are outside measured device paths. No public-library compatibility
or embedded-memory claim follows from the host implementation.

The final availability/path changes passed both additional independent CLI
mutations above and a restored real-data positive control, using a separate
copied binary with SHA-256
`ff46a58e81bc300fc5d4458484346986cb08f81b954d1f474c7e0b570cae4e8a`.
The independently rerun final workspace suite passed **68 unit tests**
(66 benchctl plus two bench-core) and **two doctests**, including the new
prospective-study and archive-report/ELF-identity regressions.

No blocking inconsistency remained in these exercised consistency/admission
gates. Final workspace lint/documentation/release and current-tree packaging
checks are separately run by the implementation owner; this review does not
substitute for their recorded outcomes.

## Limits preserved by this review

Method ELF checks establish a parsed, loadable RISC-V32 executable and required
bytes in the whole file. They do not show those bytes are in an executed branch.
Raw controls establish admitted recorded output; supplied ELF/provenance does
not authenticate what ran on a board. Synthetic comparison receipts above
reinforce that metadata consistency is not a physical execution receipt.

Role-tagged, hashed support and separate normalized actor labels prevent simple
label-only promotion. Deliberately fabricated roles, identities or misleading
proof text remain a human-review problem. The checker does not execute arbitrary
commands in proof prose. It describes files at check time, not an immutable lock
against later edits.

Local Markdown checks cover the documented inline/reference-definition subset,
tracked file/directory targets and containment. Heading fragments/queries are
stripped, external URIs are not fetched and a valid link does not prove support.

The [Linux CI workflow](../../.github/workflows/assurance.yml) configures Rust
1.96.0, full Git history and the host checks. No hosted Linux execution was
observed by this review, and branch protection is not established by that file.
Tool/dependency provisioning may use the network; artifact validators need no
board or live source lookup. No radio, sleep, energy, utilization, absolute
frequency, reviewer-authenticity or overall OS-selection result is implied.
