# ESP32-C6 anonymization

[Workspace](../../README.md) · [Projects](../README.md)

**Objective: make it as easy as possible to fully anonymize ESP32-C6 devices.**

Build a reusable Rust library, integration examples and deployment guidance that
handle every controllable identity exposure with minimal application changes.
Address the remaining application and hardware requirements explicitly so a
developer can reach the strongest anonymity achievable for their configuration.

Anonymity claims must identify the observers, interfaces and identity lifetime
they cover. The implementation must expose unresolved identifiers and required
actions rather than declaring a device anonymous after changing its MAC.
Full anonymity remains the target; a library alone cannot remove immutable
silicon identity or control arbitrary application behavior.

## Deliverables

- A Rust library that prepares identities before interface initialization,
  applies supported address/privacy controls, and detects unsupported or
  conflicting configuration without silently falling back to factory identity.
- Minimal working integrations for supported stacks, with clear initialization
  order and examples for identity lifetime, reconnect and reset behavior.
- Application integration guidance for payloads, names, credentials, BLE
  keys/bonds, logs and stored identity copies.
- Board and deployment guidance for native USB, separate USB bridges, debug
  access and immutable identifiers that software cannot rewrite.
- A per-interface exposure report and independently observed examples that
  distinguish configured, verified, unsupported and unresolved behavior.

Ease of use is part of acceptance: record required code/build changes, manual
steps and compatibility constraints for each working example. Prefer existing
SDK facilities and small adapters over a new radio or capture framework.

## Evidence and current status

The [device identity investigation](../device-identity/README.md) supplies the
identifier inventory, supported modification methods and selected on-board
results. Those observations establish a starting point; each new library
integration needs its own validation. The original evidence stays with that
project, including failures and [remaining limits](../device-identity/docs/remaining-work.md).

**Current status: the first Rust identity library, ESP-IDF SoftAP adapter and
bounded integration example are implemented.** Host checks and a C6 target build
cover this source; independently observed on-board acceptance is still pending.
The external receiver was unreachable during implementation, so no image was
flashed and there are no anonymization-project hardware results.

| Component | Use and current boundary |
| --- | --- |
| [Identity library](crates/identity/README.md) | Dependency-free `no_std` address/SSID construction from caller random bytes; allocation-free, redacted diagnostics. It does not generate entropy. |
| [ESP-IDF adapter](esp-idf/README.md) | Explicit startup entropy preparation, masked BASE/AP selection, two checked AP epochs and retained failure state. SDK readback is not an external packet witness. |
| [SoftAP example](examples/softap/README.md) | Builds for the documented C6 board; private UART protocol and requested stop deadlines support the next board trial. |

The [API contract](docs/api.md) records startup safety and all 54 guideline
decisions. The [integration design](docs/design.md) defines subsequent coverage;
the [prospective observation method](experiments/softap-identity.md) defines the
unrun physical acceptance. Other radios, native USB/bridge identity, stored
credentials and full-anonymity/unlinkability claims remain outside this first
implemented path. Packages are internal path dependencies, not published releases.

## First implementation milestone

Deliver one ESP32-C6 Rust/ESP-IDF integration that prepares a fresh Wi-Fi SoftAP
identity before initialization, validates readback, and demonstrates the
requested address and configured payload with an independent receiver. Reuse
the existing AP measurement approach and pin the actual toolchain/SDK before
implementation. ESP-IDF 5.5.6 and the documented DevKitC-1-N8 are the initial
build baseline. Independent on-board compatibility remains unverified.

That milestone must also report USB, application and other-radio exposures it
does not control. Presence of the requested address qualifies that observation;
absence of factory traffic needs a separately qualified startup observation.
Later milestones extend coverage to BLE, station and IEEE 802.15.4 integrations,
persistent identity management and additional Rust stacks.
