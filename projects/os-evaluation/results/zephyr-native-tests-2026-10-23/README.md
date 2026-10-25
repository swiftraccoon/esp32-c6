# Zephyr native latency-suite qualification inputs

Frozen **2026-10-23 before Zephyr test flashing**. This is the input record;
completed observations belong separately in `RESULTS.md`. The
[experiment contract](../../docs/experiments/zephyr-native-tests.md) and
[build record](../../docs/builds/zephyr-native-tests.md) describe the unmodified
Zephyr 4.4.0 default native suite. No board pass is established by this file.

The question is whether the maintained C6 port can boot and complete that suite
through UART0 GPIO16/17 at 115200 baud. It qualifies a native integration path;
Rust application support and comparable application performance are separate
questions. No C or assembly adaptation is authored.

## Fixed inputs and attempts

[study-manifest.json](study-manifest.json) schedules **three boots** of the same
image, with no automatic retry or replacement. Every attempt writes the frozen
Simple Boot `zephyr.bin` directly at offset zero and retains flash verification.
Do not wrap this image using an ESP-IDF bootloader or image converter.

Each capture runs the existing espflash serial monitor in a PTY/script for a
full **120-second host window**, including connection/reset, followed by host
SIGINT timeout termination. Preserve both the script's named file and its
rendered stdout mirror. The mirror remains usable when the auxiliary script
file is empty after termination. This is rendered monitor output, not an exact
raw-UART protocol or a device exit-status report.

Admission requires one ROM startup, the 16 MHz timing header, all **47 named
metric rows** exactly once in the expected order with numeric columns and empty
notes, one subsequent `PROJECT EXECUTION SUCCESSFUL`, no declared error/panic/
failure or reboot, and at least **two seconds after first observed success**.
Late or incomplete output fails. Preserve all columns and notes even on failure.
The [expected names](artifacts/expected-metrics.txt) and contract specify the
row grammar and rejection cases.

“Cycles” are nominal **16 MHz SYSTIMER UNIT1 ticks**, not CPU cycles. Most
results subtract an unprinted observer estimate; heap timing is an exception.
The two averaged integer columns can round differently. No gross endpoints,
calibration value or per-event distribution can be recovered from the printed
means. Kernel assertions are effectively disabled, and the upstream success
counter does not exhaustively check every operation. No comparison against
gross IDF/Ariel results, tail estimate or overall OS ranking is planned.

The artifact directory retains configuration/devicetree/map, source/license
evidence, resolved tools/modules, build log, output-name list and image details.
`INPUTS.sha256` preserves the pre-collection digest set, including the exact
ELF/bin identities; compiled artifacts are outside this checkout. Source copies
are selected evidence snapshots; pinned revisions identify the rest. See the
[snapshot review](../../tools/benchctl/docs/api.md) for available artifacts
and current verification commands.

## Verified current state and restoration

Preflight after Ariel's verified restoration/reset found a new digest change.
A fresh 65,536-byte prefix differed from that session's preserved image by
**1,993 bytes**, only in NVS sectors **0xb000 and 0xc000**. The current full
candidate combines that fresh prefix with the previous verified image's tail.
A separate **8,388,608-byte full-device digest verification passed** before
any Zephyr write. The cause of these persistent changes is not established.

The new private backup is ignored at
`backups/pre-zephyr-2026-10-23/flash-backup.bin`, SHA-256
`863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
The historical and pre-Ariel backups remain preserved outside this dataset.
The preflight, read and full-verification logs are retained under
[artifacts](artifacts); flash and NVS contents are not included.

Restore this **current full image** after collection, separately verify its
device digest, then hard-reset into the original application. No eFuse change
is part of qualification. A restored application's subsequent boot may update
persistent data again; the digest evidence describes the pre-reset state.
