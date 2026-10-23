# Additional C6 language and sandbox routes

Research checked: 2026-10-22. These are optional application/runtime comparisons. Their C6 targets do not establish complete coverage of every C6 peripheral. Rust remains our implementation preference; using another language for a benchmark would need a reason tied to its execution model.

## AtomVM with Erlang and Elixir

**Confirmed upstream C6 images and build documentation.** AtomVM runs a small BEAM-compatible VM with lightweight processes and message passing, hosted on ESP-IDF/FreeRTOS on ESP32. [Source](https://github.com/atomvm/AtomVM), [C6-capable release images](https://github.com/atomvm/AtomVM/releases/tag/v0.6.6), [getting started](https://doc.atomvm.org/main/getting-started-guide.html), [Apache-2.0 license](https://github.com/atomvm/AtomVM/blob/main/LICENSE).

Pros: actor-style concurrency, fault-recovery application patterns, and existing Erlang/Elixir tooling. Costs: VM memory/GC, an implemented subset of Erlang/OTP, and C native-interface work for unavailable bindings. Match host Erlang/Elixir versions to the firmware's compatibility requirements. Official C6 binaries prove a runnable route; they do not establish Thread/Zigbee bindings or all hardware drivers.

Benchmark value: process/message scalability, tail latency under GC, supervised recovery, socket/MQTT behavior and native-driver boundary costs. Compare these application properties separately from FreeRTOS task timing. Rust interoperability is custom native integration, not the normal development path.

## Toit and Jaguar

**Confirmed upstream C6 toolchain target.** The [C6 toolchain](https://github.com/toitlang/toit/tree/master/toolchains/esp32c6) includes a dedicated IDF project and configuration. Its [default configuration](https://github.com/toitlang/toit/blob/master/toolchains/esp32c6/sdkconfig.defaults) enables Wi-Fi-related settings, NimBLE and FreeRTOS. Toit is a managed language/VM on IDF; Jaguar supplies a development and deployment environment. [Source/build guide](https://github.com/toitlang/toit), [Jaguar](https://github.com/toitlang/jaguar), [documentation](https://docs.toit.io/), [LGPL-2.1 license](https://github.com/toitlang/toit/blob/master/LICENSE).

Pros: managed applications, interactive deployment and an IoT-oriented package ecosystem. Costs: adopting another language/VM, a recommended Toit-specific IDF fork, GC/heap effects, and bindings that may expose fewer features than the underlying chip. Config flags are implementation evidence, not our functional test results. Rust would require native integration.

Benchmark value: deployment turnaround, application concurrency/recovery, persistent state, network reconnect and memory behavior. Gate on a clean C6 build and the exact peripherals needed by each workload.

## NodeMCU Lua ESP32 branch

**Confirmed initial C6 support, on `dev-esp32`.** [PR #3646](https://github.com/nodemcu/nodemcu-firmware/pull/3646), merged in April 2024, added C6/H2 and native Serial/JTAG console support. The [ESP32 CI matrix](https://github.com/nodemcu/nodemcu-firmware/blob/dev-esp32/.github/workflows/build.yml) includes `esp32c6`. The repository's default `release` branch targets ESP8266 and is the wrong basis for judging C6 support. [ESP32 source/README](https://github.com/nodemcu/nodemcu-firmware/tree/dev-esp32), [docs](https://nodemcu.readthedocs.io/en/dev-esp32/), [MIT license](https://github.com/nodemcu/nodemcu-firmware/blob/dev-esp32/LICENSE).

Event-driven Lua on ESP-IDF/FreeRTOS. Pros: compact scripts, callbacks and reusable Lua/C modules. Costs: development-branch maturity, older SDK coupling, VM/GC costs, and chip-specific module restrictions. Do not conflate it with boards sold under the NodeMCU name or with Whitecat Lua RTOS. Rust is not its normal application route. Useful for scripting effort, event response, GC pauses and module footprint; revalidate each C6 module before claiming coverage.

## TinyGo

**Confirmed native C6 compiler/runtime target, separate from IDF wrappers.** Released [v0.43.0](https://github.com/tinygo-org/tinygo/releases/tag/v0.43.0) includes the [C6 target](https://github.com/tinygo-org/tinygo/blob/v0.43.0/targets/esp32c6.json), [C6 runtime](https://github.com/tinygo-org/tinygo/blob/v0.43.0/src/runtime/runtime_esp32c6.go), and [XIAO C6 board target](https://github.com/tinygo-org/tinygo/blob/v0.43.0/targets/xiao-esp32c6.json). The base C6 target is inheritable-only; choose/adapt an actual board target. [Source](https://github.com/tinygo-org/tinygo), [docs](https://tinygo.org/docs/), [BSD-style license and dependency notices](https://github.com/tinygo-org/tinygo/blob/v0.43.0/LICENSE).

Compiled Go with its own embedded runtime, not another conventional RTOS kernel. C6-specific machine sources implement GPIO/interrupts, UART, ADC, I2C and native console support; additional drivers evolve by release. This evidence does **not** establish working C6 Wi-Fi/BLE/802.15.4 stacks. Pros: goroutines/channels and native compilation. Costs: Go subset/runtime behavior, GC, and considerably narrower proven radio coverage than IDF. Benchmark no-radio footprint, goroutine/channel latency and GC behavior if Go is a serious alternative; Rust remains the closer fit to our stated preference.

## WAMR WebAssembly Micro Runtime

**Confirmed upstream ESP32-C6 IDF component target.** [Manifest](https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/idf_component.yml), [ESP-IDF example](https://github.com/bytecodealliance/wasm-micro-runtime/tree/main/product-mini/platforms/esp-idf), [source](https://github.com/bytecodealliance/wasm-micro-runtime), [documentation](https://wamr.gitbook.io/), [license](https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/LICENSE).

A Wasm execution environment above IDF/FreeRTOS, useful for portable payloads and controlled host APIs. Rust can produce Wasm modules, making this relevant without replacing our application language. GPIO/radio access still needs host bindings, and Wasm memory checks do not by themselves prove isolation of native host code or peripherals. Interpreter/AOT features must be checked against the exact C6 configuration; do not assume desktop JIT support. Core code is Apache-2.0, with LLVM exceptions on some files.

Benchmark value: native Rust versus Rust-to-Wasm footprint, call overhead, startup/load time, trapped invalid memory accesses and safe payload restart. Gate on the compiler/runtime ABI, memory limits and a deliberately small host interface.

## Embedded Swift

**Confirmed official C6 example route.** Swift's [Matter examples](https://github.com/swiftlang/swift-matter-examples) specifically target C6 DevKitC-1 N8 using ESP-IDF and ESP-Matter, with build tutorials and declared toolchain versions. [Embedded examples](https://github.com/swiftlang/swift-embedded-examples), [language manual](https://github.com/swiftlang/swift/blob/main/docs/EmbeddedSwift/UserManual.md), [example licensing](https://github.com/swiftlang/swift-matter-examples/blob/main/LICENSE.txt).

This is compiled application-language integration on the underlying vendor stack, not a new OS. Pros: an official, concrete Matter integration example and Swift/C interop. Costs: Embedded Swift is a constrained language subset, examples use specific SDK/toolchain combinations, and the example overlay is not a complete Matter API. Useful as an interoperability/developer-effort reference for Matter; it does not justify abandoning Rust without a workload-specific benefit.
