# Independent review of clocked handoff implementations

Reviewed 2026-10-23 by an agent separate from the HAL and ESP-IDF adapter
implementers. Scope: frozen protocol-2 source, relevant installed upstream source,
generated IDF configuration, and existing linked artifacts. This review performs
no code changes, builds, hardware access or deployment. Host collector behavior
has its separate [protocol-2 review](benchctl-v2.md).

## Findings and disposition

No blocking defect was found in the clock-field interpretation, separation of
initiator/responder tasks, fixed workload/checksum, timing boundaries or reported
sample counts. Two implementation defects were found and corrected before
physical protocol-2 runs:

| Finding | Consequence | Disposition |
| --- | --- | --- |
| The first Embassy responder returned after its 1008th reply. | Its last measured exchange could include task-future destruction/despawn before the initiator resumed, unlike the preceding steady-state exchanges and IDF's shutdown outside timing. | Corrected before board operation: the final responder loops indefinitely and waits for the next request after every reply. This reviewer re-read the correction and confirmed final identities with the implementation agent. |
| An earlier IDF by-value output iteration copied sample-array data into a large iterator. | The linked main frame was 34,384 bytes, larger than its configured 32 KiB stack. | The IDF implementation agent caught this during artifact inspection and changed output to borrowed iteration before board operation. This reviewer independently confirmed the resulting 18,400-byte main frame. |

The resulting workload compares full channel/executor or channel/OS-task paths.
It does not isolate a kernel context switch. Embassy and Rust `std` channels
have different implementations and scheduling rules; these differences belong
in the interpretation rather than being attributed entirely to an OS kernel.

## Independent clock-source checks

The following local primary sources were inspected:

- Exact `esp32c6` 0.24.0 PAC modules `pcr/{sysclk_conf,cpu_freq_conf,mspi_clk_conf}.rs`
  and `spi0/clock.rs`, from the pinned Cargo dependency tree.
- `esp-hal` 1.2.0 `src/soc/esp32c6/clocks.rs`, including `PRESET_160` and actual
  PCR field writes.
- ESP-IDF v5.5.6, commit `880a203ae603d08a94d06aa01e0d6761958e5e9e`,
  `components/hal/esp32c6/include/hal/clk_tree_ll.h`,
  `components/soc/esp32c6/register/soc/{pcr_reg,spi_mem_reg,reg_base}.h`, and
  `components/soc/esp32c6/include/soc/soc.h`.

The primary sources agree with both firmware adapters and the host decoder:

| Register | Address/field | Reviewed meaning |
| --- | --- | --- |
| PCR SYSCLK_CONF | `0x60096110`; source bits 16–17 | `1` selects SPLL; other sources are outside this controlled decoder. |
| PCR SYSCLK_CONF | HS divider bits 8–15 | Fixed field `2`, giving divide by three from 480 MHz SPLL to the 160 MHz HP root. |
| PCR CPU_FREQ_CONF | `0x60096118`; HS divider bits 8–15 | Legal fields `0`, `1`, `3`, giving HP-root divisors one, two, four. Bit 16 forces 120 MHz only at divider field zero. |
| PCR MSPI_CLK_CONF | `0x6009601c`; HS divider bits 8–15 | Legal fields `3`, `4`, `5` divide the 480 MHz root by four, five, six; the controlled 80 MHz source requires field five. |
| SPI0 CLOCK | `0x60002014`; bit 31 and bits 16–23 | Bit 31 selects divide by one; otherwise flash divides its input by `CLKCNT_N + 1`. The controlled flash result is 80 MHz with divider one. |

IDF `clk_tree_ll.h` explicitly defines `CLK_LL_PLL_480M_FREQ_MHZ` as 480 and
handles the exceptional 120 MHz CPU case when its getter reports overall
divider four. Its MSPI setter maps divisors four/five/six to fields three/four/five.
PAC fields match the masks/shifts used in IDF's small read-only MMIO boundary.
`DR_REG_PCR_BASE` is `0x60096000`; `DR_REG_SPI0_BASE` is `0x60002000`.

HAL `PRESET_160` requests PLL, CPU divider field zero, and MSPI divider field
five. That explains why a bootloader's earlier flash configuration is not
sufficient evidence of the application's final MSPI source frequency. Both
adapters read the four registers after warmups and after measurement; host
validation independently checks the advertised numbers and exact raw equality.

The readback establishes configuration assuming the documented SPLL source.
It does not measure oscillator error electrically, authenticate the running
firmware, or exclude a transient change between equal endpoint snapshots. Four
sequential reads are not an atomic multi-register snapshot. No authored code
changes clocks after initialization; IDF's generated configuration has PM off.
Unsupported sources/dividers and noninteger-MHz results fail instead of being
silently inferred or truncated.

