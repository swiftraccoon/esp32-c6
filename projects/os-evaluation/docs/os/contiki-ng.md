# Contiki-NG on ESP32-C6

Research checked: 2026-10-22. Status: **experimental community port with hardware evidence, hosted inside ESP-IDF/FreeRTOS**. Do not count this implementation as an independent replacement kernel.

## Evidence and links

The [joakimeriksson/contiki-ng-esp32 port](https://github.com/joakimeriksson/contiki-ng-esp32) runs Contiki-NG's cooperative process loop in one FreeRTOS task. Its README pins Contiki-NG and ESP-IDF v5.5.4, and reports real C6 CSMA/RPL-Lite/UDP interoperability with a CC2538 node. A display is optional, not essential to the port. [Upstream Contiki-NG](https://github.com/contiki-ng/contiki-ng), [documentation](https://docs.contiki-ng.org/en/develop/), [port architecture and hardware test](https://github.com/joakimeriksson/contiki-ng-esp32/blob/main/README.md).

## Supported use case

IEEE 802.15.4 via Espressif's driver, 6LoWPAN/IPv6, CSMA and RPL-Lite mesh experiments. **TSCH is explicitly unsupported**: the port does not provide the timestamp/poll-mode behavior needed, and its realtime timer uses an ESP timer task. The event loop's documented yield and single-frame receive buffering deserve scrutiny under burst traffic. These limitations are in this particular port; they do not describe Contiki-NG on every supported platform. [Port README](https://github.com/joakimeriksson/contiki-ng-esp32/blob/main/README.md)

## Pros and cons

- **Pros:** useful research platform for low-power IP mesh, protocol interoperability and simulated multi-node experiments; preserves the vendor driver environment.
- **Cons:** community integration, additional runtime layer, limited buffering, no demonstrated TSCH, and C-heavy APIs. Hardware test claims are not measurements on our board.
- **Rust:** no turnkey Rust application layer demonstrated here. Direct Rust/IDF OpenThread or Rust `no_std` networking may better fit product development; select this port when Contiki/RPL research is itself a requirement.
- **Licensing:** [upstream Contiki-NG](https://github.com/contiki-ng/contiki-ng/blob/develop/LICENSE.md) uses a BSD-style license. The port repository did not expose a top-level license through GitHub metadata during research; clarify its glue-code license before reuse. IDF/radio licensing also applies.

## Benchmark role

Second-wave mesh application comparison: join time, delivery ratio, retries, burst loss, memory per neighbor and energy per delivered message. Use at least two radio nodes or an independently characterized peer. Compare equivalent protocol workloads; RPL/6LoWPAN and OpenThread have different behavior, so do not present protocol differences as kernel performance.
