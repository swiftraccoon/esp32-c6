# Reversible CP2102N serial descriptor check

Decision question: can the fitted unlocked CP2102N expose a custom serial through
the documented configuration path, and can its entire original configuration
and serial exposure be restored? The underlying factory-assigned internal ID is
outside this change. Reuse vendor tools and the qualified USB control observer;
no new firmware, radio application or maintained USB transport is needed.

## Tools and qualification

Use the vendor-authored `cp210xsmt` sources from the pinned
[SDK archive mirror](https://github.com/mschuldt/_efm8/blob/56d116793833e9be86355a63bc9675489f6c2e38/USBXpressHostSDK/CP210x/srcpkg/cp210xsmt.tar.gz),
built unchanged for the Mac with the recorded compiler and libusb. Pin the
actual executable, dependencies, source inputs, existing `identityctl` observer,
esptool 4.12 and private one-off composition before device access. This is a
local build of vendor Linux SDK sources, not an official Mac binary release.

The simpler inspected `cp210x-cfg` can select a custom string but does not offer
the exact raw restoration required here. Switching back to the internal serial
selector alone leaves custom-string bytes behind. SMT's full `--set-config`
path preserves the supplied array and has no implicit reset or retry loop;
do not use individual setters, locking, serial allocation or built-in verification.
In particular, omit `--serial-nums`: that path rewrites serial/checksum bytes
before programming rather than preserving the supplied array.
Its parser does not reject a bad Fletcher checksum. Independent exact length,
layout, checksum and modification-whitelist checks are required.

Qualify the actual parser only inside a positively checked USB-denying sandbox.
Qualify the private sequence with successful and deliberately failing controls,
including partial/failed/timed-out writes, malformed readbacks, identity/topology
changes and mandatory restoration. Independent review must pass before a live
write. Preserve failures, original outputs and the prospective method freeze.

## Device binding and baseline

Require exactly one eligible bridge and the intended native C6 interface,
matching saved identities and physical USB topology. Recheck the bridge's
physical location throughout; its serial is expected to change. Do not rely on
the serial alone as the post-write binding. Check serial-port ownership.
Inspect the full host USB registry and require that the only USB device nodes
are the intended board pair. Before every SDK invocation, reject any additional
USB device in the fresh host inventory; recovery can proceed with only the
intended bridge if the native interface is temporarily absent.

Enter C6 ROM through the bridge using a bounded no-stub `read_mac`, validate its
factory identity, and leave it in ROM. Capture/replay the complete fixed USB
control plan using the existing qualified observer. Run its capture/replay/audit
commands from the workspace root with workspace-relative
`private/device-identity/...` directory arguments. The existing path guard
rejects absolute paths before capture. Qualify this real CLI boundary as well
as mocked subprocess behavior. Require part byte `20h`,
the original fresh serial, native factory-MAC serial, and an exact 678-byte V1
bridge configuration with valid Fletcher16 and update-enable byte 4 equal to
`FF`. Retain the original array and a complete exact-restore SMT file before any
candidate write. The bridge and native C6 are separate identity domains.

## Single custom-string trial

Create one synthetic 32-character ASCII alphanumeric serial. Starting from the
fresh original 678-byte array, change only the custom-serial selector at byte
449, descriptor length/type at 450–451, its UTF-16LE bytes from 452, and the
big-endian Fletcher16 at 676–677. Preserve unused string padding and every
unrelated setting, including update-enable, VID/PID, GPIO and UART settings.
Independently check the allowed differences and checksum. Freeze both exact
configuration files and the original/candidate arrays before the write.

Use only the part-20 and original VID/PID filter with `--device-count 1` and
one full `--set-config` command. Treat invocation as a possible write even on
nonzero status or timeout. Then explicitly reset/re-enumerate the bridge and
repeat ROM entry and complete fresh USB capture/replay. Require the exact
candidate array, unlocked byte, custom serial and unchanged native serial.
Cached host properties or a setter's zero exit status are insufficient. Before
writing, require the original cached bridge serial. For observations after a
possible write, allow only the known original or candidate cached serial,
retaining all physical topology, population and ownership gates. This permits
fresh requests when the host property lags in either direction. The fresh
candidate/restoration serial and all configuration bytes must still match their
expected state exactly; metadata never establishes programming success.
The original and candidate strings have the same length; sanitized summary
metadata can therefore be identical. Compare decoded raw serial contents.

SMT reset/list branches bypass its count validation, and the reset branch
discards the underlying reset return status. External sole-device/topology gates
and independent fresh descriptor/configuration readback therefore remain
mandatory. Bound SDK children externally because its USB calls use unbounded
timeouts. Preserve partial output and receipt completeness explicitly.

## Mandatory restoration and effects

After any attempted candidate write, run exactly one scheduled original full-array
restore, explicit reset/re-enumeration, ROM entry and independent fresh USB
capture/replay. Require all 678 bytes to equal the original, the original USB
serial and unlocked state. Do not substitute internal-selector restoration for
byte equality. Keep failure sticky even when recovery succeeds; do not retry the
candidate, automatically issue a second restore write or omit a failed cohort.
If restoration fails, investigate that failure and recover the saved original
before any further identity experiment. SMT's pre-write classifier requires a
readable V1 configuration; this method does not guarantee automatic recovery
from arbitrary configuration corruption or a missing USB device.

Finally perform a matched ROM-identity check and warning-free normal C6 hard
reset, including after earlier failures. Signal/timeout handling must preserve
the restoration obligation and permit recovery even if the native interface
temporarily disappears. Do not let receipt-writing failure prevent physical
recovery.

The bridge's configuration memory is programmed and restored. SDK enumeration
may open descriptor-compatible USB peers and query part numbers before applying
its part/VID/PID filters; it has no initial CP210x VID/PID whitelist and may
retain candidates after descriptor-read errors. Those filters restrict
programming but do not restrict all pre-filter opens. The Mac libusb
backend can seize devices and has descriptor/reconfiguration fallback paths;
enumeration is not physically passive. Bridge re-enumeration and DTR/RTS changes
can reset the C6. Esptool additionally runs common SPI/XMC startup/reset hooks.
No eFuse burn, configuration lock, key deletion, flash-array programming or
intentional unrelated configuration change is part of this method. Subsequent
original-application PHY persistence and independent application health remain
unqualified. Publish only sanitized relations/counts; raw configurations,
descriptors, serials, topology and complete output remain private.
