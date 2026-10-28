# Existing Wi-Fi receiver qualification

Policy revision 2 retains the 32-EPB quota and extends the capture deadline from
15 to 120 seconds. The original 15-second stream returned eight complete EPBs
with HTTP 200 but failed its quota; it remains failed and is not reused as a
successful sample. A separate earlier local profile rejection is also retained.
Freeze the revised policy/artifacts before new streams; this change does not
establish why the source delivered that prefix or qualify RF reception loss.

Decision question: can a bounded read-only stream from the existing remote
Kismet Wi-Fi source be saved and replayed with consistent PCAPNG framing and
supported IEEE 802.11 beacon decoding? This is receiver-path qualification on
ambient traffic. It does not transmit from, flash or reset the ESP32-C6.

The [observer inventory](../docs/observers.md) established an existing Realtek
monitor source and advancing packet counters. Counters do not establish packet
contents. The [prior-work catalog](../../../docs/research/prior-work.md) and
[reuse catalog](../../../docs/research/benchmark-reuse.md) favor independent
packet observation for later radio experiments. Wireless explorer's SDK callback
collector supplies useful complete-receipt/failure-retention practice, but has
no PCAPNG/radiotap decoder; its address-two shortcut is not reused. Kismet's
existing source stream supplies reception. New Rust code adds bounded capture,
strict saved-record validation and privacy-safe summaries rather than a second
radio scanner. No published performance result is compared.

## Inputs and observer

Pin Kismet commit `a868177ef92dc8ea0b6fb121976eeddba1854d43`
(version `2025.09.0-a868177ef`), the actual source selection and running
configuration privately. The source-scoped endpoint is
`GET /datasource/pcap/by-uuid/{UUID}/packets.pcapng` on the remote host's loopback
HTTP service. It creates a temporary stream reader; disconnecting removes that
reader. It does not tune, pause or reopen the datasource. Do not call interface
probing, source-control, credential-management or key-creation endpoints.
Preserve the host's active Kismet receiver and default network route.

The local host workspace pins Rust 1.96.0, edition/style 2024, its dependency
lock and the `identityctl`/`wifi-observer` source. The independent std-only
remote worker uses the already installed Rust 1.90.0, edition 2024, target
`x86_64-unknown-linux-gnu`. Record the actual compiler, worker binary, shared
framer source, curl and SSH identities before collection. Installing or
upgrading a toolchain is not part of this method. Pure worker controls run on
both its actual remote compiler and the host compiler.

A strict private profile selects a trusted SSH alias, remote worker, source UUID,
previously resolved credential-file paths, process/configuration guards and
worker provenance. Check read-only configuration metadata before and after the
cohort. The worker guards its expected Kismet PID and credential-file size/mtime
before access; these guards do not inspect the daemon's complete in-memory
configuration. A changed source/configuration stops collection pending
investigation. Credentials
are read only inside the remote worker and supplied to curl through stdin.
Never place credentials in argv, URLs, local records or diagnostics. HTTP headers
and curl diagnostics remain inside the worker; only bounded numeric/fixed
transport metadata leaves that process. Raw radio packets, source metadata and
SSH diagnostics stay under ignored `private/device-identity/`.

## Bounded capture

Capture a prefix ending at the thirty-second complete Enhanced Packet Block
(EPB). The sample is **32 packet records**, not a 120-second RF window. The worker
has a 120-second absolute deadline and an 8 MiB raw-byte cap; failure to obtain
that prefix remains failed. A stream can end at any complete block boundary
without a special PCAPNG end marker. A network read may prefetch bytes after the
quota boundary; record their observed byte count while leaving that suffix
uninspected and outside the declared sample. This intentional prefix stop does
not trim a failure before the quota into success. Retain received bytes within
the storage cap, including a partial
block on failure. Over-cap input keeps the bounded prefix and an explicit limit
failure; excess bytes are not an admitted observation. A timeout, premature EOF,
excessive size, malformed framing or unsuccessful
process cleanup does not become a smaller accepted sample.

Require HTTP 200 independently of the intentional stopping rule. Curl handles
HTTP transfer encoding. At the successful block boundary, stop and reap curl,
recording the planned stop separately from its actual exit/signal. Reap the SSH
transport as well. OS process/network I/O can exceed requested deadlines; no
hard real-time claim follows. Preserve exit codes, stop reasons, byte counts,
errors and cleanup outcomes. A public summary is emitted only after the whole
capture validates.

