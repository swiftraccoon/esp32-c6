# Targeted flash reads through existing esptool APIs

Revision 2 uses the address and dummy phases to clock past UID bytes. The first
852-read attempt used MOSI padding; it completed but failed its UID overlap
check. Padded results were zero while unpadded results were nonzero. That attempt
remains rejected, with its original method freeze and successful reset release.

Decision question: what UID bytes, security-register contents/locks and unused
user/key eFuse storage are present on this board? Reuse installed upstream
esptool/espefuse 4.12.0 directly. No firmware image, new transport or general
collector is required. Unique values and all raw diagnostics remain private.

## Method and effects

Resolve the single intended bridge against the saved USB baseline and require
exclusive serial ownership. Enter ROM through the existing loader's reset and
connection sequence, require the factory MAC to match baseline, then classify
readable eFuse blocks 3 through 10 as zero/nonzero in memory. Save only their
readability/blankness booleans; do not format or serialize key-block contents.
Stock espefuse initialization reads all blocks internally. An unreadable block
is unknown, even if its returned data are zero.

Use the existing `flash_spi_attach(0)` API, whose ROM payload sets legacy mode
false. This performs volatile pin/clock/controller/cache setup and flash wakeup.
Do not invoke generic CLI flash setup, a stub, program/erase, write-enable,
status-write, flash reset or high-performance configuration. The permitted
flash commands are the following fixed reads:

| Read | Opcode | Address | Dummy clocks | MOSI padding | Returned bits |
| --- | --- | --- | --- | --- | --- |
| JEDEC | `9F` | None | 0 | None | 24 |
| Status 1/2 | `05` / `35` | None | 0 | None | 8 each |
| SFDP | `5A` | 0..252, step 4, 24 bits | 8 | None | 32 |
| UID window | `4B` | Zero, lengths 0/0/32/64 bits | 32/64/64/64 respectively | None | 32 |
| UID overlaps | `4B` | Zero, lengths 32/64/40/48/56 bits | 32/32/64/64/64 respectively | None | 32 |
| Security registers | `48` | `1000`, `2000`, `3000` plus 0..1020, step 4, 24 bits | 8 | None | 32 |

Require the expected XMC family before optional reads and an idle flash status.
Capture status before and after, requiring equality. In the UID reads, the
single chip-select transaction is command, address clocks, dummy clocks then
MISO. The flash consumes its mandatory 32 dummy clocks and shifts UID data
during the remaining ignored clocks; SI values after the command are irrelevant.
No MOSI data phase is enabled. Interpret each word as four little-endian stream
bytes. Primary reads return offsets 0/4/8/12; overlaps return 4/8/9/10/11.
Require all five overlap words to match slices of the primary 16-byte window.
That window covers both documented UID widths. C specifies FF after eight UID
bytes; D specifies FF after sixteen. Padding alone does not conclusively identify
the suffix. Both address and dummy length fields have six bits; request at most
64 clocks/bits in either. Upstream does not validate those ranges. The 64-bit
address clock count is supported by the register/LL contract, not an electrical
timing measurement. The accepted collection checks the selected offsets through
the declared overlapping response bytes.

Retain the whole 256-byte SFDP response and complete ordered read schedule.
The security-register window covers all three 256-byte C regions and all three
1024-byte D regions; addresses outside C's defined regions return FF. Interpret
SR2 bits 3/4/5 as the respective permanent security-region locks. Main-array
backups exclude these regions and the UID. No write-to-test-lock is permitted.

Use the fixed 846-read schedule, a 240-second capture deadline and an outer
process deadline. Preserve partial reads and errors; missing/failed/extra reads
cannot become a passing result. The upstream helper restores only selected
controller registers. Always attempt the existing loader hard-reset release,
including after failure, and retain reset/close outcomes. Reset clears volatile
controller state and restarts the original application, which may update its
own storage. A host reset receipt does not attest physical boot or unchanged
flash contents. No flash-array write or new image belongs to these reads.

## Qualification and sources

Before board access, exercise the unchanged upstream SPI function with synthetic
register-backed inputs, including address/dummy overlap, address encoding, rejected
read widths and contrary schedules. Preserve source/API identities and the exact
plan; obtain independent review. This qualifies interpretation of the selected
API schedule, not the physical SPI timing or factory uniqueness. Replay and
independently inspect the complete saved response schedule after collection.
The [retained result](../results/flash-identity-2026-10-26/README.md) preserves the
failed first schedule and accepted revision-two observation.

- [Esptool 4.12 SPI helper](https://github.com/espressif/esptool/blob/v4.12.0/esptool/loader.py#L1485).
- [Pinned C6 SPI register layout](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/register/soc/spi_mem_struct.h).
- [Pinned HAL command transaction](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/spi_flash_hal_common.inc#L181).
- [XMC C datasheet](https://www.xmcwh.com/uploads/800/XM25QH64C_Ver1.7.pdf), UID/security sections.
- [XMC D datasheet](https://www.xmcwh.com/uploads/916/XM25QH64D_Ver1.2.pdf), UID/security sections.
