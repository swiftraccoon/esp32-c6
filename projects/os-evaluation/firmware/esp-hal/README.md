# ESP32-C6 Rust measurement adapters

Two applications run the shared [`bench-core`](../../crates/bench-core/src/lib.rs)
workload through one [measurement module](src/measurements.rs):

| Binary | Execution context | Recorded workloads per boot |
| --- | --- | --- |
| `bare-metal` | Synchronous main on the high-performance core | 64 compute, 64 timer overhead |
| `embassy` | Main initiator and one independent responder on one cooperative Embassy executor, hosted by esp-rtos | Same baseline plus 1000 request/reply round trips |

The message result measures this complete channel/executor path. Embassy tasks
share one OS thread; this is not an isolated OS thread-switch measurement.
FreeRTOS uses OS threads and a different channel implementation, which must be
disclosed in comparisons. Compute does not yield to other cooperative tasks.
These adapters do not establish radio operation, power consumption, peripheral
coverage, deadline guarantees, or an overall OS ranking.

Protocol **2** adds actual clock-register snapshots and explicit benchmark
counts. Archived protocol-1 captures remain separate evidence; this source does
not retroactively change or validate their final runtime flash frequency.

## Build

The pinned toolchain is stable Rust **1.96.0**, edition/style edition **2024**,
target **riscv32imac-unknown-none-elf**. Primary dependencies require Rust 1.95;
that dependency-declared floor is recorded without claiming a 1.95 build.
No vendor Rust compiler, nightly, authored C/assembly, `std`, or `alloc` is needed.
Upstream dependencies retain their own low-level implementation languages.

Run from this directory so Cargo reads `.cargo/config.toml`:

```text
cargo +1.96.0 build --locked --release --bin bare-metal
cargo +1.96.0 build --locked --release --features embassy --bin embassy
cargo +1.96.0 fmt --package c6-runtime-bench -- --check
cargo +1.96.0 clippy --locked --release --bin bare-metal
cargo +1.96.0 clippy --locked --release --features embassy --bin embassy
cargo +1.96.0 doc --locked --release --no-deps --bin bare-metal
cargo +1.96.0 doc --locked --release --no-deps --features embassy --bin embassy
```

The separate ELF paths are
`target/riscv32imac-unknown-none-elf/release/bare-metal` and
`target/riscv32imac-unknown-none-elf/release/embassy`. Build bare metal without
`embassy` so it does not inherit the runtime dependency graph. Embassy uses the
directly pinned `embassy-sync = 0.8.0`, matching the version already resolved by
esp-hal/esp-rtos. Release is opt-level 3, thin LTO, one codegen unit,
incremental=false, debug=2, panic=abort. Preserve ELF/source/lockfile hashes,
compiler identity and flashing commands with each capture. ELF filesystem size
includes debug information and does not measure flash or runtime RAM usage.

## Hardware and clock readback

Board: **ESP32-C6-DevKitC-1-N8**. These binaries
retain the USB-to-UART bridge route below; the separate native USB-C port,
GPIO8 RGB LED and BOOT button are not exercised by these benchmarks.
See the [board inventory](../../../../docs/hardware.md) for future fixture choices.

Only **UART0 TX GPIO16 / RX GPIO17**, connected to the CP2102N USB-to-UART bridge,
are assigned: **115200 baud, 8 data bits, no parity, one stop bit**, no flow
control. No radio, credentials, flash storage, LED, or unknown GPIO is configured.
The LP core remains unused. Each boot waits two seconds before warmup: HAL delay
for bare metal, Embassy timer after esp-rtos starts with TIMG0 timer0 and
FROM_CPU_INTR0 for Embassy. The startup wait is outside every timed sample.

HAL configuration requests **160 MHz CPU**. The controlled image settings for
the new comparison are **DIO, 80 MHz, 8 MB physical flash**. The flashing owner
must preserve the actual image/flash commands; a header or bootloader log alone
does not establish the application's final flash clock.

[`clocks.rs`](src/clocks.rs) reads the following through public, safe upstream
PAC register access, after all warmups and again after all measured samples:

