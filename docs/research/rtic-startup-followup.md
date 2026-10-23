# RTIC startup failure: bounded follow-up

Researched 2026-10-23 after the first physical v1 attempt. This is read-only
source investigation and a prospective follow-up recommendation. It changes
no v1 source, image, contract, raw capture or outcome, and starts no new board
experiment.

## What the failed attempt establishes

The [retained raw stream](../../results/rtic-qualification-2026-10-23/runs/01/serial.raw)
contains one initial startup, all seven expected application markers, then a
second `ESP-ROM:esp32c6-20220919` banner. The
[receipt stream](../../results/rtic-qualification-2026-10-23/runs/01/observations.jsonl)
records that second-banner chunk at **8,119,346,209 ns** from the host
RTS-release request. The chunk ends with the first `B` of the following output;
it contains no second reset-reason line. Capture permanently rejected the
additional startup and retained a
[failed report](../../results/rtic-qualification-2026-10-23/runs/01/run.json).
Scheduled attempts 2 and 3 were not run under the stop rule.

This is evidence of an additional startup/reset during the required observation
window. It does not identify RTC watchdog, another watchdog, brownout, external
reset or another reset mechanism. The first startup's `rst:0x1 (POWERON)`
belongs to that first startup and cannot explain the second one. Nor is the
8.119-second host receipt time the device watchdog's calibrated elapsed time.
The seven markers are useful partial execution evidence; they do not turn the
failed boot into a qualification pass or establish that RTIC is universally
incompatible with the C6.

## Why the startup watchdog remains a leading hypothesis

The unmodified
[monotonic example](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/examples/monotonic.rs)
starts its PAC SYSTIMER monotonic but never calls `esp_hal::init` or explicitly
feeds/disables an RTC watchdog. Its pinned RISC-V startup initializes execution
and interrupt routing, without the IDF application startup path.
[Pinned runtime](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-riscv-rt/src/lib.rs).

