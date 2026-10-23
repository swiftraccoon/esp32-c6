# Collector boot-text admission repair

Reviewed **2026-10-23** against the [coding standard](../coding-standard.md),
[all 54 API entries](../rust-api-checklist.md), the complete
[protocol-2 applicability matrix](benchctl-v2.md),
[queue protocol review](benchctl-queue.md) and
[study review](queue-study.md). This is the existing internal Rust 1.96.0,
edition/style edition 2024 host binary, with no dependency, unsafe code,
firmware, hardware or JSON wire-schema change.

## Reproduction before repair

The reported defect is confirmed. Both parsers previously returned `None` for
every line outside JSON. Firmware emits its first JSON after measurement and
cleanup, so a panic/reset before that record could be omitted from admission.
Ordinary protocol ordering and distinct nonce checks did not close this gap.

The preserved old release executable has SHA-256
`1b5e71a6648086369ba8f4af53e7e1622a08d990fab24448a46126abcfd4277a`.
Under the ignored `.cache/collector-boot-guard` directory, an injected ROM/panic/
`Rebooting...` prefix was followed by an unchanged valid transcript:

```sh
.cache/collector-boot-guard/benchctl-before validate \
  --input .cache/collector-boot-guard/legacy-injected.raw --platform bare-metal
.cache/collector-boot-guard/benchctl-before validate-queue \
  --input .cache/collector-boot-guard/queue-injected.raw --backend freertos-queue
.cache/collector-boot-guard/benchctl-before analyze-queue \
  --schedule .cache/collector-boot-guard/study-before/schedule.json \
  --results .cache/collector-boot-guard/study-before
```

Both direct replay commands wrongly exited **0**. The copied real study still
reported **12 complete blocks / 24 validated boots**, including its conditional
interval, after one raw transcript was prefixed. No original artifact, report,
image or provenance was edited. No run-report serial-digest field was bypassed:
the existing report has none. Raw digests are retained separately by archive
manifests and derived study evidence; that is a distinct integrity boundary.

## Shared repaired path

`boot_text.rs` now owns the bounded framer, startup state and text rejection.
Both actual serial capture loops and both archived replay paths use it. A
single recognized C6 ROM startup must precede JSON. A second startup, wrong
chip token, repeated reset reason or a reset reason after JSON permanently
rejects the stream. Known fatal/reboot diagnostic markers are checked only
outside JSON; legal protocol metadata containing those words remains data.

The 4,096-byte line bound remains. Every complete received chunk is written to
the raw artifact before parsing. Protocol parse/validation errors, text errors,
I/O errors and incomplete input produce no success summary. Capture wrappers
also revoke an already-complete `Boot` on a later stream error, preserving its
diagnostic fields without allowing its numeric summary to be reused.

Once completion is parsed, all remaining received lines are consumed. If the
chunk ends in a partial line, capture waits for its terminator within the
remaining deadline. Ordinary text gets the same result whether fragmented or
not; fatal fragments cannot disappear at a read boundary. A trailing incomplete
line at known EOF fails. Replay permits a valid final completion JSON without
a newline when EOF establishes its end, while rejecting any later records.

Capture's observation still ends after validated completion and the rest of
the received lines. It cannot reject bytes arriving after return, establish
unobserved failures before connection, authenticate a ROM banner, or recognize
every possible custom/future diagnostic format. No extra post-completion dwell
or physical reset-waveform test is claimed. Tests use fake readers through the
actual shared capture path; no board was opened, reset or flashed.

This deliberately tightens CLI replay compatibility: JSON-only `records.jsonl`
input cannot prove the single-boot requirement. A full `serial.raw` transcript
is required. The wire record schema and summary serialization are unchanged;
the breaking CLI behavior needs the repository's explicit breaking-change
commit/footer treatment.

## API applicability delta

All 54 previous matrix dispositions were reconsidered. Unlisted decisions
and the existing metadata/license exceptions remain unchanged; no new exception
or pending decision is introduced. This change adds no public Rust library,
macro interface, iterator, feature, conversion/operator, smart pointer,
generic data structure or custom destructor.

