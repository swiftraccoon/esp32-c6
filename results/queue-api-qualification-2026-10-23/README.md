# Raw-timer and queue API qualification

Both physical qualification boots passed on 2026-10-23: one Rust std-channel
image and one native FreeRTOS queue image. Each produced **1,000 observer and
1,000 validated request/reply samples**. These two pilots are excluded from the
[frozen characterization study](../idf-queue-api-2026-10-23/README.md).

The collector accepted all counter/configuration, backend correctness,
phase/blocked-state, checksum, count and cleanup checks. An independent agent
also inspected both raw transcripts without invoking the collector. Before and
after snapshots match at CPU 160 MHz / flash 80 MHz; SYSTIMER CONF is
`0xf7000002` on both. There is no external oscillator or GPIO timing calibration.

| Qualification evidence | std | Native queue |
| --- | ---: | ---: |
| Raw ticks across blocked sleep | 449101 | 455608 |
| Raw ticks across SDK WFI calls | 479602 | 479583 |
| FreeRTOS tick progress across WFI | 3 | 3 |
| Explicit SDK WFI calls | 3 | 3 |
| Counted trial readiness/blocked checks | 1008 | 1008 |
| Native queue-empty checks | Not exposed by std | 1008 |
| Backwards timer observations | 0 | 0 |

FreeRTOS ticks use another SYSTIMER counter/IRQ path, sharing the crystal.
The WFI check establishes the executed SDK path and onboard timing consistency,
not independent electrical traceability. All ordinary failure and timeout
limitations from the [experiment](../../docs/experiments/native-queue-comparison.md)
and [integration review](../../docs/reviews/queue-experiment-integration.md) apply.

Each [run directory](runs/) preserves raw bytes, parsed records, complete
report and successful offline replay. Collection-time ELF identities remain in
the reports; compiled artifacts are outside this checkout. The device-generated
nonces differ.
The two capture commands did not supply prior-nonce history; cross-run
uniqueness was verified separately. No claim of statistical independence is
made from those identifiers.

The original full 8 MiB image was verified before these writes, restored after
both pilots, separately digest-verified and reset. Transfer logs are under
[artifacts](artifacts/); the full-flash backup remains outside this dataset.
The original backup SHA-256 is
`13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`.
No eFuses were provisioned. `SHA256SUMS` records collection-time input and capture
identities. See the [snapshot review](../../docs/reviews/public-snapshot.md) for
available artifacts and current verification commands.
