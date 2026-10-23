# Independent native-suite qualification review

Reviewed **2026-10-23**. Scope: the existing Ariel native scheduler tests and
Zephyr native latency suite, their source/build contracts, retained UART
transcripts and restoration evidence. The reviewer did not build, flash,
reset or monitor the device, and did not edit source, frozen inputs or result
files. Root alone operates the serial connection.

This review separates source/build readiness from physical observations.
Unmodified upstream tests supply useful qualification controls; they do not
become a common Rust workload, a checked-message benchmark or an OS ranking.

## Correction to the initial review

The initial review accepted the Zephyr preflight contract too broadly. It
followed the frequency-print call without checking the `TC_PRINT` verbose-output
guard, and it did not establish the correction/bracketing policy separately for
every metric. It also described Ariel's first post-marker window as independently
supported when that five-second wait was operator-reported and is not retained
as an independently replayable timing artifact. The corrected evidence limits
below supersede those blanket acceptance statements. Frozen inputs, original
capture reports and console bytes are preserved.

## Ariel source and input checks

The [brief](../experiments/ariel-native-tests.md),
[build record](../builds/ariel-native-tests.md) and
[frozen archive](../../results/ariel-native-tests-2026-10-23/README.md) agree on
Ariel commit `13bddaca1c2bcd51489c044b80408e917d5946d9`, Rust 1.99.1,
four unmodified apps and twelve fixed attempts, three per app. Freeze commit
`e17c06760277200115e5038b6794d2ce155b0260` precedes the test captures.
All **27 frozen input digests** independently match the retained files.
The upstream checkout has the expected HEAD and a clean status. The source
snapshot retains its MIT/Apache-2.0 notices; the app manifests inherit that
license. No permission-unresolved paper implementation is copied.

Source inspection confirms the following boundaries:

- Yield times one batch of 10,000 initiator calls to `yield_same`. Source declares
  a second default-priority-1 thread that loops on the same operation. Its first
  printed value is a floored aggregate mean; its second is integer division of
  that already-floored mean by two. There is no peer progress counter or
  acknowledgment establishing actual per-iteration switches.
- Flags measures 1,000 `set`/`wait_any` iterations after an ID-exchange channel
  handshake. It supplies no sequence payload, individual timestamps or proof
  that every iteration begins with an identically blocked waiter. A pending
  startup flag can affect the first iteration.
- Priority and lock controls use ordinary Rust assertions before `Test passed!`.
  Their control flows check the stated execution/priority and lock ordering;
  they are not exhaustive scheduler tests. The defaults are 2 KiB thread-stack
  requests, not measured usable stack or high-water usage.
- `debug::exit(Success)` falls back to a spin loop in the selected configuration.
  The benchmark source leaves peer threads in loops after the initiator prints;
  their continuing progress is not observed by the console protocol.
  A marker therefore does not establish shutdown, continuing useful progress
  or freedom from a later hang.

The ESP helper directly brackets an entire batch with SYSTIMER UNIT0 reads.
It computes `total as usize / iterations`; on this 32-bit target, conversion
occurs before division. At nominal 16 MHz, `2^32` ticks span about 268.435 s.
The 30-second host ceiling stays below that truncation range for the admitted
short batch. This does not independently calibrate the oscillator, qualify
all backsteps or close the previously recorded ROM-copy/clock/sleep limits.
Printed ticks remain the primary result. No per-event percentile or WCET
bound follows from these averages.

The selected single-core `yield_same` rotates the same-priority runqueue only
when it contains multiple entries and then requests scheduling. The RISC-V
`schedule` implementation raises a software interrupt; the handler can return
without changing context when its next thread equals the current thread.
These are the intended scheduling mechanics, not an independent count of
executed peer iterations. The benchmark emits neither peer progress nor
context-switch counts. Consequently the printed 77 is an upstream derived
estimate, not validated single-switch latency or proof of 20,000 switches.

