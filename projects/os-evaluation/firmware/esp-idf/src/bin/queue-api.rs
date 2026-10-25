//! Matched Rust std / native FreeRTOS queue paths on the pinned ESP32-C6 SDK.
//!
//! See docs/experiments/native-queue-comparison.md for scope and admission.

#[path = "queue_api/backend.rs"]
mod backend;
#[path = "queue_api/timer.rs"]
mod timer;

use std::hint::black_box;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use timer::{Configuration, TICK_HZ, Tick, legacy_micros};

const SAMPLE_COUNT: usize = 1_000;
const OBSERVER_COUNT: usize = 1_000;
const WARMUP_ITERATIONS: u32 = 8;
const REPLY_MASK: u32 = 0xa5a5_5a5a;
const STOP: u32 = u32::MAX;
const WORKER_STACK_BYTES: usize = 8_192;
const MAIN_STACK_BYTES: usize = 32_768;

#[derive(Clone, Copy, Debug, Default)]
struct Sample {
    start_tick: u64,
    end_tick: u64,
    checksum: u32,
}

#[derive(Debug)]
struct WorkerState {
    phase: AtomicU32,
    failed: AtomicBool,
    exit_authorized: AtomicBool,
}

#[derive(Clone, Copy, Debug)]
struct WorkerIdentity {
    handle: usize,
    priority: u32,
}

#[derive(Clone, Copy, Debug)]
struct Qualification {
    monotonic_first_tick: u64,
    monotonic_last_tick: u64,
    blocked_start_tick: u64,
    blocked_end_tick: u64,
    legacy_before_us: u64,
    legacy_after_us: u64,
    wfi_start_tick: u64,
    wfi_end_tick: u64,
    wfi_rtos_tick_start: u32,
    wfi_rtos_tick_end: u32,
    wfi_calls: u32,
}

fn current_task_priority() -> u32 {
    // SAFETY: A null handle requests the current task in initialized FreeRTOS
    // task context. The generated binding retains no pointer or callback.
    unsafe { esp_idf_sys::uxTaskPriorityGet(std::ptr::null_mut()) }
}

fn worker_identity() -> WorkerIdentity {
    // SAFETY: Both generated functions are no-argument/current-task queries
    // after SDK initialization. The handle is exposed as an integer solely for
    // transfer; it is used only while the guarded Rust worker remains alive.
    let handle = unsafe { esp_idf_sys::xTaskGetCurrentTaskHandle() } as usize;
    WorkerIdentity {
        handle,
        priority: current_task_priority(),
    }
}

fn worker_loop(
    requests: backend::Receiver,
    replies: backend::Sender,
    state: &WorkerState,
) -> io::Result<()> {
    let mut phase = 0_u32;
    loop {
        // This is the only operation before the intended receive: a Release
        // phase store. On error, failed=true is published before parking. Main
        // checks phase/state/failure together and rejects any alternative path.
        // The higher-priority worker's NEXT store and re-blocking can occur
        // before main timestamps a reply: that matched instrumentation remains
        // part of each complete measured API path.
        state.phase.store(phase, Ordering::Release);
        let request = requests.recv()?;
        if request == STOP {
            return Ok(());
        }
        replies.send(request ^ REPLY_MASK)?;
        phase += 1;
    }
}

