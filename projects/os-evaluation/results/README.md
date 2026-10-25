# OS evaluation results

Each collection retains observations and context for its stated conclusion.
The table distinguishes comparisons, qualification controls and unresolved
failures; none establishes an overall OS ranking.

| Collection | Retained purpose and outcome |
| --- | --- |
| [Initial compute](initial-2026-10-22/README.md) | 15-boot harness/compiler baseline; runtime flash clock was unverified, so later clocked data governs active timing comparisons. |
| [Clocked handoff](clocked-handoff-2026-10-23/README.md) | 15 boots with runtime clock readback and checked thread/task round trips; preserves the placement-sensitive compute result. |
| [Alignment probe](alignment-probe-2026-10-23/README.md) | Five alternating captures show that code placement changes observed compute timing; the precise hardware mechanism remains unproved. |
| [Ariel startup](ariel-qualification-2026-10-22/README.md) | Two upstream startup/UART checks establish async and stackful-thread feasibility for the recorded development toolchain. |
| [Queue pilots](queue-api-qualification-2026-10-23/README.md) | Two diagnostic captures provide positive inputs for the frozen queue method; excluded from the comparison cohort. |
| [Matched queue APIs](idf-queue-api-2026-10-23/RESULTS.md) | All 24 scheduled boots form twelve std/native pairs on the same kernel. Raw samples support the API-path comparison and its limits. |
| [Ariel native tests](ariel-native-tests-2026-10-23/RESULTS.md) | Twelve expected console outputs; the first guard lacks a wait receipt and printed yield arithmetic does not validate a switch count. |
| [Zephyr native tests](zephyr-native-tests-2026-10-23/RESULTS.md) | Three failed v1 qualifications retain the missing frequency header and image warning despite upstream success banners. |
| [Zephyr Rust build](zephyr-rust-build-2026-10-23/README.md) | Build/configuration evidence for the unresolved generated-getter blocker; no image or board pass. |
| [Accounting controls](idf-accounting-qualification-2026-10-23/RESULTS.md) | Three diagnostic boots establish task-attribution/allocation controls and preserve accounting blind spots. |
| [Finite load](idf-load-qualification-2026-10-23/RESULTS.md) | Two profiles retain all jobs, queue-full losses and deadline misses. Finite-pulse qualifications, not capacity estimates. |
| [RTIC monotonic](rtic-qualification-2026-10-23/RESULTS.md) | One failed capture contains a second ROM startup; two scheduled attempts were not run. The reset cause is unresolved. |

## What belongs in a collection

Retain complete primary captures, reports needed to interpret/replay them,
configuration and source identities, schedules, acceptance rules and restoration
receipts. An adverse result is evidence when it constrains a conclusion or
demonstrates a validator's rejection boundary. Keep that evidence with the study.

Parsed `records.jsonl` mirrors, generated build databases, duplicate source
archives, empty auxiliary logs and resolved helper-build iterations are not
needed here. Large generated task bindings are represented by exact, labeled
ABI excerpts for the counter types used; they do not reproduce the full SDK.
Original input/checksum declarations are unchanged. The
[publication inventory](../../../quality/publication.json) marks retired and
previously unavailable files explicitly; full original-bundle verification is
therefore a different check from available-data replay.

Use the [collector](../tools/benchctl/README.md), [benchmark plan](../docs/benchmark-plan.md)
and [evidence gates](../docs/assurance-workflow.md) for new work.
[run-template.json](run-template.json) is an unmeasured template; unknown values
stay `null`. Keep private identifiers and full-flash images out of Git.
