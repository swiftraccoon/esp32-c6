# Clock-controlled message handoff experiment

Prepared 2026-10-22 before new on-board captures. This extends the
[first compute baseline](../../results/initial-2026-10-22/README.md) without
changing its archived evidence. Results are recorded separately when measured.

## Question and controls

How much time does a capacity-one request/reply exchange take between two
independent execution contexts in the selected Rust application stacks? Retain
the same compute workload as a control, and resolve the first experiment's
unverified flash-clock configuration before comparing the new results.

Use the same physical C6, compiler 1.96.0, compute source, seed, rounds,
optimization profile, UART and one-microsecond timer resolution. Select CPU
160 MHz and flash 80 MHz, DIO, 8 MB. Read the active CPU source/dividers and both
the MSPI source divider and downstream SPI0 divider before the first and after
the last timed sample. IDF's final snapshot follows explicit worker shutdown and
join, which remain outside measured intervals. Require identical snapshots and
independent host decoding. These are
configuration readbacks, not oscilloscope measurements; two snapshots cannot
exclude a transient clock change. Dynamic power management remains disabled.

Record four raw registers: `PCR_SYSCLK_CONF`, `PCR_CPU_FREQ_CONF`,
`PCR_MSPI_CLK_CONF` and `SPI0_CLOCK`. Their source definitions are the pinned
[ESP32-C6 PAC](https://docs.rs/esp32c6/0.24.0/esp32c6/) and the matching
[ESP-IDF clock-tree implementation](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/esp32c6/include/hal/clk_tree_ll.h).
The firmware reads configuration only; it does not experiment with raw clock
register writes. An unsupported source or divider fails the run.

## Measurement contract

| Item | Contract |
| --- | --- |
| Compute control | Eight warmups, then 64 calls of the existing 4096-round `integer-mix-v1`; expected checksum `0xa3944d55` |
| Timer observer | 64 pairs of timer reads, reported independently without subtraction |
| Message payload | One `u32` sequence number, response is sequence XOR `0xa5a55a5a` |
| Message path | Two independent tasks/threads; request and reply queues each have capacity one |
| Message warmup | Eight complete, checked round trips, excluded from reported samples |
| Message timing | 1000 sequential round trips; start before sending the request, stop after receiving its reply; verify every reply |
| Storage and logging | Bounded sample storage allocated before timing; no intentional application logging or output during timing |
| Startup | Two-second settling interval before warmups |
| Repetitions | Five warm-reset captures per platform, alternating platform order; preserve failures and outliers |
| Bare-metal control | Compute and observer only; message handoff has no numeric score because this adapter has no scheduler |

Embassy uses two async tasks and `embassy-sync` channels. ESP-IDF uses the
selected Rust standard library's `sync_channel` between the main FreeRTOS task
and a separate Rust thread. Record actual IDF priorities, requested thread stack
sizes, main stack configuration and tick rate. These primitives have different
implementations; their complete request/reply costs are the result being
measured. A round trip includes queue operations, wakeups and runtime dispatch.
It is not a direct measurement of one kernel context switch, and dividing its
duration by two does not establish one-way latency.

The one-at-a-time protocol ensures each sender consumes a matching response
before its next request. It does not measure sustained multi-producer traffic,
overload, priority interference, fairness or a saturated queue. Radios are not
initialized. No new networking, power, peripheral or security claim follows.

## Validation and provenance

Protocol 2 declares its exact benchmark names/counts, initial clock snapshot and
message configuration in `hello`; `complete` includes final clocks and explicit
counts. The host rejects missing, duplicate, out-of-order, incorrectly checked,
wrong-version or unexpected samples, an unexpected reboot, changed clocks and
inconsistent register decoding. Zero-duration message/observer samples may
reflect timer quantization; a zero compute result is invalid. Protocol 1 replay
remains supported for the original captures.

Archive the exact source snapshot, lockfiles, ELF, generated image, bootloader,
partition data, SDK configuration, build/review records and hashes. Check the
original full-flash backup against the board before changing firmware; restore
and verify the full image after the experiment. Use a separate results directory
and retain every capture attempt. Summarize within-boot nearest-rank percentiles
and the range across boots, retaining raw samples. A passing capture qualifies
this experiment only.
