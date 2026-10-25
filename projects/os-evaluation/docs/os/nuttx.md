# Apache NuttX

Research checked: **2026-10-22**. Status: **upstream ESP32-C6 support; valuable POSIX-oriented comparison; Rust remains experimental**. No firmware was built or flashed and no performance was measured.

## Identity and references

- [Project and docs](https://nuttx.apache.org/) · [Kernel GitHub](https://github.com/apache/nuttx) · [Applications GitHub](https://github.com/apache/nuttx-apps) · [Issues](https://github.com/apache/nuttx/issues).
- Core licensing: [Apache-2.0](https://github.com/apache/nuttx/blob/master/LICENSE); dependencies require separate review.
- NuttX provides a preemptive RTOS with POSIX-style interfaces, pthreads, filesystems, device files, sockets, and optional tickless operation. Its [design goals](https://nuttx.apache.org/docs/latest/introduction/about.html) emphasize standards compatibility on MCUs. It is not a Linux distribution. Generic flat/protected/kernel build options do not establish that every mode works on C6.

## What actually runs on C6

The upstream [C6 port](https://nuttx.apache.org/docs/latest/platforms/risc-v/esp32c6/index.html) includes boot/build instructions and a peripheral matrix. The [DevKitC-1 page](https://nuttx.apache.org/docs/latest/platforms/risc-v/esp32c6/boards/esp32c6-devkitc/index.html) supplies ready configurations including `nsh`, `ble`, `wifi`, `coremark`, `tickless`, `autopm`, and `mcuboot_update_agent`. This is concrete upstream board support. The **ESP32-C6-DevKitC-1-N8** matches this family and the observed 8 MB flash. The [hardware inventory](../../../../docs/hardware.md) supplies the two USB routes and board fixtures; the exact NuttX build, console selection and peripheral behavior still need qualification.

| Area | C6 evidence and boundary |
| --- | --- |
| CPU | Main RISC-V port; LP coprocessor loading and bare-metal LP applications are documented separately. |
| Basic I/O | Matrix reports GPIO, UART/LPUART, USB serial, timers, SPI, I2C including LP/master/slave, I2S, ADC, PWM, CAN/TWAI, DMA, RMT, and watchdog. |
| Wi-Fi | Supported; C6 `wifi` configuration exists. |
| BLE | Supported; `ble` configuration documents scanning nearby devices. |
| IEEE 802.15.4 / Thread | **Not established** by the C6 matrix or board configurations reviewed. The introduction's radio description is a silicon capability, not driver evidence. |
| Crypto | AES, SHA, RNG supported; matrix marks ECC, HMAC, RSA, XTS unsupported. |

See the [C6 support matrix](https://nuttx.apache.org/docs/latest/platforms/risc-v/esp32c6/index.html#peripheral-support). It also marks SDIO and SPIRAM unsupported. Temperature reporting is inconsistent: the matrix has an unsupported temperature row while the board page documents `temp`; resolve against the pinned source/build before using that feature.

## Rust reality

The official [Rust guide](https://nuttx.apache.org/docs/latest/guides/rust.html) explicitly labels support **experimental**. It lists RISCV32, requires nightly Rust and specific OS options, and recommends **`rv-virt:nsh`** as its verified test configuration. It does not validate ESP32-C6 Rust. The [Rust examples](https://nuttx.apache.org/docs/latest/applications/examples/rust/index.html) cover bare-metal hello, hello, and Slint.

For our Rust preference, this is a feasibility gate: prove a Rust app under C6 NSH, then timers, heap, threads, and C interoperability. Avoid equating RISCV32 support with a demonstrated C6 Rust `std` application. Keep application kernels in portable Rust if the ABI works; discuss any C shim or C-only baseline before implementation.

## Why evaluate it

**Pros, assessment:** the strongest candidate here for familiar Unix/POSIX patterns on a small MCU; useful for reusing portable libraries, exercising sockets/filesystems, and developing a device with an interactive diagnostics shell. Existing C6 configurations provide a broad test menu.

**Cons, assessment:** its abstraction and libc surface can add footprint versus a deliberately minimal executor; the app/kernel configuration and experimental Rust stack create integration work; C6 802.15.4 is not proven by reviewed evidence. Best fit: products with substantial file, socket, thread, or command-shell needs.

## Build-only starting point

Obtain matching pinned `nuttx` and `apps` checkouts as sibling directories, and prepare the [official build prerequisites](https://nuttx.apache.org/docs/latest/quickstart/install.html). From `nuttx`, with a suitable RISC-V toolchain already on PATH:

```sh
./tools/configure.sh esp32c6-devkitc:nsh
make -j8
```

The C6 documentation supplies this configure/build sequence; `-j8` replaces its Linux-only `nproc` example for this Mac. **Not executed here.** First build in an isolated checkout; `configure.sh` changes build configuration. Do not run flash erase commands from upstream tutorials during research.

## Benchmark and lifecycle questions

- Compare NSH-disabled minimal firmware with NSH-enabled diagnostics. Avoid charging optional shell overhead to the scheduler.
- Use CoreMark only for compiler/CPU context; add pthread handoff, semaphore latency, VFS overhead, socket payload throughput, idle/wake energy, and long-run heap behavior.
- Test the documented [MCUboot OTA flow](https://nuttx.apache.org/docs/latest/platforms/risc-v/esp32c6/index.html#mcuboot-and-ota-update), including image confirmation and rollback, as its own workload. Simple Boot and MCUboot are different boot configurations.
- Flash encryption is documented, including virtual-eFuse testing. Actual eFuse writes are irreversible: production provisioning belongs to a separate decision, not a benchmark run.
- Record both kernel and apps revisions, configuration, radio-library dependencies, and toolchain. Pin `latest` references to an actual release/commit before building; old console banners in examples are not current version evidence.

Source snapshot observed during research: NuttX [`ef611627a5cc5100c7c62d00985f1c6cd85d3e9a`](https://github.com/apache/nuttx/blob/ef611627a5cc5100c7c62d00985f1c6cd85d3e9a/Documentation/platforms/risc-v/esp32c6/index.rst). This records research provenance, not a validated kernel/apps benchmark pair.
