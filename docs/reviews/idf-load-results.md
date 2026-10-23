# Independent finite offered-load result review

Reviewed **2026-10-23**, after both scheduled qualification boots. This agent
performed **no board operations**, implementation edits or commits. Review
covered the actual raw/provenance graphs, [results report](../../results/idf-load-qualification-2026-10-23/RESULTS.md),
[independent analysis](../../results/idf-load-qualification-2026-10-23/analysis.json),
and the host-reader correction discovered during final archive replay. The
[preflight review](idf-load-independent.md) remains a separate record of what
was checked before collection.

Both retained boots satisfy the frozen **qualification** contract. Their adverse
outcomes remain in the results. This review found and independently reproduced
a subsequent archive-reader false rejection; the explicit host-only correction
passes the observed counterexamples. These findings do not establish that the
implementation, experiment coverage or wider project is complete or error-free.

## Frozen inputs and actual attempts

The source/contract revision is
`093153160316e01f0dea364ef999f3be15ddc9d2`, the exact input freeze is
`b23d17ac28391e79d6f7d4e6547b8a258339a45f`, and the pre-flash ledger binding is
`671276baa196932f6afb0dc1ca7563cf701e35e9`. All **69** manifest entries still
match. The manifest itself matches its frozen Git revision; **27** archived
source/configuration/contract files were also compared byte-for-byte with the
source revision. Captured ELF/image hashes, generated SDK configuration and
bindings agree with the corresponding profile's provenance. The dirty app
descriptor is retained rather than presented as a clean build.

The scheduled disabled and enabled attempts each contain **2,043 JSON records**,
**2,016 nominal jobs**, **12 snapshots**, and **60 task records**. Each retained
raw transcript has one ROM startup and no diagnostic rejected by the recognized
fatal/reboot guard. This is a statement about retained bytes, not authenticated
physical provenance or proof about events before the capture began. No
replacement attempt or extra physical observation was introduced by this review.

Actual configurations both have two notification entries and common tracing
support. Disabled native counter fields are u32 and all unavailable measured
counters/clock declarations are null. Enabled fields and total pointers are
u64, with the selected esp_timer counter. The regenerated bindings, rather than
an assumed common ABI, support those distinctions.

## Independent numerical and control checks

Read-only Python inspection recalculated raw outcomes before comparing capture
aggregates: slot/grid conservation; accepted/full/missed status; FIFO order and
the conservative possible-queued bound; all **3,556** completed payload
checksums; deadline and cutoff membership; callback counts; generation/drain
bounds; and stable task identities. Each queue control reports eight accepted
items, ninth-item rejection and verified FIFO/empty state. Each no-work pulse
retains all 125 completions. Each delayed control retains four deliberately
missed slots and twelve completions, without replaying old releases.

All four work pulses in each boot pass the prospective generator gate. Their
actual queue-full and deadline failures are outcomes, not reasons to discard a
pulse. The disabled period250 pulse retains **227 full rejections**, **773
accepted jobs**, **764 in-window completions**, **9 drain completions**, and
**22 deadline-met jobs**. The enabled pulse separately retains **241 full
rejections**, **759 accepted jobs**, **751 in-window completions**, **8 drain
completions**, and **21 deadline-met jobs**. Neither has unfinished accepted
work. These are separate qualification observations; no contrast is estimated.

An additional raw-to-analysis check verified all **60** distribution objects,
including nearest-rank quantiles and exact integer sums/counts, every gross
observer bracket, heap observation and stack-fill indicator. Enabled task
deltas and signed residuals were recalculated across all six whole-phase
intervals. All **50 numeric Markdown table rows** agree with that checked
analysis. No table or numerical discrepancy remains in the reviewed report.

## Actual-data adversarial checks and reader defect

The archived [offline outcomes](../../results/idf-load-qualification-2026-10-23/reader-correction/independent/adversarial-outcomes.json)
retain **26 mutations per actual transcript, 52 total**. The original frozen
collector and corrected reader reject them: fatal text before/after JSON,
additional startup, wrong/absent counter declarations, task identity/state
corruption, callback/notification/cleanup lies, missing completion, wrong
checksum, hidden control misses, impossible queue history, invented service for
missed/full rows, counter corruption, overlong drain/generation, old-slot replay,
clock drift, extra records and unknown fields. These were offline CLI mutations
of actual raw input, not additional board attempts or live-port fault injection.
The two historical executables used for the original check were byte-identical;
they are not independent validators. Temporary mutation generation is auxiliary
inspection, not a separately maintained test suite.

Final whole-archive replay nevertheless exposed a distinct failure that those
cases did not cover. In the disabled report, the precise path is
`phases[5].conditional_latencies.send_entry_to_completion_us.mean_us`:

| Quantity | Value |
| --- | --- |
| Exact integer numerator / count | `2132498 / 773` |
| Fresh value and original saved decimal | `2758.7296248382922` |
| Fresh binary64 bits | `0x40a58d75916305a1` |
| Default serde_json parsed value | `2758.729624838292` |
| Default parsed bits | `0x40a58d75916305a0` |

The frozen collector successfully replays raw input to the original JSON bytes,
but its default JSON reader parses this saved mean one ULP lower. Exact
whole-archive comparison therefore rejects `phases`. An independent Rust walk
found this sole sum/count-to-parsed-bits mismatch across both original reports.
The [retained probes](../../results/idf-load-qualification-2026-10-23/reader-correction/independent/README.md)
reproduce it with pinned serde_json 1.0.154 and demonstrate the exact-bit result
with `float_roundtrip`. Both locked reproduction recipes were executed again
from the formatted retained sources and reproduce their archived outputs exactly.

The initial full synthetic positive used uniform/integer-valued timing means;
it did not include a decimal that failed binary64 serialization/read-back.
Covering the complete protocol shape and many admission faults did not test
this numerical persistence invariant. The actual fractional mean exposed that
coverage gap. Printed-number agreement alone also failed to distinguish the
parser's one-ULP loss. The maintained regression now includes the exact
numerator/count, actual raw archive graph and strict altered-mean rejection;
that closes the observed case rather than proving all persistence behavior.

The [reader correction](idf-load-replay-correction.md), commit
`87c9343140de649aaf5dd205275763d31dc7dfc9`, enables the upstream
`float_roundtrip` feature on the same pinned dependency. It introduces no
tolerance, rounded comparison, changed calculation or evidence rewrite.
The identities remain separate:

| Executable | SHA-256 |
| --- | --- |
| Original frozen collector | `f3211f0b3762cb1ba96ef81b6110a74eefc542076abcf55ae5f3720610db8cbe` |
| Corrected host reader | `4171af48b87cebfd8ec408be9f4fe6f96a4ba0fe49061a21bc7068d18911968f` |

With that corrected reader, both standalone outputs remain byte-identical to
the original saved replays; **all 66 retained transcripts and one existing
study summary** replay consistently. A temporary actual-data archive graph
passes unchanged and rejects modifications of this mean by **one ULP in either
direction**. All 52 raw mutations also still reject. These checks preserve
strict arithmetic/report admission; the frozen historical collector still has
its documented offline-reader limitation. No new physical run is inferred.

## Device state and decision limits

The [restoration receipt](../../results/idf-load-qualification-2026-10-23/device-state.json)
agrees with the retained full-image write and separate verification logs. The
private backup is **8,388,608 bytes**, SHA-256
`863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`;
its file was independently hashed, without reading the board. Logs show full
restoration, digest verification and an RTS reset. **15:29:34 UTC** is the
operator-observed completion time, not an externally timed reset edge.
Restoration does not establish post-boot NVS constancy, interrupted-update
recovery or deployed rollback behavior.

One fixed-order boot per profile, finite pulses and one board cannot establish
a statistics-configuration effect, sustainable capacity, physical CPU/ISR/idle
residency, external timing accuracy, global memory peaks or application
headroom after removing all harness costs. Whole-phase task accounting includes
lead/drain/boundary work and a signed residual; sampled heap/stack values retain
their narrower meanings. These pilots cannot be promoted into a later effect
study and select neither an OS nor a custom runtime.

## Verification and inspection-artifact scope

Executed independently: `shasum -a 256 --check` on the frozen input manifest;
raw/provenance/Git comparisons and direct arithmetic above; corrected
`validate-load` and `replay-archives` positive/negative checks; both pinned probe
recipes; and `cargo test --workspace --locked` (**98 unit tests, two doctests**).
Formatting was checked on both retained Rust inspection sources. Root's other
build/lint/documentation/target checks remain separately reported evidence.

The [existing API inventory](../rust-api-checklist.md), firmware/observer/host
reviews and correction delta retain their implementation applicability. The
new probe sources are retrospective host inspection artifacts with no public
API, FFI, concurrency or device lifetime. `C-SERDE` and `C-VALIDATE` concern the
explicit decimal/bits check; the probes do not claim full report admission.
`C-FAILURE` is scoped to local fail-fast inspection of retained inputs; malformed
input or file errors can panic. `C-CRATE-DOC`, `C-EXAMPLE` and `C-LINK` are served
by the pinned reproduction instructions, outputs and this review. No new
production interface or reusable error-handling promise is introduced.