Create the ignored destination and raw output before launching the worker. A
completed directory has exactly seven regular files: `request.bin`,
`packets.pcapng`, `transport.stderr`, `transport-status.json`,
`worker-provenance.json`, `summary.json` and `capture-status.json`. The receipt
binds exact saved contents. Replay checks inventory, supported versions,
transport completion, request/provenance agreement, byte hashes, framing and
saved-summary byte equality. Standalone `wifi-summary` checks a saved PCAPNG
only; it cannot admit a failed collection or authenticate where packets came
from. Keep all failed directories unchanged.

## Packet interpretation

A section's byte order, version, lengths, repeated block trailers and options
must agree. Map each EPB to its declared Interface Description Block (IDB),
including actual link type, snap length and timestamp resolution. Reject
undeclared interfaces, unsupported link types, captured/original/snap-length
inconsistencies, malformed option/padding bounds and truncated blocks. Actual
IDBs govern decoding: the JSON datasource's DLT field is not a substitute.
Kismet can emit multiple IDBs or JSON-only packets; unsupported contents remain
an explicit failure instead of being assumed to be radiotap.

Radiotap lengths, presence maps, field alignment and supported flags govern the
802.11 offset and any FCS handling. Validate supported beacons' management
header, fixed fields and information-element bounds. Do not infer source
addresses for arbitrary data/control frames from address two. Summaries contain
only fixed states and counts; no MAC, SSID, source name, UUID, information element,
packet timestamp, payload or hash of a private identifier is published.
The implementation/API contract records the exact supported subset and error
policy before the source freeze.

Only a nonzero supported-beacon count exercises physical beacon decoding in a
sample. Zero leaves that coverage unexercised; it does not trigger recollection
or imply RF silence. Structural transport admission is distinct from beacon
coverage and full IEEE conformance or packet authenticity.

Kismet timestamps are source-provided, not a calibrated clock or established
monotonic sequence. Do not derive RF timing, delivery probability or capture
fidelity from them. Its bounded nonblocking streaming backlog can discard data;
a well-formed stream does not establish lossless reception. IDB mapping and a
source-scoped URL support internal consistency, not authentication of a remote
radio or isolation against a malicious server.

## Qualification and collection schedule

Before opening a real packet stream, qualify the actual parser, remote worker,
save/replay CLI and receipt path using synthetic positives and deliberately
failing controls. Include fragmented block/HTTP writes, exact boundary stops,
truncation at headers/bodies/trailers, bad byte order/version/lengths/options,
unknown interfaces/link types, invalid radiotap alignment/FCS/headers, malformed
beacon information elements, HTTP denial/redirect/HTML, worker deadline/EOF/size
failures, cleanup failures, corrupted saved content and private sentinels.
Have a separate reviewer construct contrary controls and inspect process,
credential and source-selection boundaries. Freeze method, all executable
source/configuration identities and artifacts in a commit before collection.

Plan two independent 32-EPB prefixes from the same selected source. Capture each
in a fresh ignored directory; abort the cohort on its first failed attempt and
investigate before any repeat or method change. Traffic contents/counts by
frame class can differ; no equality or performance comparison is planned.
Record read-only source/interface/configuration metadata before and after the
cohort to bound the observer state. Do not retune, open the Nordic serial port,
stop a service or create a Kubernetes workload.

Replay each complete directory unchanged and compare supported beacon/frame
counts with the installed Wireshark decoder using the same saved bytes. Keep
its exact version, commands, decoded values and diagnostics private. Differences
must be reconciled against source before publishing a pass; do not select a
favorable subset. Add adversarial copies with recomputed hashes so semantic
checks, rather than checksums alone, must reject corruptions. Public results
retain all cohorts, source freezes, scopes and a complete directory seal.

A successful result establishes bounded stream/save/replay and the declared
packet-decoding scope on this source. Reception of a known ESP32 transmitter,
MAC override correctness, startup traffic, silence, rotation, cold boot,
reconnect, correlation reduction and Nordic readiness remain unmeasured. Those
require a separate controlled-transmitter method and restoration boundary.

## Primary sources

- [Matching Kismet source endpoint](https://github.com/kismetwireless/kismet/blob/a868177ef92dc8ea0b6fb121976eeddba1854d43/datasourcetracker.cc).
- [Matching Kismet PCAPNG stream writer](https://github.com/kismetwireless/kismet/blob/a868177ef92dc8ea0b6fb121976eeddba1854d43/pcapng_stream_futurebuf.h).
- [Kismet packet capture API](https://www.kismetwireless.net/docs/api/packet_capture/).
- [Pinned PCAPNG work-in-progress draft 05](https://www.ietf.org/archive/id/draft-ietf-opsawg-pcapng-05.html).
- [Radiotap field definitions](https://www.radiotap.org/fields/defined).
