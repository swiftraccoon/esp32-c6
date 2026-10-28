# BLE identity through receiver HCI reports

One complete six-command session changed the advertising address from static
random control A to distinct control B across a checked Stop, retaining the
initialized controller and one UART handle. An independent receiver reported
**two exact A reports and one exact B report**, each with the same fresh session
nonce in the complete 26-byte advertising data. Both active SDK readbacks matched
their controls; all six responses had zero operation status, no first failure
and no automatic Stop.

The receiver reports were complete legacy LE 1M nonconnectable, nonscannable
advertisements with random address type. A's complete capture was finalized and
admitted before B. The first prestart query still reported a public address;
the second prestart query already matched B. These query results describe SDK
state, while the separate receiver supplies the address-and-payload witness.

A preceding passive baseline also contained eight reports at the original
application's expected public Bluetooth address: four advertisements with its
expected services and four scan responses with its expected name. These are
application fingerprints and a factory-derived address, not additional factory
identifiers.

The existing image was reused unchanged under the
[frozen method](../../experiments/ble-hci-presence.md). Stock TShark decoded both
whole btsnoop files; the qualified single-report checker and independent saved
replay reproduced the witnesses. Both owned monitors were reaped. Ten config
pins and eight receiver guard groups remained unchanged in each window; no
discovery command or Kismet reconfiguration was issued for this trial.

The fresh original **8,388,608-byte** backup was restored, then separately verified
against the entire device range with raw header parameters kept. Verification
and warning-free host reset completed by **2026-10-26 02:11:46 UTC**.
[Restoration details](restoration.json) qualify that boundary, without an
independent application boot or later persistence result.

The earlier controlled attempt expired its receiver window before either Start.
It remains failed: one normal Status, one cleanup Stop and verified full-flash
recovery. An earlier preparation failed a MAC-label assertion before any backup
or write. The failed BlueZ discovery and native-console attempts also remain
preserved. Independent qualification found that checker revision two accepted
a valid prefix followed by a zero-report event; revision three rejects every
unsupported report count and passed its controls before the accepted trial.

[Summary](summary.json) contains sanitized counts and states;
[provenance](provenance.json) identifies the inputs, qualification and retained
failures. Original captures, values, failed attempts and backup hashes remain
private. SHA256SUMS seals every public file except itself; a seal establishes
integrity, not capture authenticity.

This establishes stopped/restarted BLE address-and-nonce presence through the
receiver's HCI interface. It does not qualify raw RF CRC, silence, complete
startup/transition coverage, hot changes while advertising, cold power, RPA,
bonding, population uniqueness or correlation reduction. The shared nonce
deliberately links A and B. The [inventory](../../README.md) keeps these runtime
controls separate from immutable factory identities and other USB/storage IDs.