/// Proves a sequence-specific worker receive phase and an actual blocked state.
///
/// The task stays alive on errors until main explicitly authorizes shutdown;
/// successful exit requires the sentinel, which main sends after its last query.
fn wait_ready(
    worker: WorkerIdentity,
    state: &WorkerState,
    phase: u32,
    requests: &backend::Sender,
    replies: &backend::Receiver,
) -> io::Result<()> {
    let started = Tick::now();
    loop {
        if state.failed.load(Ordering::Acquire) {
            return Err(io::Error::other(format!(
                "worker failed before receive readiness for phase {phase}"
            )));
        }
        if state.phase.load(Ordering::Acquire) == phase {
            // SAFETY: This exact handle belongs to the live worker. Before the
            // only successful exit, main sends STOP after all readiness queries.
            // Error exit is guarded by exit_authorized, still false here. The
            // exposed pointer provenance comes from the SDK handle cast above.
            let task_state =
                unsafe { esp_idf_sys::eTaskGetState(worker.handle as esp_idf_sys::TaskHandle_t) };
            if task_state == esp_idf_sys::eTaskState_eBlocked
                && backend::queues_empty(requests, replies)?
                && state.phase.load(Ordering::Acquire) == phase
                && !state.failed.load(Ordering::Acquire)
            {
                return Ok(());
            }
        }
        if started.elapsed(Tick::now())? > TICK_HZ / 5 {
            return Err(io::Error::other(format!(
                "worker receive readiness exceeded 200 ms: expected phase {phase}, observed phase {}",
                state.phase.load(Ordering::Acquire)
            )));
        }
        std::thread::yield_now();
    }
}

fn rtos_tick() -> u32 {
    // SAFETY: Read-only task-context tick query, after scheduler startup.
    unsafe { esp_idf_sys::xTaskGetTickCount() }
}

fn debugger_attached() -> bool {
    // SAFETY: The generated RISC-V SDK function is a read-only debugger-status
    // query, callable in task context. No debugger is permitted during trials.
    unsafe { esp_idf_sys::rv_utils_dbgr_is_attached() }
}

fn qualify_timer() -> io::Result<Qualification> {
    let first = Tick::now();
    let mut last = first;
    for _ in 1..1_000 {
        let now = Tick::now();
        last.elapsed(now)?;
        last = now;
    }
    if first.elapsed(last)? == 0 {
        return Err(io::Error::other(
            "SYSTIMER did not progress during ordered reads",
        ));
    }
    let legacy_before_us = legacy_micros()?;
    let blocked_start = Tick::now();
    std::thread::sleep(Duration::from_millis(20));
    let blocked_end = Tick::now();
    let legacy_after_us = legacy_micros()?;
    let blocked_ticks = blocked_start.elapsed(blocked_end)?;
    let legacy_ticks = legacy_after_us
        .checked_sub(legacy_before_us)
        .and_then(|span| span.checked_mul(16))
        .ok_or_else(|| io::Error::other("legacy timer bracket overflow/backstep"))?;
    if !(TICK_HZ / 100..=TICK_HZ / 2).contains(&blocked_ticks)
        || blocked_ticks > legacy_ticks.saturating_add(32)
        || legacy_ticks.saturating_sub(blocked_ticks) > 16_000
    {
        return Err(io::Error::other(
            "blocked timer continuity/legacy scale check failed",
        ));
    }
    if debugger_attached() {
        return Err(io::Error::other(
            "debugger attached before WFI qualification",
        ));
    }
    let wfi_start = Tick::now();
    let wfi_rtos_tick_start = rtos_tick();
    let mut wfi_calls = 0_u32;
    let wfi_rtos_tick_end = loop {
        // SAFETY: This calls the existing SDK's task-context wait function with
        // interrupts enabled by normal IDF startup. Worker is blocked, no app
        // lock/critical section is held, PM is disabled and no debugger is
        // attached. The selected no-debugger RISC-V path executes SDK WFI;
        // the active 100 Hz FreeRTOS tick provides wake interrupts. This does
        // not install an idle hook, change timer clocks or enter light sleep.
        unsafe { esp_idf_sys::esp_cpu_wait_for_intr() };
        wfi_calls += 1;
        let tick = rtos_tick();
        if tick.wrapping_sub(wfi_rtos_tick_start) >= 3 {
            break tick;
        }
        if wfi_calls >= 10_000 || wfi_start.elapsed(Tick::now())? > TICK_HZ / 2 {
            return Err(io::Error::other(
                "WFI qualification exceeded bounded progress",
            ));
        }
    };
    let wfi_end = Tick::now();
    let elapsed_rtos_ticks = wfi_rtos_tick_end.wrapping_sub(wfi_rtos_tick_start);
    let elapsed_raw_ticks = wfi_start.elapsed(wfi_end)?;
    if debugger_attached()
        || !(3..=50).contains(&elapsed_rtos_ticks)
        || elapsed_raw_ticks < u64::from(elapsed_rtos_ticks - 1) * (TICK_HZ / 100)
        || elapsed_raw_ticks > u64::from(elapsed_rtos_ticks + 1) * (TICK_HZ / 100)
    {
        return Err(io::Error::other(
            "WFI counter/RTOS tick continuity check failed",
        ));
    }
    Ok(Qualification {
        monotonic_first_tick: first.0,
        monotonic_last_tick: last.0,
        blocked_start_tick: blocked_start.0,
        blocked_end_tick: blocked_end.0,
        legacy_before_us,
        legacy_after_us,
        wfi_start_tick: wfi_start.0,
        wfi_end_tick: wfi_end.0,
        wfi_rtos_tick_start,
        wfi_rtos_tick_end,
        wfi_calls,
    })
}

