# Device firmware

[Project](../README.md) · [Protocol](../docs/protocol.md) · [Results](../results/README.md)

Rust firmware for **ESP32-C6-DevKitC-1-N8**: UART0 through CP2102N, native USB
Serial/JTAG, info/ping, an on-demand passive Wi-Fi survey and bounded
[passive observation windows](../docs/observation.md) for all three radios.
Both control ports compile into
the same image. [Mac native USB checks](../results/observer-qualification-2026-10-24/image-03-results.md)
cover commands and repeated receive windows; iPhone operation remains unqualified.

## Build

The independent firmware workspace pins Rust 1.96.0 (edition/style 2024),
ESP-IDF v5.5.6 and embuild 0.33.7. Sys/hal/svc package versions are
0.38.1/0.47.0/0.53.0 from the unmodified upstream Git revision
[`a209052f378be61dd0f2d566b7fbe0f9593b0cd5`](https://github.com/esp-rs/esp-idf/tree/a209052f378be61dd0f2d566b7fbe0f9593b0cd5).
That revision initializes the Wi-Fi task stack-size field added by IDF 5.5.6;
the published svc 0.53.0 crate leaves it zero and the initial board scan failed
with SDK error 257. Exact Git sources are locked, rather than a moving branch.
The SDK commit is
`880a203ae603d08a94d06aa01e0d6761958e5e9e`.
Use libclang **21.1.8** for the verified macOS build. The initial older-wrapper
attempt with libclang 23.1.2 failed; the old OS baseline is not modified.

From this directory, with Homebrew LLVM 21 installed:

```sh
C6_WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
cargo +1.96.0 install ldproxy --version 0.3.4 --locked --root "$C6_WORKSPACE_ROOT/.tools"
export PATH="$C6_WORKSPACE_ROOT/.tools/bin:$PATH"
export LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib
export RUSTC_BOOTSTRAP=1
export CARGO_WORKSPACE_DIR="$PWD"
cargo +1.96.0 build --release --locked
cargo +1.96.0 clippy --release --locked --no-deps -- -D warnings
cargo +1.96.0 doc --release --locked --no-deps
cargo +1.96.0 fmt -- --check
```

Cargo build-std builds std and panic_abort for `riscv32imac-esp-espidf`.
Set RUSTC_BOOTSTRAP before Cargo starts, as well as the scoped configuration.
The shared linker is installed under the workspace's ignored `.tools/bin/`.
This checkout's ignored `.embuild` link uses the unmodified IDF 5.5.6 SDK and
downloaded tools under workspace `.cache/espressif/idf-5.5.6/`; it does not depend
on the OS project's directory. Generated configuration and output stay in this
project's own `target/`. Provision the pinned SDK at the metadata's `.embuild` path before
building: the extra bindgen input directly references its unmodified
`components/ieee802154/include/esp_ieee802154.h`. No authored C shim is used.
Set `CARGO_WORKSPACE_DIR` to this firmware directory when changing
`CARGO_TARGET_DIR`: the pinned build helper otherwise infers the workspace from
the output path and can select a different `.embuild` SDK cache.

Configuration: 160 MHz CPU, 1000 Hz RTOS tick, 32 KiB main task stack, 8 MB flash,
DIO/40 MHz and a 3 MiB factory partition in `partitions.csv`. Wi-Fi, Bluedroid BLE and
IEEE 802.15.4 are compiled in. GATT and SMP symbols remain enabled because the
pinned Rust service compiles those wrappers unconditionally; the application
does not start their services or pairing. BLE bond persistence, Wi-Fi persistence,
PHY calibration persistence, dynamic power management and SDK consoles are disabled.
SDK/runtime initialization allocates; callback packet storage is eight static slots.

Firmware revision 2 uses **921600 baud** on UART0; revision 1 used 115200.
The Cargo configuration copies `partitions.csv` into the SDK build using the
documented `ESP_IDF_GLOB_*` mechanism. The 1,712,496-byte recorder image exceeded
the SDK's 1500 KiB built-in factory partition during packaging; that rejected
attempt is retained. The 3 MiB factory layout leaves the remaining flash unused.
The native USB port does not use the UART baud rate. Recorder changes and
validation limits are described in the [Rust review](../docs/api.md).

Release optimization is level 3, thin LTO, one codegen unit, panic abort, with
debug information retained. SDK compilation is optimized for performance.
Compile-time metadata is still enabled; hashes identify actual artifacts, not
a promise of byte-identical future rebuilds.

## Deploy and test

Preserve and verify the board's current **full 8 MB image** under ignored
`backups/wireless-explorer/` before modifying flash. Never change eFuses for
this probe. Use the actual bootloader and partition table from the successful
esp-idf-sys output directory; stale failed builds can leave other directories.

Create a merged image with `espflash save-image --chip esp32c6 --flash-size 8mb
--flash-mode dio --flash-freq 40mhz --merge --skip-padding --bootloader <actual>
--partition-table <actual> <firmware-ELF> <output.bin>`. Record its hash and flash
it at offset zero through the verified CP2102N port. A successful transfer is
not a firmware pass; use the [desktop client](../clients/README.md) afterward.

The [first probe procedure](../experiments/control-bringup.md) defines a bounded
test and restoration. Do not select an unrelated serial port by position.
Use the [shared board map](../../../docs/hardware.md) for physical connections.
