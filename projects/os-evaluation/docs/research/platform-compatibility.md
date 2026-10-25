# Platform compatibility and maintenance audit

Research checked **2026-10-23**, before resuming adapter implementation. This
closes the bounded source/maintenance review for the next Rust candidates. It
does not change existing pins, establish a new build, or qualify a radio or
product service on the attached board.

The principal correction is that a working benchmark pin, a published release,
and a development checkout with the same package version are different inputs.
The initial research distinguished OSs and driver layers well, but did not give
enough attention to this distinction or to the support behind Rust bindings.

## Evidence and inspected revisions

Primary repositories and release/tag metadata were read through the GitHub
connector. Tags below are resolved to **commit** IDs, not annotated-tag object
IDs. A workflow definition proves configured coverage, not that its latest run
passed or every driver was exercised.

| Source | Inspected commit / release |
| --- | --- |
| Current Rust IDF monorepo | [`2ac247eca86caecb4cd626f6a1b127265f2fd012`](https://github.com/esp-rs/esp-idf/tree/2ac247eca86caecb4cd626f6a1b127265f2fd012) |
| esp-idf-sys 0.38.1 | [`9c5eb775f4a347cc3ebb9fa3fcfb9fa7aca915f7`](https://github.com/esp-rs/esp-idf/tree/9c5eb775f4a347cc3ebb9fa3fcfb9fa7aca915f7/esp-idf-sys) |
| esp-idf-hal 0.47.0 | [`0d16dcaf8411735c4da9bc80fdc2f7da33f5dc4f`](https://github.com/esp-rs/esp-idf/tree/0d16dcaf8411735c4da9bc80fdc2f7da33f5dc4f) |
| esp-idf-svc 0.53.0 | [`9f172692c8e7865cd8392537cfbdeb48245f5621`](https://github.com/esp-rs/esp-idf/tree/9f172692c8e7865cd8392537cfbdeb48245f5621/esp-idf-svc) |
| Current esp-hal family | [`0cf41deab8a8edd97a48d079b904c8f87dd9e9e5`](https://github.com/esp-rs/esp-hal/tree/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5) |
| esp-hal 1.2.0 | [`dbd951f78c1276461ea3050c2ba71f552a3508ef`](https://github.com/esp-rs/esp-hal/tree/dbd951f78c1276461ea3050c2ba71f552a3508ef) |
| esp-rtos 0.4.0 | [`63e97be553f435f29171f7ae5f37a41ad737a75a`](https://github.com/esp-rs/esp-hal/tree/63e97be553f435f29171f7ae5f37a41ad737a75a) |
| Published esp-radio 1.0.0-beta.1 | [`f769fcd27056b0837febe4209f3bae78ed764561`](https://github.com/esp-rs/esp-hal/tree/f769fcd27056b0837febe4209f3bae78ed764561) |
| Ariel development checkout already qualified | [`13bddaca1c2bcd51489c044b80408e917d5946d9`](https://github.com/ariel-os/ariel-os/tree/13bddaca1c2bcd51489c044b80408e917d5946d9) |
| Ariel latest version tag, v0.6.0 | [`cee68033f87cded63e8d62bf751d7add9b0a57ed`](https://github.com/ariel-os/ariel-os/tree/cee68033f87cded63e8d62bf751d7add9b0a57ed) |
| RTIC v2.4.0 | [`f1797184d6168e54c345b17019c237eea70b9ed4`](https://github.com/rtic-rs/rtic/tree/f1797184d6168e54c345b17019c237eea70b9ed4) |

The [toolchain record](../builds/toolchains.md), platform build records and
archived lockfiles remain authoritative for our actual local builds.

## Working pins versus next qualification choices

| Stack | Locally established input | Source-supported next choice and limits |
| --- | --- | --- |
| Rust std / IDF | Rust 1.96.0; `riscv32imac-esp-espidf`; unmodified IDF 5.5.6; sys **0.37.2**; embuild 0.33.5; std/binstart, no radio | Keep this tuple for native-operation decomposition. New wrappers **hal 0.47.0 / svc 0.53.0 require sys 0.38**; they cannot simply be added to the pinned sys 0.37.2 baseline. A separate upgrade probe is needed. |
| Wireless explorer, 2026-10-24 | Rust 1.96.0; IDF 5.5.6; sys/hal/svc from upstream Git `a209052f378be61dd0f2d566b7fbe0f9593b0cd5`; embuild 0.33.7; libclang 21.1.8 | Target build, UART control and passive Wi-Fi surveys observed. Published svc 0.53.0 omitted the SDK's new Wi-Fi task stack-size field and returned error 257; the pinned upstream fix supplies it. [Inputs, failed attempts and scope](../../../wireless-explorer/results/control-2026-10-24/README.md). |
| Direct HAL / esp-rtos / Embassy | Rust 1.96.0; `riscv32imac-unknown-none-elf`; HAL **1.2.0** with `esp32c6,unstable`; RTOS **0.4.0** with `esp32c6,embassy`; executor **0.10.0**, sync **0.8.0**, time **0.5.1**; boot descriptor **0.6.0** | Keep the released tuple for existing Embassy measurements. Released RTOS 0.4.0 does **not** export the general application `ThreadSpawner` interface found at development HEAD. A native application-thread adapter needs a deliberate different source/API choice first. |
| Ariel development input | Rust **1.99.1**, laze **0.1.39**, development commit above; workspace version **0.6.0**; HAL **1.1.1** fork, executor **0.10.0**, sync **0.8.0**, time **0.5.1** fork; UART/log contexts | Continue this input if testing current development integration, explicitly retaining its differing compiler/HAL/build settings. Async/thread startup alone passed. |
| Ariel released alternative | **Not locally built** | Actual tag **v0.6.0** declares Rust **1.95**, HAL **~1.0.0**, executor **0.9.1**, sync **0.7.2**, time **0.5.1**. Its lock resolves forked HAL **1.0.0**, radio **0.17.0**, boot descriptor **0.4.0**. It offers a possible common-compiler route but older components; compatibility with Rust 1.96.0 is not a build result. |
| RTIC | **Not locally built** | Use the unmodified **v2.4.0 C6 example** first: HAL **=1.0.0-rc.0**, PAC **0.22.0**, boot descriptor requirement **0.2.0**, RTIC C6 backend and C6 SYSTIMER monotonic. Do not replace its HAL as a routine version refresh. |

Local evidence: [IDF build](../../firmware/esp-idf/README.md),
[HAL build](../../firmware/esp-hal/README.md), [clocked adapters](../../firmware/esp-hal/README.md),
[Ariel qualification](../builds/ariel-os.md). Release evidence:
[IDF HAL manifest](https://github.com/esp-rs/esp-idf/blob/0d16dcaf8411735c4da9bc80fdc2f7da33f5dc4f/Cargo.toml),
[IDF service manifest](https://github.com/esp-rs/esp-idf/blob/9f172692c8e7865cd8392537cfbdeb48245f5621/esp-idf-svc/Cargo.toml),
[Ariel release manifest](https://github.com/ariel-os/ariel-os/blob/cee68033f87cded63e8d62bf751d7add9b0a57ed/Cargo.toml),
[Ariel release lock](https://github.com/ariel-os/ariel-os/blob/cee68033f87cded63e8d62bf751d7add9b0a57ed/Cargo.lock),
[RTIC C6 manifest](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/Cargo.toml).

The released Ariel manifest is materially different from the development tree
we ran. **“Ariel requires Rust 1.98” describes that inspected development input,
not every current Ariel release.** Its published-tag alternative should have
been considered before treating the compiler mismatch as unavoidable.

## IDF: vendor SDK and community Rust integration

The canonical Rust source is now **[esp-rs/esp-idf](https://github.com/esp-rs/esp-idf)**.
The old sys/svc repositories announce their move and are read-only; HAL's old
URL redirects. Histories are retained, crate release tags are prefixed, and
examples now share the monorepo root. This is a relocation, not evidence that
Rust IDF support ended. [Pinned migration/ownership explanation](https://github.com/esp-rs/esp-idf/blob/2ac247eca86caecb4cd626f6a1b127265f2fd012/README.md).

That README distinguishes community Rust wrappers from paid Espressif support
for esp-hal and says the wrappers lack HIL tests. The inspected
[CI definition](https://github.com/esp-rs/esp-idf/blob/2ac247eca86caecb4cd626f6a1b127265f2fd012/.github/workflows/ci.yml)
does compile/link/clippy matrices including `riscv32imac-esp-espidf`, with C6
the default MCU for that target and explicit additional MCUs. It lists SDK
5.3.6, 5.4.4, 5.5.5, 6.0.3 and 6.1. This is development-head compile coverage;
it does not certify every released wrapper, our newer 5.5.6 patch combination,
or physical driver operation. CI toolchains are also not one fixed Rust 1.96.0.

The released sys 0.38.1, hal 0.47.0 and svc 0.53.0 manifests declare **Rust 1.82**.
This package MSRV is not a guarantee that any newer Rust standard-library
source builds for every IDF version. Our actual failures already demonstrate
that distinction: Rust 1.99.1 failed `libc::AT_FDCWD`; sys 0.38.1 with the selected
5.5.6 setup failed generated `_reent` field accesses. No fix was established
by this research. Sys 0.38's new libc compatibility checks and IDF 6 support
also mean its upgrade is more than substituting a version number.
[Pinned sys changelog](https://github.com/esp-rs/esp-idf/blob/2ac247eca86caecb4cd626f6a1b127265f2fd012/esp-idf-sys/CHANGELOG.md).

IDF **6.1** is the newer stable release line inspected; **5.5.6** is the latest
5.5 patch observed. Latest publication time alone does not select a line.
IDF's [support policy at our SDK](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/SUPPORT_POLICY.md)
provides 30 months from a major/minor's first stable release: 12 service months,
then 18 maintenance months; previews lack that support promise. This policy
belongs to IDF, not an equivalent support contract for the Rust wrappers.
[6.1 release](https://github.com/espressif/esp-idf/releases/tag/v6.1),
[5.5.6 release](https://github.com/espressif/esp-idf/releases/tag/v5.5.6).

**Upgrade path:** preserve current results, then separately probe sys 0.38.1
and the released wrapper family against a selected supported IDF line and a
recorded compiler. Start with unmodified std/thread/timer examples and inspect
generated configuration/bindings. Requalify C6 behavior and any newly required
remote SDK components before adding networking or replacing the benchmark
baseline. A dependency manifest permits the tuple; successful linking and
physical qualification must still establish it.

## esp-hal, esp-rtos and Embassy: release alignment and radio contract

Published HAL **1.2.0**, RTOS **0.4.0**, and radio **1.0.0-beta.1** declare
**Rust 1.95.0**. HAL 1.2.0 and RTOS 0.4.0 already work with our compiler 1.96.0.
The inspected HEAD still advertises HAL/RTOS versions 1.2.0/0.4.0 but has
changed source and dependencies; radio HEAD advertises **beta.2**, whereas the
published release inspected is **beta.1**. GitHub's beta.1 `prerelease: false`
metadata does not remove the version's prerelease suffix.
[HAL release manifest](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/esp-hal/Cargo.toml),
[RTOS release manifest](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/Cargo.toml),
[radio release manifest](https://github.com/esp-rs/esp-hal/blob/f769fcd27056b0837febe4209f3bae78ed764561/esp-radio/Cargo.toml),
[radio release](https://github.com/esp-rs/esp-hal/releases/tag/esp-radio-v1.0.0-beta.1).

HAL backports fixes only to the latest minor in a major; its unstable APIs can
break on minor updates but are promised not to break on patch updates. The
RTOS MSRV policy permits compiler requirements to change even on a patch.
Pinning prevents accidental changes; it does not create long-term maintenance
of an older series. [HAL policy](https://github.com/esp-rs/esp-hal/blob/dbd951f78c1276461ea3050c2ba71f552a3508ef/README.md),
[RTOS policy](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/README.md).

The current [chip matrix](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/.github/chips.json)
assigns C6 both general and radio hardware runners. The
[HIL workflow](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/.github/workflows/hil.yml)
builds and runs device tests, but radio runs are excluded from its general HIL
merge gate. These are stronger coverage mechanisms than a compile-only matrix,
not a pass certificate for our release features or board.

For a next Wi-Fi/BLE probe, the published radio manifest targets HAL **~1.2.0**,
RTOS **0.4.0** in examples, RTOS driver **0.4.2**, and sync **0.8**. Its default
allocator feature and unstable BLE/coexistence APIs need explicit selection.
Our current radio-free Cargo.lock has not qualified this larger combination.

Wi-Fi/BLE require a preemptive scheduler initialized before radio startup.
The driver contract covers task lifecycle, ISR yield, IPC, clocks and timers;
linking two scheduler implementations is not supported. **Raw IEEE 802.15.4
does not itself require that scheduler**, so do not generalize the Wi-Fi/BLE
requirement to every radio path. [Published radio initialization contract](https://github.com/esp-rs/esp-hal/blob/f769fcd27056b0837febe4209f3bae78ed764561/esp-radio/src/lib.rs),
[RTOS adapter contract](https://github.com/esp-rs/esp-hal/blob/f769fcd27056b0837febe4209f3bae78ed764561/esp-radio-rtos-driver/src/lib.rs).

The released RTOS [public library](https://github.com/esp-rs/esp-hal/blob/63e97be553f435f29171f7ae5f37a41ad737a75a/esp-rtos/src/lib.rs)
exports `CurrentThreadHandle`, but not a general application `ThreadSpawner`.
Development HEAD's [thread module](https://github.com/esp-rs/esp-hal/blob/0cf41deab8a8edd97a48d079b904c8f87dd9e9e5/esp-rtos/src/thread.rs)
does expose that application API despite retaining the same package version.
The radio adapter's task-creation hooks are not a substitute public application
thread API. This is a concrete example of why package-version equality cannot
establish source-interface equality.

**Next-adapter choice:** keep the proved release tuple for Embassy, and treat
the native esp-rtos application-thread adapter as blocked on a deliberate
source/API choice. Options include qualifying the pinned development thread
API or using Ariel's maintained thread tests first; neither has been selected
or newly qualified here. Before connecting RTIC or a custom executor to radios,
resolve ownership of the scheduler, interrupt routing, timer and allocator in
a small source/design record. A new kernel is not necessary to answer that
integration question.

## Ariel: released versus development input and fork ownership

The qualified development checkout explicitly raises its MSRV to **1.98**;
our local compiler **1.99.1** passed. The v0.6.0 release instead declares **1.95**.
Both permit MSRV increases in patch updates. Choose either input deliberately;
do not mix their manifests, locks or support claims.
[Development policy](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/README.md),
[release policy](https://github.com/ariel-os/ariel-os/blob/cee68033f87cded63e8d62bf751d7add9b0a57ed/README.md).

Our development hello images resolve Ariel's HAL fork
`ce9bcf390ab6b7abc8ae3bc4989360a493c354e7`, plus Embassy time fork
`3c1b5fbf28237ea7144ae970c3aec11c38023c3f` and embedded-HAL fork
`35b4b7a7f612c4d9bfe13c595d52172a5f8b9bbb`. The release lock instead uses
HAL fork `531c629afdd80ea464682ce7f4db8baed97967a6`. These are upstream Ariel
choices, **not local patches**; they still have upgrade and attribution costs.
An upstream workspace lock contains unrelated-board packages, so total Git
dependencies cannot be charged to a C6 image without its active dependency tree.

The [support matrix](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/doc/support_matrix.yml)
lists C6 DevKitC-1 as Tier 1, including Wi-Fi/BLE; persistent storage says it
requires partitioning support. The [tier policy](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/book/src/hardware-functionality-support.md)
allows regular automatic **or manual** testing. The inspected
[build workflow](https://github.com/ariel-os/ariel-os/blob/13bddaca1c2bcd51489c044b80408e917d5946d9/.github/workflows/build.yml)
includes the C6 builder and compiler 1.98; its self-hosted runners are not by
themselves proof that a physical C6 radio test ran. No per-feature current HIL
result was verified here.

**Next-adapter choice:** first run Ariel's existing native benchmarks under one
declared input, as selected in [benchmark reuse](../../../../docs/research/benchmark-reuse.md). A common
Rust payload can then compare whole-stack behavior with compiler/HAL/placement
differences disclosed. Do not port Ariel to HAL 1.2 merely to align a table
before testing whether that change is necessary and supported.

## RTIC: real C6 support with an older integration tuple

RTIC **2.4.0** and macros **2.4.0**, monotonics **3.0.0**, and sync **1.6.0**
were released in the inspected October 21 commit. The C6 example selects the
`riscv-esp32c6-backend` and `esp32c6-systimer` feature route. Its toolchain file
says stable plus rust-src; its Cargo configuration builds core/alloc and forces
frame pointers. There is no numeric `rust-version` in the inspected RTIC,
macros or monotonics manifests; do not invent a tested minimum from “stable.”
The main crate's edition 2024 is a language floor, not a full dependency MSRV.
[C6 toolchain](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/rust-toolchain.toml),
[C6 configuration](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/examples/esp32c6/.cargo/config.toml),
[crate manifest](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/rtic/Cargo.toml).

The [build workflow](https://github.com/rtic-rs/rtic/blob/f1797184d6168e54c345b17019c237eea70b9ed4/.github/workflows/build.yml)
includes C6 in example compile/clippy. Its QEMU run matrix does not include C6;
no physical C6 CI evidence was established. RTIC's maintained backend is useful
evidence, but the old HAL release candidate and PAC 0.22 are the actual example
integration. They do not establish compatibility with our HAL 1.2/RTOS/radio
stack. No fixed LTS/backport promise was established in these inspected sources.

**Next-adapter choice:** unmodified C6 example build/boot/timer/priority
qualification on a recorded compiler first. Keep networking a separate gate.
Describe a deliberate HAL/runtime integration change as a port, with its own
lockfile and regression evidence, if later justified.

## Broader kernels: retained feasibility gates

These remain useful choices; this audit does not repeat the complete source
survey or claim a newly validated release/toolchain tuple for them.

| Candidate | Current research boundary | What must close before an authored benchmark adapter |
| --- | --- | --- |
| [Zephyr](../os/zephyr.md) | C6 kernel/board source exists; inspected Rust platform list lacks C6. Stable/LTS choice must freeze west modules and radio blobs. | Native maintained suite can qualify the kernel independently; separately prove Rust C6 compile/link, timer/thread/IRQ and console under a selected release. |
| [NuttX](../os/nuttx.md) | C6 configurations exist; official Rust guide is experimental and its demonstrated board is rv-virt. | Select matching kernel/apps releases, then establish C6 Rust ABI, timer/thread/heap and board behavior; do not infer it from generic RV32. |
| [RIOT](../os/riot.md) | C6 exists, but inspected RV32 Rust feature is disabled because of C2Rust/Clang constraints. | Resolve the actual binding/toolchain blocker or document an agreed language fallback before implementing a benchmark. |

Their exact earlier inspected commits and primary evidence remain in the linked
sheets. A generic C6 kernel pass must not become an inferred Rust-application
pass. A language fallback remains subject to the existing Rust agreement.

## Closure and remaining work

The canonical-source migration, released-versus-development alternatives,
documented compiler policies, meaningful CI/HIL distinctions and immediate
version conflicts are now researched enough to choose **bounded qualification
probes**. No dependency upgrade or new physical pass was performed here.

Before the next code change, its brief must select one exact input above,
identify the native/API semantics and active features, and state the smallest
compile/boot/timer/interrupt checks that qualify that adapter. Existing radio-free
pins can be reused without first solving every product service.

Before an adoption decision, extend this record with actual upgrade effort,
active dependency/license/source inventory, required hardware capabilities,
recovery/update/storage behavior and power policy. Source availability,
redistribution terms and technical replaceability are separate questions.
Fixed pins preserve experiments; a maintained product also needs a viable
security-update and regression path.
