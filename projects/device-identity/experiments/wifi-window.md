# Continuous Wi-Fi receiver transport

Decision: can the existing receiver retain a complete bounded stream through
explicit host markers and finalization, with independently replayable custody?
The closest prior method retains a 32-EPB prefix; reuse its strict profile,
credentials/process guards, PCAPNG framer and packet decoder. This separate
worker/protocol removes that quota without changing legacy collection or its
results. Initial qualification controls receiver transport only, with no C6
serial open, reset or firmware installation.

The host uses Rust 1.96.0; the independent std-only worker uses the receiver's
pinned Rust 1.90.0. Freeze source/config/tool/binary identities, this schedule,
the private source selection and actual passing/failing save/replay controls
before live collection. Independently review malformed/late suffixes, delayed
final flush/hash, control order/token/quotas, process cleanup, true EOF and
rehashed archive inconsistencies. Preserve every failed attempt and its original
source. A successful test/mock is not qualification of the remote receiver.

Collect two independent windows of 30 host seconds each, retaining every packet
delivered by the selected source between HTTP setup and explicit completion.
Create private output before provenance/network operations. Verify the separate
worker's executable/build identities. Send one length-framed request with a
fresh nonzero token; consume bound READY only after a saved HTTP 200 section
header and a live process. Send MARK1, wait at least 30 seconds on the host
monotonic clock, send MARK2, then FINISH3. After validating the FINISH prefix,
close input and require the successful terminal status and child cleanup.

Bounds are 120 worker seconds, 8 MiB raw data, 64 KiB diagnostic transport,
30 seconds to host READY, 10 seconds per control acknowledgment and 132 seconds
for the host transport. The worker waits at most 5 seconds for its initial
request and for EOF after FINISH. Exact deadline arrival is late. Blocking OS
I/O can exceed a timer before failure is observed; this is not hard real time.
Requests/controls are recorded before attempted writes; failed partial writes
do not establish that all requested bytes reached the peer.

MARK can fall inside a PCAPNG block. Retain its whole saved prefix and the last
complete boundary separately; do not trim a later malformed/truncated suffix to
manufacture completion. FINISH waits for a complete boundary, stops/reaps/drains
curl, checks final flush/hash/deadline, acknowledges the exact final raw bytes,
then requires input EOF without further control bytes. EOF before FINISH,
unexpected/duplicate controls, wrong tokens, deadlines, HTTP errors, overflow,
malformed bytes or failed cleanup reject the whole attempt. The worker owns a
bounded input-reader subprocess so incomplete input can be cancelled/reaped.

The exact archive has ten files: `window.json`, `request.bin`, `controls.bin`,
`host-events.jsonl`, `packets.pcapng`, `transport.stderr`,
`transport-status.json`, `worker-provenance.json`, `summary.json`, and
`capture-status.json`. Receipt hashes bind the other nine files. Replay checks
the exact inventory, request/controls, all four prefix/boundary/provenance
acknowledgments, event order/deadlines/host wait, explicit finalization and
cleanup, every packet through the complete semantic decoder, and exact saved
summary equality. A failed/partial attempt remains rejected.

Snapshot the receiver's datasource/process/config/channel/default route before
and after each window using the existing read-only guard method. Do not retune,
stop Kismet or replace its source. Publish only fixed counters/booleans and
sanitized artifact provenance; packets, identities, paths and credentials remain
private. Complete the private seal over all retained files before finalization.

Kismet writes its stream section header before registering its packet handler.
READY therefore qualifies transport prefix custody only. The endpoint has a
nonblocking finite backlog with no captured per-stream drop accounting. Host
and worker markers include scheduling/network/buffering effects and cannot
bound RF timestamps or prove absence. Independent reference reception stays
false in this report. A future before-reset/startup/Stop experiment needs its
own frozen method and reference gate before any board reset/open; this standalone
transport method does not qualify that experiment.
