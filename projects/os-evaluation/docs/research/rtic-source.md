# RTIC source and ownership contracts

The selected upstream example's clock, interrupt and scheduling assumptions
bound the [qualification](../experiments/rtic-qualification.md).

## Source and reuse decision

The selected source is RTIC v2.4.0 commit
[`f1797184d6168e54c345b17019c237eea70b9ed4`](https://github.com/rtic-rs/rtic/tree/f1797184d6168e54c345b17019c237eea70b9ed4).
The local checkout is `.cache/rtic-qualification-2026-10-23/upstream`.
The in-tree C6 project and its tracked lockfile remain unmodified. Existing
[prior work](../../../../docs/research/prior-work.md) and
[reuse decisions](../../../../docs/research/benchmark-reuse.md) favor qualifying a known
upstream integration before adding another maintained workload.

The decision question is whether this pinned Rust/RTIC/PAC/HAL tuple boots and
executes its timer-woken example on our C6 through the established UART bridge.
The closest existing test is upstream's own `monotonic` example. Its reuse is a
local replication of integration behavior; it adds no scheduler performance
measurement. The C6 example package declares MIT OR Apache-2.0. Preserve the
upstream license files with retained sources.
[Project manifest](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/Cargo.toml).

## Selected example and observable contract

`monotonic.rs` creates a SYSTIMER monotonic, starts it once from `init`, and
spawns three async tasks. Its seven application lines are `init` and one
`hello from TASK` / `bye from TASK` pair for each of `foo`, `bar`, and `baz`.
The actual delays are **2, 3, and 4 seconds** respectively. Upstream's README
instead describes 1, 2, and 3 seconds; that prose does not govern this
qualification. Delays begin at each task's delay call, after its own entry
print, rather than at one shared reset epoch.
[Example source](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/examples/monotonic.rs),
[upstream README](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/README.md).

The prospective contract accepts any permutation of the three entry lines,
requires each entry before its corresponding exit, and expects exits in
`foo`, `bar`, `baz` order. This order is an observation required of this
lightweight fixture, not a general guarantee for arbitrary tasks with relative
delays: sufficiently delayed first polling could change relative deadlines.
The contract fixes three boots, a 45-second host ceiling from the reset
request, broad per-task host receipt windows, and an actual twelve-second guard
after the final exit. Archived read/poll receipts and raw byte offsets must
establish those observation windows. Ambiguous batching or an out-of-budget
result fails the stated method; it is not repaired by widening the window
after capture.

Completion would establish repeated startup, execution of all three task
bodies, and timer-dependent resumption for this image/fixture. UART receipt
gaps include logging, bridge, host buffering, and polling. They are pacing
checks, not device timer calibration or interrupt-latency measurements.
The complete raw stream must remain subject to fatal-diagnostic and additional
boot rejection, including the guard after the final expected application line.

## Scheduling and interrupt ownership

The three tasks omit `priority`; the pinned macro's software-task default is
**0**. With `dispatchers = []` and no explicit idle task, generated main calls
the priority-zero dispatcher, which repeatedly polls task executors. A
priority-zero wake marks the executor pending without pending a software
interrupt. Consequently this example exercises cooperative future polling
and timer-interrupt wakeups. It does **not** exercise preemption between
positive-priority software tasks. The dispatcher remains a busy loop after
all three tasks finish; the quiet console does not establish idle or sleep
residency.
[Default priority](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-macros/src/syntax/ast.rs),
[main generation](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-macros/src/codegen/main.rs),
[dispatcher generation](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-macros/src/codegen/async_dispatchers.rs).

RTIC disables interrupts before `init` and enables them after publishing the
returned resources. The C6 backend maps positive-priority task interrupts to
CPU interrupt numbers 20 through 31, preserving HAL's reserved range below
20. Its generated assertions accept task priorities **1 through 14**; the
assertion rejects a value greater than or equal to 15. Software dispatchers
must be `FROM_CPU_INTR0` through `FROM_CPU_INTR3`. The runtime resource lock
raises the PLIC threshold to the ceiling plus one, or uses a global critical
section for ceiling 15, and restores the prior threshold on normal return.
These are source assumptions for this backend; the empty shared-resource
example does not qualify ceiling locking, contention, or panic unwinding.
[Initialization](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-macros/src/codegen/pre_init.rs),
[resource publication](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-macros/src/codegen/post_init.rs),
[C6 bindings](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-macros/src/codegen/bindings/esp32c6.rs),
[C6 runtime](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic/src/export/riscv_esp32c6.rs).

## Monotonic clock and resource assumptions

The C6 monotonic consumes the PAC `SYSTIMER` token at `Mono::start`. It reads
unit 0, programs target 0, maps peripheral interrupt 57 to CPU interrupt 31
at priority 15, and exports `interrupt31`. Its type-level frequency is
**16 MHz**, giving a nominal 62.5 ns tick. Starting it twice or sharing its
timer, target, mapping, or CPU vector with another subsystem is outside this
example's ownership contract. Future integration must reserve those resources
explicitly; the general backend's available-vector list alone is insufficient
to prove coexistence.
[Pinned C6 monotonic](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-monotonics/src/esp32c6.rs).

The timer queue adds one tick to a nonzero relative delay so it waits at least
the requested duration according to its timebase. The timer interrupt clears
the compare flag, wakes due futures, and programs the next deadline. The C6
backend also directly invokes the timer handler with interrupts disabled when
inserting a new earliest timer, then enables interrupts. This specific example
first inserts delays while polling its priority-zero tasks after initialization;
this review does not generalize that behavior to insertion from arbitrary
interrupt-disabled contexts.
[Timer queue](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-time/src/timer_queue.rs).

The example imports HAL runtime support but **does not call `esp_hal::init`**.
Do not assign it HAL's default CPU frequency: CPU and peripheral state inherit
startup/bootloader configuration, without application readback. Neither the
type-level tick frequency nor the coarse console qualification establishes
clock accuracy. CPU utilization, sleep, radio integration, external IRQ
response and shared-peripheral behavior remain unmeasured.

Startup watchdog handling also depends on that omission. The pinned HAL's
`init` normally configures/disables RTC, super and timer-group watchdogs;
`monotonic` bypasses it. The inspected `esp-riscv-rt` 0.12.0 startup contains
no watchdog setup. IDF 5.5.6's default bootloader configuration enables a
9000 ms RTC watchdog and relies on IDF application startup to disable it.
This no-std application does not execute that IDF application initialization.
Those IDF defaults do not establish the configuration of a different
prebuilt bootloader. The actual selected bootloader must be archived and
identified; until its configuration or the physical observation resolves
this dependency, a watchdog reset during the guard remains a concrete
qualification risk, not an established outcome. Retain a failed unmodified
attempt before proposing any HAL-initialization change or shortened contract.
The prospective guard was strengthened from five to twelve seconds before any
input freeze or board attempt so this nominal startup-watchdog horizon would
not coincide with an observation ending near four plus five seconds. This is
a source-informed qualification boundary, not a claim that quiet UART proves
continued execution or that the watchdog configuration has been measured.
[Pinned HAL initialization](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/lib.rs),
[pinned RISC-V startup](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-riscv-rt/src/lib.rs),
[IDF bootloader settings](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/bootloader/Kconfig.projbuild),
[IDF application watchdog disable](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_system/startup_funcs.c).

## Console and unknown-carrier boundary

The manifest selects `esp-println` 0.15.0 with `auto`. In that exact dependency,
C6's latched USB SOF flag selects USB Serial/JTAG if a native USB host has been
seen, otherwise the ROM UART writer is used. The fixture therefore requires
the established CP2102N bridge at 115200 baud and no native USB host connection.
A missing UART transcript could be transport selection rather than an RTIC
execution failure; retain the failed attempt and diagnose it separately.
The dependency enables the println critical-section feature, so console work
also affects interrupt scheduling.
[Pinned printer source](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-println/src/lib.rs).

`monotonic` configures no application GPIO. The other unmodified example,
`sw_and_hw`, configures **GPIO9 as a pulled-up input with a falling-edge
interrupt**. Its tasks have priorities 5 (`foo`), 2 (`bar`), and 3 (GPIO),
so a button event is intended to wait during the high-priority task and
preempt the low-priority task. Without controlled physical stimulation and
event timing, its ordered task prints cannot establish that preemption.
Unknown carrier routing makes GPIO9 an additional fixture question; this
review selects no button operation or physical `sw_and_hw` trial.
[Hardware/software example](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/examples/sw_and_hw.rs).

## Build evidence and review limits

Both selected examples built unmodified with Rust **1.96.0**, release profile,
target `riscv32imac-unknown-none-elf` and feature `riscv-esp32c6-backend`.
The locked rebuild preserved the upstream lockfile. The
[qualification result](../../results/rtic-qualification-2026-10-23/RESULTS.md)
records the subsequent failed physical attempt and retained build identities.

The integration keeps upstream HAL **=1.0.0-rc.0**, PAC **0.22.0**,
`esp-backtrace` **0.17.0**, and `esp-println` **0.15.0**. Upstream specifies
the moving `stable` channel; the actual compiler above is the reproducible
choice for this attempt. The configuration forces frame pointers and links
`linkall.x`; neither is silently normalized to the IDF/Embassy images.
[Toolchain](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/rust-toolchain.toml),
[Cargo configuration](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/.cargo/config.toml),
[link setup](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/build.rs).

No new authored Rust API, unsafe block, or FFI adapter is introduced here, so
there is no new `C-*` implementation disposition. Upstream's unsafe register,
executor and interrupt code retains upstream conventions; successful builds
do not prove memory safety or concurrency correctness. The failed board result
retains its second-startup observation; the reset cause remains unresolved.
