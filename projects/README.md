# Project organization

[Workspace](../README.md)

Each project has a directory under `projects/<name>/` and an entry in the
[workspace project table](../README.md#projects). Project choices are local:
a stack, transport or power policy selected for one project does not change
the objectives of another.

## Inside a project

Create directories as they become useful:

| Path | Contents |
| --- | --- |
| `README.md` | Objective, current status, decisions, next steps and links |
| `AGENTS.md` | Project-specific working instructions, commands and contract references; shared rules remain in the root guide |
| `docs/` | Architecture, hardware/transport notes and project decisions |
| `firmware/` | Device applications and their pinned build inputs |
| `crates/` | Libraries and protocols used within this project |
| `clients/` | Phone or desktop companions |
| `tools/` | Project-specific capture, analysis and development tools |
| `experiments/` | Bring-up procedures and prospective experiment contracts |
| `results/` | Project-owned runs, raw records, provenance and interpretation |
| `quality/` | Project-owned ledgers and method definitions |

The wireless explorer now has its own Rust protocol/client workspace and a
separate ESP-IDF firmware workspace. Consult its result records for actual
build and hardware coverage; directory presence alone establishes neither.

[Device identity](device-identity/README.md) inventories per-board identifiers
and documents their supported modification paths and verification limits.

New Rust applications should choose and pin their own compatible toolchains
and dependency sets. Establish a project-local Cargo workspace or an explicit
root membership/exclusion when adding its manifest. The root workspace serves
only workspace utilities; OS evaluation and wireless explorer have independent
host workspaces. Firmware keeps its target-specific workspace/configuration.
Do not silently upgrade one project's pins to accommodate another.

Keep project-only code local until another project actually needs it.
Use the root hardware and standards documents as shared references instead of
copying them. Put local credentials and backups under ignored
`private/<project>/` and `backups/<project>/`; use
`.cache/<project>/` for disposable builds and captures awaiting review.

## Evidence ownership

All results, ledgers and method definitions belong to their project. The
[OS collections](os-evaluation/results/README.md) preserve their original
observations; the [ownership map](../docs/organization.md) explains historical
paths and retention. New experiments apply the
[assurance workflow](../docs/assurance-workflow.md) and choose a project-owned
method/ledger format supported by their verifier.

When adding or editing tracked files, update their affected entries in
`quality/publication.json`, preserving original identities and omitted-file
declarations. For relocated files, update the entry under its recorded name and
resolve its current location through the explicit schema-2 relocation table;
do not add a second entry or manufacture a new original hash. Stage new files
before running the read-only snapshot verifier. Its inventory covers every
tracked/staged file; it does not establish hardware qualification.