| Raw field | Address | Relevant bits |
| --- | --- | --- |
| `pcr_sysclk_conf` | `0x60096110` | `SOC_CLK_SEL[17:16]`, `HS_DIV_NUM[15:8]` |
| `pcr_cpu_freq_conf` | `0x60096118` | `CPU_HS_DIV_NUM[15:8]`, `CPU_HS_120M_FORCE[16]` |
| `pcr_mspi_clk_conf` | `0x6009601c` | `MSPI_FAST_HS_DIV_NUM[15:8]` |
| `spi0_clock` | `0x60002014` | `CLK_EQU_SYSCLK[31]`, `CLKCNT_N[23:16]` |

The supported decoder requires SPLL source 1, the fixed SYSCLK HS divider field
2, CPU HS field 0/1/3, and MSPI HS field 3/4/5. SPLL is 480 MHz. CPU frequency is
`480 / (SYSCLK_HS + 1) / (CPU_HS + 1)`, except force-120 mode at CPU_HS=0.
MSPI source frequency is `480 / (MSPI_HS + 1)`. SPI0's downstream divider is 1
when bit 31 is set, otherwise `CLKCNT_N + 1`; flash bus frequency is the MSPI
source divided by this divider. Fractional integer-MHz results are rejected
instead of truncated. Unsupported fields stop the run without completion.

Supported clocks are reported as observed, even if they differ from the intended
comparison settings. The host independently decodes the raw registers and
enforces **160 MHz CPU, 80 MHz MSPI source, 80 MHz flash, divider 1**. Identical
before/after snapshots are also required. Readback establishes configuration,
not oscillator accuracy or an electrical frequency measurement. Sequential reads
and equal endpoints cannot detect a temporary change that returns to the same
configuration. No dynamic clock scaling is enabled and no authored code writes
these registers. The HAL's 160 MHz preset configures MSPI HS field 5; SPI0 must
still be observed to establish its downstream flash divider.

## Workloads and observation

Both adapters perform eight unrecorded compute warmups using `integer-mix-v1`,
seed `0x12345678`, 4096 rounds, expected checksum `0xa3944d55`. Embassy also
performs eight unrecorded request/reply warmups. Then both take the first clock
snapshot, record 64 compute and 64 timer-overhead samples, and Embassy records
1000 sequential message samples before the final snapshot.

Compute inputs/results pass through `core::hint::black_box`; the shared workload
is `inline(never)`. Each timer-overhead interval has `black_box(())` between its
timer observations. Overhead is preserved separately and never subtracted.

Embassy's static request and reply channels each hold one `u32`, using
`CriticalSectionRawMutex`. The main initiator sends index `0..999` and waits for
the independent responder to return `index XOR 0xa5a55a5a`. Warmup requests use
`0..7`; the timed sequence starts again at 0. The responder only receives and
computes/sends the reply; after each reply, including the last sample, it parks
waiting for another request. This keeps task teardown outside every interval.
Each interval begins
before request send and ends after reply receive. Reply validation occurs
immediately after the elapsed read, outside the interval, and precedes the next
sample. No other request is in flight. The timer includes channel operations,
wakes, executor polling and any runtime interrupt interference in both directions.
There is no authored interrupt handler, priority override, or extra OS thread.

`esp_hal::time::Instant` observes the system timer with **1 microsecond
resolution**. The wire value `elapsed_ns` multiplies integer microseconds by
1000, adding no precision. Zero elapsed observations are valid quantization
results. The timer does not measure sleep time; no sleep mode is configured
during measurement. Runs are far shorter than its seven-year wrap interval.

Raw arrays are fixed-size: 2048 sample bytes for bare metal and 18048 for
Embassy on this target, excluding runtime/channel/stack overhead. Embassy's
main future stores its cross-await data in the upstream statically allocated
task pool; this is not a claim that the sample array is a separate task stack.
The wire `message.sample_buffer_bytes` reports sample storage only. There is no
heap allocator, formatting, UART transfer, host request, or serial logging in a
timed region. All samples and both snapshots exist before the first JSON line.
UART output/flush may block afterward, without changing recorded durations.

## JSON-lines protocol 2

Every output event has `protocol: 2`. The host must preserve boot text and all raw
bytes separately from parsed events. One `hello`, a declared number of `sample`
events and one `complete` constitute a candidate complete run.

| Event | Fields |
| --- | --- |
| `hello` | Existing `platform`, requested `cpu_mhz:160`, legacy `sample_count:64`, `seed`, `rounds`, `workload`, `warmup_iterations:8`, `timer_resolution_ns:1000`; new `benchmarks` map and `clocks` object |
| `sample` | `benchmark`, independent zero-based `index`, `iterations`, raw `elapsed_ns`, `checksum` |
| `complete` | Actual `samples` total, `compute_samples`, `timer_overhead_samples`, optional `message_roundtrip_samples`, and `clocks_after` |

