# Why qualify RTIC next

Research and source inspection **2026-10-23**, following the
[finite IDF load qualification](../../results/idf-load-qualification-2026-10-23/RESULTS.md).
The next decision is whether another maintained Rust execution model is locally
viable on this C6 before extending matched application load. This note supplies
source and fixture semantics; the separate
[experiment contract](../experiments/rtic-qualification.md) owns acceptance,
attempt count, host timing receipts and observation bounds.

## Decision value and selected input

Qualifying RTIC adds an untested candidate to the local evidence. Repeating
IDF statistics-on/off images could estimate the whole-image configuration effect
under a new comparative contract, but would not establish another candidate's
Rust, timer or interrupt viability. It can follow when that effect changes an
instrumentation choice. The current two qualification pilots cannot supply
that effect estimate.

RTIC master still resolves to
[`f1797184d6168e54c345b17019c237eea70b9ed4`](https://github.com/rtic-rs/rtic/tree/f1797184d6168e54c345b17019c237eea70b9ed4),
the inspected release commit for RTIC/macros **2.4.0**, monotonics/time
**3.0.0**, sync **1.6.0** and common **1.2.0**. Its actual
[C6 example manifest](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/Cargo.toml)
selects the chip-specific backend and SYSTIMER monotonic. Both examples now
compile locally, unmodified, with the installed **Rust 1.96.0** and unchanged
tracked upstream lockfile. See [executed build evidence](../builds/rtic.md).
This establishes compilation, not a board pass or a numeric minimum compiler.

The alternate native esp-rtos thread path still requires a different source
choice. Our published **0.4.0** source at
[`63e97be`](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/lib.rs)
does not export the development `ThreadSpawner` API. Current development at
[`4509a940`](https://github.com/esp-rs/esp-hal/blob/4509a940bb96a0017470ebc43ba0b163dfe9981f/esp-rtos/src/thread.rs)
does, while still advertising version0.4.0 and Rust1.95.0. Its
[manifest](https://github.com/esp-rs/esp-hal/blob/4509a940bb96a0017470ebc43ba0b163dfe9981f/esp-rtos/Cargo.toml)
uses HAL~1.2.0 and thread stacks include scheduler/guard bookkeeping beyond the
nominal stack size. That remains a useful later qualification, without silently
replacing the released Embassy baseline or treating the same package version
as the same API.

## Prior work and permitted reuse

The [prior-work inventory](prior-work.md),
[Rust scheduling audit](prior-work-rust-scheduling.md),
[reuse decisions](benchmark-reuse.md) and
[compatibility review](platform-compatibility.md) were consulted before the
build. Published RTIC/Embassy scheduling measurements on nRF52840 and STM32
motivate eventual interrupt/task endpoint decomposition; their timings do not
predict the C6. This phase reuses RTIC's maintained, explicitly C6 examples
instead of authoring another startup/timer demonstration.

The C6 package and upstream repository are MIT OR Apache-2.0. Retain upstream
license notices with copied source/artifacts. Running unmodified upstream Rust
and its existing runtime/assembly dependencies does not author a new assembly
adapter or require a language fallback. No benchmark implementation or source
port is added by these build probes.

## Transport and hardware assumptions

The locked **esp-println0.15.0** source is at
[`6b758d75`](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-println/src/lib.rs#L129-L183).
Its selected `auto` backend checks C6 USB-Serial/JTAG's SOF flag. If the flag
has been set, output uses native USB-Serial/JTAG; otherwise it uses the ROM UART
printer. It does not force UART merely because the collector opens a UART port.
The selected features also include critical-section printing; `no-op` is absent.

The initial physical route is the existing external USB/UART bridge at
**115200 baud**, with no active native USB connection. C6 UART0 IOMUX defaults
are **GPIO16 TX / GPIO17 RX**, as recorded in Espressif's
[pin definitions](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/include/soc/uart_pins.h#L14-L16).
The examples do not configure UART baud/pins; they rely on ROM/bootloader
console initialization. The selected bootloader and actual output remain part
of qualification. These SoC pins do not identify the carrier model or prove
every board trace. No GPIO9 wiring or button stimulus is established here.

## Monotonic: selected physical candidate

The [actual source](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/examples/monotonic.rs)
prints seven application lines:

1. `init`.
2. Exactly one each of `hello from foo`, `hello from bar`, `hello from baz`.
3. `bye from foo`, then `bye from bar`, then `bye from baz`.

Task-entry ordering is not an acceptance requirement. All hello lines must
precede the first bye. The source calls `Mono::delay` for **2, 3 and 4 seconds**
in foo/bar/baz respectively. The README's **1, 2 and 3 seconds** is inconsistent
with these pinned source bytes. No final `complete`, JSON record or user idle
message exists. Once the tasks finish, the generated priority-zero dispatcher
has no further application output. A prospective post-marker observation
window must therefore be timed and retained by the host, rather than invented
as a device completion record. The prospective contract now specifies a
**12-second post-final-marker guard**, within its 45-second capture ceiling.

The [C6 monotonic](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-monotonics/src/esp32c6.rs)
uses nominal **16MHz SYSTIMER** ticks and owns its timer/interrupt route.
Serial receive pacing can reject immediate/reordered/missing output and check
coarse timer progress. It cannot independently establish timer accuracy,
interrupt latency, task jitter, hardware utilization or real-time deadlines.
The image does not print runtime clock registers, counters or checksummed work.

The example does **not** call `esp_hal::init`. CPU, watchdog and other startup
conditions therefore cannot be inferred from the HAL defaults used by a
different example. In particular, the pinned HAL disables/configures watchdogs
inside its ordinary init function; that path is skipped here. The runtime's
ordinary entry path has no corresponding watchdog operation.

This omission matters to the post-marker guard. Espflash4.6.0's default C6
bootloader is generated from IDF release/v6.1; its recorded manifest contains
no watchdog-disable override. IDF
[watchdog defaults](https://github.com/espressif/esp-idf/blob/v6.1/components/bootloader/Kconfig.projbuild#L281-L315)
enable the startup RTC watchdog with a nominal9,000ms timeout. The normal
disable belongs to **IDF application startup** before app_main; a no_std RTIC
image does not run that startup. This is a credible late-reset risk for the
selected configuration, not an observed reset or proof every RTIC setup fails.
Source review motivated extending the proposed guard from 5 to 12 seconds before
any input freeze or physical capture, avoiding an endpoint close to the nominal
startup-watchdog boundary. The earlier 5-second diagnostic source remains
historical. Keep the full declared guard and retain any such failure. Even a
quiet 12-second guard establishes only survival during that observation window,
not disabled watchdogs or indefinite stability. Shortening the guard,
changing bootloader policy or adding HAL init would require an explicit new
configuration/contract rather than repair of the original attempt.

## Software/hardware example: build and source only

The [second example](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/examples/sw_and_hw.rs)
has high-priority foo5, lower-priority bar2 and a GPIO handler at3. Foo enqueues
bar, burns a50,000,000-iteration nop loop and exits; bar then burns the same
loop. The user idle function prints `idle` and loops forever. With no external
GPIO event, expected application markers are:

```text
init
Inside high prio task, press button now!
Leaving high prio task.
Inside low prio task, press button now!
Leaving low prio task.
idle
```

GPIO9 is configured pull-up/falling-edge and the handler clears the interrupt
then prints `button`. Without an identified safe carrier pin and controlled
stimulus, markers alone cannot establish physical IRQ preemption or response.
This image is excluded from the initial physical schedule. It does call HAL
Config::default(), whose source CPU default is80MHz; no register readback proves
the runtime frequency. Its loop timing is not a calibrated benchmark.

## Smallest useful deliverable and next extension

The selected deliverable is an unmodified monotonic build plus a bounded
console qualification with the frozen bootloader/image/lock and observable
host guard receipts. It extends the candidate feasibility matrix while
preserving every unsuccessful attempt. No matched performance or protection,
networking, peripheral breadth or reliability claim follows from success.

Once the execution path qualifies, reuse the existing checked integer workload,
nominal-release grid, accepted/full/missed conservation, conditional latency and
drain endpoints for a separately designed Rust load adapter. The existing
released Embassy/esp-rtos adapter is also ready for that extension. Priority
tasks, async executors and stackful IDF threads require explicit topology and
timer ownership; a shared payload alone does not make them identical kernel
operations. Radio integration, native esp-rtos development threads and any
statistics-image effect study remain separately named decisions.
