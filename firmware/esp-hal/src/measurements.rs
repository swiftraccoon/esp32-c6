//! Allocation-free measurement and protocol-v2 reporting for both adapters.

use core::fmt::{self, Write};
use core::hint::black_box;

use bench_core::{DEFAULT_ROUNDS, DEFAULT_SAMPLES, DEFAULT_SEED, WORKLOAD_ID, workload};
#[cfg(feature = "embassy")]
use embassy_executor::Spawner;
#[cfg(feature = "embassy")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "embassy")]
use embassy_sync::channel::Channel;
use esp_hal::time::{Duration, Instant};

use super::clocks::ClockSnapshot;

const WARMUP_ITERATIONS: u32 = 8;
const SAMPLE_COUNT: usize = DEFAULT_SAMPLES as usize;
#[cfg(feature = "embassy")]
const MESSAGE_SAMPLES: usize = 1000;
#[cfg(feature = "embassy")]
const REPLY_XOR: u32 = 0xa5a5_5a5a;

// CriticalSectionRawMutex is safe across executor/interrupt contexts. Channel
// operations briefly enter upstream critical sections; none spans an await.
// Only the main initiator and one responder task use these capacity-one queues.
#[cfg(feature = "embassy")]
static REQUESTS: Channel<CriticalSectionRawMutex, u32, 1> = Channel::new();
#[cfg(feature = "embassy")]
static REPLIES: Channel<CriticalSectionRawMutex, u32, 1> = Channel::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Sample {
    elapsed: Duration,
    checksum: u32,
}

impl Sample {
    const EMPTY: Self = Self {
        elapsed: Duration::ZERO,
        checksum: 0,
    };
}

/// An incomplete run never produces a successful completion record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RunError {
    UnsupportedClocks,
    ChangedClocks,
    Output(fmt::Error),
    #[cfg(feature = "embassy")]
    ResponderSpawn,
    #[cfg(feature = "embassy")]
    IncorrectReply {
        index: u32,
        expected: u32,
        received: u32,
    },
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedClocks => formatter.write_str("unsupported clock configuration"),
            Self::ChangedClocks => formatter.write_str("clock configuration changed during run"),
            Self::Output(error) => write!(formatter, "protocol output failed: {error}"),
            #[cfg(feature = "embassy")]
            Self::ResponderSpawn => formatter.write_str("responder task could not be spawned"),
            #[cfg(feature = "embassy")]
            Self::IncorrectReply {
                index,
                expected,
                received,
            } => write!(
                formatter,
                "reply {index} was {received:#010x}; expected {expected:#010x}"
            ),
        }
    }
}

impl core::error::Error for RunError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Output(error) => Some(error),
            _ => None,
        }
    }
}

impl From<fmt::Error> for RunError {
    fn from(error: fmt::Error) -> Self {
        Self::Output(error)
    }
}

/// Measures the CPU/timer floor and writes no fabricated handoff observations.
///
/// The caller initializes the HAL at 160 MHz and owns a blocking UART0 writer.
/// HAL timer observations have 1 microsecond resolution. All raw samples remain
/// in bounded storage until timing and read-only clock snapshots are complete.
/// No allocation, serial access, or formatting occurs in a measured region.
///
/// # Errors
///
/// Returns an error for unsupported/changing clocks or failed output. The caller
/// must stop without emitting a success after any error.
#[cfg_attr(
    feature = "embassy",
    allow(dead_code, reason = "The Embassy binary calls run_embassy instead")
)]
pub(super) fn run(writer: impl Write) -> Result<(), RunError> {
    warmup_compute();
    let clocks = ClockSnapshot::read().ok_or(RunError::UnsupportedClocks)?;
    let (compute, timer_overhead) = measure_baseline();
    let clocks_after = finish_clocks(clocks)?;
    write_run(
        writer,
        "bare-metal",
        clocks,
        clocks_after,
        &compute,
        &timer_overhead,
        &[],
    )
}

/// Measures baseline workloads and 1000 sequential request/reply task handoffs.
///
/// One independent responder receives a `u32` sequence value and replies with
/// that value XOR `0xa5a55a5a`. Both channels have capacity one. Eight unrecorded
/// warmup pairs precede timed pairs; every reply is validated outside timing.
/// Each interval includes send/receive channel operations and scheduling in both
/// directions. No logging or allocation occurs until all samples are buffered.
///
/// The caller never cancels this future. Dropping it can leave a request/reply
/// in flight and leaves the responder waiting, so the run becomes incomplete;
/// no partial result is reported as success and no recovery is attempted.
///
/// # Errors
///
/// Returns errors for responder spawn failure, unsupported/changing clocks,
/// incorrect replies, or output failure. Missing replies suspend this future;
/// the external collector must retain the incomplete run and time out.
#[cfg(feature = "embassy")]
pub(super) async fn run_embassy(writer: impl Write, spawner: Spawner) -> Result<(), RunError> {
    let token = responder().map_err(|_| RunError::ResponderSpawn)?;
    spawner.spawn(token);
    warmup_compute();
    for index in 0..WARMUP_ITERATIONS {
        REQUESTS.send(index).await;
        let received = REPLIES.receive().await;
        validate_reply(index, received)?;
    }

    let clocks = ClockSnapshot::read().ok_or(RunError::UnsupportedClocks)?;
    let (compute, timer_overhead) = measure_baseline();

    let mut messages = [Sample::EMPTY; MESSAGE_SAMPLES];
    for (index, sample) in messages.iter_mut().enumerate() {
        // This array has exactly 1000 entries, so the index fits in u32.
        let request = index as u32;
        let start = Instant::now();
        REQUESTS.send(black_box(request)).await;
        let checksum = black_box(REPLIES.receive().await);
        let elapsed = start.elapsed();
        validate_reply(request, checksum)?;
        *sample = Sample { elapsed, checksum };
    }
    let clocks_after = finish_clocks(clocks)?;
    write_run(
        writer,
        "embassy",
        clocks,
        clocks_after,
        &compute,
        &timer_overhead,
        &messages,
    )
}

