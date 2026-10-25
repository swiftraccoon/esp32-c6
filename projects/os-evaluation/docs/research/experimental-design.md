# Experimental design for the next C6 comparison

Research completed **2026-10-23**. This closes the design-research brief in
[research readiness](research-readiness.md); it specifies a bounded next study,
not new measurements or statistical validation of historical results.

## Question, reuse and scope

The next radio-off study should determine which observed communication cost
belongs to a selected Rust API, priority arrangement or native synchronization
path. Its application is choosing an implementation or deciding whether a
smaller runtime extension deserves investigation. It will characterize the
selected configurations on this board; there is no supplied application deadline
or justified overall OS score.

Use the named native scenarios in the [reuse inventory](../../../../docs/research/benchmark-reuse.md),
with source and operation counts retained. Follow the separate
[timing and concurrency contract](timing-and-concurrency-contract.md) when
introducing a common Rust adapter. An upstream aggregate yield benchmark and a
per-event request/reply distribution remain separate datasets.

Two primary methodological references inform this design:

- [NIST's design selection](https://www.itl.nist.gov/div898/handbook/pri/section3/pri33.htm)
  and [randomized blocks](https://www.itl.nist.gov/div898/handbook/pri/section3/pri332.htm)
  connect the design to its question and organize nuisance variation.
- Kalibera and Jones, [*Rigorous Benchmarking in Reasonable Time*, ISMM 2013](https://kar.kent.ac.uk/33611/45/p63-kaliber.pdf),
  distinguish variation between iterations, executions and builds. We adopt
  that separation and allocate repetition to actual sources of variation; their
  desktop/VM repetition counts do not transfer to our C6. The paper's
  ratio-of-means analysis is not a method for estimating scheduler tail latency.

## Define what a repetition samples

| Level | What varies | What it can establish |
| --- | --- | --- |
| Event within one boot | Timer phase, cache state, scheduling and interference during that execution | A conditional event distribution; preserve sample order and any failures. |
| New boot of one exact image | Startup state and subsequent execution | Boot-to-boot repeatability of that image, with actual reset evidence. |
| Paired block | Two configurations run close together on the same board and supply | A local paired contrast with less exposure to slow environmental drift. |
| Collection session | Recorded start time, host/supply state, elapsed time and available ambient readings | Session sensitivity; a new label alone does not establish independence. |
| New binary/layout | Compiler, linker, feature or placement policy | Build/configuration sensitivity. Reflashing identical bytes is not a new build variant. |
| Different board | Silicon, flash, carrier and manufacturing variation | Hardware-population evidence, which our single board cannot supply. |

Preserve board ID, image/source hashes, boot/reset evidence, block, session,
order, configuration, counters and timestamps. A host challenge detects stale
captures; it does not independently prove that the board reset. An image's
fixed layout is part of its treatment. Rebuilding to identical image bytes does
not add sampled layout variation.

## A bounded collection design

These are repository choices for the next **characterization** study, not
universal minimum sample counts or a product acceptance test:

1. Finish correctness/timer qualification first. Archive those exploratory runs
   separately. Freeze the two images, configuration, observer sequence, warmup,
   event count and analysis plan before the characterization captures.
2. Compare two eligible implementations of **one** observable contract at a
   time. Keep equal-priority, higher-priority, cooperative-async and loaded cases
   separate. Unsupported contracts produce a status, not a fabricated number.
3. Schedule **12 paired blocks** across three recorded sessions, four
   blocks per session. Each block has one boot per configuration. Within each
   session, randomly permute two AB and two BA blocks; commit the actual schedule
   and random seed before collecting results. This balances order, rather than
   relying on all A runs preceding all B runs. Sessions describe collection
   conditions; no minimum elapsed time is implied.
4. In each boot, use the same two-second settling period and eight checked
   warmup exchanges, then retain **1,000 checked events** for a per-event
   request/reply contract. Native aggregate suites keep their upstream batch
   counts instead; their output is a batch average. Warmup is a declared
   condition, not proof that every cache or background service has stabilized.
5. Use fixed 160 MHz CPU and verified flash clock settings, radios/DFS/automatic
   sleep disabled, unchanged logging/watchdog choices and no output during
   timing. Record active clock readback before/after each boot. Keep compute
   placement diagnostics separate from the primary communication outcome.
6. Retain incomplete blocks, incorrect replies, resets and timeouts. An
   application failure is a result. If a documented collector/fixture error
   invalidates a capture, preserve it and allow one repeat of the whole block
   under the original order. A repeated fixture failure leaves the block
   incomplete. An application failure consumes its scheduled block; it is not
   replaced to obtain 12 successes. Do not repeat a slow successful run to
   replace its value. Report attempted and completed blocks and reasons
   separately.

Twelve blocks is a bounded collection budget that gives balanced sessions and
order. It does not guarantee a desired confidence-interval width, characterize
rare failures or make different runtime models equivalent. A timing backend or
adapter change after qualification starts a new frozen study.

## Prespecified analysis

Use gross durations in native counter units, with frequency and conversion
recorded. Keep the empty-observer distribution alongside them. Do not subtract
an observer percentile from every latency or compare resolution below the
qualified counter's uncertainty. Quantization/systematic uncertainty and
statistical uncertainty are separate.

For each boot, report count, correctness/failures, p50, p95, p99, minimum and
largest observed duration. Preserve the collector's nearest-rank definition:
sort ascending and choose one-indexed `ceil(p * n)`. Thus p50 for an even count
is the lower middle observation; it is not an average of the middle two.
Historical summaries retain their original definitions.

The primary contrast is **B's per-boot p50 minus A's per-boot p50 within each
block**, in ticks and converted time. Publish all 12 contrasts and their median
(average the two central block contrasts for an even count). Also show session
and AB/BA order effects and each boot's upper percentiles. A pooled distribution
may be supplementary; it does not turn events into independent boots or define
the distribution of a future board.

If independent, identically distributed block contrasts are a defensible model,
use their third-smallest and tenth-smallest values as a conservative median
confidence interval. The binomial order-statistic construction is described in
[NIST TN 2119, section 5.3](https://nvlpubs.nist.gov/nistpubs/TechnicalNotes/NIST.TN.2119.pdf).
For 12 continuous independent contrasts its coverage is
`1 - 2 * (1 + 12 + 66) / 4096 = 0.96142578125`, or about **96.14%**; report that
actual level rather than calling it exactly 95%. Ties can make coverage more
conservative. This interval describes the median paired boot-p50 contrast,
not an individual-event percentile, a ratio of medians or a worst-case bound.

Inspect chronological plots and per-session/order summaries before relying on
that model. [NIST's autocorrelation guidance](https://www.itl.nist.gov/div898/handbook/eda/section3/autocopl.htm)
explains why event dependence matters and why absence of detected correlation
does not prove randomness. With clear drift or session interactions, retain the
descriptive contrasts, report model limitations and design a follow-up; do not
silently pool, detrend or bootstrap individual events to obtain narrow intervals.
If fewer than 12 blocks complete, retain all outcomes and report the fixed study
as incomplete; the specified 12-block interval no longer applies.

## Tails, failures and stopping

A 1,000-event boot has roughly ten observations in the upper 1% of a continuous
population **if** events were independent draws from that population. For its
upper 0.1%, the chance of observing none is `0.999^1000`, about 36.8%. These are
binomial illustrations, not independence claims about our events. Do not add
precise p99.9 claims merely because the output format accepts that percentile.
At 64 samples, nearest-rank p99 equals the observed maximum.

For zero failures in `n` independent identical Bernoulli trials, the one-sided
95% exact upper failure-probability limit is `1 - 0.05^(1/n)`. See
[NIST's exact binomial method](https://itl.nist.gov/div898/software/dataplot/refman2/auxillar/exacbino.htm).
For 1,000 such trials that is about 0.299%; zero observed failures is not proof
of zero failure probability. Shared-state exchanges, boots and hours of uptime
are not interchangeable Bernoulli trials. Report failures and exposure at
their actual level before selecting a reliability model.

Stop at the frozen collection budget and report its uncertainty even if the
comparison is inconclusive. Correctness or qualification failures may stop a
study early, with all attempts retained. Do not keep collecting until an
interval excludes zero. No minimum useful performance difference is invented
for this broad survey: a later API/configuration decision can supply a threshold
and use the pilot's relevant variation to choose a new fixed study. A narrower
claim or an inconclusive result is a valid outcome.

The interval accounts for sampling under its model, not calibration error,
layout bias, a timer defect or missing capabilities. Native test results can
motivate the smallest existing API/configuration remedy; they do not alone
justify a new OS.

## Completion boundary

The experimental-design research brief is complete: the question, reusable
assets, units, order, sample budget, analysis assumptions, failure treatment and
stopping rule are specified. Implementation must still archive the actual
schedule and manifest and qualify timer/adapter correctness. Statistical
software/collector changes need a separate Rust review and meaningful numerical
checks. Physical qualification remains to be performed; no new result is claimed.
