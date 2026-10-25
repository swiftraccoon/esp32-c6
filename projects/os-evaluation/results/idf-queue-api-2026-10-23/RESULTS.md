# Matched IDF queue API results

Completed **2026-10-23** on the attached ESP32-C6 rev 0.2. All **24 scheduled
boots passed**, completing twelve paired blocks with no application failure,
fixture invalidation, retry or replacement. The native FreeRTOS queue path had
a smaller median round trip than the Rust std channel path in every block:
**19.9375 µs versus 62.875 µs**, at the nominal timer scale.

Both images use the **same ESP-IDF/FreeRTOS kernel and Rust std threads**.
This result supports evaluating the native queue API for this request/reply
workload before considering a new kernel. It does not select an operating
system or isolate a context-switch cost.

## Observed measurements

There are twelve boots per backend, each containing 1,000 checked request/reply
observations and 1,000 observer observations: **24,000 round trips and 24,000
observer events** altogether. The two earlier
[qualification pilots](../queue-api-qualification-2026-10-23/README.md) are
excluded. All durations are gross raw SYSTIMER ticks; no observer subtraction
or outlier removal was applied. One nominal tick is 0.0625 µs.

Each backend's twelve boots had the same following summaries. The percentile
columns are computed within each boot using nearest rank; events are not
treated as independent boot replications.

| Backend / measurement | Boot p50, ticks | Boot p50, nominal µs | Boot p95 / p99, ticks | Largest observed, ticks | Largest observed, nominal µs |
| --- | --- | --- | --- | --- | --- |
| Rust std `sync_channel` request/reply | 1,006 | 62.875 | 1,006 / 1,006 | 1,280 | 80 |
| Native FreeRTOS queue request/reply | 319 | 19.9375 | 319 / 319 | 426 | 26.625 |
| Std image observer | 15 | 0.9375 | 15 / 15 | 36 | 2.25 |
| Native image observer | 15 | 0.9375 | 15 / 15 | 16 | 1 |

The descriptive ratio of the boot medians is **1,006 / 319 ≈ 3.15**. The
native-minus-std boot-p50 difference is **−687 ticks / −42.9375 nominal µs**
in all twelve blocks. All three four-block collection-group medians and both
six-block order-group medians are also −687 ticks. These observations are
about the selected, instrumented API paths; they do not attribute the gap to
one operation inside either implementation.

The preregistered median interval uses the third- and tenth-smallest block
contrasts. Both endpoints are −687 ticks, producing **[−687, −687] ticks**.
The stated continuous IID coverage is 96.142578125%, conditional on independent,
identically distributed block contrasts. The interval collapses because the
observed, quantized boot medians are tied. It does **not** establish zero
measurement error, independent environmental replication, or a precise bound
on future results. It is retained to report the frozen analysis faithfully.

Direct raw-data checks found two
distinct ordered request/reply duration vectors and three distinct absolute
timestamp vectors per backend across the twelve boots. Every raw transcript
has its own ROM startup, and all 24 boot nonces and labels are distinct.
Fresh boots and balanced order support the recorded collection history;
their repeatability does not prove statistical independence. The three session
labels are consecutive collection groups, with no required elapsed separation.

## What was controlled and checked

The [frozen record](README.md), [manifest](study-manifest.json),
[schedule](schedule.json) and [input hashes](INPUTS.sha256) were committed as
`3fc94676ddeb240650738e479b068cc9ed807804` before the first scheduled capture.
The source revision was `93646f5e7e439fe2ea913048bbc7e9ac5a7ef2cd`. The
schedule used seed `11647397086240600225`, with two std/native and two
native/std blocks in each collection group. The
[attempt ledger](attempt-ledger.json) records all 24 attempts in that order.

The images share Rust 1.96.0, IDF 5.5.6, `esp-idf-sys =0.37.2`, CPU 160 MHz,
flash DIO 80 MHz, a 100 Hz kernel tick, priority 1 initiator / priority 5
worker, capacity-one `u32` queues, eight warmups and two seconds of settling.
Each image reserves a 32 KiB main stack, an 8 KiB worker stack and 48,000 bytes
of preallocated sample storage. Stack reservations are configuration choices,
not measured high-water marks. Radios are not initialized.

