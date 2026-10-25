# C6 WebAssembly prior work

Source snapshots inspected 2026-10-23. These are external experiments, with no
local build or physical replication established. Reviewed KTH thesis PDF SHA-256:
`ae482675f1a4dc6ad934040bc2ec3eef2e8f9ee08e0edcd99f82cc43fe0748ef`.

The [2025 paper](https://arxiv.org/html/2512.00035v1) evaluates C6, Pico and
nRF5340; C/Rust guests; WAMR/wasm3 interpretation; and native C. Workloads
include bubble sort and CRC-16. It measures execution time, memory and energy,
using Nordic PPK2 with five-second sleep intervals around work. C6 integration
does not use Zephyr. Repetitions are described without a precise count in the
paper; figure-level results alone are insufficient for tail analysis. It does
not establish interpreter/AOT parity, radio behavior, scheduler latency, or
hardware-enforced isolation. Treat runtime feature claims as hypotheses to
check in the selected runtime version.

The [2024 KTH thesis](https://www.diva-portal.org/smash/get/diva2:1908617/FULLTEXT01.pdf)
is the methodological predecessor cited by that paper. Its C6 execution-time
section describes averaging the first 1,000 results. Appendices contain numeric
tables; references 53–54 explicitly identify `TAOFOR4/webassembly-esp32c6`.
This establishes a traceable companion artifact, but not a complete mapping
from every later paper figure to a particular Git commit or binary.

The [companion repository](https://github.com/TAOFOR4/webassembly-esp32c6/tree/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e)
includes Rust/C guests, generated Wasm/WAT, IDF configurations, host collection
scripts and some output files. No top-level license or single run manifest
was found. Some [upstream-derived files](https://github.com/TAOFOR4/webassembly-esp32c6/blob/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e/wamr/main/CMakeLists.txt)
retain Intel Apache-2.0-with-LLVM-exception notices; that does not establish a
blanket license for all experiment code. Files below are valuable integration references, with specific
issues to resolve before reproducing a result:

| Pinned artifact | Finding and implication |
| --- | --- |
| [WAMR host adapter](https://github.com/TAOFOR4/webassembly-esp32c6/blob/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e/wamr/main/main.c) | Module loading/instantiation precede timing. The timed `app_instance_main` delta comes from `esp_timer_get_time()` but prints a `ms` suffix. Preserve raw values and resolve the unit; do not accept labels without inspecting conversion. Stack and heap arguments are each 32 KiB. |
| [Capture script](https://github.com/TAOFOR4/webassembly-esp32c6/blob/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e/wamr/python-scripts/capture_execution_time.py) | Regex expects an IDF `Execution time: ... microseconds` log, different from the inspected host's `Elapsed: ... ms` print. The checked-in parts require reconciliation; neither source presence nor an empty output is a completed replication. |
| [WAMR Rust guest](https://github.com/TAOFOR4/webassembly-esp32c6/blob/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e/wamr/esp32c6-wasm-rs/main.rs) and [wasm3 Rust guest](https://github.com/TAOFOR4/webassembly-esp32c6/blob/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e/wasm3/esp32c6-wasm-rs/main.rs) | Current mains sort 1,000 and 100 integers respectively. Neither exposes the sorted result. This is not evidence that the published matched experiments were wrong; it means this snapshot is not itself a ready-to-run matched case. Inspect optimized Wasm and enforce a result checksum to prevent eliminated work. |
| [Rust guest build](https://github.com/TAOFOR4/webassembly-esp32c6/blob/1f7620875fcb47b41ace49e098b1d9ca6ca57d6e/wamr/esp32c6-wasm-rs/build.sh) | Uses `rustc -C opt-level=3`, panic abort and the historical `wasm32-wasi` target name, without pinning rustc. Freeze a currently supported target/ABI and all guest/runtime versions for a new run. |

**Decision relevance:** a Wasm layer is additional execution/sandbox/update
machinery on a host runtime, not a competing kernel. Do not spend time merely
confirming that interpretation slows a sorting loop. If dynamic code loading
becomes a requirement, reuse the integration approach and add validated Rust
guests, cold load/instantiation cost, warm invocation cost, actual peak memory,
failure containment, host-call permissions, and coexistence with a periodic
deadline task. Keep interpreter and AOT cases separate. Test functional bounds
and traps; fast compute alone does not establish a useful sandbox.
