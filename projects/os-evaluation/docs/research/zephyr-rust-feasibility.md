# Rust application feasibility on Zephyr's C6 port

Reviewed **2026-10-23** against the selected Zephyr **4.4.0** source. Its
unmodified Rust hello sample **failed the local build gate** in generated
devicetree code. The module admits the architecture/configuration in source;
a missing C6 entry in its sample lists was not itself a rejection. The
[bounded build record](../builds/zephyr-rust.md) separates the actual generator
failure from earlier host-layout and libclang problems. No Rust ELF/image was
produced or flashed; linking, execution, thread/timer behavior and radio APIs
remain unqualified.

## Select the release's actual module

Zephyr commit `684c9e8f32e4373a21098559f748f06915f950c9` pins the optional
`zephyr-lang-rust` project to
**`dd73abc242e995784da62352fe8c70d9a6c7ac2e`**, under `modules/lang/rust`.
Use that pairing first, rather than substituting the development snapshot
`6823b690…` cited in the earlier [OS sheet](../os/zephyr.md).
[Pinned optional manifest](https://github.com/zephyrproject-rtos/zephyr/blob/684c9e8f32e4373a21098559f748f06915f950c9/submanifests/optional.yaml).

The module's [Kconfig](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/Kconfig)
enables `RUST_SUPPORTED` for RISC-V other than RV32E/RV128I when timer frequency
is not dynamically read. C6's selected configuration is RV32I with M/A/C
extensions and a static timer frequency, so it satisfies that source gate.
`CONFIG_RUST` selects `EXPERIMENTAL`; this is not a stable-support promise.

The [target mapping](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/CMakeLists.txt)
selects **`riscv32i-unknown-none-elf` for all RV32I configurations**. Do not
silently force the direct-HAL project's `riscv32imac-unknown-none-elf` target.
The narrower generated target is deliberate module behavior, although the C6
has additional instructions. Record the resulting Rust/C ABI and ELF at the
build gate; source mapping alone is not linked-ABI validation.

The pinned [`etc/platforms.txt`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/etc/platforms.txt)
contains QEMU and `m2gl025_miv` entries, with neither C3 nor C6. The
[hello sample test list](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/samples/hello_world/sample.yaml)
also includes nRF52840. These describe configured test coverage, not an
exhaustive architecture rejection list or evidence that those tests passed.

## Reuse upstream Rust hello first

The selected [hello source](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/samples/hello_world/src/lib.rs)
is `no_std`, initializes the Rust logger once and emits
`Hello world from Rust on <CONFIG_BOARD>`. Its
[configuration](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/samples/hello_world/prj.conf)
already enables Rust, allocation and logging, disables the RTT backend and
requests a **4,096-byte main stack**. Start unchanged; that reservation is not
measured stack usage.

The module builds a Rust `staticlib` into Zephyr, using an existing upstream
[`main.c`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/main.c)
entry wrapper and bindgen-generated wrappers for inline C functions. This needs
no newly authored C adapter. The application is Rust over a C kernel/HAL; it is
not an esp-hal firmware or a Rust kernel. The manifests and source retain their
upstream license notices; preserve them in any snapshot.

The [Rust crate manifest](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/Cargo.toml)
declares MSRV **1.85**. Use our installed stable **1.96.0** as an explicit probe,
with its prebuilt `riscv32i` core; no nightly/build-std step appears in this
module's build command. `zephyr-sys` uses bindgen **0.70.1**, requiring host
libclang, and generates bindings against the selected headers/configuration.
Its declarations of Rust support do not remove that host-tool dependency.

In a prepared, deliberately selected Zephyr workspace, the official enablement
is `west config manifest.project-filter +zephyr-lang-rust`, then module update.
Preserve any existing project-filter policy when applying it. Our cache had no
prior project filter, and `west update zephyr-lang-rust` fetches only this
project at the manifest pin. Build with the C6 HP target:

```sh
rustup target add --toolchain 1.96.0 riscv32i-unknown-none-elf
west config manifest.project-filter +zephyr-lang-rust
west update zephyr-lang-rust
env RUSTUP_TOOLCHAIN=1.96.0 \
  west build -b esp32c6_devkitc/esp32c6/hpcore \
  -d build-rust-hello modules/lang/rust/samples/hello_world
```

These are the upstream setup/build steps, not a claim that every directory
layout works. This repository is itself a Cargo workspace: placing the module
beneath `.cache` exposes the upstream sample to the ancestor workspace and
produced a local Cargo membership error before Rust compilation. The build
record preserves that failure and the isolated-source retry. Use source
isolation rather than altering the benchmark workspace or upstream Rust code
to mask this host-layout issue.

## Existing Rust APIs for later gates

| Need | Pinned source route | Boundary to qualify |
| --- | --- | --- |
| Threads and priorities | [`zephyr::thread`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/src/thread.rs): static thread pools, ready/start handles, priority setting and join. [`sys::thread`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/src/sys/thread.rs) also offers static kernel objects and allocation-backed closure spawning. | Pool exhaustion can panic; stack/lifetime and priority contracts require review. Existence of safe methods is not a complete soundness or C6 execution audit. |
| Time and sleeping | [`zephyr::time`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/src/time.rs): `Duration`, `Instant`, `now`, `sleep` and timeout conversions. | These use scheduler ticks from `CONFIG_SYS_CLOCK_TICKS_PER_SEC`, not raw 16 MHz SYSTIMER ticks. Do not inherit the native benchmark's 1 Hz setting or use this as a fine timing method without a new contract. |
| Timers | [`tests/timer`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/tree/dd73abc242e995784da62352fe8c70d9a6c7ac2e/tests/timer) already exercises allocation, timer counts, waiting and callbacks. | Its printed totals and success marker do not prove deadline accuracy or all dropped-timer paths. Run the upstream test before duplicating it. |
| Synchronization | [`sync`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/src/sync.rs): Mutex/Condvar/SpinMutex, portable atomics and allocation-gated channels/Arc. | Rust's selected RV32I target lacks native pointer atomics; this module uses portable-atomic with critical sections. Hardware ISA capability alone does not change the selected Rust target's atomic contract. |
| UART logging | [`logging`](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/src/logging.rs) selects printk or Zephyr logging from Kconfig; hello already uses it. | Check generated UART0 console/backend settings, allocation and logger initialization. Hello output establishes startup/logging only. |
| Missing operation | `zephyr::raw` re-exports generated `zephyr-sys` bindings. | Use maintained higher-level APIs first. Any new FFI boundary requires its own ownership/context/error review; no custom C shim has yet been shown necessary. |

The [existing philosophers sample](https://github.com/zephyrproject-rtos/zephyr-lang-rust/tree/dd73abc242e995784da62352fe8c70d9a6c7ac2e/samples/philosophers)
provides an additional unmodified thread/synchronization smoke test, with
selectable semaphore/mutex/condition-variable/channel profiles. Its progress
counters can qualify useful behavior after hello, without turning that sample
into our checked capacity-one request/reply benchmark.

## Known limits and next evidence

Upstream [issue 141](https://github.com/zephyrproject-rtos/zephyr-lang-rust/issues/141)
reports an ESP32-C6 blinky build failure in generated devicetree Rust code
(`get_instance_raw` not found). It remained open with no comments when checked.
The report provides no exact module/kernel revision. Our subsequent unmodified
hello probe reproduced **nine E0425 errors of that class** at the exact release
pairing above, after a host-libclang change allowed the low-level bindings to
compile. This establishes a concrete failure for our selected build tuple,
without proving the report used identical inputs or that all C6 Rust versions
fail. The crate compiles its generated devicetree module even if hello does not
access GPIO, so avoiding blinky did not eliminate the generator failure.

The module [critical-section implementation](https://github.com/zephyrproject-rtos/zephyr-lang-rust/blob/dd73abc242e995784da62352fe8c70d9a6c7ac2e/zephyr/src/sys.rs)
rejects `CONFIG_USERSPACE`; it documents single-CPU testing and uses Zephyr IRQ
locking. Do not infer userspace protection or multi-core correctness from
Zephyr's general feature set. Rust allocation wraps the selected libc
malloc/free, with an alignment limit in this pin; inspect the actual libc/heap
configuration before interpreting memory reservations or adopting it generally.

Admission order is **clean build/link and artifacts → UART hello → existing
thread/timer tests → reviewed common Rust workload**. Preserve failures and
configuration changes at each stage. Native Zephyr suite execution does not
qualify any of these Rust gates, radio APIs or product update/recovery behavior.
