# Independent RTIC console admission review

Reviewed **2026-10-23**, before input freeze or physical RTIC qualification.
This agent performed **no board operations**, firmware/collector edits or
commits. Review covered the [contract](../experiments/rtic-qualification.md),
unmodified pinned upstream source, actual
[console collector](../../tools/benchctl/src/rtic_console.rs), shared boot guard,
CLI and whole-archive dispatch. Executed fixtures are host-only observations;
they establish no RTIC board pass.

The selected physical scope is the `monotonic` example only, with three
scheduled warm resets and a stop after the first unexpected rejection.
`sw_and_hw` remains source/build evidence: its GPIO9 input and external
stimulus have not been qualified. The C6 checkout was independently observed
at `f1797184d6168e54c345b17019c237eea70b9ed4` with clean tracked files.

## Source and contract agreement

The actual example prints `init`, three hello/bye pairs and delays each task
for **2/3/4 seconds**, rather than the README's 1/2/3 seconds. All six hello
permutations are admitted; exits must follow the prospectively required
foo/bar/baz order after all entries. Each pacing check uses its own task's
hello, not reset or a shared entry timestamp. UART arrival intervals retain
their stated host/transport uncertainty and do not become device timing results.

The [source review](rtic-source.md) distinguishes priority-zero cooperative
polling and SYSTIMER wakeups from positive-priority preemption. The example
does not call HAL init or emit clock-register readback. It cannot qualify
GPIO response, CPU/flash timing accuracy, idle/sleep residency or networking.
The source's automatic console transport also requires the stated UART bridge
fixture and absence of an active native USB host connection.

Before freeze, watchdog source review increased the proposed post-marker
interval from five to **twelve seconds**, retaining the 45-second ceiling.
That stricter prospective change addresses the known startup-watchdog boundary;
no physical result motivated it. The original five-second counterexample below
is retained with its original inputs, rather than rewritten to the new duration.

## Concrete false-pass finding and correction

The first implementation could finish at a loop-top clock sample without
performing another read. An independently appended test against a cache-only
copy of the **actual module** supplied startup/entries and the three exits,
then left `PANIC` pending as a fifth reader chunk. After the final exit at
4.1 seconds, the fake host clock jumped to 9.1 seconds. The first collector
returned success after four reads; saved-byte replay also passed because the
pending diagnostic had never been collected.

| Executed setup | Capture succeeds | Reader calls | Pending panic unread | Replay succeeds |
| --- | --- | ---: | --- | --- |
| Original five-second implementation | Yes | 4 | Yes | Yes |
| Corrected five-second implementation | No | 5 | No | No |
| Final twelve-second implementation, clock jump to 16.1s | No | 5 | No | No |

The correction requires an actual timeout poll at or after the complete
post-marker interval. Reads continue until that poll, including pending text
after the minimum time has elapsed. An interrupted read is an error, not an
empty observation. Actual capture writes its successful terminal event at the
poll's coordinates. Replay permits a later administrative terminal time but
derives the observed endpoint and duration from the preceding poll, so a host
stall after it cannot inflate the claimed observation.

The [retained proof and outcomes](../../results/rtic-qualification-2026-10-23/artifacts/host-controls/independent/README.md)
preserve the initial actual source, independent Rust fake-reader test,
clock/read sequence, source identities and before/after outputs. The original
pre-fix module SHA-256 is
`6870fed121f34392a5318de7f8ac72af4ca2cfe51fb9c00c25794c0cdd7b9901`.
Temporary whole-crate copies and Cargo target directories are not published.
These inspection fragments are not a maintained standalone suite; the live
collector retains a regression for the same capture/receipt failure mode.

This is a reproduced code defect found before collection. Its correction closes
that counterexample; it is not evidence that every possible defect was found.

## Final executable controls

Controls used the final twelve-second release collector, SHA-256
`f06cb6e82e1e01e86dba08a5172b5e4cc0b3ebd588f3cd6fd2395cbfbdbc84dd`.
All observations in this section were generated without a real serial device.

- **78 raw/receipt cases:** 16 positives and 62 rejections matched expectations.
  Positives include all six entry permutations, byte-at-a-time and combined
  reads, CRLF, harmless trailing whitespace, inclusive pacing bounds and exact
  poll/end equality. An administrative terminal at 40 seconds still derives
  only the twelve-second interval actually witnessed by the final poll.
- Negatives include every missing/duplicate marker, wrong order, JSON,
  unexpected output, invalid UTF-8, overlong/unterminated text, panic/reboot,
  additional startup/reset and the pinned backtrace crate's exception and
  stack-overflow formats. Receipt controls challenge types, unknown fields,
  offsets, ordering, terminal reasons, missing I/O witness, pacing budgets and
  a final poll **one nanosecond short**, even with an administrative end at
  the nominal boundary.
- **30 whole-archive graphs:** three accepted controls and 27 rejections
  matched expectations. Original input/hash checks, missing receipt/replay,
  retagged experiment/platform, reserved null fields, altered derived output
  and insufficient final polls were exercised through actual archive replay.
  One accepted control is a retained failed capture and remains counted as
  failed, with zero successful console captures.
- **12 pre-serial CLI controls** reject comparison purpose, invalid boot/order,
  existing output, invalid/missing/nonloadable/wrong-architecture ELF, missing
  literal output and invalid provenance before creating output or opening a
  port. The existing-output sentinel remains unchanged.
- An actual capture-wrapper invocation against a deliberately nonexistent
  port retains original ELF/provenance bytes, errors and null result fields.
  Giving that host-only fixture complete favorable raw/receipts afterward does
  not promote its original failed status during replay.

The archive fixtures use the actual compiled ELF as an artifact container and
explicit synthetic provenance/raw/receipts. They are not simulated device
executions, physical resets or measurements of the upstream program.
The compact outcome records retain case identities, hashes, exit statuses and
diagnostics; temporary fixture generation is auxiliary inspection.

## Failure and evidence boundaries

The new plain-console route explicitly marks recognized application start in
the shared boot guard; it does not invent JSON markers or bypass startup
validation. Unknown application text and known fatal/extra-boot evidence reject
permanently. Full received chunks are retained before admission. Passing capture
also derives from the actual saved raw/receipt files; in-memory completion alone
does not finalize a report. Failed or incomplete packaging cannot count as pass.

Archive dispatch recognizes the experiment, RTIC platform or reserved receipt
artifact, including failed states. Simple retagging or deleting a receipt cannot
bypass the console-specific input/failure checks. Explicit reserved null fields
cannot be replaced by omission. These are consistency checks, not authentication
against someone deliberately fabricating all evidence and identities.

Independent verification also ran `cargo +1.96.0 test --workspace --locked`:
**110 unit tests and two doctests passed**. The final release reproduces all
**66 existing JSON protocol transcripts and one prior study**, separately
counting zero physical console captures at this preflight stage. The
[host applicability review](rtic-host.md), existing 54-entry inventory and its
affected `C-*` dispositions were read. New inspection tests are private host
fixtures with no unsafe/FFI or public API; their local assertions fail fast and
their scope/output is documented here.

The reviewed host admission controls are complete for this stated preflight.
Exact flash-image/bootloader validation, input/schedule freeze, ledger binding,
current full-image backup verification, physical attempts, restoration and
independent result review remain integration obligations. Console silence
cannot establish continued liveness, long-term reliability or an OS ranking.
