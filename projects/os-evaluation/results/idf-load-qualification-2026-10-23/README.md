# IDF offered-load qualification

Collected **2026-10-23** under the
[prospective contract](../../docs/experiments/idf-load-qualification.md) and
[protocol 5](../../docs/experiments/idf-load-protocol.md).
**Both scheduled physical boots pass the qualification controls**, with no
replacement attempts. [RESULTS.md](RESULTS.md) and [analysis.json](analysis.json)
retain the actual observations and their limits. Work-phase losses and
synthetic deadline misses remain adverse outcomes rather than qualification
failures or discarded samples.

The [schedule](schedule.json) has one conditional qualification attempt per
profile, disabled then enabled. Each image runs two generator controls and four
250 ms work pulses. All 2,016 nominal release slots per boot remain visible, including
generator misses and queue-full outcomes. They are not assigned zero latency.

[INPUTS.sha256](INPUTS.sha256) freezes the exact images, source, configurations,
bindings, collector and provenance before collection. Dirty-at-build descriptors
are disclosed in the two provenance files. The current complete private8MB
image was independently verified against the device before replacement. It
was restored across all 8,388,608 bytes, separately digest-verified and reset;
the [device-state record](device-state.json) retains those receipts.

Final QA reproduced a host reader's one-ULP false rejection of a saved
fractional mean. The subsequent [correction record](reader-correction/correction.json)
and [review](../../tools/benchctl/docs/api.md) identify the
corrected reader separately from the unchanged frozen collector. All original
raw captures, saved reports/replays and 69 frozen inputs remain unchanged.
Exact comparison remains enforced. The
[independent result review](RESULTS.md) checks raw
conservation, payloads, causal endpoints and rejection paths.

[SHA256SUMS](SHA256SUMS) seals every retained regular file other than itself.
It includes the post-capture analysis and corrected-reader evidence; it does
not replace or expand the prospective input freeze.

This is qualification, without a comparative estimator, sustainable-capacity
claim, physical CPU busy percentage, global runtime memory peak or OS choice.
