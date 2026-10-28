# Read-only USB/eFuse baseline

Collection date: 2026-10-24 in America/New_York; UTC interval
2026-10-25 02:35:49–02:36:28. Target: ESP32-C6-DevKitC-1-N8, with both USB-C
interfaces connected to the Mac. The attached-host observer has USB enumeration
and UART ROM download access. No runtime identity configuration was applied.

| Observation | Result | Interpretation |
| --- | --- | --- |
| Native Serial/JTAG USB, VID/PID `303a:1001` | One matching entry, nonempty serial property before/after the ROM operation | Host-visible serial exposure; value withheld |
| CP2102N bridge USB, VID/PID `10c4:ea60` | One matching entry, nonempty serial property before/after the C6 reset | Bridge identity is separately host-visible; configuration/lock state uninspected |
| Factory base MAC | Readable, nonzero 48-bit contents; SDK MAC integrity annotation accepted | Factory-field availability, with no transmitted-address observation |
| MAC extension | Readable, nonzero 16-bit contents; SDK MAC integrity annotation accepted | Extension-field availability |
| Optional unique ID | Readable, nonzero 128-bit contents | Does not establish uniqueness, provisioning purpose or on-air use |

The before/after summaries establish property presence in two snapshots. The
published counts cannot establish equality of serial values, and this collection
does not claim reset/reconnect or cold-boot identity stability. Product IDs
identify interface classes; they are not proof of one-board association when
multiple matching devices are attached.

## Method and state boundary

The [prospective method](../../experiments/read-only-inventory.md) used macOS
27.2 build 26B5091g on `aarch64-apple-darwin`, Python 3.14.9 and the installed
esptool/espefuse 4.12.0 from the cached ESP-IDF environment. The C6 field-table
baseline is ESP-IDF 5.5.6 commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`.
[Input digests](inputs.sha256) identify the exact method, consumer source, lock
and toolchain before hardware capture. [SDK tool digests](sdk-tools.sha256)
identify the installed capture/reset implementation and field table.

Both profiler captures, the explicitly whitelisted `summary` operation and the
ROM-only `read_mac`/hard-reset command exited zero. The reset command emitted
its hard-reset completion message at 02:35:56 UTC. The follow-up USB snapshot
completed at 02:36:28 UTC. Capture diagnostics contained no warning/error/fail
lines. The accepted SDK MAC annotations support only their documented check;
this is not a separate integrity qualification of every optional-ID bit.

No firmware was flashed, no eFuse programming was performed, and no bridge
configuration or key erase command was invoked. The existing application was
released by hard reset. Application boot behavior and any later persistence
changes were not measured; no full-flash equality/restoration claim is made.

The actual capture/save/derive/replay path passed for each retained raw input:
repeat `identityctl` output matched the complete saved summary bytes. The
virtual control is explicitly synthetic and reported all-zero fields. The
[ledger](../README.md) retains qualification findings and unresolved coverage.
Raw values, port paths, exact binary identity, timestamps, failed attempts and
command logs are omitted from this public collection.

## Retained sanitized derivatives

- [USB before ROM access](usb-before.json).
- [Filtered eFuse field summary](efuse.json).
- [USB after reset](usb-after.json).
- [Virtual eFuse control](virtual-control.json), not a board observation.
- [Complete public seal](SHA256SUMS), paths relative to this directory, excluding
  the seal itself.

Verify the retained derivatives without accessing hardware:

```sh
cd projects/device-identity/results/baseline-2026-10-24
shasum -a 256 --check SHA256SUMS
```

Raw values are intentionally unavailable in the public collection. Public seal
verification does not replay omitted raw captures or authenticate their origin.
No radio, stored-credential, bridge-lock or anonymity result is admitted.
