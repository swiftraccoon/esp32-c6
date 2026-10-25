# RTIC monotonic qualification v1 results

**The first attempt failed; no attempt qualified.** All seven expected
application markers arrived, followed by a second C6 ROM startup during the
required observation window. The collector permanently rejected the attempt.
The [frozen stop rule](schedule.json) then left attempts 02/03 **not run**.
There was no replacement attempt or change to the image/acceptance contract.

| Scheduled | Executed | Failed | Qualified | Not run under stop rule |
| ---: | ---: | ---: | ---: | ---: |
| 3 | 1 | 1 | 0 | 2 |

This is a failed qualification of the selected unmodified RTIC monotonic
example and startup configuration. It does not establish universal C6/RTIC
incompatibility. The separate software/hardware example remains build/source
evidence only.

## Fixed inputs and retained failure

The [contract](../../docs/experiments/rtic-qualification.md),
[source research](../../docs/research/rtic-qualification.md),
[build record](../../docs/builds/rtic.md) and independent source/admission reviews
preceded collection. Source/contract commit:
`8008dbf0283403980fa237981a82fec63d3f391a`; the 121-entry
[input freeze](INPUTS.sha256): `33975191f16cec1c28e5f1f87c9255fefdfd5c57`;
ledger binding: `096f8a6c754be101b62eabdb269dfa548978aa1b`.
All 121 inputs still match their frozen hashes.

[Original provenance](provenance.json) binds Rust 1.96.0,
`riscv32imac-unknown-none-elf`, clean upstream
`f1797184d6168e54c345b17019c237eea70b9ed4`, unchanged tracked lock and the
actual bootloader/partition table. The upstream firmware source and ELF are
unmodified. The retained Rust helper separately corrected image metadata,
checksum and digest; original tool outputs and failed conversion attempts
remain archived. The collector was built before its matching source commit;
no clean-at-build claim is made for that executable.

| Actual artifact | SHA-256 |
| --- | --- |
| ELF | 12dc8bd4f079b9acdfda34db652e422cafddd9869c4b92493274cb56b47645a9 |
| App image | b10d6dfdc9f84f0e02f3007e198fe38ea3ae02ca47a3222e515073cfc9cbfb29 |
| Bootloader | b1d7d018cc62f3fb30b901d5bacf95f58ab879f70ebec50d231f092e976c9318 |
| Collector | f06cb6e82e1e01e86dba08a5172b5e4cc0b3ebd588f3cd6fd2395cbfbdbc84dd |

The [raw UART](runs/01/serial.raw) has **1,495 bytes**; its
[receive receipts](runs/01/observations.jsonl) contain 105 events: 27 reads,
76 completed timeout polls, start and rejected end. Direct inspection confirms
strictly increasing host times, complete monotonic raw-byte coverage and the
final rejected receipt. [analysis.json](analysis.json) recalculates the timings
directly from raw newlines and the first covering read receipt. It also checks
all retained hashes and report nulls; these are diagnostics on a failed attempt.

The [original report](runs/01/run.json) retains
`status=failed`, reason `additional ROM startup in the same attempt`, with
**summary, console, qualification, benchmarks and comparative_score all null**.
The [standalone raw/receipt replay](runs/01/replay-error.log) also rejects.
Whole-archive replay can succeed by verifying that failed status and its retained
inputs; it cannot turn this attempt into a passing console qualification.

## What the receipts show

Times below are host-monotonic receipt seconds from the collector's RTS-release
request, rounded to six decimals. They are not independently captured device
execution times or a calibrated reset edge. Several initial lines share one
read receipt; their equal times do not mean simultaneous device execution.

| Application marker, actual order | Host receipt s |
| --- | ---: |
| init | 0.134495 |
| hello from bar | 0.134495 |
| hello from baz | 0.134495 |
| hello from foo | 0.134495 |
| bye from foo | 2.135456 |
| bye from bar | 3.132863 |
| bye from baz | 4.134166 |

The permitted hello permutation and bye order were observed. The source's
2/3/4-second waits also meet their coarse receive budgets:

| Task | Own hello→bye host interval s | Frozen budget s | Diagnostic check |
| --- | ---: | --- | --- |
| foo | 2.000961 | 1.5–3.5 | Within budget |
| bar | 2.998368 | 2.5–4.5 | Within budget |
| baz | 3.999671 | 3.5–5.5 | Within budget |

Those successful subsets do not qualify the attempt. The second ROM banner
arrived at **8.119346s**, **3.985180s after the final marker**. Its received
chunk ends with a single partial `B` after the banner. No second reset-reason
line was retained. The terminal rejected receipt is at8.119516s.

The last completed poll was at8.102499s, only **3.968333s after the final
marker**. The frozen12-second guard would require observation through at least
16.134166s and was not completed. The45-second ceiling was not the cause of
rejection. The extra startup arrived within even the historical five-second
proposal; this observation does not attribute detection uniquely to the longer
guard.

## Reset cause and decision boundary

The second ROM startup is the observed failure. Its **cause remains
unconfirmed**: the capture contains no second reset code, panic, backtrace or
watchdog diagnostic identifying that cause. Startup-watchdog inheritance is a
credible source-supported lead because this example omits HAL init and uses a
prebuilt IDF bootloader. Timing alone does not prove the lead. The first boot
reports `ESP-IDF v6.1-beta1-497-g14f663f003e`; that banner supplements the
manifest's release/v6.1 source lead without supplying a complete reproducible
bootloader build configuration.

The [source follow-up](../../docs/research/rtic-startup-followup.md) recommends
a separately frozen diagnostic capture that keeps its first rejection while
retaining more of the second startup. That could supply the missing reset
reason before changing firmware. It also evaluates narrow maintained Rust
watchdog handling, broader HAL initialization and explicit bootloader policy,
including timer/PAC/interrupt ownership. No remedy was built or tested in this
cohort. A new configuration needs a versioned qualification and new frozen
inputs; v1 stays failed.

There is no runtime CPU/flash register readback, external clock calibration,
GPIO stimulus/preemption test, radio workload, power/energy measurement, CPU
efficiency or runtime-memory-peak result. Bootloader flash-frequency text and
image headers do not establish those application properties. The result narrows
one candidate's startup feasibility; it selects neither an OS nor a custom
runtime.

## Restoration

The full **8,388,608-byte** pre-session flash image was
[restored](artifacts/restore.log), separately
[digest-verified](artifacts/restore-verify.log) and followed by an RTS hard reset.
Completion was observed at **2026-10-23 16:59:16 UTC**, not independently timed
at the reset edge. The [device receipt](device-state.json) records the private
backup identity and limits; the firmware image itself remains private. This
laboratory restoration does not establish deployed recovery or post-boot
persistent-state constancy. The [attempt ledger](attempt-ledger.json) preserves
the one failed attempt and two stopped attempts.

## Offline false-pass controls

Five checks using copies of the actual capture preserve the failed outcome:
the unchanged graph remains failed; invented passing result objects still
reject at the second ROM startup; a wrong experiment tag and a missing required
null field reject; a synthetic favorable prefix/guard cannot promote the saved
failed status. These are host controls, not additional board attempts. Complete
markers or plausible pacing do not override the second startup or missing reset
attribution. Maintained controls live in
[rtic_console.rs](../../tools/benchctl/src/rtic_console.rs).
