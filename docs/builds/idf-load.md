# Finite offered-load build and preflight

Built **2026-10-23**, separately from the
[prospective contract](../experiments/idf-load-qualification.md),
[protocol](../experiments/idf-load-protocol.md) and physical qualification.
This record establishes compilation/configuration/host controls, not a board
pass or a statistics-effect comparison.

## Build tuple and isolation

Rust1.96.0, edition/style2024, target `riscv32imac-esp-espidf`, sys0.37.2 and
IDF5.5.6 at `880a203ae603d08a94d06aa01e0d6761958e5e9e` remain pinned.
Release opt3/thinLTO/one codegen unit/panic abort/debug2 remain unchanged.
No dependency, SDK patch, authored C/assembly or application ISR was added.
The existing scoped build-std bootstrap exception remains in effect.

Use the existing installed tools/SDK with absolute paths, avoiding the earlier
isolated-target workspace-inference trap. From `firmware/esp-idf`, disabled:

```sh
env RUSTC_BOOTSTRAP=1 LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib \
  PATH="$PWD/.tools/bin:$PATH" \
  CARGO_TARGET_DIR="$PWD/.cache/load-off-target" \
  ESP_IDF_TOOLS_INSTALL_DIR="custom:$PWD/.embuild/espressif" \
  IDF_PATH="$PWD/.embuild/espressif/esp-idf/v5.5.6" \
  ESP_IDF_SDKCONFIG="$PWD/.cache/sdkconfig-load-off" \
  ESP_IDF_SDKCONFIG_DEFAULTS="$PWD/sdkconfig.defaults;$PWD/sdkconfig.load.defaults" \
  cargo +1.96.0 build --release --locked --bin load-probe --features load-probe
```

For enabled, use distinct `.cache/load-on-target`/`.cache/sdkconfig-load-on`,
append `;$PWD/sdkconfig.accounting.defaults` to the defaults list and select
feature `load-runtime-stats`. Do not replace the old queue/accounting outputs.

Both actual configurations enable trace/statistics-formatting support and
**two** notification slots; slot1 belongs to this generator while pthread uses0.
Disabled statistics produce generated U32 counter fields/total pointers;
enabled selects esp_timer and U64. Both branches pass their actual pointer
types without casts, and disabled API zeros become unavailable/null fields.
CPU160/flash80, tick100Hz, PM/radios/task-watchdog off and existing UART routing
remain unchanged. The code verifies actual task priorities and clock readback.

## Target and binary checks

Both fresh isolated release builds completed without warnings (53.47s disabled,
53.12s enabled). Following the diagnostic-status improvements described below,
both final profiles passed formatting, target Clippy with `-D warnings`, target
documentation and a final release build. The final commands use the environment
above and their corresponding feature:

```sh
cargo +1.96.0 fmt --all -- --check
cargo +1.96.0 clippy --release --locked --no-deps --bin load-probe --features load-probe -- -D warnings
cargo +1.96.0 doc --release --locked --no-deps --bin load-probe --features load-probe
cargo +1.96.0 build --release --locked --bin load-probe --features load-probe
```

Build-command durations are local provisioning/cache observations, not measured
maintenance effort. The two final commands can contend on Cargo's package cache;
those waits are retained in the logs and are not device-performance evidence.

DWARF confirms both profiles retain a24-byte atomic Record and744-byte Snapshot.
The2016 record values occupy **48,384 bytes**; twelve snapshots occupy **8,928
bytes**, separately from Arc/Vec metadata, allocator overhead, temporary SDK
arrays and stacks. SDK status scratch layouts differ naturally with U32/U64.
Disabled inspected individual frame allocations include main3120 (2032+1088),
producer entry448, consumer entry176, observer1968 and callback16 bytes.
These are not complete call-chain bounds or observed stack peaks.

Actual binaries link task notification/wait, one-shot start, blocking timer
stop and deferred delete. The callback passes notification index1/eIncrement.
Images are generated with the SDK's C6 conversion arguments, including
`--elf-sha256-offset 0xb0`, through esptool5.1.0. Header inspection reports
C6/DIO/80MHz/8MB and valid checksum/digest; embedded ELF hashes match the inputs.
Matching generated SDK bootloader/partition tables accompany each application.
Runtime register readback remains separate from an image header.

## Host and independent checks

The full local host workspace passed **96 unit tests** (94collector/two core),
**two doctests**, formatting, all-target Clippy with warnings denied,
documentation and release build. The release CLI admits independently built
full2043-record positives for both profiles and rejects the reproduced
impossible-full transcript for the intended queue-history reason.

The independent preflight used five valid synthetic cases,45 contrary
transcripts, six pre-serial failures and eleven archive mutations. Synthetic
fixtures do not establish physical execution. Existing64 protocol transcripts
and one historical study summary still replay byte-for-byte. The
[independent review](../reviews/idf-load-independent.md),
[host API review](../reviews/idf-load-host.md),
[firmware review](../reviews/idf-load-firmware.md) and
[observer review](../reviews/idf-load-observer.md) record scope and evidence.

Three issues were corrected before input freeze/physical collection: initial
generic SDK-failure diagnostics lost numeric statuses; a host queue-depth
check failed to subtract jobs proven already dequeued; and new-load derived
summary errors could bypass final failed-report retention. Their concrete
reproductions and regression checks are retained in the reviews. A suspected
analogy in the older accounting collector was disproved by its complete-time
admission check; only a protection regression was added there.

Independent full transcript fixtures are408,129/408,183 bytes, approximately
35.4s each at115200baud8N1 before source/transport variation. Maximum-width
snapshot encoding stays below4096bytes. The explicit120s capture budget covers
post-measurement output; no serial time is included in service windows.

Exact images, sources, configurations/bindings, provenance, logs and controls
must be retained in the prospective input freeze before any physical attempt.
Neither this build nor the host controls establish sustained capacity, physical
CPU utilization, runtime global peaks, an on/off effect or a platform choice.
