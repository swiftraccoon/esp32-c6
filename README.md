# ESP32-C6 workspace

A workspace for independent projects using the ESP32-C6, with shared hardware
references, Rust conventions and experiment tooling.

## Projects

| Project | Scope | Status |
| --- | --- | --- |
| [OS evaluation](projects/os-evaluation/README.md) | Compare runtimes, Rust integration and hardware support; preserve the existing research and benchmarks | Ongoing; unfinished |
| [Wireless explorer](projects/wireless-explorer/README.md) | Record Wi-Fi, BLE and IEEE 802.15.4 traffic and summarize activity; target iPhone USB-C power/data | UART/native USB receive windows observed; phone and controlled-peer checks open |
| [Device identity](projects/device-identity/README.md) | Inventory per-board identifiers and their supported modification paths | Factory inventory and selected radio changes verified; bridge serial activation remains unverified |
| [Anonymization](projects/anonymization/README.md) | Make ESP32-C6 anonymization as easy and complete as possible through Rust integration and deployment guidance | Initial Rust SoftAP integration builds; external on-board verification pending |

The board is an **ESP32-C6-DevKitC-1-N8** with two USB-C ports.
The hardware probe recorded C6
revision 0.2, 8 MB flash, a 40 MHz crystal and a CP2102N UART bridge.
See the [hardware inventory](docs/hardware.md) for port roles, onboard features
and the distinction between documented capability and tested behavior.

## Repository layout

| Location | Purpose |
| --- | --- |
| [projects/](projects/README.md) | Each project's firmware, libraries, clients/tools, documentation, methods and results |
| [docs/](docs) | Shared hardware, standards, privacy/assurance policy and reusable research references |
| [tools/workspace-tools/](tools/workspace-tools/README.md) | Privacy hooks and offline style-source maintenance; the root Cargo workspace builds these |
| [quality/](quality/README.md) | Workspace publication and privacy metadata |
| `backups/`, `private/`, `.cache/`, `.tools/` | Ignored local data and tooling; namespace new project data by project name |

The [ownership map](docs/organization.md) explains project boundaries, independent
Cargo workspaces, shared local tooling and evidence retention.

## Working here

Read [AGENTS.md](AGENTS.md), the [coding standard](docs/coding-standard.md) and
the [commit standard](docs/commit-standard.md). Rust remains the default for
authored implementation. Use the [API checklist](docs/rust-api-checklist.md)
for code changes and the [assurance workflow](docs/assurance-workflow.md)
for experiments and evidence.

From the repository root, the available source/text verification command is:

```sh
projects/os-evaluation/target/release/benchctl verify-public-snapshot --root . --ledger projects/os-evaluation/quality/evidence.json --manifest quality/publication.json
```

Build the verifier with
`cargo +1.96.0 build --manifest-path projects/os-evaluation/Cargo.toml --release --locked -p benchctl`
if needed. This checks retained raw replays and local references within the
[documented scope](projects/os-evaluation/tools/benchctl/docs/api.md); it does not qualify new
firmware, radio behavior or the iPhone USB connection.
