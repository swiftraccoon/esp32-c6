# Controlled SoftAP witness firmware

The default `device-identity-wifi-firmware` binary retains the version-one
single-start behavior below. A second `device-identity-wifi-restart` binary
implements the separate [two-phase restart method](../experiments/wifi-restart.md).

ESP32-C6 firmware for one UART-controlled AP trial per boot. UART0 uses
GPIO16 TX/GPIO17 RX, 115200 baud, 8N1. Before a Start attempt, boot and Status
do not initialize Wi-Fi.
The [shared protocol](../crates/wifi-protocol/src/lib.rs) accepts Status, Start
and Stop with one private nonce. Start configures an open, visible AP on channel
1 with a nonce-bearing SSID and the protocol's fixed synthetic, locally
administered unicast AP MAC. These controls are not an address-rotation scheme.

Only the first valid Start can initialize/start Wi-Fi, including when that
attempt fails. The AP MAC is saved, overridden and checked before start, then
checked again after the driver's AP-start event. The first Stop calls the SDK
stop and waits for the event-backed inactive state. Later Stop requests skip
that SDK call only after a previously checked stop and a current inactive
status. Every Stop restores the saved AP MAC and checks its readback. Stop is
repeatable, including after automatic stop or a lost reply.
The initialized driver remains owned and stopped until reset; no deinitialization
or RF-silence claim follows from this readback. Constructor failure can leave
partial SDK initialization without a driver handle; it is terminal and produces
only failed responses, requiring the host's reset/recovery procedure.

The application polls an independent 8000 ms software deadline from the Start
command attempt. It also requests stop after malformed UART input or transport
failure. SDK calls and scheduling are not hard real-time bounded; this deadline
does not measure or guarantee on-air duration. Automatic stop emits no unsolicited
UART line. The next Status or Stop exposes its state; any failed automatic stop
is retained as a failed exchange before a later cleanup can be admitted. Nonce,
SSID and original driver MAC readbacks belong only in private captures. SDK
readback does not prove transmitted identity: a separate receiver must observe
the expected beacon, and a failed trial remains failed.

A failed automatic cleanup is retried once per requested second without UART
activity, retaining its first failure. This interval is checked by the same
main-task loop and shares its scheduling and SDK blocking limits.

This application uses WifiDriver directly, without EspNetif, DHCP, a station,
scanning, provisioning, BLE or IEEE 802.15.4 services. The constructor receives
no NVS partition. A small Rust FFI adapter selects WIFI_STORAGE_RAM before the
first AP configuration; no persistent Wi-Fi setter is used. The wrapper's AP
builder sets a 100 TU beacon interval. Protocol bitmap defaults come from the
pinned SDK; its configuration `protocols` field is not applied by this wrapper.
The open AP can receive ambient probe/association traffic during the trial; no
claim of payload isolation or nonassociation follows from this implementation.

## Build inputs

The independent workspace pins Rust 1.96.0, language/style edition 2024,
`riscv32imac-esp-espidf`, build-std for std/panic_abort and libclang 21.1.8.
Sys/HAL/SVC are pinned to `a209052f378be61dd0f2d566b7fbe0f9593b0cd5`.
ESP-IDF v5.5.6 is commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`;
embuild is 0.33.7. The ignored `.embuild` link selects cached SDK/tools.
From this directory:

```sh
C6_WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
export PATH="$C6_WORKSPACE_ROOT/.tools/bin:$PATH"
export LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib
export CARGO_WORKSPACE_DIR="$PWD"
export RUSTC_BOOTSTRAP=1
cargo +1.96.0 build --release --offline --locked
cargo +1.96.0 clippy --release --offline --locked --no-deps -- -D warnings
cargo +1.96.0 doc --release --offline --locked --no-deps
cargo +1.96.0 fmt -- --check
```

Release uses optimization 3, thin LTO, one codegen unit, debug information and
panic abort/reboot. Configuration selects CPU 160 MHz, 1000 Hz RTOS ticks,
16 KiB main stack, 8 MB DIO/40 MHz flash and a 3 MiB factory app partition.
Ordinary logs/consoles, Wi-Fi NVS/PHY calibration storage, Bluetooth and IEEE
802.15.4 support are disabled in defaults; inspect the generated sdkconfig
before asserting actual build configuration. The NVS partition is preserved in
the flash layout but is not opened by this application.

The main task owns UART, Wi-Fi and its system event loop. Driver construction,
SDK tasks and subscriptions allocate; command/reply buffers are bounded. No
authored interrupt handler, callback or asynchronous cancellation is present.
The single scalar FFI call uses generated SDK bindings, retains no pointers,
and runs after initialization without another Wi-Fi configuration owner.
Protocol and controller tests run in the host workspace; target build/lint/docs
do not qualify hardware or receiver behavior. Preserve and independently verify
a complete 8 MB flash backup before deployment, and restore/digest-verify the
original image after the board trial under the project's controlled method.

## Two-phase restart binary

Select the new binary explicitly using the same build environment above:

```sh
cargo +1.96.0 build --release --offline --locked --bin device-identity-wifi-restart
cargo +1.96.0 clippy --release --offline --locked --bin device-identity-wifi-restart --no-deps -- -D warnings
cargo +1.96.0 doc --release --offline --locked --no-deps --bin device-identity-wifi-restart
```

Version two lives in `wifi_protocol::restart`. Exactly two ordered start attempts
share one nonce SSID: First, explicit checked Stop with original-MAC restoration,
Second, checked Stop with the same original restored. One lazy driver/loop is
retained throughout; configuration is applied once and only the disabled AP MAC
changes before the second start. Each active readback must match its own fixed
synthetic control. No client reconnect or privacy improvement is implied.

The controller consumes increasing request IDs and one session nonce. Every
protocol, transport, SDK or failed automatic-cleanup operation permanently
records the first failure and blocks subsequent starts. Successful Status/Stop
can still expose checked cleanup plus that failure; capture admission rejects
the failure latch. Automatic Stop alone cannot authorize the second start.
Each attempted phase gets its own eight-second software deadline and failed
cleanup retries without UART. SDK blocking and scheduling retain the limits
described above. The initialized driver remains owned until reset.

Pure protocol/controller controls run in the host workspace; target builds
check actual SDK integration. Neither establishes on-air presence. Use one
continuous UART connection and two separately ready receiver captures through
`wifi-ap-restart-trial`; two invocations of the single-start collector can reset
the board and cannot establish the same sequence. Deployment still requires a
fresh verified full-image backup, segmented erase-range gates and complete
restore/verification before release.
