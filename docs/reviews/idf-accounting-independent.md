# Independent IDF accounting preflight review

Reviewed **2026-10-23**, before flashing, against the
[qualification contract](../experiments/idf-accounting-qualification.md),
[method source audit](../research/idf-accounting-method.md),
[coding standard](../coding-standard.md) and existing
[all-54 API inventory](../rust-api-checklist.md). This review covers the actual
Rust firmware, host validator/capture path, generated ABI and linked image.
It establishes no physical qualification result.

## Design and source findings

The separate review challenged the design before firmware was authored. The
contract now specifies causal snapshot/busy-work boundaries, distinct worker,
sampler and idle handles, valid live-task states, the sampler running state and
idle priority. Busy accounting is explicitly `busy-after` minus `blocked-after`.
These refinements occurred before physical attempts, not by changing a failed
capture's acceptance rule.

Source reasoning confirms:

- The persistent worker is blocked at both measured boundaries, so its stored
  counter is settled. The sampler's own open interval remains incomplete.
- Both blind-spot counter reads occur inside scheduler suspension. Resume is
  unconditional before validation, error construction, output or a fallible
  return; the bounded loop contains no blocking/allocation.
- Actual workload returns are checked against the known answer and accumulated.
  Channels execute a priming exchange before baseline observations.
- SDK task-name/stack pointers are discarded. Numeric task identities remain
  valid until Stop; there are no handle queries after Stop or join.
- The direct allocation is null-checked, every byte is initialized and checked
  within bounds, and it is freed once even if the held snapshot returns an error.
- JSON is emitted after observations and worker join. Snapshot memory excludes
  later formatting/output costs and does not describe the whole harness.

No additional concrete memory-safety defect was found within this review. That
does not authenticate the hardware/image relationship or prove every transitive
SDK/standard-library path safe. A stuck kernel/control channel can still require
host timeout/reset; it is not successful cleanup.

The [firmware API/FFI delta](idf-accounting-firmware.md) was reviewed against the
existing IDF/queue applicability records. The host adds private typed records,
strict validation and ordinary blocking filesystem/serial operations; it adds
no unsafe operation. `C-VALIDATE`, `C-FAILURE`, `C-SERDE`, `C-INTERMEDIATE`,
`C-SEND-SYNC`, `C-RW-VALUE` and explicit cleanup remain relevant. The unused raw
timer helpers have a narrow documented dead-code allowance because the shared
module also supplies this binary's existing configuration decoder.

## A reproduced host defect repaired before capture

Initially, adding `2^63` to the idle task's lifetime counter in every synthetic
snapshot still passed actual CLI replay: deltas stayed small and monotonic.
Such a baseline is impossible under the selected one-core clock origin.
The repaired validator rejects each stored task counter above its snapshot
total and the blind-spot stored counter above its start clock. The exact
counterexample now rejects on a newly built CLI.

This does **not** impose a conservation equation on task deltas. A separate
synthetic positive adds 10,000 us to later sampler counters, yielding a signed
blocked-phase residual of **-10,000 us**, and remains accepted. Open intervals
at different boundaries can produce a negative delta residual. It is not idle
time or evidence that a timer ran backwards.

## Executed independent host controls

A separately constructed twelve-record synthetic transcript passed the real
`validate-accounting` CLI. Two additional positives preserved a negative signed
residual and a lifetime counter above `u32::MAX`. These are host fixtures, never
on-board observations.

All **33** independent CLI mutations rejected:

| Fault group | Cases |
| --- | ---: |
| Wrong counter width or clock | 2 |
| Missing/duplicate/changed task identities or empty snapshot | 5 |
| Decreasing/active blocked-worker counter, state, phase, priority, stack reservation, sampler state or deleted task | 8 |
| Record order, observer order or total clock outside enumeration | 3 |
| Zero busy progress or wrong workload/allocation checksum | 3 |
| Heap largest-block inequality, insufficient allocation drop or background drift | 3 |
| Missing/duplicate completion or unknown field | 3 |
| Panic, reboot, additional ROM, trailing panic, missing ROM or oversized line | 6 |