Each boot was flashed with its selected frozen image, with transfer verification
retained under [artifacts](artifacts). The collector checked backend identity,
configuration equality before/after sampling, ordered samples, timestamp widths,
checksums, worker phases, timer qualification and completed cleanup. The prior
nonce history included both excluded pilots and all preceding scheduled boots.
Capture reports retain ELF/provenance/collector hashes; those hashes describe
the supplied artifacts rather than cryptographically attesting device execution.
The flash logs provide the separate transfer evidence.

The timer passed ordered-read, blocked-sleep and explicit WFI continuity checks
in all 24 boots. Configuration snapshots retained `SYSTIMER_CONF=0xf7000002`
and the expected CPU/flash clocks. The legacy timer bracket shares SYSTIMER
UNIT0; the kernel tick uses UNIT1 and shares the crystal. These checks establish
onboard continuity and consistency within their admission bounds, without
independent oscillator calibration or proof against transient clock changes.

Queue creation, worker startup/shutdown and main-thread readiness probes are
outside each timed interval. The higher-priority worker's next-phase bookkeeping
can execute before the initiator takes its end timestamp, so that common
instrumentation contributes to the measured path. Native occupancy probes
condition execution before timing; std has no equivalent public occupancy
query. Ordinary release image placement also differs between backends.

The effective SDK configuration was initially ignored by Git's `sdkconfig`
pattern. Its exact digest was already frozen before collection. The unchanged
configuration files are now explicitly included in both this archive and the
qualification archive; this adds the previously pinned bytes to version control
without changing an experimental input.

## Reproduction and audit trail

The [machine-readable summary](summary.json) is produced by the Rust analyzer:

```sh
target/release/benchctl analyze-queue \
  --schedule results/idf-queue-api-2026-10-23/schedule.json \
  --results results/idf-queue-api-2026-10-23
```

Each directory under [runs](runs) contains untouched `serial.raw`,
`run.json` and `replay.json`. Parsed mirrors are reproducible from raw input. The archive retains link maps,
SDK files, source snapshot, build checks, capture chronology and restoration
evidence. Compiled artifacts are outside this checkout; their identities remain
in the reports. [INPUTS.sha256](INPUTS.sha256) preserves the pre-collection input
digest set; [SHA256SUMS](SHA256SUMS) records the completed collection's identities.
These historical manifests do not verify the complete current checkout. See the
[snapshot review](../../tools/benchctl/docs/api.md) for available artifacts
and current verification commands.

Independent checks recomputed every boot's min/mean/p50/p95/p99/max from raw
transcripts and all twelve paired contrasts, order/group summaries and interval
endpoints. It also checked nonce histories, frozen identities and restoration.
The host checks recorded with the frozen artifacts passed **35 unit tests and
two doctests**, formatting, Clippy with warnings denied, documentation and the
release build. Both firmware variants passed their target release builds,
Clippy, formatting and documentation checks before qualification. All 35 older
raw captures replayed without changing their archived replay reports.

## Decision and remaining experiments

This is evidence that an API choice within the existing IDF kernel materially
changes this particular workload. The next adoption question is whether an
existing Rust native-queue wrapper can meet our allocation, ownership, timeout
and cleanup requirements; see the
[source/API review](../../docs/research/idf-native-api-options.md). The measured
native image uses our narrow experimental FFI, so its numbers must not be
relabeled as measurements of an upstream wrapper.

Further selection needs equal-priority and loaded/deadline cases, independent
timing validation, memory high-water measurements, recovery behavior and the
radio/peripheral workloads the eventual board setup supports. These sample
maxima are not WCET bounds. This experiment measures neither power, radio
performance, sleep reliability, other boards, nor the complete capability of
the other candidate platforms. Historical Embassy/IDF figures use another
protocol and task arrangement and are not pooled or directly subtracted here.
The result supplies no requirement to write a custom kernel.

## Final device state

All **8,388,608 original flash bytes** were restored, independently verified
against the device, then followed by a hard reset. The
[device-state record](device-state.json), [write log](artifacts/study-restore.log)
and [separate verification log](artifacts/study-restore-verify.log) retain the
evidence. The original application's backup remains ignored locally with
SHA-256 `13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`.
No eFuse provisioning was performed. Benchmark serial bytes remain unchanged.
