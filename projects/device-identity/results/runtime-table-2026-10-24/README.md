# Runtime MAC-table and USB-property probe

Two transport-revision-2 sessions completed on ESP32-C6-DevKitC-1-N8 with both
USB-C ports connected to the Mac. Each observed four SDK table phases and
unchanged native/bridge serial properties in the Mac's snapshots. The initial
revision-1 startup failure remains retained. The original full flash image was
restored, separately verified and released with a hard reset.

The observer is UART application/API access plus an attached Mac USB host.
No application radio service was initialized. These observations establish SDK
configuration-table behavior and cached/current host properties, without an
active-driver, on-air or anonymity result.

## Frozen inputs and outcomes

The [prospective method](../../experiments/runtime-table.md),
[source inputs](inputs.sha256) and [artifact identities](artifacts.json) were
frozen at `c5ff2fcab9f1fa1f41270815e688fa90ba8eb010` before the revision-2 cohort.
All 35 inputs match that commit and remained unchanged through collection.
The [selected actual configuration](effective-sdkconfig.txt) enables four-address
allocation while Bluetooth/IEEE drivers and persistence remain disabled. The
hidden Wi-Fi capability symbol remains enabled; the application starts no
Wi-Fi service. [SDK input digests](sdk-inputs.sha256) identify the inspected
MAC implementation, API, allocation Kconfig and C6 capabilities.

| Session | UTC interval, 2026-10-25 | Startup | Table phases | Native USB / bridge serial properties |
| --- | --- | --- | --- | --- |
| [01](session-01.json) | 03:13:26–03:13:29 | 230 bytes, nine lines, one recognized ROM boot | Four accepted; saved state restored | Unchanged / unchanged; one match per domain |
| [02](session-02.json) | 03:13:54–03:13:58 | 230 bytes, nine lines, one recognized ROM boot | Four accepted; saved state restored | Unchanged / unchanged; one match per domain |

Collection date is 2026-10-24 in America/New_York. Times identify run boundaries,
not calibrated performance measurements. Both runs used the same 452,944-byte
merged image and were separated by explicit reset. Opening/releasing the bridge
can start/reset the C6; no reset-free-open or cold-boot result is claimed.

Both runs observed:

1. A complete factory/base/interface snapshot before mutation; getter calls
   populated the SDK's address caches.
2. The requested synthetic base value after the base-only setter, with all four
   cached interface entries and the direct factory bytes unchanged.
3. The four explicit synthetic station/AP/Bluetooth/IEEE table controls reading
   back correctly, with the requested base and unchanged factory bytes.
4. Full saved snapshot equality after restoring base and interface entries.

The before/during USB comparisons use exact serial-property contents. One
nonempty match existed for each interface class in each snapshot. These
properties may be cached from enumeration: equality does not establish fresh
descriptor transfers or prove that a host could not correlate the board.

## Retained failed cohort and qualification

Cohort 1, frozen at `368d57f`, stopped after its first attempt timed out with
zero received UART bytes and no complete table response. Its failure receipt,
unsuccessful cleanup and raw inputs remain private. The
[original inputs](cohort-01-inputs.sha256) and
[original artifacts](cohort-01-artifacts.json) retain that method's identity;
those source hashes resolve against the original freeze, not revised source.
A line-release diagnostic received the firmware's expected rejection of a
foreign request. Revision 2 checks explicit DTR/RTS release and separately
records/classifies startup before the first table request. The first failure
remains failed and supplies no runtime-table observation.

The [failure derivative](cohort-01-failure.json) preserves zero RX bytes, 98
transmitted bytes, no completed exchange and failed cleanup. The diagnostic
changed both control-line requests; it does not isolate their physical levels
or prove an RTS-only cause. The firmware image was unchanged between cohorts.

Before the revised cohort, 49 host tests and one doctest passed, including
fragmented live/save/replay, late errors, cleanup after failure, strict startup
grammar, IDs/phases/status, table transitions and privacy boundaries. The actual
230-byte diagnostic passed the same startup consumer. Firmware build, formatter,
Clippy and documentation checks passed separately; compilation alone supplied
no physical result.

Each real capture passed complete directory replay with byte-identical saved
summary output. Independent review also derived phase/control/restoration and
USB equality directly from raw JSON without importing the main consumer. Six
corrupted copies, including false completion, panic startup, changed restoration,
raw ID disagreement, false line-release success and a late SDK failure, were
rejected with empty stdout. All 30 original revision-2 capture files remained
unchanged. The original failed cohort also remains rejected.

## Restoration and reproduction

The fresh pre-probe backup was 8,388,608 bytes, separately verified after reading
and again before the probe write. The probe image was written and separately
verified before its first reset. After collection, all 8,388,608 original bytes
were written and independently digest-verified in the bootloader at
**2026-10-25 03:15:53 UTC**, then hard-reset completion was observed at
**03:15:54 UTC**. All three restoration command statuses were zero. The
[device-state record](device-state.json) scopes that boundary.

Restore writes explicitly preserved flash mode/frequency/size with `keep`.
Separate verification used the installed esptool 4.12.0 `keep` defaults with
`ESPTOOL_FF`, `ESPTOOL_FM` and `ESPTOOL_FS` absent. Exact argv, environment check,
original image SHA-256 and transfer diagnostics remain private. These establish
bytes before reset; application health and later persistence were not measured.
No eFuse programming or bridge-configuration command was invoked. The probe
write replaced its covered flash range; the complete original image, including
its persistence regions, was restored and verified afterward.

Unique values, concrete ports, raw records and binaries are omitted from the
public collection. A public checkout can run synthetic controls and verify the
[complete public seal](SHA256SUMS), whose paths are relative to this directory
and cover every retained regular file except the seal itself:

```sh
cd projects/device-identity/results/runtime-table-2026-10-24
shasum -a 256 --check SHA256SUMS
```

Seal verification does not reconstruct omitted raw inputs or authenticate their
origin. Source-input reproduction uses the recorded freeze commit; the original
cohort's manifest intentionally refers to its earlier source and artifact bytes.

## Remaining coverage

The experiment checks late-base caching, explicit interface-table overrides and
RAM restoration. Early-base derivation before cache population is untested.
USB comparisons concern cached/current attached-host properties.
Transmitted radio addresses, fresh USB descriptor transfers, cold boot,
reconnect, active-radio rotation and correlation reduction remain unqualified.
