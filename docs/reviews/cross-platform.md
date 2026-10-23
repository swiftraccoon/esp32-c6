# Cross-platform measurement review

Reviewed 2026-10-22 by the HAL implementation agent, reading the independent
ESP-IDF adapter without editing its files. Scope is the initial integer-work
and timer-observation trials, not a certification of either OS. See
[HAL evidence](../builds/esp-hal.md), [ESP-IDF evidence](../builds/esp-idf.md),
and [the measurement plan](../benchmark-plan.md). On-board evidence remains
the hardware owner's responsibility.

**Post-capture correction:** equal 40 MHz runtime flash clocks were not
established. All ten HAL boot logs report 80 MHz despite nominal 40 MHz image
headers. The clock erratum below supersedes any earlier inference that header
agreement controlled the runtime flash clock. Raw results and provenance remain
unchanged historical records; their `flash_mhz: 40` is not a validated HAL
runtime measurement.

## Findings and corrections

| Finding | Importance | Resolution/evidence |
| --- | --- | --- |
| Initial ESP-IDF source emitted `hello` and flushed stdout before warmup/timing. A library flush does not establish physical UART quiescence across every driver/buffering configuration. | Could contaminate early samples with UART drain or interrupt activity. | ESP-IDF owner moved stdout locking and all JSON output after both complete sample arrays. Current source has no serial formatting or output during timing. |
| HAL calibration included `black_box(())` between its observations; initial ESP-IDF calibration did not. | Small observer-contract mismatch; an empty compiler barrier usually adds no machine instruction, but the code should still use the same boundary. | ESP-IDF owner added the same boundary. Both calibration samples have zero workload iterations and zero checksum. |
| HAL waited two seconds before warmup; initial ESP-IDF source began immediately. | Boot phase and residual startup activity differed. | ESP-IDF owner added a two-second pre-measurement `std::thread::sleep`. These waits are setup, not a boot-time result. |
| Older generated ESP-IDF configuration had 80 MHz flash, while the intended common configuration was 40 MHz. | A different memory interface can change code fetch performance. | New default configuration explicitly selects DIO/40 MHz/8 MB. The current inspected generated configuration has those settings. The older build directory must not be mistaken for the final build's provenance. Archive the final generated sdkconfig with its ELF. |
| All ten HAL boot logs report 80 MHz while patched boot/app headers nominally request 40 MHz. | Runtime flash clock was not controlled equally across stacks. | HAL runtime flash frequency is unverified; the 80 MHz boot report and 80 MHz HAL MSPI source are separate evidence. ESP-IDF SDK configuration is 40 MHz. See the erratum. |

The first three corrections were confirmed in the current ESP-IDF source.
Build and actual boot verification of that source belongs to its independent
owner. A firmware build completing before these corrections is not evidence
for the corrected source.

## Runtime flash-clock erratum

The archived HAL raw transcripts, for example
[bare-metal boot 1](../../results/initial-2026-10-22/runs/01-bare-metal/serial.raw),
identify their prebuilt second-stage loader as ESP-IDF
`v6.1-beta1-497-g14f663f003e` and report `SPI Speed : 80MHz` in all ten HAL boots.
Boot/app header bytes `02/30` describe nominal DIO/40 MHz/8 MB image settings;
they do not establish the final runtime bus clock.

