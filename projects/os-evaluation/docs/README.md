# OS evaluation documentation

[Project](../README.md) · [Results](../results/README.md)

- [Comparison](comparison.md), [benchmark plan](benchmark-plan.md) and
  [hardware coverage](hardware-coverage.md) define the evaluation.
- [Platform sheets](os) and [research](research) record source support,
  measurement assumptions and unresolved questions.
- [Experiment contracts](experiments) specify acceptance rules and schedules.
  [Build records](builds) describe experiment-specific upstream/configuration work.
- [Evidence gates](assurance-workflow.md) and [ledger guide](evidence-ledger.md)
  define the OS evidence schema and verification procedures.
- Current API/safety contracts live with [bench-core](../crates/bench-core/docs/api.md),
  [benchctl](../tools/benchctl/docs/api.md),
  [HAL firmware](../firmware/esp-hal/docs/api.md) and
  [IDF firmware](../firmware/esp-idf/docs/api.md).

Use each component's README for current commands. Dated provenance describes
the original build; it does not establish that current sources are identical.
Shared hardware, standards and source catalogs live in the workspace docs.
