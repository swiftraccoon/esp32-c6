# Research and qualification gaps

The source surveys identify viable routes; the
[result collections](../../results/README.md) establish narrower observations
on this board. No overall platform choice follows from them.

## Follow-up research closure

The following source/configuration work supports the matched IDF queue method.
Its scope does not extend automatically to other images or hardware profiles.

| Contract | Established basis | Remaining boundary |
| --- | --- | --- |
| [Timing](timing-and-concurrency-contract.md) | Raw SYSTIMER reads, register decoding and endpoint/observer controls for the selected images | Nominal scale; independent electrical calibration and connected/sleep reliability are unqualified |
| [Concurrency](timing-and-concurrency-contract.md#concrete-next-comparison-and-its-admission-controls) | Same kernel, thread arrangement and priorities for std/native queue paths | Complete instrumented round trips; no isolated switch-cost inference or equivalence to async tasks |
| [Silicon exposure](errata-exposure.md) | Named linked copy routines, alignment and clock/RMT configuration traced for historical OS images | ROM-copy internals, arbitrary unaligned data, sleep/LP and continuous RMT need operation-specific controls |
| [Compatibility](platform-compatibility.md) | Exact released/development dependency and toolchain distinctions | Unbuilt tuples and upgrade effort remain unproved; a manifest version alone does not establish an API |
| [Experimental design](experimental-design.md) | Frozen paired schedule, complete cohort, declared estimator and failure handling | Quantized tied medians and consecutive sessions do not establish independent environmental replication |

## Current investigations

- [RTIC startup](rtic-startup-followup.md): obtain complete reset attribution
  before changing startup/watchdog behavior.
- [Zephyr native semantics](zephyr-native-audit.md): reconcile the compiled
  frequency header, image warning and per-row timed/corrected endpoints before
  a new qualification. Existing failed attempts retain their acceptance rule.
- [Zephyr Rust](zephyr-rust-feasibility.md): resolve the selected generated-code
  blocker and qualify the resulting image; source support alone is insufficient.
- [Native IDF API options](idf-native-api-options.md): evaluate wrapper fit and
  ownership before adopting a general-purpose queue adapter.
- [Accounting](idf-accounting-method.md) and [finite load](idf-load-method.md):
  qualify longer repeated workloads and a prospective instrumentation comparison.
  Current task attribution and finite overload are narrower than physical CPU
  utilization, sustainable capacity or global memory peaks.

The [hardware coverage plan](../hardware-coverage.md) tracks external IRQ
response, hardware offload, radios, low-power behavior, security, recovery and
fixtures. The [benchmark plan](../benchmark-plan.md) governs new comparisons;
the [utilization audit](utilization-audit.md) distinguishes elapsed latency
from the other resource questions. Adoption also needs measured update effort,
driver availability and realistic application requirements.
