# RTIC C6 build feasibility

Executed **2026-10-23**, host aarch64-apple-darwin. Both maintained upstream C6
examples compile with **Rust1.96.0**, without source, manifest, configuration or
lockfile edits. No board operation occurred in this build investigation.
The [qualification research](../research/rtic-qualification.md) records selected
semantics and [the prospective contract](../experiments/rtic-qualification.md)
owns physical acceptance.

## Source, tools and actual dependency input

The isolated checkout is
`.cache/rtic-qualification-2026-10-23/upstream`, detached at
[`f1797184d6168e54c345b17019c237eea70b9ed4`](https://github.com/rtic-rs/rtic/tree/f1797184d6168e54c345b17019c237eea70b9ed4).
Clone/checkout and later `git status --porcelain` logs are retained in the
adjacent `logs` directory. The example's **Cargo.lock is already tracked
upstream** and stayed byte-identical; its SHA-256 is
`bd4038ed0b7441efeaf1c429f3927834f31d1ed7d7cfa07b22d458b3d621deb7`.
No generated replacement lock or dependency upgrade was used.

| Input | Selected value |
| --- | --- |
| Target | riscv32imac-unknown-none-elf; installed Rust1.96.0 target/core |
| Example / framework edition | Example2021; RTIC framework2024 |
| Backend feature | riscv-esp32c6-backend, enabling the RTIC backend and esp32c6-systimer monotonic |
| RTIC family from checkout | rtic/macros2.4.0, monotonics/time3.0.0, common1.2.0 |
| HAL | Exact1.0.0-rc.0 with esp32c6,unstable; preserves upstream integration |
| Console / panic / descriptor | esp-println0.15.0(auto/critical-section), esp-backtrace0.17.0, esp-bootloader-esp-idf0.2.0 |
| Active PAC versions | HAL uses esp32c6 0.21.0; RTIC/app/monotonic0.22.0; ROM dependency0.23.2 |
| Active RISC-V crate versions | HAL0.12.1; RTIC/monotonic0.16.1 |
| Upstream Cargo flags | force-frame-pointers; target selected in example config |
| License | MIT OR Apache-2.0; upstream notices preserved |

The package is its own `[workspace]`. The parent repository's release LTO and
codegen-unit overrides do not become this example's profile. Builds use its
unmodified release defaults, not this evaluation repository's matched benchmark
profile. Frame pointers may affect performance. Its toolchain file says stable
plus rust-src; these commands select the already installed numeric compiler
explicitly. The configuration contains an unstable build-std table, but the
successful stable commands supplied no `-Z`/RUSTC_BOOTSTRAP and made no claim to
rebuild target core/alloc. RTIC's rolling stable policy is not a tested numeric
MSRV guarantee.

## Executed builds

From the isolated `upstream/examples/esp32c6` directory:

```sh
cargo +1.96.0 build --release --example monotonic --features riscv-esp32c6-backend
cargo +1.96.0 build --release --locked --example sw_and_hw --features riscv-esp32c6-backend
cargo +1.96.0 build --release --locked --example monotonic --features riscv-esp32c6-backend
```

All three passed. The first command used the existing tracked lock and left it
unchanged; the final explicit locked command also passed. The first build log
reports13.08s; sw_and_hw0.64s. Those elapsed build times include dependency/cache
conditions and are not measured developer-effort or performance comparisons.
A later verbose locked monotonic build also passed without recompilation.
`cargo tree --locked`, its feature expansion, package metadata, rustc/cargo-vV,
input/output hashes and clean source status are retained alongside the logs.

| ELF | File bytes | SHA-256 |
| --- | ---: | --- |
| monotonic | 146432 | 12dc8bd4f079b9acdfda34db652e422cafddd9869c4b92493274cb56b47645a9 |
| sw_and_hw | 159264 | 289bba0fdddc8ccbaf7a92394764fe5169445af9fdae04f813747ccf93032610 |

ELF file size is not flash usage. Allocated-section inventories and monotonic
ELF/program headers are retained. Exact application markers and the pinned
backtrace diagnostics are present in the monotonic ELF. These inspections do
not prove that a board executes them.

## Offline packaging investigation

The intended bootloader is **espflash4.6.0's default C6** binary, rather than
a borrowed IDF benchmark bootloader. Its package VCS revision is
[`c1fb39939c1b5880b2aec7f43063f19689d5b00a`](https://github.com/esp-rs/espflash/tree/c1fb39939c1b5880b2aec7f43063f19689d5b00a/espflash).
The embedded resource is22,016bytes, SHA-256
`bfe7a8a539d06d8da984d42ce02bf985f97d363f03b1513adce8277da11532ee`.
Its retained manifest/README identify generation from IDF release/v6.1, without
an exact IDF build commit or a watchdog-off override. The resource binary is
the frozen executable input; the manifest's branch name does not provide a
more exact source identity than it actually records.

Offline espflash save-image passes with explicit C6,DIO,80MHz,8MB,40MHz crystal
settings. A merged, unpadded image supplies the default bootloader and default
partition table. Extracted partition table:

```csv
# Name,Type,SubType,Offset,Size,Flags
nvs,data,nvs,0x9000,0x6000,
phy_init,data,phy,0xf000,0x1000,
factory,app,factory,0x10000,0x7f0000,
```

No table was borrowed from the IDF experiment. The default app output is18,000
bytes. Its checksum/digest verify, but esptool5.1.0 image-info exposes the
existing espflash C6 frequency-nibble problem (`0xf`) and zero app-descriptor
ELF hash. The initially extracted bootloader likewise has the invalid frequency
nibble after espflash's header rewrite. These are preserved provisional outputs,
not finalized flash inputs.

Two esptool5.1.0 conversion attempts, one ordinary and one with documented
`--use-segments`, both fail with exit2 because the unchanged old-HAL ELF has
flash segments at0x42000020 and0x42001248 within one64KB mapping. Full diagnostic
logs are retained; neither attempt produced an application output. No linker
port or upstream source edit is hidden behind these failures. Final packaging
must preserve these attempts and separately record any authorized header/hash
correction before the physical input freeze.

The targeted upstream check found latest release4.6.0 and inspected development
[`ac27e1f463527929f133e2ffb34c481fa5a83e53`](https://github.com/esp-rs/espflash/tree/ac27e1f463527929f133e2ffb34c481fa5a83e53).
The same descriptor-section detection and C6 frequency table remain there;
the inspected CLI offers no correction for this old descriptor layout. A
separate, explicitly authorized Rust helper was therefore created in
`.cache/rtic-qualification-2026-10-23/packaging-helper`, preserving the inputs.
It pins dependencies, validates original XOR/digest/header/segments, binds every
initialized allocated ELF byte even across split RAM image segments, requires
the actual descriptor mapping/magic and zero old ELF hash, then changes only
the documented frequency field, app ELF hash, XOR and appended digest. Outputs
are exclusive new files; no upstream source or ELF instruction is changed.
Its README retains all54 API dispositions and exact format sources.

The final helper passes three unit tests, all-target Clippy with warnings denied,
and rustfmt. Actual CLI controls reject a wrong ELF, wrong app/boot role and
existing output. Initial helper compile/Clippy diagnostics and its original
too-strict single-segment `.trap` binding rejection remain in the logs. Binding
was corrected to prove complete bytes across unique adjacent mappings, without
omitting the section. The final helper executable SHA-256 is
`967a7241bf4396befad1ffc2ed75d60a4a8d817b1e601280eb7f213cec44c2bd`.

| Final new offline artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| monotonic-final.bin | 18000 | b10d6dfdc9f84f0e02f3007e198fe38ea3ae02ca47a3222e515073cfc9cbfb29 |
| bootloader-final.bin | 22016 | b1d7d018cc62f3fb30b901d5bacf95f58ab879f70ebec50d231f092e976c9318 |
| partition-table.bin | 3072 | 420931e3f5af072d899b5357c043fd9f283bf165fc9df6113b6d43d82b48a06d |

Independent esptool5.1.0 image-info verifies both final XOR checksums and appended
SHA256 values, C6 chip identity, DIO/8MB and a nominal80m label; the app's embedded
ELF hash equals the original `12dc8b...645a9` identity exactly. C6 header encoding0
can denote80MHz or40MHz based on MSPI source, so this is not runtime flash-clock
readback. The selected final files are staged in the adjacent cache `artifacts`
directory for reviewed archival/input freeze; no flash command was run here.

No board operation occurred during these builds or packaging checks. The first
Cargo build updated the crates.io index and downloaded dependencies; dependency
provisioning used the network. Image generation/inspection and the correction
helper operate on local files without a device. Image validity and compilation do
not supply UART output, watchdog survival, timer behavior, runtime clock
readback, GPIO IRQ behavior or physical qualification. The unmodified
monotonic startup omits HAL init, so watchdog inheritance is an explicit
qualification risk rather than assumed disabled.
The prospective collector guard was extended from 5 to 12 seconds after the final
marker, within the 45-second ceiling, before input freeze or board capture.
This source-informed change avoids finishing near the nominal 9-second startup
watchdog boundary; it changes no ELF or packaged binary and preserves earlier
5-second diagnostic source as historical evidence.
