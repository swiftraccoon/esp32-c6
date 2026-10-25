# RTIC

Research checked: **2026-10-23**. Status: **supported C6 concurrency candidate**.
Both pinned upstream examples now build unmodified with Rust1.96.0. The
[build record](../builds/rtic.md) and
[qualification contract](../experiments/rtic-qualification.md) distinguish
compilation, image-packaging corrections and prospective physical acceptance.
The [first physical attempt](../../results/rtic-qualification-2026-10-23/RESULTS.md)
prints all seven expected markers, then restarts during the guard and fails
frozen v1. The remaining two attempts were not run. Reset cause is unconfirmed;
this is not evidence of universal RTIC/C6 incompatibility.

## Exact support evidence

RTIC's official book explicitly names **`riscv-esp32c6-backend`**. The upstream repository contains a complete **`examples/esp32c6`** project, its PAC/backend code and a C6 SYSTIMER monotonic. This is stronger evidence than generic RISC-V support. [Supported backends](https://rtic.rs/2/book/en/starting_a_project.html), [C6 example](https://github.com/rtic-rs/rtic/tree/master/examples/esp32c6), [C6 monotonic](https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/src/esp32c6.rs).

RTIC is an interrupt-driven Rust concurrency framework, not a full filesystem/networking OS. Tasks and shared-resource access are declared so the framework can apply priority-based resource scheduling. MIT OR Apache-2.0. [Project and license](https://github.com/rtic-rs/rtic), [scheduling model](https://rtic.rs/).

## C6 build route and version trap

The examined C6 example uses stable Rust with `rust-src`, target **`riscv32imac-unknown-none-elf`**, and pins **`esp-hal =1.0.0-rc.0`**. Do not silently replace it with the newest HAL while calling the example verified. Its Cargo config also enables frame pointers, which can affect benchmark performance. [Pinned manifest](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/Cargo.toml), [toolchain](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/rust-toolchain.toml), [Cargo config](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/.cargo/config.toml).

From that pinned upstream example directory, the executed build commands are:

```sh
cargo +1.96.0 build --release --locked --example monotonic --features riscv-esp32c6-backend
cargo +1.96.0 build --release --locked --example sw_and_hw --features riscv-esp32c6-backend
```

Both pass with the unchanged tracked upstream lockfile. Their `cargo run`
counterparts invoke a flashing runner. The selected physical probe is only
`monotonic`; the GPIO9/button example remains build evidence. Actual source
delays are2/3/4seconds, rather than the README's1/2/3. Its priority-zero futures
do not demonstrate positive-priority task preemption. See the
[source review](../research/rtic-source.md) for timer/vector ownership, inherited
watchdog/clocks and automatic console selection. Source inspection identified
packaging issues requiring separate corrected images; compiler success alone
does not establish flash readiness or watchdog survival.

## Board context update, 2026-10-24

The **DevKitC-1-N8** has a documented BOOT/GPIO9
button and separate native USB-C connector; see the [board map](../../../../docs/hardware.md).
The button provides a candidate fixture for a later controlled GPIO test, not
an observed interrupt/preemption result. The frozen monotonic v1 fixture still
requires the CP2102N UART route with native USB disconnected because the
selected automatic logger can switch to USB. Two available connectors do not
change that contract or explain its retained failed attempt.

## Strengths and support gaps

- **Strengths:** Rust-first and explicit C6 support; compile-time task/resource model; direct interrupt priority control; useful for deterministic control loops and comparing executor overhead.
- **Costs:** less turnkey networking and device management; application structure follows RTIC's model; old example HAL needs either isolation or a deliberate compatibility port; complete C6 wireless integration with current esp-radio is **unverified in this audit**.

GPIO/timers are evidenced by the example. Wider peripherals depend on its HAL/PAC. Chip-level Wi-Fi/BLE/802.15.4 does not prove that a particular RTIC and radio-runtime combination works: timer ownership, interrupts, critical sections and the preemptive radio driver runtime must coexist. Begin with radio disabled. [Example source](https://github.com/rtic-rs/rtic/tree/master/examples/esp32c6/examples), [shared radio audit](../research/rust-support-audit.md).

## Best benchmark role

Use timer-release-to-task latency, GPIO ISR response, priority interference, resource-lock blocking and periodic deadline misses. Keep payloads identical to Embassy/Ariel, but describe the semantic difference: an RTIC priority task is not a stackful OS thread. Match or disclose frame-pointer, optimization and HAL-version differences. A networking trial is a separate integration milestone, not part of the initial support claim.
