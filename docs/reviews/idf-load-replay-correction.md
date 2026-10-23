# Exact floating-point archive replay correction

Reviewed **2026-10-23** after both finite load qualification captures and before
finalizing their evidence. Whole-archive replay exposed a host reader defect:
valid saved results were rejected even though standalone raw replay reproduced
the originally written JSON bytes. This was a false rejection, with no altered
sample, favorable acceptance or firmware behavior established by the defect.

## Reproduction and cause

In the disabled pilot, the precise path is
`phases[5].conditional_latencies.send_entry_to_completion_us.mean_us`.
The integer sum/count are **2,132,498 / 773**. The fresh calculation and saved
decimal are `2758.7296248382922`, binary64 bits `0x40a58d75916305a1`.
The default serde_json 1.0.154 reader instead produces
`2758.729624838292`, bits `0x40a58d75916305a0`: one ULP lower.
Strict equality therefore failed at the top-level `phases` field. An independent
walk over every retained mean in both original reports found this sole
sum/count-to-parsed-bits mismatch. Python's independent raw-integer calculation
also retained the correctly rounded value; checking printed numbers alone did
not expose the reader's extra rounding.

The pinned dependency's own
[feature documentation](https://github.com/serde-rs/json/blob/v1.0.154/Cargo.toml#L67-L74)
distinguishes default best-effort parsing from the `float_roundtrip` feature,
which preserves binary64 through serialization and parsing. Its
[selected parsing paths](https://github.com/serde-rs/json/blob/v1.0.154/src/de.rs#L623-L672)
confirm that this is a parser choice. The correction enables that one feature
on the existing exact version. No dependency version or lockfile entry changes.
The documented float-parsing performance cost concerns host JSON parsing;
it is not a measured device overhead.

## Correction boundary

Report comparison remains exact. There is no tolerance, rounding normalization,
changed mean calculation, substituted rational field, ignored field or rewritten
capture. The original 69 frozen inputs, frozen source, collector executable,
original raw serial, provenance, capture reports and standalone replay outputs
remain the evidence of what was run. The corrected live reader is a subsequent
host-only revision; it must have its own commit and executable identity. It
does not retroactively become the collector used for either boot.

The firmware, physical qualification contract and data are unchanged. Final
whole-archive replay must use the corrected reader and still require exact
report-field equality and byte-identical original standalone replay output.
No additional physical boot is inferred from a host correction or its tests.

## Regression evidence and API applicability

Existing [all-54 entry dispositions](../rust-api-checklist.md) and the
[load host review](idf-load-host.md) apply. This is the affected delta:

| Entries | Decision and evidence |
| --- | --- |
| C-SERDE, C-VALIDATE | Applies: the same pinned serializer/reader now preserves the observed finite binary64 value exactly. Archive comparison remains strict and a one-ULP altered report is rejected. |
| C-FEATURE | No new authored feature definition or name. The manifest selects upstream's existing `float_roundtrip` feature without renaming it or changing default features. |
| C-PERMISSIVE | Existing dependency-license disposition is unchanged: same pinned package, no new transitive package or distribution choice. |
| C-INTERMEDIATE, C-NO-OUT | Applies: tests use exclusive temporary archives and retain original capture/replay bytes. The physical raw input is read without modification; synthetic ELF/provenance explicitly belong to the host-only graph fixture. |
| C-GOOD-ERR, C-FAILURE | Existing behavior applies: genuine report mismatch still returns a contextual error rather than being repaired or ignored. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE | Applies at private application scope: this review records the precise trigger, dependency source, evidence boundary and regression commands. |

No new public API, dependency package, unsafe/FFI, concurrent device path,
allocation policy or firmware resource lifetime is introduced. Other checklist
dispositions are inherited unchanged.

Two new tests were run **before enabling the feature** and failed for the
intended reasons. The first constructed the actual numerator/count from integer
samples and detected the exact-bit serialization/read-back loss. The second
copied the original disabled pilot's raw transcript into a temporary host-only
archive and reproduced `load capture report differs from replay: phases`.
After enabling the feature, both pass. The latter also checks that successful
replay changes neither saved report nor replay bytes, then changes only that
mean by one ULP and requires rejection through the actual whole-archive path.

Executed commands:

```sh
cargo +1.96.0 test --locked -p benchctl observed_fractional_mean_survives_exact_json_round_trip -- --nocapture
cargo +1.96.0 test --locked -p benchctl physical_fractional_mean_replays_exactly_and_rejects_one_ulp_tampering -- --nocapture
cargo +1.96.0 fmt --all
cargo +1.96.0 test --locked -p benchctl load::tests -- --nocapture
cargo +1.96.0 clippy --locked -p benchctl --all-targets -- -D warnings
```

All **17 load tests** and Clippy pass after the correction. Workspace-wide
verification, release executable identity and complete original archive replay
are separately retained by the integration owner before sealing the results.

Integration verification completed locally on macOS with Rust1.96.0:
`cargo fmt --all --check`, `cargo test --locked --workspace`,
`cargo clippy --locked --workspace --all-targets -- -D warnings`,
`cargo doc --locked --workspace --no-deps`, and
`cargo build --locked --release -p benchctl` all pass. The workspace run has
**98 unit tests and two doctests**. The corrected release reader SHA-256 is
`4171af48b87cebfd8ec408be9f4fe6f96a4ba0fe49061a21bc7068d18911968f`;
the unchanged physical collector is
`f3211f0b3762cb1ba96ef81b6110a74eefc542076abcf55ae5f3720610db8cbe`.
The corrected `replay-archives --root .` reproduces **66 protocol transcripts
and one study summary** without altering saved replay bytes. The historical
`qualify-method --root . --method quality/methods/idf-queue-v1.json` also passes
its two positive and ten negative controls. All69 frozen load inputs still
match their manifest. These are local host checks, not hosted CI or additional
physical captures.
