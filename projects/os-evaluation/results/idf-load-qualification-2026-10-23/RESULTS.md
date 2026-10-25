# IDF finite offered-load qualification results

**Both scheduled boots qualify under v1**, with no replacement attempts. Each
preserves every nominal release, including the deliberately missed control
slots, queue-full rejection and work completed after the offer window. This
qualifies this Rust/IDF method for subsequent study design. These two pilots
provide no statistics-configuration effect estimate or sustainable-capacity
result.

The [contract](../../docs/experiments/idf-load-qualification.md),
[exact protocol](../../docs/experiments/idf-load-protocol.md) and
[schedule](schedule.json) preceded collection. Implementation/contract commit:
`093153160316e01f0dea364ef999f3be15ddc9d2`; exact 69-entry
[input freeze](INPUTS.sha256): `b23d17ac28391e79d6f7d4e6547b8a258339a45f`;
ledger binding: `671276baa196932f6afb0dc1ca7563cf701e35e9`.
The two provenance files ([disabled](provenance-off.json),
[enabled](provenance-on.json)) retain Rust 1.96.0, IDF 5.5.6, sys 0.37.2,
configuration, task-accounting ABI excerpts and image identities. Both images were built
before the source commit and retain `160ff3d-dirty` app descriptors; archived
source matches the implementation commit. There is no clean-at-build claim.

The [disabled raw](runs/off-01/serial.raw) and
[enabled raw](runs/on-01/serial.raw) contain **4,032 nominal jobs, 24 snapshots
and 120 task records** across twelve phases. Each boot has all 2,043 protocol
records. [analysis.json](analysis.json) independently recalculates integer counts,
conditional distributions, FIFO/drain/deadline checks, 3,556 completed payload
checksums, memory indicators and enabled counter deltas directly from raw input.
Capture aggregates were consulted afterward. Exact distribution sums/counts are
retained as integers; no ratio or difference between profiles is calculated.
The offline rejection and numeric-persistence controls below check the actual
raw data and saved-report behavior.

## Admission and generator controls

Both actual queue controls accepted eight messages, rejected the ninth with
zero wait, verified FIFO contents and returned empty. The no-work pulse
completed all 125 jobs without missed slots or full rejection. The deliberately
late first alarm produced exactly four generator-missed slots and twelve
completed jobs, preserving misses rather than replaying a burst of old offers.
It is an expected generator-limited control, separate from work qualification.

Every work pulse has zero generator misses and meets the prospective maximum
attempt-lateness gate of period/4. Among work pulses, maximum attempt lateness
is **26 us** in the disabled boot and **22 us** in the enabled boot. These are
send-entry timestamps against the nominal grid, not independently captured queue-insertion
instants. Each phase has zero early wakes and exactly attempts+1 callbacks,
including the terminating cutoff wake. Successful stop/quiescence, empty queue,
unchanged clock configuration and joined-task/closed-timer completion receipts
are retained. No accepted job remains unfinished.

All durations below are nominal integer `esp_timer` microseconds. CPU160/flash80
and clock-register checks before/after do not independently calibrate frequency.
The **2,000 us deadline** is a synthetic method reference from nominal release,
not a product requirement. Completion exactly at cutoff counts in-window.

## Disabled-profile observations

One disabled-statistics boot, first in the fixed schedule. Each work pulse
lasts 250,000 us and executes the checked 4,096-round payload per accepted job;
controls execute zero rounds. The delayed control lasts 8,000 us.

| Phase / period us | Offered | Generator missed | Full | Accepted | In-window / drain | Deadline met | Offered deadline failures | Queue peak |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: |
| No-work / 2000 | 125 | 0 | 0 | 125 | 125 / 0 | 125 | 0 | 1 |
| Delayed / 500 | 16 | 4 | 0 | 12 | 12 / 0 | 12 | 4 | 1 |
| Work / 2000 | 125 | 0 | 0 | 125 | 125 / 0 | 125 | 0 | 1 |
| Work / 1000 | 250 | 0 | 0 | 250 | 250 / 0 | 250 | 0 | 1 |
| Work / 500 | 500 | 0 | 0 | 500 | 500 / 0 | 500 | 0 | 1 |
| Work / 250 | 1000 | 0 | 227 | 773 | 764 / 9 | 22 | 978 | 8 |

