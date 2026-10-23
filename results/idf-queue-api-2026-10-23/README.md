# Matched IDF queue API characterization

This is the **preregistration record, frozen before collection**. Completed
measurements will be recorded separately in `RESULTS.md`; this file and
`INPUTS.sha256` retain the pre-collection input record. The separate
[qualification pilots](../queue-api-qualification-2026-10-23/README.md) passed
and are excluded from this study. The question is whether the native queue API
changes the selected Rust request/reply path on the same IDF kernel; this is
not an operating-system ranking.

[study-manifest.json](study-manifest.json) freezes the schedule, source,
collector, ELF, merged-image and provenance hashes before collection.
[schedule.json](schedule.json) schedules twelve paired blocks across three
collection sessions, each balanced between two std/native and two native/std
orders. Session labels imply no minimum elapsed separation or independence.
The recorded permutation seed is `11647397086240600225`.

Each scheduled boot retains 1,000 checked request/reply and 1,000 observer
events using gross raw 16 MHz SYSTIMER ticks. Both variants use the same Rust
thread arrangement, priority 1 initiator / priority 5 worker, capacity-one u32
queues, 48,000-byte sample buffer, eight warmups, two-second settling and
SDK/compiler settings. Source, configuration, link maps and image artifacts
are under [artifacts](artifacts/). Device phase instrumentation contributes to
the path. Queues, worker startup/shutdown and main-thread readiness checks are
outside each timed interval.

The [experiment contract](../../docs/experiments/native-queue-comparison.md)
and [analysis design](../../docs/research/experimental-design.md) define
admission, failure handling, fixed stopping and the conditional median interval.
An application failure consumes its scheduled block. Only documented fixture
invalidations permit one block repeat; unsuccessful attempts remain archived.
No capture has yet contributed a comparative result to this directory.
