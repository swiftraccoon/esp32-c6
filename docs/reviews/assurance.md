# Evidence-ledger and archive verifier review

Reviewed **2026-10-23** against the [coding standard](../coding-standard.md),
[54-entry inventory](../rust-api-checklist.md), existing
[collector applicability review](benchctl-v2.md) and
[boot-text repair review](collector-boot-guard.md). Scope here is the new private
`tools/benchctl/src/assurance.rs` module and its typed
[ledger](../../quality/evidence.json). Root owns CLI/CI and qualification wiring;
this record does not certify those separate changes by implication.

The verifier uses existing dependencies on Rust **1.96.0**, edition/style edition
2024. It introduces no dependency, unsafe operation, firmware, device access,
interrupt or asynchronous context. It never repairs/reseals data automatically
and never executes a command supplied by the ledger or Markdown.

## Enforced boundaries

The full typed JSON model rejects unknown fields and unsupported enum values.
All non-unmeasured claims need scoped, hashed, role-tagged regular tracked files
and explicit limitations. Source/build/operator evidence cannot alone qualify
physical behavior. Independent reproduction requires distinct hashed result
and review files plus different recorded producer/reproducer labels after
case/whitespace normalization. Those are
structural checks, not proof that a file supports a statement or that actors are
authentically independent.

Coverage defaults are fixed to unknown/not-run. Named routes/domains and
supporting claim IDs must exist, and pairs cannot be duplicated. A qualified
override needs a qualifying observed/reproduced claim and the declared driver,
pin and fixture gates, plus raw-capture support on that qualifying claim.
Offline reproduction alone cannot fully qualify a physical domain. The current data marks only narrow partial records;
all other Cartesian-product pairs remain explicit unknown/not-run coverage.
Operator inventory may support pin/fixture declarations, but not driver/API
availability without nonfailed source/build/observed/reproduced support.

All actual immediate `results/` directories must be registered. A finalized
archive requires an exact SHA256SUMS inventory: changed, missing, duplicated or
unlisted artifacts fail. The explicit manifest base preserves historical
archive-relative and newer repository-relative records without fallback.
Input manifests must match their actual freeze Git
blob and current input hashes. This preserves the historical case where an
input digest was frozen before its bytes were separately added to Git; it does
not invent an earlier file blob. A legacy-unsealed entry is explicit and warned
about rather than described as finalized. All current collections are sealed.

A collecting entry is explicitly unfinished: it claims no finalized manifest,
checks current input digests and may temporarily lack a freeze commit. That
state produces warnings and cannot support qualified claims. Finalized input
manifests require their real freeze commit. Current records remain finalized;
this does not weaken their existing checks.

Registry/evidence paths must be normalized repository-relative references;
checksum rows use the registered repository/archive base and must remain inside
their collection. INPUTS rows use repository-relative paths.
Canonical paths remain inside the root. Evidence must be regular tracked or
staged files, and symlinks are rejected even when their target is an ignored
file inside the repository. Archive inventory rejects symlinks and non-regular
entries. Git is invoked directly with fixed commands and arguments; freeze
object IDs are full hexadecimal values rather than arbitrary revision syntax.

Hashing streams each file through a bounded 64 KiB buffer. Manifest/ledger/
Markdown metadata is read into normal host allocations. Git-tracked filenames
and metadata must be UTF-8; invalid values return errors. Checks describe the
files at the verification moment, not a filesystem lock against later edits.

Local Markdown checking covers inline links and reference definitions in
tracked Markdown, including archive reports. It ignores inline/fenced code and
external URI targets, and checks file/directory existence and tracked membership.
It strips fragments and queries; dead heading anchors are not validated. This
is a documented subset rather than a complete CommonMark renderer. The code
does not fetch external pages or claim that a valid link supports a statement.

## API applicability delta

The existing all-54 dispositions and metadata/license exceptions are reused.
The affected decisions below were reconsidered; no new exception or pending
implementation decision is introduced. No public Rust library, exported trait,
macro, iterator, custom collection/operator, smart pointer or crate feature is
added. Data remains internal to the host CLI.

