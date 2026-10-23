//! ESP32-C6 ESP-IDF/FreeRTOS clock and scheduler benchmark adapter.

use std::hint::black_box;
use std::io::{self, Write};
use std::sync::mpsc;
use std::time::Duration;

use bench_core::{DEFAULT_ROUNDS, DEFAULT_SAMPLES, DEFAULT_SEED, WORKLOAD_ID, workload};

const MESSAGE_SAMPLES: usize = 1_000;
const WARMUP_ITERATIONS: u32 = 8;
const MESSAGE_REPLY_MASK: u32 = 0xa5a5_5a5a;
const WORKER_STACK_BYTES: usize = 8_192;
const MAIN_STACK_BYTES: usize = 32_768;

/// One result, retained in bounded stack storage until timing has finished.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize,
)]
struct Sample {
    elapsed_ns: u64,
    checksum: u32,
}

/// Read-only snapshots of four ESP32-C6 clock configuration registers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
struct ClockRegisters {
    pcr_sysclk_conf: u32,
    pcr_cpu_freq_conf: u32,
    pcr_mspi_clk_conf: u32,
    spi0_clock: u32,
}

/// Clock frequencies decoded from the active PLL and bus divider fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
struct Clocks {
    cpu_mhz: u32,
    flash_source_mhz: u32,
    flash_mhz: u32,
    flash_divider: u32,
    registers: ClockRegisters,
}

impl Clocks {
    /// Reads and decodes the active PLL-based CPU and flash clock settings.
    ///
    /// # Errors
    /// Rejects unsupported sources/dividers or fractional-MHz flash clocks.
    fn read() -> io::Result<Self> {
        // SAFETY: These are the ESP32-C6's four aligned, mapped 32-bit R/W or
        // read-only clock configuration registers, not FIFO/read-to-clear data.
        // Offsets/fields match IDF v5.5.6 soc/{pcr,spi_mem}_reg.h. Reads create
        // no Rust references, retain no pointers and neither change clocks nor
        // claim peripheral ownership. PM is disabled; no application task/ISR
        // changes clock settings. Volatile is used for MMIO, not synchronization.
        let registers = unsafe {
            ClockRegisters {
                pcr_sysclk_conf: std::ptr::read_volatile(
                    (esp_idf_sys::DR_REG_PCR_BASE + 0x110) as *const u32,
                ),
                pcr_cpu_freq_conf: std::ptr::read_volatile(
                    (esp_idf_sys::DR_REG_PCR_BASE + 0x118) as *const u32,
                ),
                pcr_mspi_clk_conf: std::ptr::read_volatile(
                    (esp_idf_sys::DR_REG_PCR_BASE + 0x1c) as *const u32,
                ),
                spi0_clock: std::ptr::read_volatile(
                    (esp_idf_sys::DR_REG_SPI0_BASE + 0x14) as *const u32,
                ),
            }
        };
        let source = (registers.pcr_sysclk_conf >> 16) & 3;
        let root_divider = ((registers.pcr_sysclk_conf >> 8) & 0xff) + 1;
        let cpu_divider = ((registers.pcr_cpu_freq_conf >> 8) & 0xff) + 1;
        let mspi_divider = ((registers.pcr_mspi_clk_conf >> 8) & 0xff) + 1;
        if source != 1
            || root_divider != 3
            || !matches!(cpu_divider, 1 | 2 | 4)
            || !matches!(mspi_divider, 4..=6)
        {
            return Err(io::Error::other("unsupported clock register configuration"));
        }
        // ESP32-C6's selected SPLL is 480 MHz. IDF clk_tree_ll.h specifies a
        // special 120 MHz path when CPU_HS_DIV_NUM is zero and FORCE is set.
        let cpu_mhz = if cpu_divider == 1 && registers.pcr_cpu_freq_conf & (1 << 16) != 0 {
            120
        } else {
            480 / root_divider / cpu_divider
        };
        let flash_source_mhz = 480 / mspi_divider;
        let flash_divider = if registers.spi0_clock & (1 << 31) != 0 {
            1
        } else {
            ((registers.spi0_clock >> 16) & 0xff) + 1
        };
        if flash_source_mhz % flash_divider != 0 {
            return Err(io::Error::other(
                "flash clock is not an integer number of MHz",
            ));
        }
        let flash_mhz = flash_source_mhz / flash_divider;
        Ok(Self {
            cpu_mhz,
            flash_source_mhz,
            flash_mhz,
            flash_divider,
            registers,
        })
    }

