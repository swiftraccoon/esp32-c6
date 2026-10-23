# Ariel native test build and source evidence

Built **2026-10-23**, without board access. All four unmodified upstream test
apps built successfully at Ariel commit
[`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9).
The [pre-hardware brief](../experiments/ariel-native-tests.md) specifies test
contracts, UART markers, the twelve-boot qualification budget and timing limits.
Compilation and local image generation are not physical passes.

## Inputs and build command

Use the already qualified development input, not a silent release/HEAD upgrade:

- Rust **1.99.1**, commit `58cf8f9c35b355199727627d07ae34a4e2846d67`,
  LLVM **23.1.1**, host `aarch64-apple-darwin`.
- Target `riscv32imac-unknown-none-elf`, edition **2024**; Ariel declares MSRV
  **1.98** at this revision. This is different from the Rust 1.96 IDF study.
- Build driver **laze 0.1.39** and image tool **espflash 4.6.0**, both already
  installed under the evaluation repository's ignored `.tools/bin`.
- Board builder `espressif-esp32-c6-devkitc-1`, selected for the proven C6 UART
  route; this does not identify the attached carrier's model.
- Original upstream lockfile, SHA-256
  `159199a6c23e65309e255bde02a6d8968af062e64acfaca4bcd952dc8167409d`.
  Resolved forks and other dependency versions are recorded in the
  [earlier Ariel build](ariel-os.md); none was upgraded for these tests.

From the evaluation repository root:

```sh
env RUSTUP_TOOLCHAIN=1.99.1 PATH="$PWD/.tools/bin:$PATH" \
  .tools/bin/laze -C .cache/ariel-os/tests -g build \
  -b espressif-esp32-c6-devkitc-1 \
  -a bench-sched-yield,bench-sched-flags,threading-dynamic-prios,threading-lock \
  -s logging-over-uart,log -D CARGO_ARGS=--locked
```

The app READMEs show an nRF52840 run command, so that command alone was not C6
evidence. Their actual laze declarations admit this C6 builder; the command
above successfully built it. No `run`, flash or serial command was executed by
the build agent.

| App | Build result | Reported Cargo time |
| --- | --- | --- |
| `threading-dynamic-prios` | Passed, no warnings in retained log | 2.53 s |
| `threading-lock` | Passed, no warnings in retained log | 2.80 s |
| `bench-sched-flags` | Passed, no warnings in retained log | 1.60 s |
| `bench-sched-yield` | Passed, no warnings in retained log | 1.43 s |

These are warm-cache build times, not first-clean-build effort measurements.
No formatter, source edit or new test was needed to build unmodified upstream
code. The repository's authored API checklist does not certify all upstream
dependencies.

## Generated configuration and source review

Generated `build/build-global.ninja` confirms `cargo --locked build
--profile=release`, fat LTO, one codegen unit, embedded bitcode, forced frame
pointers and ordinary linker placement. It records `single-core`,
`executor-thread`, `idle-threads`, `threading`, `esp-println`, `panic-printing`,
`log` and `logging-over-uart` features. The benchmark manifests also select
`ariel-os/bench`. No radio feature or semihosting/std exit feature is selected.
The configured executor and ISR stacks are **8,192 / 4,096 bytes**; thread macro
defaults reserve **2,048 bytes per test thread**. These reservations do not
establish peak stack usage.

The selected board source maps UART0 TX/RX to GPIO16/17. The logging route
selects the HAL fork's esp-println UART implementation and existing ROM output;
it does not require a new general UART driver or an assumed LED pin. Root
captures at the previously demonstrated **115200 baud, 8N1** CP2102N bridge.
[Board source](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-boards/src/espressif-esp32-c6-devkitc-1.rs),
[logging configuration](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/laze-project.yml).

Ariel's ESP initialization requests `CpuClock::max()` (C6: 160 MHz). The pinned
HAL 1.1.1 fork's `SystemTimer::ticks_per_second()` derives XTAL/2.5 for C6:
nominal **16 MHz** at the identified 40 MHz crystal. The native benchmark helper
directly reads UNIT0 twice for one batch; it does not call a whole-microsecond
timer API. The result truncates the u64 delta to 32-bit usize **before** dividing
by 1,000 or 10,000, and prints only the floored mean. This is why the short host
ceiling and explicit aggregate-only interpretation matter.
[HAL timer source](https://github.com/ariel-os/esp-hal/blob/ce9bcf390ab6b7abc8ae3bc4989360a493c354e7/esp-hal/src/timer/systimer.rs),
[benchmark helper](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/src/ariel-os-bench/src/esp.rs).

Read-only ELF inspection finds each expected test trampoline and resolves
`memcpy`/`memmove` to ROM **0x400004ac / 0x400004b0** in all four images. It does
not establish those ROM routines' internal alignment behavior. No runtime
clock-register readback, external calibration, monotonicity sweep or general
erratum qualification was added to the upstream tests.

## Exact outputs and operator handoff

Original build products are under:

```text
.cache/ariel-os/build/bin/espressif-esp32-c6-devkitc-1/cargo/
  riscv32imac-unknown-none-elf/release/<app>
```

Stable handoff copies are under
`.cache/ariel-native-tests-2026-10-23/artifacts/<app>.elf` and `<app>.bin`.
Each local merged image was generated without device access:

```sh
.tools/bin/espflash save-image --chip esp32c6 --merge --skip-padding \
  --flash-mode dio --flash-freq 80mhz --flash-size 8mb \
  <app>.elf <app>.bin
```

These merge espflash's prebuilt bootloader/partition table and the app for a
write at flash offset **0**; skip-padding retains a compact image. DIO/80 MHz/
8 MB are image requests, not independently decoded final runtime clock proof.
Root must archive exact inputs and hash-verify flashing with the preserved
current pre-session image available for restoration. No eFuse change is needed.

| App | ELF SHA-256 | Merged image SHA-256 / bytes |
| --- | --- | --- |
| `bench-sched-yield` | `eca8fe3e5400cffb48ae7d2fd786e5dc048f486160da03761c2286ec58678111` | `865434c1a589c9e93ef228599bf739e348d5e74df362a32c7e4d924fd898bd39` / 164,320 |
| `bench-sched-flags` | `4a33fc518f6ced8f9791f8f986e1a4523ff48cdac80f2667cb253035c2a0596a` | `8d2556bd3250e199c79744ec5f9a00b3fd0ee8aaa645d3ddaa7ddce9d293ba64` / 164,960 |
| `threading-dynamic-prios` | `266cae2a70cf9b31f7a47ae9d258b64da2c940c235a83c8b25f2c3b097ddec70` | `11edb1aa2ca25ac37b5b11c6c3dc8e57e796f6e60cb8969c2515a581df288718` / 165,952 |
| `threading-lock` | `2b2cba8d93e305071f640897e112e33c790c1d454b17f07020c9b3d6dd290d30` | `7811dfa78171664561cff1165fbb2efdb8e5464a4b294bef9b00f50227508515` / 168,736 |

The handoff directory also retains `build.log`, `build-global.ninja`,
`Cargo.lock`, `rustc-vv.txt`, four image-generation logs, before/after upstream
status/diff files and `SHA256SUMS`. Status/diff files are empty: no upstream
source, manifest or lock change. The generated Ninja file hashes to
`2b468e3b2dc4ce28b2af04f5031ee86d55da55854c4d1ee3e7921742d6ccd77f`.
`ariel-source.tar.gz` was made with `git archive HEAD`, includes upstream license
notices and hashes to
`532ad51cfc1aad0f607a317b0880468a7544f72cf779751237574a9bd0e618c8`.
This matches the prior source snapshot at the same commit.

Expected UART completion lines, error cases, post-marker behavior and the
restore-current-image requirement are in the pre-hardware brief. Hardware
captures and their eventual pass/fail report must be archived separately by
root; these build results establish no on-board timing or assertion pass. The
operator has copied the inputs into the
[qualification archive](../../results/ariel-native-tests-2026-10-23/README.md)
before its first hardware write; that archive preserves the permanent handoff.
