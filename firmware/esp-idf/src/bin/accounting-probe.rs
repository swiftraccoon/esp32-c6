//! Qualifies task-attributed wall time and heap indicators, not CPU utilization.
//!
//! Requires its isolated 64-bit statistics SDK configuration. See
//! docs/experiments/idf-accounting-qualification.md for the frozen controls.

#[path = "queue_api/timer.rs"]
#[allow(
    dead_code,
    reason = "This diagnostic reuses configuration/public-us helpers, not the raw-tick workload API."
)]
mod timer;

use std::hint::black_box;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

use esp_idf_sys as sys;

const TASK_CAPACITY: usize = 16;
const SNAPSHOT_COUNT: usize = 6;
const PHASE_US: u64 = 250_000;
const ALLOCATION_BYTES: usize = 16_384;
const CAPS: u32 = sys::MALLOC_CAP_8BIT;
const WORKER_STACK: usize = 8_192;
const MAIN_STACK: usize = 32_768;

#[derive(Clone, Copy, Debug, Default)]
struct Task {
    handle: u32,
    number: u32,
    state: u32,
    priority: u32,
    runtime_us: u64,
    stack_free_bytes: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Heap {
    free_bytes: usize,
    allocated_bytes: usize,
    largest_free_block: usize,
    minimum_free_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Snapshot {
    worker_phase: u32,
    bracket_start_us: u64,
    bracket_end_us: u64,
    tasks_start_us: u64,
    tasks_end_us: u64,
    heap_start_us: u64,
    heap_end_us: u64,
    total_runtime_us: u64,
    count: usize,
    tasks: [Task; TASK_CAPACITY],
    heap: Heap,
}

#[derive(Clone, Copy, Debug)]
struct Identity {
    handle: usize,
    priority: u32,
}

#[derive(Clone, Copy, Debug)]
struct BusyWork {
    start_us: u64,
    end_us: u64,
    iterations: u32,
    checksum: u32,
}

#[derive(Clone, Copy, Debug)]
enum Command {
    Prime,
    Busy,
    Stop,
}

#[derive(Clone, Copy, Debug)]
struct BlindSpot {
    start_us: i64,
    end_us: i64,
    runtime_before_us: u64,
    runtime_after_us: u64,
}

fn error(message: &str) -> io::Error {
    io::Error::other(message)
}

fn identity() -> Identity {
    // SAFETY: Initialized IDF task context. Null requests the current priority;
    // the handle is retained numerically, never dereferenced by Rust.
    unsafe {
        Identity {
            handle: sys::xTaskGetCurrentTaskHandle() as usize,
            priority: sys::uxTaskPriorityGet(std::ptr::null_mut()),
        }
    }
}

fn busy_work(target_us: u64) -> io::Result<BusyWork> {
    let start_us = timer::legacy_micros()?;
    let mut iterations = 0_u32;
    let mut checksum = 0_u32;
    loop {
        let value = black_box(bench_core::workload(
            black_box(bench_core::DEFAULT_SEED),
            black_box(bench_core::DEFAULT_ROUNDS),
        ));
        if value != bench_core::DEFAULT_CHECKSUM {
            return Err(error("busy workload checksum failed"));
        }
        checksum = checksum.wrapping_add(value ^ iterations);
        iterations += 1;
        let end_us = timer::legacy_micros()?;
        let elapsed = end_us
            .checked_sub(start_us)
            .ok_or_else(|| error("busy timer moved backwards"))?;
        if elapsed >= target_us {
            return Ok(BusyWork {
                start_us,
                end_us,
                iterations,
                checksum,
            });
        }
        if iterations >= 100_000 {
            return Err(error("busy interval exceeded iteration bound"));
        }
    }
}

fn park_failed_worker() -> ! {
    // Do not delete a task whose handle main might still query. A disconnected
    // setup/control channel is a failed boot recovered by host reset.
    loop {
        std::thread::park();
    }
}

fn worker(
    commands: Receiver<Command>,
    replies: SyncSender<io::Result<BusyWork>>,
    ready: SyncSender<Identity>,
    phase: &AtomicU32,
) {
    if ready.send(identity()).is_err() {
        park_failed_worker();
    }
    drop(ready);
    loop {
        match commands.recv() {
            Ok(Command::Prime) => {
                if replies.send(busy_work(0)).is_err() {
                    park_failed_worker();
                }
                phase.store(0, Ordering::Release);
            }
            Ok(Command::Busy) => {
                if replies.send(busy_work(PHASE_US)).is_err() {
                    park_failed_worker();
                }
                phase.store(1, Ordering::Release);
            }
            Ok(Command::Stop) => return,
            Err(_) => park_failed_worker(),
        }
    }
}

fn wait_blocked(worker: Identity, phase: &AtomicU32, expected: u32) -> io::Result<()> {
    let started = timer::legacy_micros()?;
    loop {
        if phase.load(Ordering::Acquire) == expected {
            // SAFETY: This handle belongs to the persistent worker. It only
            // exits on Stop, sent after the final query. Failures park it alive.
            let state = unsafe { sys::eTaskGetState(worker.handle as sys::TaskHandle_t) };
            if state == sys::eTaskState_eBlocked && phase.load(Ordering::Acquire) == expected {
                return Ok(());
            }
        }
        if timer::legacy_micros()?.saturating_sub(started) > 200_000 {
            return Err(error("worker did not reach the declared blocked phase"));
        }
        std::thread::yield_now();
    }
}

fn capacity_control() -> io::Result<()> {
    // SAFETY: TaskStatus_t is a C record of integers/pointers, for which zero is
    // a valid representation. Capacity zero permits no writes to this allocated
    // array; total points to a live aligned u64. The U64 pointer type is an ABI
    // compile check: this source cannot compile against the old u32 binding.
    let mut status: sys::TaskStatus_t = unsafe { std::mem::zeroed() };
    let mut total = u64::MAX;
    let count = unsafe { sys::uxTaskGetSystemState(&mut status, 0, &mut total) };
    if count != 0 || total != u64::MAX {
        return Err(error(
            "insufficient-capacity control did not reject cleanly",
        ));
    }
    Ok(())
}

fn open_interval_control() -> io::Result<BlindSpot> {
    // SAFETY: Both status records have valid zero representations. This short,
    // bounded scheduler-suspended section is in task context, with interrupts
    // enabled. It contains only read-only task/timer FFI and CPU work: no
    // formatting, allocation, blocking, fallible return or panic. Both stored
    // counter reads occur inside suspension. Resume is unconditional before
    // checking anything; its possible task switch is outside the bracket.
    let mut before: sys::TaskStatus_t = unsafe { std::mem::zeroed() };
    let mut after: sys::TaskStatus_t = unsafe { std::mem::zeroed() };
    let (start_us, end_us) = unsafe {
        sys::vTaskSuspendAll();
        sys::vTaskGetInfo(
            std::ptr::null_mut(),
            &mut before,
            0,
            sys::eTaskState_eRunning,
        );
        let start = sys::esp_timer_get_time();
        let mut end = start;
        for _ in 0..100_000 {
            end = sys::esp_timer_get_time();
            if end.saturating_sub(start) >= 10_000 {
                break;
            }
            black_box(bench_core::workload(black_box(1), black_box(16)));
        }
        sys::vTaskGetInfo(
            std::ptr::null_mut(),
            &mut after,
            0,
            sys::eTaskState_eRunning,
        );
        sys::xTaskResumeAll();
        (start, end)
    };
    let value = BlindSpot {
        start_us,
        end_us,
        runtime_before_us: before.ulRunTimeCounter,
        runtime_after_us: after.ulRunTimeCounter,
    };
    if !(10_000..=50_000).contains(&value.end_us.saturating_sub(value.start_us))
        || value.start_us < 0
        || value.runtime_before_us != value.runtime_after_us
    {
        return Err(error("current-task open-interval control failed"));
    }
    Ok(value)
}

fn snapshot(worker: Identity, phase: &AtomicU32, expected: u32) -> io::Result<Snapshot> {
    wait_blocked(worker, phase, expected)?;
    // SAFETY: Generated C records contain only integer/pointer fields, all
    // permitting zero. SDK writes at most TASK_CAPACITY live array elements.
    // The u64 total-pointer and u64 field assignments check the regenerated ABI.
    // We never dereference or preserve the borrowed pcTaskName/stack pointers.
    let mut raw: [sys::TaskStatus_t; TASK_CAPACITY] = unsafe { std::mem::zeroed() };
    let mut info: sys::multi_heap_info_t = unsafe { std::mem::zeroed() };
    let mut result = Snapshot {
        worker_phase: phase.load(Ordering::Acquire),
        ..Snapshot::default()
    };
    result.bracket_start_us = timer::legacy_micros()?;
    result.bracket_end_us = timer::legacy_micros()?;
    result.tasks_start_us = timer::legacy_micros()?;
    let count = unsafe {
        sys::uxTaskGetSystemState(
            raw.as_mut_ptr(),
            TASK_CAPACITY as u32,
            &mut result.total_runtime_us,
        )
    } as usize;
    result.tasks_end_us = timer::legacy_micros()?;
    if count == 0 || count > TASK_CAPACITY {
        return Err(error("task snapshot empty or insufficient capacity"));
    }
    result.count = count;
    for (destination, source) in result.tasks.iter_mut().zip(&raw[..count]) {
        *destination = Task {
            handle: source.xHandle as u32,
            number: source.xTaskNumber,
            state: source.eCurrentState,
            priority: source.uxCurrentPriority,
            runtime_us: source.ulRunTimeCounter,
            stack_free_bytes: source.usStackHighWaterMark,
        };
    }
    result.heap_start_us = timer::legacy_micros()?;
    // SAFETY: Matching capability mask, initialized allocator/task context and
    // a live writable aligned info record. No pointer is retained by the API.
    unsafe { sys::heap_caps_get_info(&mut info, CAPS) };
    result.heap_end_us = timer::legacy_micros()?;
    result.heap = Heap {
        free_bytes: info.total_free_bytes,
        allocated_bytes: info.total_allocated_bytes,
        largest_free_block: info.largest_free_block,
        minimum_free_bytes: info.minimum_free_bytes,
    };
    Ok(result)
}

fn allocation_control(
    worker: Identity,
    phase: &AtomicU32,
) -> io::Result<(Snapshot, Snapshot, u32)> {
    // SAFETY: Task-context SDK allocation, explicit byte-capable mask. Non-null
    // returns at least ALLOCATION_BYTES writable uniquely owned bytes. Every
    // access below is in range. Volatile merely ensures this control actually
    // touches/checks storage; it supplies no synchronization. No Rust reference
    // aliases the allocation. Free occurs exactly once even if the held snapshot
    // fails. No pointer/reference survives free; later collection is separate.
    let pointer = unsafe { sys::heap_caps_malloc(ALLOCATION_BYTES, CAPS) }.cast::<u8>();
    if pointer.is_null() {
        return Err(error("qualification allocation failed"));
    }
    let mut checksum = 0_u32;
    let mut valid = true;
    for index in 0..ALLOCATION_BYTES {
        let expected = (index % 251) as u8;
        unsafe { pointer.add(index).write_volatile(expected) };
    }
    for index in 0..ALLOCATION_BYTES {
        let actual = unsafe { pointer.add(index).read_volatile() };
        valid &= actual == (index % 251) as u8;
        checksum += u32::from(actual);
    }
    let held = snapshot(worker, phase, 1);
    unsafe { sys::heap_caps_free(pointer.cast()) };
    let freed = snapshot(worker, phase, 1);
    if !valid {
        return Err(error("allocation byte verification failed"));
    }
    Ok((held?, freed?, checksum))
}

impl Snapshot {
    fn write_json(self, mut output: impl Write, id: &str) -> io::Result<()> {
        write!(
            output,
            "{{\"event\":\"snapshot\",\"id\":\"{id}\",\"worker_phase\":{},\"bracket_start_us\":{},\"bracket_end_us\":{},\"tasks_start_us\":{},\"tasks_end_us\":{},\"heap_start_us\":{},\"heap_end_us\":{},\"total_runtime_us\":{},\"tasks\":[",
            self.worker_phase,
            self.bracket_start_us,
            self.bracket_end_us,
            self.tasks_start_us,
            self.tasks_end_us,
            self.heap_start_us,
            self.heap_end_us,
            self.total_runtime_us
        )?;
        for (index, task) in self.tasks[..self.count].iter().enumerate() {
            if index > 0 {
                write!(output, ",")?;
            }
            write!(
                output,
                "{{\"handle\":{},\"number\":{},\"state\":{},\"priority\":{},\"runtime_us\":{},\"stack_free_bytes\":{}}}",
                task.handle,
                task.number,
                task.state,
                task.priority,
                task.runtime_us,
                task.stack_free_bytes
            )?;
        }
        let heap = self.heap;
        writeln!(
            output,
            "],\"heap\":{{\"free_bytes\":{},\"allocated_bytes\":{},\"largest_free_block\":{},\"minimum_free_bytes\":{}}}}}",
            heap.free_bytes, heap.allocated_bytes, heap.largest_free_block, heap.minimum_free_bytes
        )
    }
}

fn run() -> io::Result<()> {
    let (commands, receiver) = sync_channel(1);
    let (sender, replies) = sync_channel(1);
    let (ready, identity_receiver) = sync_channel(1);
    let phase = Arc::new(AtomicU32::new(u32::MAX));
    let worker_phase = Arc::clone(&phase);
    let thread = std::thread::Builder::new()
        .name("acct-worker".into())
        .stack_size(WORKER_STACK)
        .spawn(move || worker(receiver, sender, ready, &worker_phase))?;
    let worker = identity_receiver
        .recv()
        .map_err(|_| error("worker identity channel failed"))?;
    let main = identity();
    // SAFETY: The IDF idle task is persistent for scheduler lifetime. Retain its
    // identity only as a number; Rust never dereferences the handle.
    let idle = unsafe { sys::xTaskGetIdleTaskHandle() } as usize;
    let collected = (|| {
        if main.priority != 1 || worker.priority != 5 {
            return Err(error("unexpected task priorities"));
        }
        commands
            .send(Command::Prime)
            .map_err(|_| error("prime send failed"))?;
        replies
            .recv()
            .map_err(|_| error("prime receive failed"))??;
        wait_blocked(worker, &phase, 0)?;
        std::thread::sleep(Duration::from_secs(2));
        let config_before = timer::Configuration::read()?;
        capacity_control()?;
        let blind = open_interval_control()?;
        let mut snapshots = [Snapshot::default(); SNAPSHOT_COUNT];
        snapshots[0] = snapshot(worker, &phase, 0)?;
        std::thread::sleep(Duration::from_millis(250));
        snapshots[1] = snapshot(worker, &phase, 0)?;
        commands
            .send(Command::Busy)
            .map_err(|_| error("busy send failed"))?;
        let busy = replies.recv().map_err(|_| error("busy receive failed"))??;
        snapshots[2] = snapshot(worker, &phase, 1)?;
        snapshots[3] = snapshot(worker, &phase, 1)?;
        let (held, freed, checksum) = allocation_control(worker, &phase)?;
        snapshots[4] = held;
        snapshots[5] = freed;
        let config_after = timer::Configuration::read()?;
        if config_before != config_after {
            return Err(error("clock/timer configuration changed"));
        }
        Ok((
            config_before,
            config_after,
            blind,
            snapshots,
            busy,
            checksum,
        ))
    })();
    // Explicit shutdown is outside observations. Stop is the only task exit;
    // there are no handle queries after it, including on ordinary errors.
    commands
        .send(Command::Stop)
        .map_err(|_| error("worker stop failed"))?;
    thread.join().map_err(|_| error("worker join failed"))?;
    let (before, after, blind, snapshots, busy, checksum) = collected?;
    let mut output = io::stdout().lock();
    write!(
        output,
        "{{\"event\":\"hello\",\"protocol\":4,\"experiment\":\"idf-accounting-qualification-v1\",\"counter_bits\":64,\"clock\":\"esp_timer_get_time\",\"task_capacity\":{TASK_CAPACITY},\"worker_handle\":{},\"sampler_handle\":{},\"idle_handle\":{idle},\"worker_priority\":{},\"sampler_priority\":{},\"worker_stack_bytes\":{WORKER_STACK},\"sampler_stack_bytes\":{MAIN_STACK},\"target_phase_us\":{PHASE_US},\"allocation_bytes\":{ALLOCATION_BYTES},\"allocation_caps\":{CAPS},\"snapshot_storage_bytes\":{},\"config_before\":",
        worker.handle,
        main.handle,
        worker.priority,
        main.priority,
        std::mem::size_of_val(&snapshots)
    )?;
    before.write_json(&mut output)?;
    writeln!(output, "}}")?;
    writeln!(
        output,
        "{{\"event\":\"capacity-control\",\"returned_count\":0,\"sentinel_unchanged\":true}}"
    )?;
    writeln!(
        output,
        "{{\"event\":\"open-interval-control\",\"start_us\":{},\"end_us\":{},\"runtime_before_us\":{},\"runtime_after_us\":{}}}",
        blind.start_us, blind.end_us, blind.runtime_before_us, blind.runtime_after_us
    )?;
    let ids = [
        "blocked-before",
        "blocked-after",
        "busy-after",
        "heap-control",
        "heap-held",
        "heap-freed",
    ];
    for (index, (snapshot, id)) in snapshots.into_iter().zip(ids).enumerate() {
        snapshot.write_json(&mut output, id)?;
        if index == 2 {
            writeln!(
                output,
                "{{\"event\":\"busy-work\",\"start_us\":{},\"end_us\":{},\"iterations\":{},\"checksum\":{}}}",
                busy.start_us, busy.end_us, busy.iterations, busy.checksum
            )?;
        }
    }
    writeln!(
        output,
        "{{\"event\":\"allocation\",\"bytes\":{ALLOCATION_BYTES},\"caps\":{CAPS},\"checksum\":{checksum}}}"
    )?;
    write!(
        output,
        "{{\"event\":\"complete\",\"snapshots\":{SNAPSHOT_COUNT},\"worker_joined\":true,\"config_after\":"
    )?;
    after.write_json(&mut output)?;
    writeln!(output, "}}")?;
    output.flush()
}

fn main() {
    sys::link_patches();
    if let Err(error) = run() {
        // No successful completion on errors. SDK panic/reset behavior remains;
        // the collector preserves partial output and its timeout/fatal result.
        eprintln!("accounting qualification failed: {error}");
    }
}
