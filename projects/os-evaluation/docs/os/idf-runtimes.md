# Other usable C6 runtimes and frameworks

Research checked: 2026-10-22. These expand the application environment; they are not separate kernels. Treat them as optional developer-experience/application benchmarks above the ESP-IDF baseline.

## Arduino-ESP32

**Confirmed stable C6 support.** Espressif lists C6 in its stable supported-SoC table. [Support matrix](https://docs.espressif.com/projects/arduino-esp32/en/latest/getting_started.html), [GitHub](https://github.com/espressif/arduino-esp32), [API/library docs](https://docs.espressif.com/projects/arduino-esp32/en/latest/libraries.html), [license](https://github.com/espressif/arduino-esp32/blob/master/LICENSE.md).

Uses ESP-IDF and its FreeRTOS kernel. Pros: broad sensor libraries, rapid peripheral prototypes, direct access to IDF where needed. Cons: C++ ecosystem, library assumptions about chips/pins, and less explicit configuration in simple sketches. Family-level library support is not proof of C6 peripheral support; check each example/library target. Rust can interoperate through FFI, but direct `esp-idf-hal` is the simpler fit for our Rust preference. Benchmark only when library availability or Arduino integration is a decision factor. Core licensing is LGPL-2.1; bundled dependencies have separate terms.

## MicroPython

**Confirmed upstream C6 firmware.** The [generic C6 download](https://micropython.org/download/ESP32_GENERIC_C6/) explicitly lists Wi-Fi and BLE and provides release binaries. [Source](https://github.com/micropython/micropython/tree/master/ports/esp32), [C6 board definition](https://github.com/micropython/micropython/tree/master/ports/esp32/boards/ESP32_GENERIC_C6), [ESP32 quick reference](https://docs.micropython.org/en/latest/esp32/quickref.html), [MIT license](https://github.com/micropython/micropython/blob/master/LICENSE).

Python VM on the IDF port, not another kernel. Pros: REPL, quick experiments, readable hardware scripts. Cons: interpreter and garbage-collection effects, heap pressure, dynamic typing and a smaller exposed feature set than IDF. Do not infer Thread/Zigbee APIs from C6 radio hardware. Rust is not the normal application route; native extensions need additional build integration. Useful for measuring prototype effort, VM footprint, GC-related latency and messaging workflows; inappropriate as a pure scheduler comparison.

## CircuitPython

**C6 builds exist; the Espressif port still labels C6 alpha.** A current stable language release on a download page does not change that SoC maturity label. [C6 DevKitC-1 N8 firmware](https://circuitpython.org/board/espressif_esp32c6_devkitc_1_n8/), [port status](https://docs.circuitpython.org/en/latest/ports/espressif/README.html), [source](https://github.com/adafruit/circuitpython/tree/main/ports/espressif), [license inventory](https://github.com/adafruit/circuitpython/blob/main/LICENSE).

Python runtime using Espressif's SDK/drivers. Pros: approachable APIs and Adafruit's library ecosystem. Cons: alpha target, interpreter/GC costs and per-build module differences. Check the [module availability matrix](https://docs.circuitpython.org/en/latest/shared-bindings/support_matrix.html) for the exact board; a board download description is not proof that every advertised radio feature is bound. C6's native USB is fixed Serial/JTAG hardware, so do not assume USB mass-storage, HID or generic OTG behavior from S2/S3 tutorials. Rust is not a normal application route. Optional comparison for prototyping and sensor workflows, with GC pauses and file-transfer workflow recorded.

## Moddable SDK and XS JavaScript

**Confirmed C6 SDK targets.** Documentation lists `esp32/esp32c6`, XIAO ESP32C6, M5NanoC6 and other C6 platforms. [C6 platform guide](https://github.com/Moddable-OpenSource/moddable/blob/public/documentation/devices/esp32.md), [source](https://github.com/Moddable-OpenSource/moddable), [documentation](https://www.moddable.com/documentation), [licenses](https://github.com/Moddable-OpenSource/moddable/tree/public/licenses).

XS JavaScript engine and IoT modules above ESP-IDF. Pros: JavaScript application model, debugger and UI/network ecosystem. Cons: another language/runtime, C native module boundary, and no established Rust application route. The guide demonstrates C6 target integration, not blanket parity for all C6 radios and IDF features. Relevant only if scripting/UI developer experience outweighs our Rust preference; benchmark application responsiveness, memory/GC behavior and native-extension effort, not independent kernel switching.

See separate sheets for [Mongoose OS](mongoose-os.md) and the [hosted Contiki-NG port](contiki-ng.md). Protocol stacks such as OpenThread, NimBLE and Matter and firmware products such as ESPHome/ESP-AT are additional application choices, not extra independent OS kernels.

Additional confirmed language/runtime routes—AtomVM, Toit, NodeMCU Lua, TinyGo, WAMR and Embedded Swift—are covered in [other runtimes](other-runtimes.md). TinyGo uses a native runtime rather than this sheet's IDF framework model.