The benchmark map is `{"compute":64,"timer-overhead":64}` for platform
`bare-metal`; `embassy` additionally declares `"message-roundtrip":1000`.
Embassy's hello also includes `message` with transport
`embassy-sync-0.8.0-channel`, `queue_capacity:1`, `warmup_iterations:8`, and
`sample_buffer_bytes:18048`. Bare metal emits neither message metadata nor a
message count or sample. There is no invented zero-valued handoff result.

Clock objects contain `source:"register-readback"`, decoded `cpu_mhz`,
`flash_source_mhz`, `flash_mhz`, `flash_divider`, and a `registers` object with
the four raw fields above. `clocks_after` has the identical shape.

Compute samples use `iterations:4096` and the shared checksum; timer overhead
uses `iterations:0` and `checksum:0`; message samples use `iterations:1` and
the expected sequence reply. Successful totals are 128 bare-metal samples or
1128 Embassy samples. Legacy `sample_count` does not override the per-benchmark
map. Missing/duplicated/out-of-range samples, malformed/unknown events,
incorrect checksums, clock mismatches, incomplete captures and rebooted runs
must not be scored as passes.

## Failure, concurrency, and cancellation

Clock decoding/stability, spawn, reply, formatting and UART errors stop the
run. Private `RunError` implements `Debug`, `Display` and `core::error::Error`,
retaining a formatting source and reply index/expected/received details.
Entry points halt on error; they do not serialize a success afterward. Panics
spin without unwinding, intentional reboot or a completion record. Missing
replies wait indefinitely; the external collector must time out and preserve
the incomplete capture. Runtime/hardware watchdogs or faults still require
actual on-board checks.

The two channel states are shared only through safe upstream synchronization.
Critical sections occur within channel operations and never span an await.
Interrupts remain enabled outside those brief upstream sections. Neither
application cancels the main future. If it were canceled during a message
exchange, a message could remain in flight and the responder could wait
indefinitely; that abandoned run has no successful completion or recovery.
The UART borrow remains inside the main future and is not shared with the
responder. No allocation, blocking lock or interrupt-disabled critical section
is held across an await.

After successful flush, bare metal spins and Embassy awaits a permanently
pending future, permitting runtime idle handling. These different post-run
states are outside timing and do not constitute an idle-power comparison.

## Evidence and primary sources

The build instructions above and [all 54 API review decisions](docs/api.md)
describe this implementation. The
[initial results](../../results/initial-2026-10-22/README.md) preserve protocol-1
observations; [clocked-handoff results](../../results/clocked-handoff-2026-10-23/README.md)
record protocol-2 observations and their artifact provenance.
Compilation does not establish an on-board pass; raw hardware captures belong
to their separate results directory.

- [esp-hal 1.2.0 C6 clock implementation](https://docs.rs/crate/esp-hal/1.2.0/source/src/soc/esp32c6/clocks.rs)
- [esp32c6 0.24.0 PCR source](https://docs.rs/crate/esp32c6/0.24.0/source/src/pcr.rs) and [SPI0 clock fields](https://docs.rs/crate/esp32c6/0.24.0/source/src/spi0/clock.rs)
- [ESP-IDF clock-tree LL at pinned `880a203`](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/esp32c6/include/hal/clk_tree_ll.h)
- [ESP-IDF SPI-memory register specification at the same commit](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/register/soc/spi_mem_reg.h)
- [Embassy sync 0.8.0 bounded channel](https://docs.rs/embassy-sync/0.8.0/embassy_sync/channel/struct.Channel.html) and [critical-section mutex](https://docs.rs/embassy-sync/0.8.0/embassy_sync/blocking_mutex/raw/struct.CriticalSectionRawMutex.html)
- [esp-rtos 0.4.0 executor source](https://docs.rs/crate/esp-rtos/0.4.0/source/src/embassy/mod.rs)
- [esp-hal-procmacros 0.23.0 main expansion](https://docs.rs/crate/esp-hal-procmacros/0.23.0/source/src/unified_main.rs) and [Embassy executor macros 0.8.0 task storage](https://docs.rs/crate/embassy-executor-macros/0.8.0/source/src/macros/task.rs)
