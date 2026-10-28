# Read-only identity inventory

Decision question: which factory identity fields and USB serial descriptors are
readable by a connected host on this board, before attempting runtime changes?
This is a baseline inventory, with no performance measurement or comparison
between firmware configurations. The observer has attached USB and ROM download
access. It does not represent a nearby passive receiver or a paired peer.

The closest prior observations are the [hardware probe](../../../docs/hardware.md)
and wireless explorer's [USB checks](../../wireless-explorer/docs/iphone-usb.md).
Reuse their port map and installed upstream tooling. Collect current descriptors
and explicitly selected factory fields; historical model/flash observations do
not establish the present serial values or optional-field contents.

## Inputs and qualification

Use the project's Rust 1.96.0 host workspace and committed dependency lock.
The source/schema baseline is ESP-IDF 5.5.6 commit
`880a203ae603d08a94d06aa01e0d6761958e5e9e`. The locally installed SDK Python
environment supplies esptool/espefuse 4.12.0; these use underscore command names.
Do not silently substitute the globally installed esptool 5.1.0 command syntax.
Record actual host/tool versions and SHA-256 identities in the private run
manifest, together with command status, UTC start/end times and source identities.

Before hardware collection, run the actual `identityctl` consumer on synthetic
passing and failing JSON, including malformed/absent schemas, protected fields,
invalid field sizes, multiple USB matches, and private sentinels in ignored
properties. Run the SDK's virtual eFuse whitelist capture through the same
consumer. A separate reviewer constructs contrary inputs and records outcomes
in the result ledger. Qualify save/replay by saving stdout, re-running on the
same private input and comparing the complete derived bytes. Reject capture
errors even when a partial JSON file exists. Never repair a failed observation
into a passing result.

## USB host capture

Create a fresh directory under ignored `private/device-identity/`, using a
restrictive umask. Preserve command stdout/stderr and exit status. On this host:

```sh
umask 077
usb_status=0
system_profiler SPUSBHostDataType -json -timeout 15 > "$capture_dir/usb.json" 2> "$capture_dir/usb.stderr" || usb_status=$?
printf '%s\n' "$usb_status" > "$capture_dir/usb.status"
```

Set `capture_dir` to the new private run directory before using this example.
Require status zero before summarizing; preserve nonzero attempts as failures.
The obsolete `SPUSBDataType` can return an empty object with success on newer
macOS. `identityctl` rejects an absent supported schema. An accepted empty
controller list establishes only that this snapshot reports no matching device.

`usb-summary` recognizes native Serial/JTAG VID/PID `303a:1001` and the
CP2102N bridge `10c4:ea60`. It counts matching device entries and classifies serial
properties as present, empty or absent. It does not publish serial values, names,
USB topology, port paths or hashes of private input. Multiple matching devices
remain multiple entries; model IDs alone cannot associate them with one board.
Property absence is not proof that the device has no serial descriptor.

## Factory fields and device effects

Select the UART bridge from private host discovery; keep its concrete path out
of public commands and records. Check no other process owns that serial device.
Use only the installed `espefuse.py` `summary` command with an explicit whitelist:

```sh
efuse_status=0
"$idf_python" -m espefuse --chip esp32c6 --port "$serial_port" --baud 115200 \
  --before default_reset summary --format json --file "$capture_dir/efuse.json" \
  MAC MAC_EXT OPTIONAL_UNIQUE_ID \
  > "$capture_dir/efuse.stdout" 2> "$capture_dir/efuse.stderr" || efuse_status=$?
printf '%s\n' "$efuse_status" > "$capture_dir/efuse.status"
```

`idf_python` names the pinned SDK environment's Python executable. Validate the
actual version before collecting. Preserve selected MAC fields and the optional
128-bit ID privately. Never use an unrestricted all-field summary: readable key
material and unrelated configuration are outside this inventory.

