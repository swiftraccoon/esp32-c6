# ESP-IDF FreeRTOS

Research checked: 2026-10-23. Status: **confirmed upstream ESP32-C6 support**. The [initial Rust std compute/observer pilot](../../results/initial-2026-10-22/README.md) passed five measured boots using IDF 5.5.6 and esp-idf-sys 0.37.2. The [clock-controlled message experiment](../../results/clocked-handoff-2026-10-23/README.md) adds five passing boots and 5000 checked Rust std channel round trips, with a 62 µs per-boot median.

The later [matched queue API study](../../results/idf-queue-api-2026-10-23/RESULTS.md)
passes **24/24 scheduled boots** with 24,000 checked round trips. With the same
IDF kernel, Rust threads and observed priorities 1/5, every boot's median is
**62.875 µs through std channels** or **19.9375 µs through native queues** at the
nominal raw-tick scale. These are complete instrumented API paths, not isolated
kernel-primitive costs or an OS ranking. Interference, IRQ response, networking
and independent clock calibration remain unqualified; see the
[independent result audit](../reviews/queue-api-results.md).

## What it is

Espressif's vendor SDK, with its FreeRTOS port providing the kernel. ESP-IDF and FreeRTOS are one candidate stack in this comparison. The C6 runs its application scheduler on one HP core; the LP coprocessor does not make it a dual-core SMP target. Current stable documentation identifies ESP-IDF v6.1, with the default IDF kernel based on FreeRTOS v10.5.1. The alternate Amazon SMP option is documented as available only on ESP32. Pin the actual SDK and Rust crate versions when building. [Kernel documentation](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/system/freertos.html)

## Useful links

- [Source and issues](https://github.com/espressif/esp-idf), [releases](https://github.com/espressif/esp-idf/releases), [C6 getting started](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/get-started/index.html).
- Rust: [esp-idf-template](https://github.com/esp-rs/esp-idf-template), [current Rust IDF monorepo](https://github.com/esp-rs/esp-idf), [esp-idf-svc](https://github.com/esp-rs/esp-idf/tree/master/esp-idf-svc), [esp-idf-hal](https://github.com/esp-rs/esp-idf/tree/master/esp-idf-hal), [esp-idf-sys](https://github.com/esp-rs/esp-idf/tree/master/esp-idf-sys), [Rust on ESP book](https://docs.espressif.com/projects/rust/book/).
- [Examples](https://github.com/espressif/esp-idf/tree/master/examples), [supported versions policy](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/versions.html).

## Support and boundaries

| Area | ESP32-C6 assessment |
|---|---|
| Scheduling | Preemptive tasks, priorities, queues, mutexes, notifications, timers; IDF starts the scheduler before application entry. IDF also creates background tasks, so benchmark the complete task set. [Kernel guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/system/freertos.html) |
| Drivers | Vendor APIs for GPIO, UART, SPI, I2C, I2S, ADC, PWM, RMT, counters, timers, DMA-related operations, TWAI/CAN, flash, temperature sensor and more. Presence of an SDK driver does not guarantee matching safe Rust wrappers. [C6 peripheral index](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/peripherals/index.html) |
| Network | Wi-Fi, ESP-NOW, lwIP sockets, HTTP/TLS and OpenThread. BLE has NimBLE/Bluedroid integration; C6 has no Bluetooth Classic radio. Thread, Zigbee and Matter are separate protocol/application layers. [Network index](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/network/index.html), [C6 datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.html) |
| Zigbee | Supported through the additional ESP Zigbee SDK. Current v2.x documentation explicitly describes its core as a proprietary prebuilt Zigbee implementation. Do not assume that plain IDF or a Rust wrapper includes it. [Zigbee SDK](https://docs.espressif.com/projects/esp-zigbee-sdk/en/latest/esp32c6/introduction.html) |
| Storage/update | Flash partitioning, NVS, filesystem components and OTA infrastructure. Application rollback, power-fail behavior and selected filesystem still need explicit validation. [Storage](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/storage/index.html), [OTA](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/system/ota.html) |
| Power/security | Dynamic frequency scaling, modem sleep, light/deep sleep; secure boot, flash encryption, crypto and memory protection support. These are product configuration capabilities, not proof of process isolation or enabled protections. [Power](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-guides/low-power-mode/index.html), [security](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/security/security.html) |
| Debugging | USB Serial/JTAG, OpenOCD/GDB and FreeRTOS-aware inspection. [C6 debugging](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-guides/jtag-debugging/index.html) |

## Rust fit

**Strongest initial Rust-with-`std` candidate.** The C6 target is `riscv32imac-esp-espidf`. `esp-idf-hal` supplies Rust driver abstractions, `esp-idf-svc` adds services, and `esp-idf-sys` supplies FFI for gaps. Rust application code still sits above a substantial C SDK and its FreeRTOS scheduler. Rust `std::thread` and async code therefore do not make this a Rust kernel. [Target and setup](https://github.com/esp-rs/esp-idf/tree/master/esp-idf-svc), [current wrapper compatibility notes](https://github.com/esp-rs/esp-idf/blob/master/esp-idf-svc/CHANGELOG.md)

The Rust crates now share the `esp-rs/esp-idf` monorepo; the old sys/svc
repositories are archived migration references. Its README describes the Rust
wrappers as a community effort with no current HIL tests, distinct from vendor
support for the underlying SDK. Our working sys 0.37.2 baseline is also older
than the sys 0.38 family required by hal 0.47 / svc 0.53. See the
[compatibility and maintenance audit](../research/platform-compatibility.md)
before adding wrappers or selecting an upgrade.

For this older sys 0.37.2 baseline, released HAL **0.46.2** already offers a
native queue wrapper with a compatible manifest. Its allocation/error and
handle-lifetime behavior still needs an adoption review. Task notifications
coalesce bits and are not a FIFO substitute. The measured narrow FFI adapter
does not establish a performance result for that HAL wrapper; see
[native Rust API options](../research/idf-native-api-options.md).

## Tradeoffs and benchmark value

- **Pros:** direct vendor ownership, broad C6 integration, useful production services, and a practical Rust application route. A good reference for how much custom development we would need to replace.
- **Costs:** Espressif-specific APIs, SDK and crate version coupling, C/FFI boundaries, background services, and shared radio scheduling. Measure latency with radios active as well as disabled.
- **Licensing/source availability:** original IDF code is Apache-2.0; FreeRTOS is MIT, with other component licenses. Wi-Fi/PHY repositories ship binary archives under Apache-2.0 in object form; source availability remains limited despite permissive redistribution. Treat Zigbee separately. [License inventory](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/COPYRIGHT.html), [Wi-Fi binary explanation](https://github.com/espressif/esp32-wifi-lib/blob/master/README.rst), [PHY binary explanation](https://github.com/espressif/esp-phy-lib/blob/master/README.rst)
- **Benchmark role:** baseline full stack; measure Rust `std` application footprint, IRQ-to-task wakeup, task notification/queue timing, periodic deadline misses under Wi-Fi traffic, reconnect/OTA behavior and sleep energy. If an FFI path distorts a kernel test, a minimal C shim is a proposal to discuss before implementing.

**Admission gate:** reproducible Rust boot, serial output, timer/GPIO operation and a frozen dependency manifest. Enable networking only after the no-radio baseline is repeatable.
