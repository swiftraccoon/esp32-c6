# Privacy gate

Personal data belongs in ignored local storage. Publish sanitized evidence
derivatives with their own hashes. Keep original logs, full flash images and
ambient radio records under `private/` or `backups/`. Evidence-retention rules
never authorize publishing personal data.

The Rust `privacy-guard` reads staged Git objects, including binary-marked logs.
Cleaning a worktree copy after staging does not clean the index.

```sh
cargo +1.96.0 build --release --locked -p workspace-tools --bin privacy-guard
target/release/privacy-guard install
target/release/privacy-guard check
target/release/privacy-guard history
```

Installation configures local `pre-commit`, `prepare-commit-msg`, `commit-msg`
and `pre-push` hooks, refusing to overwrite unrelated hooks. The preparation
hook also checks commits made with Git's ordinary pre-commit bypass option.
Hooks reject missing private configuration and an executable that differs from
its staged source. Rebuild and reinstall after editing the guard. New clones
need installation because Git does not transfer local hooks.

Private identifiers are initialized from the account name, home-directory name
and, on macOS, account display name. They stay in `.git/privacy/terms.json`,
inside a private directory. Additional identifiers can be recorded there without
publishing them. Missing, empty or malformed local configuration fails closed.
Diagnostics report object IDs and rule names, never matched values or lines.

Checks cover personal paths and configured identifiers; emails and hardware
addresses; concrete USB serial paths; device identifier fields; raw wireless
frames and derived packet JSON; commit identities/messages; annotated tags;
existing ref names and proposed push destinations. History checks scan every
reachable object, reject shallow history and ignore Git replacement objects.
Text handling includes UTF-8, BOM-marked UTF-16/32, ASCII in NUL-bearing logs
and JSON escapes. Unsupported binary input and recognized containers fail
closed unless their exact historical bytes have been reviewed.

[Artifact exceptions](../quality/privacy-allowlist.json) apply only to exact
hashes of existing benchmark/control fixtures and compressed source/map archives.
They cannot exempt personal identifiers. Do not add exceptions automatically to
make a rejection pass. There are no directory-wide or filename-wide exemptions.
Prefer sanitized text summaries and ignored originals for new evidence.

CI scans the tracked snapshot and full fetched history with generic rules. It
does not receive the private identifier list; local hooks add that protection.
Remote branch protection is not configured by these files.

This enforces specified formats and identifiers; it does not prove arbitrary
content anonymous. Unknown names in prose and arbitrary encodings need separate
rules. Someone controlling Git configuration can disable hooks; the repository
agreement prohibits doing so. Do not bypass rejections or assume raw captures
are safe because an automated check passed.

[Review and controls](../tools/workspace-tools/docs/api.md) describe validation and API scope.
The public snapshot verifier checks artifact consistency, not privacy.
