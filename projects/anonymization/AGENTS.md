# Anonymization working agreement

Apply the [workspace agreement](../../AGENTS.md). Keep the objective, supported
integration paths and current status in [README.md](README.md); use the
[integration design](docs/design.md) for implementation and validation decisions.

- Make the strongest achievable ESP32-C6 anonymity easy to integrate. Track
  software controls, application obligations and hardware/deployment steps
  together so that unresolved exposure has a concrete explanation and next step.
- State the observer, interface, identity lifetime and operating conditions for
  every claim. Distinguish documented support, implemented behavior and external
  observations. Never infer unlinkability or silence from a changed getter or
  the absence of packets in an incomplete capture.
- Reuse the [identity inventory](../device-identity/README.md), vendor APIs and
  existing measurement tools. Keep original evidence with its owning project;
  qualify new library integrations separately.
- Prefer a small Rust library with explicit initialization and stack-specific
  adapters. Do not initialize radios, change persistent identities or perform
  credential migrations as hidden side effects. Pin the actual Rust/SDK inputs
  and review the API checklist before adding implementation packages.
- Keep automatic changes reversible. Persistent migrations need explicit
  operations, documented state loss and recovery; no startup hook silently
  programs OTP, locks configuration or erases credentials.
- Before device writes, identify the intended board and independently verify a
  fresh full-flash backup. Keep restoration and external observations separate
  from successful builds, setters and local readback.
- Keep identifiers, generated identities, keys, packets and raw reports under
  ignored `private/anonymization/` and backups under `backups/anonymization/`.
  Public diagnostics and examples use synthetic values or sanitized outcomes.

The project root workspace contains the pure identity crate. Run its formatter,
`cargo +1.96.0 test --workspace --locked`, Clippy and rustdoc checks there.
The adapter and example own independent workspaces/locks: follow
[adapter commands](esp-idf/README.md) and [example commands](examples/softap/README.md)
for selected-host controls and target builds. Target builds require external
`RUSTC_BOOTSTRAP=1` before Cargo resolves scoped build-std. Record actual SDK
configuration and generated bindings; host controls never imply an on-board pass.
The [API record](docs/api.md) owns guideline applicability and lifecycle contracts.
Use workspace privacy/public-snapshot checks before commits.
