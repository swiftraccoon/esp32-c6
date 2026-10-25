# Zephyr

Research checked: **2026-10-23**. Status: **native C6 suite built and emitted its named output, but 0/3 boots qualify under frozen v1; the selected Rust hello failed its build gate**. The [native results](../../results/zephyr-native-tests-2026-10-23/RESULTS.md) and [Rust archive](../../results/zephyr-rust-build-2026-10-23/README.md) preserve the actual attempts. These failures do not establish that every C6 Zephyr/Rust combination is unsupported.

## Identity and references

- [Project](https://www.zephyrproject.org/) · [GitHub](https://github.com/zephyrproject-rtos/zephyr) · [Documentation](https://docs.zephyrproject.org/latest/) · [Issues](https://github.com/zephyrproject-rtos/zephyr/issues).
- Core license: [Apache-2.0](https://github.com/zephyrproject-rtos/zephyr/blob/main/LICENSE). HALs, radio binaries, and other dependencies have their own licenses.
- [Release policy](https://docs.zephyrproject.org/latest/releases/index.html): stable and LTS tracks exist. The live release page identifies 4.4.0 as latest stable and 3.7.0 as LTS; `latest` documentation describes development code. Choose and pin a release plus its west module manifest before benchmarking; do not assume every feature below exists in an older LTS.

## What actually runs on C6

The upstream board page marks [`esp32c6_devkitc`](https://docs.zephyrproject.org/latest/boards/espressif/esp32c6_devkitc/doc/index.html) maintained. Its hardware target is **`esp32c6_devkitc/esp32c6/hpcore`**. The **ESP32-C6-DevKitC-1-N8** matches this board family and the observed 8 MB flash. Use the [documented port/pin map](../../../../docs/hardware.md) and record PCB revision and selected console before new runs; native USB and other newly identified fixtures remain unqualified. [XIAO ESP32C6](https://docs.zephyrproject.org/latest/boards/seeed/xiao_esp32c6/doc/index.html) has a separate definition.

| Area | C6 evidence and boundary |
| --- | --- |
| CPU | HP RISC-V target; LP-core integration through sysbuild is documented. This is not symmetric two-core scheduling. |
| Basic I/O | Board table lists GPIO, UART/USB serial, SPI, I2C, ADC, timers, watchdog, PWM, CAN/TWAI, I2S, DMA, RMT, and pulse counter. A listed device still needs appropriate configuration and pin assignment. |
| Wi-Fi | C6 Wi-Fi device listed; Espressif RF binaries are required. |
| BLE | Espressif Bluetooth HCI device listed; validate desired host features and concurrent radio operation separately. |
| IEEE 802.15.4 | Native C6 device listed. |

These rows come from the [C6 supported-features table and blob requirements](https://docs.zephyrproject.org/latest/boards/espressif/esp32c6_devkitc/doc/index.html#supported-features), rather than from the chip's advertised hardware features.

Zephyr integrates [OpenThread and a native IEEE 802.15.4 stack](https://docs.zephyrproject.org/latest/doxygen/html/group__ieee802154.html). Treat C6 Thread throughput, commissioning, sleepy-device behavior, and coexistence as separate validation gates. A radio driver entry does not establish certification, Zigbee support, or a working Matter product.

## Rust reality

Official [Rust integration](https://docs.zephyrproject.org/latest/develop/languages/rust/index.html) builds a Rust static library into a C-based Zephyr image through the optional `zephyr-lang-rust` module, Cargo, CMake, and `CONFIG_RUST`. It provides Rust APIs over OS facilities; this is not a Rust kernel or an `esp-hal` application.

The actual probe uses Zephyr **4.4.0**, its pinned Rust module
`dd73abc242e995784da62352fe8c70d9a6c7ac2e`, Rust **1.96.0** and the module's
`riscv32i-unknown-none-elf` selection. Three attempts retain: ancestor-workspace
rejection, **27 E0080** generated-layout errors, then **nine E0425** missing
`get_instance_raw` errors after a bounded clang/libclang variation. No Rust
ELF/image or board pass was produced. The [source review](../research/zephyr-rust-feasibility.md)
identifies a generator/device-availability lead; no remedy has been qualified.
Prove a new build, ABI/link and UART gate before adding a benchmark adapter.
Discuss a necessary authored C shim separately; these attempts reuse upstream
code without changing the Rust language preference.

## Why evaluate it

**Pros, assessment:** useful portability across MCU vendors; broad OS/network services; board configuration separates application logic from hardware; release/LTS policy helps maintenance planning. Best fit: connected products that may move beyond Espressif and need several drivers or protocols.

**Cons, assessment:** west/CMake/Kconfig/devicetree and module revisions add build complexity; Rust is less direct than the native Espressif Rust ecosystem; feature-rich configurations can consume memory and complicate performance attribution. Radio binaries remain a dependency even with application code in Rust.

## Executed native baseline and failed qualification

The unmodified native latency suite built with Zephyr **4.4.0**, SDK **1.0.1**
and GCC **14.3.0**; see the [build record](../builds/zephyr-native-tests.md).
Every physical attempt emits all **47** expected numeric rows and an upstream
success banner, but **0/3 qualify**: the required frequency header is disabled
by `TC_PRINT` configuration and absent from the ELF; all three ROM logs report
an image-SHA comparison failure. Transfer verification is a separate check.
The [source audit](../research/zephyr-native-audit.md) reproduces the image hash
behavior and maps all rows: **19 observer-adjusted, 28 unadjusted**, with distinct
endpoints. Printed “cycles” are nominal 16 MHz SYSTIMER ticks, not CPU cycles.

The corrected per-line parser finds all 47 rows rather than the initial false
count of 24. It changes neither frozen admission nor the original reports.
No replacement image/profile has been collected. Resolve image/output gates
and version a prospective contract before another native timing comparison.

## Benchmark and lifecycle questions

- Compare minimal kernel, Wi-Fi, BLE, and 802.15.4 images independently; record flash/RAM, idle current, interrupt latency, periodic wake jitter, and useful payload throughput.
- Include build/configuration effort and Rust integration effort in the score, alongside runtime results.
- Separate Simple Boot from production boot: the [C6 boot documentation](https://docs.zephyrproject.org/latest/boards/espressif/esp32c6_devkitc/doc/index.html#simple-boot) says Simple Boot omits security features and OTA; MCUboot is the alternate path. Benchmark signed boot/update recovery separately, with the actual partition layout.
- Do not infer process isolation or power numbers from generic Zephyr capabilities. Establish C6-specific configuration, drivers, and recovery behavior on hardware.

Initial research snapshots were Zephyr [`76280bce`](https://github.com/zephyrproject-rtos/zephyr/tree/76280bce2a050376d57e14240687b5c283dea0e7)
and Rust module `6823b690`; the later attempt archives identify their different,
exact measured/build-probed pins. No overall OS, utilization or product result
follows from either snapshot or these failed gates.
