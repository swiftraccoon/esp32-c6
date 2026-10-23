# Validation of external review feedback

Reviewed **2026-10-23** against pinned source, exact artifacts, retained console
bytes and a fresh independent source audit. The four initial feedback claims
are supported, with the evidence boundaries below. This corrects earlier review
errors; it is not a new benchmark or a declaration of completeness.

| Claim | Finding and action |
| --- | --- |
| Zephyr's required frequency header cannot appear | Confirmed. `TC_PRINT` is disabled by the selected verbose configuration, the literal is absent from the ELF and all three consoles lack it. All three attempts remain failed under frozen v1; no acceptance rule is waived. |
| Observer policy was misreported | Confirmed: 19 of 47 default rows subtract the estimate; 28 do not, and their endpoints differ. The fresh [source audit](../research/zephyr-native-audit.md) maps every row. Blanket corrected-timing descriptions are superseded. |
| Actual Rust failures were missing from the feasibility report | Confirmed as a reporting gap at review time. The completed [build archive](../../results/zephyr-rust-build-2026-10-23/README.md) now retains all three attempts: workspace rejection, 27 E0080 layout errors, then nine E0425 getter errors. No image, board pass or universal C6 incompatibility is claimed. |
| Ariel's first observation guard was overstated | Confirmed. The five-second wait is operator-reported in metadata; its actual wait receipt is absent from the archive. The console proves its retained output, not that physical guard independently. Original reports are preserved and the [results](../../results/ariel-native-tests-2026-10-23/RESULTS.md) now distinguish evidence strength. This does not prove the guard failed. |

Ariel's yield source intends to rotate equal-priority threads, but the test has
no peer-progress counter or per-iteration acknowledgment. The observed 155-tick
batch mean and upstream derived 77 value do not validate a switch count or an
isolated switch latency. Priority/lock assertion markers remain separate,
stronger checks of their particular control flows.

Two additional defects emerged. The first Zephyr inspection counted 24 rows
because multiline whitespace consumed adjacent lines; per-line replay finds all
47, while preserving the original reports and failed admission. The ROM also
reports an image SHA comparison failure in every capture. The frozen image
reproduces its hash/expected bytes, and the RAM-only image layout explains the
warning path; transfer verification does not settle image-integrity admission.
See the [failed qualification report](../../results/zephyr-native-tests-2026-10-23/RESULTS.md).

The follow-up hardware-utilization feedback is assessed separately in the
[utilization audit](../research/utilization-audit.md), including exact raw-tick
arithmetic, allocation paths and the limitations of a proposed load sweep.

## Reproduced collector admission defect

The later code feedback is also confirmed. Before repair, both validators
discarded every non-JSON line. An injected panic/reboot/second-startup prefix
could precede otherwise valid JSON and still pass. A copied real queue study
continued to report twelve complete blocks after one raw transcript was altered.
The original dataset was not modified for that reproduction.

A shared Rust boot-text/framing path now serves both actual capture helpers
and both replay paths. It requires one recognized C6 startup before protocol
records, permanently rejects known fatal diagnostics and additional boots,
and revokes numeric summaries after stream failure. Capture preserves complete
received chunks before admission. The [repair review](collector-boot-guard.md)
records regression tests, CLI compatibility and finite observation limits;
the [independent review](collector-boot-guard-independent.md) checks the change.

The same injected legacy/queue traces now fail. The copied study reports
**11 complete blocks / 23 admitted boots, with no full-study interval**.
All **61 retained protocol transcripts** replay with byte-identical numerical
output, as does the original study analysis. Their explicit pattern/startup
scan found one ROM startup each and none of the searched fatal markers. This
does not prove the absence of unrecorded failures or every possible diagnostic.

The reported packaging gaps occurred during active report corrections. The
completed manifests now verify **85 Ariel**, **61 Zephyr native** and **48 Rust
build** artifact digests. Frozen input files and original capture reports/bytes
remain preserved. Checksums establish artifact identity, not physical correctness.

## Remaining objectives, consolidated

| Area | Evidence still needed |
| --- | --- |
| Coverage | A populated platform-by-subsystem ledger linking actual drivers, Rust APIs, board pins, fixtures and outcomes. The hardware worksheet remains a coverage contract, not completed qualification. |
| CPU and capacity | Task/ISR attribution, physical idle/sleep residency, independent offered load, sustainable completions, backlog, drops, fairness/starvation and deadline misses under declared policies. |
| Memory | Exercised stack/heap peaks, fragmentation, allocation failures and usable application headroom, separating harness buffers and instrumentation. |
| Timing and generalization | External edge/ISR/application endpoints, loaded jitter, cold-start behavior, independent accuracy and variation across boards/environments. Short warm-reset observations do not establish these. |
| Hardware integration | Carrier pinout and power path; DMA/accelerators, LP core, buses, radios, coexistence and sleep/energy qualification. |
| Reliability and recovery | Sustained mixed work, watchdog/crash recovery, interrupted writes/updates, rollback and persistent-state migration. Lab restoration is a separate capability. |
| Adoption and maintenance | Comparable integration/upgrade effort, patch burden, required protection/dependency configurations and their qualification. Warm-cache build times are not total maintenance effort. |
| Architectural decision | A measured unmet requirement after smaller remedies have been tested. The matched IDF queue result supports investigating an API choice; it establishes no need for a new kernel. |

Existing elapsed-path measurements alone select neither an overall OS nor a
custom runtime. A controlled load pilot is conditional on a valid contract and
reviewed accounting/instrumentation; it does not close this entire table.

The device was restored to its current pre-Zephyr full-image state, separately
digest-verified and reset. No new firmware is flashed to repair these findings.
Known defects and source semantics take priority over a new load experiment;
any corrected acceptance profile must be versioned before prospective capture.
