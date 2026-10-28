# Anonymization integration design

The implementation should make the safe integration path the short path: select
an identity lifetime and supported interfaces, prepare identities before those
interfaces initialize, apply the controls, then start operation explicitly.
An exposure report must say which requirements are verified and what remains.
The first [adapter](../esp-idf/README.md) implements exclusive startup preparation
and two checked SoftAP epochs; its [example](../examples/softap/README.md) records
the working integration and pending physical acceptance. Other interface APIs
remain unimplemented.

## Observer and control boundaries

| Observer or surface | Required treatment |
| --- | --- |
| Nearby radio observer | Handle addresses, discovery/advertising payloads and stable fields for each enabled mode; observe startup and transitions as well as steady operation. Address replacement alone does not establish unlinkability. |
| Network or paired peer | Account for names, protocol identifiers, credentials, BLE identity/security state and peer-held records. Applications must integrate controls for their actual protocols. |
| Attached USB host | Native USB exposes a MAC-based serial. A separate bridge has its own identity and configuration. Document the hardware/connection steps needed for the selected observer requirements. |
| Stored data and privileged access | Account for writable copies and historical storage separately from factory eFuse/flash IDs. Factory identity remains readable under the access conditions documented by the inventory; an application library cannot rewrite those roots. |

Espressif documents [base/interface MAC overrides](https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32c6/api-reference/system/misc_system_api.html#mac-address)
and recommends configuring the base before interface initialization or cached
reads. Its [USB descriptor documentation](https://docs.espressif.com/projects/esp-iot-solution/en/latest/usb/usb_overview/usb_device_const_COM.html)
identifies the C6 serial as MAC-based. The [inventory](../../device-identity/README.md)
owns the board-specific immutable-root, calibration, storage and bridge findings.
Bridge serial activation after a full power cycle remains unverified.

## Rust library responsibilities

Keep identity selection and validation independent of a particular SDK where
practical; add thin adapters only for supported stacks and modes. A single
application must have an explicit owner for interface initialization and identity
changes. Arbitrary application calls that bypass that owner remain outside the
library's guarantees and must be identified during integration.

Identity generation must use a reviewed entropy source available at the actual
initialization point. Specify lifetime, address validity and collision handling;
do not derive a supposedly anonymous identity deterministically from factory
identifiers. A fixed random address can still correlate sessions. BLE privacy
also requires explicit IRK, bonding, peer and stack-specific lifecycle decisions.

Report SDK support, application, readback and external verification separately.
Unsupported hardware, late initialization, failed entropy or failed setters must
not produce a successful required-control report. Starting an interface whose
required controls failed must be inhibited by the supported integration path.
The report must identify pre-application exposure that such a gate cannot stop.

Build configuration and explicit application hooks must address payloads, logs,
credentials and stored copies. Persistent migrations need their own recovery
contract. Removing a logical storage value is not proof that historical bytes
are erased. Raw identities and keys stay out of ordinary diagnostics.

## Implementation sequence and acceptance

1. **One usable SoftAP integration.** Pin the Rust/SDK/board configuration and
   implement the smallest adapter that selects and applies valid local addresses
   before initialization. Demonstrate two independent startup identity epochs
   and one stopped/restarted change with independent packet witnesses. Publish
   actual integration steps and residual exposures.
2. **Lifecycle and application coverage.** Qualify the relevant cold-boot,
   reset, reconnect, sleep/wake and firmware-update paths, including identity
   persistence and key lifetimes. Add focused BLE, station and IEEE 802.15.4
   adapters as each mode is implemented and independently observed.
3. **Complete deployment guidance.** Test USB/bridge handling and supported
   persistent migrations where applicable. Review all enabled application
   protocols and observer requirements; unresolved exposures remain explicit.
4. **Broader compatibility and usability.** Add further Rust stacks and boards
   only with working integrations. Measure setup steps and application changes,
   and simplify them without reducing the qualified coverage.

Reuse the [runtime table evidence](../../device-identity/results/runtime-table-2026-10-24/README.md),
[AP changes](../../device-identity/results/wifi-restart-2026-10-25/README.md),
[BLE HCI witnesses](../../device-identity/results/ble-identity-2026-10-26/README.md)
and [live key-state findings](../../device-identity/results/nimble-live-2026-10-26/README.md)
as source and test-design inputs. Use existing SDK APIs and stock observers.

Presence witnesses, local readback and absence claims have different acceptance
criteria. The existing bounded AP captures prove presence only. A claim that
factory identity never appeared requires observation beginning before the
relevant startup boundary, with qualified receiver coverage and loss limits.
Host markers alone do not establish that coverage. Traffic/RF fingerprinting and
other unmeasured correlation paths remain unknown until a defined experiment
supports a claim. No blanket anonymity result follows from a partial pass.
