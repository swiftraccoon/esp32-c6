# Independent audit of clocked handoff results

Reviewed 2026-10-23. Dataset:
[`results/clocked-handoff-2026-10-23`](../../results/clocked-handoff-2026-10-23/README.md).
This agent did not operate the hardware, change captured evidence, or perform
Git operations. It independently reviewed all 15 completed capture directories,
replayed their complete raw transcripts, checked artifact identity, and
recomputed statistics from every observation.

All **15 captures pass this audit**, with exactly five requested warm-reset
captures per platform. They contain **960 compute observations, 960 timer
observations and 10,000 message roundtrips**, totaling 11,920 validated samples.
There is no missing benchmark, fabricated bare-metal scheduler result, ignored
checksum failure or excluded outlier. These results qualify the selected paths
under this experiment; they do not qualify every capability of an OS.

## Checks and evidence

| Check | Independent result |
| --- | --- |
| Entire `serial.raw` replay through the frozen collector | All 15 pass; each resulting hello/benchmark object equals its `run.json` object. Replay checks through EOF, including trailing protocol records. |
| Raw JSON versus `records.jsonl` | All 15 streams are identical after JSON key normalization. No capture record was omitted or rewritten. Non-JSON boot bytes remain in `serial.raw`. |
| Framing and sample counts | Exactly one hello and one completion per capture; 128 samples for bare-metal, 1,128 for scheduler platforms; actual ordered sample counts equal both declarations and completion totals. |
| Inputs/checksums | Every compute sample uses seed/revision/4096 rounds and checksum `0xa3944d55`; every message index `0..999` has `iterations:1` and actual reply `index XOR 0xa5a55a5a`; calibration has zero iterations/checksum. |
| Timer values | Every elapsed value is a nonnegative integer multiple of 1000 ns; compute is strictly positive. No elapsed value was removed or adjusted. |
| Independent statistics | All 40 per-boot benchmark summaries were recomputed from raw-preserved values and exactly match `run.json`, including sample counts, extrema, mean and p50/p95/p99. |
| Clocks | All 30 endpoint snapshots contain the same four raw registers and independently decode to CPU160 / MSPI80 / flash80 MHz, downstream divider one. Advertised numbers and before/after snapshots agree. |
| Run identity/order | 15 unique host boot labels; run-order indices exactly `1..15`; five captures per platform. |
| Supplied firmware identity | Every per-boot copied ELF matches its reported hash and the shared artifact ELF; one frozen ELF per platform. The section-report ELF hashes also agree. |
| Provenance | Every per-boot provenance object equals its platform's archived provenance object. Source archives, archived lockfiles, shared workload source, IDF sdkconfig and collector hash match their recorded identities. |
| Main results README | Message p50/p95/p99/max and compute p50/max tables agree with actual boot summaries. Merged-image byte counts agree with archived file sizes. |

Statistics were recomputed independently from arrays extracted with `jq`, using
sorted nearest rank `ceil(p * n / 100)` with ranks starting at one, and the
arithmetic mean of all observed elapsed values. No pooled percentile across
boots was calculated. The preserved [summary](../../results/clocked-handoff-2026-10-23/summary.json)
contains every per-boot statistic, ranges across the five boot statistics,
evidence hashes and explicit units.

## Actual ranges across boots

All values below are **microseconds**. Ranges describe the five within-boot
statistics; a single value means all five agree. Maxima include every observation.

| Platform / benchmark | Per-boot p50 | Per-boot p95 | Per-boot p99 | Largest observation |
| --- | ---: | ---: | ---: | ---: |
| Bare-metal compute | 308 | 309 | 318–319 | 319 |
| Embassy compute | 283 | 283 | 293–294 | 294 |
| IDF/FreeRTOS compute | 283 | 283 | 290 | 290 |
| Bare-metal timer observer | 1 | 1 | 7–8 | 8 |
| Embassy timer observer | 1 | 1 | 8 | 8 |
| IDF/FreeRTOS timer observer | 1 | 1 | 2 | 2 |
| Embassy message roundtrip | 7 | 8 | 8 | 20 |
| IDF/FreeRTOS message roundtrip | 62 | 63 | 63 | 69 |

The observed message maxima of 20 and 69 microseconds remain present even
though their within-boot p99 values are 8 and 63 microseconds. Sixty-four-sample
compute/calibration p99 equals that boot's maximum. The message sample count is
1,000 **within each boot**; it is not 1,000 independent boots or a demonstrated
worst-case scheduling bound. Fractional arithmetic means do not improve the
timer's 1 microsecond resolution. No timer-overhead subtraction occurs.

