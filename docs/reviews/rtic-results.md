# Independent RTIC qualification result review

Reviewed **2026-10-23**, after the first physical attempt and full restoration.
This agent performed **no board operations**, source/frozen-input edits or
commits. The [prospective contract](../experiments/rtic-qualification.md),
[raw capture](../../results/rtic-qualification-2026-10-23/runs/01/serial.raw),
[host receipts](../../results/rtic-qualification-2026-10-23/runs/01/observations.jsonl),
original report/provenance and [results analysis](../../results/rtic-qualification-2026-10-23/analysis.json)
were checked independently.

**One attempt executed and failed; neither of the other two scheduled attempts
ran.** All seven expected application lines arrived before another C6 ROM
startup. The frozen stop rule was followed, with no replacement. There is no
qualified RTIC capture, overall RTIC incompatibility result or OS ranking.

## Frozen artifacts and source agreement

The source/contract revision is
`8008dbf0283403980fa237981a82fec63d3f391a`, the exact input freeze is
`33975191f16cec1c28e5f1f87c9255fefdfd5c57`, and its pre-flash ledger binding is
`096f8a6c754be101b62eabdb269dfa548978aa1b`. The manifest's bytes match that
frozen Git revision and all **121** listed input hashes still match. Independently,
**22** retained source/configuration/contract/review snapshots match their
corresponding files in the source revision.

The attempt's ELF and original provenance bytes equal the frozen supplied
copies. ELF, application image, bootloader, partition table and collector hashes
agree with the provenance and manifest. The collector is the original frozen
executable, SHA-256
`f06cb6e82e1e01e86dba08a5172b5e4cc0b3ebd588f3cd6fd2395cbfbdbc84dd`.
Its pre-commit build is recorded without claiming a clean-at-build source tree.
The separate image-metadata corrections are not represented as upstream
firmware-source changes.

The retained raw SHA-256 is
`3e90c4a6f6d3395218816e8950c8b26ad1ed2fa9ca599cc2ac1e5008fdc2cadb`;
the receipt SHA-256 is
`fbab37da5bea25ebc6fcd5199c15c3348c831cdaf7f3f7d700d966860d11bb6a`.
The [flash log](../../results/rtic-qualification-2026-10-23/artifacts/flash.log)
records verified transfers of 22,016 bootloader bytes, 3,072 partition-table
bytes and 18,000 application bytes at their declared offsets. File hashes and
transfer logs do not independently authenticate a physical execution.

## What was actually retained

Direct byte/receipt inspection finds **1,495 raw bytes** and **105 receipt
events**: one start, 27 reads, 76 timeout polls and one rejected end. Times
strictly increase; reads advance contiguous cumulative offsets; polls/end
leave offsets unchanged; every retained byte is covered. Marker times were
recalculated from the first read receipt covering each newline, rather than
from the failed report's null summary fields.

The observed entry permutation is `bar, baz, foo`; exits are `foo, bar, baz`.
`init` and the three entry lines share the receipt at **134,494,875 ns**.
Own-entry-to-exit host intervals are **2,000,961,167 ns**, **2,998,367,792 ns**
and **3,999,670,875 ns** respectively. They satisfy the coarse receipt budgets,
but remain diagnostic subsets of a failed attempt. Equal receipt times are
not simultaneous device execution; these intervals are not timer accuracy or
interrupt-latency results.

| Event | Host receipt ns from RTS-release request |
| --- | ---: |
| Final expected `bye from baz` newline | 4,134,165,750 |
| Last completed timeout poll | 8,102,498,875 |
| Second C6 ROM startup newline | 8,119,346,209 |
| Rejected terminal receipt | 8,119,515,500 |
| Earliest endpoint meeting the required 12-second guard | 16,134,165,750 |

Only **3,968,333,125 ns** of post-marker observation reached a completed poll.
The second startup arrived **3,985,180,459 ns** after the final marker, so the
full guard was not completed. This event also falls within the original
five-second proposal: the longer prospective guard was not uniquely necessary
to detect this particular restart. The 45-second ceiling did not cause rejection.

The final received chunk contains the complete second ROM line and a single
partial `B`. Both are preserved even though admission rejects at the ROM line.
There are two ROM startups but only one reset-reason line, the initial
`rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)`. There is **no retained second
reset reason**, panic, backtrace or watchdog diagnostic identifying the cause.
The reset request metadata does not independently establish the EN waveform or
a physical reset edge.

## Failure retention and contrary checks

The [original report](../../results/rtic-qualification-2026-10-23/runs/01/run.json)
retains `status=failed`, reason `additional ROM startup in the same attempt`,
and explicit null `qualification`, `console`, `summary`, `benchmarks` and
`comparative_score` fields. Attempts 02/03 have no capture directories and the
[attempt ledger](../../results/rtic-qualification-2026-10-23/attempt-ledger.json)
records them as not run under the stop rule, rather than failures or successes.

The frozen executable's standalone `validate-rtic` rejects these actual raw and
receipt files with the same additional-startup error. Its whole-archive replay
succeeds **while preserving failure**: 66 existing JSON protocol transcripts,
zero successful console captures, one retained failed capture and one existing
study summary. Replay consistency does not qualify the failed attempt.

Five additional offline checks used temporary copies of this actual capture:
the unchanged graph remains failed; changing its status to pass with favorable
objects still rejects at the second ROM; retagging its experiment rejects;
omitting the required failed-console null rejects; and supplying a deliberately
synthetic favorable prefix/guard cannot promote its original failed status.
The last case is a counterfeit host-only control, never an observation or a
replacement capture. Original raw, receipts, report, ELF and provenance hashes
were compared before/after and remained unchanged. No actual evidence was edited.

The [results report](../../results/rtic-qualification-2026-10-23/RESULTS.md)
and analysis agree with the independent arithmetic: all **15 table
rows**, artifact hashes, receipt/marker offsets, task intervals, failure/guard
diagnostics and report null fields match. The complete restoration receipt and
its log hashes also match their analysis entries. No numerical or interpretation
discrepancy remains in those reviewed files.

## Restoration and remaining uncertainty

The [device-state receipt](../../results/rtic-qualification-2026-10-23/device-state.json)
agrees with the full write and separate digest-verification logs. The private
backup was independently hashed from disk: **8,388,608 bytes**, SHA-256
`863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
Logs record full restoration, separate matching-digest verification and an RTS
hard reset. Completion was operator-observed at **2026-10-23 16:59:16 UTC**;
this is not externally timed reset-edge evidence. No fresh full-device read,
post-boot NVS constancy or deployed recovery qualification is inferred.

Startup-watchdog inheritance is a source-supported investigation lead, not a
confirmed reset cause. The bootloader's reported version string does not supply
its complete reproducible build configuration. A future diagnostic or remedy
needs a separately versioned contract and fresh inputs; changing watchdog/startup
policy alone would not establish why v1 reset. This failed evidence remains
unchanged regardless of a later remedy's outcome.

The attempt qualifies neither long-term stability nor preemption, GPIO, radios,
CPU efficiency, runtime memory peaks or power. The controls demonstrate that
this particular restart was detected and retained; they do not establish that
the experiment or wider platform evaluation is complete or error-free.
