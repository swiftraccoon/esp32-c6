# ESP32 C6 capture and validation

This checkout includes source and textual results, without compiled artifacts.
After building, run
`projects/os-evaluation/target/release/benchctl verify-public-snapshot --root . --manifest quality/publication.json`.
See its [scope and safety contract](docs/api.md).
The strict original-evidence commands below still require complete experiment
bundles and their history; this snapshot check does not grant method admission.

`benchctl` is the Rust host tool for the C6 evaluation protocols. It replays the initial protocol 1 compute/observer trials and validates protocol 2 clock-controlled scheduler trials. Build from the repository root with `cargo +1.96.0 build --manifest-path projects/os-evaluation/Cargo.toml --locked --release -p benchctl`. The OS project's toolchain and lockfile pin its build environment. Commands below run from the repository root.

## Capture a run

```sh
projects/os-evaluation/target/release/benchctl capture \
  --purpose qualification \
  --port /dev/cu.SERIAL_PORT \
  --output projects/os-evaluation/results/example/boot-01 \
  --platform bare-metal \
  --elf path/to/firmware.elf \
  --provenance path/to/build-provenance.json \
  --boot-id bare-metal-01 \
  --run-order 1
```

Capture actively resets the board using RTS with DTR deasserted. It uses 115200 baud, 8 data bits, no parity and one stop bit. It does not flash or provision the board. The output directory must not exist: an earlier run is never overwritten.

New captures require an explicit `--purpose qualification|comparison`.
Qualification pilots cannot enter comparative analysis. A comparison additionally
requires `--method <definition>` and matching qualified ELF/provenance bytes
before serial access. New capture reports use schema 4; their JSON wire
protocol is unchanged. See [the workflow](../../../../docs/assurance-workflow.md).

The directory retains `firmware.elf`, the original `serial.raw`, parsed `records.jsonl`, and `run.json`. Failure, malformed data, checksum mismatch, duplicate/missing samples, unexpected reboot, wrong platform, incompatible configuration and timeout produce a failed run with retained evidence. Both protocol versions require 64 ordered compute samples and 64 timer observations. Protocol 2 scheduler platforms additionally require 1,000 validated message roundtrips. Completion requires the actual samples and exact reported totals. A successful capture is not a qualification of the entire OS.

Input lines are bounded to 4096 bytes. Ordinary boot messages outside JSON are retained without becoming measurements. Fatal diagnostics and additional boots reject the attempt; they are not discarded as noise. Once the stream or protocol validator rejects input, the boot cannot later become successful. Raw samples and timing outliers are preserved. `host_rustc_at_capture` observes the compiler on PATH when collecting, while the collector executable hash identifies the actual binary; firmware compiler provenance is supplied separately.

## Single-boot text admission

Capture and replay for **all five JSON protocols** share the same text guard and
bounded byte framer. Exactly one recognized `ESP-ROM:esp32c6` startup must
precede the first JSON record. Its date suffix is allowed; another chip token,
including `esp32c61`, is not. At most one initial `rst:` reason is allowed;
another startup/reset reason or one after JSON starts rejects the attempt.

Known non-JSON panic, Guru Meditation, assertion/abort, reboot, watchdog,
brownout, stack-protection and image-SHA failure diagnostics reject the attempt,
including before the first JSON record. JSON configuration strings such as
`"panic": "abort"` are parsed as protocol data, not searched as console errors.
The guard recognizes these diagnostic formats; it does not authenticate device
execution or guarantee recognition of every possible future/custom failure.

The complete received raw chunk is written before admission. A failed capture
retains those bytes and a failed report. Completion also checks the remainder
of that chunk. A pending partial line after completion is read until its newline,
EOF or timeout, so a split panic/reboot is not ignored. An unterminated tail at
EOF fails. Capture stops after valid completion and fully consumed received
lines; it cannot certify diagnostics arriving after it returns. Replay instead
checks the entire retained file through EOF.

**CLI compatibility change:** replay now requires a full boot transcript, not
JSON records alone. Use `serial.raw`; `records.jsonl` omits startup/diagnostic
evidence and is insufficient for admission. The JSON wire schema and existing
summary fields are unchanged. All 61 existing protocol 1/2/3 raw captures replay
with byte-identical archived summaries after this change. The
[boot admission contract](docs/api.md) records the
reproduced defect, regression checks and remaining observation limits.

## Replay and inspect

