# OS firmware errata exposure

This source/configuration and archived-code audit was performed on 2026-10-23.
The [shared silicon ledger](../../../../docs/research/rev02-errata-workarounds.md)
records the pinned errata. These conclusions apply to the named historical OS
images/configurations, not to a later firmware build using the same SDK.

Actual implementation inputs inspected (cached paths describe the review-time
layout; the [ownership map](../../../../docs/organization.md) gives current locations):

| Input | Revision and evidence |
| --- | --- |
| ESP-IDF | **5.5.6**, [`880a203ae603d08a94d06aa01e0d6761958e5e9e`](https://github.com/espressif/esp-idf/tree/880a203ae603d08a94d06aa01e0d6761958e5e9e); local unmodified SDK under `firmware/esp-idf/.embuild/espressif/esp-idf/v5.5.6`. |
| Native compiler | GCC **14.2.0**, Espressif **esp-14.2.0_20260121**; actual generated compile commands and response files were read. |
| Rust comparison firmware | Rust **1.96.0**, compiler commit `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`; `riscv32imac-unknown-none-elf` and `riscv32imac-esp-espidf`. |
| Direct HAL | **esp-hal 1.2.0**, crates.io VCS metadata [`dbd951f78c1276461ea3050c2ba71f552a3508ef`](https://github.com/esp-rs/esp-hal/tree/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal). |
| Direct HAL ROM bindings | **esp-rom-sys 0.1.5**, VCS metadata [`160b10794227eb84805b8676fe188c1110801e9d`](https://github.com/esp-rs/esp-hal/tree/160b10794227eb84805b8676fe188c1110801e9d/esp-rom-sys). |
| Ariel qualification | Ariel [`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9), esp-hal **1.1.1** fork [`ce9bcf390ab6b7abc8ae3bc4989360a493c354e7`](https://github.com/ariel-os/esp-hal/tree/ce9bcf390ab6b7abc8ae3bc4989360a493c354e7), Rust **1.99.1**. Rust 1.96.0 did not satisfy Ariel's declared MSRV. |

[IDF build](../../firmware/esp-idf/README.md), [HAL build](../../firmware/esp-hal/README.md)
and [Ariel build](../builds/ariel-os.md) records identify the archived inputs and
ELFs. This audit does not promote the Ariel hello examples into a timing pass.

## CPU-863: data alignment and actual copy implementations

The [official description](https://github.com/espressif/esp-chip-errata/blob/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/03-errata-description/shared/cpu-misaligned-memory-access-may-lead-to-exception.rst)
concerns a **misaligned data load** immediately before a store when the
addresses have different PMP/PMA permissions. Flash/ROM-to-RAM copying is an
example. The documented alternatives include separating the instructions,
avoiding problematic unaligned operations and compatible protection settings.
The linker repair option mentioned there is conditional on linker support.
This is distinct from the **instruction placement** effect in our
[alignment probe](../../results/alignment-probe-2026-10-23/README.md).
Matching function entry alignment neither proves nor repairs data-access safety.

### IDF 5.5.6: mitigation present, selected and linked

The retained [actual SDK configuration](../../firmware/esp-idf/artifacts/sdkconfig-v2.esp32c6)
enables:

- `CONFIG_SOC_CPU_MISALIGNED_ACCESS_ON_PMP_MISMATCH_ISSUE=y`;
- `CONFIG_ESP_SYSTEM_PMP_IDRAM_SPLIT=y`;
- `CONFIG_LIBC_NEWLIB=y` and `CONFIG_LIBC_OPTIMIZED_MISALIGNED_ACCESS=y`.

The capability flag is an affected-silicon declaration, **not itself a code
repair**. The operative implementation is selected by
[newlib/CMakeLists.txt](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/newlib/CMakeLists.txt):
it compiles IDF's RISC-V `memcpy.c`/`memmove.c`, forces their inclusion using
`-u esp_libc_include_memcpy_impl` / `-u esp_libc_include_memmove_impl`, and
selects `-O2` for those files. Their
[memcpy](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/newlib/src/port/riscv/memcpy.c)
and [memmove](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/newlib/src/port/riscv/memmove.c)
sources explicitly identify **DIG-694** and space word loads/stores with
intervening instructions. These are existing upstream C/assembly dependencies;
no implementation was authored for this review.

Read-only `llvm-nm` and `llvm-objdump --disassemble-symbols=memcpy,memmove` on
the **archived final** IDF ELF confirmed the selection. Its recorded identity
is in the [clocked-handoff provenance](../../results/clocked-handoff-2026-10-23/artifacts/esp-idf-freertos/provenance.json);
the executable is omitted from this source checkout, as declared in the
[publication inventory](../../../../quality/publication.json).

| Symbol/path | Archived ELF evidence |
| --- | --- |
| `memcpy` | Defined text at `0x408098b6`, 210-byte body, not a ROM alias. |
| `memcpy` short word loop | Load at `0x408098dc`, two pointer increments, store at `0x408098e2`. The larger unrolled loop also separates the last load from the first store. |
| `memmove` | Defined text at `0x4080998a`, 204-byte body. |
| `memmove` forward word loop | Load at `0x40809a2a`, three increments/decrements, store at `0x40809a32`; backward/unrolled paths also have intervening instructions. |

The cached native link map and compile commands at
`firmware/esp-idf/target/riscv32imac-esp-espidf/release/build/esp-idf-sys-2dd3efe9f8c63c2b/out/build/`
agree with those final addresses and identify `libnewlib.a(memcpy.c.obj)` and
`libnewlib.a(memmove.c.obj)`. Other cached failed build directories must not be
used as evidence for the final firmware.

The generated compiler response file selects
`-march=rv32imac_zicsr_zifencei`; the copy translation units use `-O2`.
A metadata-only query of that installed GCC with that `-march` reports
`-mstrict-align [enabled]`. This is useful for ordinary compiler-generated
accesses; it does not rewrite explicit assembly, ROM or every third-party
binary. No claim of whole-program repair follows from that default.

### Direct HAL and Ariel: do not substitute the IDF conclusion

Both Rust 1.96.0 targets report `generic-rv32`, `+m,+a,+c` and ILP32 in
`rustc -Z unstable-options --print target-spec-json`. The HAL target selects
`rust-lld`; the IDF target uses the native GCC linker driver through ldproxy.
The target is not a C6-specific erratum-repair target. Metadata queries did not
build source. Neither installed linker advertised `--fix-esp-pmp-misalign`
in `--help`, and no such flag was found in the selected firmware Cargo configs
or generated native link rules. Do not add it on the assumption that a
documentation mention proves these linker versions support it.

The installed Rust 1.96.0 source includes `compiler_builtins` **0.1.160**.
Its `mem/impls.rs` has aligned-word/reassembly paths when `mem-unaligned` is
absent; its build script only enables that feature for x86, AArch64 and BPF.
This source is relevant when that implementation is actually linked. It does
**not** identify the direct HAL's selected copy routines: the archived
bare-metal, Embassy and Ariel threading ELFs resolve `memcpy` to
**ROM `0x400004ac`** and `memmove` to
**ROM `0x400004b0`**. Direct HAL's
[esp-rom-sys C6 additional.ld](https://github.com/esp-rs/esp-hal/blob/160b10794227eb84805b8676fe188c1110801e9d/esp-rom-sys/ld/esp32c6/rom/additional.ld)
supplies those aliases. ROM instructions are not contained in those ELFs;
their internal alignment behavior was **not established** here. IDF's
description of C6 ROM memory routines as suboptimal for misaligned access is
a performance note, not proof of a CPU-863 correctness failure or mitigation.
The [clocked-handoff](../../results/clocked-handoff-2026-10-23/README.md) and
[Ariel](../../results/ariel-qualification-2026-10-22/README.md) collections retain
their provenance; those executables are also omitted from the source checkout.

HAL's [PMP setup](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/soc/mod.rs)
uses different permissions for executable RAM, stack and data. Removing
protection to hide a possible failure would change the experiment and weaken
the configuration; it is not the chosen mitigation.

### Bounded closure for the next workload

For a radio-off scalar handoff/timer test, record natural alignment for message
values and timer/sample storage; use typed `u32`/`u64` operations, no packed
representations, unsafe unaligned word access or arbitrary flash-byte payload
copy in the timed contract. Review the final emitted adapter/primitive path
once it exists. IDF's identified copy replacements are already accounted for.
For direct HAL/Ariel, a ROM symbol alone does not block aligned scalar work,
but it also does not close a future arbitrary-memory-copy qualification.

Before adding copy/serialization with byte-offset flash/ROM inputs, reuse
[IDF's existing misaligned-memory tests](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/newlib/test_apps/newlib/main/test_misaligned_access.c)
as the closest contract. They include misalignment and cross-PMP cases plus
IRAM implementation checks. They were **not run** in this review; their
cross-PMP assembly case inserts the mitigation and is not a proof that every
copy path works. Determine the exact selected function and validate output
across source/destination offsets and boundaries before reporting copy speed.
The existing scalar workload's passing checksums are bounded historical
evidence, not exhaustive CPU-863 coverage.

## CLK-8588: selected clock versus powered oscillator

The [official entry](https://github.com/espressif/esp-chip-errata/blob/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/03-errata-description/esp32c6/clock-rc32k-not-oscillate.rst)
applies to rev0.2. Its relevant avoidance is selecting RC_SLOW rather than the
internal RC32K source. Powering/tuning an oscillator is distinct from relying
on it as the slow clock.

| Stack | Evidence and remaining boundary |
| --- | --- |
| IDF 5.5.6 | Actual config selects `CONFIG_RTC_CLK_SRC_INT_RC=y`; dangerous RC32K opt-in is unset. [C6 Kconfig.rtc](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_hw_support/port/esp32c6/Kconfig.rtc) gates RC32K behind `RTC_CLK_SRC_USE_DANGEROUS_RC32K_ALLOWED` and warns against it at extreme temperatures. This is configuration/source evidence, not an electrical oscillator measurement. |
| esp-hal 1.2.0 | [C6 clock presets](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/soc/esp32c6/clocks.rs) use `xtal32k::default_lp_slow_clk()`. [That function](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/soc/xtal32k.rs) selects RC_SLOW unless external XTAL32K is enabled; retained local build outputs show `ESP_HAL_CONFIG_USE_XTAL32K=false`. The selected preset avoids RC32K. |
| Ariel's HAL fork | [C6 RTC code](https://github.com/ariel-os/esp-hal/blob/ce9bcf390ab6b7abc8ae3bc4989360a493c354e7/esp-hal/src/rtc_cntl/rtc/esp32c6.rs) reads `LP_CLKRST.lp_clk_conf.slow_clk_sel` through `RtcSlowClockSource::current()` when selecting the modem LP source. [Ariel initialization](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-esp/src/lib.rs) does not establish a measured slow-clock readback. Do not silently apply HAL 1.2.0's newer preset conclusion to that older fork. |

The existing timing paths use HP SYSTIMER/`esp_timer_get_time`, with automatic
light sleep disabled in the comparison, rather than an RC32K sleep-wakeup
deadline. Therefore this is not an outstanding blocker for that narrow next
test. A later sleep/LP test must record the actual selected slow source,
calibration, inherited boot state, temperature range and wake behavior; a
room-temperature run alone cannot establish low-temperature RC32K reliability.

## RMT-176: mitigation is configuration-dependent in Rust HAL

The [official RMT entry](https://github.com/espressif/esp-chip-errata/blob/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/03-errata-description/shared/rmt-idle-level-cannot-be-controlled.rst)
applies to rev0.2 and directs register-controlled idle level for continuous
transmission. The selected workloads never initialize RMT.

IDF 5.5.6's [RMT TX driver](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_driver_rmt/src/rmt_tx.c)
calls `rmt_ll_tx_fix_idle_level(..., true)` when creating the channel and
starting a transaction. Its [C6 LL function](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/esp32c6/include/hal/rmt_ll.h)
sets `idle_out_en_chn` and the level. That is a source-level mitigation trace,
not a physical RMT pass.

In [HAL 1.2.0 rmt.rs](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/src/rmt.rs),
`TxChannelConfig::default()` leaves `idle_output` false, and channel setup
forwards that value to `idle_out_en`. The inspected
[Ariel HAL fork](https://github.com/ariel-os/esp-hal/blob/ce9bcf390ab6b7abc8ae3bc4989360a493c354e7/esp-hal/src/rmt.rs)
has the same configurable behavior. A future continuous-TX test must explicitly
select the idle output/level, preserve that choice, and verify the final line
level with a suitable fixture. Do not call either HAL automatically protected
for every RMT configuration. This affects later peripheral qualification, not
today's radio-off queue test.

## Timer issue watch: separate from official errata

[ESP-IDF issue #19036](https://github.com/espressif/esp-idf/issues/19036),
reviewed through the October 15 comments, remains open and labelled
`Status: In Progress`. Reporters describe backwards `esp_timer` values and
occasional stalls on rev0.2 in radio/peripheral workloads. Reports include XIAO,
DFRobot and a commercial device; the original reporter also reported five days
without reproduction on an Espressif devkit. Vendor comments do not establish
a hardware root cause and say the timer-only investigation did not reproduce
the fault. It is **not listed in the pinned official errata**. Do not label it
a proven C6 silicon defect, prescribe a private timer-repair function as a
general solution, or retroactively invalidate our short radio-off trials.

IDF 5.5.6 already includes a
[monotonic-clock test and timer advance tests](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esp_timer/test_apps/main/test_esp_timer.c).
The monotonic test compares clocks for ten seconds with interference from
interrupts; it is a reusable smoke-test reference, not a multi-hour combined
radio/peripheral soak. The issue discussion mentions additional master-branch
tests, but that statement alone does not prove they are present in 5.5.6 or
that they isolate every reported trigger. Neither test set was run here.

Before a connected/sleep reliability experiment, inspect the latest upstream
status and reusable reproducer/test revisions, then record independent timer
reference, radio/peripheral/power profile, duration, wrap/compensation handling
and failure/recovery capture. For the next short radio-off timing experiment,
retain monotonicity checks and explicit failure records without claiming that
short absence of failure closes this issue.

## What is closed and what remains open

| Research question | Disposition |
| --- | --- |
| Which official errata affect this exact revision? | Closed against the pinned v1.3 source: CPU-863, CLK-8588 and RMT-176; all other listed entries fixed before or at v0.2. |
| Is IDF's selected memory-copy mitigation merely assumed? | Closed for the archived IDF ELF's named memcpy/memmove paths by source, config, linkage and disassembly. No whole-program certification. |
| Does the current HAL/Ariel use the same mitigated libc? | No. Archived symbols select ROM copies; their internal behavior remains unqualified. |
| Must all firmware work wait for every later peripheral erratum? | No. A radio-off aligned scalar queue/timer brief can explicitly avoid those triggers and proceed through ordinary binary/correctness qualification. |
| Are arbitrary memory copies, RMT continuous TX, sleep/LP clocks and connected timer reliability ready to score? | No. Each has the bounded prerequisite described above; readiness is per operation/configuration. |

Verification consisted of source reads, pinned upstream document reads,
installed compiler/linker metadata queries, and read-only archived ELF
symbol/disassembly inspection. No new fixture, peripheral, timer-soak or
erratum-triggering test was performed; no implementation change was made.
