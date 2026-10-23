# Independent queue experiment integration review

Reviewed **2026-10-23** by an agent separate from the firmware, collector and
study-analysis authors. Scope: the [experiment](../experiments/native-queue-comparison.md),
[timing contract](../research/timing-and-concurrency-contract.md),
[rev0.2 errata](../research/rev02-errata-workarounds.md),
[experimental design](../research/experimental-design.md), authored Rust source,
selected SDK sources, generated bindings/configuration, and final linked ELFs.
The [coding standard](../coding-standard.md) and all 54 active
[API identifiers](../rust-api-checklist.md) govern the review.

**No source-level or linked-artifact blocker remains for physical qualification.**
This reviewer did not build, flash, reset, monitor or otherwise access the
board. Build checks reported by their owners and read-only binary inspection
are not an on-board pass. Comparative collection still requires successful
physical qualification, frozen artifacts and a committed schedule/manifest.

## Findings and dispositions

| Finding | Disposition before collection |
| --- | --- |
| A sleeping Rust thread alone did not prove execution of WFI. | Added a separate explicit call to the existing SDK `esp_cpu_wait_for_intr`, debugger rejection, a bounded call loop, and raw-counter progress against 3–50 elapsed kernel ticks. No C or assembly was authored. |
| The raw timer's final HAL call resolves to ROM rather than the SDK C body. | Traced the final ELF's `0x400003cc` alias and inspected the supplied vendor ROM ELF's entry and actual snapshot/read/retry implementation. The source contract no longer rests solely on the generic C implementation. |
| Firmware and collector initially differed on duration admission. | Both now reject request/reply durations above one second and blocked qualification above 500 ms. The collector's mutation tests include these bounds. A failure is retained; its value is not clipped or replaced. |
| Ordinary measurement failures initially lost sample context. | Firmware now retains the initialized sample prefix and, after cleanup, emits labelled partial diagnostics with index, raw timestamps and checksum. These are non-JSON failure evidence without a successful hello/completion. A panic, transport fault or stuck kernel can still prevent the dump. |
| Worker phase publication is part of the measured path. | Documented: the higher-priority worker can publish its next phase and block again before main timestamps the previous reply. Main's readiness queries occur outside timing; worker bookkeeping remains inside the full path in both variants. |
| Study analysis initially learned image identity from accepted runs and lacked timestamp-order admission. | It now checks the frozen manifest's schedule, ELF and provenance hashes, matches each capture to its backend artifacts, and requires increasing capture timestamps in scheduled order. End-to-end synthetic-study tests cover complete and invalid studies. |

## Firmware ownership, readiness and timing

Only the message backend changes between feature-selected binaries. Both use
the same Rust pthread creation, observed priorities 1/5, requested 8 KiB worker
stack, 32 KiB main stack, eight warmups, sequence/checksum, phase protocol,
timestamp endpoints and shutdown. There are two capacity-one copied-`u32`
objects and at most one request in flight. The native queue uses generated SDK
bindings and the pinned header's base/send-to-back constants.

The native wrapper checks allocation and foreign statuses. Aligned initialized
`u32` storage remains live through each blocking FFI call; the kernel copies
four bytes and retains no caller pointer. `Arc` ownership spans every call,
including a blocked wait. Its final owner invokes `vQueueDelete` once, with no
remaining wrapper operation or waiter. Its documented `Send`/`Sync` invariants
depend on FreeRTOS's task-safe queue APIs; no raw handle, ISR API or cancellation
interface is exposed. Rust allocation failure outside the explicitly fallible
sample/native allocation paths can still abort/reset the firmware.

The worker publishes a sequence-specific phase immediately before its intended
receive. Main checks that phase, the actual task's blocked state, and the failure
flag; native queues additionally expose empty occupancy. Std has no public
non-destructive occupancy query: its emptiness follows from the single-inflight
protocol and reviewed worker path, rather than a fabricated kernel query.
Successful readiness counters describe admitted warmup/measured trials, not
every polling iteration. An additional pre-timer admission check is separate.

The task handle stays live through all queries. Normal exit requires main's
stop sentinel after the final interval; an error publishes failure and parks
until main authorizes exit. Release/acquire atomics publish this state; volatile
register access is not used as synchronization. Sentinel, join, queue deletion,
sample output and formatting are outside successful timed intervals. A genuinely
stuck blocking operation can require the host's timeout and reset.

