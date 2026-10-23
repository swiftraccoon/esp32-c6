# Rust support audit for ESP32-C6

Checked **2026-10-22** from primary documentation and repository source. This audit establishes upstream evidence, not a successful local build or hardware test.

## Toolchain and runtime distinctions

| Route | C6 target | Meaning |
| --- | --- | --- |
| esp-hal / Embassy / Ariel / RTIC / custom firmware | `riscv32imac-unknown-none-elf` | `no_std`; stable Rust is the normal C6 route, with components/features determined by the pinned project |
| Rust on ESP-IDF | `riscv32imac-esp-espidf` | Rust `std` built for ESP-IDF; underlying OS remains ESP-IDF FreeRTOS |

The official Rust target documentation identifies the C6 `espidf` target and requires building its standard library. Do not reuse old custom target spellings or imply that `std` is a separate OS. The C6 is RISC-V; instructions for the Xtensa `esp` compiler are not automatically applicable. [Rust target documentation](https://doc.rust-lang.org/rustc/platform-support/esp-idf.html), [HAL target table](https://docs.rs/crate/esp-hal/latest).

Embassy executors, esp-rtos threads and Ariel threads can share low-level Rust HAL/radio components. Record executor, scheduler, HAL, radio, protocol stack and bootloader individually in each result. Match dependencies where possible; otherwise report a whole-stack comparison rather than claiming an isolated kernel measurement.

## Wireless support is layered

| Layer | Verified evidence and boundary |
| --- | --- |
| Wi-Fi / BLE / ESP-NOW | esp-radio's C6 row lists implementations, including Wi-Fi/BLE coexistence. A check mark means a driver exists, not every advertised silicon capability is validated. [Radio README](https://github.com/esp-rs/esp-hal/blob/main/esp-radio/README.md) |
| Bluetooth host | esp-radio recommends TrouBLE; controller support is not a complete BLE application/profile stack. [TrouBLE](https://github.com/embassy-rs/trouble), [ESP examples](https://github.com/embassy-rs/trouble/tree/main/examples/esp32) |
| Raw 802.15.4 | C6 driver exists. Current source marks `ieee802154`, BLE and coexistence features unstable and explicitly forbids combining `ieee802154` with Wi-Fi. [Radio feature definitions](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/esp-radio/Cargo.toml) |
| Thread | esp-rs provides async Rust bindings to OpenThread with an esp-radio backend. Its C6 target has prebuilt OpenThread libraries; custom feature builds may require CMake/Ninja/Clang and C-library shims. [Binding capabilities](https://github.com/esp-rs/openthread/blob/main/openthread/README.md) |
| Thread maturity boundary | Binding README lists Thread 1.4, MTD/FTD/SED, no CSL yet, and excludes Thread Border Router functionality. This does not establish certification of our application. [Binding status](https://github.com/esp-rs/openthread/blob/main/openthread/README.md) |
| Concrete Thread entry | `examples/esp` defaults to C6 and includes UDP, embassy-net, DNS and SRP examples with esp-rtos. These sources are stronger evidence than an 802.15.4 check mark. [C6 example manifest](https://github.com/esp-rs/openthread/blob/705c5576ccfd7b417cfa4b23912a13b1b99e01f2/examples/esp/Cargo.toml) |
| Matter | `rs-matter` is an application/connectivity stack, not an OS; OpenThread's README reports integration, but a C6 Matter demonstration and required transport/provisioning must be verified for the selected dependency set. [rs-matter](https://github.com/project-chip/rs-matter), [OpenThread status](https://github.com/esp-rs/openthread/blob/main/openthread/README.md) |

The Wi-Fi/BLE path retains vendor binary components behind Rust wrappers. `no_std` does not mean the entire firmware is written in Rust or source-available. The esp-radio crate's MIT/Apache license does not replace the licenses of bundled driver libraries. [esp-radio vendor dependency explanation](https://docs.espressif.com/projects/rust/esp-radio/1.0.0-beta.0/esp32c5/esp_radio/index.html), [wireless driver libraries](https://github.com/esp-rs/esp-wireless-drivers-3rdparty).

**Benchmark consequence:** run raw 802.15.4 and Wi-Fi separately for this source family; do not promise a single-C6 Rust Wi-Fi/Thread border-router test from hardware capability alone. Compare Thread node behavior with a known peer/border router and record dataset, topology and retransmissions. Ariel or RTIC support must still be proven at their integration layers.

## Additional Rust OS eligibility

| Candidate | Audit result | Implication |
| --- | --- | --- |
| [RTIC](../os/rtic.md) | Official C6 backend and in-tree examples exist | Include radio-free timing trials; current radio integration needs proof |
| [MOS-Rust](../os/mos-rust.md) | Real C6 source and BLE adapter exist; experimental | Optional second-round candidate / custom-scheduler reference |
| [Tock upstream](https://tockos.org/hardware/) | Official hardware list and examined upstream board tree include ESP32-C3 boards; no C6 board found | Not established as runnable on our C6 |
| [tardyp/esp_tock](https://github.com/tardyp/esp_tock) | Examined repository is C6 porting documentation; [port index](https://github.com/tardyp/esp_tock/blob/0e66d2f1804ff19f6b04e55db5e4545ebdf43ebd/README_ESP32C6_PORT.md) leaves boot/process milestones unchecked | A porting plan is not a completed C6 implementation |
| [Hubris](https://github.com/oxidecomputer/hubris) | Examined upstream kernel tree has ARM backend only; [historical RISC-V discussion](https://github.com/oxidecomputer/hubris/discussions/365) does not establish C6 board support | Treat as design inspiration, not a current board benchmark |

Absence from the examined official trees is not a proof that no private or third-party port exists. Evidence needed to promote a new candidate: source revision, exact C6 board/configuration, reproducible build and observed boot/timer/interrupt behavior. Generic “RISC-V supported” is insufficient.

Additional hobby/research leads, including [ThistleOS](https://github.com/wan0net/thistle-os), [project-trobbio](https://github.com/toyo54/project-trobbio), and OpenC6, are evaluated separately in the [research-kernel references](../os/research-kernels.md). Their presence in the survey does not imply the same support maturity as the first-round candidates.

## Source snapshots and reproducibility

Repository heads were obtained from GitHub's API on the research date. These freeze the inspected evidence; **they are not recommendations to benchmark unreleased heads**.

| Repository | Examined commit |
| --- | --- |
| esp-rs/esp-hal | [`0cf41dea`](https://github.com/esp-rs/esp-hal/tree/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5) |
| esp-rs/esp-generate | [`63348938`](https://github.com/esp-rs/esp-generate/tree/63348938c7068400080e619217da80ce6a303ab1) |
| ariel-os/ariel-os | [`13bddaca`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9) |
| rtic-rs/rtic | [`f1797184`](https://github.com/rtic-rs/rtic/tree/f1797184d6168e54c345b17019c237eea70b9ed4) |
| esp-rs/openthread | [`705c5576`](https://github.com/esp-rs/openthread/tree/705c5576ccfd7b417cfa4b23912a13b1b99e01f2) |
| Eplankton/mos-rust | [`a5046fd3`](https://github.com/Eplankton/mos-rust/tree/a5046fd30a8d40adf79e26afe1d708c15286c402) |
| tardyp/esp_tock | [`0e66d2f1`](https://github.com/tardyp/esp_tock/tree/0e66d2f1804ff19f6b04e55db5e4545ebdf43ebd) |
| oxidecomputer/hubris | [`66769102`](https://github.com/oxidecomputer/hubris/tree/6676910269b897577e918075df273ab6744aa514) |
| tock/tock | [`07838d39`](https://github.com/tock/tock/tree/07838d399f90ea6683c6d7a2947deac1754ff1da) |

Before implementation, select compatible released dependency sets per candidate, commit Cargo.lock, record exact `rustc -Vv`, generator/tool versions, board/flash/pins and every Cargo feature. esp-hal explicitly warns that unstable APIs can change in minor releases and recommends pinning that series. The RTIC and MOS examples already use older exact HAL versions, so a single-version mandate may require a separately documented port. [HAL support policy](https://github.com/esp-rs/esp-hal#support-policy).
