# OS evaluation working agreement

Apply the [workspace agreement](../../AGENTS.md). Read the
[project overview](README.md), [benchmark plan](docs/benchmark-plan.md) and
[experiment gates](docs/assurance-workflow.md) before changing a measurement.

- Keep workloads in `crates/bench-core`, collection/analysis in `tools/benchctl`,
  target adapters in `firmware/`, and research, methods and results here.
- Maintain the current API/safety records for
  [bench-core](crates/bench-core/docs/api.md),
  [benchctl](tools/benchctl/docs/api.md),
  [HAL firmware](firmware/esp-hal/docs/api.md) and
  [IDF firmware](firmware/esp-idf/docs/api.md) when their contracts change.
- Run host Cargo commands from this project root with the pinned toolchain:
  `cargo +1.96.0 test --workspace --locked`. Use the corresponding formatter,
  Clippy and documentation checks for changed code and APIs.
- Enter the [HAL](firmware/esp-hal/README.md) or
  [IDF](firmware/esp-idf/README.md) firmware directory before target commands so
  Cargo loads its target configuration. Follow its exact toolchain, feature and
  SDK pins. Accounting and offered-load probes require their isolated SDK
  configurations; ordinary queue defaults are insufficient.
- For IDF builds with an external target directory, set
  `CARGO_WORKSPACE_DIR` to the firmware directory. Preserve a variant's ELF,
  configuration and provenance before another variant overwrites its output.
- Keep workload inputs, correctness checks, timed endpoints and aggregation
  comparable. Record primitive, priority, executor/thread and instrumentation
  differences. Preserve timer units and raw values; nominal clock scale and
  register readback do not establish electrical calibration.
- Version a changed acceptance rule or method before collecting new comparisons.
  Use explicit qualification/comparison purpose, freeze the prospective inputs
  and schedule, and retain the complete cohort under `results/`. Do not pool
  incompatible protocols or promote diagnostic pilots into comparison data.
- `benchctl` capture commands can reset the board. Follow the experiment's
  full-flash backup, transfer verification and restoration procedure for board
  work; a host replay does not establish physical execution or recovery.
- Preserve original ledger, input-manifest and measurement identities. Current
  documentation can evolve as a labeled derivative. From the workspace root,
  use the public-snapshot command in [README.md](README.md) for available-data
  verification; its relocation/retirement records do not satisfy complete
  original-bundle method admission or restore missing artifacts and ancestry.
