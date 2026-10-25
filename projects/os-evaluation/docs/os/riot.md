# RIOT

Research checked: **2026-10-22**. Status: **upstream ESP32-C6 support; useful low-power networking comparison; current RISC-V Rust integration is disabled**. No firmware was built or flashed and no performance was measured.

## Identity and references

- [Project](https://www.riot-os.org/) · [GitHub](https://github.com/RIOT-OS/RIOT) · [Guide](https://guide.riot-os.org/) · [API](https://api.riot-os.org/) · [Issues](https://github.com/RIOT-OS/RIOT/issues) · [Releases](https://github.com/RIOT-OS/RIOT/releases).
- Core license: [LGPL-2.1](https://github.com/RIOT-OS/RIOT/blob/master/LICENSE); inspect individual file/package notices. Record licensing obligations as part of a product decision.

## What actually runs on C6

The [`esp32c6-devkit` board](https://guides.riot-os.org/boards/esp32c6-devkit/) supports DevKitC-1/WROOM and DevKitM-1/MINI boards. Its default pin setup exposes ADC, SPI, I2C, UART, PWM, and WS2812 RGB LED. Several functions share pins; combining drivers requires checking the mapping. The **ESP32-C6-DevKitC-1-N8** uses the DevKitC/WROOM board family. Select the DevKitC/WROOM configuration, then verify its console and pin assignments against the [board inventory](../../../../docs/hardware.md); this identification does not establish a RIOT build or board pass.

| Area | C6 evidence and boundary |
| --- | --- |
| CPU | [`cpu/esp32/Makefile.features`](https://github.com/RIOT-OS/RIOT/blob/master/cpu/esp32/Makefile.features) maps C6 to RV32IMAC. Treat the LP core as a separate unverified capability. |
| Wi-Fi | ESP32 port documents station/SoftAP and ESP-NOW interfaces; the C6 capability table reports Wi-Fi support. |
| BLE | CPU features explicitly advertise `ble_nimble`, `esp_ble`, extended advertising, and 2 Mbit PHY for C6. Desired profiles and coexistence need testing. |
| IEEE 802.15.4 | CPU features advertise `esp_ieee802154`; driver integrates through RIOT's 802.15.4 HAL/SubMAC. |
| Peripheral/crypto limits | C6 table reports ADC, GPIO, timers, I2C, SPI, UART, CAN, LED PWM; motor PWM and general hardware crypto acceleration are marked unsupported. |

The [ESP32 port guide](https://guide.riot-os.org/cpus/esp32/) documents those interfaces, layered power management, flash/VFS support, and ESP-IDF as a build-time dependency. **Flash encryption is explicitly unsupported.** Its chip-level current figures are documentation data, not measurements of our board or application.

RIOT has an [OpenThread integration package](https://github.com/RIOT-OS/RIOT/tree/master/pkg/openthread), but that alone does not demonstrate a working C6 Thread product. First prove the selected package, board, radio driver, and role together. Distinguish native GNRC/6LoWPAN networking from Thread, and both from Zigbee/Matter.

## Rust support blocker

RIOT's [Rust guide](https://guides.riot-os.org/rust_tutorials/rust_in_riot/) describes Rust apps linked into the C OS using `riot-sys` and safe `riot-wrappers`, plus nightly Rust, bindgen, and patched C2Rust. Its [wrappers](https://github.com/RIOT-OS/rust-riot-wrappers) are maintained by the project; some expose familiar embedded-hal traits.

However, the verified [`cpu/riscv_common/Makefile.features` snapshot](https://github.com/RIOT-OS/RIOT/blob/c7d8ab39e5252f2a5b3ef6c8cf6d41a21265f18c/cpu/riscv_common/Makefile.features) **comments out `rust_target` for RV32**: its note attributes this to missing RISC-V type support in C2Rust 0.22.1 with Clang newer than 15. This affects the upstream C6 path. Generic Rust docs therefore overstate what is immediately usable for our chip.

Recommendation: place RIOT behind the Rust feasibility gate. A compatible pinned toolchain, upstream fix, or alternate binding approach may resolve it, but none was validated here. Discuss whether the networking comparison justifies a small C benchmark adapter before writing one. Do not present a Rust hello-world command as known working on current C6 RIOT.

## Why evaluate it

**Pros, assessment:** focused IoT networking model, explicit radio interfaces, reusable sensor/network abstractions, and a useful comparison for mesh/CoAP workloads. Good fit: constrained sensor nodes and networking experiments where portability matters.

**Cons, assessment:** current RISC-V Rust limitation conflicts with our preference; vendor ESP-IDF and binary radio components still affect dependency maintenance; flash-encryption gap may be a product exclusion. Network feature availability should be evaluated at the C6 driver level.

## Build-only starting point

Using a pinned RIOT checkout and its prepared ESP32-C6 toolchain, from the repository root:

```sh
. dist/tools/esptools/export.sh esp32c6
make BOARD=esp32c6-devkit -C examples/basic/hello-world
```

The [toolchain guide](https://guide.riot-os.org/cpus/esp32/#using-the-toolchain) documents the export script and make pattern; the [getting-started guide](https://docs.riot-os.org/getting-started/installing/) identifies the current basic example path. This combines documented parts and is **not executed here**. A prepared RIOT Docker image is another documented build option; no tooling was installed during research.

## Benchmark and lifecycle questions

- Separate scheduler/timer tests from GNRC, Wi-Fi, BLE, and native 802.15.4 images. Measure useful payload energy and wake latency alongside raw throughput.
- Add queue/backpressure behavior under radio bursts, packet loss recovery, and RAM high-water marks. Mesh experiments need additional nodes; one board cannot validate a mesh.
- Check update/rollback strategy on C6 before judging deployment suitability. Generic update packages do not prove a complete board-specific production path.
- Pin the RIOT commit, ESP-IDF package, compiler, C2Rust/Clang/Rust versions, modules, and radio dependencies. Include integration effort and unavailable requirements in the score rather than inventing a runtime result for blocked features.

Source snapshot observed during research: RIOT [`c7d8ab39e5252f2a5b3ef6c8cf6d41a21265f18c`](https://github.com/RIOT-OS/RIOT/tree/c7d8ab39e5252f2a5b3ef6c8cf6d41a21265f18c). This records research provenance, not a validated benchmark baseline.
