# ESP32 C6 hardware evaluation coverage

The board is an **ESP32-C6-DevKitC-1-N8**. The [hardware inventory](../../../docs/hardware.md) records
both USB-C ports, the PCB antenna, GPIO8 RGB LED, BOOT/RESET controls, headers
and J5 module-current jumper. These are available board features; their
presence does not qualify a driver or change historical ledger outcomes.
Evaluation includes hardware behavior and power consumption.

The evaluation covers every subsystem that this board exposes, rather than choosing one application profile in advance. A supported chip function still requires an OS driver, a Rust-accessible API, available pins and the appropriate external fixture. Record those four dimensions independently.

The inventory follows Espressif's [C6 capability definitions](https://github.com/espressif/esp-idf/blob/master/components/soc/esp32c6/include/soc/soc_caps.h). The proposed experiments below are our test design. They are not measured support claims for any OS. Check revision-specific errata when creating each test.

The populated [evidence ledger](evidence-ledger.md), stored in
[quality/evidence.json](../quality/evidence.json), records the current state
separately from this plan. It expands 26 surveyed evaluation routes across all
25 hardware domains below and ten measurement/adoption objectives. Nineteen
pairs have scoped partial evidence; the remaining 891 are explicitly unknown /
not run. No complete subsystem is qualified. The route list includes combined
stacks and frameworks and does not establish 26 independent kernels or working
local ports.

## Subsystem experiments

| Subsystem | Proposed qualification and measurement | Additional fixture or condition |
| --- | --- | --- |
| HP CPU and interrupts | Integer workloads; interrupt priority, masking, nesting and event response; register preservation across switches | GPIO stimulus and timing capture |
| LP core and LP SRAM | Program loading, shared-memory handoff, wakeup source, sensing while HP sleeps, fault recovery | Wake stimulus; power instrumentation |
| Clocks, timers and watchdogs | Monotonicity, tickless behavior, jitter, rollover, sleep continuity, watchdog recovery | External reference/capture; deliberate stalled workload |
| GPIO and pin routing | Input/output/interrupt operation, edge loss and latency, pull configurations, pin conflicts | DevKitC-1 pin map; select unused header pins, loopback and logic analyzer; BOOT/GPIO9 is a possible controlled input after startup |
| UART and LP UART | Loopback, sustained receive, framing/overflow recovery, wake and buffering | Serial peer or loopback; LP routing |
| SPI and DMA | Full-duplex correctness, transfer sizes/rates, queued transfers, CPU use and contention | Loopback or known SPI peripheral |
| I2C and LP I2C | Supported roles, repeated starts, bus recovery, timeout and sleep interaction | Known peripheral/peer with suitable pull-ups |
| I2S | Stream correctness, DMA underrun/overrun, sample-clock accuracy and CPU budget | Audio codec or digital loopback peer |
| LED PWM and motor PWM | Frequency/duty accuracy, jitter, supported synchronization/capture/fault functions | Logic analyzer; signal-only control fixture |
| RMT, pulse counter and ETM | Pulse timing/capture, counted-edge loss, event routing without task intervention | GPIO8 onboard RGB LED for an initial RMT output check; generated pulse train/capture for timing and input tests |
| Parallel I/O and DMA | Data integrity, throughput and CPU use at supported widths | Wired peer; sufficient exposed pins |
| TWAI CAN controllers | Transmit/receive, bus-off recovery, filtering, latency under load | CAN transceiver, terminated bus and independent node |
| SDIO slave | Supported host/slave exchange, throughput and recovery | Compatible SDIO host and exposed signals |
| ADC and temperature sensor | Calibration path, noise, sample throughput and concurrent-radio interference | Known analog reference; independent temperature reference |
| USB Serial/JTAG and debug | Logging, debugger attach, reset/recovery, debug overhead and sleep interaction | Exposed native USB-C on GPIO12/13; separate CP2102N USB-C route; host and console selection must be recorded |
| Flash, cache and persistence | Read/write/erase, filesystem/NVS integrity, interrupted writes, cache effects and capacity/partition accounting | Recoverable test partitions; preserved original firmware backup |
| Crypto and RNG | Available AES/SHA/RSA/ECC/HMAC/DS paths, known-answer correctness, throughput and contention | Controlled test keys; distinguish software and hardware paths |
| Memory protection and boot security | Identify actual protection domains and unsafe/DMA/FFI boundaries; inspect signed boot, encryption, update and rollback paths | Configuration and failure tests; defer irreversible eFuse provisioning |
| Wi-Fi and ESP-NOW | STA/AP where implemented, goodput/latency/loss, reconnect, discovery and power policy | Controlled AP and compatible peer |
| BLE | Host/controller roles, advertising, GATT, supported PHYs, throughput, latency and reconnection | Compatible BLE peer |
| Raw IEEE 802.15.4 | Frame exchange, channel access, loss, timing and power | Second compatible radio node |
| Thread and Zigbee | Join, supported role, delivery, sleepy behavior, security and recovery; keep separate stack results | Thread border router or Zigbee coordinator and peers |
| Matter and application protocols | Commissioning, transport, memory, reconnect and update interaction on supported stacks | Controller and required network infrastructure |
| Radio coexistence | Repeat supported simultaneous workloads and quantify interference and missed deadlines | Multiple peers; version-specific combination support |
| Sleep and power domains | Active/idle/light/deep sleep, retained state, wake sources, LP offload, energy per delivered operation | J5 for module-current measurement or declared whole-board supply boundary; calibrated instrument and wake sources still needed |

