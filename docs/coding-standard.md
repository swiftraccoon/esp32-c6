# Coding standard

This standard defines the Rust baseline for the ESP32-C6 workspace. It governs code we
author: firmware, OS adapters, benchmark workloads, shared libraries, build
helpers, and host tools. Upstream OS and vendor code retains its own conventions;
keep our additions identifiable and avoid unrelated restyling.

## Adopted sources and authority

| Local source | Authority |
| --- | --- |
| [Rust Style Guide](standards/rust-style/src/doc/style-guide/src/README.md) | Source and manifest conventions. |
| [Rust API Guidelines](standards/rust-api/src/SUMMARY.md) | Every applicable checklist entry, identified by its `C-*` identifier. |
| [Google Rust Style Guide](standards/google-rust/rustguide.md) | Supplemental implementation and design guidance. |

The [source lock](standards/sources.json) records the normative commits and exact
file hashes. Read the checked-in copies offline; upstream URLs, attribution and
licenses are in the [local reference index](standards/README.md). Updating a
toolchain does not update the baseline. Use the explicit
[updater](standards/README.md#update-from-upstream), review changed rules and
reconcile applicability records before committing a source-adoption change.

Precedence is: the user's instructions, the two explicit authority assignments
above in their respective domains, then compatible Google guidance. The Google
guide's own precedence rule is for Google's codebases; it does not override the
user's assignments here. For an actual conflict, cite both sections, retain the
explicit authority, and record the resolution. A hardware limitation needs an
explicit, narrow exception with evidence, not an unrecorded relaxation.

## Language and implementation boundaries

Rust is the default for newly authored implementation, including small host-side
automation when it merits maintained code. Use existing tools for routine
research and inspection; this preference is not a reason to recreate shell
commands or upstream tooling in Rust.

If a candidate lacks viable Rust integration, document the exact missing
capability and options: existing Rust binding, a minimal FFI adapter, an upstream
example, or an implementation in another language. Discuss the tradeoff with the
user before authoring C, C++, assembly, or another implementation language. A
vendor library already implemented in C is a dependency, not a claim that our
Rust application is entirely Rust. Building unmodified upstream examples is
useful evidence and does not require translating the OS itself.

Keep reusable workload logic separate from OS entry points, timer and scheduler
adapters, transport, and board configuration. Preserve behavior across adapters:
same payloads, queue capacities, validation, timeouts, and completion conditions.
Prefer a small direct adapter to a general framework without a concrete need.

## Formatting and tooling

Follow the pinned official chapters for
[source layout](standards/rust-style/src/doc/style-guide/src/README.md),
[items and imports](standards/rust-style/src/doc/style-guide/src/items.md),
[statements](standards/rust-style/src/doc/style-guide/src/statements.md),
[expressions](standards/rust-style/src/doc/style-guide/src/expressions.md),
[types](standards/rust-style/src/doc/style-guide/src/types.md),
and [Cargo manifests](standards/rust-style/src/doc/style-guide/src/cargo.md).
Use `rustfmt` as the mechanical formatter and review the parts it does not cover,
including manifests and documentation. Do not add nightly-only formatting options
solely to match a supplemental preference.

Before the first code build, select and record a compiler version, Rust edition,
formatting edition, target triple, components, and SDK/toolchain requirements.
Pin tool versions for reproducible runs. The pinned
[style-edition chapter](standards/rust-style/src/doc/style-guide/src/editions.md)
contains historical release-status wording; its description is not today's
compiler-support matrix. The language edition, style edition, and minimum
supported Rust version (MSRV) are separate decisions. Do not invent an MSRV
before checking the chosen HAL, OS, bindings, and host dependencies.

Prefer a supported stable compiler. If a dependency requires nightly or a vendor
toolchain, record the requirement, exact version, and affected targets. Scope
that requirement to the component that needs it. A shared host library should
not inherit firmware-only restrictions unnecessarily.

For each maintained application, commit its dependency lockfile and record SDK,
OS, submodule, tool, and binary-blob versions or hashes. Set actual package
metadata; do not invent authors, licenses, or websites to satisfy a checklist.
Choose project licensing deliberately before distribution, and flag dependency
license constraints without changing upstream license terms.

## Commit messages and checkpoints

Use Conventional Commits 1.0.0 under the
[commit standard](commit-standard.md). Commit
meaningful completed work as it progresses, including the relevant documentation,
lockfiles, review evidence and measurements. That standard distinguishes upstream
syntax from repository choices and keeps release/version/tag/push actions scoped
to their separate authorization.

## API review and applicability

The pinned checklist contains **54 active entries**. The complete inventory and
review procedure are in [rust-api-checklist.md](rust-api-checklist.md). Every code
change needs an applicability review; every applicable entry needs evidence.
Reuse a crate's established applicability record and update affected decisions
instead of duplicating a long checklist in every small change.

Use four outcomes: **applies**, **not applicable** with a concrete reason,
**exception** with scope and evidence, or **pending**. Pending is not compliance.
Private application code does not need the public surface of a general-purpose
library, but privacy does not waive naming, safety, correctness, or any guideline
whose stated scope still applies. Apply the actual pinned section, including its
qualifications, rather than treating its short checklist title as the entire
rule.

Document public behavior and useful examples; link one realistic example from
related items where the guideline permits it. Hardware-only examples must state
their target, wiring, and execution requirements. Do not label an example tested
when it was only compiled. Keep implementation details private unless callers
need a representation commitment. Use types to distinguish cycles, durations,
bytes, priorities, radio modes, and result statuses where confusion could affect
correctness. Do not force one universal priority scale across dissimilar OSes.

## Embedded and host applicability

| Situation | Repository application |
| --- | --- |
| Bare-metal or embedded `no_std` component | State whether `alloc` is used and how memory is provided. Prefer bounded storage for measured paths. A `no_std` label alone does not prove absence of allocation. |
| Rust with an OS/SDK providing `std` | Record the exact target and `std` integration. Review thread, timer, I/O, heap, and failure behavior through that integration. |
| Host tooling | Use normal `std` facilities where appropriate; preserve target-independent parsing and calculation logic for host tests. |
| Reusable records and configuration | Evaluate `C-SERDE`; an optional `serde` feature can preserve a small firmware dependency graph. Hardware handles and live peripheral ownership are not interchange records. |
| Public error type | Preserve the intent of `C-GOOD-ERR`: meaningful diagnostics and error composition. Use `core::error::Error` when supported by the selected compiler; use a documented `std`-gated implementation or narrow MSRV exception when necessary. Record this adaptation. |
| Reader/writer abstraction | Apply `C-RW-VALUE` to the actual trait contract. Do not import `std::io` into a `no_std` component merely to mirror a source example. Verify whether the embedded I/O trait supports the same borrowing pattern. |
| Peripheral/resource handle | Consider common traits individually. Do not derive `Copy`, `Clone`, `Default`, or sharing traits when doing so would duplicate ownership or make an invalid device state constructible. |
| Fixed-capacity collection | Evaluate `C-COLLECT` against explicit overflow semantics. Document a narrow exception if an infallible collection trait would hide overflow; expose checked operations. |

These are mappings and review questions, not blanket exemptions. The final
choice belongs in the component's applicability record with its target and
compiler evidence.

## Safety and concurrency

Use maintained safe HAL and OS interfaces where they satisfy the requirement.
Keep raw register access, unsafe initialization, FFI, and interrupt glue isolated
from workload algorithms. For every unsafe operation or unsafe trait
implementation, explain the invariant and why this call site preserves it.
Document the caller's obligations on public unsafe interfaces. Performance alone
is not evidence for unsafe code; measure the safe alternative if performance is
the justification.

For FFI, document ABI, layouts, ownership, pointer validity, alignment, lifetimes,
callback context, teardown ordering, and error conversion. A safe wrapper must
prevent callers from violating the foreign contract. Validate foreign integer
statuses before constructing Rust enum values. State the panic strategy and
ensure a panic cannot unwind across an incompatible boundary. Prefer generated
bindings with recorded input headers and generation versions over handwritten
copies; keep generated files identifiable.

Treat interrupts, scheduler tasks, callbacks, and the low-power processor as
distinct execution contexts. Do not infer synchronization safety from a
single application CPU. Check the selected target's atomic support and runtime
critical-section implementation; document ordering choices and shared-memory
assumptions. Volatile accesses do not provide a locking or synchronization
protocol. Review reentrancy, interrupt masking, nested contexts, DMA ownership,
and `Send`/`Sync` behavior for the actual adapter.

Keep interrupt handlers bounded. State which operations may allocate, block,
sleep, take locks, or run only from a task. Review priority inversion and lock
ordering. Do not hold a blocking lock or interrupt-disabled critical section
across `.await`. Async APIs must state cancellation behavior and what happens to
an in-progress transfer, borrowed buffer, or partially recorded sample when a
future is dropped. Do not add async merely for uniformity across OSes.

Make resource shutdown explicit when it can fail or block, so destructors remain
predictable (`C-DTOR-FAIL`, `C-DTOR-BLOCK`). Test the relevant ownership and
failure paths when they are introduced.

## Diagnostics and measurement behavior

Represent unsupported, timed out, skipped, failed, and passed as distinct
outcomes. A missing feature produces no numeric score; it is never silently
recorded as zero. Preserve raw error codes and useful context at adapter
boundaries. Use `Result` or `Option` according to the contract; avoid panic as
ordinary device, connection, capacity, or input-error handling. Explain the rare
`expect` used for a proved invariant. Tests may assert expected behavior.

Choose and record the firmware panic/reset behavior. A host process recovering
from a device reboot does not mean the benchmark survived it. Keep recovery
outside the measured region and preserve the evidence of failure.

Record logging transport, level, buffering, dropped events, and whether logging
was disabled during timing. Avoid formatting, serial output, heap growth, and
host round trips inside measured regions unless those operations are the
workload. Bound queues and define what happens when they fill. Do not expose
credentials or other secrets in logs, diagnostic formatting, or committed
configuration.

Use a consistent workload definition with specified inputs, seed when relevant,
warmup, repetitions, synchronization points, and completion validation. Keep
timer source, units, wrap handling, resolution, conversion, and observer overhead
explicit. Variable radio conditions remain variable even when a random seed is
fixed. Label deterministic workload inputs separately from environmental
repeatability.

Record build profile, optimization, LTO, clock/power settings, memory placement,
enabled OS services, task/executor configuration, and workload hash. If runtimes
require different compilers or implementations, disclose the difference and
avoid attributing the entire effect to the scheduler. Preserve raw samples and
outliers. OS evaluation uses its [benchmark plan](../projects/os-evaluation/docs/benchmark-plan.md)
to define aggregation and comparisons; other projects record their own methods.

## Verification and review evidence

Run the relevant formatter, compiler checks, lints, documentation checks, and
meaningful tests for supported target/feature combinations. At implementation
time, record the exact commands; a generic `cargo test` instruction does not
establish an embedded test strategy. Use host tests for pure workload logic,
parsers, conversions, result validation, and analysis. Use on-device checks for
interrupt timing, scheduling, peripheral behavior, power transitions, and radio
interactions. Compilation and emulation are useful evidence with narrower scope.

Review compiler and Clippy warnings. Scope a needed lint suppression to its
actual cause and document the reason. Do not impose a global warning denial
without checking the pinned toolchain and generated/vendor code. Do not add
tests that simply repeat implementation details; choose checks capable of
finding a meaningful failure.

Maintain one current API/safety record per component. It identifies execution
contexts, applicable `C-*` decisions, unsafe/FFI and blocking contracts, meaningful
regression checks, and exceptions. Update that record when the contract changes;
record routine commands/results and unrun checks in the commit or CI record.
Carry lasting findings from one-off investigations into the current contract,
test or project plan, then retire redundant process notes. An exception records the rule/source, reason,
smallest affected scope, alternatives considered, consequence, and a revisit
trigger. Routine, justified applicability decisions do not add a user approval
step. A change to the user's language preference or assigned authorities does
need discussion.

## Google guidance compatibility review

The entire pinned guide was reviewed. Supplemental mapping:

| Topic | Resolution here |
| --- | --- |
| Google precedence | User-assigned authorities prevail. |
| Chromium layout, imports, async, threading | Product-specific; inapplicable here. |
| Fuchsia mutex/testing | Product-specific; inapplicable here. |
| Extra formatting options | Stable tools suffice; official conventions govern. |
| Trait implementation scope | Retain `C-COMMON-TRAITS`. |
| Lifetime/generic naming | Retain official/API conventions where recommendations differ. |
| Constructors and examples | Retain `C-CTOR`/`C-EXAMPLE`, including their qualifications. |
| Pre-`main` restriction | Allow required, explicit firmware startup. |

Relevant pinned sections:
[API context](standards/google-rust/rustguide.md#api-design-context),
[unsafe](standards/google-rust/rustguide.md#unsafe),
[initialization](standards/google-rust/rustguide.md#global-initialization).
