# Qualification of Ariel's existing native scheduler tests

Design recorded **2026-10-23**, before test flashing. These are unmodified
upstream native tests at Ariel commit
[`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9),
using the previously qualified **Rust 1.99.1 / laze 0.1.39** input. Source/build
evidence is in [the build record](../builds/ariel-native-tests.md). This brief
does not itself establish an on-board pass.

## Decision, reuse and fixed collection budget

Can maintained Ariel native flag/yield and priority/lock tests execute on our
C6 through its existing UART route? The answer determines whether these native
operations provide a working foundation before authoring an Ariel-specific
benchmark adapter. This is qualification of existing behavior, not a matched
comparison against the IDF queue experiment or evidence for a new kernel.

The [prior-work audit](../../../../docs/research/prior-work.md) and
[reuse inventory](../../../../docs/research/benchmark-reuse.md) identify the exact upstream
tests. Run them unchanged under the repository's **MIT OR Apache-2.0** grant;
the app manifests inherit that license. Retain upstream license notices with
source snapshots. No source from the separately licensed/permission-unresolved
paper artifact is copied. No new firmware implementation or non-Rust adapter
is introduced.

Run **three scheduled boots per test, twelve total**, with a fixed **30-second
capture ceiling per boot** and at least **two seconds of observation after its
expected output**. Root alone flashes/captures; the existing espflash serial
monitor with PTY/script transcript is the transport. Preserve every attempt,
flash verification and rendered monitor output; no automatic retry replaces a
failure. These counts check repeated startup and completion, not population
statistics or tails.

Capture the full 30-second window without stopping early on a marker; a marker
arriving too late to leave the required two-second observation does not satisfy
the pass rule. The [frozen input archive](../../results/ariel-native-tests-2026-10-23/README.md)
records the operator's schedule, images and capture fixture.

For each boot require one fresh ROM startup followed by the expected completed
test line below, with no benchmark error, panic, assertion failure or additional
reset in the observed window. Numeric benchmark means must be positive integers;
the yield line's second number must equal integer division of its first by two.
Record timeout, missing output and reset separately. There is no device nonce,
per-event record or independent host challenge. The claim is bounded
host-controlled UART qualification, not the stronger frozen protocol-3 capture
contract used in the IDF experiment. These programs need not terminate after
printing their marker; quiet output alone is not additional progress evidence.

## Native tests and exact output meaning

| App | Work and priorities | Completed-output criterion |
| --- | --- | --- |
| `bench-sched-yield` | Two autostart threads at default priority **1**, each default stack **2,048 bytes**. Initiator measures **10,000** calls to `yield_same`; peer yields forever. | `took N ticks per iteration (M per context switch)` with `N > 0`, `M = floor(N/2)`. Preserve the upstream label; M is its derived aggregate estimate, not an independently measured switch latency. |
| `bench-sched-flags` | Two default-priority **1** autostart threads with 2,048-byte stacks exchange thread IDs using an Ariel `Channel`. Initiator measures **1,000** `set(peer,1); wait_any(1)` iterations; peer signals/waits forever. | `[INFO] took N ticks per iteration`, `N > 0`. This is flag signaling, with no checked sequence payload or per-event timing. |
| `threading-dynamic-prios` | Priorities **2** and **1** initially; assertions check the selected task IDs/priorities and execution order while one is raised to **5** then restored to **1**. | `[INFO] Test passed!` after every assertion in the expected sequence, followed by `exit(Success)`. |
| `threading-lock` | Four 2,048-byte-stack threads, priorities **1,2,3,2**. Assertions check initial execution and lock acquisition ordering; completion waits for flags `0b111`. | `[INFO] Test passed!` after the lock/order assertions, followed by `exit(Success)`. |

Higher numeric Ariel priority wins. Equal-priority threads cooperate; this
suite does not establish time slicing, general fairness or every preemption
case. The tests' [thread macro defaults](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-macros/src/thread.rs)
define priority 1 / 2 KiB stacks. Both benchmark initiators return after
printing, while their peers remain active. The assertion controls call
[`debug::exit`](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-debug/src/exit.rs),
which falls back to a spin loop without std/semihosting. A success marker does
not prove MCU shutdown. Panic prints diagnostics and takes the failure exit
path; preserve output and any subsequent reset.

The flag handshake can leave a startup-pending flag before the measured loop;
the first iteration is part of the upstream batch. The test does not prove
1,000 identical blocked-waiter handoffs. Flags are bit masks and coalesce;
they are not FIFO `u32` messages. No warmup, empty-observer measurement,
sequence checksum or raw batch-total output is added to these unmodified tests.

## Timing and configuration boundary

The [ESP benchmark implementation](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-bench/src/esp.rs)
reads HAL SYSTIMER **UNIT0** before and after the entire batch. It checks
subtraction and returns `total as usize / iterations`: **floor aggregate ticks
per iteration**, not CPU cycles, microseconds, individual samples or a
percentile. Logging follows the measured batch.

On the C6, `usize` is 32 bits. The cast occurs **before division**, so a delta
at or above `2^32` truncates without the 52-bit wrap error. At the nominal
16 MHz rate that threshold is about **268.435 seconds**; the 30-second host
ceiling is comfortably shorter. Hardware-counter backsteps are reported only
when the final value is below the first. No exhaustive monotonicity or observer
qualification follows from one aggregate result.

The selected HAL fork uses SYSTIMER at XTAL/2.5, nominal **16 MHz** for our
identified **40 MHz crystal**, and Ariel initializes CPU `CpuClock::max()`
(C6: 160 MHz). Retain printed integer ticks as the primary result. A nominal
conversion would be 0.0625 µs/tick; these unmodified tests do not emit runtime
CPU/flash/timer register snapshots or independently calibrate the oscillator.
Bootloader header messages alone do not prove final runtime flash frequency.

Retain the upstream release profile, fat LTO, one codegen unit, forced frame
pointers and default memory placement. The selected `single-core`,
`executor-thread`, `idle-threads`, `threading`, `log`, `logging-over-uart` and
panic-printing contexts are recorded in generated build commands; the two
benchmarks additionally enable `bench`. Radio features are not selected. The
existing [Ariel build record](../builds/ariel-os.md) identifies forked HAL and
Embassy inputs and the difference from the Rust 1.96 direct-stack baselines.

## Hardware and archive requirements

Use the confirmed chip/flash/UART identity in [hardware](../../../../docs/hardware.md), without
assuming the unknown carrier is an Espressif DevKitC. The selected upstream C6
builder routes **UART0 TX GPIO16 / RX GPIO17**; explicit UART/log selection
matches the CP2102N bridge and previous successful startup captures at **115200
8N1**. No LED, external peripheral, radio peer, JTAG or native USB assumption
is required.

Archive exact ELFs and merged DIO/80 MHz/8 MB images, source/lock/license
snapshots, build command/log, generated context flags and hashes before flashing.
Root must verify the preserved **current pre-session image** before writes,
retain esptool hash-verification output, and restore/digest-verify/reset that
current baseline after the qualification session. The historical original
remains preserved separately. Preflight found 3,979 changed bytes confined to
NVS sectors `0xa000`, `0xb000`, `0xc000` after the previous restoration/reset;
the cause is unproven. A fresh prefix overlay on the historical image produced
the current 8 MiB baseline, independently verified against the device's full
digest. Its private backup SHA-256 is
`ecc182713fa7df6b1e3841b5344f4c91528a5fef4e80a3ab37273ca4e89a661a`.
Restoring this baseline preserves the verified pre-session NVS state.
Source status/diff must remain clean. The new test
outputs belong in their own result category; do not alter historical datasets.

The [rev0.2 errata review](../../../../docs/research/rev02-errata-workarounds.md) still governs
later uses: these tests use typed flags/IDs and no arbitrary unaligned flash
copy workload, RMT or radios. The older HAL's ROM-copy behavior and slow-clock
inheritance remain unqualified. A passing native suite does not close power,
connected timer reliability, arbitrary copies or product readiness.

No raw per-event distribution, WCET, isolation claim, absolute oscillator
accuracy, API-versus-kernel attribution or cross-OS ranking is admitted by this
qualification. Failures narrow the usable native-test path and remain useful
evidence; they do not license silently rewriting the upstream test.
