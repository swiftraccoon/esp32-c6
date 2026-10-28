# Controlled SoftAP address presence witness

Decision question: does the attached ESP32-C6 emit at least one ordinary beacon
whose transmitter/BSSID equals a synthetic AP MAC set before driver start, with
the fresh private trial SSID? Compare the actual Wi-Fi driver readback with an
independent receiver. This is one pre-start override per boot, not rotation,
full-window observation, silence, anonymity or a delivery-rate experiment.

The [runtime table probe](runtime-table.md) observed cached software addresses
without starting radios. The [receiver qualification](wifi-observer.md) admitted
ambient radiotap/probe-request captures but no physical beacon. Reuse their
board/port map, SDK/toolchain pins, strict private records and receiver path.
The [prior-work](../../../docs/research/prior-work.md) and
[reuse catalog](../../../docs/research/benchmark-reuse.md) favor independent packet
observations; neither existing result proves an active AP address. New Rust
firmware/control and a nonce-bound beacon matcher supply that missing check.
No published performance number is compared.

## Fixed inputs and ownership

The target is ESP32-C6-DevKitC-1-N8. The independent
[firmware](../firmware-wifi/README.md) uses Rust 1.96.0, edition/style 2024,
`riscv32imac-esp-espidf`, scoped build-std/bootstrap, ldproxy 0.3.4 and libclang
21.1.8. Sys/HAL/service wrappers pin revision
`a209052f378be61dd0f2d566b7fbe0f9593b0cd5`; ESP-IDF 5.5.6 pins
`880a203ae603d08a94d06aa01e0d6761958e5e9e`. Preserve lockfiles, actual generated
configuration, linked Wi-Fi/PHY inputs, ELF/segments and tool identities. Host
Rust is 1.96.0; the std-only remote worker remains on its explicit Rust 1.90.0
pin. Kismet/receiver/decoder inputs retain the qualified revisions unless a
separately declared correction changes them.

UART0 uses GPIO16 TX/GPIO17 RX through the verified CP2102N at 115200 baud.
Concrete paths, original addresses, raw transcripts, nonce-bearing SSIDs,
packets, full flash images and credentials stay private. No eFuse, bridge
persistent configuration, pairing, provisioning or intentional key erasure is
part of this experiment. Native USB identity is not compared here.

The fixed synthetic locally administered unicast AP control is defined by
`wifi-protocol::AP_MAC`; it is not a collision policy or unique identifier.
Each host trial draws a fresh nonzero eight-byte nonce and forms `di-ap-` plus
16 lower-case hexadecimal digits. The SSID is a correlation marker broadcast
intentionally by this experiment, not authentication. Private UART readbacks
include the original and current six-byte AP MACs.

## Firmware behavior

Boot and Status before Start initialize UART/control only. A valid Start latches
one nonce for the boot and lazily constructs the AP-only Wi-Fi driver. Select
RAM storage before AP configuration; select channel 1, visible nonce SSID,
open authentication and at most one client. Create no EspNetif, DHCP, STA,
Bluetooth, IEEE 802.15.4 or provisioning service. Wi-Fi NVS and PHY calibration
storage are disabled in actual configuration. SDK defaults can compile lwIP/
esp_netif components without the application initializing them.

Save the original driver AP MAC, configure AP mode while stopped, set the
synthetic AP MAC and read it back before `esp_wifi_start`. Do not call the
post-start channel setter before start. After the AP-started event, require
active driver readback equal to the control. Setter/getter success alone does
not establish transmitted identity.

The controller requests automatic Stop eight seconds after the Start command
begins, independently of usable UART traffic. Scheduling and SDK calls can
exceed that deadline. A failed automatic Stop retains its first error and
retries cleanup at one-second intervals; no second Start is permitted until
reset. This bounds requested transmitter lifetime, not measured RF airtime or
an unconditional hardware shutdown guarantee.

Stop requires the AP to report inactive, restores the saved driver AP MAC while
disabled and verifies readback. Retain the initialized driver/event subscription
until reset; report `initialized=true` after ordinary Stop rather than claiming
deinitialization. Constructor failure can leave partial SDK state without an
owned handle; it is terminal/nonzero and cannot become a clean successful trial.
No normal post-initialization main return invokes hidden driver Drop cleanup.
The nonce SSID remains RAM configuration until reset; only AP MAC restoration is
claimed by Stop. Full flash restoration is a separate boundary below.

## Receiver readiness and witness

Use the existing source-scoped Kismet stream without tuning, probing interfaces,
pausing a source, opening the Nordic port or changing a service. Its legacy
32-EPB/120-second/8-MiB sample remains unchanged. An opt-in worker READY message
requires HTTP 200 and a completely saved framing-valid SHB; it binds the request,
worker/build identities, exact prefix length and prefix hash. HTTP headers and
credentials remain inside the worker.

SSH stderr can arrive before stdout. The host independently waits for the
corresponding locally saved/synchronized complete SHB, validates it and checks
that the actual SSH child is still alive immediately before sending Start.
Missing, denied, partial, duplicate, malformed or mismatched readiness known at
the final pre-Start authorization check suppresses that command. The asynchronous
check is not an atomic transaction with the UART write; later invalidity or child
exit can race Start and must fail the trial and trigger cleanup. Record a distinct
StartRequested event before writing that UART request.
Any race where the quota ends before target reception leaves the trial failed.

The receiver may reach 32 records while the AP remains active. Its receipt is
therefore a bounded prefix, never proof that the entire eight-second lifetime
or Stop interval was observed. Require at least one supported ordinary beacon
with an included verified CRC, no bad-CRC indication, broadcast destination,
unfragmented base management header, matching unicast transmitter/BSSID equal
to the AP control, and exactly the expected nonce SSID. Probe requests, another
SSID/MAC, an unchecked FCS or a transport-valid prefix without a target do not
satisfy the trial. A nonce does not authenticate the physical transmitter.

