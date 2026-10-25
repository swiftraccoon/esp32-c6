# ESP-IDF native queue comparison build evidence

Implementation-agent checks completed **2026-10-23**. Both Rust feature variants
build, link, lint and document successfully. These are build/source/binary
checks; this agent performed **no flashing or on-board qualification**.
The [experiment](../experiments/native-queue-comparison.md) defines admission,
capture order and the decision this comparison can support.

## Inputs and behavior

Rust **1.96.0**, edition/style edition **2024**, target
`riscv32imac-esp-espidf`, sys **0.37.2**, embuild **0.33.5** and IDF **5.5.6**
at `880a203ae603d08a94d06aa01e0d6761958e5e9e` remain unchanged. Release settings
are opt-level 3, thin LTO, one codegen unit, panic abort and retained debug info.
The [previous build record](../../firmware/esp-idf/README.md) supplies unchanged native SDK/tool
provenance. No dependency, lockfile, SDK source, watchdog or SDK configuration
change was introduced. The existing `src/main.rs` remains byte-for-byte intact.

`queue-api` is a separate required-feature binary. Feature `queue-api` selects
two Rust `sync_channel::<u32>(1)` objects; feature `native-queue` includes
`queue-api` and selects two capacity-one native FreeRTOS queues instead. Both
use the same Rust worker, request values, checksum, phase markers, priorities,
readiness checks, timer and output path. Both use main priority **1**, pthread
priority **5**, main stack **32,768 bytes**, worker stack **8,192 bytes** and a
**100 Hz** kernel tick; actual priorities must match before collection.

Two seconds of settling and eight warmups precede 1,000 observer samples and
1,000 checked request/reply samples. Records occupy **48,000 bytes** in bounded
heap storage allocated before timing: DWARF confirms `Sample` size 24 and
alignment 8. No formatting, serial I/O or authored allocation occurs inside a
successful measured interval. Runtime channel internals remain part of the
chosen API; there was no allocator trace. Native queues allocate at setup.

Each timed request is its sample index; the reply is index XOR `0xa5a55a5a`.
Warmup requests are 0 through 7. A separate monotonically increasing worker
phase distinguishes warmup and timed trial readiness. A phase/state pair and
the actual task handle identify its blocked receive path. Native occupancy is
queried for both queues; std occupancy is inferred from the single-inflight
contract and is explicitly reported as **zero occupancy-query checks**. An
additional admission readiness check is separate from the 1,008 trial checks.
The higher-priority worker publishes its next phase and blocks before main
usually resumes; those matched instructions are part of the measured path.

Stop value `u32::MAX`, worker join and endpoint destruction follow the final
timestamp and validation. A separate full/empty probe destroys its own queues
before collection. `cleanup_pass` means that explicit probe teardown returned;
it does not establish an allocator leak trace. Actual main-backend destruction
is reported only after the real worker joins.

## Timing qualification implemented

Raw records retain 52-bit UNIT0 SYSTIMER timestamps and **16,000,000 Hz** nominal
scale, avoiding per-event microsecond truncation. The private C ABI call to
`esp_timer_impl_get_counter_reg()` is narrowly pinned to this SDK and linked in
both images. Its 18-byte IRAM wrapper at `0x408018ca` tail-jumps to ROM entry
`0x400003cc`. The vendor ROM ELF's corresponding body at `0x40020864` confirms
snapshot request, value-valid polling, a masked 20-bit high word and repeated
LO/HI/LO reads until the low words match. This is vendor artifact inspection,
not device ROM readback attestation.

Admission code performs 1,000 ordered reads; brackets a 20 ms Rust thread sleep
with raw/legacy reads; then calls existing SDK `esp_cpu_wait_for_intr()` until
at least three FreeRTOS ticks pass. Debugger status must be false before and
after. The linked SDK function at `0x40802c4a` is 34 bytes and includes literal
`wfi` at `0x40802c62`. Its debugger-free path reaches that instruction.
WFI progress is bounded by 10,000 calls and 500 ms raw elapsed; a stuck wait
itself still requires an interrupt or the host reset boundary. Raw elapsed must
agree with the software tick span within one 10 ms tick on either side.
Neither the tick cross-check nor the public microsecond API calibrates crystal
accuracy against an independent external oscillator.

Configuration snapshots retain the four CPU/MSPI/SPI0 registers plus
`SYSTIMER_CONF`, `PCR_SYSTIMER_CONF` and `PCR_SYSTIMER_FUNC_CLK_CONF`. The selected
SDK enables UNIT0's CPU-stall bit: **true**, not false. Before/after values must
match, with XTAL source, functional/APB clock enabled, UNIT0 enabled, CPU 160 MHz
and flash 80 MHz. No register is written. The checks reject backwards or
over-width raw timestamps, and timed round trips longer than one second.

## Commands and results

Run from `firmware/esp-idf` with the README environment:

```sh
env RUSTC_BOOTSTRAP=1 \
  LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
  PATH="$PWD/.tools/bin:$PATH" \
  cargo +1.96.0 build --release --locked --bin queue-api --features queue-api
```