## Ariel capture audit

The first completed attempt has one ROM startup and the exact yield output
`took 155 ticks per iteration (77 per context switch)`. Both numbers satisfy
the frozen positive/integer-division criteria; no benchmark error, panic,
assertion-failure or subsequent startup appears in its retained console.
Its 1,674-byte console SHA-256 is
`8142b67ff7e9379c2a1ef48696f3c086a31047842bff159e74fa4f7c80b7986f`.

The operator reported that the named `script` output file was readable while
this capture ran but became empty after the host timeout sent SIGINT. The
retained file is empty; that transient transition cannot be replayed from it.
The separately mirrored stdout
retained the rendered monitor output and was copied byte-for-byte to
`console.log`; the empty `monitor.log` is preserved too. This is recovery of an
already retained output destination, not a replacement capture or filtered
transcript. The first attempt's report records a five-second post-marker wait
that the operator observed still running. That wait is not retained in the
result archive as an independently replayable timing trace. A recorded host
timeout exit of 124 is the prescribed operator stop, not an MCU exit status.

A host service interruption occurred after this first completed attempt. Its
saved result is retained without rerunning it. The frozen contract sets order
and counts without requiring uninterrupted collection or elapsed spacing.

All **12 scheduled attempts satisfy the artifact-visible console checks**. Every
rendered console has one ROM startup and one expected completed-output line,
without the rejected benchmark/panic/assertion/abort markers. Console hashes,
app/round/order, numeric results and transfer-verification evidence match each
run's report. Eleven named script files are empty; run 11's named file is
1,641 bytes and byte-identical to its console mirror. Every complete stdout
mirror is retained as the validation input. No attempt was repeated or replaced.

| Test | Independently checked attempts | Retained observation |
| --- | --- | --- |
| `bench-sched-yield` | 1, 5, 9 | Each prints 155 ticks per iteration and an upstream derived value of 77 under its switch label; actual switch counts are unverified. |
| `bench-sched-flags` | 2, 6, 10 | Each prints 215 ticks per iteration. |
| `threading-dynamic-prios` | 3, 7, 11 | Each reaches the expected success marker after its assertions. |
| `threading-lock` | 4, 8, 12 | Each reaches the expected success marker after its assertions. |

Run 1's claimed 5,000 ms lower bound is **operator-attested**; its archived
start/first-seen/end observation timestamps alone do not prove how long the
monitor remained active after the marker. Runs 2–12 record 23,708–23,760 ms
lower bounds derived from the requested-start and first-seen timestamps plus
the reported timeout exit. The arithmetic reproduces, conditional on the
operator-recorded timestamps and exit metadata. These fields are not an
independent timing instrument or a replayable record of host polling.

| Evidence | Independently checkable from retained artifacts | Limit |
| --- | --- | --- |
| Expected UART output, one startup, no rejected console marker | Yes, in all twelve consoles. | Rendered output can establish only the events it records. |
| Console identity, selected image/source hashes and transfer-verification log | Yes. | Supplied artifacts and operator logs do not cryptographically attest execution. |
| Dynamic-priority and lock assertions precede their success marker in the selected source | Yes. | This validates the particular source-defined checks, without exhaustive scheduler correctness. |
| First run's five-second post-marker wait | No; recorded as an operator observation. | No separately retained polling/monotonic timing trace reproduces it. |
| Later runs' window calculations | Arithmetic can be recomputed from report metadata. | Actual timing depends on the recorded host observations/termination status. |
| Yield peer progress and actual switch count | No. | Neither a peer counter/ack nor an execution trace is emitted. |

The bounded conclusion is **twelve UART test-output checks with limited host
window evidence**, retaining the operator's reported twelve passes. It is not
an independently reproduced proof of every admission condition. Identical
aggregate means across three boots are descriptive; they supply no independent
event distributions, uncertainty bound or general latency guarantee.

## Ariel flash baseline and restoration boundary

