# Local style references

Read these checked-in sources without fetching anything:

| Guide | Local entry points | Upstream and license |
| --- | --- | --- |
| Rust Style Guide | [Guide](rust-style/src/doc/style-guide/src/README.md), [contents](rust-style/src/doc/style-guide/src/SUMMARY.md) | [rust-lang/rust](https://github.com/rust-lang/rust/tree/main/src/doc/style-guide); [Apache-2.0](rust-style/LICENSE-APACHE) or [MIT](rust-style/LICENSE-MIT), [copyright](rust-style/COPYRIGHT) |
| Rust API Guidelines | [Contents](rust-api/src/SUMMARY.md), [checklist](rust-api/src/checklist.md), [repository review inventory](../rust-api-checklist.md) | [rust-lang/api-guidelines](https://github.com/rust-lang/api-guidelines); [Apache-2.0](rust-api/LICENSE-APACHE) or [MIT](rust-api/LICENSE-MIT) |
| Google Rust Style Guide | [Guide](google-rust/rustguide.md) | [google/styleguide](https://github.com/google/styleguide); [CC-BY-3.0](google-rust/LICENSE) |

The guide bytes are unmodified upstream sources, attributed to the projects
above. [sources.json](sources.json) records each repository, requested revision,
resolved commit and every file's SHA-256. It is the adopted revision lock for
the [coding standard](../coding-standard.md). The initial local copies retain
the previously adopted revisions; copying them does not adopt a newer baseline.
Upstream code examples and third-party links retain their original licenses
and context. These selections do not mirror every external reference.

The two book sources contain links to generated `.html` pages. Use their local
contents pages and the repository API inventory to navigate Markdown offline;
the inventory links directly to all applicable explanatory chapters. Source
bytes and upstream URLs are preserved rather than rewriting those book links.

## Check offline

From the workspace root:

```sh
cargo +1.96.0 build --offline --locked -p workspace-tools --bin style-sources
target/debug/style-sources check
```

Building needs the Rust toolchain and dependencies provisioned once. Reading
the guides and running the built checker needs no internet, Git fetch or book
renderer. CI runs the same offline check. It verifies inventories, hashes,
licenses, provenance structure and agreement with the active API inventory;
it does not establish semantic compliance or authenticate GitHub's authors.

## Update from upstream

```sh
target/debug/style-sources update
target/debug/style-sources update --source rust-style --rev main
target/debug/style-sources check
git diff -- docs/standards docs/coding-standard.md docs/rust-api-checklist.md
```

`update` fetches each repository's `HEAD`, which follows its default branch
regardless of whether it is named `main`, `master` or `gh-pages`. Use `--source`
with `rust-style`, `rust-api` or `google-rust` to update one guide. `--rev` accepts
a branch, tag or full commit hash, so a particular revision can be reproduced.
Only this explicit update operation uses the network. It records the resolved
commit and all file hashes; it never commits or pushes.

Review the upstream diff, reconcile changed requirements, API inventory entries
and applicability decisions, then commit the source lock, copies and review
together. A changed API checklist causes `check` to fail until the inventory is
reconciled. Existing frozen experiment reviews retain their original baseline.

Updates refuse modified, missing, extra or symlinked files in a source copy.
Keep annotations in authored documentation outside the three source directories.
All selected downloads and validation finish before replacement. The previous
directory remains under ignored `.cache/style-sources/` for recovery. An
interrupted install may leave `next` and `previous` directories there; inspect
them before restoring a copy or removing a stale `update.lock`. An update is
single-writer; do not edit the guide directory concurrently with installation.

The [tool contract](../../tools/workspace-tools/docs/api.md) records validation,
update-recovery limits and API decisions.