| Affected entries | Decision / evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: `BootTextGuard`, `Stream`, `MAX_LINE_BYTES`, `replay`, `capture` and explicit byte/time fields retain Rust naming and units. |
| C-COMMON-TRAITS | Applies: private state derives field-bearing `Debug` and valid pending `Default`; live state is not cloned or copied. No named constructor interface is added. |
| C-SERDE | Existing wire/report decisions apply; new live stream/guard state is not interchange data and gains no serialization. Original report schemas are unchanged. |
| C-SEND-SYNC | Applies: production state and borrowed readers/writers are exclusively used in one host thread; no manual sharing assumptions or traits are introduced. |
| C-GOOD-ERR, C-FAILURE, C-VALIDATE | Applies: contextual `Result` for text/framing/record/I/O/timeout failure; stream poison and boot-summary revocation are checked before a pass can return. Ordinary failure does not panic. |
| C-RW-VALUE, C-GENERIC, C-CALLER-CONTROL | Applies: shared helpers borrow `Read`/`Write` streams and accept a bounded line callback, without owning hardware or building a transport framework. Fake `Read` implementations exercise the same capture helper. |
| C-CRATE-DOC, C-LINK | Applies at internal application scope: module/helper documentation and README explain errors, finite observation, stricter CLI input and the original defect. |
| C-HIDDEN, C-STRUCT-PRIVATE, C-METHOD | Applies: guard/framer fields remain private; only crate-internal helpers/constants and `Boot::reject` are visible internally. State checks/revocation stay on their owners. |
| C-NO-OUT, C-INTERMEDIATE | Applies: results return through `Result`; mutable output buffers intentionally preserve raw bytes and parsed records. Failed source observations are not replaced or converted into success. |
| C-NEWTYPE, C-CUSTOM-TYPE | Applies: named guard/stream types separate startup state from protocol validation and raw framing. `Duration` distinguishes timeout values from byte counts. No exported representation wrapper is added. |
| C-DEBUG, C-DEBUG-NONEMPTY | Applies: new state/test reader types derive useful field-bearing `Debug`; errors identify marker categories without echoing arbitrary diagnostic contents. |

## Verification

The completed repaired release executable has SHA-256
`7085c56532c1c7559b8ce29eebf817132b9755067ec244a2961400776dd140d4`.
The old binary, injected traces, before/after stdout/stderr, real-study copy,
61-file replay list and `reproduction-report.json` remain ignored local
verification evidence under `.cache/collector-boot-guard`.

| Check | Result |
| --- | --- |
| `cargo +1.96.0 test --workspace --locked` | **44 unit tests and two doctests passed**: 42 benchctl and two shared-core tests. |
| `cargo +1.96.0 fmt -p benchctl -- --check` | Passed. |
| `cargo +1.96.0 clippy --workspace --locked --all-targets -- -D warnings` | Passed after moving a test-only `io` import into the test module; no suppression. |
| `cargo +1.96.0 doc --workspace --locked --no-deps` | Passed. |
| `cargo +1.96.0 build --release --locked -p benchctl` | Passed. |
| Same two injected CLI transcripts | Both now exit **1**, rejecting the pre-JSON fatal diagnostic. |
| Same injected real-study copy | **Incomplete: 11 complete blocks / 23 validated boots**, conditional interval null. Original passing report/image/provenance bytes are unchanged. |
| All 61 original archived protocol transcripts | **All pass**, and complete validation stdout is byte-identical to every archived `replay.json`: 15 protocol 1, 20 protocol 2 and 26 protocol 3. |
| Original 24-run study analysis | Complete output byte-identical to its archived `summary.json`. |

Meaningful regressions cover fatal diagnostics before JSON, resets/duplicate
ROMs, wrong-chip/missing startup, fragment boundaries, errors after an already
parsed completion, unterminated tails, maximum/overlong lines, permanently
revoked state, ordinary startup/post-completion logs and actual legal JSON
panic/reboot metadata. Separate legacy and queue capture-reader tests use full
protocol fixtures. The study regression alters only raw text while leaving
all passing report/image/provenance files intact; the block becomes invalid
and no complete-study interval is emitted.

These checks establish the repaired behavior for the tested inputs. They do
not prove that historical captures suffered hidden reboots, that every failure
format is recognized, or that the broader investigation is complete/error-free.
