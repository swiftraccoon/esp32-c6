# Embassy with esp-hal

Research checked: **2026-10-22**. Status: **first-round Rust candidate**. The [initial physical compute/observer pilot](../../results/initial-2026-10-22/README.md) passed five measured boots of the Embassy/esp-rtos stack. The [clock-controlled channel experiment](../../results/clocked-handoff-2026-10-23/README.md) adds five passing boots and 5000 checked round trips, with a 7 µs per-boot median for the selected cooperative task/channel path. Networking, priority interference and IRQ response remain unmeasured.

## What it is and why test it

Embassy supplies Rust async executors, timers, synchronization and networking components. Its executor is not a complete conventional operating system. On the C6, the proposed runnable stack is **Embassy + esp-hal + esp-rtos**, with esp-radio added for wireless tests. The current Embassy book explicitly places the ESP time driver in esp-rtos. Consequently an “Embassy” result must identify its runtime and cannot be counted as an independent kernel comparison against the same esp-rtos configuration. [Embassy book](https://embassy.dev/book/), [esp-rtos sheet](esp-rtos.md).

## Evidence and links

| Item | Reference |
| --- | --- |
| Code / licensing | [Embassy repository](https://github.com/embassy-rs/embassy), MIT OR Apache-2.0; [esp-hal repository](https://github.com/esp-rs/esp-hal), MIT OR Apache-2.0 |
| Exact C6 target | `riscv32imac-unknown-none-elf`, `esp-hal` feature `esp32c6`; [supported target table](https://docs.rs/crate/esp-hal/latest) |
| Chip-specific driver docs | [esp-hal 1.2.0 for ESP32-C6](https://docs.espressif.com/projects/rust/esp-hal/1.2.0/esp32c6/esp_hal/index.html) |
| Project bootstrap | [esp-generate](https://github.com/esp-rs/esp-generate) explicitly includes C6; select chip and Embassy options |
| Bluetooth host | [TrouBLE ESP examples](https://github.com/embassy-rs/trouble/tree/main/examples/esp32) |
| Shared radio and toolchain caveats | [Rust support audit](../research/rust-support-audit.md) |

## Capability boundaries

The executor statically allocates async tasks and does not itself require a heap. Within one executor a task must yield for its peers to run; multiple executor priorities permit preemption between executors. A long CPU calculation inside one async task therefore needs explicit design attention. Wireless components can still require heap memory and preemptive driver threads. [Executor design](https://embassy.dev/book/), [esp-rtos API source](https://github.com/esp-rs/esp-hal/blob/main/esp-rtos/src/lib.rs).

GPIO, I2C, SPI and UART are directly available through esp-hal; DMA, crypto accelerators, low-power controls and other less stable drivers must be checked against the exact C6 documentation and features. Wi-Fi, BLE, ESP-NOW and raw 802.15.4 drivers exist, with important coexistence restrictions; Thread uses an additional OpenThread binding and C++ library. A generic USB device capability must not be inferred from USB Serial/JTAG. [C6 HAL docs](https://docs.espressif.com/projects/rust/esp-hal/1.2.0/esp32c6/esp_hal/index.html), [radio details](../research/rust-support-audit.md).

## Strengths and costs

- **Strengths:** Rust throughout application logic; explicit peripheral ownership; compact async concurrency; strong embedded crate interoperability; a useful lower-level baseline for judging Ariel OS's additional integration.
- **Costs:** we assemble provisioning, updates, persistence and recovery; cooperative task behavior can distort deadline tests; radio and advanced HAL APIs evolve; Rust application safety does not cover vendor binary drivers or FFI.

These are project-fit assessments, not measured performance claims.

## Build entry and experiment

After selecting and pinning a compatible generator/toolchain release, its current documented CLI can generate an initial scaffold:

```sh
esp-generate --headless -o esp32c6 -o embassy bench-embassy
cd bench-embassy
cargo build --release
```

This is a **documented starting route, not a build executed here**. Inspect `esp-generate list-options`, the generated Cargo features and lockfile before adding Wi-Fi. Current generator syntax differs from older `--chip` tutorials. [Generator quickstart](https://github.com/esp-rs/esp-generate).

Measure idle and one-task footprint, wake latency, timer jitter, channel handoff, and the memory growth from 1/8/32 waiting tasks. Add a deliberately long CPU task to expose cooperative scheduling limits. Report radio-disabled and radio-enabled variants separately, including all driver threads and buffers. Keep HAL/radio versions aligned with Ariel where feasible, and label unavoidable differences.
