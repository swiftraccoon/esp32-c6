# RTIC monotonic qualification v1

Prospective contract, **2026-10-23**, for unmodified RTIC commit
`f1797184d6168e54c345b17019c237eea70b9ed4`,
`examples/esp32c6/examples/monotonic.rs`. This qualifies one Rust execution
route on the attached C6. It supplies no matched performance or OS ranking.
The [source research](../research/rtic-qualification.md),
[build record](../builds/rtic.md), [prior work](../research/prior-work.md) and
[reuse inventory](../research/benchmark-reuse.md) precede collection.

## Purpose and fixture

Reuse the upstream MIT/Apache-2.0 example without source edits, rather than
author another timer demo. It exercises RTIC cooperative priority-zero futures
and the C6 SYSTIMER monotonic interrupt. There is no positive-priority task
preemption test, application GPIO stimulus, radio or shared compute workload.
The separate `sw_and_hw` example is build/source evidence only: its GPIO9
button path needs an identified carrier/stimulus before physical qualification.

Use the existing CP2102N UART0 route, GPIO16/17, at115200/8N1. The example's
esp-println `auto` backend can choose native USB-Serial/JTAG; require no active
native USB connection and retain the actual host port inventory. No runtime
CPU/flash clock header is required: the example has none. It does not call HAL
init, and this probe does not add register readback or establish clock accuracy.
Preserve the bootloader, partition table, app image, exact ELF, upstream lockfile,
compiler/build options and resolved dependencies before flashing.

The selected default C6 bootloader comes from espflash4.6.0. Preserve its
original resource and original packaged outputs separately from any corrected
image headers. This older HAL's split flash mappings are rejected by both
inspected esptool conversion modes; installed espflash instead needs a narrow,
reviewed C6 frequency/app-descriptor metadata correction. Record its Rust helper,
exact byte changes, checksum/digest validation and independent image inspection
before the input freeze. This packaging change must not alter upstream firmware
source or ELF instructions. Bootloader watchdog inheritance is an explicit
source risk because the monotonic example omits HAL initialization; retain any
actual reset rather than shortening the guard or assuming watchdog survival.

## Source-derived output contract

The application emits exactly these seven newline-terminated, plain UTF-8 lines:

```text
init
hello from foo
hello from bar
hello from baz
bye from foo
bye from bar
bye from baz
```

`init` is first. Accept all six permutations of the three hello lines; require
all three before the first bye. Require byes in foo/bar/baz order, each once.
Reject missing/duplicate/reordered markers, unexpected application lines or
any nonblank line after the last marker. Blank whitespace remains harmless.
Startup before init permits only the recognized C6 ROM/build/reset/load/entry
metadata and informational IDF bootloader lines. JSON is not this wire format.
Require one recognized C6 ROM startup before the first application marker.

The pinned source waits **2/3/4 seconds** from each task's own first poll;
the README's1/2/3s statement is stale. Record host-monotonic line-receipt times,
and use these deliberately broad qualification budgets:

| Task | Own hello to own bye, host receipt interval |
| --- | --- |
| foo | 1.5–3.5 seconds inclusive |
| bar | 2.5–4.5 seconds inclusive |
| baz | 3.5–5.5 seconds inclusive |

These challenge an immediate/wrong-scale delay. They are not timer latency,
oscillator calibration or device execution-time measurements. UART buffering,
USB transport and host scheduling affect receipt times. Coalesced/out-of-budget
output fails this qualification without proving a device timer defect. Never
widen the frozen budgets to admit an observed failure.

## Complete observation and failure retention

Before the input freeze, watchdog source review increased the initial proposed
five-second guard to **twelve seconds**. A final marker near four seconds plus
only five seconds would end near the bootloader's nominal nine-second watchdog
boundary. The longer guard covers that risk with margin; it still cannot prove
watchdogs disabled or long-term stability. No physical attempt preceded this
design change. Retained early preflight counterexamples keep their original
five-second setup and are labeled separately from the final contract.

The collector measures a fixed **45-second ceiling** from the host RTS-release
request. File preparation, serial-port open and the preceding100ms reset hold
are outside that ceiling. It observes for at least **twelve seconds after the
newline completing `bye from baz`**, through the same fatal/extra-boot guard.
Successful marker receipt alone does not complete capture. EOF, read error,
deadline expiry, a partial nonblank line or fatal/reboot evidence fails, including
during the post-marker interval. Quiet UART does not prove continued liveness.

Retain `serial.raw` and a separate newline-delimited receipt of each read/poll
with host-monotonic nanoseconds and contiguous raw-byte offsets. A terminal event
records its time and reason. Validation reconstructs marker times from these
receipts, requires the full twelve-second interval and exact raw coverage, and
binds raw/receipt hashes in the saved report. Replaying raw alone cannot certify
elapsed silence. These records are inspectable host observations, not an
independent timing instrument or authentication of the physical board.

Success requires a completed serial timeout poll at or after the twelve-second
deadline. Read and validate any pending bytes until that poll; a wall-clock
jump or interrupted read cannot substitute for observing the port. Bind the
reported observation endpoint to the qualifying poll, excluding later host
scheduling or file-output delay. The final poll and its terminal event may
share that endpoint timestamp; other event times must strictly increase.

Known fatal text includes panic/reboot/watchdog/assert diagnostics and this
pinned backtrace crate's `Exception '` and `Stack overflow detected` formats.
Rejection is permanent for an attempt, even if a complete good sequence follows.
Preserve the full received chunk before admission and retain failed reports
without qualification summaries or numeric scores. No resynchronization/retry
turns a failed boot into success. Disk/process failure may leave incomplete
packaging; it cannot count as a passing attempt.

## Schedule, controls and archive

Freeze **three** warm-reset attempts of the same image, ordered1/2/3 with distinct
operator boot IDs. Stop after the first unexpected rejection, retain it, and
restore the full pre-session flash. A corrected method/image needs a separately
versioned contract and new attempts. Do not manufacture independent trials from
timestamps or host IDs; these are repeated observations on one board.

Before collection, build the exact image and Rust collector, check required
output in the actual ELF, run complete positives and contrary controls through
capture/replay/archive paths, retain [independent admission review](../reviews/rtic-admission.md),
commit the input hashes/schedule and bind that commit in the evidence ledger.
Controls include every hello permutation, malformed/overlong/split text,
wrong marker order, fatal/extra-ROM evidence before and after completion,
early EOF/read error, shortened/nonmonotonic/offset-corrupted receipts,
out-of-budget pacing and failed-report/tampered-summary non-promotion.

Retain raw text, timing receipts, original supplied provenance, ELF, capture
report, deterministic replay output, flash logs and separately verified full
restoration. Raw console evidence remains distinct from JSON benchmark protocols.
Preserve all frozen inputs. Finalize the complete checksum manifest and ledger
only after read-only archive verification/replay and independent result review.
No successful build, host fixture or console pass qualifies a full RTIC subsystem,
external GPIO IRQ response, preemption, CPU efficiency, memory peaks or energy.
