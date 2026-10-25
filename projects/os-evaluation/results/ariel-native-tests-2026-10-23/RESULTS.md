# Ariel native-test qualification results

Completed **2026-10-23** on the attached ESP32-C6 rev 0.2. All **12 scheduled
boots reached their expected UART output**: three each of the existing yield,
flag, dynamic-priority and lock-ordering tests. The original operator-reported
passes remain preserved. A later evidence audit found that the first boot's
post-marker guard is operator-attested rather than independently reconstructable
from retained artifacts. No attempt was repeated or replaced.

Interpretation corrected **2026-10-23**, after the initial report at commit
`12d02a55e4672cb8ee869f534658f6e1050daa8c`. Capture bytes, original run reports,
the schedule and frozen inputs are unchanged. Artifact-visible and operator-reported evidence are distinguished below.

The [frozen input record](README.md), [manifest](study-manifest.json),
[experiment brief](../../docs/experiments/ariel-native-tests.md) and
[build record](../../docs/builds/ariel-native-tests.md) identify the exact tests
and admission rules. The input checkpoint was
`e17c06760277200115e5038b6794d2ce155b0260`, before test flashing.

## Observations

| Upstream test | Scheduled boots | Observed output in each boot | Meaning |
| --- | --- | --- | --- |
| `bench-sched-yield` | 1, 5, 9 | `took 155 ticks per iteration (77 per context switch)` | Floored mean of a batch of 10,000 `yield_same` calls; 77 is integer division of the already-floored 155 by two. |
| `bench-sched-flags` | 2, 6, 10 | `[INFO] took 215 ticks per iteration` | Floored mean of a batch of 1,000 `set` / `wait_any` iterations after thread-ID exchange. |
| `threading-dynamic-prios` | 3, 7, 11 | `[INFO] Test passed!` | Reached the success marker after upstream priority/execution-order assertions. |
| `threading-lock` | 4, 8, 12 | `[INFO] Test passed!` | Reached the success marker after upstream execution/lock-order assertions. |

Only **three aggregate timing outputs per benchmark** are retained. Individual
operation durations and raw batch totals are not printed by these programs.
Identical means across the three boots do not establish identical event
distributions, statistical independence or an uncertainty bound. No percentile,
confidence interval or worst-case estimate is derived.

The two timing tests use default-priority **1** stackful threads. The priority
control starts at **2 / 1**, raises one thread to **5**, then restores it to
**1**. The lock control uses **1 / 2 / 3 / 2** and checks the expected acquisition
order. Higher numeric Ariel priority wins; equal-priority threads cooperate.
These tests check their particular paths, not every scheduler or lock behavior.

## Capture evidence and host interruption

The [attempt ledger](attempt-ledger.json) preserves three rounds in the frozen
order. Every attempt reflashed its selected image with transfer verification,
then used the same **30-second host monitor ceiling**. Each retained console
has one fresh ROM startup and one expected completed-output line, without the
rejected benchmark-error, panic, assertion-failure or subsequent-startup markers.

Each run under [runs](runs) retains `flash.log`, `console.log`, `monitor.log`
and `run.json`. **`console.log` is the unchanged rendered stdout mirror and the
validation input.** These are monitor transcripts, not a byte-exact UART
measurement protocol. The macOS `script` named output file was empty after
SIGINT in eleven attempts; run 11 retained a 1,641-byte named file identical to
its console. Both destinations remain archived in every attempt.

For the first attempt, the complete stdout mirror was already retained in an
ignored host log and was copied byte-for-byte into its public `console.log`.
Remaining attempts wrote that mirror directly to their console file. This
preserves received output rather than replacing a capture or filtering bytes.
No rerun was used to recover an empty auxiliary file.

A host service interruption occurred **after attempt 1 had completed**. The
saved capture remained intact, and collection resumed at attempt 2. The
contract fixed order/counts without requiring uninterrupted collection or a
minimum elapsed spacing. The interruption supplies no independent timing
replication or additional device observation.

