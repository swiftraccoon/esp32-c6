# esp-rtos

Research checked: **2026-10-23**. Status: **first-round Rust runtime candidate**, and a dependency of the Embassy stack. That stack passed the [initial physical compute/observer pilot](../../results/initial-2026-10-22/README.md); a separate esp-rtos thread workload has not been measured. The published release and development thread APIs differ despite sharing a package version.

## Identity and support

esp-rtos is Espressif's Rust RTOS implementation for esp-hal. It supplies the thread/queue facilities used by esp-radio and supports async execution through Embassy integration. It is a different implementation from ESP-IDF's FreeRTOS. Source uses `#![no_std]`; the manifest explicitly selects `esp32c6` and the target `riscv32imac-unknown-none-elf`. License: MIT OR Apache-2.0. [README](https://github.com/esp-rs/esp-hal/blob/main/esp-rtos/README.md), [manifest](https://github.com/esp-rs/esp-hal/blob/main/esp-rtos/Cargo.toml), [runtime source](https://github.com/esp-rs/esp-hal/blob/main/esp-rtos/src/lib.rs).

The measured **published 0.4.0** comes from commit [`63e97be`](https://github.com/esp-rs/esp-hal/tree/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos). The examined development snapshot [`0cf41de`](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/esp-rtos/Cargo.toml) also advertises **0.4.0**, but contains a newer public thread API. The same manifest version does not make the source equivalent. See the [compatibility audit](../research/platform-compatibility.md).

## Execution model

Development thread source documents priority scheduling and round-robin sharing between equal-priority threads. Its `ThreadSpawner` can use caller-provided static stacks or heap allocation. Joining a thread returns its result; a thread exits by returning and cannot be forcibly deleted through this API. [Development thread API](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/esp-rtos/src/thread.rs).

The [published 0.4.0 public library](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/lib.rs) exports `CurrentThreadHandle`, but has no equivalent general application `ThreadSpawner`. Its internal radio-scheduler adapter is not evidence of that public application API. Our working Embassy dependency therefore does not establish a ready-to-use native application-thread variant. Select and qualify the development API or another maintained thread layer deliberately; preserve the released baseline separately.

The main runtime source documents timer initialization, runtime-managed async integration and experimental automatic light sleep. Do not apply generic dual-core examples to the C6 application CPU. This chip's LP core is a distinct programming target, not a second symmetric application core. [Runtime source](https://github.com/esp-rs/esp-hal/blob/main/esp-rtos/src/lib.rs), [LP HAL](https://github.com/esp-rs/esp-hal/tree/main/esp-lp-hal).

## Peripherals and radio

Hardware features come from esp-hal; wireless comes from esp-radio and optional protocol stacks. esp-rtos alone does not supply a filesystem, POSIX environment or complete provisioning/OTA product. See [shared support and radio constraints](../research/rust-support-audit.md), including the distinction between radio hardware and Thread/Matter integration.

## Strengths and costs

- **Strengths:** Rust `no_std`, vendor ecosystem alignment, preemptive threads alongside async, direct control over stack allocation, and an explicit radio-driver runtime contract.
- **Costs:** younger and narrower integration than full IoT operating systems; evolving APIs; stack/heap accounting and product services remain application work; vendor radio dependencies remain in the trusted code base.

## Build and benchmark entry

Use the [esp-generate C6 flow](embassy.md#build-entry-and-experiment) for a compatible async runtime scaffold. The pinned [development thread module](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/esp-rtos/src/thread.rs) describes a synchronous variant on that development input; it cannot simply be used with our released 0.4.0 lockfile. Neither the development-thread build nor physical behavior has been qualified locally. The [timing/concurrency research](../research/timing-and-concurrency-contract.md) records the immediate supported alternatives.

Compare **esp-rtos threads** against **Embassy tasks on esp-rtos**, recording the scheduler/runtime shared by both. Useful tests are semaphore/channel wakeup, high-priority deadline response under a busy low-priority thread, equal-priority fairness, context-switch distribution and per-thread stack usage. Record timeslice configuration and include stack bookkeeping rather than counting only nominal stack bytes. This isolates execution-model tradeoffs more clearly than presenting both as unrelated OSes.