The selected bootloader's initial banner reports
`v6.1-beta1-497-g14f663f003e`. That revision resolves to
[`14f663f003eb8fd9a688c301a412a9540d29dacf`](https://github.com/espressif/esp-idf/commit/14f663f003eb8fd9a688c301a412a9540d29dacf).
It is a stronger source lead than the resource manifest's moving
`release/v6.1`, but does not recover the complete generated sdkconfig or prove
a clean/reproducible source build. The archived resource manifest contains no
watchdog-disable override. Its exact selected bytes and metadata correction
remain in the [v1 provenance](../../results/rtic-qualification-2026-10-23/provenance.json).

At that IDF revision, `BOOTLOADER_WDT_ENABLE` defaults on and
`BOOTLOADER_WDT_TIME_MS` defaults to 9000. Bootloader configuration programs and
enables RTC WDT stage 0 with a reset action. Normal cleanup is in IDF
application initialization before `app_main`; this no-std RTIC image does not
execute that cleanup. The source therefore predicts a credible startup-policy
mismatch. Source defaults plus a nearby reset time do not confirm the actual
reset cause or the running watchdog configuration.
[Pinned settings](https://github.com/espressif/esp-idf/blob/14f663f003eb8fd9a688c301a412a9540d29dacf/components/bootloader/Kconfig.projbuild#L250-L286),
[watchdog setup](https://github.com/espressif/esp-idf/blob/14f663f003eb8fd9a688c301a412a9540d29dacf/components/bootloader_support/src/bootloader_init.c#L70-L102),
[IDF application cleanup](https://github.com/espressif/esp-idf/blob/14f663f003eb8fd9a688c301a412a9540d29dacf/components/esp_system/startup_funcs.c#L122-L131).

The C6 bootloader also configures super-watchdog auto-feed and analog reset
behavior, including brownout. Several reset paths exist; one unexplained
banner cannot distinguish them.
[C6 startup](https://github.com/espressif/esp-idf/blob/14f663f003eb8fd9a688c301a412a9540d29dacf/components/bootloader_support/src/esp32c6/bootloader_esp32c6.c#L71-L106).

## Small Rust interventions and what they would test

All options below require a new prospective configuration/contract. None is
implemented or demonstrated by this note.

| Option | Source-supported mechanism | Interpretation and cost |
| --- | --- | --- |
| Narrow RTC-WDT diagnostic | The pinned HAL exposes safe `rtc_cntl::Rwdt::new` and `disable`. On C6, disable removes write protection, clears LP_WDT `wdtconfig0`, then restores protection. | Smallest authored Rust change for testing the RTC-WDT hypothesis. It avoids introducing custom register-write/FFI code or changing all HAL initialization policy. Actual pre/post register readback and initialization-context ownership still need review and qualification. |
| Ordinary HAL initialization | Call the existing `esp_hal::init(Config::default())` once, before starting Mono. The default WatchdogConfig disables reset, super and supported timer-group watchdogs. | A natural application integration route, but a broader intervention. If it survives, that alone cannot attribute the original reset specifically to RTC WDT. |
| Rebuild the upstream bootloader with an explicit watchdog policy | A pinned upstream IDF build with `BOOTLOADER_WDT_ENABLE=n` omits enabling the startup RTC watchdog after flashboot protection is cleared. | Keeps the application ELF unchanged, but changes bootloader startup/recovery policy and adds SDK build/configuration provenance. It qualifies a different application/bootloader combination. No authored C implementation is needed. |

[Pinned HAL watchdog operations](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/rtc_cntl/mod.rs#L925-L1040),
[default watchdog configuration](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/config.rs#L37-L68),
[HAL initialization](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/lib.rs#L629-L711).

The narrow driver has no peripheral-token constructor parameter and performs
its maintained register operations internally. That does not authorize sharing
it with another driver or concurrently accessing the same registers through
the PAC. A proposed diagnostic would run once during interrupt-disabled RTIC
initialization, retain raw register state before and after, and leave no other
watchdog writer active. HAL's ordinary init explicitly initializes the RTC
domain before watchdog access; the narrow path must verify its inherited
domain/access prerequisites rather than infer them from a safe constructor.

Changing `BOOTLOADER_WDT_DISABLE_IN_USER_CODE` to `y` alone does not solve this
image's missing cleanup: it places responsibility on application code, which
this RTIC image already effectively has. Merely increasing the timeout or
shortening observation could move a reset outside the window and would not
establish correct startup handling.

## HAL initialization is not a drop-in ownership proof

The pinned HAL and PAC expose distinct singleton types and acquisition state.
RTIC supplies `cx.device.SYSTIMER` as PAC **esp32c6 0.22.0** ownership;
`Mono::start` requires that PAC type. HAL init returns a separate
`esp_hal::peripherals::SYSTIMER` token, not the type Mono accepts. Adding HAL
init must explicitly retire unused duplicate tokens and keep the PAC monotonic
as the only active SYSTIMER owner. Do not add another `steal` or silently
retain both a HAL timer driver and the PAC monotonic.
[HAL singleton definitions](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/peripherals.rs),
[PAC ownership](https://github.com/esp-rs/esp-pacs/blob/4b7361c83c13672ed86b8c901ab39ddd21faff63/esp32c6/src/lib.rs),
[Mono start and routing](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic-monotonics/src/esp32c6.rs).

HAL init changes APM access filters, peripheral enable policy, RTC power/clock
configuration and clock bookkeeping, in addition to watchdog policy. RTC slow
clock calibration can loop until a nonzero result. Its default CPU setting
records an 80 MHz assumption; the C6 clock code skips reprogramming when the
requested setting equals that default. A metadata default is still not runtime
clock readback. These changes belong in a new image's provenance and should
not be described as measured scheduler cost.
[C6 RTC initialization](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/rtc_cntl/rtc/esp32c6.rs#L1105-L1160),
[C6 clock setup](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/clock/mod.rs#L494-L529).

`system::disable_peripherals` keeps UART0 and SYSTIMER enabled, which supports
initializing HAL before starting Mono. HAL init does not rerun the runtime's
whole interrupt-vector setup. Its default GPIO-handler binding checks for an
existing RTIC/custom handler or route and avoids replacing one; it can still
add its default route when no handler exists. Mono must continue to own CPU
interrupt 31 at priority 15 and its SYSTIMER target. A compile/link and mapping
review must precede physical v2 qualification; this source inspection is not
that qualification.
[Peripheral policy](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/system.rs#L126-L140),
[GPIO binding guard](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/gpio/interrupt.rs#L91-L117).

A custom bootloader alternative must also preserve the old HAL's MMU/page and
flash image compatibility, actual C6/8 MB hardware settings and corrected image
metadata. Freeze its complete sdkconfig, source/tool versions, original/final
bytes and image checks. Replacing a bootloader is a distinct integration cost,
not evidence that another kernel is required.

## Evidence a future diagnostic and v2 would need

First improve attribution without converting a failed boot into a pass. A
separately versioned diagnostic collector could preserve the first rejection
and continue a fixed short capture window solely to retain the remainder of
the second ROM/reset-reason output. It must remain failed even if favorable
markers follow. The current v1 stream stops on rejection; its missing second
reason cannot be reconstructed from a restored board or invented from timing.

For a diagnostic image, record reset cause and watchdog configuration before
initialization changes it. The pinned HAL's `reset_reason(Cpu::ProCpu)` provides
an optional typed cause; its mapping includes RTC, main and super watchdog,
brownout and software-reset variants. Preserve raw cause/register values when
available and keep unknown values explicit. Match the observed second-boot
cause with early LP_WDT enable/stage/timeout state and the selected bootloader's
actual configuration. If ambiguity remains, independent EN/power observations
can distinguish an external reset/power event from an internally requested
reset. No such observation exists for v1.
[HAL reset-cause API](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/rtc_cntl/mod.rs#L1198-L1203),
[C6 reset-cause mapping](https://github.com/esp-rs/esp-hal/blob/6b758d75b878d3ebf671a89c28de787551bdbdbc/esp-hal/src/rtc_cntl/rtc/esp32c6.rs#L1224-L1265).

If the narrow watchdog intervention is chosen, keep the bootloader, dependency
tuple and task bodies fixed; record the exact Rust delta and pre/post watchdog
state. A successful new observation would support that explicit startup
configuration. Confirming RTC WDT as the original cause needs the reset/state
evidence above, not success after a broad HAL-init change alone.

A prospective **v2** must name the chosen intervention, revised expected
diagnostic lines, complete timer/peripheral ownership, failure policy, fixed
attempt schedule and observation window extending beyond the suspected startup
horizon. Qualify its actual collector with passing and deliberately failing
controls, freeze new artifacts before collection, retain any failures and
verify restoration separately. If an extended run or deliberate watchdog
challenge is needed, declare its bounds and expected rejection prospectively.
Passing v2 would establish this revised startup route during its bounded
window. Preemption, matched workloads, hardware utilization, radios, memory
peaks, energy and deployed recovery would remain separate objectives.
