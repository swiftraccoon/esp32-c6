# MOS-Rust

Research checked: **2026-10-22**. Status: **experimental C6 implementation; second-round research candidate**. No build or on-device validation here.

## Evidence

[MOS-Rust](https://github.com/Eplankton/mos-rust) describes an experimental Rust rewrite of MOS with a preemptive kernel and shell. Unlike a plan-only port, its repository actually contains [C6 board source](https://github.com/Eplankton/mos-rust/tree/a5046fd30a8d40adf79e26afe1d708c15286c402/boards/esp32c6), including a trap handler, BSP, shell, BLE application and radio scheduler adapter. Project license is [Apache-2.0](https://github.com/Eplankton/mos-rust/blob/master/LICENSE).

The [board manifest](https://github.com/Eplankton/mos-rust/blob/a5046fd30a8d40adf79e26afe1d708c15286c402/boards/esp32c6/Cargo.toml) pins `esp-hal =1.0.0-rc.0` and `esp-wifi 0.15.1`, enabling BLE with its custom scheduler and `bleps`. The [toolchain](https://github.com/Eplankton/mos-rust/blob/master/rust-toolchain.toml) selects stable Rust, while [board configuration](https://github.com/Eplankton/mos-rust/blob/master/boards/esp32c6/.cargo/config.toml) uses `riscv32imac-unknown-none-elf` and a probe-rs flashing runner.

## Fit and limits

- **Pros:** a concrete Rust custom-scheduler reference on this exact chip; examples of adapting the vendor radio to an alternative runtime; shell and peripheral integration to study.
- **Cons:** explicitly experimental; older pinned HAL/radio stack; project-wide MOS feature claims do not prove C6 support; reproducibility and scheduler/unsafe-code review are needed before trusting results.
- **Wireless:** BLE is evidenced in C6 code. Wi-Fi, raw 802.15.4, Thread and simultaneous-protocol behavior are **unverified** for this port; do not import support claims from other MOS boards.

## Entry point and useful experiment

Inspect the pinned board source and dependencies, then attempt a build-only `cargo build --release` from `boards/esp32c6`, resolving any repository-specific configuration first. This is a candidate entry point, **not a tested build recipe**. Do not use `cargo run` during research because its runner writes firmware.

Benchmark boot/footprint, thread handoff and BLE responsiveness only after a reproducible build and timer/interrupt sanity checks. Most useful as a reference for the [custom Rust path](custom-rust.md), rather than an immediate production default.
