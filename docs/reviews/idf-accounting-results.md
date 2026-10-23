# Independent IDF accounting results review

Reviewed **2026-10-23**, using retained files and read-only host commands. This
review performs no board operations. It follows the
[independent preflight](idf-accounting-independent.md) and checks the actual
[qualification collection](../../results/idf-accounting-qualification-2026-10-23/RESULTS.md)
against its [prospective contract](../experiments/idf-accounting-qualification.md).

The three retained scheduled boots satisfy that narrow contract. Independent
raw recalculation found no disagreement with the capture reports or
[analysis.json](../../results/idf-accounting-qualification-2026-10-23/analysis.json).
That is a result about this diagnostic method, not overall project completion
or selection of an OS.

## Identity, schedule and provenance

- All 51 frozen input files still match `INPUTS.sha256`; the manifest's bytes
  match freeze commit `f80cf87d316b6c02143adbe4114ee51764f14802`.
- Twenty-two archived authored-source/manifest/contract files match source
  commit `c931379e2042b04cdb59c93e57441cc1182f5f19`. This reconciles the earlier
  dirty-at-build descriptor with the later matching commit, without claiming
  the tree was clean when built.
- The retained runs match schedule IDs `accounting-01`/`02`/`03`, order 1/2/3,
  and distinct increasing host timestamps. All carry qualification purpose,
  null comparative benchmarks, the same ELF and original provenance bytes.
- Each raw capture contains one C6 ROM startup, one reset-reason line and twelve
  JSON records. No searched panic, reboot, abort or image-SHA failure marker
  occurs. Extracted raw JSON matches `records.jsonl` byte-for-byte.
- All run-local ELF/provenance hashes match their admissions and archived
  artifacts. The frozen collector and current release have the same SHA-256,
  `31f56d6c04dd960e9fb52be16b7e2706b6fbddcd769955ae381a1cf51a6ebc8c`.

Those checks do not authenticate hardware or prove absence of unrecorded events.
The retained collector stops after completion and the received chunk; later
diagnostics are outside its observation window. Schedule labels and warm resets
are not proof of statistical independence.

## Direct raw recalculation

The independent calculation parsed each `serial.raw`, matched counters by
`(handle, task number)`, and used integer subtraction/summation. Reported
aggregates were compared afterward. All fifteen adjacent-snapshot phase
calculations, eighteen snapshot summaries, raw hashes, checksums and counts
match the assembled analysis. Array order changes without changing identity.

These values agree across all three boots:

| Quantity | Recalculated value |
| --- | ---: |
| Retained snapshots / task records | 18 / 72 |
| Stable live tasks per snapshot | 4 |
| Blind-spot wall span / stored-counter change | 10,000 / 0 us |
| Blocked-phase total-clock interval | 259,857 us |
| Blocked worker / sampler / idle-task increments | 0 / 11,217 / 258,783 us |
| Blocked sum / signed boundary residual | 270,000 / −10,143 us |
| Busy worker's own bracket | 250,215 us |
| Busy worker / sampler / idle-task increments | 250,262 / 1,060 / 0 us |
| Busy-phase total / sum / signed residual | 251,333 / 251,322 / 11 us |
| Checked busy workload calls per boot | 810 |
| Wrapping busy checksum | 2,470,336,773 |
| Requested allocation / reported allocated-byte increase | 16,384 / 16,384 bytes |
| Current-free decrease / control drift / recovery drift | 16,388 / 0 / 0 bytes |
| Checked allocation checksum | 2,041,721 |
| Minimum recorded worker / sampler stack-fill-free | 7,248 / 16,972 bytes |

The busy checksum was independently calculated as the unbounded integer sum
of `0xa3944d55 XOR i`, indices 0–809, reduced modulo `2^32`. The allocation
checksum sums `i mod 251` over all 16,384 bytes. These check record consistency;
the reviewed firmware supplies the actual per-return and per-byte checks.

