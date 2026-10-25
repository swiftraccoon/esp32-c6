# Companion clients

[Project](../README.md) · [Protocol](../docs/protocol.md)

`wxctl` is the Rust desktop client. It discovers ports without opening them,
sends control commands, records scheduled passive radio windows and replays
saved captures. There is no iPhone app or iOS USB driver yet.

From `projects/wireless-explorer`:

```sh
cargo +1.96.0 build --release --locked -p wxctl
target/release/wxctl ports
mkdir -p ../../private/wireless-explorer/captures
target/release/wxctl observe --port /dev/cu.SERIAL_PORT \
  --output ../../private/wireless-explorer/captures/session-01
target/release/wxctl replay-observation \
  --capture ../../private/wireless-explorer/captures/session-01
```

`observe` defaults to all three radios, Wi-Fi channels 1–13, IEEE 802.15.4
channels 11–26, a one-second dwell and one cycle. Use `--radio wifi`,
`--wifi-channels 1,6,11`, `--ieee802154-channels 15,20,25`, `--dwell-ms 5000`,
or `--cycles 3` to select a schedule. The serial port stays open between windows.
The [observation contract](../docs/observation.md) describes packet JSONL,
activity summaries, capture gaps and loss counters. Raw payloads/addresses are
kept in ignored private storage; review summaries before publishing them.

Both commands default to **921600 baud** for firmware revision 2.
Use `exchange --baud 115200` with revision 1 firmware. Control probes:

```sh
mkdir -p ../../.cache/wireless-explorer/captures
target/release/wxctl exchange --port /dev/cu.SERIAL_PORT \
  --output ../../.cache/wireless-explorer/captures/info-01 --command info
target/release/wxctl exchange --port /dev/cu.SERIAL_PORT \
  --output ../../.cache/wireless-explorer/captures/ping-01 --command ping --nonce 123
target/release/wxctl exchange --port /dev/cu.SERIAL_PORT \
  --output ../../.cache/wireless-explorer/captures/scan-01 --command wifi-scan
target/release/wxctl replay --capture ../../.cache/wireless-explorer/captures/scan-01
```

These examples require the project's firmware to be flashed and running.
Verify the current port using USB vendor/product identity; this machine's
CP2102N is 10c4:ea60, but the path may change. Native USB has a separate identity
and is a separate test. Output directories must be new; existing and failed
attempts are never overwritten. The client deasserts DTR/RTS and saves a nominal
one-second startup capture for `exchange`, or two seconds for `observe`, before
transmitting. It issues no reset pulse, but
opening the port or changing control lines can reset this board. Startup and
post-request bytes are retained separately.

`verify-probes --ledger <path>` replays the captures named in a project-local
functional ledger, checks their declared commands, and rejects duplicates and
path traversal. It also checks coverage against admitted transcript types and
keeps unsupported domains `not_run`. Paths are relative to the ledger's directory. This checks
recorded protocol observations, not firmware provenance, physical identity,
phone compatibility, radio delivery, or an OS comparison.

The [iPhone/USB investigation](../docs/iphone-usb.md) remains open. Keep USB-C
power plus data as the target while establishing a supported iOS access path.
Discuss any necessary Swift/Objective-C platform glue before implementation
under the repository's Rust preference.
