# IDF accounting diagnostic build

Built **2026-10-23** on the existing Apple Silicon macOS host. This record covers
compilation, configuration, ABI and binary preparation. Physical qualification
has its own retained attempts and does not follow from a successful build.

The [contract](../experiments/idf-accounting-qualification.md),
[source-method review](../research/idf-accounting-method.md),
[firmware review](../../firmware/esp-idf/docs/accounting.md) and
[host review](../../tools/benchctl/docs/api.md) define the question and limits.

## Isolated configuration and commands

The existing IDF 5.5.6/sys 0.37.2 dependency tuple and Rust 1.96.0 target
`riscv32imac-esp-espidf` remain unchanged. The new `accounting-probe` required
feature/binary uses the normal defaults plus five statistics defaults. Separate
target/configuration paths preserve historical SDK outputs and image pins.
No dependency, SDK patch, C or assembly shim was added.

From `firmware/esp-idf`, the environment is:

```sh
env RUSTC_BOOTSTRAP=1 \
  LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
  PATH="$PWD/.tools/bin:$PATH" \
  CARGO_TARGET_DIR="$PWD/.cache/accounting-target" \
  ESP_IDF_TOOLS_INSTALL_DIR="custom:$PWD/.embuild/espressif" \
  IDF_PATH="$PWD/.embuild/espressif/esp-idf/v5.5.6" \
  ESP_IDF_SDKCONFIG="$PWD/.cache/sdkconfig-accounting" \
  ESP_IDF_SDKCONFIG_DEFAULTS="$PWD/sdkconfig.defaults;$PWD/sdkconfig.accounting.defaults" \
  cargo +1.96.0 build --release --locked --bin accounting-probe \
    --features accounting-probe
```

Absolute SDK/defaults paths matter here: the upstream builder infers a
workspace from the target path. An initial isolated-target setup interpreted
relative paths beneath `.cache` and started downloading another tools tree.
That setup was intentionally terminated before a firmware build completed;
its log is retained separately. Reusing the actual installed tools and absolute
configuration/source paths resolved preparation. This is not a C6 Rust
incompatibility or an attempted board run.

The corrected fresh build completed in **51.61 s**, with five unused-helper
warnings: this binary reuses clock configuration and public microsecond reads
from the queue timer module but does not use its raw-tick workload API. A
narrow module-local `dead_code` allowance explains that reuse. No global lint
suppression was added. The final checks below passed cleanly afterward:

| Check under the environment above | Outcome |
| --- | --- |
| `cargo +1.96.0 fmt --all -- --check` | Passed. |
| `cargo +1.96.0 clippy --release --locked --no-deps --bin accounting-probe --features accounting-probe -- -D warnings` | Passed, 3.36 s. |
| `cargo +1.96.0 doc --release --locked --no-deps --bin accounting-probe --features accounting-probe` | Passed, 1.60 s. |
| Final release build command above | Passed, 3.63 s. |

Durations are local warm-cache command observations, not measured engineering
effort or a maintenance comparison. The existing scoped build-std/bootstrap
exception remains documented in the original IDF build record.

## Actual configuration, ABI and linkage

The generated configuration enables statistics, trace, statistics-formatting
support, U64 counters and the esp_timer clock. Formatting helpers are not called.
CPU/flash remain 160/80 MHz, kernel tick 100 Hz, radio/PM/task watchdog off,
main stack 32,768 bytes and panic print/reboot unchanged.

Regenerated `TaskStatus_t.ulRunTimeCounter` is **u64**, and the generated
`uxTaskGetSystemState` total argument is **`*mut u64`**. The authored code passes
that type directly; no cast adapts the old U32 ABI. Actual calls link to
`uxTaskGetSystemState`, `vTaskGetInfo`, heap-capability allocation/free/info,
scheduler suspend/resume and `esp_timer_get_time`.

The SDK selects ROM heap implementation. The linked `heap_caps_get_info` path
reaches C6 ROM `multi_heap_get_info` at `0x400004a0`; this inspection does not
establish that the excluded equivalent TLSF C walk is the executed body.

DWARF reports `Snapshot` **592 bytes**, alignment 8. Six retained records occupy
**3,552 bytes**, in addition to temporary raw status arrays, task stacks, channel
objects and other runtime/harness memory. This is not application headroom.
Inspected application frame allocations:

| Function | Bytes |
| --- | ---: |
| main (split prologue: 256 + 12,736) | 12,992 |
| snapshot | 1,824 |
| allocation control | 1,248 |
| current-task blind-spot control | 144 |
| worker entry | 224 |

These are individual frames, not a complete call-chain/interrupt stack bound or
an observed maximum. Physical snapshots retain stack-fill indicators separately.

## Image preparation before flashing

The first candidate used espflash 4.6.0 `save-image` with explicit DIO/80 MHz/
8 MB settings. `esptool 5.1.0 image-info` flagged its C6 frequency nibble `0xf`
as invalid. That candidate and diagnostics are retained; **it was not flashed**.
This is not evidence that historical captures failed for the same reason.

The selected image instead follows the pinned SDK's
[elf2image arguments](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/esptool_py/project_include.cmake#L34),
including the application ELF digest offset:

```sh
esptool --chip esp32c6 elf2image --flash-mode dio --flash-freq 80m \
  --flash-size 8MB --elf-sha256-offset 0xb0 \
  --output .cache/accounting-sdk-app.bin \
  firmware/esp-idf/.cache/accounting-target/riscv32imac-esp-espidf/release/accounting-probe
```

Both candidates contain the same **413,472-byte** application image size; the
ELF is unchanged. The selected image passes the C6 image/header/checksum/digest
inspection. The generated SDK bootloader/partition table, not an assumed
ELF-only flash layout, accompany it at addresses 0/0x8000; app is at 0x10000.
The bootloader header also reports DIO/80 MHz/8 MB. Before/after register
readback remains required for actual runtime clocks.

Exact source, ELF, selected/rejected images, generated configuration/bindings,
maps, disassembly excerpts, build/packaging logs, original provenance and
collector identity are retained in the qualification input freeze. No board
pass, current-free recovery or busy/blocked separation is asserted here.

## Host integration

The final local workspace checks passed: formatting; **80 unit tests** (78
collector, two shared-core) and **two doctests**; all-target Clippy with warnings
denied; documentation; and the release collector build. The new twelve host
tests exercise synthetic records and real capture/replay control paths. The
independent review additionally exercises CLI mutations and failed-port
retention. These are host checks with no physical measurement implied.

Historical replay still reproduces all 61 protocol-1/2/3 transcripts and the
existing study summary. The existing prospective queue method retains its
separate positive/negative controls; protocol 4 adds no queue comparison score.
