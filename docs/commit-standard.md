# Commit standard

Adopted 2026-10-22 at the user's request: **Conventional Commits 1.0.0**,
the current stable specification on the official site at review time.
The [versioned specification](https://www.conventionalcommits.org/en/v1.0.0/)
is normative. Its
[upstream repository](https://github.com/conventional-commits/conventionalcommits.org)
and [versioned source](https://github.com/conventional-commits/conventionalcommits.org/blob/master/content/v1.0.0/index.md)
provide reference material. The repository's `content/next` is development work;
future upstream changes do not silently replace this adoption.

## Specification requirements

The subject has this structure; bracketed parts are optional placeholders:

```text
<type>[optional scope][!]: <description>
```

Use `feat` for a new application/library feature and `fix` for a bug fix. Other
noun types are permitted. An optional scope names a codebase area in parentheses.
The colon and following space are required. A body, if present, starts after a
blank line; footers follow the body after another blank line.

A breaking change uses `!` immediately before the colon, or a
`BREAKING CHANGE: <explanation>` footer, or both. The specification also accepts
`BREAKING-CHANGE` as the footer token. Breaking changes can occur with any type.
Other footer tokens use hyphens rather than spaces, followed by `: ` or ` #`.
These rules come from the [1.0.0 specification](https://www.conventionalcommits.org/en/v1.0.0/#specification).

## Repository choices

The following conventions supplement the specification:

- Use lowercase types/scopes and a concise, imperative description of the result.
  Scope is optional when a meaningful change spans areas.
- Choose `docs` for documentation, `test` for tests, `perf` for performance work,
  `refactor` for behavior-preserving restructuring, `build` for build/dependency
  changes, `ci` for automation, and `chore` for other maintenance. Use `revert`
  when reverting a commit and identify the reverted hash in the body or footer.
  This list is a repository convention, not an upstream closed type list.
- Explain the compatibility impact and migration in a `BREAKING CHANGE:` footer
  for every breaking change, including subjects that use `!`. Requiring that
  footer even with `!` is our additional rule. Apply it during initial development
  as well as after a release.
- Make commits at meaningful completed checkpoints while working. Group one
  coherent purpose with its relevant documentation, lockfiles, review evidence
  and measurements. Split independent changes when practical; avoid empty or
  arbitrary checkpoint commits.
- Review the staged changes and record relevant verification in the body when
  useful. Distinguish builds, host tests and on-board results. Preserve failed
  trials and their explanations with the corresponding experiment evidence.
- Commit messages do not automatically change package versions, create releases
  or tags, or push commits. Perform those actions when they are part of the
  authorized task. Adopting this convention installs no release automation.

Suggested scopes describe the affected area:

| Scope | Area |
| --- | --- |
| `bench` | Shared workload, capture protocol, collector and benchmark behavior |
| `runtime` | Firmware adapters, execution contexts and platform integration |
| `docs` | OS research, reference sheets, standards and repository instructions |
| `results` | Preserved trials, provenance, build evidence and result analysis |

Use a narrower noun such as `esp-idf` when it makes the change clearer. These
scopes are suggestions, not a requirement to force every change into one bucket.

## Examples

These illustrate message shapes; they do not claim the changes have been made:

```text
feat(bench): add validated compute trial capture
fix(bench): reject samples after a failed boot
build(runtime): pin compatible ESP-IDF toolchain
docs: adopt Conventional Commits 1.0.0
docs(results): record independent runtime trials
```

```text
feat(bench)!: require protocol version 2

Add the timing-source fields needed to validate comparable measurements.

BREAKING CHANGE: protocol 1 transcripts are rejected. Rebuild matching version 2
firmware and collector, and keep earlier trials under their original protocol.
```
