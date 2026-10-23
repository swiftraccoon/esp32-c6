# benchctl raw-tick queue protocol review

Reviewed 2026-10-23 against the [coding standard](../coding-standard.md),
all 54 active [API identifiers](../rust-api-checklist.md) and the existing
[protocol 2 applicability matrix](benchctl-v2.md#api-applicability). Scope:
`tools/benchctl/src/queue_protocol.rs`, the queue capture/replay CLI additions
in `main.rs`, the narrowly widened `Clocks::validate` visibility in
`protocol.rs`, and the protocol 3 README. The independently authored schedule
and study analysis module has its own review. Dependencies, lockfile, host
target, Rust 1.96.0, edition/style edition 2024 and existing package/license
exceptions remain unchanged.

## Contract and execution

Protocol 3 is separate from legacy admission and serialization. No legacy
record type or acceptance rule changed; only the existing clock validator is
shared within this binary. New CLI commands explicitly select capture/replay
for a queue backend. Strict typed records reject unknown and missing fields.
Each event agrees on protocol, backend and device boot nonce. The accepted
sequence contains exactly 1,000 observer samples, 1,000 checked request/reply
samples, qualification totals and completion. Per-benchmark indices, sample
timestamps, responder phases, checksums and operation counts are validated.
Any acceptance error permanently invalidates the boot and revokes its summary.

The host independently decodes CPU/flash and timer configuration, requires the
selected IDF UNIT0 stall setting and exact initial/final raw-register equality.
Timer configuration source: pinned C6 PAC 0.24.0 `systimer/conf.rs`,
`pcr/systimer_conf.rs`, `pcr/systimer_func_clk_conf.rs`; the latter two describe
APB clock/reset and function source/enable independently. CONF bit 31 is
register clock gating; it is retained without incorrectly treating it as the
counter's function-enable bit. Raw timestamps must fit 52 bits and advance
without wrap normalization. The declared one-second message-duration guard
and 500 ms qualification guard fail a boot rather than clipping or dropping
its long observations.

Qualification requires ordered reads, a bounded blocked sleep interval checked
against the legacy timer bracket, and SDK WFI calls checked against elapsed
kernel ticks with no attached debugger. It also requires all 1,008 warmup and
measured readiness/blocked checks, native empty checks where applicable, and
independent full/empty/probe-cleanup checks. Actual worker join and backend
destruction are completion conditions. Host admission verifies consistency of
these records; source/binary review and device runs establish the implementation
behind their declarations. Serial records cannot independently authenticate
firmware, identify kernel wait lists or establish external oscillator accuracy.

Raw gross ticks remain integers. Reports include the exact rational scale
1/16,000,000 second per tick, nearest-rank per-boot percentiles and a floating
arithmetic mean. The mean may round; integer endpoints and order statistics
remain exact. No observer subtraction, microsecond truncation, invented integer
nanosecond quantum, pooled independent-sample claim or maximum-latency guarantee
is introduced. Zero observer ticks remain valid; message duration is positive.

The collector is one blocking host thread; allocations, process spawning,
file/serial I/O and reset sleep occur outside device timing. Protocol lines
remain bounded to 4,096 payload bytes. Accepted storage contains at most 2,000
sample objects; supplied artifacts and metadata remain operator-controlled.
No unsafe code, FFI, custom synchronization, interrupt/async/cancellation path
or destructor is authored here; the binary's `forbid(unsafe_code)` applies.

Capture validates artifacts/backend/deadline before resetting, refuses output
replacement and retains original raw bytes, parsed records and a failed report
on protocol failure or timeout when output I/O permits. A disk/write failure or
process termination can leave partial evidence or `running`, never a pass.
Already-read trailing records after completion are checked. Live capture ends
at the first complete boot; replay checks every record through EOF. Capture is
not a later stability test. The default 180-second deadline accommodates serial
transmission and is distinct from device-operation admission guards.

Device boot nonces are checked throughout a boot and against all explicitly
supplied `--previous-nonce` values. The study analyzer separately rejects reused
nonces across its admitted captures. These are non-cryptographic freshness
observations, not proof of physical reset or statistical independence. Missing,
failed, unsupported, incomplete or rejected observations have no numeric summary.

## API applicability

The complete 54-entry protocol 2 matrix is reused. Every identifier was checked
against this change; no decision is pending and no new exception is introduced.
Unchanged inapplicability decisions remain concrete: no public conversion,
iterator, feature, collection, macro, smart-pointer, operator, trait, builder,
generic-struct, destructor or stable public library interface was added.

Affected applicable entries and evidence:

| Entries | Evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER, C-GETTER | Rust naming and explicit `_tick`, `_ticks`, `_us`, `_hz`, `_bytes` units; state inspection returns values or shared borrows. |
| C-COMMON-TRAITS, C-SERDE, C-SEND-SYNC | Typed `Debug`/serde records; configuration derives equality for full snapshots; boot state is exclusively mutated in one host context. |
| C-GOOD-ERR, C-RW-VALUE | Contextual `Result` propagates ordinary input/I/O errors; replay accepts owned or borrowed `BufRead`, output helpers accept `Write`. No exported error type. |
| C-CRATE-DOC, C-FAILURE, C-LINK | Module docs and README explain strict wire shape, CLI examples, units, rejection, blocking, reset and measurement limits and link the experiment and reviews. No exported Rust library examples are added. |
| C-HIDDEN, C-METHOD, C-NO-OUT | Types remain private or `pub(crate)`; validation/summary methods live on owning types and return values. Mutable capture writers preserve evidence intentionally. |
| C-CTOR, C-INTERMEDIATE, C-CALLER-CONTROL | Default empty boot is valid; summary requires completed unrejected state; raw timestamps/qualification/configuration remain archived; paths, strings and nonce history are borrowed. |
| C-GENERIC, C-NEWTYPE, C-CUSTOM-TYPE, C-VALIDATE | Fixed-width wire integers and named timer/configuration/sample/qualification/boot/statistics types retain units and state distinctions. Counts, sequence, checksum, phase, backend, nonce, scale and register claims are checked before reporting. |
| C-DEBUG, C-DEBUG-NONEMPTY, C-STRUCT-PRIVATE | Field-bearing derived debugging; no credentials; wire and mutable validation fields remain private. |
| C-METADATA, C-PERMISSIVE | Existing M1/L1/L2 exceptions remain unchanged: unpublished package metadata, project license unchosen, existing MPL-2.0 serial dependency; no dependency added. Original review records distribution revisit triggers. |

## Verification

These are host parser/analysis checks, not hardware passes:

| Command | Actual result |
| --- | --- |
| `cargo +1.96.0 test -p benchctl --locked` | Passed 31 tests: 18 existing protocol/framing tests, 9 new queue protocol tests and 4 schedule/statistics tests at this checkpoint. Mutation cases cover missing/duplicate/reordered/trailing records, wrong backend/nonce/header/configuration, checksum/phase/timestamp failures, qualification and cleanup failures, malformed/overlong framing, raw tick summaries and explicit nonce freshness. |
| `cargo +1.96.0 fmt -p benchctl -- --check` | Passed. |
| `cargo +1.96.0 clippy -p benchctl --locked --all-targets -- -D warnings` | Passed without added lint suppression. |
| `cargo +1.96.0 doc -p benchctl --locked --no-deps` | Passed. |

The 31-test rerun includes the one-second/500 ms guard mutation cases.
Archived protocol 1/2 replay, release collector hash, independent integration
review and physical qualification are recorded by the root experiment owner.
No hardware was accessed by this collector implementation review.