| Entries | Decision / evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: named ledger/evidence/coverage types and filesystem/hash operations retain Rust naming; SHA-256 and count fields have explicit meanings. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: metadata types derive field-bearing Debug/Deserialize; small state enums derive Copy/Eq where meaningful. No live hardware handle is copied. |
| C-SERDE | Applies: typed records deny unknown fields and use explicit enum spellings. Serde data is validated before a consistent report is returned. |
| C-CTOR, C-METHOD | Applies: `Repository::open` is a fallible filesystem-opening operation; checked path/file access stays on the repository context. Test fixtures use `new` for valid setup. |
| C-GOOD-ERR, C-FAILURE, C-VALIDATE | Applies: contextual Result for malformed/stale/missing/escaping/inconsistent data, failed Git and I/O. Ordinary invalid input does not panic. A UTF-8 code-span expect is justified by replacing only complete ASCII-delimited spans of an existing valid string. |
| C-SEND-SYNC | Applies: single host context with owned metadata and borrowed filesystem references; no unsafe sharing or synchronization implementation. |
| C-CRATE-DOC, C-LINK | Applies at internal binary scope: module/helper documentation, ledger legend and this record explain commands, failures and epistemic limits. |
| C-RW-VALUE, C-CALLER-CONTROL, C-GENERIC | Existing decisions apply: filesystem inputs are borrowed paths; ordinary File/Read hashing streams content. No transport abstraction or caller-visible custom buffer ownership is introduced. |
| C-NO-OUT, C-INTERMEDIATE | Applies: verification returns a report through Result; errors do not mutate/reseal files or emit favorable substitute outcomes. |
| C-NEWTYPE, C-CUSTOM-TYPE, C-STRUCT-PRIVATE, C-HIDDEN | Applies: enums distinguish evidence class/role/outcome and availability states. Private structs keep interchange data separate from the checked repository context; no representation commitment is exported. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No production destructor added. The test-only fixture owns an exclusively created temporary Git directory and attempts cleanup on drop; it never targets the working repository. |

There is no authored conversion API requiring a new conversion-trait contract,
no numeric formatting wrapper, generic data structure, exposed raw pointer or
change to firmware panic/cancellation behavior. Unit-test assertions are confined
to fixtures and expected verification results.

## Verification and counterexamples

The fifteen targeted assurance tests passed using:

```sh
cargo +1.96.0 test --locked -p benchctl assurance::tests
```

They exercise actual temporary Git repositories and the production verifier:

- A valid staged ledger and exact archive inventory.
- Historical archive-relative paths with an explicit base; a wrong registered
  base fails rather than being guessed from the filesystem.
- Edited/deleted/unlisted artifacts, missing checksum manifests, duplicate rows,
  escaping paths and malformed Unicode hash rows.
- An omitted result collection, missing evidence SHA and changed support bytes.
- Operator-label-only reproduction promotion and source-only physical
  qualification, including defaults that try to qualify all unknown hardware.
- Offline reproduction labels without raw-capture support cannot fully qualify
  physical coverage; adding role support still does not prove its content.
- Operator pin/fixture inventory is not enough to declare a driver or Rust API
  available without the corresponding nonfailed evidence class.
- Reproduction with the same normalized actor or duplicate support artifact.
- Changed INPUTS and resealed final checksums after a genuine fixture Git freeze.
- Collecting inputs without a freeze commit stay explicitly unfinished, and
  cannot support qualified claims or silently become finalized packaging.
- Missing/local-escape Markdown targets with code/external-link controls.
- Tracked symlinks to both outside and ignored inside support files, including
  the local Markdown target path and a replaced parent directory whose child
  path remains in the Git index.

`cargo +1.96.0 check --locked -p benchctl` passed when the module was integrated.
The targeted tests and warning-denied all-target Clippy run passed after the
manifest-base, collecting-state and raw-capture coverage refinements. An actual
repository `verify-evidence` run passed all nine finalized collections, 671
manifest entries and the ledger's 910 pairs, with no warnings. Link counts vary
as maintained documentation is added; they are not a fixed acceptance count.
Only this module was rustfmt-formatted during concurrent implementation. The
completed [integration record](method-admission.md) separately records the final
workspace formatting, 68 unit tests/two doctests, Clippy, documentation, release
build and actual repository checks on macOS. Hosted Linux execution remains
unobserved.

No new physical observations, qualification pass, CPU/memory utilization result
or platform ranking follows from this verifier. Independent mutation review and
source/measurement controls remain separate evidence.
