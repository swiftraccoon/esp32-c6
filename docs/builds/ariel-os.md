# Build and startup qualification: Ariel OS

Recorded 2026-10-22 for unmodified upstream Ariel checkout
[`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9),
workspace package version **0.6.0**. Both the async `hello-world` and stackful
`hello-world-threading` examples built and physically printed `Hello World!`
on the attached ESP32-C6. These are narrow startup/UART/autostart smoke tests,
not benchmark results, peripheral qualification or proof of the board model.
The selected upstream board configuration is
`espressif-esp32-c6-devkitc-1`; the attached hardware remains identified only
as the previously inspected ESP32-C6 revision 0.2 / 8 MB / 40 MHz crystal.

## Inputs and reproducibility

- Checkout: `.cache/ariel-os`, exact commit above; independent
  `git status --porcelain` produced no tracked or untracked changes. No upstream
  source or manifest was patched to obtain these results.
- Target: `riscv32imac-unknown-none-elf`, Rust language edition 2024.
- Workspace MSRV: **1.98**, declared in the pinned upstream `Cargo.toml`.
  The attempted common comparison compiler **1.96.0 failed** the declared
  compiler requirements. That is a toolchain incompatibility, not an on-board
  failure. Compiler 1.98 itself was not tested here.
- Successful compiler: stable **1.99.1**, commit
  `58cf8f9c35b355199727627d07ae34a4e2846d67`, host
  `aarch64-apple-darwin`, LLVM **23.1.1**. `rustc -vV` reports
  `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Build driver: **laze 0.1.39**, locally installed under `.tools/bin`, compiled
  using Rust 1.96.0. The local install log is
  `.cache/install-laze-0.1.39.log`; the tool binary SHA-256 is
  `66761cac030ba943a8c2ea87eeba83004c48cbb15098a357cf019f9c5db23205`.
- Successful UART builds pass Cargo **`--locked`** through laze, preserving the
  upstream workspace lockfile. The default-transport 1.99.1 async build also
  passed, but only the explicitly selected UART/log images were flashed.

Relevant resolved dependencies differ from the direct HAL comparison:

| Dependency | Pinned resolved input |
| --- | --- |
| esp-hal | **1.1.1**, Ariel fork [`ce9bcf390ab6b7abc8ae3bc4989360a493c354e7`](https://github.com/ariel-os/esp-hal/tree/ce9bcf390ab6b7abc8ae3bc4989360a493c354e7) |
| esp-bootloader-esp-idf | **0.5.0**, same Ariel esp-hal fork commit |
| esp-alloc | **0.10.0**, same Ariel esp-hal fork commit |
| embassy-executor | **0.10.0**, crates.io release, not an Ariel fork |
| embassy-executor-macros | **0.8.0**, crates.io release |
| embassy-time | **0.5.1**, Ariel fork [`3c1b5fbf28237ea7144ae970c3aec11c38023c3f`](https://github.com/ariel-os/embassy/tree/3c1b5fbf28237ea7144ae970c3aec11c38023c3f) |
| embassy-embedded-hal | **0.6.0**, Ariel fork [`35b4b7a7f612c4d9bfe13c595d52172a5f8b9bbb`](https://github.com/ariel-os/embassy/tree/35b4b7a7f612c4d9bfe13c595d52172a5f8b9bbb) |
| embassy-sync | **0.8.0**, crates.io release |
| esp32c6 PAC | **0.23.2**, crates.io release |

The full archived lockfile is authoritative for every transitive input; a
workspace lockfile includes packages outside these two examples. Do not infer
that every listed radio/network crate is active in the hello images.

## Build commands and evidence

The hardware owner ran the builds from the evaluation repository root. The
initial build selected `examples/hello-world` and the C6 builder on Rust 1.96.0;
Cargo rejected the required MSRV before a valid image could run. The same
upstream async example built with Rust 1.99.1 and the default USB/defmt logging
selection. Those attempts are archived as
[the MSRV failure](../../results/ariel-qualification-2026-10-22/artifacts/build-rust-1.96-failed.log)
and [default-transport build](../../results/ariel-qualification-2026-10-22/artifacts/build-rust-1.99-default.log).

The final build explicitly selects UART and the ordinary `log` frontend,
matching the attached board's USB-to-UART bridge:

```sh
env RUSTUP_TOOLCHAIN=1.99.1 PATH="$PWD/.tools/bin:$PATH" \
  .tools/bin/laze -C .cache/ariel-os/examples -g build \
  -b espressif-esp32-c6-devkitc-1 \
  -a hello-world,hello-world-threading \
  -s logging-over-uart,log -D CARGO_ARGS=--locked
```

[The archived final build log](../../results/ariel-qualification-2026-10-22/artifacts/build-rust-1.99-uart.log)
records both successful release links: async 17.91 s, threading 2.10 s.
No compiler warning was reported in that final log. No authored adaptation code
was introduced, and the repository's authored API checklist does not certify
all unmodified upstream source.

Independent inspection of generated `build/build-global.ninja` confirmed
`cargo --locked build --profile=release`, target-specific context defines,
`-Clto=fat`, one codegen unit, embedded bitcode and **forced frame pointers**.
The upstream C6 build reserves an **8192-byte executor stack** and
**4096-byte ISR stack**. Both select single-core, executor-thread, idle-threads,
panic-printing, esp-println, log and logging-over-uart; the threading example
also selects `ariel-os/threading`. These differ from the initial Rust 1.96.0 /
esp-hal 1.2.0 / thin-LTO / no forced-frame-pointer benchmark configuration.
No cross-runtime performance attribution is valid from the hello builds.

## Physical startup evidence

The hardware owner flashed each archived image with esptool **5.1.0** and
preserved its verification log. Both report **`Hash of data verified`**.
Each image was then monitored using espflash serial mode at 115200 baud after
connection/reset; monitoring stopped with Ctrl-C after the expected line.

| Example | Upstream application context | Observed result |
| --- | --- | --- |
| `hello-world` | `#[ariel_os::task(autostart)]` async main | [Monitor](../../results/ariel-qualification-2026-10-22/hello-world/monitor.log) contains `[INFO] Hello World!`; [flash verification](../../results/ariel-qualification-2026-10-22/hello-world/flash.log) passed. |
| `hello-world-threading` | `#[ariel_os::thread(autostart)]` stackful main | [Monitor](../../results/ariel-qualification-2026-10-22/hello-world-threading/monitor.log) contains `[INFO] Hello World!`; [flash verification](../../results/ariel-qualification-2026-10-22/hello-world-threading/flash.log) passed. |

The source of each example logs once and calls `exit(ExitCode::SUCCESS)`.
Seeing the line establishes startup and the relevant autostart execution path
reaching UART logging. It does not establish the behavior after the exit call,
long-duration stability, priority/preemption behavior, scheduler latency,
async/thread interoperability, deadline guarantees, radio operation or power.

The boot logs report DIO / 80 MHz / 8 MB. These upstream hello applications do
not emit the independently decoded runtime clock snapshots used by protocol 2,
so those boot messages do not establish matched final runtime flash clocks.
The build-review agent read the preserved evidence and did not access hardware;
the root hardware owner performed the physical operations. Final original-flash
restoration evidence belongs to the enclosing hardware session record.

## Archived artifact identity

All paths below are under
[`results/ariel-qualification-2026-10-22`](../../results/ariel-qualification-2026-10-22/).
The source tarball and lockfile preserve upstream inputs; final image/ELF and
raw logs preserve the exact tested outputs.

| Artifact | SHA-256 |
| --- | --- |
| `artifacts/ariel-source.tar.gz` | `532ad51cfc1aad0f607a317b0880468a7544f72cf779751237574a9bd0e618c8` |
| `artifacts/Cargo.lock` | `159199a6c23e65309e255bde02a6d8968af062e64acfaca4bcd952dc8167409d` |
| `hello-world/firmware.elf` | `a359f54fc50731e0579190871c956fa01ed22711dd4ca42bf13a75cba72f113d` |
| `hello-world/flash.bin` | `53e1f3009de7671a27a3e771ef1970cab4e7d4ccac3b669b991b5adb64d4a9c1` |
| `hello-world/monitor.log` | `a18189028561666bc874a72f96ee1e29a393b1c21bc99bc91521301b7092c83e` |
| `hello-world-threading/firmware.elf` | `f0b40d4d7814d66b76b842b3cb32a7967f83b3b249f04dd95afa7284a294f253` |
| `hello-world-threading/flash.bin` | `86f200da4435c55dbe21819a8a94a8b6e91e2a1ff299e8cee646039e69d21339` |
| `hello-world-threading/monitor.log` | `7e6810356b32697e495bca25696bd9a524683e1aab4c5ce05d2b68a697380035` |

Next evaluation work is an authored Rust adapter using the shared payload,
matching raw protocol/clock observations, defined priorities and bounded
channels, then repeated physical captures. That work has not been performed by
qualifying these unmodified upstream examples.
