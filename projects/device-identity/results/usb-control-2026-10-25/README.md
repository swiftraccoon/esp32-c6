# USB control-pipe investigation

One complete ROM-state capture obtained both nonempty serial descriptors and
the bridge configuration. Both original-flash-state cohorts failed native
LANGID reads and remain rejected. Original application execution was not
independently verified. The [prospective method](../../experiments/usb-control.md)
separates the two failed baselines from the single conditional ROM diagnostic.
No RF identity or rotation was measured.

| Cohort | Frozen inputs | Captures attempted / planned | Outcome |
| --- | --- | --- | --- |
| [Original request plan](cohort-01-failure.json), `c21e458` | [Inputs](cohort-01-inputs.sha256), [collector](cohort-01-artifacts.json) | 1 / 2 | Native DEVICE returned 18 bytes; LANGID request for 255 bytes returned `kIOReturnNotResponding`. Bridge's five transfers validate as a partial observation only. |
| [Prefix request plan](cohort-02-failure.json), `22e9712` | [Inputs](cohort-02-inputs.sha256), [collector](cohort-02-artifacts.json) | 1 / 2 | Native four-byte LANGID prefix returned the same error. Bridge's six transfers validate as a partial observation only. |
| [Conditional ROM diagnostic](rom-result.json), `8b0af60` | [Inputs](inputs.sha256), [collector](artifacts.json), [ROM tool](rom-tool-inputs.json) | 1 / 1 | Both domains complete; exact four-file capture replay matches saved summary bytes. Separate host reset sequence completed. |

Each original-state cohort stopped on its first failure. Whole-capture replay
and raw-record summary reject both; neither is admitted by dropping its failed
domain. Their input hashes resolve against their original freeze commits.
The declared-length change did not remove the original-state error. The single
ROM observation does not establish its cause, repeatability or cross-state
serial/configuration equality.

The [ROM summary](rom-summary.json) contains counts and fixed states only:
native serial has 17 UTF-16 code units; bridge serial has 32. The bridge returned
678 bytes with layout version one and byte four equal to its documented `ff`
default. This is not a lock interpretation. ConfigSize endianness, checksum byte
order, lock state, uniqueness and programming permission remain unqualified.
Raw descriptors, configuration bytes, unique values and command logs are private.

The observer is macOS `IOUSBDeviceInterface942.DeviceRequestTO`, identified by
[native SDK inputs](native-inputs.json). The collector issues IN requests without
opening or seizing USB devices. This establishes the OS control-request boundary,
not a USB bus trace or independent proof about kernel caching. The separate ROM
entry/cleanup opens the bridge UART and resets the C6; it uploads no stub and
requests no flash/eFuse/bridge persistent-configuration content writes. ROM setup
can attach/reset flash hardware and change volatile flash power state.

The USB collector's source/native inputs were frozen. ROM-tool provenance froze
selected esptool files, but omitted its inherited C3 parent and serial backend;
[additional hashes](postcollection-inputs.json) were recorded after collection
and do not close that transitive-input gap retroactively. The executed runner
set the explicit configuration and one-attempt port-open environment; individual
process receipts saved argv but did not independently record that environment.

Entry, capture and cleanup commands all returned zero. Cleanup completed at
`2026-10-25T04:52:44.734986Z`; its exact expected `no_reset` warning was allowed,
with no unexpected warning/error or unsupported-line report. This supports host
reset-sequence completion and intended release to the original flash application.
Physical application boot, health, radio traffic and postboot storage equality
remain unmeasured. The earlier full-flash restoration belongs to the
[runtime collection](../runtime-table-2026-10-24/README.md), not this diagnostic.

Before collection, 67 workspace tests and three doctests passed, along with
formatting, Clippy, docs and release build. Independent qualification exercised
seven positive records, 64 malformed records, nine corrupt complete bundles,
planner stage failures and native request/ABI/ownership controls. Separate
review checked the actual failed subsets, ROM capture, receipt bindings and
cleanup boundary. Six rehashed semantic corruptions of the actual ROM capture
were rejected with empty stdout, including late native/bridge descriptor,
unsupported-part/layout and cleanup failures.
Synthetic controls qualify the method; they do not substitute for hardware
observations. Read-only archival checks never reopen or reset devices.

`SHA256SUMS` covers every retained regular file in this directory except itself;
paths are relative to this directory. Input manifests instead use workspace-root
paths at their stated frozen commits. The seal establishes retained-file
integrity, not authenticity or reproducibility without the private raw captures.
