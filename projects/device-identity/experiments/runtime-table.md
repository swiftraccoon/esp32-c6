# Runtime address tables and attached-host USB properties

Transport revision 2. The first revision's zero-RX attempt is retained at source
freeze `368d57f`; it preserved library DTR state without releasing RTS. A
separate line-release diagnostic obtained the probe's expected rejection of a
foreign request. The corrected collector explicitly releases both lines and
defines a separate startup interval. This does not isolate the physical state
of each control line or establish reset-free opening.

Decision question: do the pinned SDK's base/interface setters change the address
configuration table, leave previously cached interface values intact after a
base-only update, and permit restoration of the saved table? Does an attached
host continue to report its USB serial properties while those table values
change? This is a functional configuration probe, without a throughput or
anonymity score.

The closest prior observations are the [read-only baseline](read-only-inventory.md)
and the [UART firmware bring-up](../../wireless-explorer/experiments/control-bringup.md).
Reuse the shared board/port map, unmodified pinned SDK and Rust HAL build
integration. A project-owned protocol, firmware and collector keep the new
acceptance rules explicit. No existing RF capture or published number is reused
as a runtime-address observation.

## Inputs and execution boundary

Use Rust 1.96.0, edition/style 2024 and the committed host/firmware lockfiles.
The firmware target is `riscv32imac-esp-espidf`, built with `build-std`, scoped
`RUSTC_BOOTSTRAP=1`, ldproxy 0.3.4 and libclang 21.1.8. Rust sys/hal inputs are
the upstream Git revision `a209052f378be61dd0f2d566b7fbe0f9593b0cd5`;
ESP-IDF 5.5.6 is commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`.
Firmware runs as the FreeRTOS main task, 160 MHz CPU, 1000 Hz tick, 16 KiB
main stack and panic abort/reboot. Preserve actual generated configuration and
ELF/merged-image/collector/source identities privately before collection.

UART0 uses GPIO16/17 through the CP2102N at 115200 baud. Native USB remains
attached for host-property snapshots; the collector does not open native USB
serial. Both connectors provide Mac USB power/data. The fixture model is
ESP32-C6-DevKitC-1-N8, with reported PCB revision still unverified.

The application initializes UART and the SDK MAC table only. It initializes no
Wi-Fi, Bluetooth, IEEE 802.15.4, network, pairing or NVS service. Actual Kconfig
must be inspected: supported MAC-table entries and compiled driver components
are not equivalent to running services. FOUR universal-address allocation is
selected independently of radio-driver initialization. No conclusion about RF
silence follows from source/configuration inspection.

## Fixed schedule and acceptance

Collect exactly two revision-2 sessions with the same built image, separated by
an explicit hard reset through the bridge. The first also begins after an
explicit reset and one second of startup settling before opening the UART.
Neither run is a cold power cycle or cable-reconnect test.
Freeze this method, source and compiled input digests in a commit before the
sessions. If qualification exposes an error, retain that attempt, correct the
method/implementation and freeze a new version before collecting again.

Each session has four requests with correlated response IDs:

1. `defaults`: read the eight-byte factory MAC-plus-extension representation,
   runtime base, Wi-Fi station/AP, Bluetooth and IEEE 802.15.4 table values.
   These getter calls deliberately populate the SDK's interface caches. Capture
   the Mac's current USB properties while the table remains at defaults.
2. `base_override`: set a fixed synthetic locally administered unicast base.
   Read all fields again. The base must equal the requested control; factory
   and previously cached interface values must equal the first snapshot.
3. `interface_override`: set four fixed, distinct synthetic interface controls.
   Read back the requested base and interface table values, preserving factory
   bytes. Capture USB host properties while those controls remain configured.
4. `restored`: restore saved base/interface table values and verify complete
   snapshot equality with `defaults`. Preserve an SDK failure as a failure,
   including when a partial set succeeded before a later operation failed.

The controls are deliberately public test inputs, not randomization or
collision-avoidance policy. Their binary representations are never transmitted
by an application radio operation in this probe. Getter agreement qualifies
only SDK table state; active drivers may manage separate state.

The raw transcript must retain complete request/response pairs, expected IDs,
phase order, SDK statuses and fixed-width snapshots. Strict replay checks the
same sequence and relationships as live collection. Missing/extra records,
unexpected IDs/phases/versions, malformed fields, nonzero SDK status, conflicting
snapshots and failed restoration cannot be admitted by dropping records.
Capture chooses a fresh nonzero OS-random starting token and four successive
request IDs; the reply must echo each exact ID. These tokens correlate messages
without authenticating a device. `runtime-summary` validates table transcripts
only; admit a live session through directory-aware `runtime-replay`, including
completion/profiler status, raw/transcript agreement and saved summary equality.

The USB comparison uses the same schema validator as USB summary. Compare
serial-property strings only when exactly one matching device per domain has
a nonempty property in each snapshot. Multiple matches are ambiguous; missing
or empty properties remain unavailable. Equal counts alone cannot establish
equal serial values. These are current host properties and can be cached from
enumeration. No fresh descriptor read, native re-enumeration, factory-serial
derivation, reconnect stability or persistence conclusion follows.

## Qualification before flashing

Run positive and contrary controls through the actual protocol, CLI consumer,
save and replay path. Cover correct table changes/restoration; a base-only
change that incorrectly rewrites cached interfaces; wrong controls; changed
factory bytes; missing/duplicate/truncated/out-of-order frames; stale IDs;
nonzero SDK status; late errors; privacy sentinels; and USB comparisons whose
counts agree while serial contents differ. Exercise the transport with a fake
or local test stream without opening the board. A separate reviewer reproduces
controls, traces FFI sizes/selected SDK branches and records unresolved issues.

The C6 factory-default API can write eight bytes when IEEE 802.15.4 capability
is present; a six-byte buffer is unsafe even when querying other six-byte table
entries. Generated bindings and bounded arrays define the FFI buffer contract.
All MAC calls occur serially in the main task, before any radio initialization.

## Preserve and restore the board

Before flash writes, read a fresh complete 8,388,608-byte image into ignored
`backups/device-identity/`, preserve its SHA-256, and separately verify the full
range against the device without an intervening application boot. Use the
verified bridge and pinned esptool, retaining command output/status privately.
Do not program eFuses, erase keys or write bridge configuration.

Build/package with the actual bootloader and partition table from the successful
firmware build. Use 8 MB, DIO/40 MHz and the explicit application partition map.
Write the merged image at offset zero, separately verify the written range,
then reset into the probe. Retain all failed writes/startups/sessions. A transfer
completion is not an accepted runtime observation.

Use a new ignored private directory for every capture. Save all received bytes,
requests/responses, profiler stdout/stderr/status and failure markers. Preserve
port paths and raw identities privately. The capture command attempts the
restore request after a later failure where possible; this does not replace an
explicit reset or the full-flash restoration boundary.
After opening, revision 2 explicitly deasserts DTR, then RTS, checking both calls.
Releasing auto-programming lines may start/reset the application. Save a bounded
one-second startup window separately before the first request, capped at 8192
bytes and 256 bytes per line. Permit empty startup or one complete C6 ROM boot
block with validated field grammar and order (`ESP-ROM:`, `Build:`, `rst:`,
optional `Saved PC:`, `SPIWP:`, `mode:`, load records and `entry `). Reject
panic/error diagnostics, unexpected text, truncation and repeated boot blocks.
Record the requested line policy and startup
receipt; replay checks the same bytes and metadata. All non-whitespace non-JSON
traffic remains invalid after request collection starts.

The serial library/OS can flush pre-open buffers or change device state during
open/close. Raw coverage starts after successful open; discarded earlier traffic
remains unavailable. Capture completion does not qualify reset-free startup,
physical line levels or reconnect handling.

After the fixed sessions, or if the experiment cannot continue, restore the
fresh full-flash backup, separately verify all 8,388,608 bytes and release the
existing application with a hard reset. Retain the commands, statuses and UTC
completion time. This verifies bytes before reset; later application persistence
may change. No completed outcome is valid as restoration evidence by itself.

## Publication

Publish fixed-field sanitized table summaries, USB comparison states and a
claim/coverage ledger. Keep unique addresses, raw USB enumeration, transcripts,
full images and concrete serial paths private. A complete public result seal
covers all retained regular files except itself, with relative path convention
stated. Input/artifact identities and actual generated configuration support
only this probe. Radio rotation while active, early-boot exposures, cold boot,
reconnect/recovery, provisioning, stored identities and correlation reduction
remain separate experiments.