The first attempt records an **operator-reported 5,000 ms** post-marker lower
bound, based on a later five-second monitor wait still reporting the process
running. The actual wait result was not retained in this archive. Its guard
therefore cannot be independently reconstructed from these files; the saved
console does independently establish the expected output and absence of error
text within its retained bytes. This does not establish that the guard failed.
Attempts 2–12 record conservative lower bounds of
**23,708–23,760 ms**, derived from the saved request/first-seen timestamps and
the recorded full timeout window. Their arithmetic reproduces conditional on
those host records; it is not an independent physical timing trace. The
recorded bounds exceed the prescribed two seconds. Exit **124**
records the host timeout sending SIGINT; it is not a firmware exit code.
Quiet output after a marker does not prove continued useful progress or freedom
from a later hang. The source leaves benchmark peer threads in loops; the assertion tests'
`exit(Success)` falls back to a spin loop in this configuration.

## Timing and comparison boundary

The images use unmodified Ariel commit
`13bddaca1c2bcd51489c044b80408e917d5946d9`, **Rust 1.99.1**, laze **0.1.39**,
fat LTO, one codegen unit, forced frame pointers and ordinary release placement.
The source snapshot, lockfile, generated configuration, exact ELF/image pairs
and build log are under [artifacts](artifacts). No authored firmware adapter
or C/assembly implementation was introduced.

The ESP benchmark helper brackets an entire batch with SYSTIMER **UNIT0** reads,
then computes `total as usize / iterations`. Printed ticks are nominal **16 MHz**
counter ticks, not CPU cycles or whole microseconds. The CPU is requested at
160 MHz by the selected HAL configuration. These programs emit no runtime
clock-register snapshots, independent oscillator calibration, observer result,
sequence checksum or individual timestamps. Bootloader header messages do not
establish final runtime flash frequency.

On C6 the cast to 32-bit `usize` occurs before division; deltas at or above
`2^32` ticks would truncate. At the nominal rate that span is about 268.435 s,
beyond the fixed 30-second window of these admitted short batches. This is not
a general timer monotonicity or sleep qualification. The flag test can begin
with a pending startup flag, supplies no sequence-number payload and does not
prove that every iteration has an identically blocked waiter.

The yield test has no peer-progress counter or per-iteration acknowledgment.
Its marker proves that the initiator completed its batch of `yield_same`
calls; it does not prove a peer ran on every call or establish the number of
actual context switches. The printed 77-tick value is upstream arithmetic,
not a validated single-switch latency. The intended peer path in the source
is not a measurement of peer execution.

The values characterize existing Ariel batch tests. They are not matched
request/reply measurements and are not pooled or ranked against IDF, Embassy
or Zephyr results. Compiler/HAL versions, execution model, endpoints,
instrumentation and build placement differ. This qualification supplies no
WCET, radio/peripheral, power, memory high-water, sustained-load, recovery or
product-readiness result. The earlier ROM-copy and clock/sleep uncertainties
remain separate work.

## Preserved state and final restoration

Preflight found that the historical original-image digest no longer matched
the device after its previous restoration/reset. A fresh **65,536-byte** prefix
differed by **3,979 bytes** confined to NVS sectors **0xa000, 0xb000 and 0xc000**.
The cause was not established; firmware bytes outside those sectors matched.
No flash write was performed during that investigation.

A current full image was reconstructed from the fresh prefix and the historical
backup's unchanged tail, then **independently verified against the complete
8,388,608-byte device digest before test writes**. This verifies the reconstructed
image without claiming that all its bytes were freshly read. Its private
Git-ignored backup remains at `backups/pre-native-2026-10-23/flash-backup.bin`,
SHA-256 `ecc182713fa7df6b1e3841b5344f4c91528a5fef4e80a3ab37273ca4e89a661a`.
The historical backup remains unchanged and separately preserved. Neither
backup nor NVS contents are committed.

After all twelve attempts, root restored this **current full-image baseline**,
separately digest-verified it, and hard-reset into the original firmware at
**06:31:18 UTC**. See the [write log](artifacts/ariel-restore.log),
[separate verification](artifacts/ariel-restore-verify.log) and
[device-state record](device-state.json). Restoring the current baseline
preserves the newest captured NVS state. The digest evidence describes the
verification moment before reset; the application may subsequently update
persistent data. No eFuse provisioning was performed.

## Next use

Source/configuration checks, console/report agreement, observation windows and
restoration delimit the claims above. The pre-collection
[INPUTS.sha256](INPUTS.sha256) and collection-time [SHA256SUMS](SHA256SUMS)
retain their original declarations; the [publication inventory](../../../../quality/publication.json)
records currently available files.

These native paths now provide working integration references. A common Rust
adapter can next add a specified payload, explicit state checks and raw
per-event timing under a newly reviewed contract. Existing aggregate test
outputs should retain their original names and interpretation.
