# Workspace utilities

Host utilities for repository maintenance:

- `privacy-guard` installs Git hooks and checks staged files and history for
  personal data.
- `style-sources` checks the local style references and updates them from their
  upstream repositories.

From the repository root:

```sh
cargo +1.96.0 build --offline --release --locked -p workspace-tools
target/release/privacy-guard install
target/release/privacy-guard check
target/release/style-sources check
```

Build dependencies must be provisioned once. Reading the checked-in guides and
running their checker are offline operations. See the [privacy policy](../../docs/privacy.md)
and [style references](../../docs/standards/README.md) for usage and update commands.

The [API and safety contract](docs/api.md) documents input handling, Git-hook
behavior, update recovery and Rust guideline decisions.
