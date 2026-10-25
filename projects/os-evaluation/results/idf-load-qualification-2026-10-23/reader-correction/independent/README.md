# Independent decimal-reader reproduction

These are host-only inspection probes and outputs, recorded after the physical
captures. They neither modify a capture nor establish a new board result.
The original [reports](../../runs/off-01/run.json) retain the saved decimal;
the [review](../../RESULTS.md) records its path/bits.

The default parser's output identifies the sole mean whose parsed binary64
differs from a fresh division of its recorded integer sum/count. The roundtrip
probe adds one known-value print; no additional mismatch line means all visited
means preserve their calculated bits. The first probe was originally compiled
against the existing default/std serde_json rlib. The second used its separate
locked Cargo project with only `float_roundtrip` added to the exact version.

From repository root, reproduce either probe in a temporary local project:

```sh
mkdir -p .cache/float-reader-repro/src
cp results/idf-load-qualification-2026-10-23/reader-correction/independent/Cargo.lock .cache/float-reader-repro/Cargo.lock
cp results/idf-load-qualification-2026-10-23/reader-correction/independent/Cargo.default.toml .cache/float-reader-repro/Cargo.toml
cp results/idf-load-qualification-2026-10-23/reader-correction/independent/default-probe.rs.txt .cache/float-reader-repro/src/main.rs
cargo +1.96.0 run --locked --manifest-path .cache/float-reader-repro/Cargo.toml -- results/idf-load-qualification-2026-10-23/runs/off-01/run.json results/idf-load-qualification-2026-10-23/runs/on-01/run.json
```

Then replace the two copied manifest/source files with `Cargo.roundtrip.toml`
and `roundtrip-probe.rs.txt` from this directory and rerun the same command.
The supplied lockfile pins the same package graph for both feature selections.
Provisioning dependencies can require network access; parsing these retained
files requires no device or live source lookup. No tolerance or data repair is
used by either probe.
