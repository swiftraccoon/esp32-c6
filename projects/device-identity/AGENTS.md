# Device identity working agreement

Apply the [workspace agreement](../../AGENTS.md). Maintain the identifier
inventory and investigation order in [README.md](README.md), using the
[board map](../../docs/hardware.md) for the two USB-C interfaces.

- Record each identifier's source, exposure, configurable scope, persistence
  and observer. Distinguish documented APIs, inspected configuration and bytes
  actually observed on this board; unknown population or lock state stays
  unknown until inspected.
- Treat C6 factory/runtime identities and the CP2102N bridge configuration as
  distinct ownership domains. A radio MAC override does not establish a change
  to native USB descriptors, bridge identity or stored application credentials.
- Keep unique per-board values, keys, raw enumerations and radio traces under
  ignored workspace `private/device-identity/`. Publish sanitized observations
  with provenance; label redactions and synthetic controls explicitly.
- Define the observer for each experiment: passive receiver, network or paired
  peer, attached USB host, or physical/debug access. Measure cold boot, reset,
  reconnect and recovery when evaluating rotation, including traffic before an
  override takes effect and collateral identifiers in payloads or persistence.
- Prefer read-only inventory and reversible runtime configuration. Preserve
  initial state and restoration evidence. Do not program eFuses, lock bridge
  configuration or erase keys as part of inventory/runtime-rotation work.
- Pin the actual SDK/API inputs for implementations. Validate configured values
  against independent observations; a setter returning success does not prove
  the transmitted identity or absence of transmissions.
- State measured reduction and remaining correlation paths. Address rotation,
  encryption or passive settings alone cannot establish that a device is never
  identifiable. Keep source claims, software controls and physical evidence
  separately qualified in the maintained inventory and experiment records.

The host workspace pins Rust 1.96.0. Run `cargo +1.96.0 test --workspace --locked`
and the formatter, Clippy and documentation checks from this project directory.
The [tool contract](docs/api.md) records API/safety decisions and the
[inventory method](experiments/read-only-inventory.md) defines capture/reset
boundaries. Run `identityctl` summaries from the workspace root with private
input paths; the tool rejects parent traversal and symlinks. Preserve raw inputs
and failed captures privately. The [result ledger](results/README.md) owns
coverage; a summary parser pass does not authenticate a capture's origin.

Runtime probe firmware is an independent workspace under `firmware/`; follow
its README for target/SDK build configuration and the
[runtime method](experiments/runtime-table.md) for the fixed schedule. Preserve
and independently verify a fresh full-flash backup before writes; restore and
verify all 8 MB before ending a board trial. Radio services remain uninitialized
for this table probe. Admit complete sessions with `identityctl runtime-replay`;
`runtime-summary` checks only saved table exchanges. `usb-compare` compares host
property contents, with cached-enumeration and duplicate-device limits.

The independent [AP witness firmware](firmware-wifi/README.md) starts a real
SoftAP only after its nonce-bound UART command. Follow its
[method](experiments/wifi-ap.md) for receiver readiness, single-start scope,
segmented erase-range gates and mandatory fresh full-image restoration. A good
beacon witness does not qualify silence, full-window capture or rotation.

The second `device-identity-wifi-restart` binary uses the separate version-two
namespace and [restart method](experiments/wifi-restart.md). Keep one UART handle
and retained driver for its two ordered phases, require complete A admission
before B, and preserve the sticky failure latch. Fresh full-flash backup and
restoration gates still apply; stopped/restarted address presence does not
qualify hot rotation, silence or correlation reduction.
