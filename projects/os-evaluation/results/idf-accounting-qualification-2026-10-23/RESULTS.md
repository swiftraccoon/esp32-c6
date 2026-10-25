# IDF accounting qualification results

**All three scheduled boots qualify under v1**, with no replacement attempts.
This qualifies the selected Rust/IDF diagnostic path for further measurement
development. It does not establish physical CPU utilization, sustainable
capacity, global memory peaks, energy efficiency or an OS preference.

The [prospective contract](../../docs/experiments/idf-accounting-qualification.md)
and [three-boot schedule](schedule.json) preceded collection. Implementation and
contract commit: `c931379e2042b04cdb59c93e57441cc1182f5f19`; exact
[inputs](INPUTS.sha256) freeze: `f80cf87d316b6c02143adbe4114ee51764f14802`;
ledger binding: `e15f4f9be646461df21fb5c36b04728b2e50bab7`.
[Provenance](provenance.json) records Rust 1.96.0, IDF 5.5.6, sys 0.37.2,
64-bit esp_timer accounting and the exact images/configuration. The binary was
built before the implementation commit; its `f1fc734-dirty` app descriptor is
retained alongside matching committed source snapshots. No clean-at-build
claim is made.

The retained [raw 01](runs/01/serial.raw), [raw 02](runs/02/serial.raw) and
[raw 03](runs/03/serial.raw) contain **18 snapshots and 72 task records**.
[analysis.json](analysis.json) independently recomputes integer results directly
from each raw transcript; reported aggregates were compared only afterward.
The independently calculated phase fields match all three capture reports.
The offline rejection controls below exercise mutations of the actual raw data.
There is no comparative estimator or profiling-on/off image comparison.

## Task accounting controls

All values below are **nominal integer microseconds** reported by the onboard
API. Matching decoded CPU 160 MHz / flash 80 MHz and timer configuration before
and after the probe does not supply independent frequency calibration. The ROM
bootloader's `clock div:2` text is not the runtime flash-frequency evidence.

| Observation | Boot 01 | Boot 02 | Boot 03 |
| --- | ---: | ---: | ---: |
| Blocked phase total-clock interval | 259,857 | 259,857 | 259,857 |
| Blocked worker attributed increment | 0 | 0 | 0 |
| Blocked sampler attributed increment | 11,217 | 11,217 | 11,217 |
| Blocked idle-task attributed increment | 258,783 | 258,783 | 258,783 |
| Sum of all task increments, blocked phase | 270,000 | 270,000 | 270,000 |
| Signed boundary residual, blocked phase | −10,143 | −10,143 | −10,143 |
| Busy worker's own start/end span | 250,215 | 250,215 | 250,215 |
| Busy worker attributed increment | 250,262 | 250,262 | 250,262 |
| Busy phase total-clock interval | 251,333 | 251,333 | 251,333 |
| Busy sampler attributed increment | 1,060 | 1,060 | 1,060 |
| Busy idle-task attributed increment | 0 | 0 | 0 |
| Sum of all task increments, busy phase | 251,322 | 251,322 | 251,322 |
| Signed boundary residual, busy phase | 11 | 11 | 11 |

The busy worker reports **810 calls** of the checked 4,096-round workload per
boot. Its aggregate checksum **2,470,336,773** matches the independent wrapping
sum of `0xa3944d55 XOR i` for indices 0–809. Its stored runtime increment meets
the prospectively declared coarse 80–120 percent control around the busy span.
This is an attribution sanity check, not an exclusive CPU percentage or
clock-accuracy bound. All snapshots retain four live task identities, including
worker, sampler and idle, with the worker blocked and sampler running at each
boundary. Array order varies; identities are matched by handle and task number.

The negative blocked-phase residual is retained:
`259857 − (11217 + 258783 + 0 + 0) = −10143`. Snapshot counters omit the
current sampler's open running interval. The deliberate **10 ms blind-spot
control with scheduling suspended occurred before the first baseline snapshot**,
while its stored sampler counter still read 680/681 us. That earlier open
interval is charged when the sampler later blocks, importing pre-baseline work
into its 11,217 us delta. This explains the observed boundary mechanism from
source behavior; the noninstantaneous scans also cross these boundaries.
There is no trace assigning the exact 10,143 us to individual components. The
negative residual does not imply a backwards timestamp or represent idle time.
The positive busy residual is likewise not an idle measurement.

Two separate controls exposed the API boundaries. Declared capacity zero
returned zero with its total sentinel unchanged in all three boots. During
the scheduler-suspended **10,000 us** busy interval, the stored current-task
counter stayed **680→680**, **681→681** and **680→680 us**, respectively.
That is a direct demonstration that a current-task status read does not flush
its running interval; zero counter change alone cannot mean no work occurred.

The tied phase values do not make these independent statistical replicates or
establish a variability bound. These are three warm resets of one image on one
board. The raw transcripts have different hashes and differ in absolute
endpoints, lifetime counters and some one-microsecond observations.

## Observer and memory indicators

Gross API/empty-bracket durations are retained without correction:

