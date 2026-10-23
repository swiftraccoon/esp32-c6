# Independent alignment-probe results audit

Reviewed 2026-10-23 using the five preserved
[alignment runs](../../results/alignment-probe-2026-10-23/runs/).
All five physical boots pass the raw protocol and integrity checks below.
This audit used read-only analysis and wrote result documentation/summary;
it performed no hardware access, source change, build or Git mutation.
The [build](../builds/alignment-probe.md) and
[binary review](clocked-workload-binaries.md) establish the source/flag/body
identity separately.

## Design and inference boundary

Sequence: **aligned / original / aligned / original / aligned**. The first
aligned observation was already available, with a 283 microsecond median,
before choosing the subsequent alternating reflashes. This is an exploratory,
adaptive reversal experiment, not a preregistered or randomized design.
It uses one board and five RTS-requested reset sessions without a controlled
power cycle. ROM reset logs say `POWERON`; the reset request is not an external
measurement of power/cold-start behavior.

The repeated return to 308 microseconds with the original binary and
282–283 microseconds with the aligned binary supports a compiler function-layout
effect under the observed conditions. It does not attribute the change to one
specific instruction-fetch or branch mechanism. The LLVM option affects function
placement broadly, including possible caller/timer/helper placement. Neither
an isolated loop-alignment intervention nor a performance-counter comparison
was performed. An arithmetic resemblance to one extra 160 MHz cycle per round
is a hypothesis, not proof of an exact hardware penalty.

Do not replace the main experiment's original 308 microsecond result with this
probe or treat the two bare-metal builds as two operating systems. The
[main dataset](../../results/clocked-handoff-2026-10-23/README.md) remains separate.

## Independent integrity checks

Every raw transcript passed the existing Rust collector's offline operation:

```text
target/release/benchctl validate --input <run-directory>/serial.raw --platform bare-metal
```

The inspected collector executable's SHA-256 is
`6f1a1a38c5079492f957a12ea76193f6b3ce7a8fe5d9dbbbb81c1331b327ac55`,
matching capture metadata in every run. Root-owned `replay.json` records are
separate retained replay evidence.

A separate `jq` analysis of `records.jsonl` checked each of these conditions:

- Exactly 130 JSON records: one hello, 64 compute, 64 observer, one complete.
- Compute then observer indices are exactly 0 through 63 in order.
- Every compute record has protocol 2, 4096 iterations, checksum `2744405333`
  (`0xa3944d55`), positive elapsed time and 1000 ns alignment.
- Every observer record has protocol 2, zero iterations/checksum, nonnegative
  elapsed time and 1000 ns alignment.
- Hello matches captured metadata after removing its wire-only `event` key;
  platform, workload, seed, counts, warmups and timer resolution match the
  common contract.
- Completion reports 128 / 64 / 64 samples, matches actual observations and
  carries the same complete clock object as hello and captured metadata.
- All four raw clock registers are identical across all five boots and both
  snapshots. An arithmetic decoder independent of the Rust collector confirms
  PLL source 1, CPU 160 MHz / MSPI 80 MHz / flash 80 MHz / divider 1.
- Every per-boot minimum, maximum, mean and nearest-rank p50/p95/p99 was
  independently recomputed from sorted raw elapsed values and exactly matches
  `run.json`: ten benchmark summary comparisons passed.

Raw JSON lines were extracted from each `serial.raw`, canonicalized with
`jq -Sc`, and compared with canonicalized `records.jsonl`; all five comparisons
passed. The collector validates full transcripts, including boot boundaries;
the independent JSON comparison checks the saved parsed records against the
original received bytes rather than assuming they match.

Order indices are exactly 1–5, with five unique independent boot identifiers.
All capture statuses are pass with no recorded failure. There are no application
panic or benchmark-failure diagnostics in the inspected raw transcripts.
The largest observations remain in the data.

The common raw clock values are:

| Register | Decimal | Decoded relevance |
| --- | ---: | --- |
| `pcr_sysclk_conf` | 671154688 | PLL source 1, root HS divider 3 |
| `pcr_cpu_freq_conf` | 0 | CPU HS divider 1, no force-120 |
| `pcr_mspi_clk_conf` | 1280 | MSPI HS divider 6 |
| `spi0_clock` | 2147483648 | Bit 31 bypass, bus divider 1 |

Thus CPU is 480/3/1 = 160 MHz, MSPI is 480/6 = 80 MHz and flash is 80/1 =
80 MHz. This is runtime register evidence, distinct from nominal image headers
or a physical oscilloscope frequency measurement.

