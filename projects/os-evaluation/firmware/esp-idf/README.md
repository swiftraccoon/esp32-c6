# ESP-IDF / FreeRTOS benchmark

These Rust `std` binaries use unmodified ESP-IDF 5.5.6 through
[`esp-idf-sys` 0.37.2](https://github.com/esp-rs/esp-idf-sys/tree/v0.37.2).
The default `esp-idf-baseline` measures the shared `integer-mix-v1` compute workload, timer observer cost,
and a request/reply handoff between two actual FreeRTOS tasks using Rust's
bounded synchronous channels. The task/channel wrapper cost is part of the
handoff result; it is not a pure kernel context-switch measurement. No radio
is initialized and dynamic power management is off.

The separate `queue-api` binary compares std channels with native FreeRTOS
queues under matched Rust threads, priorities and raw-tick timing. Its
[completed 24-boot study](../../results/idf-queue-api-2026-10-23/RESULTS.md)
contains 24,000 checked round trips and 24,000 observer observations: boot
medians are 62.875 µs (std) and 19.9375 µs (native) at the nominal 16 MHz scale.
Both variants retain the same kernel; the measurement includes complete API
and instrumentation paths. Original firmware was restored, digest-verified and
reset afterward. [Independent result review](../../results/idf-queue-api-2026-10-23/RESULTS.md)
records the evidence and limits.

UART0 uses TX GPIO16 / RX GPIO17 through the board's CP2102N bridge, 115200 baud,
8 data bits, no parity, one stop bit. The trial requires CPU 160 MHz and 8 MB
flash in DIO mode at 80 MHz. Official SDK configuration selects the clocks;
the application reads the active CPU/MSPI/SPI0 registers before and after timing
and rejects unsupported sources/dividers, fractional-MHz flash values or changes.
It reports supported decoded clocks even when outside the trial baseline so the
host can retain raw diagnostics and reject CPU/flash clock mismatches. An image
header alone does not establish the active flash clock. Flash using IDF's matching bootloader and
partition table, not an ELF-only write at an assumed address.

Board: **ESP32-C6-DevKitC-1-N8**.
This benchmark retains the USB-to-UART bridge console described above; the
separate native USB-C port is available but unqualified by these runs.
See the [board inventory](../../../../docs/hardware.md) before choosing a new console.

## Build

`load-probe` is a separately configured finite offered-load qualification. Its
[build record](../../docs/builds/idf-load.md) specifies distinct disabled/enabled
statistics targets and notification-slot reservation. Follow the
[six-pulse contract](../../docs/experiments/idf-load-qualification.md); ordinary
queue defaults are insufficient. Qualification data cannot become a statistics
comparison or sustainable-capacity score.

The separate `accounting-probe` binary qualifies task-attributed wall counters
and a known live allocation. It requires an isolated statistics/U64 SDK build;
use the exact environment in the [build record](../../docs/builds/idf-accounting.md),
not the ordinary queue build commands below. The
[qualification contract](../../docs/experiments/idf-accounting-qualification.md)
keeps its blocked/busy/allocation controls separate from comparative scores.
It does not measure physical CPU busy time or maximum stack/heap use.

From this firmware directory, install the workspace linker once:

```sh
C6_WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
cargo +1.96.0 install ldproxy --version 0.3.4 --locked --root "$C6_WORKSPACE_ROOT/.tools"
```

Build with the pinned compiler, Rust source component and host Clang installed:

```sh
env RUSTC_BOOTSTRAP=1 \
  CARGO_WORKSPACE_DIR="$PWD" \
  LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
  PATH="$C6_WORKSPACE_ROOT/.tools/bin:$PATH" \
  cargo +1.96.0 build --release --locked
```

This checkout's ignored `.embuild` link points to the workspace's unmodified
IDF 5.5.6 cache at `.cache/espressif/idf-5.5.6/`. Build outputs remain local to
this firmware workspace. Old outputs with former absolute paths are retained
under ignored `.cache/os-evaluation/legacy-builds/`; start a fresh build here.
Use `cd projects/os-evaluation/firmware/esp-idf` from the repository root so
Cargo loads this directory's target configuration.

For the separate matched experiment, use that same environment with one of:

```sh
cargo +1.96.0 build --release --locked --bin queue-api --features queue-api
cargo +1.96.0 build --release --locked --bin queue-api --features native-queue
```

Each command produces a different image at the same target output path;
archive the selected ELF and matching SDK map before building the other variant.
The [queue build record](../../docs/builds/esp-idf-queue.md) identifies exact
measured inputs, feature selections and final image hashes.

`LIBCLANG_PATH` is this Mac's Homebrew path; substitute the host's actual
libclang directory elsewhere. The first build downloads ESP-IDF, its compiler,
and a Python environment into the ignored `.embuild` directory. ESP-IDF is a C
SDK dependency; the adapter and workload are authored in Rust.

ESP-IDF has no prebuilt Rust `std` sysroot, so Cargo rebuilds it with the unstable
`build-std` facility. `RUSTC_BOOTSTRAP=1` is a reproducibility exception scoped
to this firmware and compiler; the shared workload uses no unstable feature.
The upstream template normally uses nightly. Stable 1.96.0 is shared with the
other first-round firmware to reduce a comparison confound.

Rust 1.99.1 failed in upstream `std` because `libc::AT_FDCWD` is missing for
ESP-IDF. `esp-idf-sys` 0.38.1 failed because its stdio restoration glue accesses
newlib `_reent` fields absent from its generated bindings. Selected 0.37.2
builds without source patches; `CONFIG_ESP_CONSOLE_SECONDARY_NONE=y` addresses
its older console routing behavior. The
[initial results](../../results/initial-2026-10-22/README.md) describe the original
build baseline, and the
[clocked-handoff collection](../../results/clocked-handoff-2026-10-23/README.md)
identifies the protocol-2 artifacts. Building is separate from board evidence.

## Default protocol 2 measurement contract

The main task creates two `sync_channel::<u32>(1)` queues and a named responder
thread with an explicit 8 KiB stack. Rust's ESP-IDF thread integration uses
pthread wrappers backed by FreeRTOS tasks. The responder sends its actual
FreeRTOS priority before any workload; the initiator records its own priority.
The SDK defaults are main priority 1, worker priority 5, and a 100 Hz tick. Those
priority numbers belong to this runtime and are not a universal OS priority scale.

After two seconds of settling, the program performs eight compute warmups and
eight complete unmeasured message roundtrips. It then snapshots the active
clock registers, captures 64 compute samples, 64 timer-overhead samples and
1,000 message roundtrips, closes both channel endpoints, joins the responder,
and snapshots the clocks again. Only then does it format or write any JSON.
Fixed sample arrays occupy 18,048 bytes on this target; main's stack is 32 KiB.
This measurement storage and the worker stack affect memory-footprint comparisons.

Each compute sample brackets
`black_box(workload(black_box(seed), black_box(rounds)))` with
`esp_timer_get_time`, restarting at seed `0x12345678` for 4,096 rounds.
Timer-overhead samples make consecutive timer reads with `black_box(())`
between them and report `iterations: 0`, `checksum: 0`.

For message sample index `i`, the initiator starts its timer, sends `i`, waits
for the worker's `i XOR 0xa5a55a5a` reply, and stops its timer. Validation and
sample storage follow the second timer read. Each record reports `iterations: 1`
and the actual reply checksum. Exactly one request is in flight. The measured
path includes Rust channel bookkeeping, synchronization, wakeups and any task
switches that occur. It deliberately blocks as part of the workload. There are
no application writes, formatting, buffer growth or thread creation inside
any timed interval. Thread/channel creation, lazy startup and warmups precede
timing; shutdown and join follow it.

The selected Rust standard library implements these channels with a preallocated
array, cached thread context and retained waiter-vector capacity. The one-sender,
one-receiver request/reply pattern primes its blocking path during warmup;
no authored per-message allocation occurs. This is source inspection, not an
on-device allocator trace or a general allocation guarantee for every std
channel scheduling pattern. Native FreeRTOS queues are evaluated by the separate
`queue-api` binary, whose contract is below.
The worker remains blocked on its request queue during compute/observer sampling.

The timer returns integer microseconds; records report nanoseconds by multiplying
by 1,000. Quantization can produce zero overhead samples. No observer subtraction
is performed. The scheduler and enabled interrupts remain active.

Protocol 2 emits `hello` with explicit benchmark counts, task/channel metadata
and raw clock registers, all 1,128 samples, then `complete` with matching
`clocks_after`. Registers are `pcr_sysclk_conf`, `pcr_cpu_freq_conf`,
`pcr_mspi_clk_conf` and `spi0_clock`. The host independently decodes them,
validates counts/checksums/resolution and rejects incomplete/merged runs.
Protocol 1 pilot results remain historical and must not be pooled with this run.

Channel disconnection, bad replies, unsupported/changing clocks, timer conversion
or transport failure return errors. Both channel endpoints are closed and the
worker is joined explicitly outside timing on ordinary measurement failures.
There is no firmware deadline for a stuck task; host capture timeout/reset is
the recovery boundary. A failed run must remain visible. Panics abort and IDF
prints diagnostics before rebooting. A final flush can fail after completion
was queued; the collector judges received bytes rather than assuming success.

For this default binary, hardware-specific unsafe code is limited to generated timer/priority FFI and
four read-only MMIO accesses with documented invariants. There is no authored
C wrapper, interrupt handler, async task, custom allocator or unsafe trait impl.
See the [protocol 2 API and safety review](docs/api.md).

## Matched protocol 3 queue contract

Feature `queue-api` selects two `sync_channel::<u32>(1)` objects; feature
`native-queue` selects two capacity-one native FreeRTOS queues. Both use the
same Rust worker with observed main/worker priorities **1 / 5**, an **8 KiB**
worker stack, **32 KiB** main stack and **100 Hz** kernel tick. A preallocated
**48,000-byte** heap buffer stores 1,000 observer and 1,000 round-trip records;
these storage reservations are not a runtime memory high-water measurement.

Both variants retain raw 52-bit SYSTIMER UNIT0 ticks at a nominal **16 MHz**.
They qualify ordered reads, a blocked sleep and SDK WFI progress, and record
timer configuration alongside CPU/flash readback. Cross-checks share the crystal
and do not independently calibrate its frequency. Sequence-specific phase and
blocked-task state checks precede each timed request; native queues additionally
check empty occupancy, whereas std emptiness is inferred from the reviewed
single-inflight path. The worker's next-phase bookkeeping can occur before
main resumes and remains part of the measured interval.

After two seconds and eight checked warmups, each timed request/reply uses the
sample index and XOR checksum. A reserved stop value, worker join and backend
destruction occur after the final timestamp. Native queue handles and the
private SDK timer call have narrow reviewed Rust FFI boundaries. Success emits
protocol 3 after collection/cleanup; failures retain partial diagnostics where
possible and do not emit a successful completion.

The [experiment contract](../../docs/experiments/native-queue-comparison.md),
[API/safety review](docs/api.md) and
[integration review](docs/api.md) specify
admission bounds, lifecycle and provenance. This contract differs from the
historical protocol 2 experiment; do not pool their samples or directly rank
the new native path against its older Embassy result. Existing compatible
[HAL wrappers](../../docs/research/idf-native-api-options.md) remain an adoption
option, not a measured substitute for this narrow adapter.
