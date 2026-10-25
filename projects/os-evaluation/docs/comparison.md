# ESP32 C6 platform comparison

The initial comparison is a portfolio: a vendor stack, a Rust async/thread stack, an integrated Rust OS, an interrupt-driven Rust framework, and a portable RTOS. Extend it across all exposed hardware as drivers and fixtures permit. No overall platform is selected.

The attached chip and flash are confirmed in [hardware](../../../docs/hardware.md). **Upstream support below means source/documentation evidence, not a successful local firmware test.** The linked sheets contain primary sources and exact boundaries. Generic chip-family, RISC-V, protocol-stack or host-driver support does not qualify as a C6 OS port.

Local evidence has advanced separately: [clock/message tests](../results/clocked-handoff-2026-10-23/README.md)
pass on bare metal, Embassy/esp-rtos and Rust std/IDF; [Ariel startup tests](../results/ariel-qualification-2026-10-22/README.md)
pass for an async task and a stackful thread. The message timings describe the
chosen channel/runtime paths and priorities. The [alignment probe](../results/alignment-probe-2026-10-23/README.md)
also demonstrates why matching compute source and bytes alone is insufficient.
These tests do not qualify networking, power, peripheral coverage or a product.

The [matched IDF API study](../results/idf-queue-api-2026-10-23/RESULTS.md)
subsequently completed 24 passing boots with the same kernel, Rust threads and
priorities: std-channel boot medians **62.875 µs**, native-queue boot medians
**19.9375 µs**, at the nominal raw-counter scale. This demonstrates a useful
API-path choice before a kernel replacement. Instrumentation, implementation
and ordinary release placement remain part of the comparison; it does not
isolate context-switch cost or rank it against historical Embassy results.
The [result review](../results/idf-queue-api-2026-10-23/RESULTS.md) records independence and
calibration limits.