## Artifact and provenance identity

All five per-run ELF files were hashed and compared byte-for-byte with the
expected variant. Three aligned ELFs match the probe artifact; two originals
match the exact main bare-metal artifact. Each run's provenance object was
canonicalized and compared with its variant's preserved provenance; all five
match. The collector hash and shared source/lock/workload identities agree.

| Evidence | Verified SHA-256 |
| --- | --- |
| Aligned ELF | `19184daf8b628b690655fa0ea23453fd5519f630e58450141a7280395d33f741` |
| Original ELF | `be14bac3563b52d5e043485c9f798a4e23d6ed8702b72c9a3028c1cf46f58d0b` |
| Aligned merged image | `295f5d358f713ce14389a3d769eeeffe106eb73489006ef4d80824198911a8ca` |
| Original merged image | `f404a4840ea2c6d700140e37550f627c00b6463e4dee0f962d2ee979f5e80053` |
| Shared source archive | `5d46a17fae89ca31c79802c55aa2610fd2affeee24af35f28c59ba9a8e3504c6` |
| Shared workload source | `27447ee945989a682ee778646f488e7c986d8831a38515e1057eaa143992ab9e` |
| Shared HAL Cargo.lock | `fe83fea7296603dd9a461ed872fcecab02b277754a18060b0ac0e8cbc48ed89f` |
| Common 54-byte workload body | `b917090559ff1edf22ad5ac21975f1a998e5594b9ad23db9344d5c71c2b3af49` |

The source archive and original artifacts live in the sibling clocked-handoff
experiment and are intentionally reused. The probe's source-input comparison
with that archive passed before capture. Rust source and dependency versions
are unchanged; the aligned build's additional RUSTFLAGS is explicitly recorded.

## Raw timing assessment

All values in the table are microseconds. Means are arithmetic summaries of
integer-microsecond observations, not more precise measurements.

| Order | Variant | Compute mean | Compute p50 | Compute p95 | Compute p99 / maximum | Observer mean | Observer maximum |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | Aligned | 282.640625 | 283 | 283 | 291 | 0.90625 | 10 |
| 2 | Original | 308.328125 | 308 | 309 | 319 | 0.859375 | 7 |
| 3 | Aligned | 282.625 | 282 | 283 | 291 | 0.875 | 9 |
| 4 | Original | 308.328125 | 308 | 309 | 319 | 0.875 | 8 |
| 5 | Aligned | 282.625 | 282 | 283 | 291 | 0.875 | 9 |

Aligned runs contain 192 compute samples spanning 282–291 microseconds;
original runs contain 128 spanning 308–319 microseconds. Total observations
are 320 compute and 320 observer samples. Every observer p50/p95 is
1 microsecond, and zeros remain valid. Each boot's compute maximum and observer
maximum occur at index 0; those observations were not removed or attributed to
a particular cause. Cache/timer-path effects remain possible explanations.

Percentiles use nearest rank: for 64 samples, p50 is sorted element 32, p95
is 61 and p99 is 64, so p99 is the maximum. Samples in a boot are not independent
boots. Three aligned/two original boots are insufficient to establish a
population latency bound or worst-case guarantee; no inferential confidence
interval or significance claim is made. No observer subtraction or outlier
exclusion was used.

[The machine-readable summary](../../results/alignment-probe-2026-10-23/summary.json)
retains per-boot statistics, sequence, exact variant identities, sample counts,
clock object, audit outcomes and limitations. Its aggregate means are explicitly
arithmetic means of the per-boot means; raw files remain the underlying evidence.

## Verification and final device state

Tools were the captured `benchctl`, jq 1.7.1, macOS `cmp`, `shasum`, `tar` and
`rg`. Numerical checks sort raw nanoseconds and compute `add / length`, using
zero-based indices 31/60/63 for nearest-rank percentiles. This independent
calculation does not use `run.json` summaries as its oracle. `jq` parsed the
finished summary, and `git diff --check` passed for this review, README and
summary. No implementation or test was added for this audit.

The hardware operator subsequently confirmed full original 8,388,608-byte
restoration, successful separate digest verification, and reset to the original
application. The sibling
[device-state record](../../results/clocked-handoff-2026-10-23/device-state.json)
and its restoration logs retain this operational evidence. This read-only
review did not perform the restoration or imply a new qualification of the
original application.
