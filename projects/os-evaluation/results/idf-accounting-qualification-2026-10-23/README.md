# IDF accounting qualification

Completed **2026-10-23** under the
[prospective contract](../../docs/experiments/idf-accounting-qualification.md).
All **three scheduled qualification boots passed**, with no replacement.
[Results](RESULTS.md), [raw-derived analysis](analysis.json) and the
[independent result review](RESULTS.md) retain
the findings and scope. The attempt budget is in [schedule.json](schedule.json);
there is no comparative estimator.

Exact prepared bytes are frozen by [INPUTS.sha256](INPUTS.sha256), with the
input freeze commit `f80cf87d316b6c02143adbe4114ee51764f14802`, bound in the
ledger before capture. [SHA256SUMS](SHA256SUMS) seals the completed collection.
[Provenance](provenance.json) distinguishes the dirty-at-build descriptor from
the matching committed source snapshots. The original device state was
independently verified against the full private 8 MB backup before replacement.
After collection, the full image was restored, separately verified and reset;
see [device-state.json](device-state.json).

The first image-converter candidate was rejected during header inspection,
before flashing. Both that candidate/diagnostics and the selected SDK-generated
image remain in the inputs. A zero exit status from image-info did not suppress
its frequency warning. The selected image has a valid C6 header/digest; runtime
clock readback still must pass on the board.

This is qualification of task-attributed wall counters and allocator indicators.
It does not establish physical CPU utilization, exclusive ISR time, sustainable
load capacity, global memory peaks, energy efficiency or a platform selection.
