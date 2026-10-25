# Workspace ownership

Keep code, research, methods and results with the project that uses them.
A crate shared among OS adapters belongs to OS evaluation. Promote it to a
workspace package when another project actually depends on it.

| Owner | Locations |
| --- | --- |
| [OS evaluation](../projects/os-evaluation/README.md) | Its crates, collector, firmware, docs, results and quality ledger under `projects/os-evaluation/` |
| [Wireless explorer](../projects/wireless-explorer/README.md) | Its firmware, host protocol/client, docs and functional captures |
| [Device identity](../projects/device-identity/README.md) | Its identifier inventory and investigation plan |
| [Workspace utilities](../tools/workspace-tools/README.md) | Privacy guard and offline style-source updater |
| Shared references | Root `docs/`: hardware, standards, privacy/assurance policy and cross-project source catalogs |
| [Publication metadata](../quality/README.md) | Root `quality/`: privacy policy and tracked-file availability/identity inventory |

Maintain API and safety contracts with their components. Keep experiment
interpretation with its result collection and link to it instead of repeating
its tables in a project README or review page. Fold resolved review findings
into the current contract, test or method. Routine verification belongs in
commit/CI records; temporary diagnostics belong in ignored storage.

## Working instructions

Track `AGENTS.md` at the workspace root and each project root. Project guides
add commands, contract references and constraints specific to that project;
shared standards remain in the workspace guide. Read the relevant project
guide before editing its files, even from a workspace-root session.

Use ignored `AGENTS.override.md` for local instructions. Codex selects it before
`AGENTS.md` in the same directory and loads at most one file per directory.
Each local override must therefore begin by reading and following the tracked
`AGENTS.md` alongside it, then apply its local additions. This also keeps root
privacy and evidence rules in force. `AGENTS.local.md` is not a default discovery
name. See the [official discovery rules](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

Local override files are workstation state: do not stage them or force-add them.
New clones can create one at any applicable directory using this minimal body:

```markdown
Read and follow [AGENTS.md](AGENTS.md) in this directory before applying
the local additions below.

## Local additions
```

## Cargo and local resources

The root Cargo workspace contains `workspace-tools`. OS evaluation and wireless
explorer each own a host workspace, lockfile and compatible toolchain. Firmware
has independent manifests and target configuration. Enter a firmware directory
before invoking Cargo so its `.cargo/config.toml` is loaded.

Installed `ldproxy` lives in ignored `.tools/bin/`. The shared, unmodified
IDF 5.5.6 SDK/tools live in ignored `.cache/espressif/idf-5.5.6/`; each consuming
firmware directory reaches it through an ignored `.embuild` link. Application
configurations, Rust wrapper versions and build outputs remain project-owned.
With an external `CARGO_TARGET_DIR`, set `CARGO_WORKSPACE_DIR="$PWD"` from the
firmware directory; the pinned helper otherwise infers the SDK workspace from
the output path.

Namespace private captures, backups and scratch data by project under ignored
`private/`, `backups/` or `.cache/`.

## Evidence retention and historical paths

Keep primary observations, complete compared cohorts including failures,
necessary configuration/provenance and the evidence behind current claims.
A failed run can establish an unresolved blocker or guard against a false pass.
A resolved compiler iteration, generated mirror, duplicate snapshot or copied
review walkthrough needs a continuing use to remain in Git. Retire an obsolete
study as a whole; do not remove unfavorable observations from a retained study.

The [OS results index](../projects/os-evaluation/results/README.md) states each
collection's purpose. Removed generated/review files remain recoverable from Git
history, and the [publication inventory](../quality/publication.json) records
unavailable historical inputs. Current documentation is maintained text, not an
immutable copy of its former review page.

The inventory distinguishes omitted original inputs from retired publications
added later. Both remain unavailable; a retired addition gains no invented
original hash. Sealed result text can retain its historical links without
requiring obsolete review pages to remain active documentation.

The inventory retains original recorded names/hashes and exact current
`relocations`. Historical `results/`, `quality/evidence.json` and
`quality/methods/` now live under `projects/os-evaluation/`; original ledger,
raw capture and input-manifest declarations retain their identities. Rebased
Markdown and extracted ABI notes are current derivatives.
Full-artifact admission still requires original inputs at their recorded paths;
publication aliases do not reconstruct omitted files or freeze ancestry.
