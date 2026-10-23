# RTIC monotonic qualification

Collected **2026-10-23** under the
[prospective contract](../../docs/experiments/rtic-qualification.md).
**The first attempt failed; zero qualified.** All seven expected application
markers appeared, followed by a second ROM startup during observation.
Attempts two and three were not run under the stop rule. No replacement or
acceptance change occurred. [RESULTS.md](RESULTS.md), [analysis.json](analysis.json)
and the [attempt ledger](attempt-ledger.json) retain the exact scope.

The [schedule](schedule.json) fixes three warm-reset attempts of the unmodified
upstream monotonic example and stops after the first unexpected rejection.
The final method observes twelve seconds after the final expected marker,
using retained raw UART bytes and host read/poll receipts. Early five-second
host-only counterexamples are historical preflight evidence; no physical
attempt used that initial proposal.

The second startup's cause is unconfirmed; its reset-reason line was not
retained. Startup watchdog inheritance is a source lead. The
[follow-up](../../docs/research/rtic-startup-followup.md) separates the next
diagnostic from any firmware/bootloader remedy. The independent
[result review](../../docs/reviews/rtic-results.md) checks the actual failure
and prevents its successful substeps from becoming qualification.

[provenance.json](provenance.json) identifies the original ELF and separately
corrected image metadata, bootloader, partition table and collector. The
[build record](../../docs/builds/rtic.md),
[source review](../../docs/reviews/rtic-source.md) and
[host review](../../docs/reviews/rtic-host.md) retain the actual source/clock,
transport, watchdog and observation limits. Original tool outputs, failed
conversions and the narrow Rust packaging helper remain alongside final images.

The [121-entry input manifest](INPUTS.sha256) is frozen at Git commit
`33975191f16cec1c28e5f1f87c9255fefdfd5c57`, before flashing. The full original
flash was restored, separately verified and reset; see the
[device receipt](device-state.json). [SHA256SUMS](SHA256SUMS) seals all retained
regular files except itself; that final seal does not expand the prospective
input freeze. There is no comparative estimator, calibrated device timing,
GPIO/preemption qualification, CPU efficiency, memory peak or OS-selection
result from this collection.