Entering ROM download mode resets the C6 and interrupts its application. The
summary command closes the port without a final application reset. Release it
with an explicitly recorded esptool operation and hard reset, including
after a failed capture. Record a subsequent USB snapshot and the reset command
status. This is application restart evidence, not a bytewise flash restoration
or a claim that the original application's persistence cannot change on boot.
No new firmware, flash erasure, eFuse programming, bridge configuration writes
or key changes belong to this method.

```sh
reset_status=0
"$idf_python" -m esptool --chip esp32c6 --port "$serial_port" --baud 115200 \
  --before no_reset --after hard_reset --no-stub --connect-attempts 2 read_mac \
  > "$capture_dir/reset.stdout" 2> "$capture_dir/reset.stderr" || reset_status=$?
printf '%s\n' "$reset_status" > "$capture_dir/reset.status"
```

If connection fails before that command reaches its reset step, preserve the
failure and investigate device state; command invocation alone is not reset
completion. The command's address output also belongs in private storage.
Always run the reset step after a summary attempt, then require both recorded
statuses to be zero before admitting the factory observation. Do not place an
early-exit shell option around the two commands that skips reset on failure.

The pinned esptool 4.12 CLI performs generic SPI attachment, flash-ID checks,
conditional XMC startup, flash reset commands and possible volatile flash
configuration before `read_mac`. Thus this release is not a narrow MAC-register
read/reset, even with `--no-stub`. It performs no requested main-array write;
the restarted application may change its own stored state.

`efuse-summary` checks selected-field identity, declared widths, readability and
encoding before classifying zero/nonzero contents. Nonzero is not evidence of
uniqueness, provisioning purpose or on-air exposure. An unreadable field remains
unknown. Preserve coding warnings in raw command diagnostics: JSON success
alone does not qualify the integrity of every eFuse block.

## Targeted provisioning metadata extension

The selected metadata extension retains the original three factory fields for
board correspondence and adds these exact canonical names:

```text
BLOCK_USR_DATA CUSTOM_MAC WR_DIS RD_DIS
KEY_PURPOSE_0 KEY_PURPOSE_1 KEY_PURPOSE_2
KEY_PURPOSE_3 KEY_PURPOSE_4 KEY_PURPOSE_5
SECURE_BOOT_EN SPI_BOOT_CRYPT_CNT ENABLE_SECURITY_DOWNLOAD DIS_DOWNLOAD_MODE
```

The installed C6 definition expands `BLOCK_USR_DATA` to the full 256 BLOCK3
data bits. `WR_DIS` bit 22 protects that block; `RD_DIS` bits 0–5 describe the
six key blocks. Aliases such as `USER_DATA` are unsuitable: summary filtering
uses canonical names and may silently omit an alias. Check the complete field
set, widths/readability and original factory-field equality before interpreting
the metadata. Validate names/widths in virtual mode before the device read.

Keep raw user data, custom MAC, selected factory values and diagnostics private.
Publish only blankness/protection booleans and documented key-purpose/security
enums. Excluding key fields from output does not prevent stock initialization
from reading all blocks internally. Purpose metadata neither reads key contents
into the saved output nor establishes that user-purpose storage is empty. Use
the same exclusive-port, reset, status and failure-retention rules above.

## Publication and limits

Keep raw enumerations, actual values, source digests, concrete port paths and
diagnostics private. Publish only explicitly sanitized derived summaries,
tool/source versions, command outcomes, controls and interpretation. Each retained
public result collection has a complete SHA-256 seal excluding the seal itself;
the project ledger records claims and coverage. Replays are read-only and do not
open devices, reset hardware or refresh records.

The baseline does not inspect CP2102N lock/configuration memory, flash unique IDs,
stored credentials, runtime interface addresses, radio traffic, cold-boot USB
enumeration or reset/reconnect stability. Setter success, address rotation,
transmission silence and reduction in correlation remain unmeasured. Define
separate observers, controlled peers/receivers and prospective schedules before
those experiments.