The preflight mismatch investigation was independently checked without printing
private flash contents. Both old and current image candidates are 8,388,608
bytes. The fresh prefix is 65,536 bytes, and the current candidate equals that
prefix followed by the historical backup's unchanged tail. Exactly 3,979 bytes
differ: 16 in sector 0xa000, 3,923 in 0xb000 and 40 in 0xc000. All three sectors
are inside the existing NVS range 0x9000–0xefff. No cause for those changes is
established by that comparison.

The current candidate's SHA-256 is
`ecc182713fa7df6b1e3841b5344f4c91528a5fef4e80a3ab37273ca4e89a661a`.
The retained full-device verification log reports a digest match for all
8,388,608 bytes before test writes. This validates the reconstructed full
image, including its tail, without claiming the entire image was freshly read.
Restoring this current baseline preserves the captured latest NVS state.
The historical backup remains separately preserved.

The final Ariel [write log](../../results/ariel-native-tests-2026-10-23/artifacts/ariel-restore.log),
[separate verification log](../../results/ariel-native-tests-2026-10-23/artifacts/ariel-restore-verify.log)
and [device state](../../results/ariel-native-tests-2026-10-23/device-state.json)
record the full current image being written, a successful device-digest
verification of all 8,388,608 bytes and then an RTS hard reset.
These are root's board operations; the reviewer checked
the retained evidence. That evidence describes the verification moment;
an application can update persistence after reset. This is not a claim that
the flash will remain byte-identical indefinitely after the original app boots.

## Zephyr source/build boundary

The pinned local Zephyr 4.4.0 native suite uses a separately defined test thread
at preemptive priority 10. `main` at priority 0 only joins it. Source inspection
therefore confirms preemptive yield workers at priority 8; cooperative workers
are priority −11 with the selected 16 cooperative priorities. Changing main's
priority to repair a supposed cooperative/preemptive mismatch is unnecessary.

The configured timing rate is 16,000,000 Hz. Retained final-image disassembly
routes `arch_timing_counter_get` through `sys_clock_cycle_get_64` to the
SYSTIMER UNIT1 getter. Printed `cycles` are this timer's ticks, not 160 MHz CPU
cycles. The later [47-row source audit](../research/zephyr-native-audit.md)
documents **19 adjusted and 28 unadjusted metrics**: 39 sum individual spans,
while eight bracket a complete loop. The adjustment subtracts the minimum of
ten timestamp-read calibration batches, including their loop work, once from
the operation sum before averaging. It is not an isolated two-read control;
over-subtraction is possible, with no measured size available from the console.
The matrix matches all 47 frozen names in order. Preserve each row's actual
endpoints/correction policy rather than describing most rows as adjusted or
only heap as unadjusted. The initial review did not establish this mapping.

The upstream profile uses a 1 Hz kernel tick, PM off and time slicing off.
Although `CONFIG_ASSERT=y` appears in generated configuration,
`CONFIG_FORCE_NO_ASSERT=y` sets effective `__ASSERT_ON` to zero. The terminal
`PROJECT EXECUTION SUCCESSFUL` reflects the suite's limited `error_count`;
the inspected increment is in the heap-allocation failure path. Qualification
must additionally require all expected unique metric rows and no failed/error
notes, panic, fatal output or extra startup in the observed window.

The default profile has 47 named metric rows, without userspace variants.
Most use 1,000 iterations; heap allocation/free uses 100 iterations of a
10-byte allocation. This is not 47,000 independently retained observations.
`PRINT_STATS_AVG` computes printed floored ticks separately from average
nanosecond conversion, so the rounded columns need not satisfy exact
`printed ns = printed ticks * 62.5`. Terminal success does not establish
checked FIFO payloads, Rust integration, arbitrary memory correctness, radio
support or product readiness.

