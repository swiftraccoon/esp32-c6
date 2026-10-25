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
