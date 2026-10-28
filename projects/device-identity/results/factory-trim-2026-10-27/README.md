# Factory calibration export

The stock-tool read retained all **24 documented factory trim/calibration
fields**, spanning 153 defined bits in BLOCK1/2. The exact 27-row JSON also
contains the factory MAC and both block-version fields. Its raw widths, layout,
decoded values and readable metadata pass independent saved replay; the MAC
matches the original baseline after the tool's byte-order conversion. The
block version supports the selected fields. No missing row was filled with zero.

These values close the inventory's retained-readout gap for the documented
factory calibration fields. They are possible stable correlation material,
without a documented collision-free identity, measured population uniqueness
or 153 proven bits of entropy. Already-populated RS-coded BLOCK1/2 have no
supported rewrite path even when field write-protection flags are unset.

The [prospective method](../../experiments/factory-trim-read.md) was committed
before collection. Qualification used three actual stock virtual-summary cases
and 31 final offline guard/recovery controls. The actual stock version-zero
case exits successfully with only three rows, omitting all trim rows; exact-set
admission rejects it. Independent contrary controls and the earlier corrected
qualification failures remain private. Tool success alone does not admit a read.
The [pinned stock summary implementation](https://github.com/espressif/esptool/blob/v4.12.0/espefuse/efuse/base_operations.py)
and [C6 field table](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/efuse/esp32c6/esp_efuse_table.csv)
provide the selection and interpretation source evidence.

The complete export, source/layout checks, all 77 frozen inputs and the final
reset receipts independently replay. The final ROM factory-MAC check matches
the saved unit, and a warning-free normal C6 hard-reset sequence completes.
No eFuse burn, C6 flash-array programming, bridge configuration write, lock or
key deletion command was requested. No recovery remains pending.

The stock constructor internally reads blocks beyond the selected formatted
rows, and common SPI/XMC startup/reset hooks are broader than a pure eFuse
read. This observation does not establish calibration accuracy, application
health, later storage persistence, RF exposure or a new independent identifier.
No new bridge programming trial occurred; its active custom serial after a
full power cycle remains unverified.

Raw values, their combined vector, identifying hashes and host records remain
in ignored private storage. The [summary](summary.json) and
[provenance](provenance.json) publish coverage and access checks only.
`SHA256SUMS` covers every retained public regular file except itself, relative
to this directory. It establishes retained-byte integrity, not hardware
authenticity or a public replay of the private raw observation.