Raw timestamps are 52-bit UNIT0 SYSTIMER ticks at nominal 16 MHz. The SDK owns
the snapshot handshake. Backwards or out-of-width values fail rather than being
normalized as wrap. Before/after register evidence independently checks CPU,
flash, timer source/enables and the SDK's **enabled CPU-stall bit**. That bit is
not mistakenly required to be clear. CONF bit 31 is retained as register clock
gating, separate from the counter's functional enable.

The legacy microsecond bracket shares UNIT0. The selected FreeRTOS tick uses
SYSTIMER UNIT1, so the WFI comparison adds a separate counter/interrupt path,
not an external oscillator reference. WFI qualification checks the selected SDK
call path with no debugger and normal enabled interrupts; it neither measures
sleep current nor establishes external frequency accuracy. Both variants retain
gross observations and their empty-observer distribution without subtraction.

## Independently inspected linked evidence

Stable copies were read from `firmware/esp-idf/.cache/queue-api/`. Their hashes
matched the firmware author's final build identities:

| Variant | ELF SHA-256 | Main / worker-body frame |
| --- | --- | --- |
| Std channel | `0282b814660461f7c88d3eb64c7d701e620e418d53be7d53b5eb96834a82d32b` | 768 / 192 bytes |
| Native queue | `fd6501102658572f12850512116401eaf57f6c330becd288ed8c825117e6fac8` | 768 / 192 bytes |

These frames are individual prologues, not measured stack high-water marks or a
whole-call-chain stack bound. The 2,000 samples occupy 48,000 preallocated heap
bytes, outside timing. They are not a claimed total runtime RAM footprint.

Both ELFs contain `esp_timer_impl_get_counter_reg` at `0x408018ca`, size
`0x12` (18 bytes), passing counter zero to ROM alias `0x400003cc`. The supplied
`esp32c6_rev0_rom.elf` from SDK tool package `esp-rom-elfs/20260528` has SHA-256
`788e1d38724aeb8fd974fa10c4a7b089c02627d35342ce84b9e0b12b239f3551`.
Its entry jumps to `0x40020864`: UNIT0 requests snapshot bit 30, waits for valid
bit 29, reads LO/HI/LO, masks the 20-bit high word and retries when the low
words differ. This inspects the vendor artifact; it does not attest a fresh
readback of this board's ROM. Bounded disassembly is preserved under
[firmware artifacts](../../firmware/esp-idf/artifacts/queue-api/rom-timer-body.txt).

SDK `esp_cpu_wait_for_intr` is at `0x40802c4a`, size `0x22` (34 bytes), with the
literal WFI instruction at `0x40802c62`. Its no-debugger path reaches that
instruction. `memcpy` is linked text at `0x408099fe` / `0x40809ba8` respectively;
its short word loop separates load and store with pointer increments, consistent
with the selected IDF CPU-863 mitigation. This is bounded emitted-path evidence,
not whole-program erratum certification.

The reviewed firmware source hashes are retained in
[inputs.sha256](../../firmware/esp-idf/artifacts/queue-api/inputs.sha256).
The firmware author reported both release builds, formatter, both target Clippy
checks and both documentation checks passing without warnings. This reviewer
did not duplicate those builds.

## Collector and paired analysis

The [collector review](benchctl-queue.md) covers strict protocol-3 framing,
counts, nonce/backend/configuration agreement, ordered raw endpoints,
qualification, completion and sticky rejection. Cross-reading against firmware
confirmed matching field names, types and success requirements. A failed or
incomplete boot has no numeric summary. The nonce is a startup observation,
not authenticated firmware identity or proof of independent resets.

The [study review](queue-study.md) covers its API applicability and verification.
Independent inspection confirmed balanced two-AB/two-BA order within each of
three sessions, seed-derived schedule validation, frozen image/provenance
admission, raw replay against capture reports, unique admitted nonces, stable
configuration and chronological admitted captures. Missing, failed, incomplete
and invalid runs remain distinct. A failed scheduled block is not replaced to
obtain a successful subset. Permitted fixture invalidation/repeat needs the
explicit external attempt ledger; the analyzer does not authorize it.

