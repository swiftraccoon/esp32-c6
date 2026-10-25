# Wireless explorer working agreement

Apply the [workspace agreement](../../AGENTS.md). Read the
[protocol](docs/protocol.md), [observation contract](docs/observation.md) and
[API/safety contract](docs/api.md) for changes to capture or replay behavior.

- Keep the host protocol/client in this Cargo workspace and device code in the
  independent `firmware/` workspace. Preserve the selected Rust, SDK, wrapper
  and dependency pins; use the [firmware instructions](firmware/README.md) for
  target builds and the [client instructions](clients/README.md) for capture.
- Run host checks from this project root, including
  `cargo +1.96.0 test --workspace --locked`. For buffer changes, also compile
  and run its standalone host controls:
  `rustc +1.96.0 --test --edition=2024 firmware/src/buffer.rs -o target/buffer-tests`
  followed by `target/buffer-tests`. Check firmware changes from `firmware/`
  so Cargo loads its target configuration; set `CARGO_WORKSPACE_DIR` to that
  directory when using an external target directory.
- Preserve explicit receive windows, bounded storage and truthful loss,
  truncation and timestamp semantics. One shared RF path cannot continuously
  cover every radio/channel; no reports do not establish absence of traffic.
- Keep observation modes receive-only. Main owns radio transitions and serial
  output. Callbacks borrow SDK buffers only until return, copy into bounded
  slots, and do no allocation, formatting, waiting or serial I/O. Preserve the
  buffer publication, SDK frame-return and shutdown invariants in the contract.
- Keep live/save/replay validation aligned. Exercise fragmented, truncated,
  corrupted, incomplete and retained-failure cases when their paths change.
  Do not overwrite an existing attempt or rehabilitate it by dropping errors.
- Raw packets, addresses and reconstructed packet JSONL belong under ignored
  workspace `private/wireless-explorer/`. Review aggregate outputs for personal
  data before publishing sanitized derivatives and their provenance.
- Verify the intended board/interface before opening a port. Serial opens and
  control-line changes can reset it; firmware revision determines UART baud.
  Follow the full-flash backup/restoration procedure before changing flash.
- Use `target/release/wxctl verify-probes --ledger results/evidence.json` for
  the existing control ledger. Recorder sessions use `replay-observation`.
  Keep those validators' coverage limits explicit; known-peer fidelity, RF
  silence, iPhone access and recovery need their own physical observations.
