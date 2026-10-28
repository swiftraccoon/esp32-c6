# Runtime identity probe firmware

Rust firmware for ESP32-C6-DevKitC-1-N8, controlled through CP2102N UART0 at
115200 baud, 8N1, GPIO16 TX/GPIO17 RX. The native USB port stays available to
the attached host for enumeration snapshots. No Wi-Fi, Bluetooth, IEEE 802.15.4,
network interface, provisioning, NVS or PHY service is initialized by this code.
Software getters do not establish transmitted identities or RF silence.

The [shared protocol](../crates/runtime-protocol/src/lib.rs) defines four
ordered phases: defaults, late base override, explicit interface table overrides
and restoration of the saved table. The first phase populates the SDK's cached
interface addresses. Changing BASE later does not invalidate those entries.
Only the explicit per-interface setters change the cached values. All synthetic
override addresses are fixed locally administered unicast test values. They are
not a randomization scheme or population-unique identifiers.

Snapshots include private factory/base/interface values and must be captured
under ignored `private/device-identity/`. Debug formatting of protocol snapshots
is redacted. A failed command returns its status without a partial snapshot.
Restoration is available whenever the initial snapshot was saved, including
after a partial mutation or response failure, and attempts every saved setter.
Transport failure or a failed restoration requires the host's reset/recovery
procedure; a setter success alone is no restoration observation.

## Build inputs

The independent workspace uses Rust 1.96.0, language/style edition 2024,
`riscv32imac-esp-espidf`, build-std for std/panic_abort, and libclang 21.1.8.
Sys/HAL are pinned to upstream revision
`a209052f378be61dd0f2d566b7fbe0f9593b0cd5`; embuild is 0.33.7. ESP-IDF 5.5.6
is commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`. The ignored `.embuild`
link selects the workspace's cached SDK/tools. Enter this directory before
running Cargo so target configuration is loaded.

```sh
C6_WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
export PATH="$C6_WORKSPACE_ROOT/.tools/bin:$PATH"
export LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib
export RUSTC_BOOTSTRAP=1
export CARGO_WORKSPACE_DIR="$PWD"
cargo +1.96.0 build --release --locked
cargo +1.96.0 clippy --release --locked --no-deps -- -D warnings
cargo +1.96.0 doc --release --locked --no-deps
cargo +1.96.0 fmt -- --check
```

The release profile has optimization level 3, thin LTO, one codegen unit,
debug information and panic abort. Hardware configuration is 160 MHz CPU,
1000 Hz RTOS tick, 16 KiB main stack, 8 MB flash DIO/40 MHz and a 3 MiB factory
app partition. SDK consoles, ordinary logs, Bluetooth and IEEE driver support,
Wi-Fi/PHY persistence and dynamic power management are disabled in defaults.
The C6 hidden Wi-Fi capability symbol may remain enabled even though this
application never initializes Wi-Fi; inspect actual generated sdkconfig and
compiled/link inputs before making a configuration claim.

Generated bindings include the pinned `esp_mac.h`. C6's direct factory API
writes eight bytes, including MAC_EXT; its buffer is eight bytes. Every other
pointer call checks the SDK's reported width against its Rust array size.
The small FFI module owns these invariants. Calls run synchronously from the
main task with no concurrent MAC-table users or retained foreign pointers.
The table is ordinary unsynchronized SDK global RAM, so adding radio services
or another MAC API caller requires a fresh concurrency review.

UART driver setup and SDK std initialization allocate. Protocol framing and
snapshot/line buffers have fixed capacities. The main task can sleep or block
on SDK I/O; there are no authored callbacks or ISR operations. Partial writes
retry with a two-second application deadline, but SDK calls do not have a hard
real-time guarantee. Panic abort/reboot clears the live table and invalidates
the in-progress host session. No persistent write or eFuse programming API is
called by the application.

Before deployment, preserve and verify the complete current 8 MB flash image.
Use the actual generated bootloader/partition table to package the executable,
and restore/digest-verify the pre-session image after the experiment. Builds
alone do not qualify board behavior or restoration.