The signed primary contrast is native minus std boot-p50. Even-count medians
average the central pair using wider arithmetic. Only twelve complete blocks
receive the third-/tenth-order-statistic interval, whose stated continuous IID
coverage is 96.142578125%. It is conditional inference about the median paired
contrast, not event-tail latency or a hardware-population result. Session/order
evidence must still be inspected before relying on that model.

The root author reported 35 workspace unit tests plus two documentation tests,
Clippy, documentation and release-build checks passing. Reviewed synthetic tests
exercise 24 distinct boots, twelve known contrasts, interval endpoints,
failed-block suppression of the interval, timestamp disorder and changed frozen
artifacts. These are host/software checks; no synthetic transcript is treated
as a physical measurement. Affected API entries include `C-SERDE`,
`C-SEND-SYNC`, `C-GOOD-ERR`, `C-FAILURE`, `C-NEWTYPE`, `C-CUSTOM-TYPE`,
`C-VALIDATE`, `C-DTOR-FAIL` and `C-DTOR-BLOCK`; complete applicability decisions
belong to the component reviews, with no new public API or license exception
introduced by this integration review.

## Read-only hardware preparation audit

The retained original backup was independently checked by local file size and
SHA-256: **8,388,608 bytes**, at ignored path
`backups/original-2026-10-22/flash-backup.bin`, hash
`13cde854575fbb57fe3b49ba6f77dd96d32abae487ec5640c590771f57c9faf4`.
The [previous device-state record](../../results/clocked-handoff-2026-10-23/device-state.json)
and its write/verification logs establish the previous full restoration,
independent digest verification and RTS reset. This reviewer did not query the
present flash state; the operator must retain current pre-flash verification.

Esptool 5.1.0's locally inspected help confirms this restore sequence. These
commands are a preparation reference and were **not executed** by this reviewer:

```sh
esptool --chip esp32c6 --port /dev/cu.SERIAL_PORT --baud 115200 --after no-reset write-flash 0 backups/original-2026-10-22/flash-backup.bin
esptool --chip esp32c6 --port /dev/cu.SERIAL_PORT --baud 115200 --after hard-reset verify-flash 0 backups/original-2026-10-22/flash-backup.bin
```

Keep the original header settings; this sequence adds no encryption, force,
eFuse or full-chip-erase option. Retain transfer evidence with the backup and
collection records. Measurement serial bytes remain unchanged.

Before comparative captures, the operator still needs to:

1. Archive each feature-selected ELF immediately, before the other build
   replaces the shared Cargo output path. Retain its matching map, SDK config,
   generated bindings, source/lock snapshot, bootloader, partition table,
   merged image, hashes and build/inspection logs. `cp`, `tar` and `shasum`
   operate on these local artifacts; do not rebuild while freezing them.
2. Generate a new merged image with local espflash 4.6.0 `save-image`, DIO,
   80 MHz, 8 MB and 40 MHz XTAL. Pass the **matching** IDF bootloader and
   partition CSV/offset; do not substitute the historical protocol-1 bootloader
   or blindly reuse an old cached SDK output directory.
3. Preserve separate exploratory physical runs proving timer, WFI, full/empty,
   task/readiness, checksum and shutdown behavior for both backends. Qualification
   failures stop admission; changing the timing/backend then requires new frozen
   images. Source review has not performed these checks.
4. Freeze and commit the actual seeded 12-block schedule and manifest before
   characterization. Record each session's host/supply/environment and the
   attempt ledger; retain failures and the stated fixture-repeat limit.
5. Use `capture-queue` at 115200 8N1 with DTR deasserted and an RTS warm-reset
   request, a new output directory, the correct backend/ELF/provenance and prior
   observed nonces. Its default 180-second deadline accommodates the 2,000
   post-measurement serial records. Record flash evidence separately, replay
   retained raw captures, and restore/independently verify the full original
   image after the session.

The actual selected configuration has PM and the task watchdog disabled,
Bluetooth disabled, a 100 Hz kernel tick and no radio initialization. Wi-Fi
support is **compiled in** (`CONFIG_ESP_WIFI_ENABLED=y`); that is different from
initializing the radio. Existing interrupt/watchdog/panic choices remain part of
the measured platform. No new peripheral pins, radio peer or power instrument
is required for this UART-only study. Carrier identity, external timing/power
qualification and long-duration connected behavior remain later-phase limits.
