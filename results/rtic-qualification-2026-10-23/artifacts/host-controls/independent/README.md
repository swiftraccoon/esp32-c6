# Independent RTIC host admission controls

All evidence in this directory is host-only. The current receipts use the final
twelve-second post-marker contract and collector SHA recorded in each outcome
file. No file is a physical RTIC capture.

The CLI outcomes contain16 positive and62 negative raw/receipt fixtures, including
all six hello permutations, byte/chunk/CRLF framing, inclusive pacing/guard bounds,
missing I/O witness, malformed data and administrative-end timing. Archive
outcomes contain3 accepted controls and27 rejections; one accepted control is a
retained failed capture and counts as failed, never as a successful console run.
Preflight outcomes include12 rejections before any port open, an actual wrapper
attempt against a deliberately nonexistent port, failed-status non-promotion and
replay of the66 prior JSON captures/one study. Fixture generation and duplicate
ELF graphs remain temporary inspection assets, not a maintained standalone suite.

The separate guard proof preserves the actual pre-fix false pass and fixed
reruns, including the initial five-second design. That historical setup is
explicitly distinct from the final twelve-second prospective contract.
