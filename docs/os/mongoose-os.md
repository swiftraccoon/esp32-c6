# Mongoose OS

Research checked: 2026-10-22. Status: **C6 core platform merged; end-user integration incomplete in the port's evidence**. Framework on ESP-IDF/FreeRTOS, not an additional independent kernel.

## Evidence and links

[C6 PR #636](https://github.com/cesanta/mongoose-os/pull/636) merged on 2026-07-26, adding `esp32c6` against vanilla ESP-IDF v6.0.2. The author reports validation on C6 revision 0.2 with 8 MB flash: GPIO/interrupts, UART, timers, filesystem/config persistence, Wi-Fi AP/STA, DHCP, MQTT and TLS. OTA was also tested in the author's environment, but the PR lists separate integration work. [C6 source directory](https://github.com/cesanta/mongoose-os/tree/master/platforms/esp32c6), [project](https://github.com/cesanta/mongoose-os), [docs](https://mongoose-os.com/docs/).

The PR explicitly leaves companion library compatibility, publication of the C6 build image, `mos` manifest/flash support, and separate OTA upstreaming as follow-ups. A merged platform directory therefore does **not** prove a fresh install works. The older 2025 forum answer saying C6 is unsupported predates this merge. [Port scope and follow-ups](https://github.com/cesanta/mongoose-os/pull/636)

## Support boundaries and tradeoffs

- **Pros:** device configuration, persistent state and IoT services may reduce application plumbing; real C6 hardware validation exists.
- **Cons:** incomplete distribution/toolchain evidence and a new port; existing ESP32 libraries can still depend on older IDF APIs. BLE, 802.15.4, Zigbee, power modes and security configurations were not established by this PR's validation.
- **Rust:** C/JavaScript are the established application routes. A Rust FFI integration would be custom work with little benefit over using Rust directly on IDF for our current objective; discuss before writing such a harness.
- **License:** current core repository [license](https://github.com/cesanta/mongoose-os/blob/master/LICENSE) is Apache-2.0. Dependencies and optional services require their own inventory. Do not conflate Mongoose OS with the separate [Mongoose networking library](https://github.com/cesanta/mongoose).

## Benchmark role

Optional application-framework comparison after ESP-IDF is stable: compare boot/config persistence, MQTT reconnect, TLS memory, OTA recovery and engineering effort. Scheduler numbers would primarily measure FreeRTOS plus framework load. First gate: reproduce a clean C6 build without unpublished local components and document which follow-ups are now resolved.
