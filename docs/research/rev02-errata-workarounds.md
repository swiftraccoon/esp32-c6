# ESP32-C6 revision 0.2 errata

The [hardware inventory](../hardware.md) records the ESP32-C6-DevKitC-1-N8 and
observed chip revision 0.2. Applicability below is pinned to Espressif's errata
source at the 2026-10-23 inspection; it is not a new hardware test.

## Sources and exact scope

The authoritative snapshot is Espressif's
[`esp-chip-errata` commit `0875216a89d4c295549a535e7aabdbfaefbab53e`](https://github.com/espressif/esp-chip-errata/tree/0875216a89d4c295549a535e7aabdbfaefbab53e).
`git ls-remote ... HEAD` returned that revision on the review date; the raw
summary, descriptions and revision history were then inspected at that commit.
The C6 [revision history](https://github.com/espressif/esp-chip-errata/blob/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/revision-history/esp32c6-revision-history.rst)
identifies document **v1.3, 2026-08-18**, adding CPU-863. The
[pinned affected-revision table](https://github.com/espressif/esp-chip-errata/blob/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/02-errata-summary/esp32c6-errata-summary.rst)
and conditional C6 sections govern the ledger below. A search result pointing
to an older PDF is insufficient; the mutable [rendered summary](https://docs.espressif.com/projects/esp-chip-errata/en/latest/esp32c6/02-errata-summary/index.html)
is convenient for reading but is not the frozen revision.

## Complete official applicability ledger

All ten entries in the pinned C6 summary are accounted for here. “Fixed” means
the official description names a silicon revision containing the fix; it is
not a local regression-test result. A peripheral remaining untested is separate
from whether an old erratum applies.

| Erratum | Applies to our v0.2? | Exposure and required disposition |
| --- | --- | --- |
| **CPU-863** — misaligned data load followed by store across differing PMP/PMA permissions | **Yes** | Relevant to misaligned loads followed by stores across differing protection/memory permissions. Audit the actual linked copy/load paths; natural alignment and the documented instruction-spacing workaround have distinct scopes. |
| **CLK-8588** — internal RC32K may fail at low temperature | **Yes** | Internal RC32K may fail at low temperature. Select an unaffected clock source or qualify the actual clock/environment; a room-temperature run does not resolve the erratum. |
| **RMT-176** — continuous-TX final idle level | **Yes** | For continuous transmission, select register-controlled idle output and verify the final line level for the actual driver/configuration. |
| **CPU-206** — HP execution in LP SRAM and deferred write/deadlock pattern | **No; fixed in v0.2** | Do not impose an older-revision compressed-instruction/fence workaround on our chip merely because LP-core work is planned. LP loader, synchronization and recovery still need qualification for their own correctness. |
| **CLK-6996** — inaccurate RC_FAST calibration | **No; fixed in v0.1** | Later oscillator calibration still needs accuracy measurement; this earlier-revision defect is not its explanation on our chip. |
| **RES-7080** — incorrect reporting of RTC-watchdog system reset | **No; fixed in v0.1** | Watchdog/reset behavior is still unqualified; do not label a future failure this erratum without different evidence. |
| **SPI-304** — auto-suspend read abnormality | **No; fixed in v0.2** | Flash erase/suspend/cache behavior remains a future test. No revision-specific workaround is required by this entry for our observed chip. |
| **ADC-305** — SAR ADC/GDMA duplicate data | **No; fixed in v0.2** | ADC/DMA qualification remains required, but this fixed defect is not evidence against v0.2. |
| **ADC-1477** — SAR ADC lower-four-bit precision loss | **No; fixed in v0.2** | This does not establish calibrated ADC accuracy, noise or throughput on our board. |
| **WIFI-9686** — FTM initiator limitation | **No; fixed in v0.2** | Hardware fix does not establish that a chosen driver/Rust API implements FTM initiation. |

The fixed-revision conclusions were checked in the pinned individual
descriptions under [shared](https://github.com/espressif/esp-chip-errata/tree/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/03-errata-description/shared)
and [C6-specific](https://github.com/espressif/esp-chip-errata/tree/0875216a89d4c295549a535e7aabdbfaefbab53e/docs/en/03-errata-description/esp32c6)
sources, rather than inferring “not affected” from an omitted search result.


The [OS firmware exposure audit](../../projects/os-evaluation/docs/research/errata-exposure.md)
traces the historical IDF, direct-HAL and Ariel copy, clock and RMT paths.
Other firmware must establish its own selected configuration and linked behavior.