Published espflash 4.6.0 source at commit
`c1fb39939c1b5880b2aec7f43063f19689d5b00a` embeds its C6 bootloader and patches
header settings plus the checksum, rather than recompiling the bootloader
configuration. Its [bootloader manifest](https://github.com/esp-rs/espflash/blob/c1fb39939c1b5880b2aec7f43063f19689d5b00a/espflash/resources/bootloaders/manifest.yaml)
uses SDK `release/v6.1` and a C6 flash-size fragment. The shipped unpatched C6
bootloader has SHA-256
`bfe7a8a539d06d8da984d42ce02bf985f97d363f03b1513adce8277da11532ee`.
The manifest does not provide the complete generated bootloader sdkconfig.

The [C6 loader hardware initialization at the reported SDK revision](https://github.com/espressif/esp-idf/blob/14f663f003e/components/bootloader_support/src/esp32c6/bootloader_esp32c6.c#L78)
contains a compile-time 80 MHz branch that sets the MSPI source divider and
SPI0/SPI1 flash divisors. Its [flash initialization](https://github.com/espressif/esp-idf/blob/14f663f003e/components/bootloader_support/bootloader_flash/src/bootloader_flash_config_esp32c6.c#L205)
can overwrite the in-memory header speed with the compile-time 80 MHz choice
before printing the speed. This explains why changing image header bytes
alone cannot validate runtime control. This source-level explanation is an
inference from the reported revision and preserved log, not a register capture.

After boot, [esp-hal 1.2.0 C6 clock initialization](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/soc/esp32c6/clocks.rs#L57)
sets an 80 MHz MSPI source for the selected 160 MHz CPU preset, with a
480 MHz PLL divided by six. A source clock is not itself the final SPI flash
bus clock: downstream divisors and their final state matter. No register
snapshot or physical clock measurement was captured after HAL initialization.

The conservative record is therefore: **HAL boot reports 80 MHz; HAL MSPI
source configuration is 80 MHz; final runtime flash bus frequency is
unverified.** ESP-IDF's separately built SDK is configured for 40 MHz, also
without an external clock measurement. The pilot does not establish equal
runtime flash clocks and cannot support a controlled cross-stack speed claim.
No elapsed samples, counts, hashes, or immutable captured JSON are corrected
by guessing another frequency.

Before a new comparison, record the actual source/divider configuration after
all initialization and verify the resulting flash clock, or use a deliberately
built and pinned loader/HAL configuration with validated clock behavior.
Any changed firmware/configuration requires a fresh versioned capture set;
it must not replace this pilot's evidence.

## Workload and compiler assessment

Both adapters call the same `bench_core::workload` with seed `0x12345678` and
4096 rounds, retain its checksum, and place runtime inputs/output through
`black_box`. The shared function is `inline(never)`. There are eight unrecorded
warmups, 64 retained compute samples, and 64 retained timer-overhead samples.
There is no serial output, heap allocation, formatting, or host interaction
inside a measured region. Checksum validation occurs on the host afterward.

The selected Rust compiler is 1.96.0, with opt-level 3, thin LTO, one codegen
unit, and panic=abort. The HAL target is `riscv32imac-unknown-none-elf`; ESP-IDF
uses `riscv32imac-esp-espidf` and builds `std` through its documented scoped
bootstrap configuration. These are different target/runtime environments even
with the same compiler and workload. ESP-IDF SDK C code uses its performance
configuration; it is not generated by the Rust compiler. Do not attribute all
elapsed differences to the scheduler alone.

Read-only `llvm-nm`, `llvm-objdump`, `llvm-readobj`, and direct byte-range hashing
of all three final release ELFs establish identical **54-byte function bodies**
(18 instructions, including compressed instructions). The loop, branch
condition, multiply, shifts, and return are present. Only symbol naming and
placement differ. This checks the actual workload machine code, beyond common
source or compiler settings.

| Build | Workload address | ELF section | Section file offset | Function file offset |
| --- | --- | --- | --- | --- |
| Bare metal | `0x42011658` | `.text` at `0x42010020` | `0x7020` | `0x8658` = 34392 |
| Embassy | `0x42010358` | `.text` at `0x42010020` | `0x9020` | `0x9358` = 37720 |
| ESP-IDF | `0x42000ca8` | `.flash.text` at `0x42000020` | `0xe020` | `0xeca8` = 60584 |

In all three files the exact 54-byte range has SHA-256
`b917090559ff1edf22ad5ac21975f1a998e5594b9ad23db9344d5c71c2b3af49`.
Function file offset is section file offset plus function address minus
section address. This direct range check neither writes nor transforms an ELF.
It does not establish that timer adapters, caches, interrupts, or complete
application machine code are identical.

Tools were the installed Rust 1.96.0 `llvm-tools` component, LLVM 22.1.2, under
`/Users/builduser/.rustup/toolchains/1.96.0-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/`,
plus macOS `dd` and `shasum`. Commands below run from the repository root;
resolve the `llvm-*` commands from that directory:

```text
llvm-nm --print-size firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal
llvm-nm --print-size firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy
llvm-nm --print-size firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline
llvm-objdump --disassemble-symbols=_ZN10bench_core8workload17h2bec9f0ad370606aE firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy
llvm-objdump --disassemble-symbols=_RNvCs3RkLThg6PCt_10bench_core8workload firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline
llvm-readobj --sections firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline
dd if=firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/bare-metal bs=1 skip=34392 count=54 2>/dev/null | shasum -a 256
dd if=firmware/esp-hal/target/riscv32imac-unknown-none-elf/release/embassy bs=1 skip=37720 count=54 2>/dev/null | shasum -a 256
dd if=firmware/esp-idf/target/riscv32imac-esp-espidf/release/esp-idf-baseline bs=1 skip=60584 count=54 2>/dev/null | shasum -a 256
```

The reviewed full ELF hashes are:

| Build | SHA-256 |
| --- | --- |
| Bare metal | `c2c46633ab3d7b9fb1b6ea9541da8746200f27c0c7f0203bdaab53bb8bfe9175` |
| Embassy | `c6ff29be0b56004b5b9cfe43c2f5e55a30415324abda94fc97a19ae84214894c` |
| ESP-IDF | `793f3cf07d2e23b3623fbe6657d077d078c539f960fe2db97bea3dea055fe1a9` |

An earlier ESP-IDF ELF, SHA-256
`c21775a4d52a701f57bf9993438cb0e78f32077556739cec22aa6c698fdde23d`,
was also inspected. Its later source-comment-only rebuild changed debug/ELF
bytes; the final ELF above was rechecked and has the same workload address,
size, file offset, and 54-byte workload hash.

Final ESP-IDF uses unmodified `esp-idf-sys` 0.37.2 with SDK 5.5.6. Its inspected
generated configuration is
`firmware/esp-idf/target/riscv32imac-esp-espidf/release/build/esp-idf-sys-2dd3efe9f8c63c2b/out/sdkconfig`,
SHA-256 `8a832482dd631eb325cf5f897bf5fa4420cb2e743284d82c55e9fd0c7725b061`.
It confirms DIO, 40 MHz flash, 8 MB flash size, 160 MHz CPU, 100 Hz FreeRTOS,
UART0 GPIO16/17 at 115200 baud, disabled power management, and disabled normal
SDK/bootloader logging. The source reviewed after corrections has SHA-256
`eeee677889598547e4038598f71cc58a720b4f1286bf3a7d05a3be32682e5077`.

## Timer interpretation

The HAL adapter calls `esp_hal::time::Instant`, whose selected ESP32-C6
implementation reads system-timer unit 0 and converts ticks to integer
microseconds. ESP-IDF calls `esp_timer_get_time`, which its pinned source
aliases to its system-timer implementation. Both interfaces expose one
microsecond resolution. ESP-IDF's [versioned timer documentation](https://docs.espressif.com/projects/esp-idf/en/v5.5.6/esp32c6/api-reference/system/esp_timer.html#obtaining-current-time)
describes that API's startup epoch and use from tasks. Multiplication by 1000
is only a unit conversion, not nanosecond precision.

The API paths and their cost differ: IDF crosses a generated C binding into
its timer implementation; the HAL can inline register-access/conversion code.
The elapsed observation includes observer effects from those paths. Separate
back-to-back calibration samples characterize that cost only at the available
quantization. Zero calibration values are legitimate. No calibration value
is subtracted, and an average overhead must not be interpreted as an exact
per-sample correction. Both sets retain all observations/outliers.

All timed work runs on the high-performance core at configured 160 MHz.
Dynamic power management is disabled in ESP-IDF; the HAL code does not request
sleep during measurement. The LP core is unused. esp-rtos and FreeRTOS are
configured for 100 Hz scheduler ticks and leave interrupts enabled. A CPU
sample can contain interrupt/scheduler activity; the bare-metal baseline has
no corresponding RTOS tick. That is a property of each configured stack, not
an isolated scheduler-latency measurement. A two-second setup sleep says
nothing about sleep accuracy or wake latency.

## Protocol, memory, and reporting

The current protocol implementations agree on workload identifier, counts,
units, seed, rounds, warmup, checksums, per-benchmark indices, and completion.
The host collector checks both complete sample sets and rejects missing,
duplicated, malformed, reboot-interrupted, wrong-configuration, and incorrect
checksum runs. The collector's complete-event validation derives completeness
from retained samples rather than trusting optional count text. Successful
completion demonstrates this run's work/output, not extended post-run
stability. The capture ends on completion, so it is not a soak test.

Both adapters use bounded stack sample storage; ESP-IDF also has SDK/std heap
and tasks outside the measured workload. Compare actual ELF allocated sections
without treating their sum or ELF file bytes as runtime RAM use or image size.
The actual nonzero address reservations requiring separate interpretation are:

| Reservation | Bare metal | Embassy | ESP-IDF | Interpretation |
| --- | ---: | ---: | ---: | --- |
| `.stack` | 442844 B | 438868 B | Absent | Remaining HAL RAM reserved for stack; neither stack use nor a static application-data requirement. |
| `.text_gap` | 53396 B | 51092 B | Absent | `SHT_NOBITS`, flash virtual-address alignment; no payload bytes in the ELF. Image padding must be checked in the actual image. |
| `.flash_rodata_dummy` | Absent | Absent | 262144 B | `SHT_NOBITS`, allocated and marked writable at `0x42000020`; overlaps `.flash.text` starting at that same address. This mapping reservation is not physical RAM or extra flash payload. |
| `.rtc_reserved` | Absent | Absent | 24 B | Actual RTC address reservation at `0x50003fe8`; report separately as reserved RTC space, not measured active data. |
| `.iram0.text_end` | Absent | Absent | 2 B | IRAM alignment at `0x4080b236`, not executable instructions. |
| `.iram0.data` | Absent | Absent | 8 B | `SHT_NOBITS` IRAM data/alignment section; retain separately rather than calling it flash payload. |

Zero-sized sections make no byte contribution. Non-allocated debug, symbol,
unwind, and metadata sections also do not become application flash/RAM use
merely because they occupy the ELF file. The collector retains a complete
section inventory; raw totals by generic ELF section kind are descriptive only.
For example, its generic kind classification may place the 262144-byte
`.flash_rodata_dummy` in uninitialized data. That total must not be relabeled
physical RAM, and its overlap must not be added a second time to flash code.

RAM-resident code, initialized data, zeroed data, task stacks, heap use,
reserved address gaps, image padding, bootloader, and partition metadata need
distinct accounting. Nonzero IRAM data/alignment and RTC reservations should
remain visible with their stated meaning. Excluding reservations from an
application-data total does not make those address ranges freely available.
No dynamic heap/stack high-water result exists from static inspection.

The configured waits and post-completion behavior differ in implementation:
HAL bare metal busy-waits initially and spins after output, Embassy awaits
timers/pending futures, and ESP-IDF sleeps then returns through its runtime.
These actions are outside timing and must not become a cross-platform idle
power or boot-performance comparison. Panic/reboot policies likewise differ
and must remain explicit failure-policy metadata.

The present review supports these adapters as an initial compute/calibration
experiment after corrected builds and on-board validation. It provides no
evidence about Wi-Fi/BLE/802.15.4, task handoff, queues, interrupt latency,
allocation, peripheral behavior, power, determinism, or complete OS suitability.
