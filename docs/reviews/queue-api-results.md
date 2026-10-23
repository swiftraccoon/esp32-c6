# Independent review of the matched queue API results

Reviewed **2026-10-23**, after all 24 scheduled captures and restoration.
Scope: [the frozen study](../../results/idf-queue-api-2026-10-23/README.md),
every raw transcript/capture report, the schedule and input manifest,
[attempt ledger](../../results/idf-queue-api-2026-10-23/attempt-ledger.json),
linked-image identities and the
[Rust analyzer output](../../results/idf-queue-api-2026-10-23/summary.json).
The reviewer is separate from the firmware, collector and analysis authors.
No serial operation, build, new capture, retry or measurement-file edit was
performed by this reviewer.

**All 24 scheduled boots pass the independent data/provenance audit.** There
are 24,000 checked request/reply observations and 24,000 observer observations,
with twelve complete paired blocks. Independent JSON parsing and arithmetic
from `serial.raw` reproduce every published per-boot summary and the prescribed
paired analysis. The two preliminary qualification captures remain excluded.

## Freeze, capture and admission evidence

Commit `3fc94676ddeb240650738e479b068cc9ed807804`, recorded at
05:31:07 UTC, precedes the first capture timestamp, 05:32:49.026 UTC. The last
capture starts at 05:52:33.937 UTC. The retained
[input hashes](../../results/idf-queue-api-2026-10-23/INPUTS.sha256),
[manifest](../../results/idf-queue-api-2026-10-23/study-manifest.json) and
[seeded schedule](../../results/idf-queue-api-2026-10-23/schedule.json) match
their frozen versions and the measured artifacts. No firmware, collector,
analysis-source, preregistration or input-hash change occurred during collection.

The effective SDK configuration's bytes were pinned by its committed input
digest before collection, although the generated file itself was initially
ignored. The retained local file matches that frozen digest; the unchanged file
is explicitly included in the final archive. This distinction
does not make its identity a post-measurement choice.

Each raw transcript contains one ROM startup, one hello, 1,000 observer samples,
1,000 message samples, one qualification and one completion. Parsed records
match the raw JSON records exactly. The audit independently checks:

- Ordered indices, raw timestamp endpoints, widths, checksums and responder
  phase values; no missing, duplicate, backwards or excess observation.
- Observed priorities 1/5, 100 Hz kernel tick, capacity-one four-byte messages,
  eight warmups, 48,000-byte sample storage and the requested stack reservations.
- Per-boot nonce consistency, 24 distinct observed nonces, distinct boot labels,
  complete prior-study nonce history and increasing scheduled capture times.
- Frozen backend ELF, merged-image, provenance and collector hashes; each
  capture's ELF and embedded provenance match its selected frozen backend.
- Per-boot before/after configuration equality and the same configuration
  across all 24 boots. Decoded CPU/MSPI/flash settings are 160/80/80 MHz,
  divider one. SYSTIMER uses XTAL, with the required enables and SDK stall
  policy; its retained CONF value is `0xf7000002` throughout.
- Qualification totals, empty/full probes, worker join and backend destruction.
  Main-thread readiness is checked outside timing; the higher-priority worker's
  next-phase bookkeeping remains part of the measured path.

The [ledger](../../results/idf-queue-api-2026-10-23/attempt-ledger.json) has
exactly 24 attempted/pass entries in the frozen order, with no application
failure, fixture invalidation or repeat. Each of three consecutive collection
groups contains two std/native and two native/std blocks. Group labels and
fresh nonce observations do not establish statistical independence.

## Independently derived values

The following are **per-boot values**: each of the twelve boots for a backend
has these same summary statistics. They are not pooled independent samples.
Units remain gross raw ticks; nominal conversion is one tick = 0.0625 µs.

| Backend / benchmark | Count per boot | Min | p50 | p95 | p99 | Largest observed | Mean |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Std channel / request-reply | 1,000 | 1,005 | 1,006 | 1,006 | 1,006 | 1,280 | 1,006.54 |
| Native queue / request-reply | 1,000 | 318 | 319 | 319 | 319 | 426 | 318.964 |
| Std image / observer | 1,000 | 14 | 15 | 15 | 15 | 36 | 14.821 |
| Native image / observer | 1,000 | 14 | 15 | 15 | 15 | 16 | 14.801 |