The 308 versus 283 microsecond compute median difference remains in these
captures. Matching source and workload bytes do not by themselves control code
placement. The separate [binary review](clocked-workload-binaries.md) and
[alignment probe](../../results/alignment-probe-2026-10-23/README.md) must be read
before attributing that difference to runtime overhead. That probe's additional
boots are excluded from this dataset and its five-boots-per-platform counts.

## Readback and artifact identity

Every initial and final raw register object is:

```text
pcr_sysclk_conf   0x28010200
pcr_cpu_freq_conf 0x00000000
pcr_mspi_clk_conf 0x00000500
spi0_clock       0x80000000
```

The independently reviewed [implementation/register evidence](clocked-handoff-implementation.md)
establishes the decode: SPLL480 selected; system divide three; CPU divide one
without force-120; MSPI divide six; SPI0 divider bypass. This controls the
configuration ambiguity from the initial pilot. It does not electrically
calibrate the clock or exclude transient changes between equal snapshots.

| Artifact | Verified SHA-256 |
| --- | --- |
| Bare-metal ELF | `be14bac3563b52d5e043485c9f798a4e23d6ed8702b72c9a3028c1cf46f58d0b` |
| Embassy ELF | `8d06354ca14e9a51550fd3183c9f29750f32fc73880140c6d9ca8151e0a18d44` |
| IDF ELF | `2c29bbbc2c7fe9ebb433907d9a2121c121ad67c460d4f1ead99f50e552991cf9` |
| Collector executable | `6f1a1a38c5079492f957a12ea76193f6b3ce7a8fe5d9dbbbb81c1331b327ac55` |
| HAL source archive | `5d46a17fae89ca31c79802c55aa2610fd2affeee24af35f28c59ba9a8e3504c6` |
| IDF source archive | `aa860679084bd723623c283f119b778ee7b99c37fc7a8d4c69ff2bfbd84b9958` |
| HAL archived Cargo.lock | `fe83fea7296603dd9a461ed872fcecab02b277754a18060b0ac0e8cbc48ed89f` |
| IDF archived Cargo.lock | `72d9f6a550922507b682a7ebc40623da22e62cdf12c138249a09de40ef5adace` |
| Both archived workload sources | `27447ee945989a682ee778646f488e7c986d8831a38515e1057eaa143992ab9e` |
| Archived generated IDF sdkconfig | `5ee0be1128539831db16b63d7008985e16849b216bd4a278076e294a51fe3758` |

Both source archives contain the final reviewed adapter files; their critical
source hashes match the independent implementation review, including Embassy's
corrected indefinitely parked responder. All provenance records consistently
identify source commit `8f4294b349a6e493b4dfe0d1c01b08afe4c81626` and Rust
1.96.0 / LLVM22.1.2. Per-boot raw/parsed/report hashes are in `summary.json`.

## Independence and practical limits

The collector actively requested a reset for every run, using RTS with DTR
deasserted. Boot IDs and order are host labels. The serial protocol contains no
device-generated nonce, so it cannot independently authenticate a new boot.
Several complete transcripts are byte-identical at this deterministic input and
coarse timer resolution:

- Bare-metal runs 06 and 10.
- Embassy runs 05, 07 and 12.
- IDF/FreeRTOS runs 03, 09 and 14.

These duplicates are preserved and explicitly recorded rather than deduplicated
or counted as a validation failure. The independent-reset interpretation relies
on the operator's capture/reset evidence, not on distinct timing values. This
was one physical board with requested warm resets, not cold power cycles or a
multi-board population.

Embassy measures cooperative tasks on one executor/OS thread; IDF measures its
Rust bounded-channel path across actual FreeRTOS tasks with observed priorities
one and five. Different queue/wakeup implementations, wrappers and priorities
are part of the measured paths. The comparison does not isolate a kernel
context switch, and a roundtrip cannot be divided by two to infer one-way latency.

Radios, IRQ response, peripheral coverage, power/sleep, security, stack/heap
peaks, cold boot and long-duration stability remain unmeasured. Readback and
artifact hashes cannot authenticate a serial peer or prove that a supplied ELF
is the loaded firmware; preserved flash-transfer evidence supplies that separate
operational link. The protocol transcripts audited here remain unchanged.
Original-firmware restoration is a separate operator action recorded in the
dataset's device-state evidence.

## Commands used

The frozen collector was run once per directory:

```sh
target/release/benchctl validate --input <run>/serial.raw --platform <platform>
```

Raw-versus-parsed comparisons used `rg '^\{'`, `jq -Sc .` and `diff -u`.
Per-boot records/provenance were extracted read-only with `jq`; independent
arithmetic checked every index/checksum/count/clock decode and recomputed every
statistic. `shasum -a 256` verified ELF/collector/archives/reports/transcripts;
`tar -xOf` plus SHA-256 checked source and lockfile bytes inside archives.
All checks completed successfully. The only files authored by this results
audit are this review and the aggregate `summary.json`.
