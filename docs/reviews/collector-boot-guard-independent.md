# Independent review of collector boot-text admission

Reviewed **2026-10-23**. Scope: the reproduced non-JSON admission defect,
the shared Rust capture/replay repair, its tests and the retained protocol
archives. The reviewer did not edit implementation, existing result files,
frozen inputs or hardware, and did not commit changes. Temporary mutated input
copies and CLI outputs are confined to ignored `.cache` directories.

The repair addresses a demonstrated admission bug. The checks below establish
its behavior for the inspected source and tested inputs; they do not establish
that every possible failure format or unobserved device event is detected.

## Pre-fix behavior reproduced independently

The preserved old executable has SHA-256
`1b5e71a6648086369ba8f4af53e7e1622a08d990fab24448a46126abcfd4277a`.
Its parsers discarded every non-JSON line before protocol validation. The
actual capture loops used the same policy, so finding valid JSON later could
conceal earlier retained panic/reboot text or more than one ROM startup.

Using copies of real protocol 1, 2 and 3 transcripts, the reviewer independently
tested these seven mutations in each version:

| Mutation | Old replay behavior |
| --- | --- |
| Guru Meditation panic before the ROM banner | Accepted. |
| `Rebooting...` before hello | Accepted. |
| Another C6 ROM banner before hello | Accepted. |
| `abort()` diagnostic among measurement records | Accepted. |
| Another `rst:` reason among measurement records | Accepted. |
| Rust `panicked at` after completion, with newline | Accepted. |
| `Rebooting...` after completion, without newline | Accepted. |

All **21 injected replays wrongly exited 0**, with validation stdout
byte-identical to their valid controls. The temporary fixtures and matrix are
under `.cache/collector-independent-review`; no original transcript was edited.

The reviewer also reran the author's copied real queue study. One altered raw
file contains two ROM banners plus Guru Meditation and reboot text; every one
of its 24 `run.json` files remains byte-identical to the original study. The
old analyzer still returned **12 complete pairs / 24 admitted boots**, with
the complete-study interval. This demonstrates a study-admission consequence,
not just a parser-unit discrepancy.

There is no captured-serial-digest field in the existing run-report schema.
The analyzer derives a raw digest, and archive manifests separately retain raw
file identities. No nonexistent digest field was bypassed or modified in this
reproduction, and no report-schema change is needed to repair the text guard.

## Source and state review

The [implementation review](collector-boot-guard.md) records the coding-standard
and API-applicability delta against the existing 54-entry reviews. Independent
inspection covers the new
[`boot_text.rs`](../../tools/benchctl/src/boot_text.rs), both adapters and the
study caller:

- Both actual serial-port capture functions delegate to their typed
  `capture_stream` adapter, which calls the shared byte-reader/guard. Both replay
  paths use the same framer/guard; this is not a validate-only repair.
- The guard requires one recognized C6 ROM startup before JSON. The chip token
  has a boundary, so `esp32c61` is rejected. Additional ROMs, repeated reset
  reasons and reset reasons after JSON are rejected.
- Non-JSON diagnostic markers are checked before passing a line to protocol
  parsing. JSON is treated as protocol data, including diagnostic words inside
  legal metadata; the existing typed record validation still applies afterward.
- Framing remains bounded to 4,096 bytes per line. A complete received chunk
  is written to raw output before its bytes are admitted. Later unread chunks
  are outside that retained observation.
- A rejected stream is poisoned. Actual capture adapters also revoke the typed
  boot's numeric-summary eligibility on any stream error, including a text
  error after completion was already parsed. A later valid record cannot
  restore the rejected attempt.
- Completion consumes the rest of the received chunk. A nonempty pending line
  is buffered until newline, EOF or timeout; an ordinary line split across reads
  is not rejected merely for its segmentation. Unterminated trailing text at
  known EOF is rejected. A valid final completion JSON can be parsed at EOF
  without its newline.

The implementation adds no unsafe code, FFI, dependency, firmware, concurrent
execution context or public Rust library API. The private state owns its bounded
line buffer and borrows its reader/writers. Errors remain `Result` values;
ordinary input failure is not implemented by panic. The actual serial readers
retain their 100 ms read timeout; the overall deadline is checked between reads,
not a preemption guarantee for an arbitrary blocking `Read` implementation.

## Independent repaired-binary checks

The reviewed repaired release executable has SHA-256
`7085c56532c1c7559b8ce29eebf817132b9755067ec244a2961400776dd140d4`.
The reviewer independently ran these checks:

