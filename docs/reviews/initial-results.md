# Independent review of the initial on-board results

Reviewed 2026-10-22 after all 15 captures completed, using the preserved
[run evidence](../../results/initial-2026-10-22/runs/) and
[artifacts](../../results/initial-2026-10-22/artifacts/). This review performed
read-only file analysis; it did not access the board, serial port, or flash.
Source/build comparability and machine-code evidence are documented in
[the cross-platform review](cross-platform.md).

All **15 boots passed** the audited compute/calibration protocol: five boots
per platform, each with 64 compute samples and 64 timer-overhead samples.
There are **960 compute and 960 calibration observations** overall. These are
measurements on one device under warm resets, with 15 distinct boot records;
they are not 1920 independent hardware trials or evidence across boards.

**Methodology erratum:** the intended equal 40 MHz runtime flash clock was
not verified. Every HAL boot reports 80 MHz, while its image headers nominally
request 40 MHz and captured provenance records 40. HAL initialization also
selects an 80 MHz MSPI source; downstream divisors were not observed during
measurement. ESP-IDF's SDK is configured for 40 MHz. These protocol passes do
not establish an equal-flash-clock experiment. The
[source-backed clock erratum](cross-platform.md#runtime-flash-clock-erratum)
supersedes earlier clock-control inferences. Captured JSON and raw logs are
retained unchanged; no runtime frequency is fabricated to repair them.

## Independent integrity checks

For every archived `records.jsonl`, an independent `jq` analysis confirmed:

- Exactly 130 records: one hello, 64 compute, 64 calibration, one complete.
- Each benchmark has exactly the indices 0 through 63, without gaps or repeats.
- Every compute record has 4096 iterations, checksum `2744405333`
  (`0xa3944d55`), positive elapsed time, and microsecond-aligned units.
- Calibration records have zero iterations/checksum, nonnegative elapsed time,
  and microsecond-aligned units. Zero observations are retained.
- The recorded hello matches `run.json`; all runs have the same seed,
  workload revision, 160 MHz CPU declaration, eight warmups, and 1000 ns
  timer resolution.
- Recomputed minimum, maximum, mean, and nearest-rank p50/p95/p99 exactly
  match both per-boot benchmark summaries in `run.json`.

All 15 preserved raw serial transcripts also passed the existing collector's
offline `benchctl validate` operation. Orders are exactly 1 through 15, with
15 unique boot identifiers. Platform order was varied:

| Platform | Run order indices |
| --- | --- |
| Bare metal | 1, 5, 9, 10, 14 |
| Embassy/esp-rtos | 2, 6, 7, 12, 13 |
| ESP-IDF/FreeRTOS | 3, 4, 8, 11, 15 |

SHA-256 checks of every per-boot copied ELF agree with its `run.json`, the
corresponding archived artifact, and its section inventory. All runs use the
same collector executable hash,
`2b623407dd5f9298a502c934b829f977dfb3337b50c9baf853b586c345c41def`.
Per-platform provenance is identical across its five boots. Source archive
hashes, workload source extracted directly from both archives, and each
archived firmware lockfile agree with the provenance. The archived
`sdkconfig.esp32c6` hash matches ESP-IDF provenance.

| Artifact | Verified SHA-256 |
| --- | --- |
| Bare-metal ELF | `c2c46633ab3d7b9fb1b6ea9541da8746200f27c0c7f0203bdaab53bb8bfe9175` |
| Embassy ELF | `c6ff29be0b56004b5b9cfe43c2f5e55a30415324abda94fc97a19ae84214894c` |
| ESP-IDF ELF | `793f3cf07d2e23b3623fbe6657d077d078c539f960fe2db97bea3dea055fe1a9` |
| HAL source archive | `c1a667d640e4ba616a647b751cfced489fd65358db4cafb12f56154de9c18656` |
| ESP-IDF source archive | `fe93690fb9420ee4aa299e9691df740de4efca0fdba9a6d6dde79bae9dfa0440` |
| Workload source in both archives | `27447ee945989a682ee778646f488e7c986d8831a38515e1057eaa143992ab9e` |
| HAL firmware lockfile | `5f6082fbe117dc7b79a10a7452f79d0407fff370fd2e422d167195d25e4c2687` |
| ESP-IDF firmware lockfile | `72d9f6a550922507b682a7ebc40623da22e62cdf12c138249a09de40ef5adace` |
| Generated ESP-IDF configuration | `8a832482dd631eb325cf5f897bf5fa4420cb2e743284d82c55e9fd0c7725b061` |

## Timing interpretation

All values below are **microseconds**. Each platform has five boots and 320
observations of each benchmark. “Mean” is the mean across the 320 compute
observations, equivalently the equally weighted mean of its five 64-sample
boot means. Ranges describe the five separate per-boot statistics; they are
not confidence intervals.

| Platform | Compute mean | Range of boot means | Range of boot p50 | Boot p95, all five | Boot p99, all five |
| --- | ---: | ---: | ---: | ---: | ---: |
| Bare metal | 282.675 | 282.640625–282.765625 | 283 | 283 | 291 |
| Embassy/esp-rtos | 282.66875 | 282.65625–282.671875 | 282–283 | 283 | 293 |
| ESP-IDF/FreeRTOS | 283.0375 | 282.96875–283.0625 | 283 | 283 | 289 |

Fractional means are arithmetic summaries of integer-microsecond observations;
they do not establish corresponding submicrosecond measurement precision.
The p50 convention is nearest rank, not the arithmetic mean of the two middle
observations. With 64 samples the p50 is sorted observation 32, p95 is 61, and
p99 is 64: **per-boot p99 is exactly the observed maximum**. It does not
establish a long-run 99th-percentile guarantee or worst-case bound.

| Platform | Calibration mean | Calibration p50/p95, every boot | Largest calibration observation |
| --- | ---: | ---: | ---: |
| Bare metal | 0.859375 | 1 / 1 | 7 |
| Embassy/esp-rtos | 0.884375 | 1 / 1 | 10 |
| ESP-IDF/FreeRTOS | 1.0375 | 1 / 1 | 2 |

No calibration value was subtracted. The different timer API paths remain
part of the observer effect. Shared compiler and byte-identical 54-byte
workload bodies control important variables, while timer implementation,
code placement, cache effects, interrupts, and SDK services still vary.
These results demonstrate functioning compute/calibration adapters. The small
compute differences do not determine an OS choice, and the unresolved runtime
flash-clock difference prevents a controlled clock-matched comparison.

A repeated positional pattern deserves a separate future experiment. In all
five bare-metal boots, compute index 0 is 291 microseconds and calibration
index 0 is 7 microseconds. In all five Embassy boots, compute index 0 is 293
microseconds; calibration index 0 is 9 or 10 microseconds. IDF's compute index
0 is 288 microseconds and its largest values, 289 microseconds, recur at
indices 26 and 62 in every boot. These are observations. First-entry cache or
timer-path effects and periodic interruption are plausible explanations, not
established causes. Workload warmup does not prove that every measurement-loop
and timer path is warm. Preserve these samples and investigate with explicit
instrumentation or a separately versioned timer/measurement warmup; silently
dropping index 0 would change this pilot's protocol and tail statistics.

## Footprint interpretation

The archived `firmware.bin` files are merged image files with boot and app
headers at offsets 0 and `0x10000`. Their measured filesystem lengths are:

| Platform | Merged image bytes | Debug ELF file bytes |
| --- | ---: | ---: |
| Bare metal | 167648 | 1494912 |
| Embassy/esp-rtos | 173040 | 1932700 |
| ESP-IDF/FreeRTOS | 429488 | 8129240 |

Merged image lengths include the particular bootloader, partition/layout
choices, address padding, and application image. ELF file lengths include
debug/symbol information. Neither column is an OS minimum or peak RAM result.
The images' nominal header mode/size/frequency bytes agree across all three
builds; the clock erratum explains why this does not prove equal runtime
flash frequency.

The complete section inventories retain the evidence needed for narrower
accounting. A raw sum of their generic kinds is unsuitable for RAM comparison:
HAL `.stack` reserves 442844/438868 bytes of remaining RAM, HAL `.text_gap`
reserves 53396/51092 bytes of flash address alignment, and IDF
`.flash_rodata_dummy` reserves 262144 bytes at the same virtual start address
as `.flash.text`. The latter is marked writable/NOBITS but is not physical
RAM or extra code payload. IDF also retains 24 bytes of RTC reservation and
10 bytes of IRAM alignment/data reservation. These remain allocated/reserved
addresses; removing them from an application-data sum does not make them free.
See [the complete exclusions and addresses](cross-platform.md#protocol-memory-and-reporting).

Actual runtime task-stack high-water, heap use, remaining usable memory,
power, boot latency, scheduler/interrupt latency, fairness, networking, and
peripheral behavior were not measured. This pilot's fixed synchronous compute
task cannot establish those properties. Selecting the next experiments should
use those missing capabilities, rather than a ranking inferred from these
near-identical compute observations.

## Audit methods

Tools: macOS `jq` 1.7.1, `shasum`, `tar`, `xxd`, the collector executable
identified by its captured hash above, and the LLVM inspection documented in
the cross-platform review. The independent summary calculation sorts raw elapsed values, uses
`ceil(percent * 64) - 1` as its zero-based nearest-rank index, and compares
every resulting field with `run.json`; it does not use those summaries as an
oracle. All raw-record checks and all 30 recomputed per-boot benchmark
summaries passed.

The offline validation command was run for each of the 15 run directories:

```text
target/release/benchctl validate --input <run-directory>/serial.raw --platform <recorded-platform>
```

SHA-256 checks covered each `runs/*/firmware.elf`, all three artifact ELFs,
both source tarballs, and `artifacts/esp-idf-freertos/sdkconfig.esp32c6`.
`tar -xOzf` streamed each archive's workload source and firmware lockfile
directly into `shasum -a 256`, without extracting or modifying source files.
No additional firmware test or hardware operation was required for this audit.
