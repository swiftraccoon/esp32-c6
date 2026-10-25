//! Finite offered-load qualification; no sustainable-capacity or OS score.

#[path = "load_probe/observer.rs"]
mod observer;
#[path = "load_probe/runtime.rs"]
mod runtime;
#[path = "queue_api/timer.rs"]
#[allow(
    dead_code,
    reason = "This diagnostic reuses configuration/public-us helpers, not the raw-tick API."
)]
mod timer;

use std::io::{self, Write};
use std::sync::atomic::Ordering;
use std::time::Duration;

use observer::{Identity, Snapshot};
use runtime::{Harness, PhaseResult, SPECS};

fn run() -> io::Result<()> {
    let coordinator = Identity::current();
    if coordinator.priority != 1 {
        return Err(io::Error::other("coordinator priority differs from one"));
    }
    let mut harness = Harness::new()?;
    let (timer_handle, timer_priority) = harness.timer_identity();
    std::thread::sleep(Duration::from_secs(2));
    let config_before = timer::Configuration::read()?;
    let mut snapshots = Vec::new();
    snapshots
        .try_reserve_exact(12)
        .map_err(|_| io::Error::other("snapshot reservation failed"))?;
    let mut phases = [PhaseResult::default(); 6];
    for spec in SPECS {
        harness.warmup(spec.index)?;
        snapshots.push(Snapshot::capture(
            spec.index,
            "before",
            harness.producer_phase.load(Ordering::Acquire),
            harness.consumer_phase.load(Ordering::Acquire),
        )?);
        let epoch = timer::legacy_micros()?
            .checked_add(20_000)
            .ok_or_else(|| io::Error::other("epoch overflow"))?;
        phases[spec.index as usize] = harness.run_phase(spec, epoch)?;
        snapshots.push(Snapshot::capture(
            spec.index,
            "after",
            harness.producer_phase.load(Ordering::Acquire),
            harness.consumer_phase.load(Ordering::Acquire),
        )?);
    }
    let config_after = timer::Configuration::read()?;
    if config_before != config_after {
        return Err(io::Error::other("clock/timer configuration changed"));
    }
    // Timer quiescence and accepted deletion precede producer exit. No task
    // handle or saved SDK name pointer is queried after successful joins.
    harness.finish()?;
    let enabled = cfg!(feature = "load-runtime-stats");
    let profile = if enabled {
        "runtime-stats-enabled"
    } else {
        "runtime-stats-disabled"
    };
    let counter_bits = if enabled { 64 } else { 32 };
    let counter_clock = if enabled {
        "\"esp_timer_get_time\""
    } else {
        "null"
    };
    let mut output = io::stdout().lock();
    write!(
        output,
        "{{\"event\":\"hello\",\"protocol\":5,\"experiment\":\"idf-load-qualification-v1\",\"profile\":\"{profile}\",\"runtime_stats_enabled\":{enabled},\"counter_bits\":{counter_bits},\"counter_clock\":{counter_clock},\"task_capacity\":16,\"coordinator_handle\":{},\"producer_handle\":{},\"consumer_handle\":{},\"timer_handle\":{timer_handle},\"coordinator_priority\":{},\"producer_priority\":{},\"consumer_priority\":{},\"timer_priority\":{timer_priority},\"coordinator_stack_bytes\":32768,\"producer_stack_bytes\":8192,\"consumer_stack_bytes\":8192,\"notification_index\":1,\"notification_entries\":2,\"queue_capacity\":8,\"record_count\":2016,\"record_storage_bytes\":48384,\"snapshot_count\":12,\"snapshot_storage_bytes\":{},\"warmups_per_phase\":8,\"config_before\":",
        coordinator.handle,
        harness.producer.handle,
        harness.consumer.handle,
        coordinator.priority,
        harness.producer.priority,
        harness.consumer.priority,
        12 * std::mem::size_of::<Snapshot>()
    )?;
    config_before.write_json(&mut output)?;
    writeln!(output, "}}")?;
    writeln!(
        output,
        "{{\"event\":\"queue-control\",\"capacity\":8,\"accepted\":8,\"full_rejected\":1,\"fifo_pass\":true,\"empty_pass\":true}}"
    )?;
    for spec in SPECS {
        let phase = phases[spec.index as usize];
        let cutoff = phase.epoch_us + spec.window_us();
        writeln!(
            output,
            "{{\"event\":\"phase\",\"index\":{},\"id\":\"{}\",\"period_us\":{},\"slots\":{},\"window_us\":{},\"rounds\":{},\"initial_delay_us\":{},\"base_index\":{},\"epoch_us\":{},\"cutoff_us\":{cutoff},\"deadline_us\":2000}}",
            spec.index,
            spec.id,
            spec.period_us,
            spec.slots,
            spec.window_us(),
            spec.rounds,
            spec.initial_delay_us,
            spec.base_index,
            phase.epoch_us
        )?;
        snapshots[2 * spec.index as usize].write_json(&mut output)?;
        for index in 0..spec.slots {
            let global = spec.base_index + index;
            let record = &harness.records[global as usize];
            // Acquire final status publishes producer fields and, for completed
            // work, the consumer's acquired producer fields plus result fields.
            let status = match record.status.load(Ordering::Acquire) {
                1 => "generator-missed",
                2 => "queue-full",
                4 => "completed",
                _ => return Err(io::Error::other("incomplete job record after drain")),
            };
            writeln!(
                output,
                "{{\"event\":\"sample\",\"phase\":{},\"index\":{index},\"global_index\":{global},\"status\":\"{status}\",\"attempt_offset_us\":{},\"start_offset_us\":{},\"finish_offset_us\":{},\"checksum\":{},\"queue_depth\":{}}}",
                spec.index,
                record.attempt.load(Ordering::Relaxed),
                record.start.load(Ordering::Relaxed),
                record.finish.load(Ordering::Relaxed),
                record.checksum.load(Ordering::Relaxed),
                record.queue_depth.load(Ordering::Relaxed)
            )?;
        }
        writeln!(
            output,
            "{{\"event\":\"phase-end\",\"index\":{},\"generation_done_us\":{},\"drain_done_us\":{},\"timer_stop_pass\":true,\"queue_empty\":true,\"notification_strays\":0,\"callback_count\":{},\"early_wakes\":{}}}",
            spec.index,
            phase.generation_done_us,
            phase.drain_done_us,
            phase.callback_count,
            phase.early_wakes
        )?;
        snapshots[2 * spec.index as usize + 1].write_json(&mut output)?;
    }
    write!(
        output,
        "{{\"event\":\"complete\",\"phases\":6,\"nominal_samples\":2016,\"producer_joined\":true,\"consumer_joined\":true,\"timer_closed\":true,\"config_after\":"
    )?;
    config_after.write_json(&mut output)?;
    writeln!(output, "}}")?;
    output.flush()
}

fn main() {
    esp_idf_sys::link_patches();
    if let Err(error) = run() {
        // Setup/control/drain failure has no successful completion. Some such
        // failures precede all JSON; raw startup/error evidence remains, not a
        // fabricated partial favorable result. Host timeout/reset is recovery.
        eprintln!("load qualification failed: {error}");
    }
}
