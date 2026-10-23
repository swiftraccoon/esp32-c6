# IDF accounting host review

Reviewed **2026-10-23**, before physical qualification. Scope:
`tools/benchctl/src/accounting.rs`, its CLI branches, crate-only reuse of the
existing clock/configuration decoder and protocol-4 archival replay dispatch.
The [prospective experiment contract](../experiments/idf-accounting-qualification.md)
defines the observations and acceptance rules. No board was opened, reset or
flashed while implementing or testing these host components.

## Behavior and evidence boundary

`capture-accounting` requires explicit purpose `qualification` and rejects
comparison before input reads, output creation or serial access. It records
schema-4 admission with exact ELF and original provenance hashes, retaining both
input files. Original inputs are reread and checked before serial access.
Started failures retain a failed report and raw received bytes, with no
favorable qualification summary. Existing output directories are not replaced.

Capture and `validate-accounting` use the same typed record/state validator and
the existing single-boot/fatal-text guard. Twelve records have one exact order;
unknown fields, extra/missing records, changed configuration or rejected
controls cannot produce a summary. A later valid hello does not recover a
rejected boot. Capture's finite observation ends after accepted completion and
its received chunk; later arriving diagnostics remain unobserved.

The validator requires 64-bit esp_timer counter declarations, complete stable
task identity sets, distinct nonzero handles/task numbers, live state values,
worker blocked/sampler running boundaries, priorities and known stack bounds.
It checks ordered gross observer endpoints, advancing total clocks, nondecreasing
task counters and individual counters within the snapshot's total clock.
Known task identities remain matched even if array order changes. It never
dereferences device task-name pointers.

Capacity and current-task blind-spot controls, bounded blocked/busy phases,
checked workload progress and direct-allocation controls must all pass. Busy
work lies causally between its surrounding snapshots. Coarse 80–120 percent
attribution acceptance distinguishes busy work from blocking; it is not a
clock-accuracy bound or physical CPU-utilization measurement.

Derived output retains task deltas, idle-task attribution and signed boundary
residuals, without observer subtraction or sum-conservation assumptions. Checked
arithmetic rejects sums outside u64 or residuals outside the signed report
range. Both positive and negative residuals are legal. Heap and stack-fill
indicators do not establish instantaneous global peaks, fragmentation, exclusive
ISR costs or power.

`replay-archives` recognizes these qualification captures, checks the preserved
ELF/provenance input identities and admission, reruns the actual validator and
compares both capture fields and complete `replay.json` bytes. Failed reports
remain failures even when their raw input could otherwise validate; a failed
report carrying a favorable qualification summary is rejected. Protocols 1–3
and the historical study keep their existing replay behavior.

## API applicability delta

The [54-entry inventory](../rust-api-checklist.md) and existing
[collector](benchctl.md), [boot guard](collector-boot-guard.md) and
[method-admission](method-admission.md) dispositions were reconsidered. The
affected decisions below add no exception or pending applicability. This is
private Rust 1.96.0 host application code: no firmware, unsafe operation, FFI,
dependency, async context, interrupt handler or public library API is added.

| Entries | Decision / evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: named protocol/control/task/heap types and explicit `_us`/`_bytes`/identity fields distinguish meanings. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: interchange records derive field-bearing Debug/serde; live boot state is not copied or shared. Default represents a pending boot, not accepted data. |
| C-SERDE, C-VALIDATE | Applies: required typed records deny unknown fields; exact order, widths, task sets, causal clocks, checksums and control limits precede success. |
| C-GOOD-ERR, C-FAILURE | Applies: contextual Result for source, parsing, clock, identity, arithmetic, serial and output failures; ordinary invalid data does not panic. |
| C-CUSTOM-TYPE, C-NEWTYPE, C-STRUCT-PRIVATE, C-HIDDEN | Applies at internal scope: separate record/control/snapshot types preserve domain distinctions; representations remain private. No exported unit-wrapper commitment is added. |
| C-METHOD, C-CTOR | Applies: validation and poisoning belong to Boot/Snapshot owners; no public constructor API is introduced. Test Fixture::new creates an exclusive directory. |
| C-SEND-SYNC | Applies: one host context exclusively owns validator and transport state. No manual Send/Sync or sharing invariant is introduced. |
| C-RW-VALUE, C-CALLER-CONTROL, C-GENERIC | Applies: borrowed Read/Write paths support actual serial readers and fragmented synthetic transports without a parallel parser. |
| C-NO-OUT, C-INTERMEDIATE | Applies: fallible operations return reports through Result; raw output buffers intentionally preserve evidence. A poisoned/incomplete boot cannot expose a favorable summary. |
| C-CRATE-DOC, C-LINK, C-EXAMPLE, C-QUESTION-MARK | Applies at internal application scope: module/CLI documentation and realistic qualification/replay commands state effects and limits. Examples are not represented as device runs. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No production destructor added. Test-only cleanup targets an exclusively created temporary fixture and ignores cleanup failure. |

Other existing decisions remain unchanged: no authored macros, iterator or
collection trait, conversion/operator, smart pointer, feature or reusable
generic data-structure API. Blocking host filesystem/serial/Git operations and
host allocation occur outside measured device paths. Existing project metadata
and license exceptions are not silently changed.

## Executed host checks

```sh
cargo +1.96.0 fmt -p benchctl
cargo +1.96.0 check --locked -p benchctl
cargo +1.96.0 test --locked -p benchctl accounting::tests -- --nocapture
cargo +1.96.0 clippy --locked -p benchctl --all-targets -- -D warnings
```

Check and Clippy passed. All **12 targeted accounting tests passed**. An initial
test expected a generic clock diagnostic; the actual shared decoder correctly
returned its specific CPU-readback diagnostic. Correcting that test assertion
did not change the acceptance rule or decoder.

The synthetic positive retains a task counter above u32 and a negative signed
boundary residual. Its aggregate checksums use independent fixed expected
values. Adverse fixtures exercise unknown/missing/duplicate/order fields,
wrong width/clock, empty/over-capacity/lifecycle task sets, duplicate/zero task
identities, invalid/deleted states, incorrect priority/stack/phase/blocking,
counter decrease/impossible absolute counters, arithmetic overflow, causal
endpoint violations, wrong checksums, absent progress, failed capacity/open
controls, heap inequality/drop/drift/recovery errors and missing cleanup.

Fake readers exercise pre-JSON panic, reboot and additional ROM startup at
one-, seven- and 1,024-byte read boundaries through both actual capture and
replay. Failure permanently revokes summary admission. Separate tests reject
comparison before input/output/port work, require purpose at CLI parsing and
retain schema-4 failure/input identities for a deliberately nonexistent serial
path. That path is a host failure fixture, not a device operation.

A temporary protocol-4 archive exercises successful replay, altered capture
summary, changed purpose, stale original provenance and failed-status
preservation. These are generated host fixtures, never historical physical
captures. Final repository-wide integration, artifact checks and any physical
qualification are recorded separately by the primary agent. Passing these
checks does not establish scientific completeness or board authenticity.