    /// Writes a complete clock object after all timed measurements.
    ///
    /// # Errors
    /// Propagates the caller's writer failures.
    fn write_json(self, mut output: impl Write) -> io::Result<()> {
        write!(
            output,
            "{{\"source\":\"register-readback\",\"cpu_mhz\":{},\"flash_source_mhz\":{},\"flash_mhz\":{},\"flash_divider\":{},\"registers\":{{\"pcr_sysclk_conf\":{},\"pcr_cpu_freq_conf\":{},\"pcr_mspi_clk_conf\":{},\"spi0_clock\":{}}}}}",
            self.cpu_mhz,
            self.flash_source_mhz,
            self.flash_mhz,
            self.flash_divider,
            self.registers.pcr_sysclk_conf,
            self.registers.pcr_cpu_freq_conf,
            self.registers.pcr_mspi_clk_conf,
            self.registers.spi0_clock,
        )
    }
}

/// A monotonic timestamp in microseconds since ESP-IDF startup.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct TimestampMicros(i64);

impl TimestampMicros {
    fn now() -> Self {
        // SAFETY: esp_timer_get_time takes no arguments, retains no pointers,
        // and is callable after IDF startup from either application task. Its
        // generated C ABI binding returns microseconds as a signed 64-bit value.
        Self(unsafe { esp_idf_sys::esp_timer_get_time() })
    }

    /// Returns elapsed nanoseconds at the timer's 1,000 ns resolution.
    ///
    /// # Errors
    /// Returns an error if the timer moved backwards or conversion overflowed.
    fn elapsed_ns(self, end: Self) -> io::Result<u64> {
        let elapsed_us = end
            .0
            .checked_sub(self.0)
            .ok_or_else(|| io::Error::other("esp timer subtraction overflowed"))?;
        let elapsed_us =
            u64::try_from(elapsed_us).map_err(|_| io::Error::other("esp timer moved backwards"))?;
        elapsed_us
            .checked_mul(1_000)
            .ok_or_else(|| io::Error::other("nanosecond conversion overflowed"))
    }
}

fn current_task_priority() -> u32 {
    // SAFETY: In normal FreeRTOS task context, the documented null handle asks
    // for the calling task's priority. No pointer is dereferenced or retained by
    // the caller; the generated return type is a 32-bit unsigned value on C6.
    unsafe { esp_idf_sys::uxTaskPriorityGet(std::ptr::null_mut()) }
}