fn prefix(output: &mut impl Write, event: &str, boot_nonce: u64) -> io::Result<()> {
    write!(
        output,
        "{{\"protocol\":3,\"event\":\"{event}\",\"boot_nonce\":{boot_nonce},\"backend\":\"{}\"",
        backend::NAME
    )
}

fn run() -> io::Result<()> {
    // One opaque startup nonce identifies records, not a cryptographic identity
    // or proof of a host-requested reset. Radio-off esp_random is not assumed
    // to provide cryptographic entropy. It is generated once, outside timing.
    // SAFETY: SDK RNG no-argument reads after system startup retain no pointers.
    let boot_nonce = unsafe {
        (u64::from(esp_idf_sys::esp_random()) << 32) | u64::from(esp_idf_sys::esp_random())
    };
    backend::qualify()?;
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(SAMPLE_COUNT + OBSERVER_COUNT)
        .map_err(|error| io::Error::other(format!("sample storage allocation: {error}")))?;
    samples.resize(SAMPLE_COUNT + OBSERVER_COUNT, Sample::default());
    let sample_buffer_bytes = std::mem::size_of_val(samples.as_slice());
    let (request_tx, request_rx) = backend::channel()?;
    let (reply_tx, reply_rx) = backend::channel()?;
    let (identity_tx, identity_rx) = std::sync::mpsc::sync_channel(1);
    let state = Arc::new(WorkerState {
        phase: AtomicU32::new(0),
        failed: AtomicBool::new(false),
        exit_authorized: AtomicBool::new(false),
    });
    let worker_state = Arc::clone(&state);
    let main_priority = current_task_priority();
    let responder = std::thread::Builder::new()
        .name("queue-responder".into())
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || -> io::Result<()> {
            let result = identity_tx
                .send(worker_identity())
                .map_err(|_| io::Error::other("worker identity receiver disconnected"))
                .and_then(|()| worker_loop(request_rx, reply_tx, &worker_state));
            if result.is_err() {
                worker_state.failed.store(true, Ordering::Release);
                // Prevent task-handle invalidation while main can still query
                // readiness. Main authorizes exit before explicit join; park
                // is outside normal sampling and unpark cannot lose a token.
                while !worker_state.exit_authorized.load(Ordering::Acquire) {
                    std::thread::park();
                }
            }
            result
        })?;

    let mut captured_samples = 0_usize;
    let measurements = (|| -> io::Result<(WorkerIdentity, Configuration, Qualification)> {
        let worker = identity_rx
            .recv()
            .map_err(|_| io::Error::other("worker identity missing"))?;
        if worker.handle == 0 || main_priority != 1 || worker.priority != 5 {
            return Err(io::Error::other(
                "actual task priority/handle is off baseline",
            ));
        }
        std::thread::sleep(Duration::from_secs(2));
        for phase in 0..WARMUP_ITERATIONS {
            wait_ready(worker, &state, phase, &request_tx, &reply_rx)?;
            request_tx.send(phase)?;
            if reply_rx.recv()? != phase ^ REPLY_MASK {
                return Err(io::Error::other("warmup reply mismatch"));
            }
        }
        // Additional admission check is separate from the counted 1008 trial
        // readiness checks. Both APIs use the identical worker phase protocol.
        wait_ready(worker, &state, WARMUP_ITERATIONS, &request_tx, &reply_rx)?;
        let config_before = Configuration::read()?;
        let qualification = qualify_timer()?;
        for (index, sample) in samples[..OBSERVER_COUNT].iter_mut().enumerate() {
            let start = Tick::now();
            black_box(());
            let end = Tick::now();
            *sample = Sample {
                start_tick: start.0,
                end_tick: end.0,
                checksum: 0,
            };
            captured_samples += 1;
            start.elapsed(end).map_err(|error| {
                io::Error::other(format!(
                    "observer {index}, start={}, end={}: {error}",
                    start.0, end.0
                ))
            })?;
        }
        for (index, sample) in samples[OBSERVER_COUNT..].iter_mut().enumerate() {
            let sequence =
                u32::try_from(index).map_err(|_| io::Error::other("sample index exceeds u32"))?;
            wait_ready(
                worker,
                &state,
                sequence + WARMUP_ITERATIONS,
                &request_tx,
                &reply_rx,
            )?;
            let start = Tick::now();
            request_tx.send(sequence)?;
            let checksum = reply_rx.recv()?;
            let end = Tick::now();
            *sample = Sample {
                start_tick: start.0,
                end_tick: end.0,
                checksum,
            };
            captured_samples += 1;
            let elapsed = start.elapsed(end).map_err(|error| {
                io::Error::other(format!(
                    "roundtrip {index}, start={}, end={}: {error}",
                    start.0, end.0
                ))
            })?;
            if elapsed > TICK_HZ || checksum != sequence ^ REPLY_MASK {
                return Err(io::Error::other(format!(
                    "roundtrip {index}: start={}, end={}, elapsed={elapsed} ticks, checksum={checksum}, expected={}; duration limit={TICK_HZ}",
                    start.0,
                    end.0,
                    sequence ^ REPLY_MASK
                )));
            }
        }
        Ok((worker, config_before, qualification))
    })();

    // Every successful sample has ended before sentinel, endpoint destruction
    // or worker join. On ordinary errors release a parked failed worker, or
    // drain a pending reply and request normal stop. A corrupt/stuck kernel
    // can still need the collector's timeout/reset; it is never a passed run.
    state.exit_authorized.store(true, Ordering::Release);
    responder.thread().unpark();
    let stop_result = if state.failed.load(Ordering::Acquire) {
        Ok(())
    } else {
        reply_rx.try_recv().and_then(|_| request_tx.send(STOP))
    };
    drop(request_tx);
    drop(reply_rx);
    let worker_result = responder
        .join()
        .map_err(|_| io::Error::other("queue worker panicked"));
    let outcome = measurements.and_then(|(worker, config_before, qualification)| {
        stop_result?;
        worker_result??;
        let config_after = Configuration::read()?;
        if config_after != config_before {
            return Err(io::Error::other(
                "clock/timer configuration changed during run",
            ));
        }
        Ok((worker, config_before, qualification, config_after))
    });
    if let Err(error) = &outcome {
        // Preserve completed raw samples from a failed measurement as clearly
        // labelled diagnostics, without a hello or successful complete record.
        // A transport/panic/stuck-kernel failure can still interrupt this dump.
        eprintln!(
            "queue failure: boot_nonce={boot_nonce}, backend={}, captured_samples={captured_samples}: {error}",
            backend::NAME
        );
        for (offset, sample) in samples[..captured_samples].iter().enumerate() {
            let (benchmark, index) = if offset < OBSERVER_COUNT {
                ("timer-overhead", offset)
            } else {
                ("message-roundtrip", offset - OBSERVER_COUNT)
            };
            eprintln!(
                "queue partial: boot_nonce={boot_nonce}, benchmark={benchmark}, index={index}, start_tick={}, end_tick={}, checksum={}",
                sample.start_tick, sample.end_tick, sample.checksum
            );
        }
    }
    let (worker, config_before, qualification, config_after) = outcome?;

    let stdout = io::stdout();
    let mut output = stdout.lock();
    prefix(&mut output, "hello", boot_nonce)?;
    write!(
        output,
        ",\"platform\":\"esp-idf-freertos\",\"tick_hz\":16000000,\"tick_width_bits\":52,\"sample_count\":{SAMPLE_COUNT},\"observer_count\":{OBSERVER_COUNT},\"warmup_iterations\":{WARMUP_ITERATIONS},\"queue_capacity\":1,\"message_bytes\":4,\"main_priority\":{main_priority},\"worker_priority\":{},\"main_stack_bytes\":{MAIN_STACK_BYTES},\"worker_stack_bytes\":{WORKER_STACK_BYTES},\"rtos_tick_hz\":100,\"sample_buffer_bytes\":{sample_buffer_bytes},\"sample_storage\":\"preallocated-heap\",\"config_before\":",
        worker.priority
    )?;
    config_before.write_json(&mut output)?;
    writeln!(output, "}}")?;
    for (offset, sample) in samples.iter().enumerate() {
        let (benchmark, index, phase_sequence, blocked) = if offset < OBSERVER_COUNT {
            ("timer-overhead", offset, 0, false)
        } else {
            let index = offset - OBSERVER_COUNT;
            (
                "message-roundtrip",
                index,
                index + WARMUP_ITERATIONS as usize,
                true,
            )
        };
        prefix(&mut output, "sample", boot_nonce)?;
        writeln!(
            output,
            ",\"benchmark\":\"{benchmark}\",\"index\":{index},\"start_tick\":{},\"end_tick\":{},\"checksum\":{},\"phase_sequence\":{phase_sequence},\"blocked\":{blocked}}}",
            sample.start_tick, sample.end_tick, sample.checksum
        )?;
    }
    prefix(&mut output, "qualification", boot_nonce)?;
    writeln!(
        output,
        ",\"monotonic_reads\":1000,\"backsteps\":0,\"monotonic_first_tick\":{},\"monotonic_last_tick\":{},\"blocked_start_tick\":{},\"blocked_end_tick\":{},\"legacy_before_us\":{},\"legacy_after_us\":{},\"blocked_wait_method\":\"std-thread-sleep\",\"debugger_attached\":false,\"wfi_verified\":true,\"wfi_start_tick\":{},\"wfi_end_tick\":{},\"wfi_rtos_tick_start\":{},\"wfi_rtos_tick_end\":{},\"wfi_calls\":{},\"readiness_checks\":1008,\"blocked_checks\":1008,\"queue_empty_checks\":{},\"empty_receive_pass\":true,\"full_send_pass\":true,\"cleanup_pass\":true}}",
        qualification.monotonic_first_tick,
        qualification.monotonic_last_tick,
        qualification.blocked_start_tick,
        qualification.blocked_end_tick,
        qualification.legacy_before_us,
        qualification.legacy_after_us,
        qualification.wfi_start_tick,
        qualification.wfi_end_tick,
        qualification.wfi_rtos_tick_start,
        qualification.wfi_rtos_tick_end,
        qualification.wfi_calls,
        if cfg!(feature = "native-queue") {
            1008
        } else {
            0
        },
    )?;
    prefix(&mut output, "complete", boot_nonce)?;
    write!(
        output,
        ",\"samples\":2000,\"message_roundtrip_samples\":{SAMPLE_COUNT},\"timer_overhead_samples\":{OBSERVER_COUNT},\"worker_joined\":true,\"backend_destroyed\":true,\"config_after\":"
    )?;
    config_after.write_json(&mut output)?;
    writeln!(output, "}}")?;
    output.flush()
}

fn main() {
    esp_idf_sys::link_patches();
    if let Err(error) = run() {
        // Failed/partial boots never emit a successful complete record. Keep
        // SDK panic/reset behavior and collector timeout; no watchdog changes.
        eprintln!("queue benchmark failed: {error}");
    }
}
