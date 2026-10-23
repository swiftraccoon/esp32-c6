# RT-Thread

Research checked: 2026-10-22. Status: **confirmed upstream C6 BSP, early and limited**. This is a separate RTOS kernel. Support is much narrower than the general RT-Thread feature catalog.

## Evidence and links

The [ESP32-C6-DevKitC-1 BSP PR #11836](https://github.com/RT-Thread/rt-thread/pull/11836) merged on 2026-09-28. The [current BSP README](https://github.com/RT-Thread/rt-thread/blob/master/bsp/ESP/ESP32_C6/README.md) documents actual C6 hardware bring-up. [Kernel source](https://github.com/RT-Thread/rt-thread), [documentation](https://www.rt-thread.io/document/site/), [license](https://github.com/RT-Thread/rt-thread/blob/master/LICENSE).

## What works on C6

The BSP reports GPIO, UART0 console, ADC1 and optional GDBStub. Its ADC is explicitly **uncalibrated**, including a large ground offset in the author's test. I2C, SPI, PWM, Wi-Fi, BLE and 802.15.4 remain marked pending. A reported SPI loopback did not match the transmitted byte and is not support evidence. These are upstream author reports, not measurements on our board. [BSP support table](https://github.com/RT-Thread/rt-thread/blob/master/bsp/ESP/ESP32_C6/README.md)

The target is a C6-WROOM-1 board with 8 MB flash, 40 MHz crystal and UART0 on GPIO16/17. Matching flash capacity alone does not establish that our carrier, pins or USB port match this BSP. The bootloader is C6-specific; a C3 bootloader must not be substituted. [Board instructions](https://github.com/RT-Thread/rt-thread/blob/master/bsp/ESP/ESP32_C6/README.md)

## Build and Rust considerations

Uses SCons/Env and Espressif's RISC-V GCC toolchain. The README currently recommends a specific auxiliary ESP-IDF fork commit (`4fa003074a73774369d38123749fe64b7b4acbd3`) rather than a moving branch. The [package sources PR #17](https://github.com/RT-Thread-packages/esp-idf/pull/17) subsequently merged on 2026-10-08, while the BSP README still warns that its normal package download lacks C6 sources. Resolve this documentation/package discrepancy with a clean build; do not assume package installation is sufficient.

There is no demonstrated turnkey Rust C6 BSP in these sources. Calling the kernel from Rust through FFI is plausible but would require a small integration project; this is an engineering inference. Discuss that work before choosing a C-heavy benchmark harness.

## Pros and cons

- **Pros:** another real preemptive kernel; upstream C6 entry and documented hardware behavior; Apache-2.0 core license; useful scheduler/memory comparison without radios.
- **Cons:** very recent C6 port, missing major buses and all listed radios, unusual package dependency path, and limited Rust integration evidence. The OS's generic networking/filesystem ecosystem does not establish C6 driver availability.

## Benchmark role and admission gate

Use as a second-wave **kernel-only** candidate: boot, tick continuity, task switching, queues, mutexes, footprint and long stress runs. Mark radio and unsupported-driver tests unavailable; zero throughput would falsely imply an implemented but slow stack. Gate admission on a clean pinned build, validated interrupt routing and timer stability. Driver completion work must be tracked separately from benchmark time.