Percentiles use nearest rank: sort all 1,000 durations and select one-indexed
`ceil(p * 1000)`. No observer value is subtracted. The corresponding request/reply
boot-p50s are **62.875 µs** and **19.9375 µs**, at the nominal counter scale.
The observed maxima are 80 µs and 26.625 µs; these are sample maxima, **not WCET
bounds** or a proof about unobserved tails.

The twelve block contrasts, in block order, are:

```text
native minus std boot-p50, ticks:
[-687, -687, -687, -687, -687, -687, -687, -687, -687, -687, -687, -687]
```

Their median is **−687 ticks / −42.9375 µs**. Each session's four-block
median and both six-block order-group medians are also −687 ticks. The
third- and tenth-smallest contrasts both equal −687, reproducing the analyzer's
conditional interval **[−687, −687] ticks**. Its stated continuous IID coverage
is `1 - 2 * (1 + 12 + 66) / 4096 = 0.96142578125`.

This zero-width interval reflects identical **observed, quantized boot medians**.
There are still varying event durations. Fingerprinting each boot's ordered
1,000 request/reply durations finds two distinct duration vectors per backend;
fingerprinting the absolute start/end pairs finds three timestamp vectors per
backend. Separate startup transcripts and distinct nonces support the capture
history, while these repetitions limit evidence about independent environmental
variation. The interval supplies no guarantee of zero calibration error,
placement sensitivity, future variation or independence. Its conditional model
is separate from the descriptive result that all twelve observed contrasts agree.

## Timing qualification and limits

All 24 qualifications report 1,000 ordered initial reads with no backstep,
then a requested 20 ms blocked sleep and explicit SDK WFI calls. The retained
raw/legacy blocked spans are:

| Image | Raw blocked ticks | Outer legacy bracket, converted to ticks | WFI ticks | WFI calls / elapsed RTOS ticks |
| --- | --- | --- | --- | --- |
| Std | 449,101 | 449,184 | 479,602–479,603 | 3 / 3 |
| Native | 455,608 | 455,728 | 479,583 | 3 / 3 |

Their ordering and declared admission bounds hold in every boot. No debugger
is reported. Every boot has 1,008 successful trial readiness/blocked checks;
native has 1,008 empty-queue checks and std reports zero occupancy queries.
Std emptiness is inferred from its reviewed single-inflight control path.

Legacy time shares SYSTIMER UNIT0. The FreeRTOS tick uses UNIT1 in the same
peripheral and shares XTAL, so this is onboard continuity/consistency evidence,
not independent frequency calibration. Equal endpoint register snapshots cannot
exclude a transient change between them. The review does not close long-duration,
radio/sleep timer reliability or arbitrary erratum-triggering memory-copy cases.

The result characterizes two ordinary release images and complete instrumented
Rust API paths on one C6, with the same IDF kernel and worker arrangement.
Different implementations and code/data placement remain part of that treatment.
Pre-interval state/occupancy probes also condition the execution; the native
occupancy query has no equivalent public std query. This is sufficient to show
an observed same-kernel API-choice difference, while further attribution needs
its own controlled experiment. Historical Embassy numbers are not pooled or
ranked against this new contract. No kernel-switch cost, overall OS winner,
hardware-population estimate or custom-kernel requirement follows from it.

## Final device state

The [device-state record](../../results/idf-queue-api-2026-10-23/device-state.json)
and retained [write](../../results/idf-queue-api-2026-10-23/artifacts/study-restore.log)
and [separate verification](../../results/idf-queue-api-2026-10-23/artifacts/study-restore-verify.log)
logs confirm restoration of all 8,388,608 bytes, successful device digest
comparison and a subsequent RTS hard reset at the end of collection.
The ignored original backup independently retains SHA-256
`13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`.
The root operator performed these board operations; this reviewer inspected
their recorded evidence. Final firmware is the original application, with no
eFuse provisioning recorded.