```sh
projects/os-evaluation/target/release/benchctl validate --input path/to/serial.raw --platform bare-metal
projects/os-evaluation/target/release/benchctl inspect --elf path/to/firmware.elf --output sections.json
```

Replay validates the whole archived transcript. Percentiles use nearest rank; with only 64 samples, p99 equals the maximum. These are measurements within a boot, not 64 independent boots. Timer readings are quantized to 1000 ns. Zero calibration duration is valid; zero compute duration is rejected. Timer overhead is reported independently and is not subtracted.

## Protocol 2 contract

The hello retains protocol 1's shared seed, rounds, workload revision, `cpu_mhz: 160`, `sample_count: 64`, eight compute warmups, and `timer_resolution_ns: 1000`. Its `benchmarks` object must exactly match the selected platform:

| Platform | Required benchmark sample counts |
| --- | --- |
| `bare-metal` | `compute: 64`, `timer-overhead: 64` |
| `embassy`, `esp-idf-freertos` | `compute: 64`, `timer-overhead: 64`, `message-roundtrip: 1000` |

Each benchmark has ordered, unique indices starting at zero. Compute and timer records keep protocol 1's workload/checksum rules. A message record has `iterations: 1`, `checksum: index XOR 0xa5a55a5a`, and elapsed nanoseconds divisible by 1000. A zero message duration is retained as resolution evidence. It is not interpreted as zero-cost scheduling.

Scheduler hello records require a `message` object with a nonempty descriptive `transport`, `queue_capacity: 1`, and `warmup_iterations: 8`. Additional diagnostic fields, including actual priorities, stack sizes and sample-buffer sizes, are preserved. Bare-metal omits `message`; an unavailable scheduler benchmark never becomes a zero-valued result.

Hello's `clocks` and completion's `clocks_after` use this exact shape; register values below illustrate the supported divider configuration, not an observed device reading:

```json
{
  "source": "register-readback",
  "cpu_mhz": 160,
  "flash_source_mhz": 80,
  "flash_mhz": 80,
  "flash_divider": 1,
  "registers": {
    "pcr_sysclk_conf": 66048,
    "pcr_cpu_freq_conf": 0,
    "pcr_mspi_clk_conf": 1280,
    "spi0_clock": 2147483648
  }
}
```

The host decodes the four raw registers independently of the advertised MHz. It requires the supported SPLL source/system divider, CPU 160 MHz, flash source 80 MHz, flash divider 1 and resulting flash 80 MHz. Missing registers, inconsistent declarations, unsupported sources/dividers and any difference between initial and final snapshots fail validation. This reads configuration; it does not establish externally calibrated frequency or prove that no transient change occurred between snapshots. A passing protocol 2 `run.json` includes the final snapshot. Firmware source and flash/build provenance still establish which supplied image was loaded; the serial protocol alone cannot authenticate the running image.

Completion includes `samples: 128`, `compute_samples: 64`, and `timer_overhead_samples: 64` for bare-metal. Scheduler completion instead uses `samples: 1128` and additionally `message_roundtrip_samples: 1000`. Bare-metal omits that last field. Advertised counts cannot substitute for the actual validated samples. Unknown benchmark declarations and unknown protocol 2 hello configuration fail.

Existing protocol 1 firmware records retain their validation/output behavior: declared CPU configuration, no clock-readback requirement, no scheduler result, and numeric completion metadata discarded from parsed records. All 15 initial archived validation outputs were compared byte for byte with the preceding collector binary. Newly reserved optional fields such as `benchmarks`, `clocks` and completion counts now have typed shapes even in protocol 1; hypothetical old diagnostic metadata using those names with other shapes can fail decoding. No supplied archive has that collision.

ELF inspection lists allocated sections and their kinds. File size includes debug information. `.stack`, `.text_gap`, and IDF mapping reservations such as `.flash_rodata_dummy` must not be summed into measured RAM or flash use. Use the actual generated image and individual section meanings, and measure stack/heap usage separately.

The [API and safety contract](docs/api.md) covers all 54 API entries, rejection
boundaries and dependency-license exceptions. See the [benchmark plan](../../docs/benchmark-plan.md) for the wider experiment protocol.

## Protocol 3 queue comparison

The [same-IDF queue experiment](../../docs/experiments/native-queue-comparison.md)
uses a separate strict protocol. Existing `capture` and `validate` retain their
protocol 1/2 JSON contracts, with the stricter boot-text admission above. Select
the newly built image's exact backend explicitly:

