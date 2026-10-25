# ESP32-C6 ecosystem audit

Checked 2026-10-22 against primary project documentation, upstream source trees and issue/merge records. This is a bounded audit, not a claim that no private or hobby port exists. “ESP32 supported,” a RISC-V CPU port, or a C6 acting as a radio coprocessor does not establish an OS running on the C6 itself. No benchmark result is inferred from documentation.

## Evidence classes

- **Confirmed:** a C6-specific upstream board/SoC target or a documented runnable C6 port exists. This does not prove complete drivers or production readiness.
- **Experimental:** meaningful C6 implementation evidence exists, with integration gaps, narrow drivers or a community-only implementation.
- **Unverified for C6:** the project exists, but this audit found no sufficient C6 BSP/build/hardware evidence.
- **Not a separate kernel:** a runtime, framework, protocol stack or application whose scheduler comes from another candidate.

## Confirmed independent kernel choices

| Candidate | Evidence and boundary | Reference |
|---|---|---|
| ESP-IDF FreeRTOS | Official C6 kernel documentation, vendor drivers and services. C6 uses one HP scheduler core. | [Sheet](../os/esp-idf-freertos.md), [upstream](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/system/freertos.html) |
| Zephyr | Upstream C6 board support; use its target-specific peripheral inventory. | [Sheet](../os/zephyr.md), [board](https://docs.zephyrproject.org/latest/boards/espressif/esp32c6_devkitc/doc/index.html) |
| Apache NuttX | Upstream C6 architecture and board documentation. | [Sheet](../os/nuttx.md), [C6 platform](https://nuttx.apache.org/docs/latest/platforms/risc-v/esp32c6/index.html) |
| RIOT | Upstream C6 DevKit board entry; assess the C6-specific support table. | [Sheet](../os/riot.md), [board](https://doc.riot-os.org/group__boards__esp32c6__devkit.html) |
| RT-Thread | C6 BSP merged September 2026; GPIO/UART/uncalibrated ADC/GDBStub, with major buses/radios pending. Kernel-only experimental admission. | [Sheet](../os/rt-thread.md), [merged PR](https://github.com/RT-Thread/rt-thread/pull/11836) |

Rust-native executor/scheduler choices and their C6 proof are recorded in the dedicated Rust research sheets. Embassy, RTIC, a HAL, a runtime and a conventional RTOS differ in their scheduling/resource models; count runnable stacks and measured capabilities rather than marketing names.

## Confirmed runtimes and frameworks with C6 implementations

| Candidate | Classification and evidence | Reference |
|---|---|---|
| Arduino-ESP32 | Stable C6 framework support; ESP-IDF/FreeRTOS underneath. | [Runtime sheet](../os/idf-runtimes.md), [official matrix](https://docs.espressif.com/projects/arduino-esp32/en/latest/getting_started.html) |
| MicroPython | Official generic C6 binaries and board source; VM on the ESP32 IDF port. | [Runtime sheet](../os/idf-runtimes.md), [C6 downloads](https://micropython.org/download/ESP32_GENERIC_C6/) |
| CircuitPython | C6 builds available; SoC port itself labeled alpha. | [Runtime sheet](../os/idf-runtimes.md), [port status](https://docs.circuitpython.org/en/latest/ports/espressif/README.html) |
| Moddable SDK/XS | Official C6 platform identifiers; JavaScript runtime on IDF. | [Runtime sheet](../os/idf-runtimes.md), [platform guide](https://github.com/Moddable-OpenSource/moddable/blob/public/documentation/devices/esp32.md) |
| AtomVM | Official C6 Erlang/Elixir VM images on IDF/FreeRTOS. | [Additional runtimes](../os/other-runtimes.md), [release assets](https://github.com/atomvm/AtomVM/releases/tag/v0.6.6) |
| Toit/Jaguar | Dedicated upstream C6 IDF toolchain and configuration. | [Additional runtimes](../os/other-runtimes.md), [C6 source](https://github.com/toitlang/toit/tree/master/toolchains/esp32c6) |
| NodeMCU Lua | Initial C6 support merged in `dev-esp32`, with C6 CI target; default ESP8266 branch is misleading here. | [Additional runtimes](../os/other-runtimes.md), [PR #3646](https://github.com/nodemcu/nodemcu-firmware/pull/3646) |
| TinyGo | Released native C6 compiler/runtime and XIAO board target; not an IDF wrapper or conventional independent RTOS kernel. | [Additional runtimes](../os/other-runtimes.md), [v0.43.0 target](https://github.com/tinygo-org/tinygo/blob/v0.43.0/targets/esp32c6.json) |
| WAMR | Upstream Wasm runtime explicitly lists C6 as an IDF component target; Rust-to-Wasm is a relevant application option. | [Additional runtimes](../os/other-runtimes.md), [manifest](https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/idf_component.yml) |
| Embedded Swift | Official C6 Matter application example using IDF/ESP-Matter; application-language route, not an OS. | [Additional runtimes](../os/other-runtimes.md), [official example](https://github.com/swiftlang/swift-matter-examples) |
| Mongoose OS | C6 core platform merged July 2026, with companion tools/libraries/OTA integration follow-ups identified. IDF-based framework. | [Sheet](../os/mongoose-os.md), [merged PR #636](https://github.com/cesanta/mongoose-os/pull/636) |
| Contiki-NG community C6 port | Documented physical C6 802.15.4 CSMA/RPL-Lite tests, implemented as a FreeRTOS task. TSCH unsupported. | [Sheet](../os/contiki-ng.md), [port](https://github.com/joakimeriksson/contiki-ng-esp32) |
| OpenC6 BIOS | Hobby C6 host/payload firmware with custom isolation/filesystem claims; README explicitly retains FreeRTOS as scheduler. Claimed tested on C6-Zero 8 MB. Interesting architecture study; claims require source review and reproduction. | [Source/README](https://github.com/Rompass/openc6-bios) |

## Candidates screened out of the first benchmark wave

| Project | C6 finding | What would change admission |
|---|---|---|
| Eclipse ThreadX | Generic `risc-v32` port exists. C6 bring-up issue #575 is now titled **CANCELLED**, closed September 8, 2026, with no linked PR. An old search snippet still described it as in progress. This is not delivery evidence. [Issue](https://github.com/eclipse-threadx/threadx/issues/575), [ports](https://github.com/eclipse-threadx/threadx/tree/master/ports) | A maintained C6 startup/linker/interrupt/timer port plus reproducible physical-board results. |
| RTEMS | C6 chip-definition MR !1186 exists, but primary GitLab API reports it closed with `merged_at: null`; it is narrower than a board port anyway. No runnable C6 BSP was verified in this audit. [MR](https://gitlab.rtems.org/rtems/rtos/rtems/-/merge_requests/1186), [API record](https://gitlab.rtems.org/api/v4/projects/rtems%2Frtos%2Frtems/merge_requests/1186) | Current upstream BSP, working image recipe, console/timer/interrupt proof. Treat this as unverified, not a categorical impossibility. |
| Apache Mynewt OS | Current `hw/mcu` inventory has no Espressif family. IDF's use of **NimBLE from Mynewt** does not establish Mynewt's kernel on C6. [MCU tree](https://github.com/apache/mynewt-core/tree/master/hw/mcu), [IDF NimBLE](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/bluetooth/nimble/index.html) | A C6 Mynewt MCU/BSP target and executable build/hardware evidence. |
| AliOS Things | Current chip inventory includes ESP32/C3/S3 families; C6 support was not established. C3/S3 support is insufficient. [Chip tree](https://github.com/alibaba/AliOS-Things/tree/master/hardware/chip) | A C6-specific supported target with current toolchain and timer/interrupt/driver evidence. |
| Huawei LiteOS | Current target tree inspected; no C6 target established. Generic RISC-V capability does not close the board-port gap. [Targets](https://github.com/LiteOS/LiteOS/tree/master/targets) | A maintained C6 BSP and hardware proof. |
| Embox | Current Espressif template/source evidence is for **C3**, not C6; no C6 target established. [Espressif templates](https://github.com/embox/embox/tree/master/platform/espressif/templates), [release notes](https://github.com/embox/embox/releases) | A C6-specific BSP and reproducible image. |
| ChibiOS / PicoRTOS | No C6 BSP found in the current upstream source trees inspected. This is an evidence gap, not a claim that either kernel is unportable. [ChibiOS](https://github.com/ChibiOS/ChibiOS), [ChibiOS-Contrib](https://github.com/ChibiOS/ChibiOS-Contrib), [PicoRTOS](https://github.com/jnaulet/picoRTOS) | C6 board/startup/timer/interrupt support and hardware proof. |
| Whitecat Lua RTOS | No C6 implementation established in its source tree. This project is distinct from NodeMCU's confirmed C6 branch. [Source](https://github.com/whitecatboard/Lua-RTOS-ESP32) | A C6-specific maintained port and working SDK recipe. |
| seL4 | Official supported-platform list has no ESP32-C6. Generic RV32 support is not a C6 port or a verified C6 configuration. [Platforms](https://docs.sel4.systems/Hardware/) | A supported port meeting kernel hardware requirements, plus explicit proof status if formal verification is a decision factor. |
| Standard Linux distributions | No native C6 BSP/distribution was verified. The C6's small MCU memory/privilege model is not a normal Linux distribution target. Linux supports NOMMU systems in general, so lack of a conventional MMU alone is not a proof of impossibility. [C6 datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.html), [Linux NOMMU](https://docs.kernel.org/admin-guide/mm/nommu-mmap.html) | A concrete native C6 port and memory budget; emulated Linux or a C6 attached to a Linux host must be classified separately. |
| Bare upstream FreeRTOS RISC-V port | A generic kernel/CPU port is useful custom-OS input, but does not replace C6 startup, interrupt routing, linker layout, drivers and radio integration. Avoid double-counting it with the IDF port. [Kernel source](https://github.com/FreeRTOS/FreeRTOS-Kernel) | Deliberately fund and document a separate C6 integration project. |

## Traps to avoid when revisiting sources

1. **Search snippets are stale.** Open the actual issue/PR. ThreadX's cancelled C6 issue and Mongoose OS's newly merged platform both changed the conclusion reached from older indexed content.
2. **A package merge is not a working release.** RT-Thread's [auxiliary sources PR](https://github.com/RT-Thread-packages/esp-idf/pull/17) merged, but the [package index proposal #2043](https://github.com/RT-Thread/packages/pull/2043) closed unmerged and the BSP README still recommends a pinned fork commit. Reproduce the documented path.
3. **Coprocessor support is different.** An [RT-Thread ESP-Hosted host driver](https://github.com/Evlers/rt-thread_esp-hosted) can use C6 as a Wi-Fi/BLE peripheral while RT-Thread runs on a different MCU. It is not a C6 kernel port.
4. **Chip hardware is not software support.** BLE-capable hardware does not imply a runtime BLE binding; an 802.15.4 radio does not imply Thread, Zigbee, Matter or TSCH. C6 is not C3, C5, C61 or S3.
5. **Even primary pages can contain copied hardware prose.** The CircuitPython C6 download page contains broad peripheral/USB claims inconsistent with the C6 datasheet. Use it as firmware-build evidence; use Espressif's [datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.html) for chip capabilities.
6. **Open license does not guarantee implementation source.** Espressif's Wi-Fi and PHY repositories explicitly provide Apache-2.0 object libraries. Zigbee SDK v2.x separately describes a proprietary protocol core. Track license, source availability and replaceability as separate properties. [Wi-Fi](https://github.com/espressif/esp32-wifi-lib/blob/master/README.rst), [PHY](https://github.com/espressif/esp-phy-lib/blob/master/README.rst), [Zigbee](https://docs.espressif.com/projects/esp-zigbee-sdk/en/latest/esp32c6/introduction.html)

For the decision, prioritize reproducible full-stack candidates first; use narrow kernel ports and hosted runtimes to answer specific questions. Unsupported features are capability gaps, not zero-valued performance scores.

## Repository snapshots observed during this audit

These branch heads locate the research context. They are **not** tested or selected benchmark versions, and do not pin transitive dependencies. Stable release selection must account for Rust wrapper compatibility and each port's documented SDK requirements. Checked on 2026-10-22 in America/New_York; UTC commit dates can cross into October 23.

| Repository / branch | Observed commit |
|---|---|
| ESP-IDF / master | [`383dda6c182e3dabf6355160166fe007df775629`](https://github.com/espressif/esp-idf/commit/383dda6c182e3dabf6355160166fe007df775629) |
| RT-Thread / master | [`2f7aba80fbbe3e3e44997dfe9819665f01a82534`](https://github.com/RT-Thread/rt-thread/commit/2f7aba80fbbe3e3e44997dfe9819665f01a82534) |
| Mongoose OS / master | [`391c718ee9b1e6070c6aca1c5d67b62c24d53477`](https://github.com/cesanta/mongoose-os/commit/391c718ee9b1e6070c6aca1c5d67b62c24d53477) |
| Hosted Contiki-NG C6 / main | [`36a0734ba2303b3466fddd91a3f70c72a3cd7c6a`](https://github.com/joakimeriksson/contiki-ng-esp32/commit/36a0734ba2303b3466fddd91a3f70c72a3cd7c6a) |
| Arduino-ESP32 / master | [`6b41cf6f6f272538ee91746f26c5068e2ca692e4`](https://github.com/espressif/arduino-esp32/commit/6b41cf6f6f272538ee91746f26c5068e2ca692e4) |
| MicroPython / master | [`b70ae1aca657bb93e2857dac4e2a2d21a3803200`](https://github.com/micropython/micropython/commit/b70ae1aca657bb93e2857dac4e2a2d21a3803200) |
| CircuitPython / main | [`9bf49b36e41d3a8cea38fb9b48b02bdea13584b8`](https://github.com/adafruit/circuitpython/commit/9bf49b36e41d3a8cea38fb9b48b02bdea13584b8) |
| Moddable / public | [`80076c48f5ab87ce29de728071b7408599071a12`](https://github.com/Moddable-OpenSource/moddable/commit/80076c48f5ab87ce29de728071b7408599071a12) |
