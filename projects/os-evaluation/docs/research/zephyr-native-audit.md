# Zephyr native-suite measurement semantics

Independent source/artifact audit, **2026-10-23**. The audited source is Zephyr 4.4.0 commit
[`684c9e8f32e4373a21098559f748f06915f950c9`](https://github.com/zephyrproject-rtos/zephyr/tree/684c9e8f32e4373a21098559f748f06915f950c9)
with Espressif HAL commit `b7953b8019361d09e613f7011d2ccc41b984d087`.
The retained ELF SHA-256 is
`a9929eea623a79386cf4c9c90fbfeb3e73e38a9201e562576aaa7c1e63957f48`;
the Simple Boot image is
`71d71cfcf07de94aee9624f1cae9b83e4a8aff75352876b027a7cdbf4ad360a3`.
Both match the actual archived artifacts, and both pinned source trees were
clean during this audit. This source inspection introduces no new board observation.

**The frequency-header objection and the 19/28 correction-count objection are
confirmed.** The native attempts cannot pass their frozen acceptance rule.
The captured suite completion and metric text remain observations; they do not
become an admitted performance result by removing an inconvenient requirement
after collection. A prospective correction needs a separately versioned
contract and explicit treatment of the image warning below.

## The required frequency header is compiled out

The [frozen contract](../experiments/zephyr-native-tests.md) requires
`Timing results: Clock frequency: 16 MHz` in every capture. Upstream
[`main.c:107–110`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/main.c#L107)
passes that format to `TC_PRINT`. The actual
[`tc_util.h:124–134`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/subsys/testsuite/include/zephyr/tc_util.h#L124)
maps `TC_PRINT` to `print_nothing` unless `CONFIG_ZTEST_VERBOSE_OUTPUT` is
defined. That option is absent from the retained `.config` and actual generated
`autoconf.h`. `print_nothing` ignores its format argument.

Direct ELF string inspection finds neither `Timing results` nor `Clock frequency`;
it does find the final `PROJECT EXECUTION %s` format. Captured output likewise
has no required frequency header. The metric rows print through a different
`printk` path in
[`utils.h`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/utils.h#L90),
and final reporting uses a summary path. Therefore 47 visible metric rows and
the final success banner are consistent with the absent header; they do not
rescue the written acceptance rule. Configuration/disassembly evidence of a
16 MHz counter is separate from a frequency line actually observed on UART.

## Exact timing and correction semantics

The default compiled list contains **47 metrics: 19 explicitly subtract the
timestamp-overhead estimate and 28 do not**. Among the 28 unadjusted metrics,
**eight time one complete loop**, while **twenty sum individually timed spans**.
All 19 adjusted metrics also sum individual spans. Thus neither “most metrics
subtract overhead” nor “only heap timing is unadjusted” describes this image.

The counter path is SYSTIMER UNIT1 at nominal 16 MHz, not the 160 MHz CPU
clock. Printed `cycles` are timer ticks. The source computes printed average
ticks and average nanoseconds separately, so their integer rounding can differ.
The console does not retain individual durations, gross endpoints, the
calibration value or distributions.

This counter interpretation agrees with the retained
[linked disassembly](../../results/zephyr-native-tests-2026-10-23/artifacts/timer-disassembly.txt):
`arch_timing_counter_get` reaches `sys_clock_cycle_get_64`, then the getter
passes counter index **1** to ROM `systimer_hal_get_counter_value`. The
[resolved configuration](../../results/zephyr-native-tests-2026-10-23/artifacts/zephyr.config)
sets `CONFIG_SYS_CLOCK_HW_CYCLES_PER_SEC=16000000`. The
[common timing conversion source](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/arch/common/timing.c#L23)
and [driver](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/drivers/timer/esp32_sys_timer.c#L63)
explain that path without inventing an observed frequency header.

Notation for the mapping below:

- **Σ**: sum a timestamp difference for each operation/iteration, then divide
  by the iteration count. Ordinary batches have `N = 1,000`; heap has `N = 100`
  successful 10-byte allocations/frees.
- **B**: bracket the entire N-operation loop once, then divide. Loop overhead
  remains in this span.
- **O**: subtract `timestamp_overhead_adjustment(0, 0)` **once from the summed
  batch total before averaging**. This is the minimum of ten calibration
  batches, each with an outer timestamp pair and an N-call timestamp-read loop.
  It is not a printed per-event two-read correction. See
  [`timing_sc.c:48–58,94–118`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/timing_sc.c#L48).
- For split blocking paths, **s** is the receiver's timestamp immediately before
  its blocking call, **m** the producer's timestamp before its wake operation,
  and **f** the receiver's timestamp immediately after its call returns. These
  are separate spans, not one full request/reply or external interrupt response.

The table follows the actual expected console order. “None” means no explicit
subtraction in the inspected source, not zero measurement overhead.

| Metric | Mode | Timed endpoints in the selected source | Subtraction | Source |
| --- | --- | --- | --- | --- |
| `thread.yield.preemptive.ctx.k_to_k` | Σ | Initiator before `k_yield` → peer's timestamp after switch, before peer yields back | O | [Y] |
| `thread.yield.cooperative.ctx.k_to_k` | Σ | Same endpoint pair, cooperative worker class | O | [Y] |
| `isr.resume.interrupted.thread.kernel` | Σ | Timestamp near end of offloaded ISR → interrupted thread after `irq_offload` returns | O | [I] |
| `isr.resume.different.thread.kernel` | Σ | ISR timestamp after giving semaphore → awakened thread after `k_sem_take` returns | O | [I] |
| `thread.create.kernel.from.kernel` | Σ | Before → after creation of a dormant other thread | O | [T] |
| `thread.start.kernel.from.kernel` | Σ | Before `k_thread_start` → child's entry timestamp | O | [T] |
| `thread.suspend.kernel.from.kernel` | Σ | Victim before self-suspend → initiator after victim suspends | O | [T] |
| `thread.resume.kernel.from.kernel` | Σ | Initiator before resume → victim immediately after self-suspend returns | O | [T] |
| `thread.abort.kernel.from.kernel` | Σ | Before → after aborting the other thread | O | [T] |
| `fifo.put.immediate.kernel` | Σ | Before → after `k_fifo_put`, no waiting receiver | O | [F] |
| `fifo.get.immediate.kernel` | Σ | After put/before get → after `k_fifo_get(K_NO_WAIT)` | O | [F] |
| `fifo.put.alloc.immediate.kernel` | Σ | Before → after `k_fifo_alloc_put`, no waiting receiver | None | [F] |
| `fifo.get.free.immediate.kernel` | Σ | After alloc-put/before get → after get of allocated queue entry | None | [F] |
| `fifo.get.blocking.k_to_k` | Σ | s before blocking get → m before producer put | None | [F] |
| `fifo.put.wake+ctx.k_to_k` | Σ | m before put → f after receiver get returns | None | [F] |
| `fifo.get.free.blocking.k_to_k` | Σ | s before blocking get → m before producer alloc-put | None | [F] |
| `fifo.put.alloc.wake+ctx.k_to_k` | Σ | m before alloc-put → f after receiver get returns | None | [F] |
| `lifo.put.immediate.kernel` | Σ | Before → after `k_lifo_put`, no waiting receiver | O | [L] |
| `lifo.get.immediate.kernel` | Σ | After put/before get → after `k_lifo_get(K_NO_WAIT)` | O | [L] |
| `lifo.put.alloc.immediate.kernel` | Σ | Before → after `k_lifo_alloc_put`, no waiting receiver | None | [L] |
| `lifo.get.free.immediate.kernel` | Σ | After alloc-put/before get → after get of allocated queue entry | None | [L] |
| `lifo.get.blocking.k_to_k` | Σ | s before blocking get → m before producer put | None | [L] |
| `lifo.put.wake+ctx.k_to_k` | Σ | m before put → f after receiver get returns | None | [L] |
| `lifo.get.free.blocking.k_to_k` | Σ | s before blocking get → m before producer alloc-put | None | [L] |
| `lifo.put.alloc.wake+ctx.k_to_k` | Σ | m before alloc-put → f after receiver get returns | None | [L] |
| `events.post.immediate.kernel` | B | Before → after loop of `k_event_post`, no waiters | None | [E] |
| `events.set.immediate.kernel` | B | Before → after loop of `k_event_set`, no waiters | None | [E] |
| `events.wait.immediate.kernel` | B | Before → after loop of wait-any with already-set bits, reset=false | None | [E] |
| `events.wait_all.immediate.kernel` | B | Before → after loop of wait-all with already-set bits, reset=false | None | [E] |
| `events.wait.blocking.k_to_k` | Σ | s before reset=true wait-any → m before producer set | O | [E] |
| `events.set.wake+ctx.k_to_k` | Σ | m before set → f after receiver wait-any returns | O | [E] |
| `events.wait_all.blocking.k_to_k` | Σ | s before reset=true wait-all → m before producer post | None | [E] |
| `events.post.wake+ctx.k_to_k` | Σ | m before post → f after receiver wait-all returns | None | [E] |
| `semaphore.give.immediate.kernel` | B | Before → after loop of gives, no waiters, count grows toward N | None | [S] |
| `semaphore.take.immediate.kernel` | B | Before → after loop of N nonblocking takes from available count | None | [S] |
| `semaphore.take.blocking.k_to_k` | Σ | s before blocking take → m before peer's first give | O | [S] |
| `semaphore.give.wake+ctx.k_to_k` | Σ | m before first give → f after receiver's first take returns | O | [S] |
| `condvar.wait.blocking.k_to_k` | Σ | s before condvar wait → m before peer signal | None | [C] |
| `condvar.signal.wake+ctx.k_to_k` | Σ | m before signal → f after wait returns/reacquires mutex | None | [C] |
| `stack.push.immediate.kernel` | Σ | Before → after `k_stack_push`, no waiting receiver | O | [K] |
| `stack.pop.immediate.kernel` | Σ | After push/before pop → after nonblocking pop | O | [K] |
| `stack.pop.blocking.k_to_k` | Σ | s before blocking pop → m before producer push | None | [K] |
| `stack.push.wake+ctx.k_to_k` | Σ | m before push → f after receiver pop returns | None | [K] |
| `mutex.lock.immediate.recursive.kernel` | B | Before → after loop of N recursive locks on one uncontended mutex | None | [M] |
| `mutex.unlock.immediate.recursive.kernel` | B | Before → after loop of N corresponding unlocks | None | [M] |
| `heap.malloc.immediate` | Σ | Direct `timing_counter_get` before → after each `k_malloc(10)` | None | [H] |
| `heap.free.immediate` | Σ | Direct counter read before → after each `k_free` | None | [H] |

[Y]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/thread_switch_yield.c#L37
[I]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/int_to_thread.c#L56
[T]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/thread.c#L80
[F]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/fifo.c#L34
[L]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/lifo.c#L34
[E]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/events.c#L28
[S]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/sema_test_signal_release.c#L22
[C]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/condvar.c#L21
[K]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/stack.c#L29
[M]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/mutex_lock_unlock.c#L22
[H]: https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/tests/benchmarks/latency_measure/src/heap_malloc_free.c#L14

The ISR rows use `irq_offload` software-generated interrupts and start near
ISR return, after an optional semaphore give. They do not measure an external
edge, IRQ-entry latency, the complete ISR duration or load interference.
Likewise, a row's “blocking get” label does not by itself specify a complete
blocking-call duration: the source deliberately divides the receiver/waker
path at m. Preserve these endpoints when considering any future comparison.

### The correction is not an isolated two-read observer control

For the kernel-only configuration, the actual calibration is
`O = min_j(delta(start; N timestamp calls in a loop; finish))`, for ten j.
It contains the loop's increment/branch work and the outer measurement
intervals in addition to the timestamp reads. The adjusted result is
`(sum_i(delta(s_i, f_i)) - O) / N`, with further integer conversion for output.
Each operation span in that sum has different endpoints from the calibration
loop: its surrounding iteration bookkeeping can be outside the timed span.

Taking a minimum can reduce interrupt/noise contamination, but it does not
remove calibration loop work or make the control structurally identical to
the operation spans. The subtraction therefore does not establish an unbiased
isolated operation cost. **Over-subtraction is possible**; its actual size or
even sign for a particular row cannot be quantified from these console means
because neither O nor the gross summed intervals is printed. This is a method
limitation established by source, not a measured correction error of a claimed
number of ticks. Retaining source and unmodified console output allows this
distinction; reconstructing unprinted raw totals would invent data.

## Image warning reproduced from the frozen file

The retained consoles report **`SHA-256 comparison failed:`**, show calculated
and expected values, then say **`Attempting to boot anyway...`**. The native
suite subsequently prints its rows and terminal success. This warning was
missed by the initial local rejection-text scan and requires its own image
qualification. Successful transfer MD5 and the suite's banner do not erase it.

Parsing the archived `zephyr.bin` directly reproduces both ROM values:

| Byte-level evidence | Value |
| --- | --- |
| Segment count in ROM image header | 2 RAM segments |
| RAM segment lengths | 40,304 and 6,464 bytes |
| RAM checksum byte offset | `0xb6df`; checksum is valid |
| End of ROM-visible image through checksum | `0xb6e0` |
| SHA-256 of bytes `[0, 0xb6e0)` | `4d04a62e914caf8e16b81c0996f6c456e61f4c9c1573dc9bc00cbe85e314c046` |
| Next 32 bytes, at `0xb6e0` | `0000000010490000000000000000000000000000000000000000000000000000` |
| Extended-header `hash_appended` byte, offset 23 | `0` |

The first digest exactly equals the ROM's printed “Calculated” value; the next
32 file bytes exactly equal its printed “Expected” value. Thus the warning is
consistent with the frozen input layout itself, rather than requiring random
serial corruption as an explanation. This is not an assertion that every ROM
integrity behavior is now understood or that the warning can be ignored.

The actual build command uses esptool 5.6.0 `elf2image --ram-only-header`.
In that installed primary implementation, `esptool/cmds.py:3507–3509` explicitly
sets `append_digest = False` for this option; its help text describes a RAM-only
header with no SHA-256. `bin_image.py:863–901` writes RAM segments and their
checksum before the hidden flash-segment/padding layout. Zephyr's pinned
[`soc/espressif/common/CMakeLists.txt`](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/soc/espressif/common/CMakeLists.txt#L38)
selects that option for Simple Boot. The retained image-info output checks the
ROM-visible checksum but does not certify this warning away.

Upstream [esptool commit `5d9d5be`](https://github.com/espressif/esptool/commit/5d9d5be)
documents the intentional digest omission: the boundary of the RAM-visible
prefix differs from the complete file's boundary. A bounded primary-source
search also found the same warning pattern in the older C6
[Simple Boot support discussion](https://github.com/zephyrproject-rtos/zephyr/discussions/62138).
That historical report supports treating this as an image/loader integration
question worth researching, rather than silently diagnosing random transfer
damage. It does not prove that this revision's warning is benign or alter our
frozen rejection rule. No broad issue-status or current support conclusion is
inferred from that discussion.

No replacement image or flash experiment was made during this audit. A later
fix needs separate source/tool/ROM review, a new image identity and an explicit
prospective integrity criterion. Existing attempts stay failed under the frozen
contract even if a later profile can print the missing header.

### Flash-frequency header alias

The installed esptool 5.6.0 C6 primary implementation,
`esptool/targets/esp32c6.py:84–88`, maps **both `80m` and `40m` to `0x0`**,
with a comment about a ROM MSPI divider workaround. The frozen header's size/
frequency byte is `0x30`. Its low nibble therefore does not uniquely identify
80 MHz. The image-info tool's `80m` label is an alias interpretation, not
independent evidence excluding 40 MHz. The actual CMake command explicitly
requested 80 MHz and the later Simple Boot `flash_init` log reports 80 MHz,
while ROM startup reports `clock div:2`. Preserve those distinct observations;
none supplies the before/after runtime register readback used in the matched
IDF study. The earlier wording that the decoded header independently proves
80 MHz is too strong.

## Independent parsing boundary

The draft global multiline row expression used `\s*` around an empty notes
column. In multiline matching, that can consume a newline and fold the next
metric into notes, producing 24 matches instead of 47. Applying the expression
with `fullmatch` independently to each CR-normalized line finds all 47 expected
names in order, with numeric columns and empty notes in the inspected consoles.
That corrects extraction only. It neither supplies the missing header nor
removes the ROM warning. Initial extraction reports and unchanged console bytes
must remain retained alongside any corrected extraction.

Remaining platform-selection gaps:
these upstream probes do not supply matched cross-candidate Rust workloads,
loaded deadlines/interrupt behavior, runtime memory peaks, radio/coexistence,
sleep/energy, recovery/update qualification or maintenance-effort measurements.
This audit verifies the concrete claims above within its source/artifact scope;
it does not establish that the project or the audit is complete or error-free.