| Snapshot | Task enumeration us, boots 01 / 02 / 03 | Heap-info us, each boot | Empty bracket us, boots 01 / 02 / 03 |
| --- | --- | ---: | --- |
| blocked-before | 970 / 969 / 970 | 39 | 2 / 2 / 2 |
| blocked-after | 969 / 968 / 968 | 28 | 2 / 2 / 2 |
| busy-after | 969 / 969 / 969 | 28 | 1 / 1 / 1 |
| heap-control | 969 / 968 / 969 | 28 | 2 / 2 / 2 |
| heap-held | 929 / 930 / 930 | 30 | 1 / 1 / 1 |
| heap-freed | 930 / 930 / 930 | 28 | 1 / 2 / 2 |

The approximately one-millisecond task-enumeration call includes the selected
stack-fill scans while scheduling is suspended; interrupts remain possible.
These are gross costs of this path, not an isolated switch-accounting overhead
or a universal correction. They support keeping diagnostic collection outside
a future measured service path. No historical RTT is adjusted with these values.

Each boot performs one touched and checked **16,384-byte MALLOC_CAP_8BIT (4)**
allocation, for three separate allocation controls. Its byte checksum is
**2,041,721**: 65 complete cycles of bytes 0–250 plus bytes 0–68.
Every boot records the following same heap values:

| Snapshot state | Current free bytes | Allocated bytes | Largest free block bytes | Lifetime minimum free bytes |
| --- | ---: | ---: | ---: | ---: |
| blocked-before, blocked-after, busy-after, heap-control | 421,272 | 52,736 | 401,408 | 421,272 |
| heap-held | 404,884 | 69,120 | 385,024 | 404,884 |
| heap-freed | 421,272 | 52,736 | 401,408 | 404,884 |

The no-allocation control drift is zero. Current free falls **16,388 bytes**
while reported allocated bytes rise **16,384**, then current free and largest
block return to their control values. The four-byte accounting difference is
retained; this one allocation does not establish general allocator overhead.
The lifetime minimum correctly stays low after free. A single live block and
these aggregate indicators do not establish fragmentation or an exact global
instantaneous heap peak; capability classes are not summed.

The minimum recorded worker stack-fill-free indicator is **7,248 bytes** of its
8,192-byte reservation; the sampler's is **16,972 bytes** of 32,768. Reservation
minus those recorded minima is 944 and 15,796 bytes, respectively. These are
exercised fill-pattern indicators, not measured minimum stack pointers or proof
that the reservations can safely shrink. They precede worker join and output,
exclude the separate interrupt stack and miss later/unexercised paths.

The six stored snapshot values occupy **3,552 bytes**: the archived
[ELF layout](artifacts/preflight/snapshot-layout.txt) reports 592 bytes each.
That is one explicit harness component, not its total RAM cost. Temporary SDK
status arrays, compiler copies, helper/call frames, channels, Rust/SDK state and
later output allocations remain separate. The
[frame inspection](artifacts/preflight/frame-excerpts.txt) records additional
stack reservations. The heap control and snapshot storage are also distinct
from the older queue harness's 48,000-byte sample slice.

## Preparation, recovery and limits

An initial build setup was deliberately abandoned while its dependency tooling
began installing a duplicate toolchain under the new target directory. The
[setup note](artifacts/preflight/setup-interruption-note.json) classifies this
as **operator-reported termination**, with a partial build log and no archived
exit receipt; it is not a compiler/linker or Rust-feasibility failure. The later
successful build's unused-helper warnings and final clean build are retained.

The first converted candidate produced an
[invalid-frequency warning](artifacts/preflight/accounting-image-info.log)
despite a zero image-info exit status. It was retained and rejected **before
flashing**. The selected [SDK-generated image](../../../../results/idf-accounting-qualification-2026-10-23/artifacts/image.bin), header
inspection and transfer record are separate artifacts. Qualification concerns
that selected image and its actual runtime readbacks, not the rejected candidate.

The preserved private full image was independently checked against the complete
device flash before replacement; this was verification of the retained image,
not a claimed fresh full-flash read. After the three attempts, the complete
**8,388,608 bytes** were [restored](artifacts/restore.log), then a
[separate verification](artifacts/restore-verify.log) reported a matching digest
and reset via RTS. [device-state.json](device-state.json) records the verified
backup identity and observed completion. Application changes after that reset,
including NVS writes, are outside the verification claim. Laboratory restoration
does not establish deployed crash/update recovery.

The next controlled-load design can use these qualified API observations with
their stated limits. It still needs independently specified arrivals, bounded
backlog/deadline semantics, complete work/drop accounting and a separate
instrumentation-cost control. Physical CPU busy/sleep, exclusive ISR occupancy,
global runtime memory peaks, radio/coexistence, energy and wider platform
selection remain open.

## Offline rejection controls

Independent checks applied sixteen mutations to each of the three actual raw
captures: 48 rejected cases covering fatal/reboot text, counter width/clock,
impossible or decreasing counters, task identity/state, checksum/allocation
controls, blind-spot behavior, configuration drift and missing completion.
Frozen and then-current collectors reproduced the saved reports exactly; those
executables were byte-identical, so they were not independent validator
implementations. Maintained counterexamples live in
[accounting.rs](../../tools/benchctl/src/accounting.rs).
