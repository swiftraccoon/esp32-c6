# Wireless explorer results

[Project](../README.md)

The [2026-10-24 control probe](control-2026-10-24/README.md) retains seven
successful UART exchanges, two failed attempts, build inputs/logs, review
controls, and the restoration record. Two passive Wi-Fi surveys reported 9
and 6 APs. The [functional ledger](evidence.json) records the narrow observed
coverage for that control probe.

The [passive recorder qualification](observer-qualification-2026-10-24/README.md)
retains build inputs, review controls and aggregate observations from Wi-Fi,
BLE and IEEE 802.15.4. Corrected image 03 completed both repeated receive
schedules on UART and Mac native USB, including Wi-Fi reception after IEEE
windows. Native info/ping passed; one startup failure remains preserved.
The [results and limits](observer-qualification-2026-10-24/image-03-results.md)
and [recorder coverage](observer-qualification-2026-10-24/coverage.json) describe
that functional scope. Raw ambient captures and device identifiers remain private.

From the project directory, run
`target/release/wxctl verify-probes --ledger results/evidence.json`.
This replays successful post-request records; the original failed captures
remain rejected by `wxctl replay`. Recorder sessions use `replay-observation`;
they are not validated by the earlier control ledger. Phone access, controlled
peer fidelity, RF silence, recovery, coexistence and energy remain unqualified.

Use one dated directory per investigation/attempt, with an explicit status.
Retain raw data, failures and the conditions needed to interpret them:

- Board and PCB revision, phone/iOS, peers, antenna setup, cable, active USB
  ports and power source.
- Firmware/client source revisions, exact SDK/crate/toolchain inputs,
  configuration, build logs and artifact hashes.
- Radio roles, channels, PHYs, payloads, traffic and active control transport.
- Received raw records, timestamps and their clock/units, overflow/loss counts,
  disconnects and resets.
- Commands, observed outcomes, checks that could not run and interpretation
  limited to those observations.

Keep credentials and full-flash backups in ignored private locations.
Before comparisons or finalized evidence, apply the
[assurance workflow](../../../docs/assurance-workflow.md). The current ledger
supports functional UART/Wi-Fi transcript observations only; comparisons need
their own frozen method, artifact identities and reviewed qualification.
A build success does not establish radio or iPhone operation.