/// A separate Embassy task performs only the fixed reply operation.
#[cfg(feature = "embassy")]
#[embassy_executor::task]
async fn responder() {
    // Park on the next receive after every reply, including the final sample.
    // Task teardown must not distinguish the last measured round trip.
    loop {
        let request = REQUESTS.receive().await;
        REPLIES.send(request ^ REPLY_XOR).await;
    }
}

#[cfg(feature = "embassy")]
fn validate_reply(index: u32, received: u32) -> Result<(), RunError> {
    let expected = index ^ REPLY_XOR;
    if received == expected {
        Ok(())
    } else {
        Err(RunError::IncorrectReply {
            index,
            expected,
            received,
        })
    }
}

fn warmup_compute() {
    for _ in 0..WARMUP_ITERATIONS {
        black_box(workload(black_box(DEFAULT_SEED), black_box(DEFAULT_ROUNDS)));
    }
}

fn measure_baseline() -> ([Sample; SAMPLE_COUNT], [Sample; SAMPLE_COUNT]) {
    let mut compute = [Sample::EMPTY; SAMPLE_COUNT];
    let mut timer_overhead = [Sample::EMPTY; SAMPLE_COUNT];
    for sample in &mut compute {
        let start = Instant::now();
        let checksum = black_box(workload(black_box(DEFAULT_SEED), black_box(DEFAULT_ROUNDS)));
        let elapsed = start.elapsed();
        *sample = Sample { elapsed, checksum };
    }
    for sample in &mut timer_overhead {
        let start = Instant::now();
        black_box(());
        let elapsed = start.elapsed();
        *sample = Sample {
            elapsed,
            checksum: 0,
        };
    }
    (compute, timer_overhead)
}

fn finish_clocks(clocks: ClockSnapshot) -> Result<ClockSnapshot, RunError> {
    let clocks_after = ClockSnapshot::read().ok_or(RunError::UnsupportedClocks)?;
    if clocks == clocks_after {
        Ok(clocks_after)
    } else {
        Err(RunError::ChangedClocks)
    }
}

fn write_run(
    mut writer: impl Write,
    platform: &str,
    clocks: ClockSnapshot,
    clocks_after: ClockSnapshot,
    compute: &[Sample],
    timer_overhead: &[Sample],
    messages: &[Sample],
) -> Result<(), RunError> {
    write!(
        writer,
        "{{\"protocol\":2,\"event\":\"hello\",\"platform\":\"{platform}\",\"cpu_mhz\":160,\"sample_count\":{DEFAULT_SAMPLES},\"seed\":{DEFAULT_SEED},\"rounds\":{DEFAULT_ROUNDS},\"workload\":\"{WORKLOAD_ID}\",\"warmup_iterations\":{WARMUP_ITERATIONS},\"timer_resolution_ns\":1000,\"clocks\":"
    )?;
    clocks.write(&mut writer)?;
    write!(
        writer,
        ",\"benchmarks\":{{\"compute\":{},\"timer-overhead\":{}",
        compute.len(),
        timer_overhead.len(),
    )?;
    if !messages.is_empty() {
        write!(writer, ",\"message-roundtrip\":{}", messages.len())?;
    }
    write!(writer, "}}")?;
    if !messages.is_empty() {
        let sample_buffer_bytes = (compute.len() + timer_overhead.len() + messages.len())
            * core::mem::size_of::<Sample>();
        write!(
            writer,
            ",\"message\":{{\"transport\":\"embassy-sync-0.8.0-channel\",\"queue_capacity\":1,\"warmup_iterations\":{WARMUP_ITERATIONS},\"sample_buffer_bytes\":{sample_buffer_bytes}}}"
        )?;
    }
    writeln!(writer, "}}")?;

    write_samples(&mut writer, "compute", DEFAULT_ROUNDS, compute)?;
    write_samples(&mut writer, "timer-overhead", 0, timer_overhead)?;
    write_samples(&mut writer, "message-roundtrip", 1, messages)?;

    write!(
        writer,
        "{{\"protocol\":2,\"event\":\"complete\",\"samples\":{},\"compute_samples\":{},\"timer_overhead_samples\":{}",
        compute.len() + timer_overhead.len() + messages.len(),
        compute.len(),
        timer_overhead.len(),
    )?;
    if !messages.is_empty() {
        write!(writer, ",\"message_roundtrip_samples\":{}", messages.len())?;
    }
    write!(writer, ",\"clocks_after\":")?;
    clocks_after.write(&mut writer)?;
    writeln!(writer, "}}")?;
    Ok(())
}

fn write_samples(
    mut writer: impl Write,
    benchmark: &str,
    iterations: u32,
    samples: &[Sample],
) -> fmt::Result {
    for (index, sample) in samples.iter().enumerate() {
        // Runs last seconds, far below the system timer's seven-year wrap.
        // Multiplying these bounded microsecond durations by 1000 fits in u64.
        let elapsed_ns = sample.elapsed.as_micros() * 1000;
        writeln!(
            writer,
            "{{\"protocol\":2,\"event\":\"sample\",\"benchmark\":\"{benchmark}\",\"index\":{index},\"iterations\":{iterations},\"elapsed_ns\":{elapsed_ns},\"checksum\":{}}}",
            sample.checksum,
        )?;
    }
    Ok(())
}
