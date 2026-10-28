# Read-only factory calibration export

Decision question: can the existing stock tool retain the documented factory
trim/calibration vector on this board, with its raw representation and layout
version? The inventory currently names these fields without retaining their
values. They are possible stable correlation material, not another documented
globally unique serial or a source of 153 proven entropy bits.

Reuse esptool/espefuse 4.12, the existing ROM serial/reset path and the pinned C6
field definitions. No new firmware or maintained extraction tool is required.
This method reads 24 trim/calibration rows plus the factory MAC and both block
version rows, for an exact **27-row** JSON export. Raw values remain private.

## Fixed selection and interpretation

Select these groups by canonical stock field names:

| Group | Rows | Defined bits |
| --- | ---: | ---: |
| BLOCK1 active/light/deep-sleep bias/debug and bias gap | 7 | 28 |
| BLOCK2 temperature, OCODE, ADC initialization/voltage/channel coefficients | 17 | 125 |
| MAC and block major/minor versions | 3 | 53 |

Pin all 27 names and widths in the private input plan, matching the SDK table
and actual stock definitions. Preserve JSON `raw_value`, decoded value,
`bit_len`, block and access metadata. Verify raw integer bounds against each
width, the exact factory MAC against the saved baseline, and metadata
consistency. Record readable write-protected fields as such; `writeable` does
not confer rewrite capability on already-populated RS-coded BLOCK1/2.
The pinned tool's MAC `raw_value` byte order is reversed relative to its
formatted MAC. Validate that conversion with the saved baseline and a
nonuniform synthetic MAC, including the final ROM identity check.

Stock `summary` filters the fields it loaded but can silently omit requested
names. In particular, its C6 constructor loads the trim/calibration rows
conditionally on block version. A zero exit status or a plausible scaled value
therefore cannot admit the export. Require the exact expected key set, no
duplicates, canonical names/widths/blocks, readable rows and raw bits. Version
zero or another unsupported interpretation remains an explicitly rejected or
unsupported full export; never fill omitted rows with zero. Raw-field retention
does not attest that every value has a valid calibration interpretation.

## Capture and recovery

Before collection, freeze the prospective method, exact stock tools/packages,
selected definitions, host composition, whitelist and saved unit binding.
Require the sole intended bridge/native pair, matching saved serials and
physical locations, with relevant serial endpoints unowned. Use the existing
field-selected stock JSON command through the bridge. Bound the child, preserve
stdout/stderr, partial output and failed receipts, and retain failure sticky.

Always perform an independent matched ROM factory-MAC check and warning-free
normal C6 hard reset after the attempted read, including validation failures,
timeouts and interruptions. Recovery uses the intended bridge even if native
USB is temporarily absent. A successful host reset sequence does not attest
application health, later persistence or RF behavior.

The stock tool reads blocks internally beyond the selected formatted rows;
this is not a claim of limited electrical read coverage. Its common SPI/XMC
startup/reset hooks remain broader than a pure eFuse read. No physical eFuse
burn, configuration lock, bridge programming, security-register operation,
key deletion or C6 flash-array programming command is requested.

Qualify the actual stock virtual-summary path before collection, including a
synthetic supported-version case and the real silent-omission behavior.
Exercise missing/extra/duplicate/protected/wrong-width/raw-value and unit-binding
contrary cases, then obtain independent review of the final composition.
Virtual controls must use explicit synthetic files and never a physical port.
Independently replay the complete raw export and reset receipts; seal all
retained files. Publish coverage and qualified limits, without raw trim values,
their combined vector or a hash that identifies it.
