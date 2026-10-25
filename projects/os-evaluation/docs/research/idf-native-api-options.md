# Native FreeRTOS APIs available to our Rust IDF baseline

Primary-source review, **2026-10-23**. This is a source/API selection note, not a
build or performance result. It does not modify the frozen experiment, firmware,
dependencies or current captures. The baseline remains Rust **1.96.0**, IDF
**5.5.6**, and exact `esp-idf-sys =0.37.2`.

## Existing wrapper compatible with the baseline

Released **esp-idf-hal 0.46.2**, commit
[`0119ee33080438f52d181d38e2ac77c206818e1c`](https://github.com/esp-rs/esp-idf/tree/0119ee33080438f52d181d38e2ac77c206818e1c),
contains native queue and task-notification wrappers. Its
[manifest](https://github.com/esp-rs/esp-idf/blob/0119ee33080438f52d181d38e2ac77c206818e1c/Cargo.toml)
declares Rust **1.82** and dependency `esp-idf-sys = "0.37.1"` with defaults
disabled. That requirement permits our exact **0.37.2**. HAL defaults enable
`std,binstart`; its `std` enables `alloc`. This is manifest compatibility, not a
new clean build or C6 hardware pass under our SDK/configuration.

The maintained project now lives in the [Rust IDF monorepo](https://github.com/esp-rs/esp-idf).
The newer HAL **0.47.0** requires sys **0.38**, so it is not the appropriate
drop-in for this frozen sys 0.37.2 baseline. Availability of the older release
does not establish a backport/support promise for its series. See the
[compatibility audit](platform-compatibility.md).

All APIs below were read in the actual released
[`src/task.rs`](https://github.com/esp-rs/esp-idf/blob/0119ee33080438f52d181d38e2ac77c206818e1c/src/task.rs),
where `queue`, `notification` and `asynch` are inline modules.

| Option | Actual semantics | Adoption considerations |
| --- | --- | --- |
| `task::queue::Queue<T>` | `T: Copy`; `send_back` plus `recv_front` gives copied FIFO values. Task calls use `xQueueGenericSend` / `xQueueReceive`; ISR calls select the corresponding `FromISR` operation. Timeout is FreeRTOS ticks, ignored in ISR context. | This is the closest existing wrapper for a capacity-one checked `u32` request/reply contract. Send/receive report failure/timeout; ISR results expose whether a higher-priority task woke, requiring the caller's yield decision. |
| `task::notify` and `wait_notification` | Notification index **0**; sends use **eSetBits**, waits clear **all bits on exit**. Values are `NonZeroU32`. ISR and task branches differ. | Pending values combine by bitwise OR. Repeated signals can coalesce; there is no FIFO or arbitrary message sequence. A wake flag plus separately synchronized payload/acknowledgment would be a different explicitly reviewed contract. |
| `task::notification::Notification` / `Notifier` | An allocated `Arc` stores the current task handle; waiting remains task notification index 0. `PhantomData<*const ()>` keeps the receiver tied to its originating task; notifier clones are shareable. | Creation must be in task context. `Notifier::notify` is unsafe and documents that it must not outlive the originating task, otherwise a dangling task handle can corrupt memory. `Arc` keeps the notifier allocation alive, not the FreeRTOS task. This is not a lifetime-free replacement for task ownership. |
| `task::asynch::Notification` | Atomic pending bits plus `AtomicWaker`; async waiting takes accumulated bits. | A wake/bit-notification primitive, not a FreeRTOS FIFO or a stackful task notification benchmark. Do not substitute it under an unchanged queue label. |
| `task::create` / `destroy` | Thin raw FreeRTOS task creation/deletion wrappers with raw callback argument and handle. | Unsafe API; worker argument, callback behavior and task lifetime still need explicit invariants. Upstream recommends ordinary `std::thread` for general application use; raw tasks are a deliberate native-operation/integration choice. |

## Queue ownership and failure behavior need review

`Queue::new(size)` calls `xQueueGenericCreate`, allocates FreeRTOS queue storage,
and returns an owned handle. The inspected constructor **does not check the
returned pointer for null** and casts `usize` capacity/item size to `u32`.
Therefore it does not provide a fallible Rust allocation boundary. A production
adoption needs an allocation-failure/capacity policy; do not call it fully
qualified merely because the wrapper exists.

Owned queue drop calls `vQueueDelete`. Task users must have finished before the
last owner drops it. The type is `Send + Sync` when `T: Send + Sync`, but an
exposed raw handle or foreign blocked task is not made lifetime-safe by those
traits. `unsafe Queue::new_borrowed` checks non-null and skips deletion, yet
has no Rust lifetime parameter tying it to the external allocation. The caller
must preserve the queue, its item layout/type and cleanup ordering. This can
wrap separately managed storage, but it does not remove the ownership audit.

These points support evaluating the existing HAL wrapper before maintaining a
general-purpose duplicate. They do **not** require replacing the already frozen,
narrow experimental FFI or relabeling its eventual result as a wrapper result.
Before adopting either route, compare null/error handling, finite timeout,
cleanup and task lifetime against the intended API. No broader safety review
of the entire HAL crate is claimed here.

## What std channel blocking actually reaches

The selected Rust standard-library source is commit
[`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`](https://github.com/rust-lang/rust/tree/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std).
Its bounded-channel blocking machinery uses an mpmc
[`Context`](https://github.com/rust-lang/rust/blob/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src/sync/mpmc/context.rs)
that parks/unparks the thread. The
[`thread_parking` selection](https://github.com/rust-lang/rust/blob/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src/sys/sync/thread_parking/mod.rs)
routes the ESP-IDF Unix target to the
[`pthread` parker](https://github.com/rust-lang/rust/blob/ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96/library/std/src/sys/sync/thread_parking/pthread.rs).
That parker has atomic fast paths; when blocking is necessary it coordinates
a pthread mutex and condition variable. A channel operation that does not
block need not execute the entire pthread wait path.

In pinned IDF 5.5.6,
[`pthread_cond_var.c`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/pthread/pthread_cond_var.c)
allocates the condition object in `pthread_cond_init`. Its wait path creates a
**stack-backed static counting semaphore**, inserts a waiter in a protected
list, releases the caller mutex, waits, removes/deletes the semaphore, then
reacquires the caller mutex. Signal finds a waiter under its own lock and gives
that semaphore. **This source does not support claiming a heap allocation on
every wait.** Initial condition-object allocation and repeated static-semaphore
bookkeeping are different operations; initialization can occur before timed
samples. Mutex contention, fast paths and actual waits depend on execution.

These are source-level candidate costs to separate in future experiments.
They do not attribute any observed duration or ratio to allocation, condition
variables, scheduling or a particular instruction. No in-flight or historical
performance samples were consulted for this note.

## Other crates and bounded decision

The separate [FreeRTOS-rust project](https://github.com/lobaro/FreeRTOS-rust/blob/8b024f1174a0b40668d546e50c40b81a7df14a90/README.md)
documents a build helper that compiles an upstream FreeRTOS kernel, port and
heap into the Rust application. That is not a demonstrated adapter to the
already-running IDF C6 kernel. It adds integration work rather than supplying
an established drop-in for this baseline; no C6/IDF compatibility or current
support guarantee was verified here.

Bounded upstream issue searches for queue-null/notifier lifetime and IDF
pthread-condition/semaphore topics yielded no directly relevant verified issue
to cite. That is not proof that no issue exists. The concrete cautions above
come from pinned implementation, not an inferred issue status.

For the next **adoption** review, prefer first checking released HAL **0.46.2**
for the exact FIFO requirement, while retaining its error/ownership caveats.
Use notifications only when an explicit coalescing wake/bit contract fits, or
after separately designing a payload protocol. Keep direct sys FFI for a
deliberately narrow missing operation or controlled test, with its small safety
boundary documented. No dependency or API choice is changed by this research.
