# Independent guard counterexample

These are host-only preflight inspection fragments and executed outputs, not
physical captures or a maintained standalone test suite. No board was accessed.

`pre-fix-rtic-console.rs.txt` is the actual first collector module, copied before
correction. Its proposed guard was five seconds. The independent appended test
feeds four chunks through the real capture function: startup/entries, then the
three exits. After the last exit at4.1s, the fake host clock jumps to9.1s while a
fifth `PANIC` chunk remains pending. The old collector and its saved-byte replay
both incorrectly pass without reading that chunk. The first output records
reader_calls=4 and pending_panic_unread=true. A passing reproduction test here
means that the defect was reproduced; it is not a correctness pass.

The corrected five-second rerun consumes the fifth chunk and rejects capture
and replay. Subsequently, before input freeze or any physical attempt, watchdog
source review made the prospective guard twelve seconds. The final twelve-second
rerun jumps instead to16.1s and again reads, retains and rejects the pending panic.
The original five-second source, test, clock sequence and outputs are preserved
unchanged; final12 files have separate names and identities.

`proof.json` records exact clock/read sequences, source/test hashes, commands and
outcomes. Source identity files refer to copied actual module snapshots, before
appending the independent test. The corresponding post-fix production source is
retained by the repository and its input freeze; temporary Cargo target folders
and duplicate whole-crate copies are intentionally not archived.

To reconstruct a probe, make an isolated copy of the repository's benchctl crate
with its pinned lockfile and bench-core dependency, retaining the same helper
modules. For the original reproduction, replace only rtic_console.rs with the
retained pre-fix source and append independent-fake-reader-test.rs.txt. For the
final corrected case, use the frozen live module and append
independent-final12-rerun-test.rs.txt. Run Rust1.96.0 cargo test with the matching
independent_loop_top filter and --nocapture. The retained Cargo commands record
the original temporary paths, not paths expected to exist in a fresh checkout.

The permanent live regression covers the same defect and the source now binds
success to an actual timeout poll at/after the guard boundary. Administrative
end-record delay cannot enlarge the observed interval. This closes the observed
counterexample; it establishes neither device liveness nor exhaustive coverage.
