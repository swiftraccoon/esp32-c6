# Ariel OS startup qualification

**Two physical smoke tests passed:** the unmodified upstream async
`hello-world` and stackful-thread `hello-world-threading` examples each booted
and printed `[INFO] Hello World!` on the attached ESP32-C6. This dataset contains
startup/UART/autostart evidence, with no benchmark measurements or OS ranking.

| Example | Execution path | ELF bytes | Merged image bytes | Evidence |
| --- | --- | ---: | ---: | --- |
| `hello-world` | Async autostart task | 1484208 | 163328 | [Verified flash](hello-world/flash.log), [serial output](hello-world/monitor.log) |
| `hello-world-threading` | Stackful autostart thread | 1466212 | 163632 | [Verified flash](hello-world-threading/flash.log), [serial output](hello-world-threading/monitor.log) |

ELF sizes include debug data and are not flash-use scores. These single startup
observations do not establish long-duration stability, scheduler latency,
preemption, async/thread interoperability, timer accuracy, radio functionality,
peripheral coverage, memory high-water marks, power or deadline behavior.

The upstream source is commit
[`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9),
Ariel workspace version 0.6.0. Independent review confirmed a clean upstream
working tree. The selected board configuration is
`espressif-esp32-c6-devkitc-1`; that selection does not prove the attached board
is that model. The inspected device is ESP32-C6 revision 0.2 with 8 MB flash
and a 40 MHz crystal.

The checkout requires Rust 1.98. The attempted Rust 1.96.0 build failed the
declared compiler requirement; stable **Rust 1.99.1** and **laze 0.1.39** built
the examples. Rust 1.98 itself was not tested. The archived
[MSRV failure](artifacts/build-rust-1.96-failed.log),
[default-transport async build](artifacts/build-rust-1.99-default.log), and
[final UART build](artifacts/build-rust-1.99-uart.log) preserve that distinction.
Only the final UART/log images were flashed. The lockfile and generated
[Ninja build commands](artifacts/build-global.ninja) preserve exact dependencies
and compiler flags. Compiler/HAL versions, fat LTO and forced frame pointers
differ from the controlled benchmark baseline, so these images are not
performance comparisons.

The successful build command, run from the evaluation repository root, was:

```sh
env RUSTUP_TOOLCHAIN=1.99.1 PATH="$PWD/.tools/bin:$PATH" \
  .tools/bin/laze -C .cache/ariel-os/examples -g build \
  -b espressif-esp32-c6-devkitc-1 \
  -a hello-world,hello-world-threading \
  -s logging-over-uart,log -D CARGO_ARGS=--locked
```

The root hardware operator used espflash 4.6.0 to merge each archived ELF into
a DIO / 80 MHz / 8 MB image with 40 MHz crystal setting, then esptool 5.1.0
to write it at offset 0 using 115200 baud. Both flash logs report
`Hash of data verified` and leave the device in bootloader mode. Each separate
espflash serial monitor connection/reset at 115200 baud captured the ROM boot,
bootloader output and the expected hello line; the operator stopped monitoring
with Ctrl-C after that observation. The ROM logs report reset reason `0x1`
(`POWERON`). These are monitor/reset sessions, without a claimed physical
power-cycle count. Exact argument vectors are in [provenance.json](provenance.json).

The bootloader reports DIO / 80 MHz / 8 MB. These hello applications do not
read back the final runtime CPU and SPI0 flash clock registers, so boot text
does not establish matched final clocks. Both upstream applications log once
and call `exit(ExitCode::SUCCESS)`; seeing the line proves the path reached
UART logging, without independently testing behavior after the exit call.

Before either qualification image was flashed, the hardware owner verified
the full original **8 MiB** device image against the ignored local backup,
SHA-256 `13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`.
The backup is deliberately outside this dataset. After the enclosing controlled
experiment and alignment probe, the full original image was restored,
independently digest-verified against the device and hard-reset into the original
application. The [restoration record](../clocked-handoff-2026-10-23/device-state.json)
and its linked transfer logs retain the evidence.

[SHA256SUMS](SHA256SUMS) records the collection-time identities of source,
lockfiles, build/tool logs, generated files, ELFs, images, monitor/flash logs,
this README and provenance. Compiled artifacts are outside this checkout;
the historical manifest does not verify the complete current checkout.
Serial monitor transcripts are unchanged. See the
[snapshot review](../../tools/benchctl/docs/api.md) for available artifacts
and current verification commands.
The [build review](../../docs/builds/ariel-os.md)
records dependency forks, toolchain details and further limits.