Primary links:
[PAC PCR](https://docs.rs/crate/esp32c6/0.24.0/source/src/pcr.rs),
[PAC SPI0](https://docs.rs/crate/esp32c6/0.24.0/source/src/spi0/clock.rs),
[HAL C6 clocks](https://docs.rs/crate/esp-hal/1.2.0/source/src/soc/esp32c6/clocks.rs),
[pinned IDF clock LL](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/hal/esp32c6/include/hal/clk_tree_ll.h),
[pinned IDF PCR registers](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/register/soc/pcr_reg.h),
[pinned IDF SPI-memory registers](https://github.com/espressif/esp-idf/blob/880a203ae603d08a94d06aa01e0d6761958e5e9e/components/soc/esp32c6/register/soc/spi_mem_reg.h).

## Task independence and measured intervals

Embassy's initiator and responder are separate executor tasks. Static request
and reply channels each hold one `u32` and use upstream
`CriticalSectionRawMutex`. No authored critical section spans an await. The
initiator sends its sequence and waits for a reply; the independent responder
alone receives the request and computes the XOR reply. On this cooperative
executor, awaiting the empty reply queue gives the responder an opportunity to
run. Compute has no await and does not run concurrently with that cooperative
responder. The UART borrow belongs exclusively to the initiator.

IDF's `std::thread::Builder` creates a separate responder thread through Rust
1.96.0 `std/src/sys/thread/unix.rs`; its ESP-IDF path sets pthread stack
attributes, calls `pthread_create`, and later calls `pthread_join`. IDF's
`components/pthread/pthread.c` implements creation with
`xTaskCreatePinnedToCore`. These are separate actual FreeRTOS tasks, not two
functions inside one executor task. Priorities are queried from both tasks and
recorded; the generated configuration defaults are initiator priority one,
worker priority five, and 100 Hz tick. Their unequal priorities affect this
path and must remain visible. Priority numbers are not portable across OSes.

Both implementations use capacity-one request/reply queues, eight complete
unmeasured warmup exchanges, then 1,000 measured sequences starting again at
zero. Each interval starts before sending a request and ends immediately after
receiving its reply. The response is `index XOR 0xa5a55a5a`; validation and
sample storage occur after the second timer observation. There is at most one
request in flight. Timing includes the actual queue operations, wakeups,
executor/task scheduling and interrupt interference in both directions.

Both adapters perform eight compute warmups, then snapshot clocks, record 64
shared compute and 64 timer observations, and finally record message samples.
The same `bench-core` seed, 4096 rounds and workload revision/checksum apply.
Timer observations are integer microseconds multiplied by 1000; zero
calibration/message observations express quantization, not an inferred
zero-cost scheduler. No observer subtraction occurs. Bare-metal reports no
message benchmark or scheduler metadata.

All JSON formatting, UART writes, final flush and host round trips occur after
sample collection and the final clock snapshot. IDF additionally closes channels
and joins the worker before its final snapshot, outside measured intervals.
Task/channel creation and priority metadata exchange precede warmup/timing.
The host requires actual complete sample sets as well as truthful advertised
counts, and refuses invalid/incomplete/rebooted runs without a substitute score.

## Memory, allocation and failure review

HAL sample arrays are bounded, with no allocator in either adapter. Cross-await
Embassy state lives in upstream task storage. `sample_buffer_bytes` describes
the arrays, not total runtime RAM, stack use or task/channel overhead.

IDF buffers 18,048 sample bytes in its main stack and requests an 8,192-byte
worker stack. Channel/thread construction allocates before timing. Inspected
Rust 1.96.0 `std/src/sync/mpmc/{array,context,waker}.rs`: bounded channel
storage is allocated initially; blocking contexts are cached in thread-local
storage; waiter-vector removal retains capacity. Eight warmups exercise the
same one-initiator/one-responder pattern. This supports the narrower claim of
no authored per-message allocation and warmed channel state. It is not an
on-board allocator trace or a guarantee for arbitrary channel contention.

The generated IDF configuration confirms `CONFIG_ESP_MAIN_TASK_STACK_SIZE=32768`,
`CONFIG_FREERTOS_HZ=100`, `CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5`, main on CPU0,
and `CONFIG_PM_ENABLE` unset. The final IDF ELF SHA-256 is
`2c29bbbc2c7fe9ebb433907d9a2121c121ad67c460d4f1ead99f50e552991cf9`.
Using its pinned GCC objdump, this reviewer independently inspected the linked
`esp_idf_baseline::main` prologue at `0x42006942`: it subtracts 256 bytes and
then `0x46e0` (18,144), totaling an 18,400-byte frame. The difference from
32,768 is 14,368 bytes for caller/nested frames and other stack use. This is
arithmetic margin, not a measured stack high-water mark or a complete static
worst-case stack proof.

Clock/timer, channel, spawn, reply and output failures never deliberately emit a
later successful completion. IDF closes both endpoints and joins its worker on
ordinary measurement error. A hung responder has no firmware deadline; the host
must preserve the incomplete run and time out. HAL stops on failure/panic;
IDF panic handling can reboot and the collector rejects a merged/restarted boot.
Cancellation/recovery is outside this single-run workload. A final transport
flush can fail after completion was queued; physical evidence is the received
transcript, not an assumption about the writer's result.

## Final artifact check

IDF source SHA-256 at review:
`5155d6052ca0e3054ec1cb999c8be52447cd9606c0b73df9aa97661c80defcc7`.
HAL clock-decoder source SHA-256 at review:
`809acc669b5d8ca7081b5e361fa62f3f3979578633afa5cb78146650ef213f68`.

The corrected HAL `responder` now uses an infinite receive/reply loop. Its final
reply is followed by the same pending request receive as all preceding replies,
so the last measured exchange no longer performs task destruction/despawn.
The installed Embassy executor 0.10.0 `src/raw/mod.rs` confirms that a returning
future would otherwise run `drop_in_place` and `state.despawn` on `Poll::Ready`.

Final HAL measurement-source SHA-256:
`e43ee6c0226cc815237c9fb780e5a7cd1ce8de0f043dbdcac61ddfe802edef6f`.
Final Embassy ELF SHA-256, independently read and confirmed post-fix with the
implementation agent:
`8d06354ca14e9a51550fd3183c9f29750f32fc73880140c6d9ca8151e0a18d44`.
The agent reports all seven required build/formatter/lint/doc checks passed
without warnings; this reviewer did not repeat those builds. Bare-metal's final
ELF identity reported by its owner is
`be14bac3563b52d5e043485c9f798a4e23d6ed8702b72c9a3028c1cf46f58d0b`.

Physical capture/replay evidence will be reviewed separately when available.
