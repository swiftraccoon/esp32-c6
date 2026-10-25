# Workspace tools API and safety contract

This contract covers [privacy-guard](../src/bin/privacy-guard.rs),
[style-sources](../src/bin/style-sources.rs) and their host tests. The
[coding standard](../../../docs/coding-standard.md) and its
[pinned sources](../../../docs/standards/sources.json) govern the implementation.
The applicability table below accounts for all 54 active entries in the
[Rust API inventory](../../../docs/rust-api-checklist.md).

The [package](../Cargo.toml) contains private command-line binaries, with no
exported Rust library. Rust 1.96.0, language/style edition 2024 and the root
[dependency lock](../../../Cargo.lock) define the build. Hook installation and
the Git integration tests require Unix; the selected local host target is
`aarch64-apple-darwin`. Other hosts need their own executed checks. Usage belongs
in the [utility README](../README.md), [privacy policy](../../../docs/privacy.md)
and [style reference guide](../../../docs/standards/README.md).

## Privacy boundary

`check` reads every stage-zero index blob through Git, including unchanged
tracked files. It scans the indexed path and indexed bytes rather than the
worktree copy. Unmerged entries, symlinks, submodules, malformed object framing
and unsupported object identities fail. Git reads disable replacement objects.
The current object reader accepts 40-digit hexadecimal Git identities; it does
not support a SHA-256-format Git repository.

Local checks load a nonempty private identifier list from Git metadata. Hooks
cannot select `--public-only`; that explicit command-line option runs generic
rules without private terms or the commit-identity check. The artifact exception
list is read from the index. Exceptions bind an exact SHA-256 and can waive only
the raw/opaque artifact categories; they cannot waive a personal-data rule.

| Entry point | Required checks |
| --- | --- |
| `check` / `pre-commit` | Indexed paths and bytes; local author/committer identities unless explicitly running the standalone public-only command. |
| `prepare-commit-msg` / `commit-msg` | Index, identities and the bounded message file. |
| `history` | Complete local referenced history, object contents, tree names and annotated tag objects. Shallow history fails. |
| `pre-push` | Index, proposed local/destination reference names, complete local referenced history and proposed tips. This scans more than only the objects missing from a remote. |

Every hook also compares the indexed guard source with the source embedded in
its executable. A missing or changed source requires rebuilding and reinstalling
the guard. This binds that source file, not the entire dependency graph or the
trustworthiness of the executable. Installation keeps terms in private Git
metadata, sets restrictive Unix permissions and refuses unrelated existing
hooks or a different configured hooks directory.

Scanning checks ordinary UTF-8, BOM-marked UTF-16/UTF-32, NUL-bearing text and
decoded JSON string tokens. Unsupported encodings and recognized binary/container
formats require an exact artifact exception. Generic rules include personal
paths, email/device identifiers, private-key markers and captured packet data.
This is pattern detection, not a guarantee that arbitrary semantic personal
information or deliberately concealed content can be recognized. Accepted
opaque artifacts still require a sound review of their contents.

Batch-read Git object bodies are limited to 64 MiB, private terms to 64 KiB and
commit messages to 1 MiB. These are per-input bounds: the public exception list,
Git inventories, push-input lines, aggregate history and temporary decoded
copies have no global memory/time budget. Missing policy, exceeded bounds,
parsing and Git/I/O failures reject the operation.

Privacy diagnostics print rule names and abbreviated object identities, never
matched values or paths. The top-level error handler withholds underlying
parser/I/O errors, and Git stderr is not forwarded. Private records' derived
`Debug` output must not become a logging path. An operator who disables hooks,
changes policy or bypasses the tool is outside this enforcement boundary.

## Style source boundary

`check` uses local files only. It validates the lock schema, fixed repository
identities, normalized paths, required licenses, exact selected inventories and
SHA-256 digests. It compares active upstream checklist identifiers with the
authored API inventory, including checked entries and multiple links on a line;
standalone reference definitions do not create requirements. A pass establishes
agreement with the adopted lock, not that the lock has an independently trusted
origin or that application code follows the guides.

`update` resolves the requested upstream revision through Git; default `HEAD`
follows the remote's default branch without assuming a branch name. Arguments
are passed directly to Git, without shell interpolation. The updater removes
caller Git redirection variables, disables replacement objects and filesystem
monitoring, and checks each cache's expected origin. Cache metadata must be a
regular directory tree; gitdir files, symlinks, shared-directory redirects and
alternate object stores reject before fetching. This is not a sandbox for
hostile local Git configuration or concurrent filesystem replacement.