At period250, conservation is `1000 = 0 + 227 + 773` and
`773 = 764 + 9`; accepted deadline misses are **751**. All accepted work
finishes by the bound, with drain done **2,413 us after cutoff**. The final
end-marker/drain receipt occurs after the last payload completion.

Nominal-release-to-completion distributions are conditional on completed jobs:
the 227 rejected jobs have no completion latency. Percentiles use nearest rank.
Gross service is the start-to-finish wall span, including interruptions.

| Phase / period us | Conditional p50 / p95 / p99 / max us | Gross service min / p50 / p95 / max us | Maximum attempt lateness us |
| --- | --- | --- | ---: |
| No-work / 2000 | 36 / 36 / 43 / 83 | 2 / 2 / 2 / 3 | 25 |
| Delayed / 500 | 36 / 37 / 37 / 37 | 1 / 2 / 2 / 2 | 19 |
| Work / 2000 | 317 / 324 / 324 / 324 | 283 / 283 / 290 / 290 | 18 |
| Work / 1000 | 317 / 317 / 317 / 317 | 283 / 283 / 283 / 283 | 18 |
| Work / 500 | 317 / 317 / 317 / 318 | 283 / 283 / 283 / 283 | 19 |
| Work / 250 | 2835 / 2938 / 2948 / 2954 | 283 / 314 / 345 / 374 | 26 |

The disabled ABI has 32-bit native counter fields; measured task counters,
total counters and counter-clock declaration remain **null** throughout.
There is no disabled-profile CPU attribution estimate.

## Enabled-profile observations

One enabled-statistics boot, second in the fixed schedule, with the same phase
definitions and common authored timestamp/record/snapshot instrumentation.

| Phase / period us | Offered | Generator missed | Full | Accepted | In-window / drain | Deadline met | Offered deadline failures | Queue peak |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: |
| No-work / 2000 | 125 | 0 | 0 | 125 | 125 / 0 | 125 | 0 | 1 |
| Delayed / 500 | 16 | 4 | 0 | 12 | 12 / 0 | 12 | 4 | 1 |
| Work / 2000 | 125 | 0 | 0 | 125 | 125 / 0 | 125 | 0 | 1 |
| Work / 1000 | 250 | 0 | 0 | 250 | 250 / 0 | 250 | 0 | 1 |
| Work / 500 | 500 | 0 | 0 | 500 | 500 / 0 | 500 | 0 | 1 |
| Work / 250 | 1000 | 0 | 241 | 759 | 751 / 8 | 21 | 979 | 8 |

At period250, conservation is `1000 = 0 + 241 + 759` and
`759 = 751 + 8`; accepted deadline misses are **738**. Drain is done
**2,274 us after cutoff**; no accepted job violates the drain bound.

| Phase / period us | Conditional p50 / p95 / p99 / max us | Gross service min / p50 / p95 / max us | Maximum attempt lateness us |
| --- | --- | --- | ---: |
| No-work / 2000 | 41 / 41 / 47 / 90 | 2 / 2 / 2 / 6 | 28 |
| Delayed / 500 | 40 / 43 / 43 / 43 | 1 / 2 / 2 / 2 | 24 |
| Work / 2000 | 321 / 328 / 328 / 328 | 283 / 283 / 290 / 290 | 21 |
| Work / 1000 | 321 / 321 / 321 / 322 | 283 / 283 / 283 / 283 | 22 |
| Work / 500 | 321 / 321 / 321 / 322 | 283 / 283 / 283 / 283 | 22 |
| Work / 250 | 2883 / 2984 / 2992 / 2999 | 283 / 318 / 353 / 360 | 22 |

## Whole-phase task attribution

Enabled counters use the actual regenerated U64 ABI and `esp_timer_get_time`.
Before/after snapshots match the same five live task identities by handle and
task number. Producer/consumer/timer/coordinator identities come from the hello;
the fifth numeric SDK task is reported as other SDK task without inferring
physical idle residency from its attributed time.