The matcher reuses the complete PCAPNG/radiotap/beacon validator. Every packet
and late block is validated; a good earlier witness cannot hide later corruption.
Only fixed states/counts leave the consumer. Keep collateral payloads and actual
identifiers private. Channel configuration/metadata, source-provided clocks,
loss/filtering and transmitter authenticity remain separately qualified.

## Trial schedule and admission

Before hardware writes, qualify the actual protocol/controller, worker READY,
UART host, save/replay path and witness matcher with passing and deliberately
failing controls. Include HTTP/SHB/binding/race failures with zero Start writes,
stale/nonzero IDs and nonces, UART startup/noise/truncation, partial/lost Start,
automatic Stop failure/retry, incorrect driver readbacks, bad/missing FCS,
wrong beacon role/SSID/MAC, late malformed records and failed MAC restoration.
A separate reviewer constructs contrary controls and reviews SDK/lifecycle/FFI
branches. Freeze source, method, generated configuration, tools and executable
identities in a commit before collection.

Plan exactly two trials with the same built image, separated by an explicit
normal reset and a fresh nonce. They are reset-separated boots, not cold power
cycles or reconnect tests. Start the first after deployment verification/reset.
Abort the cohort after its first failed trial; preserve it and investigate before
repeating collection or changing the method.

Each trial opens UART, explicitly releases DTR then RTS and records the separate
one-second bounded startup interval using the qualified C6 ROM grammar. Open/
release can reset the device and can discard pre-open bytes; no reset-free or
early-radio coverage is claimed. Require initial Idle/uninitialized/no-MAC state.
Then obtain bound receiver READY plus liveness, send Start and validate its
correlated active driver snapshot. Host waits eight seconds after the Start ACK
before normal Stop. Stop must report Stopped/inactive and the same saved MAC
from Start restored; initialization can remain true. Drain UART through receiver
completion and a 200-ms suffix check. Require complete receiver replay and a
nonzero qualified target-beacon count. Receiver transport success alone cannot
admit a failed trial.

The eight-second host wait is enforced by the live compiled path. Replay checks
event order and monotonic recorded host elapsed values, not an eight-second
duration threshold. Fast synthetic sessions qualify control flow with injected
waiting; neither those records nor the host wait establish RF airtime.

Host bounds are a 30-second readiness wait, five seconds per UART exchange,
256-byte requests, 1024-byte replies and 64 KiB raw UART storage. Receiver
transport keeps its existing bounds and 132-second SSH ceiling. These software
bounds do not provide a hard real-time guarantee for OS/SDK I/O.

Attempt idempotent Stop after any possible Start, including lost ACK or later
receiver/parse failure. If it cannot be confirmed, preserve failure and mark
recovery required; do not automatically reset in the capture module or convert
later successful cleanup into trial success. External recovery/full-flash restoration
always follows. A completed trial requires the exact 12 root files plus the
legacy seven-file receiver subdirectory, receipt/hash/request agreement, startup
and UART raw/transcript agreement, event order, readiness, Stop, witness and
saved-summary byte equality. `wifi-ap-replay` is read-only and opens no device.

## Preserve and restore the original device

Before any write, read a fresh complete 8,388,608-byte image into ignored
`backups/device-identity/`, retain its SHA-256 and separately verify that entire
range against the device without an intervening application boot. Pin esptool
4.12.0 and its actual Python/serial/reset inputs. Use explicit bounded connection
attempts and an explicit empty esptool configuration; retain commands/statuses
privately. Inspect security/encryption state and the original partition table.
Do not use force, erase-all or irreversible provisioning.

Package the actual generated bootloader, partition table and application as
separate segments, preserving their source and image/header identities. Deploy
aligned segments at 0, 0x8000 and 0x10000, with frozen keep-header parameters;
separately verify each written segment before reset into the probe. Assess the
writer's 4-KiB-rounded erase footprint, not only file lengths: bootloader below
0x8000, partition-table footprint only 0x8000..0x8fff, and application footprint
fully inside an ORIGINAL executable partition with no original data/key overlap.
Reject deployment if those gates fail. Do not write merged FF padding across
NVS/PHY data pages. Retain all failed read/write/verify/reset attempts.

After the fixed trials or any post-write failure, always enter ROM, restore the
fresh complete backup and independently verify all 8,388,608 bytes before release.
If a restoration write/verify fails, retain it and attempt the same exact
restoration once more; neither attempt is a new observation trial. Do not release
an unverified partial image as successful recovery.
This restoration physically rewrites stored-data sectors back to backup bytes;
it is not an intentional key deletion. Record separate normal-reset completion
with pinned line semantics. Esptool's reset notice precedes line operations:
require zero exit and no unexpected warning, allowing only the documented
no-reset preconnection notice. Unsupported/NOT-reset reports leave release
unresolved. The verified image equality is before reset; subsequent application
boot/health, persistence and radio activity remain unmeasured.

## Evidence and remaining scope

Replay both immutable successful directories and retain every rejected cohort.
Independently decode the entire saved packets with the frozen Wireshark tool and
check exact target/CRC/header/SSID matches privately. Publish only scoped counts,
readback/restoration states and source/artifact identities, with a complete
relative-path archive seal. Evidence seals do not authenticate raw sources.

This witness can establish observed pre-start AP override/readback and matching
beacon contents. It does not qualify STA/BLE/IEEE addresses, full startup or Stop
traffic, silence, rotation, factory-value erasure, population uniqueness,
cold boot, reconnect, network privacy or correlation reduction. USB/bridge
identity and application credentials remain independent ownership domains.