/// Buffers compute, observer and two-task handoff samples before UART output.
///
/// # Errors
/// Returns thread/channel failures, invalid replies/clocks, timer failures and
/// UART failures. A failed measurement emits no successful complete record.
fn run() -> io::Result<()> {
    let mut compute = [Sample::default(); DEFAULT_SAMPLES as usize];
    let mut timer_overhead = [Sample::default(); DEFAULT_SAMPLES as usize];
    let mut messages = [Sample::default(); MESSAGE_SAMPLES];
    let (request_tx, request_rx) = mpsc::sync_channel::<u32>(1);
    let (reply_tx, reply_rx) = mpsc::sync_channel::<u32>(1);
    let sender_priority = current_task_priority();
    let responder = std::thread::Builder::new()
        .name("message-responder".into())
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || -> io::Result<()> {
            reply_tx
                .send(current_task_priority())
                .map_err(|_| io::Error::other("priority metadata receiver disconnected"))?;
            while let Ok(sequence) = request_rx.recv() {
                reply_tx
                    .send(sequence ^ MESSAGE_REPLY_MASK)
                    .map_err(|_| io::Error::other("message reply receiver disconnected"))?;
            }
            Ok(())
        })?;

    // Always close both channel endpoints and join the worker outside timing,
    // including when the inner measurement returns an error.
    let measurements = (|| -> io::Result<(u32, Clocks)> {
        let receiver_priority = reply_rx
            .recv()
            .map_err(|_| io::Error::other("message worker did not report its priority"))?;
        std::thread::sleep(Duration::from_secs(2));
        for _ in 0..WARMUP_ITERATIONS {
            black_box(workload(black_box(DEFAULT_SEED), black_box(DEFAULT_ROUNDS)));
        }
        for sequence in 0..WARMUP_ITERATIONS {
            request_tx
                .send(sequence)
                .map_err(|_| io::Error::other("warmup request receiver disconnected"))?;
            let reply = reply_rx
                .recv()
                .map_err(|_| io::Error::other("warmup reply sender disconnected"))?;
            if reply != sequence ^ MESSAGE_REPLY_MASK {
                return Err(io::Error::other("message warmup reply mismatch"));
            }
        }
        let clocks = Clocks::read()?;

        for sample in &mut compute {
            let start = TimestampMicros::now();
            let checksum = black_box(workload(black_box(DEFAULT_SEED), black_box(DEFAULT_ROUNDS)));
            let end = TimestampMicros::now();
            *sample = Sample {
                elapsed_ns: start.elapsed_ns(end)?,
                checksum,
            };
        }
        for sample in &mut timer_overhead {
            let start = TimestampMicros::now();
            black_box(());
            let end = TimestampMicros::now();
            *sample = Sample {
                elapsed_ns: start.elapsed_ns(end)?,
                checksum: 0,
            };
        }
        for (index, sample) in messages.iter_mut().enumerate() {
            let sequence = u32::try_from(index)
                .map_err(|_| io::Error::other("message sequence does not fit u32"))?;
            let start = TimestampMicros::now();
            request_tx
                .send(sequence)
                .map_err(|_| io::Error::other("message request receiver disconnected"))?;
            let checksum = reply_rx
                .recv()
                .map_err(|_| io::Error::other("message reply sender disconnected"))?;
            let end = TimestampMicros::now();
            if checksum != sequence ^ MESSAGE_REPLY_MASK {
                return Err(io::Error::other("message reply mismatch"));
            }
            *sample = Sample {
                elapsed_ns: start.elapsed_ns(end)?,
                checksum,
            };
        }
        Ok((receiver_priority, clocks))
    })();
    drop(request_tx);
    drop(reply_rx);
    let worker_result = responder
        .join()
        .map_err(|_| io::Error::other("message worker panicked"))?;
    let (receiver_priority, clocks) = measurements?;
    worker_result?;
    let clocks_after = Clocks::read()?;
    if clocks != clocks_after {
        return Err(io::Error::other(
            "clock registers changed during measurement",
        ));
    }

    let sample_buffer_bytes = std::mem::size_of_val(&compute)
        + std::mem::size_of_val(&timer_overhead)
        + std::mem::size_of_val(&messages);
    let stdout = io::stdout();
    let mut output = stdout.lock();
    write!(
        output,
        "{{\"protocol\":2,\"event\":\"hello\",\"platform\":\"esp-idf-freertos\",\"cpu_mhz\":160,\"sample_count\":{DEFAULT_SAMPLES},\"seed\":{DEFAULT_SEED},\"rounds\":{DEFAULT_ROUNDS},\"workload\":\"{WORKLOAD_ID}\",\"warmup_iterations\":{WARMUP_ITERATIONS},\"timer_resolution_ns\":1000,\"benchmarks\":{{\"compute\":64,\"timer-overhead\":64,\"message-roundtrip\":{MESSAGE_SAMPLES}}},\"message\":{{\"transport\":\"rust-std-sync-channel\",\"queue_capacity\":1,\"warmup_iterations\":{WARMUP_ITERATIONS},\"sender_priority\":{sender_priority},\"receiver_priority\":{receiver_priority},\"worker_stack_bytes\":{WORKER_STACK_BYTES},\"main_stack_bytes\":{MAIN_STACK_BYTES},\"tick_hz\":100,\"sample_buffer_bytes\":{sample_buffer_bytes}}},\"clocks\":"
    )?;
    clocks.write_json(&mut output)?;
    writeln!(output, "}}")?;
    for (index, sample) in compute.iter().enumerate() {
        writeln!(
            output,
            "{{\"protocol\":2,\"event\":\"sample\",\"benchmark\":\"compute\",\"index\":{index},\"iterations\":{DEFAULT_ROUNDS},\"elapsed_ns\":{},\"checksum\":{}}}",
            sample.elapsed_ns, sample.checksum
        )?;
    }
    for (index, sample) in timer_overhead.iter().enumerate() {
        writeln!(
            output,
            "{{\"protocol\":2,\"event\":\"sample\",\"benchmark\":\"timer-overhead\",\"index\":{index},\"iterations\":0,\"elapsed_ns\":{},\"checksum\":0}}",
            sample.elapsed_ns
        )?;
    }
    for (index, sample) in messages.iter().enumerate() {
        writeln!(
            output,
            "{{\"protocol\":2,\"event\":\"sample\",\"benchmark\":\"message-roundtrip\",\"index\":{index},\"iterations\":1,\"elapsed_ns\":{},\"checksum\":{}}}",
            sample.elapsed_ns, sample.checksum
        )?;
    }
    write!(
        output,
        "{{\"protocol\":2,\"event\":\"complete\",\"samples\":1128,\"compute_samples\":64,\"timer_overhead_samples\":64,\"message_roundtrip_samples\":{MESSAGE_SAMPLES},\"clocks_after\":"
    )?;
    clocks_after.write_json(&mut output)?;
    writeln!(output, "}}")?;
    output.flush()
}

fn main() {
    esp_idf_sys::link_patches();
    if let Err(error) = run() {
        // The collector checks received bytes. Report failure outside timing;
        // a final flush can fail after completion has been queued.
        eprintln!("benchmark failed: {error}");
    }
}