Later evidence has narrower acceptance limits. [Ariel's native tests](../results/ariel-native-tests-2026-10-23/RESULTS.md)
retain twelve expected outputs and the operator's original pass reports, but
the first observation guard is operator-attested without an archived wait
receipt. Its derived yield value is arithmetic, without a validated peer switch
count. [Zephyr native v1](../results/zephyr-native-tests-2026-10-23/RESULTS.md)
admits **0/3 boots**, despite each emitting 47 rows and a success banner: the
required header is compiled out and ROM image-SHA warnings remain. The
[Rust probe](../results/zephyr-rust-build-2026-10-23/README.md) failed three
build stages and produced no image or board pass. None of these observations changes the separate matched-IDF result into a CPU-utilization or capacity
measurement.

## Kernel and execution model choices

| Candidate | Execution model | Rust position | Main evaluation value | Main limitation or gate |
| --- | --- | --- | --- | --- |
| [ESP-IDF FreeRTOS](os/esp-idf-freertos.md) | Preemptive threads, vendor SDK | Established `std` application route | Vendor capability and lifecycle baseline | SDK/crate coupling; Rust wrappers may lag C APIs |
| [esp-rtos](os/esp-rtos.md) | Rust preemptive threads, async integration | Native `no_std` | Thread scheduling and radio-runtime contract | Younger runtime; product services assembled separately |
| [Embassy stack](os/embassy.md) | Cooperative tasks within executor; proposed ESP runtime is esp-rtos | Native `no_std` | Async memory/wakeup tradeoffs | Long-running task interference; shares runtime with thread variant |
| [Ariel OS](os/ariel-os.md) | Priority-preemptive threads plus Embassy; equal-priority threads cooperate | Native `no_std` | Integrated Rust OS and portable APIs | C6 persistent-storage abstraction missing; selected APIs narrower than silicon |
| [RTIC](os/rtic.md) | Interrupt-priority concurrency and resource management | Native `no_std` | Control loops, priorities and deadline behavior | Old pinned example HAL; wireless integration unverified |
| [Zephyr](os/zephyr.md) | Configurable RTOS threads and services | Official integration; selected C6 Rust hello failed its build gate | Cross-vendor portability and services | Native v1 admitted 0/3; image/output defects and Rust generator/build gate remain |
| [NuttX](os/nuttx.md) | Preemptive RTOS with POSIX-style services | Experimental; C6 Rust unverified | Threads, sockets, filesystems and diagnostics | Rust integration and 802.15.4 coverage gaps |
| [RIOT](os/riot.md) | IoT RTOS and networking ecosystem | Current RV32 Rust target disabled | Native mesh/network comparison | C2Rust/Clang blocker; flash encryption unsupported |
| [RT-Thread](os/rt-thread.md) | Preemptive RTOS | C6 Rust unverified | Additional kernel and BSP study | New C6 BSP; major buses and radio drivers pending |
| [Custom Rust](os/custom-rust.md) | Event loop first; scheduler only if justified | Native `no_std` | Overhead floor and targeted architectural experiments | We own integration, recovery and maintenance |

ESP-IDF's `std` and Rust `no_std` represent different library/runtime environments. Neither implies the application is entirely independent of C, assembly, vendor libraries, or FFI. Treat these dependencies explicitly in build and maintenance comparisons.

## Radio evidence

| Stack | Wi-Fi | BLE | Raw 802.15.4 | Thread and other protocols |
| --- | --- | --- | --- | --- |
| ESP-IDF FreeRTOS | Upstream | Upstream | Upstream API | OpenThread; additional Zigbee SDK; Matter is another application stack |
| Embassy with esp-rtos and esp-radio | C6 driver | Controller plus separate host such as TrouBLE | C6 driver, unstable feature in inspected source | OpenThread Rust bindings/examples exist; Wi-Fi plus 802.15.4 forbidden by inspected esp-radio features |
| Ariel OS | C6 matrix lists support | C6 matrix lists support | Integration unverified | Prove required Thread/Matter integration separately |
| RTIC | Integration unverified | Integration unverified | Integration unverified | Radio-runtime and interrupt ownership need proof |
| Zephyr | Board device listed | HCI device listed | Device listed | Stack exists; qualify C6 role and configuration on hardware |
| NuttX | C6 configuration | C6 scanning configuration | Not established | Not established for C6 in reviewed configurations |
| RIOT | C6 port | NimBLE/ESP BLE features | C6 driver feature | OpenThread package exists; selected C6 combination needs proof; GNRC is distinct |
| RT-Thread C6 BSP | Pending | Pending | Pending | Unavailable through the reviewed BSP |

Each row inherits the qualifications in its reference sheet, including source versions. Support for a controller is not proof of an application profile. Zigbee, Thread, Matter, BLE Mesh, and Bluetooth Classic are not interchangeable. Test coexistence only for combinations the selected stack supports.

## Wider application environments

[Arduino, MicroPython, CircuitPython and Moddable](os/idf-runtimes.md) are useful application/developer-experience comparisons. Their runtimes and garbage collection, where present, change application latency and memory needs while retaining the underlying IDF platform. CircuitPython's C6 port is labeled alpha.

[Additional C6 routes](os/other-runtimes.md) include AtomVM's Erlang/Elixir VM, Toit/Jaguar, NodeMCU Lua, native TinyGo, WAMR and Embedded Swift. WAMR is particularly relevant to our Rust preference: it permits a native-Rust versus Rust-to-Wasm payload comparison with explicit host bindings and memory limits. TinyGo is a native runtime rather than an IDF wrapper; its C6 radio coverage is not established by the reviewed target sources.

[Mongoose OS](os/mongoose-os.md) has a merged C6 platform but unresolved integration evidence; qualify a clean build before timing it. [Contiki-NG's community C6 port](os/contiki-ng.md) enables a distinct RPL/6LoWPAN experiment within a FreeRTOS task; it does not provide TSCH. Compare equivalent application behavior and identify protocol differences.

[MOS-Rust](os/mos-rust.md), [project-trobbio, ThistleOS and OpenC6](os/research-kernels.md) belong in a research group. They may reveal valuable scheduler, loader, isolation or LP-core ideas, but C6 source code and architectural claims are only the beginning of qualification.

The [ecosystem audit](research/ecosystem-audit.md) and [Rust audit](research/rust-support-audit.md) explain why other familiar OS names were not admitted as working C6 stacks. A port can move into the comparison when source, a C6-specific build and hardware operation are demonstrated.

## Evaluation order

The [prior-work review](../../../docs/research/prior-work.md) and
[experiment admission brief](benchmark-plan.md#prior-work-and-experiment-admission)
govern new tests. Reuse maintained scheduler benchmarks and published radio
fixtures before creating equivalents. Current local results calibrate the
board and selected Rust API paths; subsequent experiments must state which
remaining decision they can change.

Resolve observable acceptance/image gates and per-metric timing semantics
before another comparison. A load sweep is conditional on a matched workload
and reviewed CPU/ISR accounting, offered-load/capacity and memory methods; see
the [utilization audit](research/utilization-audit.md). It is not an automatic
all-platform sweep. Hardware offload, radios, sleep/energy, external IRQ response,
recovery and measured upgrade effort still need their own evidence.

1. **Prepare the board:** use the DevKitC-1-N8 and [documented port/pin map](../../../docs/hardware.md); record PCB revision, chosen USB route and any external wiring. Preserve existing flash, choose compatible pinned dependencies, and review the [coding standard](../../../docs/coding-standard.md).
2. **Establish baselines:** Rust application on ESP-IDF/FreeRTOS, Embassy/esp-rtos, a thread variant, and a small `no_std` event loop. Keep features aligned and radio disabled initially.
3. **Compare alternative models:** Ariel OS and RTIC; probe Zephyr Rust before implementing its benchmark adapter. Prove timers/interrupts and common-code correctness first.
4. **Expand hardware coverage:** use the [subsystem worksheet](hardware-coverage.md), adding networking, power, LP core, buses, storage, crypto and recovery as fixtures allow.
5. **Add broader platforms:** NuttX and RIOT after their Rust gates; RT-Thread for its supported kernel features; application runtimes and hobby systems in their own categories. Discuss C adapters or a language fallback if those gates cannot be met.
6. **Make the architectural decision:** compare capabilities and raw results, quantify integration/maintenance effort, and prototype a custom layer only where existing stacks miss a concrete objective.

Stages can proceed independently where their fixtures and build paths permit. This is an integration order, not a reduction in scope.

## Decision method

Because the scope is all hardware-supported workloads, **do not use a single default weighted score**. Publish capability coverage, timing, footprint, energy, reliability and engineering effort side by side. Identify platforms that offer useful tradeoffs across these dimensions.

[Decision criteria CSV](decision-criteria.csv) includes optional scenario weights for future projects. The columns each total 100; no platform has been scored. Before applying one, define mandatory feature thresholds, the exact workloads, and acceptable limits. Use the same thresholds for all candidates.

For an optional numerical ranking, define a utility from 0 to 1 for every metric before seeing results: 0 at an unacceptable limit, 1 at a target, with bounded interpolation between them. Reverse the direction for latency/energy/size versus throughput. State how multiple metrics form each criterion, then compute `100 × sum(weight × utility) / sum(weight)`. Exclude candidates that fail mandatory gates. Do not convert unsupported requirements, failed tests or unmeasured values into favorable scores; report them and defer an overall ranking until mandatory data exists. Exclude a criterion as irrelevant only for the entire scenario, with an explicit weight renormalization applied to every candidate.

For custom OS work, include the cost of drivers, radio adaptation, power, debug tools, crash recovery, signed updates and ongoing upgrades. A custom event loop is a valuable baseline even if a new scheduler never becomes necessary.