Only selected regular Git blobs are copied, without checkout or execution of
downloaded source. Required license/attribution files remain in the selection.
Limits are 16 MiB per blob, 64 MiB per selected source and 10,000 files per source.
Local snapshot reads are not subject to those download bounds.

The updater validates installed copies before fetching, prepares all requested
replacements, validates the candidate and compares the installed tree again
before replacement. An exclusive update-lock file coordinates cooperating
updaters. A failed download leaves installed copies intact. Installation retains
the previous directory and attempts rollback if the candidate rename fails.
The two renames are not crash-atomic: termination or rollback failure can leave
recovery directories or a stale lock. Follow the reference guide's recovery
procedure instead of deleting those files blindly.

If the newly installed upstream checklist differs from the authored inventory,
the command returns failure **after installing the new sources**. Reconcile the
inventory or restore the retained previous copy; a nonzero exit is not a promise
that installation never occurred. The tool leaves all changes uncommitted.

## Execution and resource ownership

Both binaries forbid authored unsafe code. They use synchronous `std`, heap
allocation, filesystem operations and child processes. No authored FFI, threads,
async tasks, custom synchronization traits or cancellation API are present.
Filesystem and Git/network operations can block without a timeout. Release
panics abort; ordinary invalid input and operational failures use `Result`.
Process termination can bypass cleanup and leave partial installation state.

The privacy object reader owns its Git child and both pipes. Its destructor
attempts to kill and reap that child on success and error paths, discarding
cleanup errors. Reaping can block; the scoped exception below records the lack
of a separate shutdown API. Test-directory cleanup likewise attempts removal
without panicking. It acts only on directories created for the fixture.
Style updates release their file lock explicitly and report lock-removal or
installation errors; they have no authored destructor.

## API guideline applicability

“Not applicable” identifies an absent construct, rather than an exemption for
private code. The linked inventory supplies each pinned rule's explanatory
section. Applicable decisions include the operational boundaries above.