The N8 board has no PSRAM. Its native USB is fixed serial/JTAG, with no
general-purpose USB device-class or OTG capability. Bluetooth Classic and
interchangeable dual-core scheduling must not be inferred from another ESP32
chip. Examples still need a C6-specific driver and configuration check.

## Capability record

For every platform/subsystem pair, record one driver state: `upstream`,
`community`, `missing`, or `unverified`. Separately record the scoped Rust API,
board/pin and fixture state, source revision through its linked proof, claim
class and execution outcome. The checked ledger uses `not_run`, `partial`,
`qualified`, `failed` and `blocked`: `partial` prevents a successful UART log
or compute control from silently becoming a whole-subsystem pass. Evidence
classes distinguish source, builds, observations, operator reports and
independent reproduction. The legend documents which structural promotions
are rejected and which semantic judgments still require review.

Missing fixtures are not OS defects. Upstream source entries are not successful
hardware tests. A frozen native test can emit all expected rows yet fail its
qualification contract, as the Zephyr record does; the UART output remains a
separate partial observation. A successful same-kernel API comparison likewise
does not qualify CPU utilization, every driver or a custom-runtime requirement.

Start with common tests that need only the board. Expand to external peripherals and radio peers as fixtures are available. No unavailable fixture prevents documenting the remaining capabilities.

The [prior-work review](../../../docs/research/prior-work.md) and
[reuse inventory](../../../docs/research/benchmark-reuse.md) select existing tests and reference
fixtures where possible. Keep this full inventory, but give each implemented
experiment a decision question and explicit replication/extension contract.
Published range or mesh studies do not require repeating their entire sweep to
choose an OS; first prove the required Rust-accessible behavior and its effect
on concurrent application deadlines, delivery and recovery.

## How to choose platforms after broad testing

Publish a capability table and independent plots for memory, timing, energy, radio performance and development effort. Prefer the set of platforms that are not consistently worse across the measured dimensions. Choose application-specific weights only when a real project supplies requirements; the example weights in [decision criteria](decision-criteria.csv) are optional scenarios, not an agreed overall ranking.

Retain the [custom Rust baseline](os/custom-rust.md), [LP/core and loader research systems](os/research-kernels.md), and high-level runtimes as separate categories. This keeps architectural differences visible while preserving the full hardware scope.
