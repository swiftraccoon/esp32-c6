# Original NimBLE live identity state

Decision question: what host IRK buffers, Secure Connections generation flag
and store callback state are present in the original application, and how do
the observed IRKs compare across two explicit chip resets? Reuse the matching
original ELF/BIN, installed Espressif OpenOCD and stock esptool. No firmware
replacement, pairing, target function call or new application is required.

The preceding source/ELF audit identifies the candidate fields; it does not
measure their live values or RNG success. This method observes two reset-separated
boots of that same original application. It is not a population-uniqueness,
entropy, RPA, radio-silence or credential-absence experiment.

## Fixed inputs and device binding

Pin original ESP-IDF 5.4.1, NimBLE revision
`a9a4c701981b86c91fdc63fce4cb7bc05c69caac`, the matched ELF/BIN, OpenOCD
`v0.12.0-esp32-20241016`, its target/interface scripts and selected implementation
sources, esptool 4.12.0, the exact Tcl/host compositions and data admission.
Preserve their digests privately before either live capture. Derive addresses and
sizes offline from ELF symbols. Obtain the expected 256-byte application descriptor
from the BIN load segment at its ELF VMA, preserving the image-conversion ELF-digest
patch rather than assuming raw ELF bytes are identical.

Require the sole intended bridge/native USB pair to match saved serial identities.
Check serial-port ownership, bind OpenOCD `adapter serial` explicitly, and require
the ROM factory MAC before each requested chip-reset release. Unique values,
concrete ports, live buffers and hashes of identifying data stay private.

## Two-boot sequence and selected reads

For exactly two boots, perform a checked esptool ROM MAC query and normal hard
reset, wait three host seconds, then run one bounded OpenOCD capture. Every
separate esptool process uses default reset, 115200 baud, no stub and an empty
configuration file. Its common SPI/XMC startup/reset hooks remain broader than a
pure eFuse read; no flash-array, security-register, eFuse or bridge programming
command is requested.

Use no live GDB client. Disable all three server ports, flash support and RTOS
awareness; use `reset_config none` and disable semihosting in the examine-end
handler. No breakpoint, flash bank, `verify_image`, loader algorithm or target
function call is admitted. Read with stock `dump_image`, in this order:

1. Require a running target, halt and confirm halted state.
2. Save the full application descriptor and compare its exact saved bytes.
3. Save the host sync byte and require the source-defined GOOD value, 2.
4. Save the two 16-byte IRK buffers and five four-byte callback pointers.
5. Save the one-byte SC generation flag; require 0 or 1. Only for 1, save the
   64-byte public key. Exclude the private scalar and negotiated link keys.
6. Save sync again, require 2, resume, poll and confirm running state after a
   further 100 host milliseconds; shut down the debugger.

The baseline exports 311 selected bytes, or 375 with the public key. The nominal
32-bit-aligned selected-read spans total 320/384 bytes, excluding driver-internal
debug/register/MMIO activity. Check the aligned ranges against the private-scalar
extent as well as the requested ranges.

Zero/equal IRKs and nonnull callbacks are valid observations. Record them without
silently imposing random-generation or source-hypothesis success. A clear SC flag
does not establish absence of an earlier failed key-generation attempt. Report
the between-boot comparisons only after both complete captures pass admission.

## Effects, failures and recovery

This is an active debug inspection. OpenOCD initialization temporarily halts and
resumes the core, resets its Debug Module, and accesses CSRs. Its normal C6 halt
handler writes watchdog/interrupt MMIO. The stock memory path may use SBA,
program-buffer loads or abstract access, with scratch-register/cache restoration
on resume. These effects can disturb execution and radio timing. Retain the
exact selected source trace; do not describe the operation as physically passive.

Bound each OpenOCD child to 30 seconds and each reset command to 45 seconds.
Preserve available output, partial dumps and nonzero results. Pipe capture can
be incomplete on timeout/interruption; mark that receipt and reject the attempt.
Require checked
running/resume markers and reject unexpected error, warning, transport or PMP
diagnostics even if process status is zero. Only exact known deprecated-port
notices and the disabled-flash warning are expected. Pinned OpenOCD requires
`shutdown error` for failure; numeric `shutdown 1` does not produce the expected
nonzero exit status in this version.

Abort the whole two-boot attempt on the first acquisition/admission failure.
The Tcl attempts resume/shutdown after handled acquisition errors; external
termination may prevent that cleanup. Retain the failure even if recovery
succeeds. The host always performs an independent final normal chip reset,
including on exceptions, timeouts and signals, to clear transient debug state.
Require matched ROM identity and warning-free reset output. Final-boot key
values remain uncaptured. This recovery does not attest later application
health, RF behavior or unchanged PHY persistence.

Qualify the actual saved-byte guards, lengths, flags, log admission and host
cleanup with passing and contrary controls before collection. Preserve the
initial pinned-version shutdown failure and corrected method. Independently
replay the complete raw cohort and recovery receipts; seal all retained files.
Publish only counts, population/equality observations and qualified limits.
