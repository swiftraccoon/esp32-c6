# Existing benchmark assets to reuse

Reviewed 2026-10-23. Existing projects already supply useful scheduler, radio,
power and compute tests. Reuse their workloads and instrumentation where their
contracts fit; retain our Rust collector for common correctness checks and
provenance. A new implementation needs a concrete missing capability or an
integration reason. The [prior-work review](prior-work.md) explains which
decisions need new measurements.

## Upstream assets and their roles

| Asset | Verified source and revision | Reuse decision |
| --- | --- | --- |
| Zephyr kernel latency suite | [Latency README](https://github.com/zephyrproject-rtos/zephyr/blob/76280bce2a050376d57e14240687b5c283dea0e7/tests/benchmarks/latency_measure/README.rst), [test configuration](https://github.com/zephyrproject-rtos/zephyr/blob/76280bce2a050376d57e14240687b5c283dea0e7/tests/benchmarks/latency_measure/tests.yaml) | Qualify the unmodified suite on C6 before designing Zephyr-specific kernel microbenchmarks. It already distinguishes yield, ISR return/reschedule, synchronization and task lifecycle operations. |
| Zephyr kernel object suite | [SysKernel README](https://github.com/zephyrproject-rtos/zephyr/blob/76280bce2a050376d57e14240687b5c283dea0e7/tests/benchmarks/sys_kernel/README.txt), [test configuration](https://github.com/zephyrproject-rtos/zephyr/blob/76280bce2a050376d57e14240687b5c283dea0e7/tests/benchmarks/sys_kernel/tests.yaml) | Reuse named object-operation scenarios and validation for semaphore/FIFO/LIFO/stack/slab tests. Report their aggregate averages as such, separately from per-event distributions. |
| Ariel native scheduler tests | [Current yield test](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks/bench-sched-yield), [thread-flag test](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9/tests/benchmarks/bench-sched-flags) | First qualify these on the attached C6. Preserve their operation counts and timing units; use them to inform a comparable Rust native-thread adapter. See [the source audit](prior-work-c6-os.md). |
| Ariel paper's experiment repository | [Author artifact](https://github.com/elenaf9/ariel-os-benchmarks/tree/eaa22a553c08646b584abf0a04c84d6da7ddd40f) | Independently implement the published preemption contract where useful, or resolve source-copy permission first. Distinguish it from two-channel request/reply timing. See [the scheduling review](prior-work-rust-scheduling.md). |
| Electric UI radio experiment | [Public artifacts](https://github.com/Scottapotamas/embedded-wireless-latency-eval/tree/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577), [MIT license](https://github.com/Scottapotamas/embedded-wireless-latency-eval/blob/a4c16a10af1ed971e7e7b7b7a367f38b6bdbd577/LICENSE) | Reuse C6 raw-802.15.4 packet sweeps, captures and external timing design after checking [the radio review](prior-work-c6-radio-runtimes.md). Its older-ESP32 Wi-Fi/BLE results do not identify C6 behavior. |
| Johansson and Poli radio study | [Experiment repository](https://github.com/Anton-Johansson-it/performance-evaluation-esp32-c6/tree/03c1205878af427709426774294abba0a2dbfb1c) | Reuse configuration and measurement ideas after reconciling thesis descriptions with actual packet-generation code. These are protocol comparisons within IDF, not OS comparisons. |
| Existing IDF compute port | [esp-idf-benchmark](https://github.com/nopnop2002/esp-idf-benchmark/tree/8bf1b6a8b9cb0ab5ac8c535e4766a963578ef74b) | Optional CPU/compiler calibration. It already has C6 Dhrystone/Whetstone runs for IDF 5.1 and different optimization settings; do not rebuild a custom equivalent to score operating systems. |

The Zephyr source snapshot above is upstream HEAD at inspection, not a selected
benchmark release. Neither suite was built or run on this C6 in this review.
Their integration-platform lists do not establish a C6 pass. Select a supported
release, qualify its timing backend, and archive configuration before using
its numbers. The latency suite also has user/kernel and object-statistics
variants; protection and instrumentation settings are experimental factors,
not free performance improvements.

The IDF compute port is MIT-licensed C. Reading its results or evaluating its
unmodified code does not require a new language choice. Any authored C/C++ or
assembly adapter still needs the language discussion in the coding standard.
An independently authored Rust workload should cite the experimental contract
it follows rather than silently copying restricted source.

## Measurement practice to adopt

Espressif's [C6 performance guidance for our selected SDK](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-guides/performance/speed.html)
already warns that small routines can change timing with binary/cache layout.
It distinguishes microsecond timer overhead from lower-overhead cycle timing
and recommends external timing where appropriate. Our placement result is
local calibration of a known issue, not discovery of a new optimization.

Zephyr's [Latmon example](https://docs.zephyrproject.org/latest/samples/net/latmon/README.html)
is a useful external GPIO-response and loopback-calibration reference. It
requires another GPIO-capable networked system and a matching Latmus service;
it is not already a runnable fixture for our single connected board. We can
adopt its external stimulus/response boundary without committing to its full
networked tooling.

Before adapting any asset, record:

1. The decision it could change and the exact timed endpoints.
2. Its source revision, input sizes, clock/timer configuration, measurement
   units, operation count and correctness check.
3. Source license and whether we will run unmodified code, port permitted code,
   or independently implement the published protocol in Rust.
4. The available board pins, peers and instruments, including calibration.
5. Which results are direct replications, controlled variations or exploratory
   measurements. Changed inputs or backends create a new result category.

Known unmodified workloads are valuable controls. Our maintained code should
add missing validation, common Rust application behavior, raw distributions,
build provenance or failure/recovery coverage, rather than duplicate a mature
upstream test just to give it a new name.
