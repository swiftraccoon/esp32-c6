# Continuous receiver qualification

Two separate 30-host-second windows passed explicit FINISH, process cleanup and
complete ten-file replay. They retained 18 management packets; an independent
PCAP/CRC walk and pinned TShark 4.6.9 agree that every included CRC is valid.
No C6 serial open, reset or firmware installation occurred. Target reception,
packet loss, RF timing, startup exposure and silence remain unqualified.

| Admitted window | Requested host wait | Packets | Beacons | Bad/unqualified FCS | Exact replay |
| --- | --- | --- | --- | --- | --- |
| [First](window-01.json) | 30 seconds | 10 | 0 | 0/0 | Pass |
| [Second](window-02.json) | 30 seconds | 8 | 0 | 0/0 | Pass |

The [method/source](../../experiments/wifi-window.md) was frozen at `4eb96eb`.
The remote Rust 1.90 worker and initial controller [artifacts](artifacts.json)
were frozen at `d0c3c32`. The first attempted window was rejected: the worker
finished successfully, but the host could classify normally reaped process
ownership as an I/O failure. Its nine files remain unchanged, with a failed
receipt and no successful summary; replay still rejects the original attempt.
The [failed record](failed-attempt-01.json) is retained alongside both admissions.

A deterministic counterexample and ten actual normal EOF/exit/reap controls
qualified the correction. [Revised controller inputs](artifacts-revision-2.json)
were frozen at `f807518` before both new windows. The original artifact file's
collecting state is its preserved precollection snapshot; [derivation.json](derivation.json)
records finalization. No failed attempt was repaired into success.

Admission checks one bound request, MARK1/2 and FINISH3, all four acknowledgment
prefixes, hashes, complete boundaries and packet counts, host event order and
deadline/lower-wait gates, input EOF, terminal status, exit/cleanup and complete
packet semantics. Both saved summaries replay byte for byte. All 20 admitted
files and the nine failed original files remained unchanged. Six fully rehashed
actual-bundle corruptions reject. The [independent checks](independent-checks.json)
cover all 18 packet lengths/management classifications/FCS values, without a
capture or display filter. These are independent decodings of the same saved
bytes, without another RF replication or transmitter authentication.

The monitor stayed on channel 1/2412 MHz. Guarded source/process/configuration,
ten configuration records and default-route snapshots were unchanged before
and after. The selected datasource was not retuned or controlled; each stream
used its own bounded worker process. Local process-owner checks opened no
serial device. Profiles, addresses, packets, diagnostics and credential metadata
remain private.

READY establishes custody of an HTTP 200 section header. Kismet registers the
packet handler afterward and can discard packets in its finite nonblocking
backlog without captured per-stream loss accounting. Host/worker markers include
buffering and scheduling, so 30 host seconds is not an RF observation duration.
Zero beacons establishes neither their absence nor C6 silence. No independent
RF reference was qualified, and this collection cannot admit a future C6
startup/change/Stop timeline without that method's additional gates.

`SHA256SUMS` covers every retained regular file except itself, relative to this
directory. Hashes establish retained-byte integrity, without authenticating
hardware origin. Raw records, all failed attempts and exact collector binaries
remain in the sealed ignored archive; runtime libraries are not hermetic.
