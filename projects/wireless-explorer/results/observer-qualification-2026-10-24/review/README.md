# Recorder review artifacts

These files retain sanitized source snapshots, controls and output for the
[passive recorder qualification](../README.md).
Original evidence remains in ignored private storage. Public copies replace
the personal home-directory prefix with `/Users/builduser`; their checksums
describe those derivatives. Build outputs, Cargo targets,
dependency caches, compiled test executables and generated fixture directories
are excluded. The fixture constructors remain in the Rust test sources.

| Artifact | Scope |
| --- | --- |
| [First host snapshot](host-probe/Cargo.toml), [controls](host-probe/src/observation/review.rs), [original output](host-controls-attempt-1.log) | Nine tests passed and the hidden-suffix rejection test failed. All three capped JSON files admitted invalid trailing bytes. This is an intentionally retained failing version. |
| [Final host snapshot](host-probe-final/Cargo.toml), [controls](host-probe-final/src/observation/review.rs), [original output](host-controls-attempt-2.log) | Twelve tests passed after bounded reads, startup checks and file inventories were corrected. |
| [Buffer controls](buffer-controls.rs), [output](buffer-controls.log) | Six independent controls plus two imported source tests passed. |
| [Reference escape control](reference-escape.rs), [diagnostic](reference-escape.log) | Compilation failed as expected because a slot reference cannot escape its closure. |
| [First workspace output](host-tests.log), [final workspace output](host-tests-final.log) | Actual workspace test command output before and after the archive correction. |
| [Initial inputs](review-inputs.sha256), [final inputs](final-review-inputs.sha256) | Digests of the reviewed repository source files at the recorded snapshots; paths are relative to the repository root. They are not hashes of later source revisions. |
| [Callback disassembly](ieee-rx-disassembly.txt) | Build-03 callback inspection; the review records its ELF/configuration identities and limits. |

[SHA256SUMS](SHA256SUMS) covers every regular file in this review directory except
itself. Paths in that manifest are relative to this directory. Verify it here:

```sh
shasum -a 256 -c SHA256SUMS
```

## Reproduction paths

The preserved manifests and Rust `#[path]` attributes contain the sanitized
checkout prefix `/Users/builduser/Git/esp32-c6`. They import the repository's
`wireless-protocol`, control transcript module, or firmware buffer instead of
duplicating those dependencies. The logs retain the `.cache` path suffixes.
Rust 1.96.0 and the locked host dependencies were used on macOS. Symlink controls
use Unix APIs. Path substitution does not establish a new test execution.

For a new run, copy the relevant probe and control sources to a scratch
directory. If the checkout has a different absolute path, replace only that
prefix in the copied manifests/path attributes and record the substitution.
Use the matching repository source revision or verify its recorded input
digests before interpreting the result as a reproduction. The host fixture
constructor writes synthetic archives beside its Cargo manifest, so run copied
sources rather than writing generated fixtures into this evidence directory.
Keep new logs distinct from the retained originals.

The first host snapshot is expected to fail its hidden-suffix control; the final
snapshot is expected to pass. The reference-escape program is expected to fail
compilation. These controls do not access serial hardware.
