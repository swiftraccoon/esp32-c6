# Source snapshot verifier review

Reviewed against the [coding standard](../coding-standard.md), all
[54 API entries](../rust-api-checklist.md), and the complete
[benchctl applicability baseline](benchctl.md). This host verifier checks
retained source, text artifacts and raw measurements without opening a device.

## Artifacts and checks

`verify-public-snapshot` reads `quality/publication.json` with kind
`source-snapshot-v1`. Its four file states are `retained`, `modified`, `omitted`
and `added`. Each entry records original/current SHA-256 identities and an
applicable reason. Retained identities must match; modified identities must
differ; omitted entries have no current identity; added entries have no original
identity. Nullable fields must be explicit.

The verifier rejects malformed hashes, unknown fields, duplicate or non-normal
paths, symlink traversal, incomplete tracked-file inventory and changed current
bytes. The manifest cannot hash itself. Omitted paths must not be tracked and
their artifact contents are not read. Markdown links may name an unavailable
artifact only through its exact declared omission; those targets are counted.
Unrelated missing links still fail.

Original Git IDs and hashes are recorded declarations. They do not establish
unavailable original artifacts or pre-capture commit ancestry. Archive
`SHA256SUMS`, `INPUTS.sha256` and the evidence ledger must remain byte-identical;
the existing claim, failure and coverage rules still apply. Commands
`verify-evidence`, `replay-archives` and `qualify-method` retain their complete
artifact requirements and do not select this limited route automatically.

Protocol `serial.raw`, observation receipts and deterministic replay outputs
must remain retained. Existing validators check protocol definitions, checksums,
completeness, fatal/extra-startup behavior and derived values. Capture-report
fields and saved replay bytes must match actual replay. Updating a manifest
hash cannot admit an inconsistent numerical report. Failed RTIC, accounting
and load attempts cannot gain favorable summaries.

Omitted ELFs are not inspected. Exact-artifact method qualification, comparative
study replay and native console semantics beyond existing adapters remain
explicitly unavailable. A consistent source snapshot is not a new physical
qualification or a complete platform comparison.

## API applicability

All 54 baseline decisions were reconsidered. The affected entries below retain
their application; other package, dependency and API decisions remain in the
baseline. No dependency, unsafe/FFI implementation or firmware execution context
is added.

| Entries | Evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Original/current identities and the four named file states distinguish their roles. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Typed records derive useful Debug/Deserialize; Copy/Eq apply only to the status enum. |
| C-SERDE, C-VALIDATE | Strict records, required nullable fields, checked identities, exact inventory and raw-derived comparisons precede success. |
| C-GOOD-ERR, C-FAILURE | Contextual Results retain parsing, path, Git, identity and unavailable-artifact errors; ordinary failures do not panic. |
| C-SEND-SYNC | One host context owns immutable lineage/state. The local assurance counter is not shared across threads. |
| C-RW-VALUE, C-CALLER-CONTROL, C-GENERIC | Paths/lineage are borrowed and existing validators retain their I/O contracts. |
| C-CUSTOM-TYPE, C-NEWTYPE | Entry, Status and Publication distinguish identities and availability; SHA-256 uses an exact checked representation. |
| C-METHOD, C-CTOR | Fallible loading constructs a complete validated map; methods enforce reference and retention requirements. |
| C-NO-OUT, C-INTERMEDIATE | Structured results keep checked artifacts and unavailable checks visible without writing evidence. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE, C-QUESTION-MARK | Module/CLI documentation and this review describe the host command and its limits; no public Rust library interface is introduced. |
| C-HIDDEN, C-STRUCT-PRIVATE, C-STRUCT-BOUNDS | Representation remains internal; assurance borrows an optional immutable resolver without public trait-bound commitments. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No production destructor is added; test destructors remove only their exclusively created temporary directories. |

Allocation, file reads and fixed Git subprocesses may block on the host, outside
device timing regions. No command from a manifest, ledger or Markdown file is
executed.

## Verification coverage

Fixtures omit original ELFs and commit objects to exercise the limited route
and the separate complete-artifact failures. Contrary controls challenge
identities, missing/duplicate/unlisted mappings, self hashing, path escape,
symlinks, changed historical manifests or ledger, a one-ULP numerical mutation
with an updated current hash, missing links and failed RTIC promotion. Workspace
checks continue to exercise complete-artifact admission and method fixtures.