The impossible-lifetime baseline is an additional rejection above those 33.
Five CLI preflights reject before output creation or attempted serial open:
comparison purpose, missing purpose, zero timeout, missing ELF and invalid
provenance. A valid qualification command targeting a nonexistent device creates
a failed attempt with raw-file and original-provenance retention, null
qualification/benchmarks and the serial-open failure. No real device is opened.

A derived host-only protocol-4 archive independently passes `replay-archives`.
Changing the report's worker result, promoting purpose to comparison, changing
original provenance bytes, retaining a favorable qualification under failed
status or adding comparative benchmark fields each rejects. A failed record
with a null qualification remains failed:
one failure retained, zero successful transcripts. This is metadata/replay
consistency, not authentication of an execution receipt.

The copied CLI used for these mutation checks has SHA-256
`9fb07180e46c4bdc2f2066bac8e46ef55e8764e69d11d8e8a925ee21608023b3`.
Inputs, CLI outputs and mutation summaries remain in the ignored
`.cache/accounting-independent/` directory. Maintained executable regressions
are in [accounting.rs](../../tools/benchctl/src/accounting.rs); cache fixtures
are supplementary local review evidence, not a public physical dataset.

Independently executed commands included:

```sh
cargo +1.96.0 build --locked -p benchctl
cargo +1.96.0 test --locked -p benchctl accounting::tests --quiet
cargo +1.96.0 test --workspace --locked --quiet
target/debug/benchctl validate-accounting --input <synthetic-raw>
target/debug/benchctl capture-accounting <fake-port-preflight-arguments>
target/debug/benchctl replay-archives --root <derived-host-fixture>
```

The twelve targeted tests and full **80 unit tests** (78 collector, two shared
core) plus **two doctests** passed. Fake transport tests exercise fragmented
capture and permanent fatal/reboot rejection through the real capture reader,
not only a separate parser. Root separately records final formatting, Clippy,
documentation, release build and public-archive checks.

## Compiled ABI and image inspection

The actual isolated generated bindings have
`TaskStatus_t.ulRunTimeCounter: u64` and
`uxTaskGetSystemState(..., pulTotalRunTime: *mut u64)`. Both firmware pointer and
field assignments require that ABI. Raw `TaskStatus_t` contains integer/raw
pointer fields with valid zero representations; Rust never treats a saved
foreign name pointer as an owned string.

Independent inspection of the linked disassembly confirms checked calls to
`bench_core::workload` with seed `0x12345678`/4,096 rounds and comparison with
`0xa3944d55`; task statistics and heap APIs are linked. Heap metadata calls use
the ROM alias at `0x400004a0`, not an assumed inspected C TLSF implementation.
Observed fixed frames include main **12,992 bytes**, snapshot **1,824**, worker
entry **224**, busy work **80**, allocation control **1,248** and blind-spot
control **144**. These are individual compiled frames, not a complete call-chain
bound or runtime stack measurement. Six snapshots occupy 3,552 bytes; that is
distinct from the main frame and other transient storage.

A maximum-width sixteen-task snapshot serializes to 2,667 bytes under this
numeric schema, below the shared 4,096-byte line limit.

The first espflash-produced application candidate contains frequency nibble
`0xf`. Independent esptool 5.1.0 `image-info` prints
`Warning: Invalid flash frequency (0xf)` **while returning exit status zero**.
Preflight rejected that header; it was not a failed board attempt. The warning
does not establish a cause for an earlier experiment's ROM warning.

The selected SDK/esptool-generated `.cache/accounting-sdk-app.bin` instead
decodes as C6, DIO, 8 MB, 80 MHz with valid checksum/validation hash and embedded
ELF SHA matching the built ELF:

| Artifact | SHA-256 |
| --- | --- |
| Built ELF | `c535eb9e8a30fdffdbc922c0dfdc1c63426a35dbadf43721241325a47edd1794` |
| Selected application image | `c3cfe3791b3c83e8ba7215eafab13e33ef60c3e2a99ffd0c6379edf2f3a5e824` |

No remaining execution blocker was found for the frozen narrow qualification
after these checks. This review performs **no board operations**. Physical
attempts, image-transfer evidence, fresh backup and full restoration verification
remain the primary agent's separate work. Success would still not establish
physical CPU utilization, exclusive ISR attribution, global heap/stack peaks,
capacity, instrumentation-on/off cost, external accuracy, power or OS selection.