| Enabled phase | Total-clock interval us | Producer | Consumer | Timer | Coordinator | Other SDK task | Sum | Signed residual |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| No-work | 271761 | 2141 | 1915 | 1765 | 1865 | 264172 | 271858 | -97 |
| Delayed | 29667 | 278 | 189 | 184 | 1690 | 27329 | 29670 | -3 |
| Work / 2000 | 271664 | 2205 | 37068 | 1764 | 1670 | 228957 | 271664 | 0 |
| Work / 1000 | 271671 | 4322 | 73770 | 3514 | 1679 | 188373 | 271658 | 13 |
| Work / 500 | 271664 | 8579 | 147518 | 7014 | 1671 | 106882 | 271664 | 0 |
| Work / 250 | 273863 | 15834 | 222456 | 14014 | 1685 | 19873 | 273862 | 1 |

These are whole generation+drain+boundary intervals, including the lead before
the epoch, not exact 250ms offer-window accounting. Scheduling/interrupt work
may be charged to task intervals; stored current-task counters omit the open
running interval. The signed residual is total-clock delta minus task-delta
sum and is retained without clamping. There is no deliberate 10ms blind-spot
injection in this experiment. A zero residual does not establish exclusive
CPU attribution. No division by the offer window or physical CPU percentage
is made.

## Observer and memory indicators

The [observer review](../../firmware/esp-idf/docs/load-observer.md) explains the
brackets and applicability. Gross scan durations are preserved without
subtraction; before/after scans occur outside each offered/service window.

| Disabled phase | Task enumeration before / after us | Heap-info before / after us | Empty bracket before / after us |
| --- | --- | --- | --- |
| No-work | 1508 / 1504 | 50 / 37 | 3 / 2 |
| Delayed | 1508 / 1503 | 39 / 38 | 1 / 1 |
| Work / 2000 | 1498 / 1500 | 40 / 38 | 2 / 1 |
| Work / 1000 | 1499 / 1510 | 39 / 38 | 1 / 2 |
| Work / 500 | 1498 / 1500 | 39 / 38 | 2 / 1 |
| Work / 250 | 1506 / 1502 | 39 / 38 | 1 / 1 |

| Enabled phase | Task enumeration before / after us | Heap-info before / after us | Empty bracket before / after us |
| --- | --- | --- | --- |
| No-work | 1505 / 1500 | 50 / 38 | 3 / 1 |
| Delayed | 1507 / 1499 | 37 / 38 | 2 / 1 |
| Work / 2000 | 1496 / 1496 | 38 / 37 | 1 / 1 |
| Work / 1000 | 1497 / 1506 | 37 / 37 | 2 / 2 |
| Work / 500 | 1497 / 1496 | 37 / 37 | 1 / 1 |
| Work / 250 | 1504 / 1499 | 37 / 37 | 2 / 2 |

Every disabled snapshot reports MALLOC_CAP_8BIT free **353,220 bytes**,
allocated **119,984**, largest block **327,680**, and lifetime minimum-free
**352,276**. Every enabled snapshot reports free **352,984**, allocated
**120,044**, largest block **327,680**, and lifetime minimum-free **352,032**.
Each profile's settled before/after free and allocated drift is zero. This does
not observe transient allocations, fragmentation under pressure or a global
heap peak. The two separate profiles are not used to estimate memory overhead.

| Disabled role | Declared stack bytes | Minimum observed fill-free bytes | Reservation minus fill-free bytes |
| --- | ---: | ---: | ---: |
| Coordinator | 32768 | 27852 | 4916 |
| Producer | 8192 | 7016 | 1176 |
| Consumer | 8192 | 7564 | 628 |
| Timer task | Not declared by this harness | 3856 | Not calculated |

| Enabled role | Declared stack bytes | Minimum observed fill-free bytes | Reservation minus fill-free bytes |
| --- | ---: | ---: | ---: |
| Coordinator | 32768 | 27776 | 4992 |
| Producer | 8192 | 7012 | 1180 |
| Consumer | 8192 | 7560 | 632 |
| Timer task | Not declared by this harness | 3872 | Not calculated |

