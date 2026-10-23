# Evidence and coverage ledger

The ledger's hashes and historical finalized states identify complete
experiment bundles, including compiled artifacts not included in this checkout.
The separate [snapshot inventory](../quality/publication.json) and
[verification scope](reviews/public-snapshot.md) describe the available checks.

[quality/evidence.json](../quality/evidence.json) is the checked inventory of
retained collections, claim scope and platform/domain coverage. It currently
contains **twelve collections, 35 claims, 26 evaluation routes and 35 domains**:
the 25 hardware rows in [hardware coverage](hardware-coverage.md), plus ten
measurement/adoption objectives. Routes include combined stacks, framework
layers and a bare-metal control; they are not 26 independent kernels or 26
locally proved C6 ports.

The full Cartesian product contains **910 platform/domain pairs**. Nineteen
have explicit, scoped **partial** evidence. The other **891** default to
`unverified` / `not_run`. **No complete subsystem is marked qualified.** Unknown
means that this ledger has no established pair-specific evidence, not that a
driver or API is absent upstream. The platform reference sheets preserve the
broader source survey while those details await individual tracing.

## What the classifications mean

These are evidence classes, not a numeric confidence ladder:

| Class | Minimum retained support | What it does not establish |
| --- | --- | --- |
| `source_supported` | Hashed `source` reference and explicit scope | Compilation, board execution or general driver correctness. |
| `build_checked` | Hashed `build_log` for the recorded tuple | UART execution, a correct ABI or product readiness. |
| `observed` | Hashed `raw_capture` reference | Unrecorded events, external timing accuracy or an exhaustive functional test. Rendered monitor text is labeled as such. |
| `operator_reported` | Hashed `operator_record` | An independently retained receipt or a replayable physical observation. |
| `independently_reproduced` | Distinct hashed `reproduction` and `review` files; recorded producer and a different reproducer | Authenticity or statistical independence of those actors, an independent physical rerun, or proof that their interpretation is correct. Scope identifies what was reproduced. |
| `failed` | Hashed retained failure evidence | Universal incompatibility of the platform or every configuration. |
| `unmeasured` | Explicit unmeasured scope and limitation | Any favorable score or implicit hardware support. |

Every non-unmeasured claim pins its support bytes with SHA-256. Changing only
the first Ariel guard's label from operator-reported to independently reproduced
must fail the structural gate: its record has neither the required reproduction
nor review evidence/actor fields. A reviewer must still inspect whether the
files actually support that claim. Supplying false role labels or invented
prose can evade a metadata-only check; hashes cannot evaluate meaning.

Each claim separately records an outcome (`partial`, `qualified`, `failed`,
`not_run` or `blocked`), its exact scope, reporter/producer and limitations.
A source-only or operator-only label cannot qualify physical coverage. A
qualified coverage override additionally needs a qualifying observed/reproduced
claim with `raw_capture` support, known or unnecessary pins, an available or unnecessary fixture, and a
verified driver origin. This schema does not make the human qualification
decision on its own.
Driver/Rust API availability needs nonfailed source, build, observed or
reproduced support; an operator inventory alone cannot establish it. Scoped
operator records may establish reported pin/fixture inventory, while remaining
distinct from a successful hardware test.

## Current physical evidence at a glance

| Route | Explicit partial domains | Important boundary |
| --- | --- | --- |
| Bare-metal Rust control | HP compute, clock snapshots, UART0 output, flash/code placement | Compute is calibration; IRQ behavior, CPU utilization, persistence and the cache mechanism remain unqualified. |
| Embassy on esp-rtos | HP compute/selected async message path, clock snapshots, UART0 output | This combined stack does not qualify an isolated esp-rtos native application-thread API; priorities and models differ from IDF. |
| ESP-IDF / FreeRTOS with Rust | Selected std/native request/reply paths, bounded timer/WFI checks, UART0 output, task-attribution/allocation controls and finite offered-load pulses | Finite pulses retain generator misses, queue losses and synthetic deadlines. They do not establish sustainable capacity, fairness, physical CPU busy/idle, exclusive ISR time, runtime global peaks, energy efficiency or a whole-OS ranking. |
| Ariel OS | Native test markers/assertion paths and UART0 output | The first guard remains operator-attested. Yield lacks a peer-progress counter; 77 ticks is derived arithmetic, not validated switch latency. |
| Zephyr | Native C execution-path/UART output | The native method admits 0/3 boots. The selected Rust hello fails before an image is produced; neither fact proves universal C6 incompatibility. |
| RTIC | UART0 markers and coarse timer-dependent cooperative resumption | The first attempt fails after another ROM startup; two later attempts are not run. Successful substeps do not qualify the failed boot, establish preemption or identify the reset cause. |

