# Ariel native-test qualification inputs

Frozen **2026-10-23 before test flashing**. This file records the collection
inputs; completed observations belong separately in `RESULTS.md`. The
[experiment brief](../../docs/experiments/ariel-native-tests.md) and
[build record](../../docs/builds/ariel-native-tests.md) define the unmodified
upstream tests and their limits. No native-test board pass is established here.

The decision is whether Ariel's existing native yield/flag benchmarks and
priority/lock assertion tests run through the attached C6's known UART route.
This is upstream-test qualification, separate from the common checked-message
workload and from an OS performance comparison.

## Fixed attempts and evidence

The [manifest](study-manifest.json) pins Ariel commit, compiler, tools, image
hashes and twelve scheduled attempts: three rounds, each containing
`bench-sched-yield`, `bench-sched-flags`, `threading-dynamic-prios` and
`threading-lock` in that order. No automatic retry replaces a failure.

Every attempt reflashes its selected merged image at offset zero, preserving
transfer verification. The existing espflash serial monitor runs in a PTY with
a script transcript and a **30-second host ceiling**, including connection and
reset. It is stopped by host timeout, not by a claimed firmware exit code.
Require one ROM startup, the test-specific completed-output criterion and at
least two seconds of observation after its first completed-output marker,
without a subsequent panic, benchmark error, assertion failure or reboot in
the observed window. Connection or capture failure remains a failed attempt.

These logs are rendered monitor transcripts, not a byte-exact raw UART
protocol. The unmodified programs supply no nonce, challenge, runtime clock
snapshots or individual-event records. Retain positive integer batch means
as upstream ticks; require the yield-derived switch value to equal integer
division of the iteration value by two. No cross-runtime timing attribution,
tail estimate or statistical interval is planned.

The build log, generated configuration, compiler identity, Cargo lockfile and
source snapshot are retained under [artifacts](artifacts). `INPUTS.sha256`
records their pre-collection digests and the exact ELF/image identities;
compiled artifacts are outside this checkout. App source and build
configuration remain unmodified; upstream license notices are in the snapshot.
See the [snapshot review](../../tools/benchctl/docs/api.md) for available
artifacts and current verification commands.

## Current state preserved before any test write

The historical original backup's full-device digest check failed during this
session's preflight. A fresh 65,536-byte flash prefix differed in **3,979 bytes**
in sectors **0xa000, 0xb000 and 0xc000**, all within the existing NVS partition
at 0x9000–0xefff. No difference was found in the rest of the prefix.

The current full-image candidate was reconstructed from that fresh prefix and
the historical backup's remaining bytes. A **separate full-device digest
verification of all 8,388,608 bytes passed**, establishing that this reconstructed
image matches current device contents, including the unread tail. No flash
write occurred during this investigation. The source of the NVS changes was
not established; original-application persistence is a possibility.

The current private backup is Git-ignored at
`backups/pre-native-2026-10-23/flash-backup.bin`, SHA-256
`ecc182713fa7df6b1e3841b5344f4c91528a5fef4e80a3ab37273ca4e89a661a`.
The historical `13cde854…c9faf4` backup remains unchanged. Backup and NVS
contents remain outside this dataset; sizes, offsets, hashes and transfer
verification records are retained. See [initial mismatch](artifacts/preflight-verify.log),
[prefix read](artifacts/prefix-read.log) and
[full candidate verification](artifacts/current-candidate-verify.log).

After the session, restore the **current verified full-image backup**, separately
digest-verify it against the device, then hard-reset into the original firmware.
This preserves the newest captured persistent state. No eFuse provisioning is
part of this qualification. A later boot may itself update persistent data;
restoration evidence describes the verification moment before that reset.
