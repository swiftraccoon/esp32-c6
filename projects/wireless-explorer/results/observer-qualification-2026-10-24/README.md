# Passive recorder qualification

Status: **completed for the recorded functional scope**. The Rust recorder
received ambient Wi-Fi, BLE and IEEE 802.15.4 reports on ESP32-C6-DevKitC-1-N8
through CP2102N UART0 at 921600 baud and Mac native USB. Ambient captures are
not comparative benchmarks or controlled delivery measurements. The
[image 03 results](image-03-results.md) record successful repeated-mode reception,
the native startup failure and verified restoration. [Coverage](coverage.json)
keeps unqualified features explicit. [SHA256SUMS](SHA256SUMS) seals every retained
file with archive-relative paths, excluding only itself.

## Method and inputs

The [observation contract](../../docs/observation.md) defines the receive modes,
window scheduling, timestamps, buffer loss, snapshots, wire records and replay
checks. The [API and safety contract](../../docs/api.md) covers applicability
and execution contexts; [retained controls](review/README.md) preserve contrary
tests and failures.

[source-inputs-01.sha256](source-inputs-01.sha256) freezes repository-relative
source/configuration/method bytes before board capture. The firmware uses Rust
1.96.0, ESP-IDF 5.5.6, esp-rs `a209052f`, embuild 0.33.7 and libclang 21.1.8.
Image 02 has a 3 MiB factory partition within 8 MiB flash; application size is
1,712,496 bytes (54.44%). Exact configuration is retained under
[builds/](builds/).

| Artifact | SHA-256 |
| --- | --- |
| Firmware ELF, image 02 | `9cd528cebbb3db62f48a1a7a1181130f9ce7864ba5de25be297160803838213f` |
| Merged image 02 | `9fc8c3453e8a59ed66034fd09ab2288743e5544301f1f8b95d39641cda519c69` |
| Desktop collector 01 | `f69cfe6aaf681ea029dba05c40472f0e6bab569ee8e31015270167d899e07b17` |

Executables/images remain in ignored `.cache/wireless-explorer/observer-2026-10-24/`.
Nearby device identifiers and payloads remain in ignored
`private/wireless-explorer/captures/`. This public archive retains method,
source identities, build/host checks, review controls and aggregate outcomes.
Public-source replay cannot reproduce private RF captures without those files.

## Retained build and review failures

- Build 01 disabled GATT/SMP symbols, which the pinned service still references;
  the upstream wrapper build failed. Those symbols are enabled but unused by the
  recorder; bond persistence is disabled.
- Initial target Clippy rejected three unnecessary casts. Initial host Clippy
  flagged bounded enum size and two expression simplifications; the enum has a
  documented allocation-free exception and the expressions were corrected.
- Image packaging 01 rejected the 1,712,496-byte application against the SDK's
  1500 KiB partition. The custom 3 MiB layout is the packaged image 02.
- Build 05 could not find the custom CSV in the SDK build directory. Cargo's
  documented resource-glob configuration now copies it there; build 06 passes.
- Independent host controls exposed acceptance of trailing bytes hidden beyond
  metadata read caps. The original failing snapshot/log and corrected controls
  are retained in [review/](review/README.md).

## Initial image 02 board plan

1. Preserve and verify a fresh full 8 MiB backup while the board stays in the
   bootloader; no application reset between read and verification.
2. Write image 02, verify the transferred bytes and boot it.
3. Capture ambient Wi-Fi/BLE/802.15.4 windows, replay every completed session,
   and retain failures. Repeat mode transitions in one open serial session.
4. Treat zero-packet windows as startup/completion observations only. No known
   802.15.4 transmitter, controlled RF peer, independent silence monitor, native
   USB cable or iPhone path is part of the initial image 02 checks.
5. Restore and verify the full original image, then release reset.

The fresh backup was read and digest-verified before image preparation. Its
private SHA-256 is `bc1f2d438df400c6353daf549daf784c546d60c46f80ec8b9293b32445a26f1a`.
No observer firmware had been flashed at the initial input checkpoint.

## Follow-up capture state

Image 02 subsequently completed three private capture sessions. The
[capture review](capture-adversarial-review.md) records their replay results
and the unresolved Wi-Fi silence after returning from IEEE 802.15.4. That
transition is not qualified as successful radio reuse.

[Source inputs 02](source-inputs-02.sha256) describe the correction that stops
continuous IEEE receive and puts that radio to sleep before disabling it.
Target build and Clippy passed. The later [image 03 checks](image-03-results.md)
completed the frozen UART and native USB schedules with reception after IEEE
receive teardown. A native startup failure is retained separately from its
successful retry. The original full 8 MiB image was restored and digest-verified
again before hard reset. Ambient raw records, original build logs and transfer
diagnostics remain private. Public build logs use sanitized home paths and do
not claim original byte identity.