All UART overrides concern the known **UART0 GPIO16/17 logging route**. They
do not qualify serial receive, flow control, overload recovery, every UART API
or LP UART. All CPU overrides are narrow tests; no IRQ or utilization result
is inferred from their existence. Other routes retain their survey links and
unknown/not-run domain records rather than borrowed results from an underlying
kernel or related chip.

The ten objective domains keep task/ISR accounting, capacity/fairness/deadlines,
runtime memory, external IRQ response, cold-start/generalization, matched
end-to-end applications, sustained mixed load, recovery/updates, maintenance
effort and the architectural decision visible for every route. Radio, DMA,
LP-core, bus and sleep/energy gaps also remain visible in the hardware domains.

The [finite-load collection](../results/idf-load-qualification-2026-10-23/RESULTS.md)
retains two qualification boots with 4,032 nominal release slots. Original
captures, saved reports and frozen collector remain unchanged. A subsequent
[reader correction](reviews/idf-load-replay-correction.md) fixes a one-ULP
JSON parse false rejection; its executable is recorded separately. That host
repair does not become a new physical attempt or change the frozen contract.

The [RTIC collection](../results/rtic-qualification-2026-10-23/RESULTS.md)
preserves one failed attempt, two not run and unchanged frozen inputs. Separate
build, partial-marker and failed-admission claims keep its useful observations
distinct from a passing program. Its missing second reset-reason line remains
a gap for a new diagnostic, not a reason to infer a watchdog cause.

## Archive identity and historical boundaries

Every current collection is registered separately from its scientific outcome.
A failed collection can have valid packaging; passing packaging cannot convert
failed qualification into admitted data. The verifier compares registered
collections with actual immediate result directories so that an omitted
collection cannot silently disappear from the inventory.

For finalized packaging, `SHA256SUMS` covers every regular file in the collection
except that manifest itself. The check rejects a missing manifest, missing or
changed covered file, duplicate entry, unlisted new artifact and escaping path.
The registered `manifest_base` explicitly selects repository-relative or
archive-relative paths, preserving older manifests without guessing their
base from whichever file happens to exist.
Registered `INPUTS.sha256` files additionally match their pre-capture Git commit
and current input digests. That historical digest record can bind bytes that
were added to Git later; it does not invent a missing earlier Git blob.

A new, unfinished collection can use `collecting`, with no claimed final
checksum manifest. Its current input digests are checked; its freeze commit may
temporarily be null, which produces an explicit warning. That is preparation,
not a frozen primary-capture gate. A qualified claim cannot depend on proof
inside a collecting or legacy-unsealed archive. Finalizing a collection with
an INPUTS manifest requires its real freeze commit; historical observations
must not be assigned a fabricated earlier checkpoint.

The initial collection's new checksum seal records its **current retained
bytes**, added after collection and audits. It is explicitly retrospective,
with no original pre-capture input freeze. The paired IDF, Ariel native and
Zephyr native records retain their actual input freezes. The ledger preserves
this distinction rather than assigning every collection the strongest history.

## Updating and checking

When new evidence arrives, add a scoped claim with its evidence class, exact
support paths/hashes and limits. Add only the affected coverage override;
never change the unknown defaults into a platform-wide pass. Keep failure
evidence and separate a changed profile or method from an earlier failed one.
Refresh a support hash only after reviewing how the changed bytes affect the
claim. A report edit is not merely a checksum repair.

The complete experiment-bundle check is:

```sh
cargo run --locked -p benchctl -- verify-evidence \
  --root . --ledger quality/evidence.json
```

It checks the typed ledger, archive identity/completeness, evidence paths and
hashes, coverage references and supported local Markdown links. It runs without
opening a serial port or fetching external links. Its machine report is the
current coverage summary; do not maintain a second independent claim ledger.
The snapshot inventory records current file identities rather than
another set of experimental claims. Use `verify-public-snapshot` for the
source and textual-result checkout as described at the top of this document.
The link check covers file/directory targets in inline links and reference
definitions, ignoring fenced/inline code and external URIs. It strips local
heading fragments and queries rather than validating anchors. This is a
documented Markdown subset, not a complete renderer. Referenced evidence files
and local file targets must be regular tracked/staged files, not symlinks to
ignored content.

This command is an executable consistency check. Source semantics, physical
qualification, instrument calibration, proof of peer progress and the validity
of a new comparative method still need their own controls and review. See the
[feedback audit](reviews/2026-10-23-feedback-audit.md) and
[utilization-method audit](research/utilization-audit.md) for the defects and
unmeasured questions that prompted these distinctions.
