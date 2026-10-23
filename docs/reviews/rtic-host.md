# RTIC host collection review

Reviewed 2026-10-23 against the [coding standard](../coding-standard.md),
the [54-entry inventory](../rust-api-checklist.md), and the complete
[benchctl applicability baseline](benchctl.md). This records the affected
entry delta for [rtic_console.rs](../../tools/benchctl/src/rtic_console.rs)
and the integration owner's CLI, archival dispatch and shared boot-text changes.
It supplies no on-board result.

## Contract, ownership and failure boundary

The [prospective qualification](../experiments/rtic-qualification.md) reuses
one unmodified RTIC C6 monotonic example. The collector requires explicit
qualification purpose, a nonempty operator boot ID and order 1 through 3.
Comparison, existing output, invalid provenance and an invalid ELF reject
before output creation or serial access. ELF preflight requires a little-endian
RISC-V32 executable with nonempty loadable data and all seven literal markers
inside loadable segments. These checks do not authenticate the flashed image
or prove that arbitrary ELF contents execute correctly.

Started attempts retain original supplied ELF/provenance bytes, `serial.raw`,
`observations.jsonl`, and a schema-4 `run.json`. Initial state is `running`.
Serial setup, reset, capture, explicit file flushing and final derivation from
the actual saved bytes share a fallible outcome path. A successful report
requires saved-file replay, available byte hashes and the complete observation
contract. Failure keeps its error chain and null qualification, console,
summary, benchmark and comparative-score fields when output remains writable.
Disk/process failure can leave incomplete packaging; it cannot establish pass.

The single host thread owns the serial handle, bounded line buffer and parsed
state. Ordinary serial reads time out after 100 ms; the reset request holds RTS
for 100 ms before releasing it with DTR deasserted. The 45-second clock starts
immediately before the RTS-release request. This does not independently measure
the EN waveform or the device's reset instant. Whole received chunks are
written before receipt/text admission, preserving trailing diagnostics from a
chunk that already contains a rejecting line. No retry or resynchronization
rehabilitates a rejected attempt.

Both capture and saved-byte replay use the same bounded console framer and
`BootTextGuard`. Every newline passes through the fatal/extra-boot guard;
the first recognized `init` calls `start_console`. UTF-8, startup prefixes,
the seven markers, all hello permutations and ordered exits are validated
without inventing a JSON protocol version. Before `init`, informational IDF
lines must have numeric `I (...)` timestamps and a known `boot`,
`boot.esp32c6` or `esp_image` tag. Unknown application text, JSON, partial
nonblank trailing text and duplicate/reordered markers reject. Payload lines
are limited to the shared 4096-byte bound. Rejection is permanent.

