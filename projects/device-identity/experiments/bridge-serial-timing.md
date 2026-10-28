# Bridge serial reset timing distinction

Decision question: does one bounded delay setting around the existing SDK reset
produce the exact custom serial response missing from the preceding trials?
Reuse the [bridge serial method](bridge-serial.md), unchanged vendor SMT,
existing USB observer and original/candidate array construction. All baseline,
topology, USB population, ownership, Fletcher, exact fresh-response admission,
single-write/restoration and final C6 reset requirements still apply.

## Source hypothesis and fixed schedule

A [Silicon Labs employee answer](https://community.silabs.com/s/question/0D58Y0000ASRBBOSQ5/i-am-using-cp2102n-usb-pidvid-assigned-by-silicon-labsi-write-the-pidvid-directly-on-the-ic-and-use-it-but-sometimes-it-returns-to-silicon-labs-own-pidvidi-would-like-to-know-the-cause-of-this-problem-and-would-like-some-advice?language=en_US)
recommends checking actual configuration values and increasing waits both
before and after Reset because configuration takes time. That report concerned
Windows VID/PID programming; it specifies no numeric wait and establishes no
cause or remedy for this Mac serial result. Earlier local trials reset
immediately after SetConfig and initially waited three seconds afterward.

Select exactly one setting for this distinction:

1. After the one candidate SetConfig, wait **10 host seconds** before its SDK
   reset, then initially wait **10 host seconds** before bounded discovery.
2. Perform the same complete fresh candidate capture/replay. Exact requested
   serial and all 678 candidate bytes remain necessary for trial success.
3. On entering mandatory restoration after any possible candidate write, wait
   an additional **10 host seconds** before the one original SetConfig. Recovery
   already ignores subsequent signals. This conservative pause also covers an
   interrupted or failed candidate pre-reset wait.
4. Wait **10 host seconds** before the original SDK reset and initially wait
   **10 host seconds** afterward. Complete the exact original capture/replay
   and matched, warning-free normal C6 reset.

The existing bounded absence-discovery retries add only their defined read-only
polling delays. No additional candidate/original write, reset opcode, firmware
update or automatic timing search is allowed. A wait error makes the attempt
fail; cleanup still attempts the original write/readback and final reset.

## Qualification and interpretation

Freeze this prospective schedule, actual compositions, tools and inputs before
collection. Qualify exact pause ordering on the actual host composition,
including cancellation during the candidate wait and the mandatory recovery
pause before another write. Retain preceding failed cohorts without repair.
Review unchanged validators and CLI arguments by exact source/diff binding;
rerun controls affected by the new timing and cleanup behavior.

Ten seconds is a selected test setting, not a vendor minimum or guaranteed fix.
There is no independent electrical clock, USB bus analyzer, bridge MCU-reset
attestation or full power removal. A host sleep receipt supports only that host
interval. A successful fresh custom response would qualify presence for this
sequence; it would not establish that the waits caused it or are universally
sufficient. A failure leaves this setting rejected without proving all possible
delays ineffective. Original restoration remains mandatory in either outcome.