```sh
projects/os-evaluation/target/release/benchctl capture-queue \
  --purpose qualification \
  --port /dev/cu.SERIAL_PORT \
  --output projects/os-evaluation/results/example/runs/s1-b01-01-std \
  --backend rust-std-sync-channel \
  --elf path/to/queue-std.elf \
  --provenance path/to/build-provenance.json \
  --boot-id s1-b01-01-std --run-order 1
projects/os-evaluation/target/release/benchctl validate-queue \
  --input results/example/runs/s1-b01-01-std/serial.raw \
  --backend rust-std-sync-channel
```

Use `freertos-queue` for the native image. Capture defaults to 180 seconds
because transmitting 2,000 post-measurement records at 115200 baud exceeds the
legacy 30-second window. This deadline includes boot and serial transmission;
the separate one-second message-duration admission guard rejects an excessively
slow completed exchange as a failed boot, retaining its raw evidence. A hang
still reaches the host deadline. Capture preserves the original
serial bytes, parsed records, supplied ELF, provenance, executable hashes and
failed status. It never flashes or replaces an existing trial directory.

Every protocol 3 event carries `protocol: 3`, the same device-generated
`boot_nonce` and the selected `backend`. The event sequence is one `hello`,
1,000 ordered `timer-overhead` samples, 1,000 ordered `message-roundtrip`
samples, one `qualification` and one `complete`. Unknown events/fields, absent
fields, incorrect backend, changed nonce and reordered/missing/duplicate
records fail. Non-JSON boot messages remain raw evidence. With
`--previous-nonce <u64>` repeated for earlier observations, hello also rejects
a reused nonce. The nonce is a non-cryptographic startup observation; it does
not authenticate firmware, prove electrical reset or establish statistical
independence. A lone capture cannot discover nonces that were not supplied.

Hello requires the pinned `esp-idf-freertos` platform, `tick_hz: 16000000`,
`tick_width_bits: 52`, `sample_count: 1000`, `observer_count: 1000`,
`warmup_iterations: 8`, `queue_capacity: 1`, `message_bytes: 4`, observed
`main_priority: 1`, `worker_priority: 5`, `rtos_tick_hz: 100`,
`main_stack_bytes: 32768`, `worker_stack_bytes: 8192` and
`sample_storage: "preallocated-heap"`. `sample_buffer_bytes` must be positive
and at most 128 KiB; exact type layout remains image provenance. Allocating
that storage occurs before measured intervals.

Each sample contains `index`, `start_tick`, `end_tick`, `checksum`,
`phase_sequence` and `blocked`. Raw timestamps must fit 52 bits, have ordered
endpoints and never precede the preceding recorded endpoint. Backwards values
are rejected rather than interpreted as a decades-long wrap. Observer metadata
is checksum/phase zero and `blocked: false`; a zero observer duration is
retained. Message metadata requires `checksum: index XOR 0xa5a55a5a`,
`phase_sequence: index + 8`, `blocked: true` and a positive duration. Readiness
checks occur outside the interval. The higher-priority responder can publish
its next phase before the main thread completes the previous receive; that
worker bookkeeping remains part of the measured application path.

`config_before` and `config_after` contain the existing `clocks` object plus
`timer`. The latter requires `source: "xtal"`, `enabled: true`,
`unit0_enabled: true`, `unit0_stalls_cpu: true` and three raw register values:
`systimer_conf`, `pcr_systimer_conf`, `pcr_systimer_func_clk_conf`. The host
independently decodes UNIT0 enable/stall bits 30/28, PCR APB enable/reset bits
0/1 and XTAL selector/function-enable bits 20/22. Bit 31 of SYSTIMER CONF is
retained without treating register clock gating as the counter's function
enable. Both full configuration snapshots must be equal. This is decoded
configuration evidence, not independent oscillator calibration or proof against
transient intermediate changes.

Qualification retains at least 1,000 ordered counter reads, zero `backsteps`,
the first/last ticks, a sleep interval bracketed by legacy microseconds and
the SDK WFI-call interval bracketed by FreeRTOS ticks. The sleep check must
span at least 10 ms in raw ticks and agree with its outer legacy bracket
within the declared 1 ms allowance (plus two microseconds for quantization).
The nominal 20 ms sleep qualification may span at most 500 ms.
The WFI check requires no attached debugger, 1–10,000 calls and 3–50 elapsed
100 Hz kernel ticks; raw elapsed ticks must agree within one kernel tick.
Qualification timestamps precede observer samples. These checks establish
continuity and implementation/scale consistency using onboard observations;
the legacy API shares the SYSTIMER source and is not an external reference.

