# Ariel OS

Research checked: **2026-10-23**. Status: **async/stackful startup checks passed; twelve native-test boots retain their expected outputs with partial qualification evidence**. The first native observation guard is operator-attested without an archived wait receipt. Original pass reports remain preserved; they are not twelve independently established full qualifications. See the [results](../../results/ariel-native-tests-2026-10-23/RESULTS.md) and [native build record](../builds/ariel-native-tests.md).

## Identity and evidence

Ariel is a Rust library OS combining Embassy and HAL components with a preemptive scheduler, portable peripheral APIs, network security facilities and the `laze` build system. It targets microcontrollers rather than desktop processes. License: MIT OR Apache-2.0. [Design overview](https://ariel-os.github.io/ariel-os/dev/docs/book/), [GitHub](https://github.com/ariel-os/ariel-os).

The published hardware matrix explicitly lists **ESP32-C6-DevKitC-1**, builder **`espressif-esp32-c6-devkitc-1`**, as **Tier 1**. Tier 1 means regular project testing, not a guarantee that every peripheral works. The underlying C6 target is `riscv32imac-unknown-none-elf`. The **DevKitC-1-N8** matches this family; the [hardware inventory](../../../../docs/hardware.md) now records its native USB-C port and onboard fixtures. Check the selected console, pin ownership and PCB revision when expanding tests. [Hardware matrix](https://ariel-os.github.io/ariel-os/dev/docs/book/hardware-functionality-support.html), [target definition](https://github.com/ariel-os/ariel-os/blob/main/laze-project.yml).

## Features and limits on C6

| Area | Evidence-based status |
| --- | --- |
| Portable hardware APIs | Matrix lists logging/debug channel, GPIO, I2C controller, SPI main, UART and hardware RNG |
| Wi-Fi / BLE | Both marked supported for DevKitC-1 |
| Persistent storage | Matrix marks C6 persistent storage unsupported through its portable abstraction; do not infer support from flash being present |
| General USB / Ethernet | No portable user USB, USB Ethernet or native Ethernet entry for this C6 board |
| 802.15.4 / Thread / Matter | Not established by this matrix. Lower-level Rust support exists, but Ariel integration requires a separate proof |
| Rust | Application and OS are Rust; repository supports stable Rust and can raise its minimum compiler in patch updates |

Sources: [portable functionality matrix](https://ariel-os.github.io/ariel-os/dev/docs/book/hardware-functionality-support.html), [current compiler policy](https://github.com/ariel-os/ariel-os#minimum-supported-rust-version-msrv-and-policy), [radio audit](../research/rust-support-audit.md).

The scheduler is tickless and priority-preemptive; threads at equal priority cooperate, so **equal-priority time slicing is not provided**. This matters when comparing it to FreeRTOS or esp-rtos. Async tasks and stackful threads can coexist. The C6 has one HP application core, so its benchmark does not test Ariel's multicore advantages. [Multithreading design](https://ariel-os.github.io/ariel-os/dev/docs/book/multithreading.html), [thread/async example](https://github.com/ariel-os/ariel-os/tree/main/examples/thread-async-interop).

## Strengths and costs

- **Strengths:** Rust-native application development; less peripheral/network setup boilerplate; async plus preemptive work; portable APIs; explicit C6 testing tier; a natural comparison with manually assembled Embassy.
- **Costs:** younger ecosystem and changing APIs; extra `laze` tooling; curated versions may lag a needed crate feature; portable API coverage is narrower than chip capability; equal-priority cooperative behavior needs deliberate workload design.

These are selection assessments, not a comparative latency or power result.

## Verified build entry

The latest inspected **released v0.6.0** at [`cee68033`](https://github.com/ariel-os/ariel-os/tree/cee68033f87cded63e8d62bf751d7add9b0a57ed) declares Rust **1.95** and resolves an older HAL/Embassy combination than the development checkout below. It is a possible Rust 1.96 comparison route, not yet a local build or hardware pass. The [compatibility audit](../research/platform-compatibility.md) separates both dependency tuples; the development checkout's compiler requirement is not a universal Ariel requirement.

The unmodified checkout at [`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9) requires Rust **1.98**. Rust 1.96.0 failed its declared compiler requirements; stable **1.99.1** with **laze 0.1.39** built both hello examples successfully. Rust 1.98 itself was not tested. From this evaluation repository root, the final UART/log build was:

```sh
env RUSTUP_TOOLCHAIN=1.99.1 PATH="$PWD/.tools/bin:$PATH" \
  .tools/bin/laze -C .cache/ariel-os/examples -g build \
  -b espressif-esp32-c6-devkitc-1 \
  -a hello-world,hello-world-threading \
  -s logging-over-uart,log -D CARGO_ARGS=--locked
```

Both archived images were flashed with hash verification and printed `[INFO] Hello World!` over UART: one through an async autostart task and one through an autostart stackful thread. This establishes those startup/logging paths only. Native USB and peripheral behavior were not exercised by those UART startup runs. The [build record](../builds/ariel-os.md) preserves exact toolchain, forked HAL/Embassy inputs, hashes, commands, raw logs and limits. Their compiler, HAL version and upstream optimization settings differ from the benchmark baseline. For new applications, follow the [prerequisites](https://ariel-os.github.io/ariel-os/dev/docs/book/getting-started.html), [ariel-os-hello](https://github.com/ariel-os/ariel-os-hello) or [template](https://github.com/ariel-os/ariel-os-template).

The same checkout and Rust 1.99.1 tuple produce expected output in three boots
each of `bench-sched-yield`, `bench-sched-flags`, `threading-dynamic-prios` and
`threading-lock`. Yield prints **155 ticks per iteration**, flags **215**.
The derived **77** is integer arithmetic; no peer-progress counter validates a
switch count or isolated switch latency. These are floored aggregate means at
the nominal 16 MHz scale, without individual samples or runtime clock readback.
Priority/lock markers follow ordinary assertions. The first post-marker guard
has only operator-reported support; later window arithmetic depends on saved
operator metadata. The [results](../../results/ariel-native-tests-2026-10-23/RESULTS.md)
preserve those distinctions and the original reports.

## Benchmark hooks

The native outputs provide integration references with the limits above.
Before a common workload, define peer-progress/state checks, observable timing
guards and clock evidence. Then add the same checked Rust payload used by other
candidates under a new contract.
Measure high-priority wake latency, async/thread handoff, per-thread stack cost
and ready/sleeping task scaling. Compare to Embassy with aligned esp-hal/radio
versions where possible; shared low-level drivers limit OS attribution. Keep
upstream batch means separate from a new per-event application contract.
