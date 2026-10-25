# First control and passive survey probe

[Project](../README.md) · [Protocol](../docs/protocol.md)

Decision question: can our Rust firmware receive commands over the existing
CP2102N connection, produce correlated replies, and return internally
consistent passive Wi-Fi discovery counts? This is a functional development
probe, with no timing score, OS comparison, controlled AP delivery claim,
or radio-range conclusion.

The closest prior repository check is the ESP-IDF UART smoke/benchmark transport;
its frozen collectors and results are not repurposed. Reuse the unmodified
ESP-IDF scan driver and Rust wrappers, with an authored Rust adapter. No
published wireless performance number is compared. The project's
[prior-work notes](README.md#prior-work-and-permitted-reuse) remain the entry
point before any later benchmark.

## Prospective procedure

1. Identify the USB port as CP2102N 10c4:ea60 and the board as
   DevKitC-1-N8; record chip/flash observations separately from the PCB revision.
   Keep native USB unplugged for this first probe. Power comes from the Mac.
2. Read the full 8 MB flash into a new ignored private backup directory and
   verify it against the device before writing. Record tool versions and the
   private backup hash; do not publish flash contents or unique device IDs.
3. Build firmware and client from the pinned lockfiles. Preserve source/config
   digests, exact ELF/merged-image hashes, build failures, and the successful
   generated SDK configuration. Run host positive/negative controls and retain
   the [independent review](../docs/api.md).
4. Flash only the matching merged image. Verify the written range before
   application validation. Do not interpret flash-tool completion as a test.
5. After startup settles, collect `info`, a known `ping` challenge, and a
   passive `wifi_scan`. Repeat the survey once, then collect `info` to check
   continued service. Use fresh output directories and retain every failure.
   Commands are sequential; no AP association or credentials are requested.
6. Validate raw records and saved JSON through `wxctl replay` and the functional
   ledger. Info must report UART and the implemented flags; ping must echo its
   challenge; survey counts must conserve under the 32-record cap. Ambient AP
   counts are observations without a known-AP oracle or expected numeric value.
7. Restore the complete private flash backup and verify all 8 MB against it.
   Record restoration separately from the probe outcomes.

Host controls include fragmentation, caps, versions/fields, matching IDs/nonces,
count/bin inconsistencies, boot/panic/duplicate/incomplete records, altered saved
responses, and preserved failures. They qualify only this acceptance path.
No comparison is authorized by passing the functional replay command.

Native USB, iPhone-only power, iPhone USB data, disconnect/reconnect, BLE,
802.15.4, controlled peer exchange, coexistence and energy remain outside this
probe. Extend the method and review before adding those outcomes. The immutable
historical OS ledger remains unchanged; use the project's own result records.