Repeat with `--features native-queue` for the native image. The environment
retains the existing scoped `build-std` exception; no authored C or assembly
shim was needed. Both configurations were checked individually:

| Check | std control | Native queues |
| --- | --- | --- |
| Final `cargo +1.96.0 build --release --locked --bin queue-api --features …` | Passed, 4.30 s | Passed, 4.20 s |
| `cargo +1.96.0 clippy --release --locked --no-deps --bin queue-api --features …` | Passed, no warnings, 1.81 s | Passed, no warnings, 0.70 s |
| `cargo +1.96.0 doc --release --locked --no-deps --bin queue-api --features …` | Passed, no warnings, 2.11 s | Passed, no warnings, 1.26 s |
| `cargo +1.96.0 fmt --all --check` | Passed for crate | Same source |
| Linked main / worker application frame | 768 / 192 bytes | 768 / 192 bytes |
| Timer/private ABI, native path, WFI and mitigated copy symbol inspection | Passed | Passed |
| Physical sequence, readiness, timer, WFI, UART and clock qualification | Not run by this agent | Not run by this agent |

Frames are the inspected function prologues, not a complete static stack bound
or measured high-water mark. Build-only checks cannot prove blocking behavior,
MMIO values, runtime memory margin or timer continuity. No tests were authored
merely to repeat the implementation. The collector's meaningful parser and
negative-case tests are recorded separately.

The first compiler attempt found module re-export visibility errors; scoped
crate visibility fixed them. Final checks are against the hashes below. An
exploratory `cargo rustc … -C link-arg=-Wl,-Map=…` did not produce the requested
map because the later SDK map argument takes precedence. Final ordinary builds
were repeated; their actual SDK-generated `libespidf.map` was copied immediately
after each successful link. These maps contain each image's Rust symbols.

## Exact artifacts and failure evidence

Stable scratch copies are under ignored
`firmware/esp-idf/.cache/queue-api/{std,native}.{elf,map}`. The operator must
archive those exact images/maps into the experiment's permanent result tree
before another build replaces target outputs.

| Artifact | SHA-256 |
| --- | --- |
| std ELF | `0282b814660461f7c88d3eb64c7d701e620e418d53be7d53b5eb96834a82d32b` |
| native ELF | `fd6501102658572f12850512116401eaf57f6c330becd288ed8c825117e6fac8` |
| std map | `01c4ec0e88772061e00c16d4ec68e4028335dabf4d0f5783be6b2fb6ceb544a0` |
| native map | `ba7d03dd11ca57879b47e5aa1551040ce01543606f9cce67a7a9aaf9d88c8a76` |
| Vendor `esp32c6_rev0_rom.elf`, tool package 20260528 | `788e1d38724aeb8fd974fa10c4a7b089c02627d35342ce84b9e0b12b239f3551` |

The matching sys output is
`target/riscv32imac-esp-espidf/release/build/esp-idf-sys-2dd3efe9f8c63c2b/out`
relative to `firmware/esp-idf`. Its unchanged generated SDK configuration hashes
to `5ee0be1128539831db16b63d7008985e16849b216bd4a278076e294a51fe3758`;
bootloader `f296275389320ab2c9accdb44813442c8994724a18e90d646189950627ce562b`;
partition table `7f00b6c042a89b15b0cac534f82ed988caf29278ff5700b0c511eb1b5bb7c820`.
Use this DIO80 bootloader/partition set, not the older DIO40 pilot bootloader.

[Input hashes](../../firmware/esp-idf/artifacts/queue-api/inputs.sha256),
[std adapter](../../firmware/esp-idf/artifacts/queue-api/std-adapter.txt),
[native adapter](../../firmware/esp-idf/artifacts/queue-api/native-adapter.txt),
[std worker](../../firmware/esp-idf/artifacts/queue-api/std-worker.txt),
[native worker](../../firmware/esp-idf/artifacts/queue-api/native-worker.txt),
[std SDK paths](../../firmware/esp-idf/artifacts/queue-api/std-sdk-paths.txt),
[native SDK paths](../../firmware/esp-idf/artifacts/queue-api/native-sdk-paths.txt),
[ROM entry](../../firmware/esp-idf/artifacts/queue-api/rom-timer-entry.txt) and
[ROM body](../../firmware/esp-idf/artifacts/queue-api/rom-timer-body.txt)
retain bounded binary inspection evidence. IDF's existing mitigated `memcpy` /
`memmove` text remains linked; see the [errata ledger](../../../../docs/research/rev02-errata-workarounds.md).

Successful output is protocol 3 only, after capture and teardown. Ordinary
measurement or cleanup errors print boot/backend context and all captured raw
samples as explicitly labelled `queue partial` diagnostics, then return an error
without a successful complete record. A panic, stuck kernel or transport failure
can prevent that dump; host timeout/reset retains the observed bytes. Failed
attempts must not become numeric passes. A nonce identifies one startup's
records; its radio-off RNG source is not assumed cryptographically strong and
does not itself prove a host-requested reset.

The [API/safety review](../../firmware/esp-idf/docs/api.md) accounts for all 54 active
guidelines. No OS ranking, isolated wakeup/context-switch cost, radio, power,
sleep, general protection or long-duration claim follows from these builds.