The shared guard adds the pinned backtrace dependency's `Exception '` and
`Stack overflow detected` fatal formats. Its handling of JSON records remains
unchanged; the new plain-console entry point records that application execution
has started so subsequent ROM/reset text rejects. Existing protocol behavior
and new plain-console behavior have separate regression controls.

## Host timing receipts

Receipts are typed, newline-delimited JSON with unknown fields denied:

| Event | Exact fields besides `event` |
| --- | --- |
| `start` | `at_ns: u64`, `raw_end: u64`, both zero |
| `read` | `at_ns: u64`, `raw_end: u64`; cumulative offset strictly advances |
| `poll` | `at_ns: u64`, `raw_end: u64`; timeout read, offset unchanged |
| `end` | `at_ns: u64`, `raw_end: u64`, `reason` |

Terminal reasons are `guard_elapsed`, `eof`, `read_error`, `timeout` and
`rejected`. Only `guard_elapsed` is eligible for success. A receipt's read
timestamp assigns the receipt time of every newline completed by that read;
split lines are timed when their newline arrives. The validator requires one
initial start, exact raw-byte coverage, monotonic offsets, increasing times,
one final end and no later events. Exact adjacent successful `poll`/`end`
coordinates may share a timestamp. Other equal timestamps reject.

The final nonterminal event must be an actual timeout poll at least twelve
seconds after `bye from baz`. Reads continue until that poll, including reads
after the minimum interval. An interrupted read is a terminal read error, not
evidence of quiet UART. A loop-top clock sample alone cannot end capture.
Actual capture stamps successful end at the final poll time. Replay also
permits a later administrative end within the 45-second ceiling, but derives
`observation_end_at_ns` and `guard_observed_ns` solely from the preceding poll.
Host processing stalls after that observation cannot inflate observed silence.

Each task's own hello-to-bye interval must satisfy its frozen inclusive host
pacing budget. Coalesced markers, implausibly fast delays, missing receipts,
insufficient observation, EOF, read errors and fatal/extra-boot text fail even
when a complete favorable marker sequence exists. Host buffering and polling
remain part of these observations. No device timer accuracy, interrupt latency,
CPU efficiency, positive-priority preemption or power estimate is produced.

## Replay and evidence packaging

The module returns only `qualification` and `console` derived fields.
Archive input verification binds qualification-only admission, original
ELF/provenance hashes, parsed provenance equality, raw/receipt hashes, boot
metadata and explicit null reserved score fields. Failed reports must retain
explicit null qualification/console fields and a nonempty failure reason;
missing fields do not stand in for explicit nulls.

The integration owner dispatches this adapter using the experiment identifier,
`rtic` platform or reserved receipt artifact, including failed captures. Every
derived passing field is compared with actual replay and saved `replay.json`
must be byte-identical to deterministic output. Console successes are counted
separately from historical JSON protocol transcripts. This boundary prevents
retagging a failed console report from bypassing its input/failure checks.
Complete artifact manifests and external physical provenance remain separate
archive obligations.

## API applicability delta

All 54 baseline decisions were reviewed for this change. The table updates
affected entries; remaining baseline decisions and narrowly scoped metadata/
licensing exceptions remain unchanged. No dependency or project-license choice
was added. This unpublished Rust 1.96.0 / edition-2024 binary still exports no
public Rust library API.

| Entries | Decision and evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: named receipt events and `_ns` / raw byte boundaries distinguish host pacing, administrative termination and device timing. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies: receipts/reasons derive Debug, Copy, Eq and serde; marker copies carry only static names and integers. Capture borrows inputs and never duplicates a serial handle. |
| C-SERDE, C-VALIDATE | Applies: typed deny-unknown-fields receipts, exact integer units, chronology, raw coverage, final timeout witness, framing, output order and ELF/provenance identities precede favorable derivation. |
| C-GOOD-ERR, C-FAILURE | Applies: contextual Results preserve ordinary validation/port/file errors. No production unwrap, expectation or panic is introduced. Failed reports retain diagnostics and cannot retain scores. |
| C-RW-VALUE, C-CALLER-CONTROL, C-GENERIC | Applies: the private capture core accepts reader/writers and an injected clock by value, allowing borrowed streams and deterministic fake transport/clock tests. Input paths and caller buffers remain borrowed. |
| C-CUSTOM-TYPE, C-NEWTYPE | Applies: Capture, Receipt and EndReason distinguish operations/statuses; Duration/Instant distinguish host clocks. Named fixed-u64 nanosecond fields have one enforced time origin. |
| C-SEND-SYNC | Applies: exclusively owned single-thread production state; no authored unsafe sharing. Tests use Rc/Cell on one thread and a relaxed atomic only for exclusive temporary-directory names. |
| C-METHOD, C-CTOR | Applies: fallible named capture/validation operations; Console default is a valid empty attempt and methods preserve its irreversible rejection state. Test Temporary::new exclusively creates its owned directory. |
| C-NO-OUT, C-INTERMEDIATE | Applies: derived values return through Results; caller-owned raw/receipt writers preserve partial observations on failure. No automatic evidence repair or resealing occurs. |
| C-CRATE-DOC, C-LINK | Applies at private application scope: module/CLI docs, contract, source review and this record explain reset, blocking, timing and failure limits. |
| C-EXAMPLE, C-QUESTION-MARK | Existing application decision remains: no public Rust API or fallible rustdoc example. Operator CLI examples are documented separately and not claimed physical executions. |
| C-HIDDEN, C-STRUCT-PRIVATE | Applies: validation state, receipt representation, clock seam and framing methods remain private. Only crate-level CLI/replay integration is exposed. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No production destructor added; raw/receipt flushing is explicit and fallible. Test cleanup ignores removal errors solely for its exclusively created temporary directory. |

There is no authored unsafe/FFI, interrupt callback, async cancellation, custom
collection, iterator API, smart pointer or public representation commitment.
Host allocation and blocking I/O occur outside device execution. Upstream
serial/ELF/serde internals retain their own safety obligations; tests do not
establish an exhaustive proof of those dependencies.

## Verification and preflight corrections

The module's eleven test groups cover all entry permutations, complete fake
capture/replay at one/seven/1024-byte boundaries, inclusive pacing limits,
actual poll witnesses and observed-end attribution. Contrary controls exercise
fatal and extra-boot text before/after markers, invalid UTF-8, unknown/duplicate/
misordered lines, missing completion, partial trailing text, overlong lines,
typed receipt corruption, timestamps/offsets, EOF, read errors, interruption and
expiry. Tests also challenge input preflight without port access and retain an
actual nonexistent-port failure with original artifact hashes; temporary
archive inputs challenge retagging, stale identities and failed-score promotion.

Independent review found a real defect in the early five-second design, before
any input freeze or device run:
the first implementation could finish from a loop-top clock after a host stall
without reading pending fatal text. The final timeout-poll requirement fixes
that capture/replay gap. Its regression advances the injected clock immediately
after the final marker receipt while the reader still holds a panic; capture
must read and reject it. A later administrative end is separately tested so it
cannot inflate reported observation time. This finding is evidence for the
review gate, not proof that no further defect exists.

Before physical collection or input freeze, the source review identified the
unmodified example's startup-watchdog dependency and a nominal nine-second
bootloader watchdog configuration. The guard was made stricter: twelve
seconds after the final marker, with the same 45-second ceiling. The permanent
host-stall regression uses the configured guard duration; the original
five-second counterexample remains historical preflight evidence. No observed
board outcome motivated or was admitted by this contract change.

An initial test build exposed a moved test-fixture value and an unnecessary
parenthesis warning; both were corrected. The first timeout-negative fixture
used exactly the inclusive 45-second boundary and legitimately passed; the
expiry fixture was corrected to 45 seconds plus one nanosecond. No acceptance
budget was widened.

Final targeted checks passed with the twelve-second guard:

```sh
rustfmt +1.96.0 --edition 2024 --check tools/benchctl/src/rtic_console.rs
cargo +1.96.0 test --locked -p benchctl rtic_console::tests -- --nocapture
cargo +1.96.0 clippy --locked -p benchctl --all-targets -- -D warnings
git diff --check
```

All eleven module test groups passed. Full workspace verification, independent
CLI/archive adversarial controls, historical replay, final release executable,
physical attempts and sealed-result checks belong to the integration owner and
are not established by these targeted checks.
