# Offered-load observer safety contract

The [adapter contract](api.md) and
[load firmware contract](load.md) define the surrounding ownership.
This helper copies task, heap and stack indicators with fixed, typed SDK storage.

## Representation and SDK boundary

`Identity::current` copies the initialized SDK task handle numerically and reads
its priority. Rust never dereferences that handle. The caller must keep actor
tasks alive through subsequent SDK queries; ownership and shutdown belong to
the coordinator/producer, not this read-only helper.

`Snapshot`, `Task` and `Heap` contain only owned numeric values and a static
boundary label. All fields are private except Identity's crate-internal numeric
members. Snapshot Default is explicitly an **unfilled reservation**, not a valid
measurement; capture must replace it before admission. Private count/boundary
fields prevent external mutation into an out-of-bounds slice or an unescaped
JSON label. Capture accepts only the six phase indices and literal before/after
labels. Known actor counters passed by the coordinator are retained as observed;
this helper does not silently replace them with expected values.

Both features use the same Rust `Option<u64>` record fields, preserving their
stored Rust layout and reservations. The two native ABI branches deliberately
differ:

| Build | Actual typed SDK call and stored interpretation |
| --- | --- |
| `load-runtime-stats` enabled | Pass `&mut u64` as total output and assign each generated runtime field directly to u64. Wrong U32 bindings fail compilation instead of being widened/cast. Store `Some` values. |
| Disabled | Pass `&mut u32`, require returned total/task fields to be zero, and store `None`. A nonzero result fails rather than silently suppressing active accounting. Serialized null means unavailable, never measured zero CPU cost. |

Native raw status-array sizes can differ between the U32/U64 SDK definitions;
that does not change the common authored Rust snapshot layout. Generated
bindings, native configuration, actual layouts and linked calls still need
inspection for each profile. Runtime zero checks alone cannot prove every
configuration option.

The fixed 16-record raw array and heap-info record use valid zero representations
of generated C integer/raw-pointer fields. No Rust reference or owned device
resource is fabricated. SDK output points to live aligned writable storage,
with a checked capacity. A zero/over-capacity count fails before slicing or
copying. No name or stack pointer survives the function; only the returned
initialized prefix is copied. The u32 handle representation is specific to this
32-bit target and is not offered as a portable host API.

## Context, timing and memory scope

Capture runs only in initialized IDF task/allocator context, outside load. The
coordinator establishes stable actor lifetimes, blocked state and atomic phase
observations beforehand. This module accepts those phase observations as
parameters; it does not inspect shared application atomics, wait for actors,
alter their priorities or prove timer-callback quiescence.

`uxTaskGetSystemState` suspends scheduling while enumerating tasks and scanning
their stack-fill state; interrupts remain possible. Preserve gross task-call
endpoints, separate heap-call endpoints and the empty-clock bracket, without
subtracting a correction. The sampler's open interval is not flushed. Participant
counter deltas over settled before/after boundaries describe the whole phase,
including drain/boundary work, rather than exact in-window CPU busy time.

The pinned snapshot implementation can report an indefinite notification wait
as state 3, while queue/event-list waits can be state 2. Actor snapshot
boundaries must match the protocol's intended command/queue blocked path; a
numeric state alone does not prove the intended wait site.

`heap_caps_get_info` receives only MALLOC_CAP_8BIT and a live aligned record.
Its current free/largest/lifetime-minimum fields are copied separately; no
overlapping capability classes are summed. Lifetime regional minima do not
establish an exact global peak. Stack-fill indicators are not minimum stack
pointers or future worst-case bounds. ROM heap linkage retains the prior
inspection limitation.

There is no authored allocation in successful capture or serialization, no
mutable static, async cancellation, callback, ISR, unsafe trait implementation,
queue ownership, destructor, task deletion or clock write. SDK calls can take
internal locks and suspend scheduling; they are unsuitable for an ISR or the
per-job service path. Error construction can allocate outside the offered
window, and panic/reset is not claimed to perform graceful cleanup.

Serialization borrows the immutable snapshot and accepts its generic writer by
value, allowing the caller to pass a mutable borrow. It emits numeric fields and
explicit nulls, followed by a newline. The static boundary was validated before
capture; no supplied arbitrary string requires handwritten escaping. Writer
errors return to the caller; partial transport output is not successful protocol
completion. Timing, file/serial I/O and final completion policy remain outside
this module.

## Affected API decisions

| Entries | Decision / evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: Snapshot/Task/Heap/Identity names and `_us`/`_bytes` fields retain explicit meanings. |
| C-FEATURE | Applies at integration boundary: the module has explicit, separately typed runtime-on/off branches. Root owns feature declarations and generated configuration selection; both must be verified. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: scalar snapshots derive Copy/Clone/Debug/Default; Debug reads no hardware. No live resource owner becomes Copy. Default is documented unfilled storage. |
| C-SERDE | Applies: explicit protocol-5 numeric/null serialization and strict host deserialization, as in the existing private firmware records; no reusable interchange library is exposed. |
| C-SEND-SYNC | Applies: copied snapshots contain no SDK pointers or mutable shared views. No manual unsafe Send/Sync implementation is needed. Actor ownership/concurrency remains a caller obligation reviewed separately. |
| C-GOOD-ERR, C-FAILURE, C-VALIDATE | Applies: contextual io::Error for invalid phase/boundary, unusable capacity and unexpected disabled counters. Native width mismatches are compile errors. Full role/identity/clock/heap validation remains explicit host admission, not an inferred source pass. |
| C-RW-VALUE, C-CALLER-CONTROL | Applies: writers passed by value, immutable receiver, no forced stdout/global transport, no implicit flush or allocation policy. |
| C-NEWTYPE, C-CUSTOM-TYPE, C-CTOR, C-METHOD | Applies: Identity and Snapshot distinguish SDK identity from observations; named fields distinguish units and phase roles. `capture` is fallible associated behavior, Default only reserves storage, and output borrows an existing snapshot. |
| C-STRUCT-PRIVATE, C-HIDDEN | Applies: observation internals remain private, with only required parent-module APIs visible. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE | Applies at private binary scope: module/API comments, protocol and this review state context and limitations. Actual target command/run evidence is supplied by integration records, not a claimed doctest. |
| C-NO-OUT, C-INTERMEDIATE | Applies: SDK outputs are converted into owned values returned via Result; failed observations are not resealed or assigned favorable substitute data. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No destructor added. All external calls/output remain explicit. |

Other all-54 dispositions are inherited: no new macro API, iterator/collection
traits, conversions/operators, smart-pointer owner or general data structure is
introduced. The shared timer helper keeps its prior pinned FFI/MMIO review.