| Entries | Decision and evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies. Rust casing and related names consistently distinguish policies, sources, revisions, hashes and checked paths. |
| C-CONV, C-CONV-TRAITS, C-CONV-SPECIFIC | Not applicable. No authored conversion-method/trait API; byte decoders and digest functions perform validation or computation, and ordinary value conversions use standard facilities. |
| C-GETTER | Not applicable. No authored field-accessor API. |
| C-ITER, C-ITER-TY | Not applicable. No authored iterator type or collection-iteration API. |
| C-FEATURE | Not applicable to authored feature names: the package defines none. Dependency features retain their upstream names; `serde_json` selects `float_roundtrip`. |
| C-COMMON-TRAITS | Applies. Production configuration/resource types derive useful `Debug`, and interchange types derive their required Serde traits. No valid default exists for an unloaded policy or Git child. Live resources are not copied; private workflow records do not promise general cloning, ordering or hashing interfaces. |
| C-COLLECT | Not applicable. Collections are standard maps, sets and vectors; no custom collection is introduced. |
| C-SERDE | Applies to the source lock's `Serialize`/`Deserialize` representation. Exception S1 covers the privacy inputs' intentionally absent serialization path. Strict input structs reject unknown fields. |
| C-SEND-SYNC | Applies. Compiler-derived auto traits follow owned fields; no manual sharing implementation or cross-thread use is introduced. |
| C-GOOD-ERR | Applies at the command boundary. Internal contextual `anyhow::Result` composes errors; there is no custom public error type. Privacy failures deliberately expose safe rule identifiers rather than sensitive error chains. |
| C-NUM-FMT | Not applicable. No authored numeric/bitfield wrapper type; hashes use upstream hexadecimal formatting and validated strings. |
| C-RW-VALUE | Not applicable. No authored generic `Read`/`Write` function; helpers take paths/bytes or own the specific Git pipes. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable. No authored macros. Invoking upstream derives and formatting/error macros does not define a macro API. |
| C-CRATE-DOC, C-EXAMPLE, C-LINK | Applies at application scope. Module descriptions, CLI help, this contract and linked usage guides explain real commands and their effects. No public Rust items require separate call examples. |
| C-QUESTION-MARK | Not applicable to examples: usage examples are shell commands, not fallible Rust examples. Test assertions/unwraps are fixture checks, not recommended error handling. |
| C-FAILURE | Applies. Validation errors, withheld diagnostics, resource bounds, blocking and interrupted-update behavior are specified above. Production errors propagate without an invented success result. |
| C-METADATA | Exception M1. The manifest records actual package purpose, version, edition, Rust requirement and `publish = false`; absent publication metadata is not fabricated. |
| C-RELNOTES | Not applicable while no release/publication process is offered. Significant repository changes follow the commit standard; publishing requires revisiting release documentation. |
| C-HIDDEN, C-STRUCT-PRIVATE | Applies. Implementation types and fields remain private; JSON schemas and command behavior are documented without exporting Rust representations. |
| C-SMART-PTR, C-OVERLOAD, C-DEREF | Not applicable. No authored smart pointer, operator overload or dereference implementation. |
| C-METHOD, C-CTOR | Applies. `Policy::load` and `Objects::open` name fallible construction; policy queries and object reads are methods. Guide operations use explicit path/specification data. Test repositories use `new` for an exclusively owned fixture. |
| C-NO-OUT | Applies. Checks return results, records or tuples. The recursive guide walker modifies one caller-owned accumulator, using the rule's existing-data exception rather than concealing a returned result. |
| C-INTERMEDIATE | Applies. Scanning returns a set of rule identifiers; source checking returns the validated lock; fetching returns provenance and exact bytes for candidate assembly. |
| C-CALLER-CONTROL, C-GENERIC | Applies. Helpers borrow paths, strings and byte slices when ownership is unnecessary. Concrete ordered collections and contiguous byte buffers match these fixed workflows; no reusable generic framework is introduced. |
| C-OBJECT, C-SEALED | Not applicable. No authored traits or downstream implementations. |
| C-NEWTYPE | Applies to the test `Repository(PathBuf)` resource boundary. Production paths use `Path`; source/Git identities remain private, dynamically validated strings, without a claim of static interchange safety. |
| C-CUSTOM-TYPE | Applies to named policy/source records and the style CLI's `Action` enum. Exception T1 covers two private privacy-mode booleans. |
| C-BITFLAG | Not applicable. No combinable authored bit flags. |
| C-BUILDER | Not applicable to authored APIs. Existing `Command` and `OpenOptions` builders configure resources; no custom value requires an additional builder. |
| C-VALIDATE | Applies. Index/object framing, private policy, source identity, paths, revisions, modes, schemas, inventories, hashes and licenses are checked before their corresponding success boundary. |
| C-DTOR-FAIL | Applies. The Git-child and test-fixture destructors ignore cleanup errors and do not panic. Fallible style installation/lock cleanup is explicit. |
| C-DTOR-BLOCK | Exception D1. Git-child reaping and test-directory removal can block without a separate nonblocking shutdown path. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies to production records through field-bearing derives. Privacy-sensitive debug data is not emitted. The private test owner has no public formatting contract. |
| C-NEWTYPE-HIDE, C-STRUCT-BOUNDS | Not applicable. No exported implementation-dependent return type or authored generic data structure. |
| C-STABLE | Not applicable to a stable public Rust library API: none is exported. Private dependencies and command/schema compatibility are controlled by the lockfile and validation tests. |
| C-PERMISSIVE | Exception M1 for the package's unselected license. Dependency declarations and copied-source licenses retain their own terms. |

## Scoped exceptions

