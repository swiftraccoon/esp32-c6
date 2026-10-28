# Bridge configuration programming and serial activation

The CP2102N accepts the complete custom-serial configuration: a fresh GetConfig
response matches all **678 candidate bytes**, including the custom-string
selector, UTF-16LE payload and Fletcher16 checksum. However, the fresh USB serial
response still contains the original serial after the selected Mac SDK reset.
The complete serial-change trial therefore remains **failed**. Configuration
write/readback and active descriptor replacement are distinct observations.

All **678 original bytes** and the original fresh bridge/native serials were
restored and independently replayed. The update-enable byte remains `FF`.
Each physical attempt finished its matched, warning-free normal C6 reset
sequence. No bridge configuration change or restoration remains pending.

| Attempt | Observations | Outcome |
| --- | --- | --- |
| First | Existing observer rejected an absolute output path before USB capture; no SDK invocation or bridge write | Failed; normal C6 reset verified |
| Second | One candidate write/reset reported success, but the candidate host-property predicate stopped before fresh candidate capture; complete original restore/readback succeeded | Failed; candidate contents/exposure unobserved |
| Third | All three complete captures replay; candidate GetConfig exactly matches the requested array, while its active serial response remains original; complete original restore/readback succeeded | Failed exposed-serial criterion; configuration readback observed |

The second and third attempts each issued one candidate SetConfig and exactly
one original SetConfig, with explicit SDK resets. Five complete USB captures
contain 50 successful transfer rows in total; the third attempt contributes
three captures and 30 rows. All pass whole-directory replay and the separate
existing bridge Fletcher validator. Independent review reproduces all three
third-attempt arrays, both serial domains, original restoration, final reset
and all 486 frozen inputs. No failed cohort was replaced by a passing prefix.

Only selector/string/checksum bytes were permitted to differ. Unrelated
settings and unused padding match the original. This checks programming of the
documented configuration image; it does not establish power-cycle persistence,
change the factory-assigned internal ID or demonstrate active custom-serial
exposure.

The [method](../../experiments/bridge-serial.md) uses unchanged vendor-authored
SMT sources built for the Mac and the existing qualified USB observer. Prospective
versions and exact compositions remain retained. The first failure exposed a
real CLI argument boundary missing from the original mocks; actual capture-path
controls and revised strict mocks qualify the corrected relative arguments.
The second failure prompted fresh observation under only known original/candidate
cached properties. Fresh expected bytes still decide admission.

The SDK ignores `libusb_reset_device`'s result. The selected Darwin path requests
host re-enumeration and checks device/configuration descriptors rather than
serial-string contents; it does not establish a bridge MCU reset or power loss.
Examined existing programmers offer the same reset class. AN571's interface
RESET request is explicitly ignored and is not an activation alternative.
No distinct supported firmware-reset command was identified in those sources.

A full bridge power-cycle activation check remains unperformed. In published
DevKitC-1 revisions 1.2–1.4, both USB inputs feed the supply powering the bridge.
Disconnecting only UART USB leaves it powered through native USB. A full cycle
requires both cables and any other supply/backpower removed. The bridge RSTb
has a pull-up but no documented connection to C6 GPIO, the header or C6 RESET.
The actual PCB revision, supply collapse and activation after such a cycle were
not measured. This is a next validation step, without a claim that it must
resolve the observed activation limit.

Raw configurations, strings, host paths, registry data and complete diagnostics
remain private. The [summary](summary.json) and [provenance](provenance.json) are
sanitized derivatives. `SHA256SUMS` covers every retained public regular file
except itself relative to this directory. It verifies retained bytes rather
than hardware authenticity. No eFuse burn, C6 flash-array programming, key
deletion or configuration lock was requested. Later application health, PHY
persistence and RF behavior remain unmeasured.

Primary reset/power sources:

- [AN978 configuration/reset API](https://www.silabs.com/documents/public/application-notes/an978-cp210x-usb-to-uart-api-specification.pdf).
- [Selected libusb Darwin implementation](https://github.com/libusb/libusb/blob/v1.0.30/libusb/os/darwin_usb.c#L2140).
- [Existing programmer reset/replug behavior](https://github.com/irrwisch1/cp210x-cfg/blob/b417a28b3367a65990ad501c6cb98da2c32b7ea4/src/main.c#L478).
- [AN571 ignored interface reset](https://www.silabs.com/documents/public/application-notes/AN571.pdf).
- [Official board schematic, sheet 2](https://dl.espressif.com/dl/schematics/esp32-c6-devkitc-1-schematics_v1.4.pdf#page=2).