Qualification also requires all 1,008 warmup/measured readiness and blocked
checks, 1,008 native empty-queue checks (zero for the std backend), and passed
independent empty/full/probe-cleanup checks. Completion requires exactly 2,000
actual samples, matching per-benchmark totals, worker join and backend teardown.
Qualification booleans summarize reviewed firmware checks; the serial protocol
cannot independently inspect task wait lists or memory ownership.

Historical protocol-3 reports use `schema_version: 3`; new capture reports use
schema **4** with mandatory purpose/admission metadata. Accepted summaries remain under
`benchmarks`, with `sample_count`, `min_ticks`, `p50_ticks`, `p95_ticks`,
`p99_ticks`, `max_ticks` and `mean_ticks`. Scale is the rational
`tick_scale_seconds: {"numerator": 1, "denominator": 16000000}`. Gross raw
ticks and outliers are retained; no observer subtraction, per-event whole
microsecond truncation or invented integer nanosecond quantum is applied.
Failure produces no numeric summary. The
[API contract](docs/api.md) records API
applicability and actual verification.

## Frozen queue study

New comparisons require a schema-2 study manifest pinning the method definition
and schema-4 comparative capture receipts. Qualification pilots and missing
purpose cannot be promoted. Legacy schema-1 studies/schema-3 reports are
supported only with the known historical collector identity, preserving the
existing study without claiming this new gate ran during it.

`queue-schedule --seed <u64> --output <study>/schedule.json` writes a new,
balanced 12-block/24-boot schedule and refuses overwrite. It uses the recorded
deterministic permutation algorithm; keep the schedule, seed and image manifest
committed before collecting comparative runs.

`analyze-queue --schedule <study>/schedule.json --results <study>` replays every
scheduled raw transcript and checks its report, image, provenance, nonce,
configuration and chronological timestamp. It requires
`<study>/study-manifest.json` with `schema_version: 1`, experiment
`idf-native-queue-v1`, the schedule's `schedule_sha256`, and an `images` object
keyed by `rust-std-sync-channel` and `freertos-queue`. Each image entry supplies
`elf_sha256` and `provenance_sha256`, matching
`artifacts/std/{firmware.elf,provenance.json}` or the equivalent `native` path.
Additional human-readable manifest metadata can record the collection scope.

The output retains each scheduled attempt and native-minus-std boot-p50
contrast. Missing, failed or invalid attempts make the study incomplete; only
12 complete blocks produce the explicitly conditional median interval.
Fixture invalidations and their one permitted block repeat need an external
attempt ledger; the analyzer never automatically replaces application failures.
See the [design](../../docs/research/experimental-design.md) and
[analysis review](../../docs/experiments/native-queue-comparison.md) for the estimator and limits.

## Finite offered-load qualification

Protocol 5 retains all 2,016 nominal release rows across six finite pulses,
including generator misses and queue-full rejections. It uses a separate
qualification-only command; purpose and expected statistics profile are
mandatory. The default capture timeout is 120 seconds because the records are
printed only after collection and cleanup, over 115200-baud UART.

```sh
projects/os-evaluation/target/release/benchctl capture-load --purpose qualification \
  --profile runtime-stats-disabled --port /dev/cu.SERIAL_PORT \
  --output .cache/load-pilot-off-01 --elf path/to/firmware.elf \
  --provenance path/to/provenance.json --boot-id load-off-01 --run-order 1
projects/os-evaluation/target/release/benchctl validate-load --profile runtime-stats-disabled \
  --input .cache/load-pilot-off-01/serial.raw
```

The enabled image uses `--profile runtime-stats-enabled`. Capture resets the
device and preserves original ELF/provenance bytes, raw input, parsed records
and schema-4 `run.json`, including failures. It does not flash. Archive the
standalone replay as `replay.json`; whole-archive replay binds its contents to
the capture report and retained input identities. Comparison purpose is rejected
before output creation or serial access. These pilots supply no comparative
score or promotion path into an instrumentation-cost study.

