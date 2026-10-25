# ESP32-C6 operating system evaluation

Compare Rust integration, runtime behavior and hardware support on the
ESP32-C6-DevKitC-1-N8. The evaluation is ongoing; no overall platform or custom
runtime is selected.

## Start here

| Reference | Purpose |
| --- | --- |
| [Comparison](docs/comparison.md) | Candidate overview and decision criteria |
| [Benchmark plan](docs/benchmark-plan.md) | Workloads, controls, timed endpoints and aggregation |
| [Results](results/README.md) | Collection outcomes, retained purpose and interpretation limits |
| [Hardware coverage](docs/hardware-coverage.md) | Tested and missing peripheral/radio/power capabilities |
| [Documentation](docs/README.md) | Platform sheets, methods, experiment contracts and source audits |
| [Evidence gates](docs/assurance-workflow.md) and [ledger](docs/evidence-ledger.md) | Admission rules, declared claims and verification |
| [Hardware inventory](../../docs/hardware.md) | Board, port roles and fixtures |

The same-kernel queue comparison supports investigating the Rust API path before
replacing the OS. Accounting and finite-load probes qualify narrower measurement
methods; physical utilization, sustainable capacity and global memory peaks remain
unmeasured. RTIC needs reset attribution, and the selected Zephyr Rust route has
a generated-code build blocker. Each collection records its actual evidence;
driver, radio, power and maintenance work remains necessary for platform selection.

## Build and verify

The host workspace owns [bench-core](crates/bench-core/docs/api.md) and
[benchctl](tools/benchctl/README.md). Run here:

```sh
cargo +1.96.0 test --workspace --locked
cargo +1.96.0 build --release --locked -p benchctl
```

Enter [HAL firmware](firmware/esp-hal/README.md) or
[IDF firmware](firmware/esp-idf/README.md) before its build commands so Cargo loads
the target configuration. Their SDK/toolchain and workspaces are independent.

From the repository root, check available historical data and source inventory:

```sh
projects/os-evaluation/target/release/benchctl verify-public-snapshot \
  --root . --ledger projects/os-evaluation/quality/evidence.json \
  --manifest quality/publication.json
```

This verifies declared availability and retained raw replay. It cannot restore
omitted inputs, original freeze ancestry or full-artifact admission. See
[ownership and retention](../../docs/organization.md) for historical names.

## Kernel and Rust platform sheets

Each sheet covers actual C6 evidence, project/source links, license, supported features, Rust fit, strengths, costs, and useful experiments.

| Candidate | Classification |
| --- | --- |
| [ESP-IDF FreeRTOS](docs/os/esp-idf-freertos.md) | Vendor RTOS stack; established Rust application route with `std` |
| [Zephyr](docs/os/zephyr.md) | Portable RTOS; native v1 qualification and selected Rust build probe failed |
| [Apache NuttX](docs/os/nuttx.md) | POSIX-oriented RTOS; experimental Rust integration |
| [RIOT](docs/os/riot.md) | IoT/networking RTOS; current RV32 Rust integration blocker |
| [RT-Thread](docs/os/rt-thread.md) | Recent, limited upstream C6 kernel port |
| [esp-rtos](docs/os/esp-rtos.md) | Rust `no_std` runtime with preemptive threads |
| [Embassy with esp-hal](docs/os/embassy.md) | Async executor stack; proposed configuration uses esp-rtos |
| [Ariel OS](docs/os/ariel-os.md) | Integrated Rust library OS with C6 board support |
| [RTIC](docs/os/rtic.md) | Rust interrupt-driven concurrency framework with a C6 backend |
| [Custom Rust firmware or OS](docs/os/custom-rust.md) | Bare-metal baseline and staged custom-development choices |

## Frameworks and experimental systems

| Reference | Included systems |
| --- | --- |
| [Application runtimes](docs/os/idf-runtimes.md) | Arduino-ESP32, MicroPython, CircuitPython, Moddable/XS |
| [Additional language and sandbox routes](docs/os/other-runtimes.md) | AtomVM Erlang/Elixir, Toit/Jaguar, NodeMCU Lua, TinyGo, WAMR, Embedded Swift |
| [Mongoose OS](docs/os/mongoose-os.md) | Newly integrated C6 framework above IDF/FreeRTOS |
| [Contiki-NG C6 port](docs/os/contiki-ng.md) | Community mesh networking port hosted in a FreeRTOS task |
| [MOS-Rust](docs/os/mos-rust.md) | Experimental Rust scheduler, shell and C6 BLE adapter |
| [Research systems](docs/os/research-kernels.md) | project-trobbio, ThistleOS, OpenC6 BIOS |

Embassy, esp-rtos and Ariel share components; Arduino and several other runtimes retain FreeRTOS. The comparison distinguishes execution models and full stacks instead of counting every name as a different kernel.

The [ecosystem audit](docs/research/ecosystem-audit.md) documents excluded or unverified candidates such as ThreadX, RTEMS, Mynewt, AliOS and LiteOS. The [Rust support audit](docs/research/rust-support-audit.md) covers Tock, Hubris, exact targets, radio layers, source snapshots and compatibility traps. These are a survey of public support evidence; new ports can be admitted using the same build/board proof requirements.
