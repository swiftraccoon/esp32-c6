//! Qualification-only protocol 4 for task-attributed wall time and heap controls.
//!
//! Capture and replay share the same single-boot guard and record validator.
//! Gross endpoints and signed boundary residuals remain descriptive evidence;
//! they are not physical CPU utilization, ISR accounting, power or capacity.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::qualification::{Admission, CapturePurpose};
use crate::queue_protocol::Configuration;
use crate::{boot_text, compiler_version, digest, write_json};

pub(crate) const EXPERIMENT: &str = "idf-accounting-qualification-v1";
const SNAPSHOT_IDS: [&str; 6] = [
    "blocked-before",
    "blocked-after",
    "busy-after",
    "heap-control",
    "heap-held",
    "heap-freed",
];
const ALLOCATION_BYTES: u32 = 16_384;
const HEAP_TOLERANCE: i64 = 1_024;
const WORKLOAD_ANSWER: u32 = 0xa394_4d55;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case", deny_unknown_fields)]
enum Record {
    Hello(Hello),
    CapacityControl(CapacityControl),
    OpenIntervalControl(OpenIntervalControl),
    Snapshot(Snapshot),
    BusyWork(BusyWork),
    Allocation(Allocation),
    Complete(Complete),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Hello {
    protocol: u32,
    experiment: String,
    counter_bits: u32,
    clock: String,
    task_capacity: u32,
    worker_handle: u32,
    sampler_handle: u32,
    idle_handle: u32,
    worker_priority: u32,
    sampler_priority: u32,
    worker_stack_bytes: u32,
    sampler_stack_bytes: u32,
    target_phase_us: u64,
    allocation_bytes: u32,
    allocation_caps: u32,
    snapshot_storage_bytes: u32,
    config_before: Configuration,
}

impl Hello {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.protocol == 4 && self.experiment == EXPERIMENT,
            "wrong accounting protocol or experiment"
        );
        ensure!(
            self.counter_bits == 64 && self.clock == "esp_timer_get_time",
            "accounting requires 64-bit esp_timer wall-time counters"
        );
        ensure!(self.task_capacity == 16, "wrong accounting task capacity");
        let handles = [self.worker_handle, self.sampler_handle, self.idle_handle];
        ensure!(
            handles.iter().all(|&handle| handle != 0)
                && handles.into_iter().collect::<BTreeSet<_>>().len() == 3,
            "worker, sampler and idle handles must be distinct and nonzero"
        );
        ensure!(
            self.worker_priority == 5 && self.sampler_priority == 1,
            "wrong accounting task priorities"
        );
        ensure!(
            self.worker_stack_bytes == 8_192 && self.sampler_stack_bytes == 32_768,
            "wrong accounting stack reservations"
        );
        ensure!(
            self.target_phase_us == 250_000
                && self.allocation_bytes == ALLOCATION_BYTES
                && self.allocation_caps == 4,
            "wrong accounting phase or allocation contract"
        );
        ensure!(
            self.snapshot_storage_bytes > 0,
            "missing snapshot storage size"
        );
        self.config_before.validate()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CapacityControl {
    returned_count: u32,
    sentinel_unchanged: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OpenIntervalControl {
    start_us: u64,
    end_us: u64,
    runtime_before_us: u64,
    runtime_after_us: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Task {
    handle: u32,
    number: u32,
    state: u32,
    priority: u32,
    runtime_us: u64,
    stack_free_bytes: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Heap {
    free_bytes: u32,
    allocated_bytes: u32,
    largest_free_block: u32,
    minimum_free_bytes: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    id: String,
    worker_phase: u32,
    bracket_start_us: u64,
    bracket_end_us: u64,
    tasks_start_us: u64,
    tasks_end_us: u64,
    heap_start_us: u64,
    heap_end_us: u64,
    total_runtime_us: u64,
    tasks: Vec<Task>,
    heap: Heap,
}

impl Snapshot {
    fn task(&self, handle: u32) -> Result<&Task> {
        self.tasks
            .iter()
            .find(|task| task.handle == handle)
            .with_context(|| format!("snapshot {} lacks required task handle {handle}", self.id))
    }

    fn validate(&self, hello: &Hello, index: usize) -> Result<()> {
        ensure!(
            self.id == SNAPSHOT_IDS[index],
            "wrong accounting snapshot order"
        );
        ensure!(
            self.worker_phase == u32::from(index >= 2),
            "wrong accounting worker phase"
        );
        ensure!(
            self.bracket_start_us <= self.bracket_end_us
                && self.bracket_end_us <= self.tasks_start_us
                && self.tasks_start_us <= self.total_runtime_us
                && self.total_runtime_us <= self.tasks_end_us
                && self.tasks_end_us <= self.heap_start_us
                && self.heap_start_us <= self.heap_end_us,
            "unordered accounting observer endpoints"
        );
        ensure!(
            !self.tasks.is_empty() && self.tasks.len() <= hello.task_capacity as usize,
            "empty or over-capacity task snapshot"
        );
        let mut handles = BTreeSet::new();
        let mut numbers = BTreeSet::new();
        for task in &self.tasks {
            ensure!(
                task.handle != 0
                    && task.number != 0
                    && handles.insert(task.handle)
                    && numbers.insert(task.number),
                "zero or duplicate task identity"
            );
            ensure!(
                task.state <= 3,
                "deleted or invalid task is not a stable live task"
            );
            ensure!(
                task.runtime_us <= self.total_runtime_us,
                "task runtime exceeds the snapshot's total clock"
            );
        }
        let worker = self.task(hello.worker_handle)?;
        let sampler = self.task(hello.sampler_handle)?;
        let idle = self.task(hello.idle_handle)?;
        ensure!(
            worker.state == 2,
            "worker is not blocked at snapshot boundary"
        );
        ensure!(sampler.state == 0, "sampler is not running at its snapshot");
        ensure!(idle.priority == 0, "idle task priority differs from zero");
        ensure!(
            worker.priority == hello.worker_priority && sampler.priority == hello.sampler_priority,
            "snapshot task priority differs from contract"
        );
        ensure!(
            worker.stack_free_bytes <= hello.worker_stack_bytes
                && sampler.stack_free_bytes <= hello.sampler_stack_bytes,
            "stack-fill indicator exceeds reservation"
        );
        ensure!(
            self.heap.largest_free_block <= self.heap.free_bytes
                && self.heap.minimum_free_bytes <= self.heap.free_bytes,
            "heap largest block or lifetime minimum exceeds current free bytes"
        );
        Ok(())
    }

    fn follows(&self, earlier: &Self) -> Result<()> {
        ensure!(
            earlier.heap_end_us <= self.bracket_start_us
                && earlier.total_runtime_us < self.total_runtime_us,
            "nonadvancing or overlapping snapshot clocks"
        );
        ensure!(
            self.tasks.len() == earlier.tasks.len(),
            "task lifecycle changed snapshot set"
        );
        for previous in &earlier.tasks {
            let task = self.task(previous.handle)?;
            ensure!(
                task.number == previous.number,
                "task handle was reused with a changed task number"
            );
            ensure!(
                task.runtime_us >= previous.runtime_us,
                "task runtime counter decreased; wrap is not normalized"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BusyWork {
    start_us: u64,
    end_us: u64,
    iterations: u32,
    checksum: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Allocation {
    bytes: u32,
    caps: u32,
    checksum: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Complete {
    snapshots: u32,
    worker_joined: bool,
    config_after: Configuration,
}

#[derive(Debug, Default)]
struct Boot {
    hello: Option<Hello>,
    capacity: Option<CapacityControl>,
    open_interval: Option<OpenIntervalControl>,
    snapshots: Vec<Snapshot>,
    busy: Option<BusyWork>,
    allocation: Option<Allocation>,
    complete: Option<Complete>,
    next: usize,
    rejected: bool,
}

fn elapsed(start: u64, end: u64, label: &str) -> Result<u64> {
    end.checked_sub(start)
        .with_context(|| format!("{label} clock decreased"))
}

fn busy_checksum(iterations: u32) -> u32 {
    (0..iterations).fold(0_u32, |sum, index| {
        sum.wrapping_add(WORKLOAD_ANSWER ^ index)
    })
}

fn allocation_checksum() -> u32 {
    (0..ALLOCATION_BYTES).map(|index| index % 251).sum()
}

fn signed_difference(left: u64, right: u64, label: &str) -> Result<i64> {
    i64::try_from(i128::from(left) - i128::from(right))
        .with_context(|| format!("{label} exceeds signed summary range"))
}

impl Boot {
    /// Rejects permanently, including parse/order errors before a later hello.
    fn accept_line(&mut self, line: &[u8]) -> Result<bool> {
        ensure!(!self.rejected, "accounting boot was already rejected");
        let result = (|| {
            let line = line.trim_ascii();
            if !line.starts_with(b"{") {
                return Ok(false);
            }
            let record = serde_json::from_slice(line).context("malformed accounting record")?;
            self.accept(record)
        })();
        if result.is_err() {
            self.rejected = true;
        }
        result
    }

    fn accept(&mut self, record: Record) -> Result<bool> {
        match (self.next, record) {
            (0, Record::Hello(hello)) => {
                hello.validate()?;
                self.hello = Some(hello);
            }
            (1, Record::CapacityControl(control)) => {
                ensure!(
                    control.returned_count == 0 && control.sentinel_unchanged,
                    "insufficient-capacity control failed"
                );
                self.capacity = Some(control);
            }
            (2, Record::OpenIntervalControl(control)) => {
                let duration = elapsed(control.start_us, control.end_us, "open interval")?;
                ensure!(
                    (10_000..=50_000).contains(&duration)
                        && control.runtime_before_us == control.runtime_after_us
                        && control.runtime_before_us <= control.start_us,
                    "current-task open-interval control failed"
                );
                self.open_interval = Some(control);
            }
            (3..=5 | 7..=9, Record::Snapshot(snapshot)) => {
                let hello = self.hello.as_ref().context("snapshot preceded hello")?;
                snapshot.validate(hello, self.snapshots.len())?;
                if let Some(previous) = self.snapshots.last() {
                    snapshot.follows(previous)?;
                } else {
                    ensure!(
                        self.open_interval
                            .as_ref()
                            .context("missing open interval")?
                            .end_us
                            <= snapshot.bracket_start_us,
                        "snapshot preceded open-interval control completion"
                    );
                }
                self.snapshots.push(snapshot);
            }
            (6, Record::BusyWork(busy)) => {
                let duration = elapsed(busy.start_us, busy.end_us, "busy workload")?;
                ensure!(
                    (250_000..=500_000).contains(&duration)
                        && (1..=100_000).contains(&busy.iterations),
                    "busy workload has wrong duration or no bounded progress"
                );
                ensure!(
                    busy.checksum == busy_checksum(busy.iterations),
                    "wrong busy workload checksum"
                );
                self.busy = Some(busy);
            }
            (10, Record::Allocation(allocation)) => {
                ensure!(
                    allocation.bytes == ALLOCATION_BYTES
                        && allocation.caps == 4
                        && allocation.checksum == allocation_checksum(),
                    "wrong direct-allocation size, capabilities or byte checksum"
                );
                self.allocation = Some(allocation);
            }
            (11, Record::Complete(complete)) => {
                ensure!(
                    complete.snapshots == 6 && complete.worker_joined,
                    "incomplete accounting cleanup"
                );
                complete.config_after.validate()?;
                ensure!(
                    complete.config_after
                        == self.hello.as_ref().context("missing hello")?.config_before,
                    "accounting clock/timer configuration drift"
                );
                self.complete = Some(complete);
            }
            _ => bail!("unexpected, duplicate or out-of-order accounting record"),
        }
        self.next += 1;
        if self.next == 12 {
            self.report_fields()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn report_fields(&self) -> Result<Value> {
        ensure!(!self.rejected, "rejected accounting boot has no summary");
        ensure!(self.next == 12, "incomplete accounting boot has no summary");
        let hello = self.hello.as_ref().context("missing accounting hello")?;
        let busy = self.busy.as_ref().context("missing busy workload")?;
        ensure!(self.snapshots.len() == 6, "missing accounting snapshots");
        let blocked_before = &self.snapshots[0];
        let blocked_after = &self.snapshots[1];
        let busy_after = &self.snapshots[2];
        let blocked_delta = blocked_after.total_runtime_us - blocked_before.total_runtime_us;
        ensure!(
            (200_000..=500_000).contains(&blocked_delta)
                && blocked_after.task(hello.worker_handle)?.runtime_us
                    == blocked_before.task(hello.worker_handle)?.runtime_us,
            "blocked phase duration or zero-worker-runtime control failed"
        );
        ensure!(
            blocked_after.heap_end_us <= busy.start_us
                && busy.end_us <= busy_after.bracket_start_us,
            "busy workload lies outside its causal snapshot boundaries"
        );
        let busy_span = elapsed(busy.start_us, busy.end_us, "busy workload")?;
        let busy_delta = busy_after.task(hello.worker_handle)?.runtime_us
            - blocked_after.task(hello.worker_handle)?.runtime_us;
        ensure!(
            u128::from(busy_delta) * 100 >= u128::from(busy_span) * 80
                && u128::from(busy_delta) * 100 <= u128::from(busy_span) * 120,
            "busy worker runtime is outside the coarse 80-120 percent control"
        );
        let control = &self.snapshots[3].heap;
        let held = &self.snapshots[4].heap;
        let freed = &self.snapshots[5].heap;
        let control_drift = i64::from(control.free_bytes) - i64::from(busy_after.heap.free_bytes);
        let allocation_drop = control
            .free_bytes
            .checked_sub(held.free_bytes)
            .context("held allocation increased current free heap")?;
        let recovery_drift = i64::from(freed.free_bytes) - i64::from(control.free_bytes);
        ensure!(
            control_drift.abs() <= HEAP_TOLERANCE,
            "heap background control drift exceeds bound"
        );
        ensure!(
            (ALLOCATION_BYTES..=ALLOCATION_BYTES + HEAP_TOLERANCE as u32)
                .contains(&allocation_drop),
            "direct allocation free-heap drop is outside control bounds"
        );
        ensure!(
            recovery_drift.abs() <= HEAP_TOLERANCE,
            "heap did not recover within control bound"
        );
        let observations: Vec<_> = self
            .snapshots
            .iter()
            .map(|snapshot| {
                json!({
                    "id": snapshot.id,
                    "empty_timer_bracket_us": snapshot.bracket_end_us - snapshot.bracket_start_us,
                    "task_enumeration_us": snapshot.tasks_end_us - snapshot.tasks_start_us,
                    "heap_info_us": snapshot.heap_end_us - snapshot.heap_start_us
                })
            })
            .collect();
        Ok(json!({
            "hello": hello,
            "capacity_control": self.capacity,
            "open_interval_control": self.open_interval,
            "snapshots": self.snapshots,
            "busy_work": busy,
            "allocation": self.allocation,
            "config_after": self.complete.as_ref().context("missing completion")?.config_after,
            "qualification": {
                "blocked_phase": phase_summary(blocked_before, blocked_after, hello)?,
                "busy_phase": phase_summary(blocked_after, busy_after, hello)?,
                "observer_gross": observations,
                "heap_control_drift_bytes": control_drift,
                "heap_allocation_drop_bytes": allocation_drop,
                "heap_recovery_drift_bytes": recovery_drift,
                "limits": [
                    "Task counters attribute switch intervals including interrupts and possible WFI; not exclusive CPU time or physical busy/idle",
                    "Signed unattributed boundary residual includes current-task open intervals and noninstantaneous scans; it is not idle time",
                    "Gross observer costs are retained without subtraction; no instrumentation-on/off control",
                    "Allocator and stack-fill indicators do not establish global runtime peaks or fragmentation",
                    "Onboard nominal microsecond clock, not an external frequency calibration"
                ]
            }
        }))
    }
}

fn phase_summary(before: &Snapshot, after: &Snapshot, hello: &Hello) -> Result<Value> {
    let total = after.total_runtime_us - before.total_runtime_us;
    let mut sum = 0_u64;
    let mut deltas = BTreeMap::new();
    for task in &before.tasks {
        let delta = after.task(task.handle)?.runtime_us - task.runtime_us;
        sum = sum
            .checked_add(delta)
            .context("summed task runtime overflow")?;
        deltas.insert(
            task.handle,
            json!({"number": task.number, "runtime_us": delta}),
        );
    }
    Ok(json!({
        "total_elapsed_us": total,
        "worker_attributed_us": after.task(hello.worker_handle)?.runtime_us - before.task(hello.worker_handle)?.runtime_us,
        "idle_task_attributed_us": after.task(hello.idle_handle)?.runtime_us - before.task(hello.idle_handle)?.runtime_us,
        "task_deltas": deltas,
        "summed_task_runtime_us": sum,
        "unattributed_boundary_residual_us": signed_difference(total, sum, "boundary residual")?
    }))
}

/// Replays full raw input; rejected/incomplete records cannot yield a summary.
pub(crate) fn validate_bytes(bytes: &[u8]) -> Result<Value> {
    validate(io::Cursor::new(bytes))
}

pub(crate) fn validate_file(path: &Path) -> Result<Value> {
    validate(File::open(path)?)
}

fn validate(input: impl Read) -> Result<Value> {
    let mut boot = Boot::default();
    boot_text::replay(input, |line| boot.accept_line(line))?;
    boot.report_fields()
}

#[derive(Debug)]
pub(crate) struct Capture<'a> {
    pub(crate) purpose: CapturePurpose,
    pub(crate) port: &'a str,
    pub(crate) output: &'a Path,
    pub(crate) elf: &'a Path,
    pub(crate) provenance: &'a Path,
    pub(crate) boot_id: &'a str,
    pub(crate) run_order: u32,
    pub(crate) timeout: Duration,
}

/// Requests one warm reset for diagnostic qualification, never a comparison.
///
/// Preflight rejection changes no device/output. Started attempts preserve raw
/// input and are marked failed when the final report can be written. An earlier
/// output directory is never overwritten.
pub(crate) fn capture(config: Capture<'_>) -> Result<()> {
    ensure!(
        config.purpose == CapturePurpose::Qualification,
        "accounting capture is qualification only; comparison is unsupported"
    );
    ensure!(
        !config.timeout.is_zero(),
        "capture timeout must be positive"
    );
    ensure!(
        !config.output.exists(),
        "result directory already exists; refusing overwrite"
    );
    let firmware = fs::read(config.elf).context("reading accounting firmware ELF")?;
    let provenance_bytes = fs::read(config.provenance).context("reading accounting provenance")?;
    let admission = Admission {
        purpose: CapturePurpose::Qualification,
        elf_sha256: digest(&firmware),
        provenance_sha256: digest(&provenance_bytes),
        method: None,
    };
    let provenance: Value = serde_json::from_slice(&provenance_bytes)?;
    ensure!(provenance.is_object(), "build provenance must be an object");
    let collector = fs::read(std::env::current_exe()?)?;
    let compiler = compiler_version()?;
    if let Some(parent) = config.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(config.output)?;
    fs::write(config.output.join("firmware.elf"), &firmware)?;
    fs::write(config.output.join("provenance.json"), &provenance_bytes)?;
    let mut report = json!({
        "schema_version": 4, "admission": admission, "status": "running",
        "platform": "esp-idf-freertos", "experiment": EXPERIMENT,
        "timestamp_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        "independent_boot_id": config.boot_id, "run_order_index": config.run_order,
        "reset_kind": "RTS warm-reset request; DTR deasserted",
        "serial": {"baud": 115200, "data_bits": 8, "parity": "none", "stop_bits": 1},
        "firmware_sha256": digest(&firmware), "collector_version": env!("CARGO_PKG_VERSION"),
        "collector_executable_sha256": digest(&collector), "host_rustc_at_capture": compiler,
        "provenance": provenance, "hello": null, "qualification": null,
        "benchmarks": null, "failure": null,
        "limitations": [
            "Qualification-only diagnostic; no comparative estimator or OS score",
            "Warm reset does not establish statistical independence or authenticate flashed firmware",
            "64-bit task-attributed wall microseconds include interrupts and possible WFI, not physical CPU utilization",
            "No external timing, power, capacity, runtime-peak or instrumentation-cost qualification",
            "Capture ends after completion and its received chunk; later diagnostics are unobserved",
            "host_rustc_at_capture observes PATH, not the compiler used to build this executable"
        ]
    });
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    let mut raw = File::create(config.output.join("serial.raw"))?;
    let mut records = File::create(config.output.join("records.jsonl"))?;
    let mut boot = Boot::default();
    let result = (|| {
        // Bind both the saved admission and original input paths before serial
        // access. Changed inputs fail this attempt; no device is opened.
        admission.check_inputs(&fs::read(config.elf)?, &fs::read(config.provenance)?)?;
        let mut port = serialport::new(config.port, 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .context("opening accounting serial port")?;
        port.write_data_terminal_ready(false)?;
        port.write_request_to_send(true)?;
        std::thread::sleep(Duration::from_millis(100));
        port.clear(serialport::ClearBuffer::Input)?;
        port.write_request_to_send(false)?;
        capture_stream(&mut port, &mut raw, &mut records, &mut boot, config.timeout)
    })();
    raw.flush()?;
    records.flush()?;
    report["hello"] = serde_json::to_value(&boot.hello)?;
    match &result {
        Ok(()) => {
            for (key, value) in boot
                .report_fields()?
                .as_object()
                .context("accounting report must be an object")?
            {
                report[key] = value.clone();
            }
            report["status"] = json!("pass");
        }
        Err(error) => {
            report["status"] = json!("failed");
            report["failure"] = json!(format!("{error:#}"));
        }
    }
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    result?;
    println!(
        "Validated accounting qualification boot {}: {}",
        config.boot_id,
        config.output.display()
    );
    Ok(())
}

/// Actual connected-port reader path, also exercised by fake transports.
fn capture_stream(
    input: &mut impl Read,
    raw: &mut impl Write,
    records: &mut impl Write,
    boot: &mut Boot,
    timeout: Duration,
) -> Result<()> {
    let result = boot_text::capture(input, raw, timeout, |line| {
        let complete = boot.accept_line(line)?;
        if line.trim_ascii().starts_with(b"{") {
            records.write_all(line.trim_ascii())?;
            records.write_all(b"\n")?;
        }
        Ok(complete)
    });
    if result.is_err() {
        boot.rejected = true;
    }
    result
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use clap::Parser;

    fn configuration() -> Value {
        json!({
            "clocks": {"source": "register-readback", "cpu_mhz": 160,
                "flash_source_mhz": 80, "flash_mhz": 80, "flash_divider": 1,
                "registers": {"pcr_sysclk_conf": 0x0001_0200_u32,
                    "pcr_cpu_freq_conf": 0, "pcr_mspi_clk_conf": 0x500,
                    "spi0_clock": 0x8000_0000_u32}},
            "timer": {"source": "xtal", "enabled": true, "unit0_enabled": true,
                "unit0_stalls_cpu": true, "systimer_conf": 0x5000_0000_u32,
                "pcr_systimer_conf": 1, "pcr_systimer_func_clk_conf": 1 << 22}
        })
    }

    /// Synthetic observations exercise host rules, never physical execution.
    pub(crate) fn transcript() -> Vec<Value> {
        let base = 5_000_000_000_u64;
        let mut records = vec![
            json!({"event": "hello", "protocol": 4, "experiment": EXPERIMENT,
                "counter_bits": 64, "clock": "esp_timer_get_time", "task_capacity": 16,
                "worker_handle": 1, "sampler_handle": 2, "idle_handle": 3,
                "worker_priority": 5, "sampler_priority": 1,
                "worker_stack_bytes": 8192, "sampler_stack_bytes": 32768,
                "target_phase_us": 250000, "allocation_bytes": ALLOCATION_BYTES,
                "allocation_caps": 4, "snapshot_storage_bytes": 8192,
                "config_before": configuration()}),
            json!({"event": "capacity-control", "returned_count": 0, "sentinel_unchanged": true}),
            json!({"event": "open-interval-control", "start_us": base + 10,
                "end_us": base + 10010, "runtime_before_us": 1000, "runtime_after_us": 1000}),
        ];
        for (index, offset) in [20_000, 270_000, 530_000, 531_000, 532_000, 533_000]
            .into_iter()
            .enumerate()
        {
            let start = base + offset;
            let busy_done = index >= 2;
            let free = if index == 4 { 283_600 } else { 300_000 };
            records.push(json!({"event": "snapshot", "id": SNAPSHOT_IDS[index],
                "worker_phase": u32::from(busy_done),
                "bracket_start_us": start, "bracket_end_us": start + 1,
                "tasks_start_us": start + 2, "tasks_end_us": start + 30,
                "heap_start_us": start + 31, "heap_end_us": start + 40,
                "total_runtime_us": start + 20,
                "tasks": [
                    {"handle": 1, "number": 10, "state": 2, "priority": 5,
                     "runtime_us": 4_500_000_000_u64 + if busy_done {250000} else {0}, "stack_free_bytes": 7000},
                    {"handle": 2, "number": 20, "state": 0, "priority": 1,
                     "runtime_us": 1000 + index as u64 * 100, "stack_free_bytes": 30000},
                    {"handle": 3, "number": 30, "state": 1, "priority": 0,
                     "runtime_us": 250_000_000_u64 + match index {0 => 0, 1 => 250000, _ => 259000}, "stack_free_bytes": 1000}
                ],
                "heap": {"free_bytes": free, "allocated_bytes": 380000 - free,
                    "largest_free_block": free - 1000, "minimum_free_bytes": 280000}
            }));
            if index == 2 {
                records.push(json!({"event": "busy-work", "start_us": base + 270100,
                    "end_us": base + 520100, "iterations": 1000,
                    "checksum": 4_216_364_460_u32}));
            }
        }
        records.push(json!({"event": "allocation", "bytes": ALLOCATION_BYTES,
            "caps": 4, "checksum": 2_041_721}));
        records.push(json!({"event": "complete", "snapshots": 6,
            "worker_joined": true, "config_after": configuration()}));
        records
    }

    pub(crate) fn raw(records: &[Value]) -> Vec<u8> {
        let mut bytes = b"ESP-ROM:esp32c6-20220919\nrst:0x1 (POWERON)\n".to_vec();
        for record in records {
            serde_json::to_writer(&mut bytes, record).unwrap();
            bytes.push(b'\n');
        }
        bytes
    }

    fn reject(records: &[Value], reason: &str) {
        let error = validate_bytes(&raw(records)).unwrap_err();
        assert!(format!("{error:#}").contains(reason), "{error:#}");
    }

    #[test]
    fn synthetic_positive_keeps_u64_counters_gross_observers_and_signed_residual() {
        let report = validate_bytes(&raw(&transcript())).unwrap();
        assert!(
            report["snapshots"][0]["tasks"][0]["runtime_us"]
                .as_u64()
                .unwrap()
                > u64::from(u32::MAX)
        );
        assert_eq!(
            report["qualification"]["blocked_phase"]["unattributed_boundary_residual_us"],
            -100
        );
        assert_eq!(
            report["qualification"]["busy_phase"]["worker_attributed_us"],
            250000
        );
        assert_eq!(
            report["qualification"]["observer_gross"][0]["task_enumeration_us"],
            28
        );
        assert_eq!(report["qualification"]["heap_allocation_drop_bytes"], 16400);
        assert!(report.get("benchmarks").is_none());
    }

    #[test]
    fn complete_typed_schema_rejects_unknown_missing_duplicate_or_wrong_record_fields() {
        let mut records = transcript();
        records[0]["counter_bits"] = json!(32);
        reject(&records, "64-bit");
        let mut records = transcript();
        records[0]["clock"] = json!("cycles");
        reject(&records, "64-bit");
        let mut records = transcript();
        records[0]["surprise"] = json!(true);
        reject(&records, "unknown field");
        let mut records = transcript();
        records[3]["tasks"][0]["surprise"] = json!(true);
        reject(&records, "unknown field");
        let mut records = transcript();
        records[3].as_object_mut().unwrap().remove("heap");
        reject(&records, "missing field");
        let mut records = transcript();
        records[3]["tasks"][0]["runtime_us"] = json!(2.5);
        reject(&records, "malformed accounting");
        let mut records = transcript();
        records.insert(1, records[0].clone());
        reject(&records, "out-of-order");
        let mut records = transcript();
        records.swap(6, 7);
        reject(&records, "out-of-order");
        let mut records = transcript();
        records.pop();
        reject(&records, "incomplete");
        let mut records = transcript();
        records.push(records.last().unwrap().clone());
        reject(&records, "out-of-order");
    }

    #[test]
    fn snapshots_require_complete_stable_distinct_live_task_identities() {
        for (path, value, reason) in [
            ("handle", 0, "zero or duplicate"),
            ("handle", 2, "zero or duplicate"),
            ("number", 20, "zero or duplicate"),
            ("number", 0, "zero or duplicate"),
            ("state", 4, "stable live task"),
            ("state", 5, "stable live task"),
        ] {
            let mut records = transcript();
            records[3]["tasks"][0][path] = json!(value);
            reject(&records, reason);
        }
        let mut records = transcript();
        records[3]["tasks"] = json!([]);
        reject(&records, "empty or over-capacity");
        let mut records = transcript();
        records[4]["tasks"].as_array_mut().unwrap().pop();
        reject(&records, "lacks required task");
        let mut records = transcript();
        records[4]["tasks"][0]["number"] = json!(11);
        reject(&records, "handle was reused");
        let mut records = transcript();
        records[4]["tasks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"handle": 4, "number": 40,
            "state": 2, "priority": 1, "runtime_us": 0, "stack_free_bytes": 1000}));
        reject(&records, "lifecycle changed");
        let mut records = transcript();
        records[3]["tasks"] = json!(
            (0..17)
                .map(|i| json!({"handle": i+1,"number": i+1,
            "state": 2,"priority": 1,"runtime_us": 0,"stack_free_bytes": 1000}))
                .collect::<Vec<_>>()
        );
        reject(&records, "over-capacity");
    }

    #[test]
    fn task_boundaries_and_phase_progress_are_checked() {
        for (record, field, value, reason) in [
            (3, "state", 1, "worker is not blocked"),
            (3, "priority", 4, "priority differs"),
            (3, "stack_free_bytes", 8193, "exceeds reservation"),
        ] {
            let mut records = transcript();
            records[record]["tasks"][0][field] = json!(value);
            reject(&records, reason);
        }
        let mut records = transcript();
        records[3]["tasks"][1]["state"] = json!(2);
        reject(&records, "sampler is not running");
        let mut records = transcript();
        records[3]["tasks"][2]["priority"] = json!(1);
        reject(&records, "idle task priority");
        let mut records = transcript();
        records[5]["worker_phase"] = json!(0);
        reject(&records, "wrong accounting worker phase");
        let mut records = transcript();
        records[4]["tasks"][0]["runtime_us"] = json!(4_500_000_001_u64);
        reject(&records, "zero-worker-runtime");
        let mut records = transcript();
        records[4]["tasks"][0]["runtime_us"] = json!(4_499_999_999_u64);
        reject(&records, "counter decreased");
        let mut records = transcript();
        records[5]["tasks"][0]["runtime_us"] = json!(4_500_100_000_u64);
        reject(&records, "80-120");
    }

    #[test]
    fn gross_endpoints_and_causal_busy_boundaries_cannot_be_reordered() {
        let mut records = transcript();
        records[3]["bracket_end_us"] = json!(5_000_019_999_u64);
        reject(&records, "unordered accounting observer");
        let mut records = transcript();
        records[3]["total_runtime_us"] = json!(5_000_020_100_u64);
        reject(&records, "unordered accounting observer");
        let mut records = transcript();
        for field in [
            "bracket_start_us",
            "bracket_end_us",
            "tasks_start_us",
            "tasks_end_us",
            "heap_start_us",
            "heap_end_us",
            "total_runtime_us",
        ] {
            records[4][field] = records[3][field].clone();
        }
        reject(&records, "nonadvancing or overlapping");
        let mut records = transcript();
        records[6]["start_us"] = json!(5_000_269_999_u64);
        records[6]["end_us"] = json!(5_000_519_999_u64);
        reject(&records, "causal snapshot");
        let mut records = transcript();
        records[11]["config_after"]["timer"]["systimer_conf"] = json!(0x5000_0001_u32);
        reject(&records, "configuration drift");
        let mut records = transcript();
        records[0]["config_before"]["clocks"]["cpu_mhz"] = json!(80);
        reject(&records, "readback");
    }

    #[test]
    fn positive_controls_require_expected_failures_and_checked_work() {
        for (record, field, value, reason) in [
            (1, "returned_count", json!(1), "capacity control failed"),
            (
                1,
                "sentinel_unchanged",
                json!(false),
                "capacity control failed",
            ),
            (
                2,
                "runtime_after_us",
                json!(1001),
                "open-interval control failed",
            ),
            (
                2,
                "end_us",
                json!(5_000_000_011_u64),
                "open-interval control failed",
            ),
            (6, "iterations", json!(0), "no bounded progress"),
            (6, "iterations", json!(100001), "no bounded progress"),
            (6, "checksum", json!(0), "wrong busy workload checksum"),
            (10, "caps", json!(8), "direct-allocation"),
            (10, "checksum", json!(0), "direct-allocation"),
            (11, "worker_joined", json!(false), "cleanup"),
            (11, "snapshots", json!(5), "cleanup"),
        ] {
            let mut records = transcript();
            records[record][field] = value;
            reject(&records, reason);
        }
    }

    #[test]
    fn heap_controls_reject_impossible_indicators_missing_drop_and_failed_recovery() {
        let mut records = transcript();
        records[3]["heap"]["largest_free_block"] = json!(300001);
        reject(&records, "heap largest block");
        let mut records = transcript();
        records[3]["heap"]["minimum_free_bytes"] = json!(300001);
        reject(&records, "lifetime minimum");
        let mut records = transcript();
        records[8]["heap"]["free_bytes"] = json!(300000);
        reject(&records, "free-heap drop");
        let mut records = transcript();
        records[9]["heap"]["free_bytes"] = json!(301025);
        reject(&records, "recover");
        let mut records = transcript();
        records[7]["heap"]["free_bytes"] = json!(301025);
        reject(&records, "background control drift");
    }

    #[test]
    fn impossible_absolute_counters_and_summary_overflow_are_rejected() {
        let mut records = transcript();
        records[3]["tasks"][0]["runtime_us"] = json!(u64::MAX);
        reject(&records, "exceeds the snapshot");
        let mut records = transcript();
        records[2]["runtime_before_us"] = json!(u64::MAX);
        records[2]["runtime_after_us"] = json!(u64::MAX);
        reject(&records, "open-interval control failed");

        for sum_overflow in [false, true] {
            let mut records = transcript();
            // Every individual counter remains <= its matching total clock;
            // arithmetic must still reject an overflowing gross sum/residual.
            let large_base = u64::MAX - 2_000_000;
            for record in &mut records {
                for field in [
                    "start_us",
                    "end_us",
                    "bracket_start_us",
                    "bracket_end_us",
                    "tasks_start_us",
                    "tasks_end_us",
                    "heap_start_us",
                    "heap_end_us",
                    "total_runtime_us",
                ] {
                    if let Some(old) = record[field].as_u64() {
                        record[field] = json!(large_base + (old - 5_000_000_000));
                    }
                }
            }
            records[3]["tasks"][1]["runtime_us"] = json!(0);
            if sum_overflow {
                records[3]["tasks"][2]["runtime_us"] = json!(0);
            }
            for index in [4, 5, 7, 8, 9] {
                let total = records[index]["total_runtime_us"].as_u64().unwrap();
                records[index]["tasks"][1]["runtime_us"] = json!(total - 1);
                if sum_overflow {
                    records[index]["tasks"][2]["runtime_us"] = json!(total - 1);
                }
            }
            reject(
                &records,
                if sum_overflow {
                    "summed task runtime overflow"
                } else {
                    "signed summary range"
                },
            );
        }
    }

    #[derive(Debug)]
    struct Fragmented<'a> {
        bytes: &'a [u8],
        width: usize,
    }

    impl Read for Fragmented<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = output.len().min(self.width).min(self.bytes.len());
            output[..count].copy_from_slice(&self.bytes[..count]);
            self.bytes = &self.bytes[count..];
            Ok(count)
        }
    }

    #[test]
    fn actual_capture_and_replay_guard_reject_fatal_before_first_json_and_revoke_summary() {
        let records = transcript();
        let valid = raw(&records);
        for fault in [
            "panic: attempted work failed",
            "Rebooting...",
            "ESP-ROM:esp32c6-20220919",
        ] {
            let bytes = [
                format!("ESP-ROM:esp32c6-20220919\nrst:0x1\n{fault}\n").into_bytes(),
                valid.splitn(3, |&b| b == b'\n').nth(2).unwrap().to_vec(),
            ]
            .concat();
            for width in [1, 7, 1024] {
                let mut boot = Boot::default();
                let mut raw_capture = Vec::new();
                let mut parsed = Vec::new();
                assert!(
                    capture_stream(
                        &mut Fragmented {
                            bytes: &bytes,
                            width
                        },
                        &mut raw_capture,
                        &mut parsed,
                        &mut boot,
                        Duration::from_secs(1)
                    )
                    .is_err()
                );
                assert!(boot.report_fields().is_err());
                assert!(bytes.starts_with(&raw_capture));
                assert!(String::from_utf8_lossy(&raw_capture).contains(fault));
                assert!(
                    boot.accept_line(serde_json::to_string(&records[0]).unwrap().as_bytes())
                        .is_err()
                );
                assert!(
                    validate(Fragmented {
                        bytes: &bytes,
                        width
                    })
                    .is_err()
                );
            }
        }
        let mut boot = Boot::default();
        let mut retained = Vec::new();
        capture_stream(
            &mut io::Cursor::new(&valid),
            &mut retained,
            &mut Vec::new(),
            &mut boot,
            Duration::from_secs(1),
        )
        .unwrap();
        assert_eq!(retained, valid);
        assert_eq!(
            boot.report_fields().unwrap(),
            validate_bytes(&valid).unwrap()
        );
        let duplicate = serde_json::to_vec(records.last().unwrap()).unwrap();
        assert!(boot.accept_line(&duplicate).is_err());
        assert!(boot.report_fields().is_err());
    }

    #[derive(Debug)]
    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new() -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "c6-accounting-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn comparison_is_rejected_before_files_output_or_port_and_purpose_is_explicit() {
        let fixture = Fixture::new();
        let output = fixture.0.join("new-output");
        let error = capture(Capture {
            purpose: CapturePurpose::Comparison,
            port: "/c6-accounting-test-port-does-not-exist",
            output: &output,
            elf: &fixture.0.join("absent.elf"),
            provenance: &fixture.0.join("absent.json"),
            boot_id: "fixture",
            run_order: 1,
            timeout: Duration::from_secs(1),
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains("qualification only"));
        assert!(!output.exists());
        assert!(
            crate::Cli::try_parse_from([
                "benchctl",
                "capture-accounting",
                "--port",
                "/unused",
                "--output",
                "unused",
                "--elf",
                "unused",
                "--provenance",
                "unused",
                "--boot-id",
                "fixture",
                "--run-order",
                "1"
            ])
            .is_err()
        );
    }

    #[test]
    fn failed_serial_open_retains_input_identities_and_no_favorable_summary() {
        let fixture = Fixture::new();
        let elf = fixture.0.join("input.elf");
        let provenance = fixture.0.join("input.json");
        fs::write(&elf, b"synthetic host-only artifact").unwrap();
        fs::write(&provenance, b"{\n  \"fixture\": true\n}\n").unwrap();
        let output = fixture.0.join("failed");
        let error = capture(Capture {
            purpose: CapturePurpose::Qualification,
            port: "/c6-accounting-test-port-does-not-exist",
            output: &output,
            elf: &elf,
            provenance: &provenance,
            boot_id: "fixture",
            run_order: 1,
            timeout: Duration::from_secs(1),
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains("opening accounting serial port"));
        let report: Value =
            serde_json::from_slice(&fs::read(output.join("run.json")).unwrap()).unwrap();
        assert_eq!(report["status"], "failed");
        assert_eq!(report["schema_version"], 4);
        assert_eq!(report["admission"]["purpose"], "qualification");
        assert_eq!(
            report["admission"]["elf_sha256"],
            digest(&fs::read(&elf).unwrap())
        );
        assert_eq!(
            report["admission"]["provenance_sha256"],
            digest(&fs::read(&provenance).unwrap())
        );
        assert_eq!(
            fs::read(output.join("provenance.json")).unwrap(),
            fs::read(provenance).unwrap()
        );
        assert!(report["qualification"].is_null());
        assert!(report["benchmarks"].is_null());
        assert!(output.join("serial.raw").exists());
    }

    #[test]
    fn archival_replay_preserves_protocol4_identities_outputs_and_failed_status() {
        let fixture = Fixture::new();
        let directory = fixture.0.join("results/qualification/boot-01");
        fs::create_dir_all(&directory).unwrap();
        let bytes = raw(&transcript());
        let replay = validate_bytes(&bytes).unwrap();
        let provenance = b"{\n  \"synthetic_host_fixture\": true\n}\n";
        let elf = b"synthetic host-only artifact";
        let mut report = replay.clone();
        report["schema_version"] = json!(4);
        report["status"] = json!("pass");
        report["experiment"] = json!(EXPERIMENT);
        report["collector_executable_sha256"] = json!(digest(b"synthetic collector"));
        report["firmware_sha256"] = json!(digest(elf));
        report["provenance"] = serde_json::from_slice(provenance).unwrap();
        report["admission"] = json!({"purpose": "qualification", "elf_sha256": digest(elf),
            "provenance_sha256": digest(provenance), "method": null});
        report["benchmarks"] = Value::Null;
        report["failure"] = Value::Null;
        fs::write(directory.join("serial.raw"), &bytes).unwrap();
        fs::write(directory.join("firmware.elf"), elf).unwrap();
        fs::write(directory.join("provenance.json"), provenance).unwrap();
        write_json(File::create(directory.join("run.json")).unwrap(), &report).unwrap();
        write_json(
            File::create(directory.join("replay.json")).unwrap(),
            &replay,
        )
        .unwrap();
        assert_eq!(
            crate::qualification::replay_archives(&fixture.0).unwrap()["successful_protocol_transcripts"],
            1
        );

        let mut changed = report.clone();
        changed["qualification"]["busy_phase"]["worker_attributed_us"] = json!(250001);
        write_json(File::create(directory.join("run.json")).unwrap(), &changed).unwrap();
        assert!(
            format!(
                "{:#}",
                crate::qualification::replay_archives(&fixture.0).unwrap_err()
            )
            .contains("differs from replay")
        );
        changed = report.clone();
        changed["benchmarks"] = json!({"unsupported_score": 1});
        write_json(File::create(directory.join("run.json")).unwrap(), &changed).unwrap();
        assert!(
            format!(
                "{:#}",
                crate::qualification::replay_archives(&fixture.0).unwrap_err()
            )
            .contains("qualification-only")
        );
        changed = report.clone();
        changed["admission"]["purpose"] = json!("comparison");
        write_json(File::create(directory.join("run.json")).unwrap(), &changed).unwrap();
        assert!(
            format!(
                "{:#}",
                crate::qualification::replay_archives(&fixture.0).unwrap_err()
            )
            .contains("qualification-only")
        );
        write_json(File::create(directory.join("run.json")).unwrap(), &report).unwrap();
        fs::write(directory.join("provenance.json"), b"{}").unwrap();
        assert!(
            format!(
                "{:#}",
                crate::qualification::replay_archives(&fixture.0).unwrap_err()
            )
            .contains("inputs changed")
        );
        fs::write(directory.join("provenance.json"), provenance).unwrap();

        changed = report.clone();
        changed["status"] = json!("failed");
        changed["failure"] = json!("synthetic host-only failure");
        changed["qualification"] = Value::Null;
        write_json(File::create(directory.join("run.json")).unwrap(), &changed).unwrap();
        let retained = crate::qualification::replay_archives(&fixture.0).unwrap();
        assert_eq!(retained["failed_captures_retained"], 1);
        assert_eq!(retained["successful_protocol_transcripts"], 0);
        changed["qualification"] = report["qualification"].clone();
        write_json(File::create(directory.join("run.json")).unwrap(), &changed).unwrap();
        assert!(
            format!(
                "{:#}",
                crate::qualification::replay_archives(&fixture.0).unwrap_err()
            )
            .contains("no favorable summary")
        );
    }
}
