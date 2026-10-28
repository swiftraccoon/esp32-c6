# SoftAP library integration example

This ESP32-C6-DevKitC-1-N8 example uses the [Rust adapter](../../esp-idf/README.md)
to prepare a volatile base and two random AP MAC/SSID identities before Wi-Fi
initialization. It initializes an inactive AP at boot and waits for explicit
UART commands before requesting transmission. It has no station, network
interface, DHCP, BLE, IEEE 802.15.4 or NVS application service.

The example is a private verification aid. Its UART replies intentionally expose
the generated addresses/SSIDs to the attached host; this transport and both USB
serials are outside its radio-identity claim. Keep all UART bytes private. A
generic library integration does not need this UART protocol.

## Integrating the adapter

Add `c6-anonymization-esp-idf` as a path dependency on `../../esp-idf` from this
example, or the corresponding checkout path from another application. Use its
pinned SDK configuration and read the complete startup safety contract first.
The essential application sequence is:

```rust,ignore
use c6_anonymization_esp_idf::{Epoch, SoftAp};
use esp_idf_hal::peripherals::Peripherals;

let peripherals = Peripherals::take()?;
// SAFETY: exclusive early startup, before ADC/RF/MAC users or competing tasks;
// the selected build disables other radios and persistent PHY/Wi-Fi storage.
let mut ap = unsafe { SoftAp::initialize_at_startup(peripherals.modem)? };
// Prearm an application cleanup deadline before each potentially blocking Start.
ap.start(Epoch::First)?;
// Application work; Stop is mandatory after every attempt, including errors.
ap.stop()?;
ap.start(Epoch::Second)?;
ap.stop()?;
```

This abbreviated sequence omits error cleanup and is not a complete executable.
The actual example retains ownership, deadlines and sticky failures. Drop does
not stop RF: the adapter deliberately retains initialized resources until reset.
Both identities are sampled before Wi-Fi initialization; the second changes AP
MAC/SSID while stopped and leaves the boot's masked base unchanged.

## Building and host controls

The example pins Rust 1.96.0, edition/style 2024, ESP-IDF 5.5.6 revision
`880a203ae603d08a94d06aa01e0d6761958e5e9e`, esp-rs
`a209052f378be61dd0f2d566b7fbe0f9593b0cd5`, embuild 0.33.7 and its own lockfile.
Build from this directory so Cargo loads `.cargo/config.toml`. With the existing
SDK cache linked at ignored `.embuild`:

```sh
export PATH="$(git rev-parse --show-toplevel)/.tools/bin:$PATH"
export LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib
export CARGO_WORKSPACE_DIR="$PWD"
export RUSTC_BOOTSTRAP=1
cargo +1.96.0 build --release --locked --offline --features firmware
cargo +1.96.0 clippy --release --locked --offline --features firmware -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.96.0 doc --release --locked --offline --features firmware --no-deps
cargo +1.96.0 fmt --all -- --check
```

The external bootstrap setting is needed before Cargo decides to build the
target standard library. The actual generated SDK configuration must disable
the RNG bring-up bypass, other radios, normal consoles/logs and persistent
Wi-Fi/PHY storage. The adapter rejects a bypass-enabled target build.

Pure request/deadline controls run without SDK or hardware on a selected host:

```sh
cargo +1.96.0 test --locked --offline --lib --target aarch64-apple-darwin --config 'unstable.build-std=[]'
cargo +1.96.0 clippy --locked --offline --lib --tests --target aarch64-apple-darwin --config 'unstable.build-std=[]' -- -D warnings
```

Use the actual host triple on other systems. Target compilation and host
controls do not establish on-board behavior or complete startup observation.

## Private UART protocol

UART0 uses GPIO16 TX/GPIO17 RX, 115200 baud, 8N1. Every bounded JSON request is
LF-terminated and contains `domain: "anon_softap"`, `version: 1`, an increasing
nonzero `id`, and `operation`: `status`, `start_first`, `stop` or `start_second`.
Unknown/duplicate fields, wrong namespaces, late/repeated IDs and malformed or
oversized frames fail. This is a trusted-host correlation protocol, not
authentication. It has no arbitrary address, channel or SDK command fields.

Replies contain the immediate status, sticky first failure, SDK active state,
current MAC readback and both expected private MAC/21-byte SSID arrays. Success
requires both zero status and no retained failure. Initial construction failure
emits a fixed initialization-failure record with its numeric error and never
retries. Debug implementations redact addresses and SSIDs.

The normal boot sequence is Status, StartFirst, explicit Stop, StartSecond,
explicit Stop. Each Start attempt is consumed and its requested eight-second
Stop deadline is armed before calling the adapter. A late SDK completion is
rejected. The loop polls during UART receive/transmit; malformed input or write
failure requests checked cleanup. Failed cleanup retries at most once per
requested second despite repeated transport errors. SDK blocking and RTOS
scheduling prevent a hard physical RF-duration guarantee.

Automatic Stop alone does not authorize the second request. Successful cleanup
never clears a prior failure. Stop retains masked identities instead of
restoring factory-derived values. Cached SDK inactive state is not independent
Stop-event attribution or RF-silence evidence. Recovery from a failed device
trial requires the external reset/restoration procedure.

The [prospective observation method](../../experiments/softap-identity.md)
defines the next board trial. No image is installed until receiver readiness,
qualification, input freeze and fresh full-image preservation are complete.