| Check | Observed result |
| --- | --- |
| Same 21 mutated real transcripts | All exit nonzero, without favorable validation JSON on stdout. |
| Legal protocol-1 hello metadata containing `panic: abort`, Guru Meditation and reboot words | Accepted; JSON text does not become a console diagnostic. |
| Wrong `esp32c61` ROM token | Rejected. |
| Complete ordinary post-completion text line | Accepted. |
| Same altered real-study copy, unchanged reports/images/provenance | Incomplete: **11 complete pairs / 23 admitted boots**, no complete-study interval. The invalid attempt's reason is the fatal-text guard, not a hash mismatch. |
| Original unchanged real-study analysis | Complete stdout byte-identical to its archived `summary.json`. |
| All 61 original protocol raw files | All validate; stdout is byte-identical both to the old executable's output and each archived `replay.json`. |
| `cargo +1.96.0 test --workspace --locked` | **44 unit tests and two doctests pass**: 42 collector tests and two shared-core tests. |

Source inspection and the executed tests confirm that fake readers exercise
the actual typed legacy and queue capture adapters, in addition to the shared
guard's tests. They cover pre-JSON failures, permanently rejected state,
already-complete boot revocation, chunk-fragmented diagnostics, pending ordinary
and fatal post-completion lines, missing/wrong/duplicate startup and line bounds.
The study regression starts from a complete valid fixture and changes raw text
only, leaving valid reports/images/provenance intact; the interval is then
withheld. These are host ingestion tests, not serial-driver/reset-waveform or
new on-board experiments.

The author additionally records formatting, warning-denied Clippy, documentation
and release-build checks in its implementation review. The independent review
ran the workspace tests and CLI checks above and inspected the source; it does
not relabel author-run checks as separately executed reviewer checks.

## Retained-file scan and exact scope

A separate read-only scan enumerated **61** `results/**/serial.raw` files,
14,161,558 bytes in total:

| Dataset | Files | Protocol |
| --- | --- | --- |
| Initial compute/observer | 15 | 1 |
| Clocked handoff | 15 | 2 |
| Alignment probe | 5 | 2 |
| Queue qualification pilots | 2 | 3 |
| Matched queue study | 24 | 3 |

Totals are **15 protocol 1, 20 protocol 2 and 26 protocol 3**; queue backends
have thirteen files each. The 54-file subset discussed earlier omits the five
alignment files and two queue pilots. Native Ariel/Zephyr rendered console logs
are not this JSON protocol archive set.

Each retained raw file contains one literal `ESP-ROM:` occurrence, one `rst:`
line and one JSON hello; its ROM/reset lines precede hello. The reset text is
`rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)` in every file. Maximum
CR/LF-split line payload is 854 bytes. These are file observations, not proof
of physical power cycling or complete capture of every device event.

The scan split CR/LF framing, trimmed lines, parsed JSON-leading lines and
excluded JSON text from a case-insensitive diagnostic-pattern search. Its
explicit families were panic/panicked-at, Guru Meditation, abort-called,
rebooting/restarting, assertion-failed, fatal, brownout, watchdog/WDT, heap
corruption, stack smashing/canary/overflow and SHA-comparison failure. It found
no matching non-JSON line or malformed JSON-leading line in these files. This
pattern scan is distinct from the repaired guard's exact marker policy; the
archive contains no JSON panic text, so synthetic legal-metadata controls are
still necessary. The negative finding does not prove that no failure occurred
outside retained bytes, or that no unrecognized diagnostic exists.

## Compatibility and remaining boundaries

The new admission rule deliberately rejects JSON-only transcripts without their
ROM startup. `serial.raw`, rather than `records.jsonl`, is the replay input.
This is a stricter CLI contract requiring the repository's breaking-change
commit/footer treatment; the JSON wire schema and summary serialization remain
unchanged. Historical collector identities and original captures must remain
preserved; a stricter later replay does not mean this guard ran during those
original physical captures.

The guard is a conservative marker policy, not a semantic classifier for every
possible console sentence or future/custom fault format. JSON metadata is
excluded; recognized substrings in non-JSON prose reject the attempt. It cannot
authenticate a printed ROM banner, observe text before acquisition, or detect
diagnostics arriving after capture returns with completion and no pending line.
Replay checks all retained bytes through EOF; capture has a finite observation
boundary. No post-completion dwell, physical serial test, independent timing
reference, retrospective absence of all reboots or general completeness claim
is established by this repair.
