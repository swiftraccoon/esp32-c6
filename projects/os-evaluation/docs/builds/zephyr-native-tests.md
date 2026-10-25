# Zephyr native latency-suite build

**Review correction, interpretation revision 2 (2026-10-23):** this build's
required frequency print was compiled out, so none of the three subsequent
boots passes its frozen contract. All also report a ROM SHA comparison warning.
The [source audit](../research/zephyr-native-audit.md) corrects the observer
policy and header-frequency interpretation below. The original artifacts and
capture inputs remain unchanged; see [results](../../results/zephyr-native-tests-2026-10-23/RESULTS.md).

Build evidence recorded **2026-10-23**. The unmodified upstream
`tests/benchmarks/latency_measure` suite builds for
`esp32c6_devkitc/esp32c6/hpcore`. This establishes a build and image-generation
path, not an on-board pass or C6 Rust integration. The
[qualification brief](../experiments/zephyr-native-tests.md) defines the separate
physical collection and its limits.

## Pinned inputs and reuse

| Input | Selected version / identity |
| --- | --- |
| Zephyr release | **4.4.0**, [`684c9e8f32e4373a21098559f748f06915f950c9`](https://github.com/zephyrproject-rtos/zephyr/tree/684c9e8f32e4373a21098559f748f06915f950c9) |
| Espressif HAL module | [`b7953b8019361d09e613f7011d2ccc41b984d087`](https://github.com/zephyrproject-rtos/hal_espressif/tree/b7953b8019361d09e613f7011d2ccc41b984d087) |
| Zephyr SDK / compiler | **1.0.1**, macOS arm64; `riscv64-zephyr-elf-gcc` **14.3.0**, target ABI `ilp32`, ISA `rv32imac_zicsr_zifencei` |
| Host build tools | Python **3.14.8**, west **1.5.0**, CMake **4.4.3**, Ninja **1.13.2**, dtc **1.8.1** |
| Final image converter | **esptool 5.6.0**, selected through the isolated Python environment's `PATH` |
| Final configuration | Default suite `prj.conf` plus C6 board defaults; no extra configuration or devicetree overlay |

The [reuse inventory](../../../../docs/research/benchmark-reuse.md) identifies this existing
suite before implementation. The suite, board and inspected timer sources carry
**Apache-2.0** notices; preserve those notices and root licenses. No new C,
C++ or assembly implementation is authored. SDK/runtime dependencies retain
their individual licenses; this build is not a full distribution-license audit
or qualification of radio blobs. Rust integration remains a separate gate.

The SDK minimal and RISC-V archives were checked against release hashes:

- Minimal archive SHA-256:
  `867063901f39528a6175a80ebc20367bd6cb440593e7e2650eda30392f1f6b65`.
- RISC-V archive SHA-256:
  `bc9026ee24c34d79f89bb9f1eb7f86a60232ed8eb1b85d301178482f6544cee7`.

The workspace is `.cache/zephyr-qualification/workspace`; only the required
`hal_espressif` module was selected for this native radio-off build. Pinned
`west.yml`, `west-optional.yml`, resolved module records and Python package
versions are retained with the handoff. This is not an assertion that all
projects listed in west's broader manifest were fetched or linked.

## Build selection and recorded setup failures

The cached final build is
`.cache/zephyr-qualification/build-latency-qualified`, with permanent handoff
copies under `.cache/zephyr-qualification/artifacts`. A reproduction command
matching its source, module and tool selections, from the evaluation root, is:

```sh
cd .cache/zephyr-qualification/workspace
env PATH="$PWD/../venv314/bin:$PATH" \
  ZEPHYR_SDK_INSTALL_DIR="$PWD/../zephyr-sdk-1.0.1" \
  ZEPHYR_TOOLCHAIN_VARIANT=zephyr \
  ../venv314/bin/west build -p always \
  -b esp32c6_devkitc/esp32c6/hpcore \
  -d ../build-latency-qualified zephyr/tests/benchmarks/latency_measure \
  -- -DZEPHYR_MODULES="$PWD/modules/hal/espressif"
```

The recorded final configure/link/image-generation log reports success with
esptool **5.6.0**. The compiler uses **`-Os`**, no LTO flag, section garbage
collection and the upstream linker placement. Generated configuration and
`compile_commands.json` are authoritative for individual translation units.

Two setup issues are retained rather than presented as board failures:

- Python **3.11** was rejected because this Zephyr revision requires **3.12+**.
  A separate 3.14 virtual environment resolved that requirement.
- An earlier successful link/image conversion picked the host's esptool
  **5.1.0** through `PATH`, below the HAL's declared **5.2** requirement. The
  final isolated-environment rebuild uses 5.6.0; its ELF/image pair is the
  handoff below. Do not substitute the earlier `build-latency` outputs.

The final build has one consequential CMake warning: the suite requests a
**2,048-byte** system heap but the board requests at least **4,096 bytes**.
The build raises the effective heap to **4,096**, visible as
`-DK_HEAP_MEM_POOL_SIZE=4096` in compile commands. The retained `.config` still
shows the 2,048-byte request, so that value alone is not the effective heap.
The final C compilation reports no warning in the retained log.

## Resolved execution and timer configuration

| Area | Actual final selection |
| --- | --- |
| CPU / timer | CPU requested **160 MHz**; hardware counter frequency **16,000,000 Hz**, kernel tick **1 Hz** |
| Concurrency | One HP core; `main` priority **0** joins a separate test thread at preemptive priority **10**; preemptive yield workers **8**, cooperative yield workers **−11** |
| Protection / instrumentation | Userspace off, PM off, timeslicing off, hardware stack protection off; `CONFIG_FORCE_NO_ASSERT=y` disables `__ASSERT` despite `CONFIG_ASSERT=y` also being present |
| Stack requests | Main 2,048 B, ISR 2,048 B, idle 512 B; suite test/worker stack requests are 1,024 + 1,536 extra = **2,560 B**; reservations are not peaks |
| Timing batches | Default **1,000** iterations for most operations; heap uses **100** allocations/frees of **10 bytes** |
| Boot / console | Simple Boot, UART0 TX **GPIO16**, RX **GPIO17**, **115200 8N1**; DIO / 8 MB header, requested 80 MHz with an ambiguous 40/80 MHz encoding |

The priority assignments come from the suite's actual `K_THREAD_DEFINE` and
yield helper. Changing main's priority is unnecessary: main only waits for the
separate test thread. Higher numeric priority is not universally higher urgency
in Zephyr; preserve these native priority values and classes.

The [common timing backend](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/arch/common/timing.c)
calls `k_cycle_get_64`; the
[C6 timer driver](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/drivers/timer/esp32_sys_timer.c)
returns SYSTIMER **counter 1** through the HAL. Linked disassembly confirms
that path, ending in ROM `systimer_hal_get_counter_value` at **0x400003cc**.
The printed **“cycles” are nominal 16 MHz timer ticks, not 160 MHz CPU cycles**.
The configured frequency and printed nanoseconds use that counter rate;
they are not an independent oscillator calibration.

Only **19 of 47 default rows** subtract the timestamp observer estimate; **28
do not**. Eight unadjusted rows time a whole loop, while twenty sum individual
spans. Calibration selects the minimum of ten timestamp-read batches and also
includes loop/outer-read overhead. Its actual bias is not recoverable from
these printed averages. The [47-row map](../research/zephyr-native-audit.md)
records each endpoint and correction. Gross totals, calibration values and
individual durations are not printed; do not pool these mixed averages with
our gross protocol-3 RTTs.

## Exact handoff and flashing boundary

| Artifact | SHA-256 / size |
| --- | --- |
| `zephyr.elf` | `a9929eea623a79386cf4c9c90fbfeb3e73e38a9201e562576aaa7c1e63957f48` |
| `zephyr.bin` | `71d71cfcf07de94aee9624f1cae9b83e4a8aff75352876b027a7cdbf4ad360a3` / **162,288 bytes** |
| `zephyr.config` | `9797f6033f9593443984125a177888c6fbe5a22a8b704fd4cdcf049167e6a382` |
| `zephyr.dts` | `8ed622e820be0d8cb504028f8d8dd7ca33f6392a98b4175cf515d2f50c3d8a05` |
| `zephyr.map` | `b3db2b230179792ca76f82194610d332cfdeaa95b9025b97ee370998c30b96cb` |

**Write this Simple Boot `zephyr.bin` directly at offset 0.** It is already the
Zephyr image including its simple-loader layout. Do not use espflash
`save-image` to wrap the ELF in an unrelated ESP-IDF bootloader/partition table.
The generated runner requests offset 0 and flash **8 MB / DIO / 80 MHz**.
The image converter aliases C6 40/80 MHz encodings, so its image-info label is
not independent proof of frequency. The frozen RAM-only image layout also
reproduces the later ROM SHA comparison warning; MD5 transfer verification
does not resolve that separate check. An unused Espressif Kconfig flash-size string
still says 2 MB; the generated image command, runner and decoded image header
use the devicetree's 8 MB capacity. Root must retain flash verification and
preserve/restore its current verified full-image backup.

ELF file bytes include debug information. The final linker reports 138,020 B
in its FLASH region and 65,408 B in `sram0_0_seg`; aliased regions and ELF file
size must not be added into a fabricated complete flash/RAM or peak-use total.

The handoff includes ELF/bin/map/config/DTS, compile commands, runner, image-info,
Python inventory, manifest, disassembly, 47 expected metric names and provenance.
Its two source tarballs preserve the suite/board/timer and selected HAL source
evidence plus licenses; they are selected snapshots, not full dependency trees.
Pinned repository revisions identify the remaining source. Both upstream
working trees are clean. No board operation, custom native adapter or Rust
application qualification was performed by the build agent.
