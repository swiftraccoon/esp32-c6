# Bridge serial delayed-reset result

The single declared timing setting did **not** produce the requested serial
response. All 678 custom configuration bytes read back exactly, including the
custom selector, string and valid Fletcher16. The USB serial response still
contains the original string. The whole serial-change trial remains failed.

All 678 original configuration bytes, the original bridge serial and the native
C6 serial were restored and independently replayed. Update-enable remains `FF`.
The final matched, warning-free normal C6 reset sequence completed; no
configuration restoration remains pending. No C6 firmware programming, eFuse
burn, key deletion or bridge lock was requested.

The [prospective method](../../experiments/bridge-serial-timing.md) tests the
vendor's suggestion to wait around Reset using one selected setting: ten host
seconds before and after each SDK reset, plus ten seconds on entering mandatory
restoration. The underlying USB observer, vendor SMT, configuration editor,
fresh-response checks and single original write remain unchanged.

Actual receipt intervals, measured from the earlier child's completion to the
later child's start, include the intended pause and intervening host work:

| Boundary | Host seconds |
| --- | ---: |
| Candidate write → SDK reset | 10.944476208 |
| Candidate SDK reset → initial discovery | 10.006220959 |
| Candidate Fletcher audit → original write | 11.048546417 |
| Original write → SDK reset | 10.960939667 |
| Original SDK reset → initial discovery | 10.004926667 |

Three complete USB captures contain 30 successful transfer rows. All pass
whole-directory replay and the separate existing Fletcher validator.
Independent review reproduces the complete candidate/original arrays, serial
relationships, five intervals, final reset and all 739 frozen inputs. Exactly
one candidate SetConfig, one original SetConfig and two SDK resets were invoked.
All preceding failed cohorts remain preserved.

The source hypothesis came from [Silicon Labs employee guidance](https://community.silabs.com/s/question/0D58Y0000ASRBBOSQ5/)
about Windows VID/PID configuration. It gives no numeric delay or confirmed
resolution for this Mac serial case. This result rejects the selected setting;
it does not establish the cause, exhaust possible delays or prove that a full
power cycle would resolve activation. Current official SDK package downloads
returned HTTP 403, so newer implementations were not inspected or substituted.

The selected SDK still ignores the OS reset return. There is no independent USB
bus trace, bridge MCU-reset/power-loss attestation or power-cycle persistence
result. The recorded USB responses are those returned through the attached
host's control-request API; lower kernel caching was not independently excluded.
Later application health, PHY persistence and RF behavior remain unmeasured.

Raw strings/configurations, host records and complete diagnostics remain private.
The [summary](summary.json) and [provenance](provenance.json) are sanitized
derivatives. `SHA256SUMS` covers every retained public regular file except itself
relative to this directory, establishing retained-byte integrity rather than
hardware authenticity.