The negative blocked residual has an expected source-derived explanation.
The deliberate ten-millisecond scheduler-suspended blind-spot control occurs
**before the first baseline**. Its work is still inside the sampler's open
interval: the stored counter is 680/681 us both during that control and at the
first snapshot. A subsequent delay/context switch charges the older interval,
so later counter deltas import work preceding the elapsed-time boundary. The
noninstantaneous scans add boundary effects. No trace identifies every
component of the 10,143 us difference; neither its sign nor the positive busy
residual represents idle time, a backwards clock or an exact conservation test.

Gross observer brackets across eighteen snapshots are **1–2 us** for the empty
timer bracket, **929–970 us** for task enumeration and **28–39 us** for heap info.
They are retained costs of this configured path, including stack-fill scans and
possible interrupt interference. They do not isolate instrumentation-on/off
overhead and are not subtracted from prior or prospective workloads.

The free-heap decrease exceeds the requested allocation by four bytes, while
allocated bytes increase by the requested size. The report retains this
accounting difference. One allocation does not establish general allocator
overhead or fragmentation. The lifetime minimum remains low after freeing;
it is a sum of region minima, not a simultaneous global peak.

The tied coarse phase results do not establish zero variability or absolute
precision. Raw hashes differ; initial stored counters and some endpoints/
observer durations differ by a microsecond. There are three warm resets on one
board and image, without a comparative estimator or independent timing fixture.

## Counterexamples using the actual captures

Sixteen mutations were made separately from each actual raw capture, producing
**48 distinct counterexamples**. Every one rejected for its intended category:

| Counterexample group | Mutations per boot |
| --- | ---: |
| Panic or reboot before JSON; additional ROM; panic after completion | 4 |
| Wrong counter width or clock | 2 |
| Impossible lifetime-counter baseline; decreasing worker counter | 2 |
| Missing worker, changed task number or wrong phase | 3 |
| Wrong busy checksum; missing allocation drop | 2 |
| Advancing stored counter in blind-spot control; configuration drift | 2 |
| Missing completion | 1 |

Both frozen and current collector paths reproduced each original `replay.json`
byte-for-byte and rejected these mutations. The binaries are byte-identical;
repeating their paths is not an independent validator implementation. One
local matcher initially expected a shortened error phrase; the validator had
already rejected the impossible baseline correctly. Correcting the matcher
confirmed that intended rejection without code, data or acceptance-rule changes.

The independently run current `replay-archives --root .` reproduces all **64
retained protocol transcripts and the existing study summary**. Native Ariel/
Zephyr console suites are outside that JSON-protocol command; this does not
convert failed native qualifications into passes. The maintained protocol-4
capture/replay counterexamples are in
[accounting.rs](../../tools/benchctl/src/accounting.rs). Supplementary local
raw calculations and mutations remain under `.cache/accounting-independent/`.

## Preparation and restoration evidence

The result report correctly distinguishes the setup interruption's
operator-reported exit from its incomplete archived log. It does not call that
setup a Rust/compiler incompatibility or a failed physical boot. The rejected
converter candidate remains separate from the selected SDK-generated image;
the frequency warning returned zero from image-info, as already checked during
preflight. Neither event changes the three-boot attempt budget.

The public restore logs record a full 8,388,608-byte write, a separate successful
digest verification and hard reset through RTS. The private backup file's size
and SHA-256 independently match
[device-state.json](../../results/idf-accounting-qualification-2026-10-23/device-state.json):
`863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
This review reads those receipts; it does not reread the board. The recorded
completion timestamp is an operator-observed completion, not an electrical reset
timestamp. Persistent-state changes after reset and deployed recovery remain
outside this laboratory restoration claim.

The checked results support further development of task-inclusive accounting
and heap indicators in this Rust/IDF profile. Physical busy/sleep residency,
exclusive task/ISR time, independently offered load, capacity/deadlines, global
runtime peaks, instrumentation cost, external accuracy, radio, energy and an
overall platform decision remain unestablished.
