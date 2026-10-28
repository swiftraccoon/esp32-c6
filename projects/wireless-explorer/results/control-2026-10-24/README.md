# Control and passive Wi-Fi bring-up, 2026-10-24

[Project](../../README.md) · [Procedure](../../experiments/control-bringup.md) ·
[Provenance](provenance.json) · [API and safety contract](../../docs/api.md)

The corrected Rust firmware completed UART info/ping exchanges and two passive
Wi-Fi surveys on **ESP32-C6-DevKitC-1-N8**. The surveys reported
**9** and **6** APs, with consistent channel totals and zero unreported records.
These are ambient discovery observations, not a controlled RF accuracy,
throughput, range, coexistence, power or OS comparison result.

The original full **8 MB flash image was restored and verified** before
releasing reset. No eFuses were changed. The project firmware is available to
build and flash; it is not left installed on the board.

## Actual attempts

| Capture | Image/client state | Outcome |
| --- | --- | --- |
| [info-01](info-01/failure.txt) | First image; client did not explicitly release RTS | Failed: 15-second response deadline, empty command capture. Retained unchanged. |
| [info-02](info-02/response.json) | First image; DTR/RTS release and startup capture added | UART info returned; Wi-Fi command implemented, BLE/802.15.4 false; counters zero. |
| [ping-01](ping-01/response.json) | Same first image, corrected serial opening | Challenge 123 echoed. |
| [scan-01](scan-01/failure.txt) | First image; published svc 0.53.0 | Failed: SDK Wi-Fi error 257 (`ESP_ERR_NO_MEM`). Raw error response retained. |
| [info-03](info-03/response.json) | Pinned upstream Wi-Fi initialization fix | UART info returned. |
| [ping-02](ping-02/response.json) | Corrected image | Challenge 123 echoed. |
| [scan-02](scan-02/response.json) | Corrected image | 9 APs: channel 1 = 1, channel 6 = 1, channel 8 = 7. |
| [scan-03](scan-03/response.json) | Corrected image, another serial opening | 6 APs: channel 1 = 1, channel 3 = 1, channel 6 = 1, channel 8 = 3. |
| [info-04](info-04/response.json) | Corrected image | UART info returned; counters zero. |

Every corrected-client exchange retained 230 bytes of ROM startup output in
`startup.raw`. Opening this CP2102N connection and releasing its control lines
produced a fresh boot. Therefore these records establish individual exchanges
across boots, not persistent-session operation, two scans within one boot, or
reset-free reconnects. The initial timeout was resolved by releasing both
DTR and RTS and capturing a nominal second of startup before sending a command.
Post-request boot/panic text still fails command validation.

## Build and SDK findings

The final [firmware lockfile](../../firmware/Cargo.lock) pins the unmodified
esp-rs Git revision
[`a209052f`](https://github.com/esp-rs/esp-idf/tree/a209052f378be61dd0f2d566b7fbe0f9593b0cd5).
Its Wi-Fi adapter sets `wifi_task_stack_size` to the SDK default **6656 bytes**.
The SDK 5.5.6 default initializer includes that field; the published svc 0.53.0
crate left it at zero. The corrected build emits the applicable patch-version
configuration, and the subsequent surveys succeeded. No vendor-source patch or
new C/C++ implementation was authored.

Retained build sequence:

- [Attempt 1](builds/firmware-attempt-1.log): sys 0.37.2, hal 0.46.2, svc 0.52.1,
  libclang 23.1.2; failed on opaque SPI/RMT bindings and an SD struct field.
- [Attempt 2](builds/firmware-attempt-2.log): published sys 0.38.1, hal 0.47.0,
  svc 0.53.0, libclang 21.1.8; dependencies built, then the application failed
  on the removed HAL prelude import. Changed to `units::Hertz`.
- [Attempt 3](builds/firmware-attempt-3.log) and
  [first physical image build](builds/firmware-final.log): target build passed;
  the latter includes derived Hash traits. The first physical image failed
  Wi-Fi initialization as recorded above.
- [Upstream-fix build](builds/firmware-upstream-fix.log): Rust 1.96.0, IDF 5.5.6,
  pinned Git wrappers, embuild 0.33.7 and libclang 21.1.8 passed.
  [Target Clippy](builds/firmware-upstream-clippy.log) and
  [documentation](builds/firmware-upstream-doc.log) also passed.

[Generated configuration](builds/sdkconfig.esp32c6) records 8 MB flash,
DIO/40 MHz, 160 MHz CPU, 100 Hz tick, 32 KiB main stack, no SDK consoles,
Bluetooth/PM disabled, and Wi-Fi/PHY NVS persistence disabled. The app occupies
947,232 of the 1,048,576-byte factory partition. Larger radio stacks will need
a deliberate partition change; 8 MB flash does not add RAM or PSRAM.

Exact final ELF, merged image, collector and SDK configuration hashes are in
[provenance](provenance.json). The source-input digest sets are chronological:
[initial](source-inputs.sha256), [serial fix](source-inputs-after-host-fix.sha256),
and [final Git fix](source-inputs-git-fix.sha256). Earlier sets describe earlier
bytes and must not be rewritten to match the final source. This was development
bring-up, without a pre-collection Git freeze or comparative qualification.
The initial timeout client's executable was not retained; its request, raw
empty capture, failure record and input digests remain. No exact reproduction
of that original executable's serial-opening behavior is claimed.

## Backup and restoration

Private full images and transfer logs remain under
`backups/wireless-explorer/pre-control-2026-10-24`. The first read was followed
by a reboot of the original firmware; subsequent verification failed. A second
read held the board in its bootloader and verified successfully. The snapshots
differed by 1,992 bytes at offsets 0xc020–0xcfdd, within the original NVS
partition (0x9000–0xefff), consistent with startup persistence. Both snapshots
and the failed verification were preserved.

The verified quiescent snapshot was restored in full with esptool.py 4.12.0,
then `verify_flash` checked all 8,388,608 bytes with flash parameters kept
unchanged. Its digest matched before the final hard reset. The original
application may subsequently write its own NVS; restoration does not assert
continued byte identity after that boot. Full backup contents and unique device
IDs are not published. The public provenance is an operator record, not an
independent authentication of hardware or flash tools.

## Verification and remaining coverage

Nine project unit tests, one doctest, host Clippy/docs/build and firmware target
build/Clippy/docs passed. The independent review retains positive and contrary
CLI controls, failed review controls and their corrections under [review/](review/).
The host CI definition includes project checks; no hosted CI execution is claimed.

From `projects/wireless-explorer`:

```sh
target/release/wxctl verify-probes --ledger results/evidence.json
target/release/wxctl replay --capture results/control-2026-10-24/scan-02
```

The ledger replays seven successful captures. Replaying `info-01` or `scan-01`
must fail because their original failure markers remain; CI also checks those
two rejections. Capture replay validates post-request records and saved results,
not startup diagnostics, collector identity, power, or peer correctness.
The archive's `SHA256SUMS` uses paths relative to this directory and covers all
retained files except itself. The final source digest check runs from the
repository root. Executable artifacts remain local and are not part of this
source/text archive.

Public logs and connection metadata are sanitized derivatives: personal home
paths use `/Users/builduser` and the serial endpoint uses `/dev/cu.SERIAL_PORT`.
Archive checksums describe these public bytes. Original logs and metadata stay
in ignored private storage. Protocol samples and reported outcomes are unchanged.

Native USB, iPhone power/data, controlled peer traffic, BLE, IEEE 802.15.4,
coexistence, persistent-session recovery, loss accounting and energy are
**not run**. No iPhone model/iOS or board PCB revision has been supplied.