**M1 — unpublished metadata and licensing.**
[Metadata](../../../docs/standards/rust-api/src/documentation.md#c-metadata)
and [licensing](../../../docs/standards/rust-api/src/necessities.md#c-permissive)
call for publication metadata and a permissive crate license. This package has
no selected project license, authors, repository URL, keywords or categories.
Inventing these or copying a dependency's license into the package would be
misleading. `publish = false` prevents registry publication; it does not grant
distribution rights or settle licensing for a repository export. Select the
project terms and complete accurate metadata before offering a distributable
release. The locked dependency declarations are permissive, with
`unicode-ident` additionally declaring `Unicode-3.0`; declarations alone do not
replace required notices or a distribution review. Copied guides keep their
upstream attribution/licenses independently of this package.

**S1 — input-only privacy records.**
[Serde](../../../docs/standards/rust-api/src/interoperability.md#c-serde)
normally supplies both interchange directions. `PrivateTerms` and `Exception`
implement `Deserialize` only: checks consume these inputs and provide no record
export API. Adding unused `Serialize` implementations would not serve that
contract; private terms must not acquire a diagnostic serialization path.
The installer writes its narrowly constructed initial JSON separately. Revisit
when a deliberate configuration-edit/export interface is introduced; that work
must define what may leave private storage.

**T1 — private boolean modes.**
[Meaningful argument types](../../../docs/standards/rust-api/src/type-safety.md#c-custom-type)
favor named choices. `Policy::load(public_only)` and
`check_index(require_source)` retain booleans within one binary. Their meanings
are fixed, and hook dispatch forces private terms and source matching where
required. Named enums would make call sites clearer but add no current mode;
the consequence is that callers must inspect argument names. Replace these
flags with named modes if another mode or a reusable API is introduced, and
retain the hook-bypass regression checks.

**D1 — blocking cleanup.**
[Destructor blocking](../../../docs/standards/rust-api/src/dependability.md#c-dtor-block)
normally warrants an alternative shutdown path. These short-lived synchronous
commands use kill/wait cleanup to attempt child reaping on every ordinary exit,
including errors; test owners similarly remove their temporary directories.
An explicit close/timeout API is not implemented. Cleanup errors are discarded
and completion has no time bound. Revisit before embedding the tools in a
long-running service or promising bounded cancellation; do not describe this
destructor as nonblocking or guaranteed to reap a child.

## Regression checks

The source tests exercise behavior through parsers, files and real temporary
Git repositories. Preserve positive controls alongside rejection cases.

| Location / tests | Contract protected |
| --- | --- |
| Privacy unit tests: `detects_personal_data_in_binary_and_utf16_without_printing_it`, `encoded_identifiers_and_renamed_binary_formats_reject` | Encoded/NUL-bearing identifiers, escaped JSON and binary disguises reject; sanitized text remains admissible. |
| Privacy unit test: `identifiers_and_payloads_reject_with_narrow_artifact_exceptions` | Personal-data and capture rules; exact artifact hashes do not permit changed bytes or private-storage paths. |
| [Git integration tests](../tests/privacy_guard.rs): `staged_binary_leak_blocks_even_when_worktree_is_clean` | The index is authoritative, and diagnostics withhold the matching value. |
| `missing_policy_stale_binary_and_private_commit_message_block` | Required local policy, executable/source agreement and commit-message checks. |
| `push_rejects_a_leak_hidden_in_an_earlier_commit`, `derived_radio_records_and_replacement_objects_cannot_hide_a_leak` | Clean tips do not hide ancestors; replacement refs and derived capture records do not evade scanning. |
| `public_tip_cannot_be_pushed_under_a_private_reference_name` | Destination-reference checks plus a successful clean push to a temporary local repository. |
| Style unit tests: `rejects_path_and_revision_injection`, `offline_check_rejects_edits_extra_missing_files_and_bad_provenance` | Normalized selections, revision syntax, inventories and exact locked content. |
| `reads_git_blob_bytes_and_rejects_symlink_entries`, `rejects_local_symlinks_and_linked_ancestors`, `rejects_redirected_git_metadata_before_fetch` | Byte-preserving copies, regular file modes and nonredirected source/cache storage. |
| `active_inventory_ignores_unused_reference_definitions` | Active checklist IDs survive checked/alternate formatting; unused definitions do not create entries. |

Run from the repository root after provisioning the locked dependencies:

```sh
cargo +1.96.0 fmt --all -- --check
cargo +1.96.0 test --offline --locked -p workspace-tools
cargo +1.96.0 clippy --offline --locked -p workspace-tools --all-targets -- -D warnings
cargo +1.96.0 doc --offline --locked -p workspace-tools --no-deps
cargo +1.96.0 run --offline --locked -p workspace-tools --bin style-sources -- check
```

The unit suite does not exercise every update failure or live upstream change.
Changes to fetch/install behavior also need an isolated integration fixture
covering a non-`main` default branch, partial download failure, local edits during
fetch, preserved previous copies and an offline check with Git unavailable.
Never use an ordinary verification run to update the adopted source pins.
