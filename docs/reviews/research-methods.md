# Research-method and next-experiment review

Reviewed **2026-10-23**. Scope: the five priority briefs from the
[research-readiness audit](../research/research-readiness.md), the corrected
platform sheets and the [next API experiment](../experiments/native-queue-comparison.md).
This is research/design review, not implementation or physical validation.

## Independent review

| Reviewer | Work and cross-check |
| --- | --- |
| `audit_measurement_gaps` | Traced pinned timer/concurrency sources, generated bindings and existing SDK symbols; authored the timing/concurrency brief. Reviewed the experimental-design interval, sample units, scope and incomplete-block treatment. |
| `audit_hardware_gaps` | Authored the complete rev0.2 ledger with actual SDK configuration and final ELF copy-path inspection. Independently checked the proposed private timer binding, task-state behavior and native queue experiment's safety/qualification limits. |
| `audit_platform_gaps` | Authored release/maintenance evidence and corrected IDF canonical links. Independently checked NIST's quantile construction and the design arithmetic; re-read pinned errata/newlib source and re-disassembled the final IDF copy functions, plus HAL ROM aliases. |
| Primary agent | Reviewed all briefs, reconciled released esp-rtos versus development APIs, wrote the fixed characterization design and concrete same-IDF experiment, and integrated navigation/milestones. |

Substantive corrections incorporated:

- Published esp-rtos 0.4.0 does not expose development `ThreadSpawner`; a matching
  package version is not evidence of identical APIs or dependencies.
- Ariel's qualified development checkout and published v0.6.0 have different
  compiler requirements. The release alternative is source-supported, not a
  newly built or physically qualified tuple.
- Twelve scheduled blocks are an attempt budget. Application failures consume
  it; only documented fixture invalidations permit a bounded repeat. The design
  does not collect until twelve successful blocks or a favored result.
- SYSTIMER source, enable and stall readback plus blocked/WFI continuity checks
  are explicit admission requirements for the next timer implementation.

## Verification

- `git diff --check` passed.
- A read-only local-link check found **115 existing local targets in 12 changed
  or added documents**, including this review record. It checks file
  existence, not remote content, fragments or hardware behavior.
- The errata author checked all 31 remote primary links in that brief. Other
  briefs cite inspected pinned sources; repository heads, release tags, package
  VCS metadata and actual measured lockfiles are distinguished.
- Independent arithmetic and a standard `awk` calculation agree on the
  12-block interval coverage **0.96142578125**, illustrative `0.999^1000`
  **0.367695**, and zero-failure upper bound **0.00299125** for 1,000 independent
  trials. Their independence and estimand limits are stated in the design.
- Changes are documentation only. No firmware source, lockfile or historical
  measurement was changed, no build/test/flash was run, and the board was not
  accessed. Existing archived ELFs were inspected read-only. Cargo tests would
  not validate these source/design findings.

The bounded research is sufficient to implement and qualify the named
radio-off API experiment. Its private timer linkage, Rust FFI invariants,
correctness, active configuration, final code placement and physical behavior
remain to be established. Later fixtures, radio/power contracts, arbitrary
memory copies and product recovery/security are still outside this closure.
