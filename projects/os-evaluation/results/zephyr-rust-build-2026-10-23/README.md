# Zephyr C6 Rust hello build probe

Recorded **2026-10-23**. **The selected unmodified Rust hello did not build**:
after resolving host-layout/tool compatibility problems, generated devicetree
Rust failed name resolution. No final firmware ELF/image was produced, no
serial port was opened and no firmware was flashed for this probe. Native
Zephyr qualification is a separate experiment.

The decision is whether our selected Zephyr release provides a working Rust
application route before writing a benchmark adapter. Reuse the official
`samples/hello_world`, with its own Rust/configuration and upstream C/generated
wrappers, rather than author another hello or a C shim. The
[source feasibility review](../../docs/research/zephyr-rust-feasibility.md) and
[build record](../../docs/builds/zephyr-rust.md) explain the source gate,
commands, existing APIs and remaining work.

## Inputs and retained outcomes

Fixed kernel **Zephyr 4.4.0**, commit
`684c9e8f32e4373a21098559f748f06915f950c9`; its optional manifest pins Rust
module `dd73abc242e995784da62352fe8c70d9a6c7ac2e`. HAL is
`b7953b8019361d09e613f7011d2ccc41b984d087`. The compiler is stable
**Rust 1.96.0**, target **`riscv32i-unknown-none-elf`** selected by the module,
with Zephyr SDK **1.0.1 / GCC 14.3.0** on macOS arm64.

| Attempt | Change / stage | Retained result |
| --- | --- | --- |
| 1: sample within this repository's `.cache` | Official module/sample under the ancestor Cargo workspace | Cargo rejects sample workspace membership before compiling Rust. [Log](artifacts/rust-hello-build.log). |
| 2: pristine source copy outside that ancestry | Explicit HAL/Rust module selection; Homebrew libclang 23.1.4, default host clang executable search | **27 E0080 layout errors** in generated `zephyr-sys`; e.g. Rust's one-byte `k_thread` conflicts with the generated 232-byte C size. [Log](artifacts/rust-hello-isolated-build.log). |
| 3: one bounded host-tool compatibility variation | Existing Homebrew clang/libclang **21.1.8** selected together; same source, Rust compiler and dependency lock | `zephyr-sys` compiles with four generated `unnecessary transmute` warnings; `zephyr` then fails with **nine E0425 errors** for missing parent `get_instance_raw` calls in generated flash-partition code. [Log](artifacts/rust-hello-clang21-build.log). |

The third outcome is the remaining build blocker for this recorded tuple.
Changing the selected host library and executable search allowed the low-level
bindings to compile; the precise cause of the earlier layout failure was not
isolated. No layout check, source, generator or generated output was patched,
and no compiler/module version search continued after these attempts.

The sample's generated configuration admits `RUST_SUPPORTED`, enables Rust,
allocation, UART console/logging and a 4 KiB main stack, with scheduler frequency
**1,000 Hz**. This differs from the native latency suite's 1 Hz profile.
`CONFIG_FLASH` is off, while the devicetree still contains flash partitions.
The generator's partition rule emits a parent getter although its controller
rule does not cover the C6's `espressif,esp32-flash-controller` compatible.
This is a source-level lead for a targeted remedy, not a tested patch or proof
that adding a compatible string alone is sufficient.

Upstream [issue 141](https://github.com/zephyrproject-rtos/zephyr-lang-rust/issues/141)
reports the same missing-getter diagnostic class for C6 blinky without exact
version pins. Our result establishes a local failure with recorded inputs,
not that the issue used identical inputs or that every C6 Rust combination is
unsupported. A native-kernel pass does not make this Rust build pass.

## Provenance and integrity

[provenance.json](provenance.json) records all three failed stages, tools and
scope. Under [artifacts](artifacts) are full logs, CMake/Ninja configuration,
generated DTS/configuration, low-level bindings, generated devicetree source,
wrapper C and build-script output. The original Rust module source snapshot
includes its license notices. **139 original source files** in the isolated
copy match that snapshot byte-for-byte; the generated sample `Cargo.lock` is
retained separately. Kernel, HAL and fetched Rust-module Git worktrees remained
clean. The temporary optional-module project filter was recorded and restored
to its pre-probe setting.

Upstream invokes Cargo without `--locked`; retain the resolved
[Cargo.lock](artifacts/Cargo.lock) when reproducing this attempt. This was a
bounded compatibility probe, not a preregistered performance measurement.
[SHA256SUMS](SHA256SUMS) covers this completed build-only archive.

No memory footprint, latency, power, boot reliability or Rust driver score is
reported. The next useful step is a reviewed generator/device-availability
remedy or a verified upstream fix, followed by a new build gate and UART hello.
