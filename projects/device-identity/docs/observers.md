# Identity observers and qualification

An observer's API, permissions and capture format determine which claims it can
support. Device enumeration and packet counters do not qualify delivery fidelity,
address visibility, silence or correlation reduction.

## Attached Mac

The Mac supplies UART access, current USB properties and native control-pipe
requests. In the discovery snapshot, its single Wi-Fi interface was active and
carried the default route.
The installed tcpdump manual warns that monitor mode can disassociate it;
read-only discovery did not change that interface or start a radio scan.

CoreWLAN's local SDK headers require Location Services authorization to return
BSSID fields. CoreBluetooth exposes peer UUIDs and advertisement data rather
than a documented raw advertising-address field. Their observer permissions and
raw-address coverage remain unqualified. BlueTool is installed but its private
control behavior was not invoked. The read-only class authorization preflight
is a possible future CoreBluetooth check before creating a manager.

## Existing remote Linux receiver

Saved inventory identifies an nRF Sniffer for Bluetooth LE (`1915:522a`), Realtek
RTL8812 USB Wi-Fi (`0bda:8812`) and Intel AX200 Wi-Fi/Bluetooth on a remote Linux
host. Exact host, USB serials, addresses, source UUIDs and credentials remain in
private inventory.
SSH and Kubernetes inventory established availability at discovery without
changing the cluster, deploying a pod or claiming a device. Read-only checks on
2026-10-27 could not reach the saved SSH endpoint; the configured Kubernetes API
refused connections. Current source, interface and process state therefore
remain unverified. Fresh reachability and method-specific readiness checks are
required before reusing this receiver.

The saved Realtek driver was `rtw88_8812au`. Its monitor interface was already up
on 2.4 GHz channel 1, while the host's default route used its separate Intel Wi-Fi
interface. Kismet owned the monitor receiver and the Nordic serial port; their
existing configuration was preserved. No helper was stopped, serial command
sent, receiver retuned or packet stream started during discovery.

Kismet reported version `2025.09.0-a868177ef`, matching upstream commit
`a868177ef92dc8ea0b6fb121976eeddba1854d43`. Initial read-only API snapshots showed:

| Source | API state | Cumulative packets across two snapshots | Scope |
| --- | --- | --- | --- |
| Realtek Wi-Fi / linuxwifi | Running, no error, not paused | 5436 to 5465 | Intake advanced; delivery fidelity and target-device reception unqualified |
| Linux Bluetooth | Running, no error, not paused | 7 to 7 | Existing packets; no new reception established in that interval |
| Nordic / nrf51822 | Running, no error, not paused | 0 to 0 | Serial ownership/startup does not establish scanner readiness or RF silence |

The API reported no hopping but an empty current channel, so the channel-1
observation comes from a separate read-only `iw` snapshot. It is not a guarantee
that the source remains on that channel indefinitely. Source DLTs are radiotap
127 for Wi-Fi and BLE-with-pseudo-header 256 for Nordic. The later
[bounded Wi-Fi collection](../results/wifi-observer-2026-10-25/README.md) validated
two 32-EPB radiotap prefixes, saved replay and FCS handling. Zero beacons left
physical beacon decoding unexercised in that ambient collection. The subsequent
[controlled AP witness](../results/wifi-ap-2026-10-25/README.md) matched 64
CRC-verified nonce/TA/BSSID beacons from two probe boots, with no receiver tuning.
Full-window coverage, hot changes while running, RF silence and the Nordic
stream remain unqualified.

The [restart cohort](../results/wifi-restart-2026-10-25/README.md) subsequently
used four independent prefixes for two same-boot A-to-B sessions. All 128
records were retained: 120 exact CRC-verified target beacons and eight other
management frames. The latter left both optional whole-capture profile
comparisons unqualified. Source/interface/process/configuration and channel-1
pre/post checks agreed, with no receiver control or tuning. These bounded
presence witnesses do not establish complete transition coverage, silence,
delivery fidelity or hot changes while running.

The [matching Nordic helper source](https://github.com/kismetwireless/kismet/blob/a868177ef92dc8ea0b6fb121976eeddba1854d43/capture_nrf_51822/capture_nrf_51822.c)
accepts protocol versions 1/2/3 and reads the configured serial stream without
an explicit scan-start command. Zero packets leave firmware/protocol/scanning
state and RF conditions unresolved. Do not open a serial port owned by Kismet
or infer an unsupported radio from this count.

Existing per-user credentials authorized JSON GETs for status, types and source
inventory. Credentials stayed inside the host and were supplied to curl through
stdin; they were not placed in argv, URLs or public files. The maintained Rust
[worker](../tools/kismet-worker/README.md) reads approved remote credential paths
and keeps credentials/HTTP headers on that host. JSON inspection helpers remain
private research scratch.

## Packet stream and next controls

The [source-scoped packet endpoint](https://github.com/kismetwireless/kismet/blob/a868177ef92dc8ea0b6fb121976eeddba1854d43/datasourcetracker.cc#L967)
and [packet API documentation](https://www.kismetwireless.net/docs/api/packet_capture/)
provide future packets without tuning, opening or pausing a datasource. The
[receiver method](../experiments/wifi-observer.md) bounds block saving,
truncation, failure retention, replay and the supported decoder subset. Each new
controlled-transmitter test must freeze its own schedule, qualify the expected
address/payload observations and cover startup/recovery. Captured timestamps are
not a calibrated RF clock; bounded streaming can lose records. Keep raw captures
private and check intended control addresses independently.

Avoid `/datasource/list_interfaces.json` as a passive inventory shortcut: that
endpoint probes capture helpers. Reuse saved datasource metadata for selection.
Do not create API keys, change credentials or issue source-control requests as
part of this read-only inventory.
