# Controlled BLE advertising firmware

This independent ESP32-C6 workspace contains a host-testable protocol/controller
and the `device-identity-ble-firmware` target binary. Boot and Status leave
Bluetooth uninitialized. UART0 uses GPIO16 TX/GPIO17 RX, 115200 baud, 8N1.
No serial port or radio is accessed by host tests or image generation.

The [project API contract](../docs/api.md#controlled-ble-firmware) records all
guideline decisions and exception E1 for internal raw status seams.
[Selected software inputs](software-inputs.json) retain the build-time image and
configuration record. The later [HCI presence result](../results/ble-identity-2026-10-26/README.md)
qualifies that saved image for two checked static-random advertising controls;
the build-time qualification flag is historical. Current source uses a local
selected-Start guard in place of the saved image's general role policy; this
revision has not been installed or observed on the board. Raw RF CRC and silence
remain unqualified.

The domain-tagged `ble_advertising` version-one protocol accepts Status,
StartFirst, Stop, StartSecond and Stop with increasing nonzero IDs and one
fresh nonzero eight-byte nonce. Each phase consumes one attempt, including
failed attempts. The second phase requires a successful first Start and an
explicit checked Stop; automatic Stop alone does not authorize it. Every
protocol, transport, callback, SDK or adapter error permanently blocks Starts.
Cleanup can still report its checked state and the first failure, without
converting a failed session into a pass.

Each phase uses its fixed synthetic static-random address from `protocol.rs`.
Advertising is legacy LE 1M, nonconnectable and nonscannable, at a fixed 100 ms
configured interval on all three primary advertising channels. Raw data is one
custom 128-bit Service Data AD containing the shared nonce. There is no local
name AD, scan-response configuration, GATT application, pairing, scanning,
connection initiation, privacy/RPA setter or persistent identity setter.
At the hardware Start boundary, a local operation guard accepts only the
selected `StartFirst` or `StartSecond` command and rejects Status/Stop. The
controller separately enforces nonce binding, ordered attempts, deadlines and
checked cleanup. No application-role policy is involved.

The random address is set only while checked inactive. Setter completion does
not establish the address in use: the SDK's local-used-address API follows
`own_addr_type`, which may still report a public or previous address before
start. The protocol therefore reports prestart query availability/value/status
separately. After the start completion it requires an actual random-type used
address equal to that phase's fixed control. Prestart unavailability is retained
explicitly; it is never replaced with the configured address. Active query
failure or mismatch fails the attempt.

Stop requires its successful GAP completion and inactive event state. It retains
the initialized controller and random RAM settings until reset; it does not
clear to a public address, deinitialize the driver or prove RF silence. Stopped
used-address queries report their actual availability. Each phase has an 8000 ms
main-task software Stop deadline armed before setup, checked while UART is idle
or backpressured. The adapter checks the phase deadline before requesting Start.
Failed cleanup retries at a requested one-second interval; unsolicited late
Start activity after failure requests cleanup again. SDK blocking and RTOS
scheduling prevent a hard real-time or on-air-duration guarantee. No unsolicited
automatic-stop UART line is emitted.

These are SDK host-stack reports. In the pinned source,
`bta_dm_ble_advstop` emits the GAP event from `BTM_BleAdvStop`'s returned status;
a later HCI advertising-disable failure is only logged by
`btm_ble_write_adv_enable_complete`. A good GAP Stop therefore does not prove
that the controller disabled RF. The used-address API likewise reads host-stack
address state rather than independently querying a controller register. Both
limits require the independent observer and external reset/restoration gates.

## SDK and safety boundary

Rust 1.96.0, edition/style edition 2024, `riscv32imac-esp-espidf`, build-std and
panic abort are selected. Sys/HAL use esp-rs revision
`a209052f378be61dd0f2d566b7fbe0f9593b0cd5`. ESP-IDF v5.5.6 is
`880a203ae603d08a94d06aa01e0d6761958e5e9e`; embuild is 0.33.7 and libclang
21.1.8. The same revision's safe `BtDriver` was evaluated but cannot compile
with both GATTC/GATTS disabled: its `bt/ble.rs` unconditionally compiles GATT
modules whose generated SDK functions are absent. SVC is a source reference,
not a linked dependency of this binary.

`initialization.rs` retains safe HAL Modem ownership and initializes through
generated SDK bindings. Its 65-field configuration transcribes the pinned SDK's
C6 `BT_CONTROLLER_INIT_CONFIG_DEFAULT` in `components/bt/include/esp32c6/include/esp_bt.h`,
compared with the C6 branch of pinned SVC `BtDriver::init`. There is no authored
C shim or handwritten ABI layout. The singleton initialization-attempt latch is
never cleared, including after partial failure. Fully initialized ownership is
retained without Drop/deinit. Partial initialization cannot be represented as
clean stopped ownership: it remains terminal and requires external reset.
The generated controller configuration uses one bounded allocation deliberately
retained until reset even after partial failure. The SDK forwards its pointer
to a binary controller library; this adapter does not assume that pointer is
unused after initialization returns, and creates no later Rust reference to it.

`sdk.rs` is the other small unsafe boundary. One static GAP callback reads only
the generated union member selected by its SDK event, rejects null/unknown
events and retains no pointers. It uses 32-bit atomics only, without callback
allocation, locks, logging or blocking. The main task alone arms/takes completion
results and owns UART/configuration. Release/acquire publication and a distinct
publishing state prevent cleanup from rearming a slot while its prior callback
publishes. A completion of another kind, duplicate completion or nonzero status
latches permanently; a late previous completion cannot satisfy a different Stop.
An in-flight/failed operation has unknown reported activity until a successful
event. SDK callbacks have no request ID or generation. Normal nonoverlapping
operations rely on the pinned SDK's FIFO API/callback queues and one event per
processed operation; a delayed duplicate of the same kind during a later
operation is indistinguishable. The mailbox only rejects a duplicate that is
unsolicited or mismatches the current kind. After a timeout, a repeated Stop may
observe an earlier queued Stop event. The permanent failure latch prevents that
cleanup from admitting the session or authorizing another Start; no exact
same-kind event attribution or authenticated SDK execution is claimed.

The raw AD array remains owned until reset. Pinned
`esp_ble_gap_config_adv_data_raw` uses the synchronous BTC deep-copy hook before
return; queued processing frees the SDK's copy through its deep-free hook.
Random-address and advertising-parameter calls likewise copy their fixed inputs
into queued arguments. Callbacks borrow only their SDK-supplied union for the
duration of the call. All unsafe sites state pointer, lifetime and ownership
invariants; no panic may unwind across the callback ABI.

Defaults disable IEEE 802.15.4, GATT client/server, SMP, bonding/cache NVS,
normal logs/consoles and PHY calibration storage. The NVS partition is retained
and is not opened by application code. Check generated sdkconfig before making
a selected-build assertion. Wi-Fi is never initialized by this application;
the SDK's hidden `ESP_WIFI_ENABLED` symbol defaults to true on C6 and cannot be
disabled by sdkconfig defaults. That compiled capability is distinct from an
initialized or started Wi-Fi driver. This is an ESP-IDF/std program: initialization and
SDK tasks allocate and may block. The pure library itself is no_std without
alloc; protocol serialization uses fixed buffers. Private records serialize
nonce/address/readback/status only for private captures; all value-bearing Debug
implementations redact them. Stateless frame errors are typed and static. The
internal Radio adapter preserves raw i32 SDK statuses as private protocol data;
it is not a general error-reporting API. There is no cancellation by dropping a
live operation or resource.

## Offline checks and image boundary

An ignored `.embuild` link selects the cached SDK/tools. From this directory:

```sh
C6_WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
export PATH="$C6_WORKSPACE_ROOT/.tools/bin:$PATH"
export LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib
export CARGO_WORKSPACE_DIR="$PWD"
export RUSTC_BOOTSTRAP=1
cargo +1.96.0 test --locked --offline --lib --target aarch64-apple-darwin --config 'unstable.build-std=[]'
cargo +1.96.0 clippy --locked --offline --lib --target aarch64-apple-darwin --config 'unstable.build-std=[]' -- -D warnings
cargo +1.96.0 build --release --locked --offline --features firmware --bin device-identity-ble-firmware
cargo +1.96.0 clippy --release --locked --offline --features firmware --bin device-identity-ble-firmware --no-deps -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.96.0 doc --release --locked --offline --features firmware --no-deps
cargo +1.96.0 fmt --all -- --check
```

Use the actual native host triple for host checks elsewhere. Release retains
optimization 3, thin LTO, one codegen unit, debug info and abort/reboot behavior;
CPU is 160 MHz, RTOS ticks 1000 Hz, main stack 16 KiB, flash 8 MiB DIO/40 MHz.
The existing three-MiB factory partition description is preserved. It does not
authorize enlarging the experiment's separate one-MiB selected-image erase slot.
Generate the selected image offline with upstream `espflash save-image`; inspect
its exact size, segment addresses and sector-rounded extent against that prior
slot before any deployment method is frozen. A failed fit must remain a build
limitation.

Neither host tests, target compilation nor SDK readbacks qualify scanning,
transmitted identity, full-window capture, physical silence or privacy
improvement. Hardware deployment requires an independently verified fresh full
8 MiB backup, approved segmented write bounds, a qualified independent BLE
observer, private complete capture, checked cleanup, full-image restoration and
verification followed by the normal reset. Those operations are external to
this workspace and are not performed by its build/test commands.
