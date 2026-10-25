# Custom Rust firmware and operating systems

Research checked: **2026-10-22**. Status: **bare-metal baseline implemented; a full OS is an engineering decision after measurements**. The [initial physical compute/observer pilot](../../results/initial-2026-10-22/README.md) passed; no custom scheduler or new OS has been implemented.

## Three meanings of “from scratch”

| Scope | What we would own | Value to this evaluation |
| --- | --- | --- |
| Minimal firmware | Rust `no_std`, existing startup/PAC/HAL, one event loop and interrupts | Establish the overhead floor without designing a scheduler |
| Custom execution layer | Keep HAL, add a scheduler/executor, IPC, stacks, timer management and radio-runtime adapters | Investigate a specific measured scheduling or memory weakness |
| New platform/OS | Startup, HAL/drivers, memory protection, network integration, update/recovery and long-term maintenance | Much larger scope; justify it with requirements existing stacks fail |

These are proposed scopes, not three existing products. Rust is appropriate for all three; low-level context switching may still require audited assembly, and radio/OpenThread dependencies can retain C/C++ and opaque binaries.

## Existing foundation

esp-hal explicitly supports C6 with target **`riscv32imac-unknown-none-elf`** and offers `esp-lp-hal` for its separate LP core. The C6 can therefore use upstream stable Rust for normal `no_std` work rather than an Xtensa compiler fork. [HAL repository](https://github.com/esp-rs/esp-hal), [target table](https://docs.rs/crate/esp-hal/latest), [Rust training setup](https://docs.espressif.com/projects/rust/no_std-training/02_2_software.html).

Use [esp-generate](https://github.com/esp-rs/esp-generate) with C6 selected and async/wireless options initially disabled, then build the generated project with `cargo build --release`. Pin generator, Rust and dependencies, and preserve the generated linker/startup configuration. This is a source-supported entry route, not a local build result. Existing components are MIT OR Apache-2.0 where declared; adopting them still means checking the final dependency and radio-binary licenses. [Generator licensing](https://github.com/esp-rs/esp-generate#license), [radio audit](../research/rust-support-audit.md).

## What a clean baseline should contain

- Fixed CPU clock; monotonic timestamp source; one GPIO measurement point; watchdog handling; serial results emitted after timing ends.
- A tiny event-loop workload and the same CPU payload used on every OS; build-size and reserved/used-memory accounting.
- Documented bootloader/partition layout and safe reflash/recovery path. Reuse the standard boot path first so startup engineering does not obscure scheduler comparisons.
- Two idle modes reported separately: CPU wait-for-interrupt and actual chip sleep. JTAG, USB bridges, regulators and LEDs can dominate whole-board power.

The first implementation needs no homemade scheduler. Add one only if the experiment requires it.

## What a custom OS would have to solve

| Responsibility | Questions to answer before committing |
| --- | --- |
| Interrupts / scheduler | Nested priorities, interrupt masking time, register/context preservation, task starvation and stack overflow detection |
| Radio ownership | Can the required esp-radio runtime contract be satisfied without esp-rtos? Who owns timers, heaps, interrupts, PHY and coexistence? |
| Memory / isolation | Flat-address-space Rust safety versus genuine per-task protection; unsafe code, DMA and FFI boundaries; no assumption that PMP hardware automatically provides process isolation |
| Power | Sleep eligibility, wake sources, clock transitions, radio recovery and LP-core handshakes |
| Product lifecycle | Signed updates, rollback, boot recovery, crash capture, persistent-state migration, provisioning and credentials |
| Maintenance | Chip errata, driver/blob updates, protocol vulnerabilities, dependency upgrades, regression hardware and ownership when maintainers leave |

esp-radio exposes an OS-adapter layer through [esp-radio-rtos-driver](https://github.com/esp-rs/esp-hal/tree/main/esp-radio-rtos-driver), while [esp-rtos](esp-rtos.md) already implements the common runtime. Replacing it is an integration project; it does not create an independent open-source Wi-Fi driver. Thread additionally involves [OpenThread bindings and libraries](https://github.com/esp-rs/openthread).

## Decision gate

Build our own execution layer only after identifying a repeatable deficiency—such as missed deadlines, excessive minimum footprint, unsuitable scheduling, or an isolation requirement—and demonstrating a small prototype that fixes it. Track engineering hours and working radio/update features alongside bytes and microseconds. A custom scheduler winning a radio-free microbenchmark would not establish that a new production OS is the best choice.