The validator checks the nominal grid, every ID/status/checksum, FIFO service,
quiescence/wakeup counts, queue bounds, offered-work counts and synthetic
deadlines. It retains conditional quantiles with missed/rejected work counted
separately, nullable unavailable counters, gross observer durations and settled
whole-phase attribution including drain. Neither profile is an uninstrumented
application; global memory peaks, physical CPU busy time and sustained capacity
remain outside this qualification. See the
[contract](../../docs/experiments/idf-load-qualification.md),
[protocol](../../docs/experiments/idf-load-protocol.md) and
[host contract](docs/api.md).

## Read-only evidence checks

The plain-text RTIC qualification uses its own adapter and the same diagnostic
guard. It records host-monotonic observation receipts alongside raw console
bytes, including a full twelve-second interval after its last expected marker:

```sh
projects/os-evaluation/target/release/benchctl capture-rtic --purpose qualification \
  --port /dev/cu.SERIAL_PORT --output .cache/rtic-pilot-01 \
  --elf path/to/monotonic.elf --provenance path/to/provenance.json \
  --boot-id rtic-01 --run-order 1
projects/os-evaluation/target/release/benchctl validate-rtic --input .cache/rtic-pilot-01/serial.raw \
  --observations .cache/rtic-pilot-01/observations.jsonl
```

Capture requests reset but does not flash. Raw text alone cannot establish the
elapsed post-marker interval: replay requires the original timing receipt too.
The [RTIC contract](../../docs/experiments/rtic-qualification.md) fixes the seven
source-derived markers, coarse host-arrival budgets, 45-second ceiling, failure
retention and qualification limits. These are console/timer method checks,
without benchmark scores, calibrated device timing or a preemption result.

```sh
projects/os-evaluation/target/release/benchctl verify-evidence --root . --ledger quality/evidence.json
projects/os-evaluation/target/release/benchctl replay-archives --root .
projects/os-evaluation/target/release/benchctl qualify-method --method quality/methods/idf-queue-v1.json
```

`verify-evidence` checks all registered collection inventories/digests, frozen
inputs, scoped evidence classes, coverage references and supported local
Markdown targets. Stage new proof files before a final check; an untracked
local file is insufficient for a committed reference. It neither reseals data
nor validates scientific semantics from a label.

`replay-archives` compares successful protocol outputs and supported study
summaries byte-for-byte with retained results. Failed captures remain failures;
rendered native-console suites are not silently treated as JSON measurements.
RTIC raw/receipt replays are counted separately as `successful_console_captures`.

`qualify-method` checks exact artifacts, matching positive captures, full metric
contracts, required compiled/output text and five actual rejection controls per
variant. These are consistency/control checks, not independent hardware or
accuracy certification. The included method uses old pilots retrospectively
for future exact-image captures. No command opens a serial port.

The [assurance workflow](../../../../docs/assurance-workflow.md),
[ledger](../../docs/evidence-ledger.md) and
[method review](../../docs/assurance-workflow.md) specify limits and
prospective collection requirements.

## Accounting qualification

Protocol 4 is a separate diagnostic defined by the
[frozen accounting contract](../../docs/experiments/idf-accounting-qualification.md).
It checks task-attributed wall-time and heap controls, without a comparative
estimator or CPU-utilization score. Explicit qualification purpose is required;
comparison purpose is rejected before output creation or serial access.

```sh
projects/os-evaluation/target/release/benchctl capture-accounting --purpose qualification \
  --port /dev/cu.SERIAL_PORT --output projects/os-evaluation/results/example/boot-01 \
  --elf path/to/accounting.elf --provenance path/to/provenance.json \
  --boot-id qualification-01 --run-order 1
projects/os-evaluation/target/release/benchctl validate-accounting \
  --input results/example/boot-01/serial.raw
```

The first command requests a warm reset and needs the prepared qualification
image on the device. It preserves the original provenance bytes as
`provenance.json`, ELF, raw input, accepted record lines and a schema-4 report.
Failure retains evidence with no favorable qualification summary. The second
command is read-only and uses the same strict validator and single-boot guard.
Save its successful stdout as `replay.json` when packaging a completed attempt.

All six snapshots, complete task identities and gross observer endpoints remain
visible. Signed `unattributed_boundary_residual_us` can be positive or negative;
it is not idle time. Stack-fill/allocator indicators establish neither global
runtime peaks nor fragmentation. The [host contract](docs/api.md)
records synthetic and adverse controls separately from physical qualification.
