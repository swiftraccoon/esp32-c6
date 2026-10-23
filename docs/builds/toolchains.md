# Initial benchmark toolchains

Implementation uses Rust **1.96.0**, host `aarch64-apple-darwin`. This compiler was already installed on the development host. The repository selects language and formatting edition **2024** and pins the compiler in `rust-toolchain.toml`. Both firmware families and the host collector use this compiler so shared-code measurements have the same Rust compiler version.

Host tools use `aarch64-apple-darwin`. Bare-metal and Embassy firmware use `riscv32imac-unknown-none-elf`; ESP-IDF uses `riscv32imac-esp-espidf` with a separately built Rust standard library. Rust source, formatter, linter and the bare-metal target are explicit components. Component availability and final build results belong in the associated build records.

The shared dependency-free workload uses only features available with edition 2024's Rust 1.85 minimum. Individual application MSRVs follow their dependencies; their toolchain remains pinned to 1.96.0. No project license or authorship is inferred from the upstream dependencies. These unpublished packages are local evaluation code.

Release configurations use optimization level 3, thin LTO, one codegen unit and aborting panics. Native vendor C toolchain settings are recorded separately; this is a whole-stack comparison, not an assertion that Rust and C compilation are identical.

The flasher version selected for the first `no_std` builds is **espflash 4.6.0**, installed under the ignored repository `.tools/` directory. The existing esptool is **5.1.0**. Firmware dependency versions, IDF revision, SDK configuration and exact commands are recorded in the platform build records and lockfiles.

ESP-IDF standard-library compilation requires Rust's `build-std` support and `ldproxy`; any unstable build support is scoped to that firmware rather than changing the host or shared-crate compiler. [Rust ESP-IDF target requirements](https://doc.rust-lang.org/rustc/platform-support/esp-idf.html)

Rust 1.99.1 was rejected for this comparison because its ESP-IDF standard-library build failed on an unavailable `libc::AT_FDCWD` reference before compiling the application. The common compiler version is selected for compatible stacks, not recency alone. Detailed diagnostics belong in the IDF build record.
