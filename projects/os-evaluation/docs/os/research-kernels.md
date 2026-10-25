# Additional ESP32 C6 research systems

Checked 2026-10-22. These projects add useful architectural experiments to the survey. Their claims have not been reproduced on our board. Give each a separate build and code-review budget before comparing it with an established OS.

## project trobbio

[Repository and instructions](https://github.com/toyo54/project-trobbio) · [inspected revision](https://github.com/toyo54/project-trobbio/tree/f98e816108f70a73bcae68a1380bdc094cb3d20a) · [BSD-2-Clause license](https://github.com/toyo54/project-trobbio/blob/f98e816108f70a73bcae68a1380bdc094cb3d20a/LICENSE).

A teaching kernel specifically for C6, using Rust and assembly without ESP-IDF or esp-hal. It implements its own startup, traps, preemptive scheduler, GPIO, UART, timer and temperature-related scheduling. Its documented build uses stable Rust and `riscv32imac-unknown-none-elf`, with `cargo build --release` after setup. [Architecture and build](https://github.com/toyo54/project-trobbio/blob/f98e816108f70a73bcae68a1380bdc094cb3d20a/README.md)

**Value:** a concrete reference for learning how to own startup and scheduling. **Cost:** deliberately narrow scope; no established radio, storage, update or isolation coverage. Keep it in the custom-OS experiment group. Its thermal thresholds are raw sensor codes rather than calibrated Celsius, so align or disable that scheduling policy before timing comparisons. HP-only scheduling does not imply the chip lacks its separate LP core.

First gate: reproduce boot and timer operation, inspect register preservation and watchdog behavior, then measure handoff and footprint. No build was attempted here.

## ThistleOS

[Repository](https://github.com/wan0net/thistle-os) · [inspected revision](https://github.com/wan0net/thistle-os/tree/ccdd270a93f526cfb0144d9125566add0225a231) · [BSD-3-Clause license](https://github.com/wan0net/thistle-os/blob/ccdd270a93f526cfb0144d9125566add0225a231/LICENSE).

The project describes a beta Rust system for graphical devices, dynamic application/driver loading and recovery. Its architecture retains ESP-IDF and FreeRTOS underneath. It claims C6 architecture support; that claim is weaker than a reproduced build and boot for our [DevKitC-1-N8](../../../../docs/hardware.md). [Project scope](https://github.com/wan0net/thistle-os/blob/ccdd270a93f526cfb0144d9125566add0225a231/README.md)

**Value:** application loading and device-management ideas for a richer embedded environment. **Cost:** the primary hardware is an S3 device with much more memory and peripherals; runtime-loaded application assumptions need a C6 RAM audit. A Rust upper layer does not replace the underlying scheduler.

First gate: identify a C6-specific configuration and build artifact, confirm operation without assumed PSRAM/display/SD hardware, then qualify loading, recovery and footprint. Network or driver coverage on another board does not transfer automatically. No verified C6 build recipe is asserted here.

## OpenC6 BIOS

[Repository](https://github.com/Rompass/openc6-bios) · [inspected revision](https://github.com/Rompass/openc6-bios/tree/d980b1469eb77021ecd7bf9304cf2996204f9e30) · [MIT license](https://github.com/Rompass/openc6-bios/blob/d980b1469eb77021ecd7bf9304cf2996204f9e30/LICENSE).

A C6-focused firmware host with payload loading, a syscall interface, LP-core supervision, Wi-Fi management and update experiments. The project reports testing on an 8 MB C6-Zero; it retains ESP-IDF/FreeRTOS. Its published setup uses IDF and C/C++ tooling. [Architecture and setup](https://github.com/Rompass/openc6-bios/blob/d980b1469eb77021ecd7bf9304cf2996204f9e30/README.md)

**Value:** directly relevant LP-core supervision, recovery and modular-payload experiments. **Cost:** C/C++ integration conflicts with our application-language preference; its custom ABI and protection claims need independent validation. A rollback design still needs interrupted-update tests.

First gate: inspect the pinned board wiring, build configuration, partition layout, payload protection and actual task scheduler. Then test LP failure recovery and update behavior. Discuss any C/C++ harness before implementation. Treat marketing descriptions of isolation or recovery as hypotheses to test.

For another concrete experimental Rust scheduler and BLE adapter, see [MOS-Rust](mos-rust.md). For Tock and Hubris eligibility, see the [Rust support audit](../research/rust-support-audit.md).
