# ESP32-C6 SoftAP adapter

`c6-anonymization-esp-idf` prepares a volatile base address and two SoftAP
MAC/SSID identities during one exclusive startup entropy window. It exposes
explicit First Start, checked Stop and Second Start operations. Initialization
never starts Wi-Fi; no operation programs eFuses, changes bridge configuration,
erases keys or restores a factory-derived address. The
[example](../examples/softap) owns UART commands and its cleanup deadline.

Only ESP32-C6 with ESP-IDF 5.5.6 and the pinned esp-rs revision is supported.
The firmware's actual configuration must disable other radios, persistent
Wi-Fi/PHY storage and automatic radio startup. A changed SDK getter does not
establish an emitted identity, startup coverage, RF silence or anonymity.

## API and ownership

`unsafe SoftAp::initialize_at_startup(Modem<'static>)` requires normal application
startup before any ADC, RF, network-interface or competing MAC-table user.
This includes tasks, callbacks and interrupts; the unsplit HAL Modem must be
uniquely owned. The caller must exclude later competing raw SDK users too.
A process-wide atomic attempt latch is consumed before hardware operations.
There is no supported retry after partial initialization in the same boot.

The adapter calls the existing `bootloader_random_enable`, fills exactly 34
bytes, then calls `bootloader_random_disable` before fallible processing or
Wi-Fi initialization. SAR enable resets/configures ADC resources, and disable
does not restore arbitrary prior ADC state. The startup precondition is necessary.
Known zero/bring-up-bypass samples and identity collisions reject; these checks
are not entropy-health or freshness tests. The bring-up RNG bypass is also
rejected by a target compile guard.

The first six sample bytes produce a local-unicast BASE; each following
14-byte group produces one independent MAC/SSID identity. All three normalized
MACs must differ, and the two SSIDs must differ. BASE remains unchanged through
both epochs. BASE and the first AP table entry are set before interface-table
reads or `WifiDriver::new`. After initialization with no NVS handle, the adapter
selects RAM storage, configures AP-only mode and sets/checks the driver AP MAC.
Before each Start it reads back the exact selected SSID, channel and open
authentication configuration as well as MAC. The SDK's nonempty default SSID
is cleared before the selected identity is copied. Neither the public SDK nor
this library can attest that an application honored the no-earlier-users rule.

`start(Epoch)` consumes its attempt before configuration. Second requires an
explicit checked Stop after First. It changes MAC/SSID only while the AP is
inactive, requests Start, then requires event-backed active state and exact MAC
readback. Every order, SDK, timeout or readback failure sticks and prohibits
future Starts. `stop()` can be retried, retains the selected masked values and
does not clear the first failure. State observations wait at most one requested
second, checking elapsed time before admission; blocking SDK calls and RTOS
scheduling do not have a hard bound.
The wrapper's running state is an event cache initially marked Stopped. After
a failed or in-flight Start, false can predate the cleanup Stop; it is not a
fresh command-correlated hardware acknowledgment. The failure latch still blocks
later Starts. Failed physical trials require the external reset/recovery path.

`identity()` exposes the expected observable MAC/SSID for integration.
`readback_mac()` and `active()` query SDK state; diagnostic failures also latch.
False from `active()` can mean failed or unverified state and does not establish
silence. `failure()` retains the first fixed diagnostic; `Error::code()` returns
the original SDK status or a fixed local status for private UART reporting.
Debug redacts identities. No live driver handle is exposed and the adapter is
not Sync. Its methods run synchronously in task context and can allocate,
sleep, lock or block through the SDK; there are no authored callbacks or ISRs.

The initialized driver and event loop use `ManuallyDrop`. Dropping this handle
deliberately retains both until reset and **does not stop RF**. Call Stop after
every Start attempt, including failures, and retain the handle while retrying
cleanup. This avoids the pinned wrapper's fallible stop/deinit destructor.
Initialization failures retain any created resources for chip-reset recovery.
The example continues polling its cleanup deadline and never returns after a
driver might have been initialized. Panic abort/reboot invalidates the epoch.

The only authored FFI calls use generated bindings and synchronous fixed-length
buffers: 34 writable RNG bytes, six-byte MAC inputs/checked table outputs, and
the fixed RAM-storage enum. No foreign pointer is retained. The caller's
exclusive startup contract protects the SDK's unsynchronized global MAC table.
The attempt latch uses acquire/release ordering to reject concurrent attempts;
it does not serialize unrelated SDK users.

## Builds and checks

Rust 1.96.0, language/style edition 2024, `riscv32imac-esp-espidf`, scoped
build-std for std/panic-abort, LLVM/libclang 21.1.8 and the existing cached SDK
are used. esp-rs HAL/SVC/sys pin `a209052f378be61dd0f2d566b7fbe0f9593b0cd5`;
ESP-IDF 5.5.6 pins `880a203ae603d08a94d06aa01e0d6761958e5e9e` and embuild is
0.33.7. The firmware owns its final SDK/partition configuration. No license has
been selected for this internal unpublished project.

From this directory, with `.embuild` pointing to the existing SDK cache:

```sh
C6_WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
export PATH="$C6_WORKSPACE_ROOT/.tools/bin:$PATH"
export LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib
export CARGO_WORKSPACE_DIR="$PWD"
export RUSTC_BOOTSTRAP=1
cargo +1.96.0 build --release --locked --offline
cargo +1.96.0 clippy --release --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.96.0 doc --release --locked --offline --no-deps
cargo +1.96.0 fmt -- --check
```

Pure plan/sequence tests run on an explicitly selected host target with
`--config 'unstable.build-std=[]'`. They cover collision normalization, sentinels
and epoch/Stop ordering without SDK or device access. Builds and host controls
do not establish hardware or receiver behavior. Before any image write, preserve
and independently verify a fresh full-flash backup; restore/verify it after the
trial. Original identity experiments retain their own frozen source and results.

The project API checklist records common guideline decisions. Adapter-specific
decisions here apply the workspace unsafe/FFI contract and cover C-VALIDATE,
C-CUSTOM-TYPE, C-DTOR-FAIL,
C-DTOR-BLOCK, C-SEND-SYNC, C-GOOD-ERR, C-DEBUG and C-EXAMPLE. Unsafe startup is
explicit rather than a safe interface that conceals uncheckable global/ADC
preconditions. No shutdown success or historical RF qualification is inferred
from Drop or a newly built image.
