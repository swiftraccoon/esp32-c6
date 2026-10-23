# Zephyr C6 Rust hello build evidence

Recorded **2026-10-23**. Three bounded attempts to build the official Rust hello
sample ended in failure. The final attempt reaches the Rust crate's generated
devicetree and fails with nine E0425 name-resolution errors. **No final
`zephyr.elf` or `zephyr.bin`, board execution or serial access** is established.
The [build-only archive](../../results/zephyr-rust-build-2026-10-23/README.md)
preserves failures and exact diagnostic inputs; the
[source review](../research/zephyr-rust-feasibility.md) explains the selected API
and architecture route.

## Fixed inputs and source ownership

| Input | Selection |
| --- | --- |
| Zephyr | 4.4.0, `684c9e8f32e4373a21098559f748f06915f950c9` |
| `zephyr-lang-rust` | Release optional-manifest pin `dd73abc242e995784da62352fe8c70d9a6c7ac2e` |
| `hal_espressif` | `b7953b8019361d09e613f7011d2ccc41b984d087` |
| Rust | Stable 1.96.0; `riscv32i-unknown-none-elf` installed for this probe; edition 2021 upstream sample |
| Native toolchain | Zephyr SDK 1.0.1, RISC-V GCC 14.3.0 |
| Host environment | Apple Silicon macOS; existing Python 3.14.8 venv, west 1.5.0, CMake 4.4.3, Ninja 1.13.2 |
| Binding generator | Upstream `zephyr-sys` requires bindgen 0.70.1; resolved lockfile retained |
| Board | `esp32c6_devkitc/esp32c6/hpcore`; no new assumption about the carrier model or peripheral pins |

Only the optional Rust module was fetched. It was activated with
`west config manifest.project-filter +zephyr-lang-rust` for the initial probe;
the workspace previously had no project filter. The pre-probe setting is
restored afterward, and before/active/restored configurations are archived.
No kernel/HAL/native-suite source, native build artifacts or root Cargo
workspace were changed. The fetched module's Git worktree remains clean.

The upstream hello sample enables Rust/allocation/logging, requests a 4,096-byte
main stack and disables the RTT backend. Generated configuration confirms
`CONFIG_RUST_SUPPORTED=y`, `CONFIG_RUST=y`, `CONFIG_RUST_ALLOC=y`,
`CONFIG_UART_CONSOLE=y`, UART logging and **1,000 Hz** scheduler ticks.
Userspace and dynamic timer-frequency reading are off. Stack reservation is
not high-water usage. The expected successful hello line would contain
`Hello world from Rust on esp32c6_devkitc`; no such execution was attempted.

## Attempts and commands

From `.cache/zephyr-qualification/workspace`, the first attempt used the
official sample path and a new `../build-rust-hello` build directory, with the
existing SDK/venv, Rust 1.96.0 and `LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib`.
Cargo rejected the sample because it was beneath this evaluation repository's
ancestor workspace without being a member. That is host-layout friction,
not an embedded compiler or C6 runtime failure.

For subsequent attempts, `git archive` of the exact Rust module was extracted
to a fresh host temporary directory outside the ancestor Cargo workspace.
Explicit `ZEPHYR_MODULES` selected the existing HAL and this pristine Rust copy.
The copied upstream source/configuration stayed unchanged; 139 original files
were independently byte-compared with the retained snapshot. The sample's
newly generated Cargo lockfile is retained; no upstream Rust or C was authored.

The second attempt, build directory `../build-rust-hello-isolated`, used
Homebrew **libclang 23.1.4**. Bindings produced one-byte opaque records for
several kernel types while retaining larger C-layout expectations. Compilation
stopped at **27 E0080 constant-evaluation errors**, including `k_thread`
(1 versus 232 bytes), `k_mutex` (1 versus 20) and `k_timer` (1 versus 56).
The build-script trace also records host clang resource headers selected from
the Xcode-beta toolchain. Do not suppress those layout assertions.

One bounded variation selected the already installed Homebrew
**clang/libclang 21.1.8** together, using a new
`../build-rust-hello-clang21` directory and the same source/compiler/lockfile.
This moved past `zephyr-sys` compilation, which emitted four generated
`unnecessary transmute` warnings. It then failed at **nine E0425 errors** in
generated flash-partition getters. This distinguishes host binding-tool
compatibility from the remaining Rust generator failure; it does not isolate
whether the library version, resource headers or another host-tool difference
caused the earlier layout behavior.

To reproduce that final failure with the retained source and lockfile, from
the evaluation repository root:

```sh
RUST_PROBE_ROOT="$PWD"
RUST_PROBE_SOURCE=$(mktemp -d "${TMPDIR:-/tmp}/esp32-c6-zephyr-rust-dd73-XXXXXX")
tar -xzf results/zephyr-rust-build-2026-10-23/artifacts/rust-module-source.tar.gz \
  -C "$RUST_PROBE_SOURCE"
cp results/zephyr-rust-build-2026-10-23/artifacts/Cargo.lock \
  "$RUST_PROBE_SOURCE/samples/hello_world/Cargo.lock"
cd .cache/zephyr-qualification/workspace
env RUSTUP_TOOLCHAIN=1.96.0 ZEPHYR_TOOLCHAIN_VARIANT=zephyr \
  ZEPHYR_SDK_INSTALL_DIR="$RUST_PROBE_ROOT/.cache/zephyr-qualification/zephyr-sdk-1.0.1" \
  LIBCLANG_PATH=/opt/homebrew/opt/llvm@21/lib \
  PATH="$RUST_PROBE_ROOT/.cache/zephyr-qualification/venv314/bin:/opt/homebrew/opt/llvm@21/bin:$PATH" \
  ../venv314/bin/west build -b esp32c6_devkitc/esp32c6/hpcore \
  -d ../build-rust-hello-reproduce "$RUST_PROBE_SOURCE/samples/hello_world" -- \
  "-DZEPHYR_MODULES=$PWD/modules/hal/espressif;$RUST_PROBE_SOURCE"
```

Use a new build directory if the named directory already contains another
configuration. The unmodified module's CMake command does not pass `--locked`;
its actual commands, paths and generated configuration remain in the archived
logs/Ninja/CMake files. No additional version sweep or generated-code edit was
performed after this final failure.

## Remaining generator gate

The generated devicetree calls `super::super::super::get_instance_raw` from
flash partitions. The module's `dt-rust.yaml` matches these partitions under
`soc-nv-flash`, but its flash-controller compatible list omits the C6's
`espressif,esp32-flash-controller`, so the expected parent getter is absent.
The hello configuration also leaves `CONFIG_FLASH` off. A future remedy must
check generator/controller/device availability together; adding one compatible
string alone is not a demonstrated solution. The selected upstream C flash
driver does define that compatible, but it is not selected merely because the
DT node exists.

Upstream issue 141 reports the same diagnostic class on C6 blinky with
unspecified pins. Our exact-input hello failure is new local evidence, without
proving the report used the same source. No general C6 Rust impossibility,
unsafe ABI workaround, custom C shim or kernel replacement follows from it.
Review a targeted Rust generator/configuration remedy or an upstream fix next;
then establish build/link, UART, thread and timer gates before a Rust benchmark.

Checks for this work were build attempts, generated-input inspection,
source-byte comparison and clean upstream Git status. There was no maintained
authored implementation change needing a new API checklist, and no on-device
test, host-unit-test, Clippy or formatter pass is claimed for upstream code.