These are sampled stack-fill indicators while actors remain alive, before
joins/output/deferred timer deletion. They are not minimum stack pointers,
interrupt-stack measurements, future worst-case bounds or evidence to resize
stacks safely. Linked frame/layout excerpts are retained in each profile's
artifact directory.

Each image explicitly stores **48,384 bytes** of per-job record payload and
**8,928 bytes** of snapshot payload: **57,312 bytes** combined. That excludes
allocator metadata, task stacks/TCBs, queue and timer/context storage, native
status scratch arrays, call frames and transport output. These reservations
are harness costs; the heap/stack observations do not isolate application
headroom after removing every harness allocation.

## Replay correction and device restoration

Final whole-archive QA found a host JSON reader defect after the two passing
captures. The disabled period250 conditional send-entry-to-completion mean is
`2132498 / 773 = 2758.7296248382922`. The original writer and standalone raw
replay agree byte-for-byte, but default serde_json parsing rounded that saved
decimal one ULP lower and caused strict archive equality to reject. The
[separate correction review](../../tools/benchctl/docs/api.md)
records failing-then-passing exact regressions and the pinned `float_roundtrip`
feature correction. Original capture reports/replays/raw inputs and the frozen
collector remain unchanged; the corrected reader has a separate subsequent
identity. No tolerance or rewritten evidence is used.

The corrected host revision is `87c9343140de649aaf5dd205275763d31dc7dfc9`;
its executable and the original collector have separate identities in the
[correction record](reader-correction/correction.json). The retained
[corrected-reader replay](reader-correction/corrected-reader-replay.json)
reproduces all 66 archived protocol transcripts and one existing study summary,
including both original load reports and byte-identical saved load replays.
All 69 original frozen load inputs still match their manifest. The integration
review records 98 passing workspace unit tests, two doctests and the relevant
formatting, Clippy, documentation and release checks. These host checks create
no additional physical observation.

After both attempts, the full **8,388,608-byte** pre-load flash image was
[restored](artifacts/restore.log) and separately
[verified](artifacts/restore-verify.log), followed by an RTS reset; completion
was observed at **2026-10-23 15:29:34 UTC**. The flash image remains in the
private backup path rather than this result archive. This laboratory restoration
does not qualify deployed recovery or interrupted updates.

## Decision boundary

The useful advance is a finite nominal-arrival method with retained rejected
demand, conditional latency, queue occupancy, whole-phase counter attribution
and sampled memory together. Unlike the earlier request/reply probe, this
experiment retains nominal demand independently of each completion. Each
profile reaches an adverse work pulse without erasing rejected offers or work
completed outside the offer window from the record.

The fixed-order, one-boot-per-profile, one-board pilots cannot estimate a
statistics-configuration effect, variation across boots/boards or sustained
capacity. A future effect study needs new prospectively admitted comparative
captures; these pilots cannot be promoted into it. Both images retain substantial
common measurement work, so neither supplies an uninstrumented baseline.
External timing accuracy, physical idle/sleep/ISR residency, global runtime
memory peaks, independently off-device offered load, mixed sustained workloads,
radios/coexistence, hardware offload, energy, deployed recovery and maintenance
effort remain unestablished. These results select neither an OS nor a custom
runtime.

## Offline rejection and numeric persistence controls

The retained [actual-data controls](reader-correction/independent/adversarial-outcomes.json)
record 26 mutations per transcript, 52 total. Both original and corrected readers
reject corrupt boot text, clocks/counters, task identity, notification/cleanup
claims, payloads, queue/release history and incomplete records.

Whole-archive replay also exposed a saved mean parsed one ULP low:
`2132498 / 773` was saved as `2758.7296248382922`, while the default JSON reader
returned `2758.729624838292`. The
[pinned reproduction](reader-correction/independent/README.md) demonstrates the
difference. The current collector enables `float_roundtrip` and tests this
fractional mean plus one-ULP altered-report rejection. No tolerance or original
observation change is introduced. The frozen collector retains its historical
reader limitation. Synthetic integer-valued means had missed this case.