The [Zephyr build record](../builds/zephyr-native-tests.md) preserves a successful
build. All five primary artifact hashes match the provenance record, the
expected metric list contains 47 unique names, both inspected upstream working
trees were clean, and direct ELF disassembly confirms the UNIT1 getter path.
Those facts do not establish that the complete preflight contract was correct.
The [brief](../experiments/zephyr-native-tests.md) fixes three 120-second captures
with at least two seconds after success and all 47 ordered numeric rows with
empty notes, but also requires a frequency header that the selected profile
does not emit.

`TC_PRINT` becomes `print_nothing` unless `CONFIG_ZTEST_VERBOSE_OUTPUT` is
enabled. It is not enabled in this default image. The earlier acceptance missed
that guard, so observing no frequency header is consistent with the selected
source/configuration; it is not evidence of a wrong timer rate. It still fails
the frozen output admission requirement and must not be silently waived after
collection.

The initial offline row inspection applied `\s*` across a multiline transcript.
It consumed newlines and treated alternate metric lines as preceding notes,
returning 24 matches. Applying the same row grammar separately to each line
recovers all 47 exact ordered names with numeric columns and empty notes in
the inspected completed consoles. Preserve the initial inspection reports and
the separately corrected replay. Correcting this parser error does not resolve
the missing header or the retained ROM `SHA-256 comparison failed` warning.

Physical acceptance remains separate from terminal suite completion. The
three observed consoles reach `PROJECT EXECUTION SUCCESSFUL`, while the frozen
frequency-header gate fails and the boot-time SHA warning remains an explicit
qualification issue. This is **0/3 qualified boots under the frozen contract**,
despite upstream suite-completion output.

| Boot | Ordered numeric metric rows, empty notes | Required 16 MHz header | ROM digest warning | Upstream success banner |
| --- | --- | --- | --- | --- |
| 01 | 47 | Absent | Present | Present |
| 02 | 47 | Absent | Present | Present |
| 03 | 47 | Absent | Present | Present |

The initial 24-row inspection is a host parsing failure; the missing promised
header is a source/configuration contract error; the ROM warning is a separately
recorded boot/image-integrity issue. They must not be merged into a claim that
the C6 kernel never ran, nor corrected into a qualification pass after seeing
the outputs. Any changed output or image profile requires a new explicit gate
and retained failed baseline. The later source audit supplies the missing
endpoint/correction mapping; it does not admit these failed captures as timing
comparisons or supply unprinted gross observations.

The same audit reproduces the ROM's calculated and expected SHA values directly
from the frozen image, and explains that the image-header frequency code aliases
40/80 MHz in this tool version. These findings narrow the image-format and
frequency claims; they neither waive the warning nor replace runtime readback.

For this Zephyr session, root preserved a newer pre-session baseline with
SHA-256 `863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
Its fresh prefix plus the previously verified tail independently matches the
candidate file; 1,993 changed bytes are confined to NVS sectors 0xb000 and
0xc000. All 31 frozen input digests still match. The final
[write log](../../results/zephyr-native-tests-2026-10-23/artifacts/zephyr-restore.log)
and [separate verification log](../../results/zephyr-native-tests-2026-10-23/artifacts/zephyr-restore-verify.log)
record restoration of all 8,388,608 bytes, a device-digest match and then an RTS
hard reset. This is retained evidence of root's operations, without a claim
that post-reset application persistence is immutable.

The separate Rust hello probe preserves an ancestor-workspace rejection, then
27 E0080 binding-layout errors with the first isolated host-tool selection,
then nine E0425 generated-devicetree lookup errors with the bounded older-Clang
variation. The [build record](../builds/zephyr-rust.md) and
[feasibility note](../research/zephyr-rust-feasibility.md) correctly state that
no final Rust ELF/image or board execution resulted. The reviewer checked the
diagnostics, generated input examples and all 48 build-archive digests; it did
not rerun or repair the builds. These failures are local integration evidence,
not proof that C6 Rust is impossible or that a native C suite qualifies Rust.
