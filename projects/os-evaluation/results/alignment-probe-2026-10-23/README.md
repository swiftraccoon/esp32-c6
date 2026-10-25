# Exploratory function-alignment results

All **five physical boots passed**. The aligned binary produced compute
medians of **282–283 microseconds**; reflashing the original binary restored
**308 microseconds** on both intervening boots. Every boot had identical
observed **160 MHz CPU / 80 MHz MSPI source / 80 MHz flash** clocks before and
after timing. The 54-byte workload body remained byte-identical.

This supports a function-placement effect in these two builds. The compiler
option changes function placement broadly, so it does not isolate one loop,
prove an exact one-cycle fetch penalty, or establish an OS performance ranking.
The original measurements remain intact in the
[separate main dataset](../clocked-handoff-2026-10-23/README.md).

## Sequence and observations

This was an **exploratory, adaptive experiment**. After the first aligned boot
returned a 283 microsecond median, we chose alternating original/aligned
reflashes to investigate reproducibility. The five-boot sequence was not
preregistered, randomized or a fixed statistical design chosen before seeing
the first result.

Each boot records 64 compute and 64 timer-observer samples after the same
two-second settling period and eight compute warmups. Compute uses seed
`0x12345678` and 4096 `integer-mix-v1` rounds. All observations and outliers are
retained; no timer overhead was subtracted. Times below are **microseconds**.

| Order / raw evidence | Binary | Compute p50 | Compute p95 | Compute maximum / p99 | Observer maximum |
| --- | --- | ---: | ---: | ---: | ---: |
| [1](runs/01-bare-metal) | Aligned | 283 | 283 | 291 | 10 |
| [2](runs/02-original) | Original | 308 | 309 | 319 | 7 |
| [3](runs/03-aligned) | Aligned | 282 | 283 | 291 | 9 |
| [4](runs/04-original) | Original | 308 | 309 | 319 | 8 |
| [5](runs/05-aligned) | Aligned | 282 | 283 | 291 | 9 |

That is **320 compute and 320 observer observations** across five resets on
one ESP32-C6 revision 0.2: three aligned boots and two original boots. These
are repeated observations within boots, not 640 independent hardware trials.
Observer p50/p95 is 1 microsecond in every boot; zero observer values remain
valid at the timer's 1 microsecond resolution. With 64 samples, nearest-rank
p99 equals the observed maximum. First sample index 0 contains each boot's
largest compute and observer value; none was removed. Fractional means in
[summary.json](summary.json) do not imply submicrosecond precision.

The collector requested RTS resets without an externally controlled power
cycle; ROM reset logs report `POWERON`. Boot time, cold-start behavior and
power use were not measured by this experiment.

## What changed

The aligned build adds
`RUSTFLAGS='-C llvm-args=-align-all-functions=2'` with the same Rust 1.96.0,
source, dependencies, target, release settings and clock setup. It uses an
isolated ignored target directory. The five full-width loop instructions move
from addresses modulo four equal to two to addresses modulo four equal to zero.
Their exact 54-byte/18-instruction body hash stays
`b917090559ff1edf22ad5ac21975f1a998e5594b9ad23db9344d5c71c2b3af49`.
Caller/timer/helper placement can change too. See the
[build record](../../docs/builds/alignment-probe.md),
[binary inspection](../../docs/research/code-placement.md).

| Variant | ELF SHA-256 | Merged image SHA-256 |
| --- | --- | --- |
| Aligned | `19184daf8b628b690655fa0ea23453fd5519f630e58450141a7280395d33f741` | `295f5d358f713ce14389a3d769eeeffe106eb73489006ef4d80824198911a8ca` |
| Original | `be14bac3563b52d5e043485c9f798a4e23d6ed8702b72c9a3028c1cf46f58d0b` | `f404a4840ea2c6d700140e37550f627c00b6463e4dee0f962d2ee979f5e80053` |

Aligned build and inspection logs are under [artifacts](artifacts). The original
variant used the exact main dataset's bare-metal ELF and image, identified by its
[provenance](../clocked-handoff-2026-10-23/artifacts/bare-metal).
Both variants reference the same preserved HAL source archive in that sibling
dataset. Each run retains its raw serial bytes and
capture metadata; offline replay and an independent count/checksum/clock/statistic
audit passed for all five.

[SHA256SUMS](SHA256SUMS) records collection-time archive identities. Compiled
artifacts are outside this checkout, so this historical manifest does not verify
the complete current checkout. Raw protocol transcripts are unchanged. See the
[snapshot review](../../tools/benchctl/docs/api.md) for available artifacts
and current verification commands.

The full original **8,388,608-byte flash was restored, verified against the
backup and reset** after the experiments. Restoration logs and device state
are recorded in the sibling [device-state record](../clocked-handoff-2026-10-23/device-state.json).
