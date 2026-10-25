# Zephyr native-suite qualification: three failed attempts

Collected **2026-10-23**. **Zero of three scheduled boots qualify under the
frozen v1 contract.** Each console contains all 47 expected numeric metric
rows with empty notes and an upstream `PROJECT EXECUTION SUCCESSFUL` banner,
but none contains the required frequency header. Every boot also reports a
ROM SHA-256 comparison failure before attempting to boot anyway.

These are retained failed qualification attempts, not admitted performance
results. No attempt was repeated, replaced or turned into a pass after changing
the rules. The original [input record](README.md), [manifest](study-manifest.json)
and [INPUTS.sha256](INPUTS.sha256) remain unchanged at freeze commit
`f156320`. The [source audit](../../docs/research/zephyr-native-audit.md)
documents errors in the preflight review and timing descriptions.

## What the captured output establishes

| Attempt | Expected rows in order | Empty notes | Upstream success banner | Required frequency header | ROM SHA failure | v1 admission |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 47 | Yes | Yes | Absent | Present | Failed |
| 2 | 47 | Yes | Yes | Absent | Present | Failed |
| 3 | 47 | Yes | Yes | Absent | Present | Failed |

Every capture retains one C6 ROM startup and the selected suite's named output.
Transfers report their data hash verified. The upstream terminal banner is
limited completion evidence; it does not validate every operation, prove Rust
integration or override a failed local admission criterion. No overall latency,
utilization, allocator or OS score is derived from these observations.

The three fixed 120-second host windows used the same Simple Boot image and
UART route. Recorded post-marker lower bounds are 113,736 / 113,750 / 113,745 ms,
conditional on the saved host request/first-seen timestamps and timeout status.
They are not independently calibrated device timing traces. No device nonce
or cryptographic execution attestation is supplied by the unmodified suite.

## The missing header is a preflight defect

In the selected source, the frequency header uses `TC_PRINT`. Without
`CONFIG_ZTEST_VERBOSE_OUTPUT`, that macro expands to `print_nothing`, while
metric rows use another print path. The resolved configuration disables that
verbose setting, the header literal is absent from the ELF, and all consoles
lack it. The written v1 rule was therefore impossible for this fixed image.

The configured timer frequency and linked SYSTIMER UNIT1 path remain source/
build evidence for nominal 16 MHz ticks. The missing print does not establish
an incorrect timer frequency. It establishes that the preflight review failed
to reconcile an acceptance rule with the actual output configuration.

No corrected profile has been collected. A future v2 would need its own
explicit configuration, image/contract freeze and prospective attempts; it
cannot retroactively admit these captures.

## Original inspection and corrected replay

The initial host inspection applied the row expression across the entire text.
Its whitespace matching consumed adjacent lines, producing a false count of
24 rows and false nonempty-note findings. Applying the same grammar separately
to each line reproduces all 47 rows in the expected order, with numeric columns
and empty notes. This fixes an inspection defect without changing the output
or the required rows.

The original `run.json`, `metrics.json` and attempt ledger are retained unchanged.
Each run adds `replay.json` and `replayed-metrics.json` with analysis revision
**2**. The [summary](summary.json) separates these corrected counts from the
unchanged failed admission. The initial missing-header and ROM-failure findings
remain valid. `console.log` retains rendered stdout; the auxiliary `monitor.log`
is also preserved. Neither is represented as a raw UART measurement protocol.

## Timing interpretation correction

The preflight description that most rows subtract observer overhead was wrong.
The default suite has **19 observer-adjusted and 28 unadjusted rows**. Of those
28, eight use whole-loop batch spans and twenty accumulate individually timed
spans. The [47-row endpoint/correction map](../../docs/research/zephyr-native-audit.md)
must accompany any future interpretation.

Printed “cycles” are nominal SYSTIMER ticks, not CPU cycles. Printed average
ticks and nanoseconds are rounded separately. The observer calibration includes
loop/outer-read costs and does not exactly match all operation intervals;
over-subtraction is possible, but its actual bias is not quantified here.
The unprinted calibration and gross totals cannot be recovered from these means.
The original `metrics.json` blanket correction note is superseded by this audit.

## Image-integrity warning

All three ROM transcripts include `SHA-256 comparison failed:` and
`Attempting to boot anyway...`. Transfer verification and this boot-time image
check are different mechanisms. Independent analysis reproduces the ROM's
calculated hash and expected bytes from the frozen binary's RAM-only header
layout. The converter deliberately omits an appended digest for that layout;
the warning is consistent with the archived image, rather than evidence of
random serial corruption. This explanation does not waive the failed contract
or establish a complete boot-integrity qualification.

The requested flash frequency is 80 MHz, but the converter aliases the 40/80 MHz
header encodings on C6. The image-info label cannot independently distinguish
them. ROM prints divider two; Simple Boot later prints 80 MHz. No runtime
register readback or electrical frequency calibration is provided by this suite.

## Restoration and remaining work

The pre-session current state was preserved after another NVS-only change,
then independently verified against the full device digest before test writes.
After all three attempts, all **8,388,608 bytes** of that current backup were
restored, separately verified and followed by an RTS hard reset. See
[device state](device-state.json), [write log](artifacts/zephyr-restore.log) and
[verification](artifacts/zephyr-restore-verify.log). The private backup SHA-256
is `863dc7b17018aea2a530ee27da5bedd675298f9a70e5f04c32f4a0cc3c43f9c4`.
No eFuse provisioning occurred. Later application persistence can change flash
again; this verification describes the state before reset.

The [per-metric source audit](../../docs/research/zephyr-native-audit.md),
complete failed attempts and collection-time [archive hashes](SHA256SUMS)
record this checkpoint. Resolve the output/image preflight defects and per-row semantics
before another native comparison. The separate
[Rust build probe](../zephyr-rust-build-2026-10-23/README.md) failed its build
gate and produced no Rust image or board result. Matched workloads, utilization,
loaded deadlines, memory peaks, radios, sleep/energy and recovery remain open.
