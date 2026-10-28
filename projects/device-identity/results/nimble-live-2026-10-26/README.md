# Original NimBLE live identity state

Two reset-separated snapshots of the original application contain nonzero
16-byte default/current IRK buffers. They are equal within each boot and differ
between boots. All five key-provider/store callback pointers are null in both
snapshots. The Secure Connections generation flag is zero in each, so the
conditional public-key read was not executed. No pairing was induced and no
private scalar or negotiated link key was requested.

Each capture exports 311 selected bytes. The complete 256-byte application
descriptor matches the original BIN, and the host sync byte is GOOD before and
after the selected reads. Both captures pass the exact file inventory, saved-byte
gates, ordered debugger markers and diagnostic admission. Each confirms resume
to a running target before debugger shutdown.

The [method](../../experiments/nimble-live-state.md) was committed at
`0b5d60a0a33e24bf48ea32f1f410d3011eb79a27` before collection. A private
53-file freeze binds the actual tools, source trace, compositions, original
ELF/BIN and saved expectations. Qualification used stock OpenOCD's actual Jim
interpreter without adapter initialization, followed by host failure/recovery
controls and independent contrary controls. The initial incorrect shutdown exit
assumption and rejected host-admission controls remain preserved.
Independent saved replay verifies all 55 sealed capture files, all 53 frozen
inputs, both selected-byte inventories and three matched reset receipts, with
inputs unchanged. The selected freeze is not a hermetic host-environment capture.

This is active debug inspection: initialization and halt/resume affect execution,
debug registers, watchdogs and interrupts. Selected memory reads can use the
stock program-buffer path. No firmware replacement, target function call,
eFuse burn, bridge configuration or flash-array programming command was issued.
The esptool reset path still runs its common SPI/XMC startup/reset hooks.

The independent final ROM-identity check and warning-free normal hard reset
completed at **2026-10-26 19:21:21 UTC**. The source-supported reset sequence
is intended to clear transient debug state and releases the chip for another
boot whose keys were not captured. It does not independently
attest application health, later RF behavior or unchanged PHY persistence. The
earlier verified full-flash restoration remains a historical boundary.

The observations establish populated and reset-varying IRK buffers on this
build. They do not measure RNG entropy, prove population uniqueness, show IRK
distribution/RPA use or recover pairing history. A clear SC flag does not
exclude an earlier failed generation attempt. Null callbacks establish the
captured configuration rather than absence of peer-side or active connection
state.

Raw keys, USB identities, descriptors, dumps and complete tool output remain
private. The [summary](summary.json) and [provenance](provenance.json) contain
sanitized observations. `SHA256SUMS` covers every retained regular file except
itself relative to this directory; it verifies retained public bytes, not
hardware authenticity.
