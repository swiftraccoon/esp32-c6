//! Strict protocol 3 admission for the same-IDF queue API experiment.
//!
//! Raw counter observations remain integer ticks with a rational 1/16,000,000
//! second scale. A rejected, incomplete or restarted boot has no summary.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::protocol::Clocks;
use crate::{boot_text, compiler_version, digest, write_json};

const SAMPLES: u32 = 1_000;
const WARMUPS: u32 = 8;
const TICK_HZ: u64 = 16_000_000;
const COUNTER_LIMIT: u64 = 1 << 52;
const REPLY_MASK: u32 = 0xa5a5_5a5a;
const STD_BACKEND: &str = "rust-std-sync-channel";
const NATIVE_BACKEND: &str = "freertos-queue";

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
enum Record {
    Hello(Hello),
    Sample(Sample),
    Qualification(Qualification),
    Complete(Complete),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Hello {
    protocol: u32,
    boot_nonce: u64,
    backend: String,
    platform: String,
    tick_hz: u64,
    tick_width_bits: u32,
    sample_count: u32,
    observer_count: u32,
    warmup_iterations: u32,
    queue_capacity: u32,
    message_bytes: u32,
    main_priority: u32,
    worker_priority: u32,
    main_stack_bytes: u32,
    worker_stack_bytes: u32,
    sample_buffer_bytes: u32,
    sample_storage: String,
    rtos_tick_hz: u32,
    config_before: Configuration,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Configuration {
    clocks: Clocks,
    timer: TimerConfiguration,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TimerConfiguration {
    source: String,
    enabled: bool,
    unit0_enabled: bool,
    unit0_stalls_cpu: bool,
    systimer_conf: u32,
    pcr_systimer_conf: u32,
    pcr_systimer_func_clk_conf: u32,
}

impl Configuration {
    pub(crate) fn validate(&self) -> Result<()> {
        self.clocks.validate()?;
        let timer = &self.timer;
        // C6 PAC 0.24.0 PCR/SYSTIMER fields, matching pinned IDF 5.5.6.
        // APB enable/reset, XTAL function source/enable, UNIT0 enable/stall.
        ensure!(
            timer.pcr_systimer_conf & 3 == 1,
            "SYSTIMER APB disabled or reset"
        );
        ensure!(
            timer.pcr_systimer_func_clk_conf & (1 << 20) == 0,
            "SYSTIMER source is not XTAL"
        );
        ensure!(
            timer.pcr_systimer_func_clk_conf & (1 << 22) != 0,
            "SYSTIMER function clock disabled"
        );
        ensure!(
            timer.systimer_conf & (1 << 30) != 0,
            "SYSTIMER UNIT0 disabled"
        );
        ensure!(
            timer.systimer_conf & (1 << 28) != 0,
            "unexpected IDF UNIT0 CPU-stall configuration"
        );
        ensure!(
            timer.source == "xtal"
                && timer.enabled
                && timer.unit0_enabled
                && timer.unit0_stalls_cpu,
            "timer declarations disagree with supported register configuration"
        );
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    protocol: u32,
    boot_nonce: u64,
    backend: String,
    benchmark: String,
    index: u32,
    start_tick: u64,
    end_tick: u64,
    checksum: u32,
    phase_sequence: u32,
    blocked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Qualification {
    protocol: u32,
    boot_nonce: u64,
    backend: String,
    monotonic_reads: u32,
    backsteps: u32,
    monotonic_first_tick: u64,
    monotonic_last_tick: u64,
    blocked_start_tick: u64,
    blocked_end_tick: u64,
    legacy_before_us: u64,
    legacy_after_us: u64,
    blocked_wait_method: String,
    wfi_verified: bool,
    debugger_attached: bool,
    wfi_start_tick: u64,
    wfi_end_tick: u64,
    wfi_rtos_tick_start: u32,
    wfi_rtos_tick_end: u32,
    wfi_calls: u32,
    readiness_checks: u32,
    blocked_checks: u32,
    queue_empty_checks: u32,
    empty_receive_pass: bool,
    full_send_pass: bool,
    cleanup_pass: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Complete {
    protocol: u32,
    boot_nonce: u64,
    backend: String,
    samples: u32,
    message_roundtrip_samples: u32,
    timer_overhead_samples: u32,
    config_after: Configuration,
    worker_joined: bool,
    backend_destroyed: bool,
}

/// Describes one accepted boot in raw ticks; no observer subtraction is made.
#[derive(Debug, Serialize)]
pub(crate) struct QueueRunSummary {
    pub(crate) backend: String,
    pub(crate) boot_nonce: u64,
    pub(crate) tick_hz: u64,
    pub(crate) benchmarks: BTreeMap<String, TickSummary>,
}

/// Nearest-rank statistics of all observations within one accepted boot.
#[derive(Debug, Serialize)]
pub(crate) struct TickSummary {
    pub(crate) sample_count: usize,
    pub(crate) min_ticks: u64,
    pub(crate) p50_ticks: u64,
    pub(crate) p95_ticks: u64,
    pub(crate) p99_ticks: u64,
    pub(crate) max_ticks: u64,
    pub(crate) mean_ticks: f64,
}

impl TickSummary {
    fn from_samples(samples: &[Sample]) -> Self {
        // Only complete boots call this with exactly SAMPLES validated entries.
        let mut ticks: Vec<_> = samples
            .iter()
            .map(|sample| sample.end_tick - sample.start_tick)
            .collect();
        ticks.sort_unstable();
        let rank = |percent: usize| ticks[(ticks.len() * percent).div_ceil(100) - 1];
        Self {
            sample_count: ticks.len(),
            min_ticks: ticks[0],
            p50_ticks: rank(50),
            p95_ticks: rank(95),
            p99_ticks: rank(99),
            max_ticks: ticks[ticks.len() - 1],
            mean_ticks: ticks.iter().map(|&tick| tick as f64).sum::<f64>() / ticks.len() as f64,
        }
    }
}

#[derive(Debug, Default)]
struct Boot {
    hello: Option<Hello>,
    samples: BTreeMap<String, Vec<Sample>>,
    last_end_tick: Option<u64>,
    qualification: Option<Qualification>,
    complete: Option<Complete>,
    rejected: bool,
}

fn backend_supported(backend: &str) -> Result<()> {
    ensure!(
        matches!(backend, STD_BACKEND | NATIVE_BACKEND),
        "unknown queue backend {backend}"
    );
    Ok(())
}

fn ticks_ordered(start: u64, end: u64) -> Result<()> {
    ensure!(
        start < COUNTER_LIMIT && end < COUNTER_LIMIT,
        "timestamp exceeds 52-bit counter"
    );
    ensure!(
        end >= start,
        "counter moved backwards; wrap is not normalized"
    );
    Ok(())
}

impl Boot {
    /// A validation error permanently rejects this boot, even if ignored by a caller.
    fn accept(&mut self, record: Record, backend: &str, previous_nonces: &[u64]) -> Result<bool> {
        ensure!(!self.rejected, "boot already rejected");
        let result = self.accept_inner(record, backend, previous_nonces);
        if result.is_err() {
            self.rejected = true;
        }
        result
    }

    fn common(&self, protocol: u32, nonce: u64, backend: &str) -> Result<()> {
        let hello = self.hello.as_ref().context("record before hello")?;
        ensure!(protocol == 3, "protocol mismatch");
        ensure!(
            nonce == hello.boot_nonce,
            "boot nonce changed during capture"
        );
        ensure!(backend == hello.backend, "backend changed during capture");
        Ok(())
    }

    fn accept_inner(
        &mut self,
        record: Record,
        backend: &str,
        previous_nonces: &[u64],
    ) -> Result<bool> {
        backend_supported(backend)?;
        ensure!(self.complete.is_none(), "record after completed boot");
        match record {
            Record::Hello(hello) => {
                ensure!(self.hello.is_none(), "duplicate hello or unexpected reboot");
                ensure!(hello.protocol == 3, "unsupported queue protocol");
                ensure!(hello.backend == backend, "unexpected queue backend");
                ensure!(
                    !previous_nonces.contains(&hello.boot_nonce),
                    "boot nonce was already observed"
                );
                ensure!(
                    hello.platform == "esp-idf-freertos",
                    "unexpected queue platform"
                );
                ensure!(
                    hello.tick_hz == TICK_HZ && hello.tick_width_bits == 52,
                    "unsupported counter scale or width"
                );
                ensure!(
                    hello.sample_count == SAMPLES
                        && hello.observer_count == SAMPLES
                        && hello.warmup_iterations == WARMUPS,
                    "wrong sample or warmup counts"
                );
                ensure!(
                    hello.queue_capacity == 1 && hello.message_bytes == 4,
                    "wrong queue layout"
                );
                ensure!(
                    hello.main_priority == 1
                        && hello.worker_priority == 5
                        && hello.rtos_tick_hz == 100,
                    "wrong observed priorities or kernel tick rate"
                );
                ensure!(
                    hello.main_stack_bytes == 32_768 && hello.worker_stack_bytes == 8_192,
                    "wrong stack reservations"
                );
                ensure!(
                    hello.sample_storage == "preallocated-heap"
                        && hello.sample_buffer_bytes > 0
                        && hello.sample_buffer_bytes <= 131_072,
                    "unsupported sample storage or buffer size"
                );
                hello.config_before.validate()?;
                self.hello = Some(hello);
            }
            Record::Sample(sample) => {
                self.common(sample.protocol, sample.boot_nonce, &sample.backend)?;
                ensure!(
                    self.qualification.is_none(),
                    "sample after qualification totals"
                );
                ticks_ordered(sample.start_tick, sample.end_tick)?;
                if let Some(last) = self.last_end_tick {
                    ensure!(sample.start_tick >= last, "sample timestamps out of order");
                }
                match sample.benchmark.as_str() {
                    "timer-overhead" => {
                        ensure!(
                            !self.samples.contains_key("message-roundtrip"),
                            "observer sample after message phase"
                        );
                        ensure!(
                            sample.checksum == 0 && sample.phase_sequence == 0 && !sample.blocked,
                            "invalid observer metadata"
                        );
                    }
                    "message-roundtrip" => {
                        ensure!(
                            self.samples.get("timer-overhead").map_or(0, Vec::len)
                                == SAMPLES as usize,
                            "message phase before complete observer phase"
                        );
                        ensure!(
                            sample.checksum == sample.index ^ REPLY_MASK,
                            "wrong reply checksum"
                        );
                        ensure!(
                            sample.phase_sequence == sample.index + WARMUPS && sample.blocked,
                            "message responder phase or blocked qualification mismatch"
                        );
                        ensure!(
                            sample.end_tick > sample.start_tick,
                            "nonpositive message duration"
                        );
                        ensure!(
                            sample.end_tick - sample.start_tick <= TICK_HZ,
                            "message duration exceeds the declared one-second admission guard"
                        );
                    }
                    _ => anyhow::bail!("unknown benchmark {}", sample.benchmark),
                }
                let entries = self.samples.entry(sample.benchmark.clone()).or_default();
                ensure!(
                    sample.index as usize == entries.len() && sample.index < SAMPLES,
                    "duplicate, missing, reordered or excess sample"
                );
                self.last_end_tick = Some(sample.end_tick);
                entries.push(sample);
            }
            Record::Qualification(qualification) => {
                self.common(
                    qualification.protocol,
                    qualification.boot_nonce,
                    &qualification.backend,
                )?;
                ensure!(self.qualification.is_none(), "duplicate qualification");
                self.require_sample_counts()?;
                ensure!(
                    qualification.monotonic_reads >= SAMPLES && qualification.backsteps == 0,
                    "timer monotonicity qualification failed"
                );
                ticks_ordered(
                    qualification.monotonic_first_tick,
                    qualification.monotonic_last_tick,
                )?;
                ensure!(
                    qualification.monotonic_last_tick > qualification.monotonic_first_tick,
                    "timer failed to advance during qualification"
                );
                ticks_ordered(
                    qualification.blocked_start_tick,
                    qualification.blocked_end_tick,
                )?;
                ensure!(
                    qualification.monotonic_last_tick <= qualification.blocked_start_tick,
                    "qualification timer phases out of order"
                );
                if let Some(first) = self
                    .samples
                    .get("timer-overhead")
                    .and_then(|samples| samples.first())
                {
                    ensure!(
                        qualification.wfi_end_tick <= first.start_tick,
                        "qualification followed timed samples"
                    );
                }
                ensure!(
                    qualification.legacy_after_us >= qualification.legacy_before_us,
                    "legacy timer moved backwards"
                );
                let legacy_ticks = (qualification.legacy_after_us - qualification.legacy_before_us)
                    .checked_mul(16)
                    .context("legacy timer scale overflow")?;
                let blocked_ticks =
                    qualification.blocked_end_tick - qualification.blocked_start_tick;
                ensure!(
                    blocked_ticks >= 160_000,
                    "blocked counter interval shorter than 10 ms"
                );
                ensure!(
                    blocked_ticks <= TICK_HZ / 2,
                    "blocked counter qualification exceeds 500 ms guard"
                );
                ensure!(
                    blocked_ticks <= legacy_ticks.saturating_add(32)
                        && legacy_ticks <= blocked_ticks.saturating_add(16_000),
                    "raw counter scale disagrees with legacy timer bracket"
                );
                ensure!(
                    qualification.blocked_wait_method == "std-thread-sleep"
                        && qualification.wfi_verified
                        && !qualification.debugger_attached,
                    "unsupported blocked-continuity method, missing WFI qualification or debugger attached"
                );
                ticks_ordered(qualification.wfi_start_tick, qualification.wfi_end_tick)?;
                ensure!(
                    qualification.blocked_end_tick <= qualification.wfi_start_tick,
                    "WFI qualification phases out of order"
                );
                let rtos_delta = qualification
                    .wfi_rtos_tick_end
                    .wrapping_sub(qualification.wfi_rtos_tick_start);
                ensure!(
                    (3..=50).contains(&rtos_delta)
                        && (1..=10_000).contains(&qualification.wfi_calls),
                    "invalid WFI call or elapsed kernel tick qualification"
                );
                let wfi_ticks = qualification.wfi_end_tick - qualification.wfi_start_tick;
                ensure!(
                    wfi_ticks >= u64::from(rtos_delta - 1) * 160_000
                        && wfi_ticks <= u64::from(rtos_delta + 1) * 160_000,
                    "SYSTIMER did not track blocked WFI interval against kernel ticks"
                );
                ensure!(
                    qualification.readiness_checks == SAMPLES + WARMUPS
                        && qualification.blocked_checks == SAMPLES + WARMUPS,
                    "incomplete responder readiness or blocked checks"
                );
                let expected_empty = if backend == NATIVE_BACKEND {
                    SAMPLES + WARMUPS
                } else {
                    0
                };
                ensure!(
                    qualification.queue_empty_checks == expected_empty,
                    "wrong native empty-queue checks"
                );
                ensure!(
                    qualification.empty_receive_pass
                        && qualification.full_send_pass
                        && qualification.cleanup_pass,
                    "backend full/empty/cleanup qualification failed"
                );
                self.qualification = Some(qualification);
            }
            Record::Complete(complete) => {
                self.common(complete.protocol, complete.boot_nonce, &complete.backend)?;
                self.require_sample_counts()?;
                ensure!(
                    self.qualification.is_some(),
                    "missing timer/backend qualification"
                );
                ensure!(
                    complete.samples == SAMPLES * 2
                        && complete.message_roundtrip_samples == SAMPLES
                        && complete.timer_overhead_samples == SAMPLES,
                    "incorrect completion counts"
                );
                ensure!(
                    complete.worker_joined && complete.backend_destroyed,
                    "worker/backend teardown failed"
                );
                complete.config_after.validate()?;
                ensure!(
                    complete.config_after
                        == self.hello.as_ref().context("missing hello")?.config_before,
                    "clock or timer configuration changed during boot"
                );
                self.complete = Some(complete);
            }
        }
        Ok(self.complete.is_some())
    }

    fn require_sample_counts(&self) -> Result<()> {
        for name in ["timer-overhead", "message-roundtrip"] {
            ensure!(
                self.samples.get(name).map_or(0, Vec::len) == SAMPLES as usize,
                "incomplete {name} samples"
            );
        }
        Ok(())
    }

    fn summary(&self) -> Result<QueueRunSummary> {
        ensure!(
            !self.rejected && self.complete.is_some(),
            "incomplete or rejected boot has no summary"
        );
        let hello = self.hello.as_ref().context("missing hello")?;
        Ok(QueueRunSummary {
            backend: hello.backend.clone(),
            boot_nonce: hello.boot_nonce,
            tick_hz: hello.tick_hz,
            benchmarks: self
                .samples
                .iter()
                .map(|(name, samples)| (name.clone(), TickSummary::from_samples(samples)))
                .collect(),
        })
    }

    fn report_fields(&self) -> Result<Value> {
        Ok(
            json!({"hello": self.hello, "qualification": self.qualification,
            "config_after": self.complete.as_ref().map(|complete| &complete.config_after),
            "benchmarks": self.summary()?.benchmarks,
            "tick_scale_seconds": {"numerator": 1, "denominator": TICK_HZ}}),
        )
    }
}

fn parse_record(line: &[u8]) -> Result<Option<Record>> {
    let trimmed = line.trim_ascii();
    if !trimmed.starts_with(b"{") {
        return Ok(None);
    }
    serde_json::from_slice(trimmed)
        .context("malformed queue protocol record")
        .map(Some)
}

/// Validates all archived records, including any records following completion.
pub(crate) fn validate_file(path: &Path, backend: &str, previous_nonces: &[u64]) -> Result<Value> {
    validate(BufReader::new(File::open(path)?), backend, previous_nonces)?.report_fields()
}

/// Replays in-memory method controls through the same complete boot validator.
pub(crate) fn validate_bytes(
    bytes: &[u8],
    backend: &str,
    previous_nonces: &[u64],
) -> Result<Value> {
    validate(std::io::Cursor::new(bytes), backend, previous_nonces)?.report_fields()
}

fn validate(input: impl BufRead, backend: &str, previous_nonces: &[u64]) -> Result<Boot> {
    backend_supported(backend)?;
    let mut boot = Boot::default();
    boot_text::replay(input, |line| {
        if let Some(record) = parse_record(line)? {
            boot.accept(record, backend, previous_nonces)
        } else {
            Ok(false)
        }
    })?;
    boot.summary()?;
    Ok(boot)
}

#[derive(Debug)]
pub(crate) struct Capture<'a> {
    pub(crate) admission: &'a crate::qualification::Admission,
    pub(crate) port: &'a str,
    pub(crate) output: &'a Path,
    pub(crate) backend: &'a str,
    pub(crate) elf: &'a Path,
    pub(crate) provenance: &'a Path,
    pub(crate) boot_id: &'a str,
    pub(crate) run_order: u32,
    pub(crate) timeout: Duration,
    pub(crate) previous_nonces: &'a [u64],
}

/// Requests a warm reset and retains successful or failed protocol-3 evidence.
///
/// Artifact, backend and timeout checks precede reset. Output is never replaced.
pub(crate) fn capture(config: Capture<'_>) -> Result<()> {
    backend_supported(config.backend)?;
    ensure!(
        !config.timeout.is_zero(),
        "capture timeout must be positive"
    );
    ensure!(
        !config.output.exists(),
        "result directory already exists; refusing overwrite"
    );
    let firmware = fs::read(config.elf).context("reading firmware ELF")?;
    let provenance_bytes = fs::read(config.provenance)?;
    config
        .admission
        .check_inputs(&firmware, &provenance_bytes)?;
    let provenance: Value = serde_json::from_slice(&provenance_bytes)?;
    ensure!(provenance.is_object(), "build provenance must be an object");
    let collector = fs::read(std::env::current_exe()?)?;
    let compiler = compiler_version()?;
    if let Some(parent) = config.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(config.output)?;
    fs::write(config.output.join("firmware.elf"), &firmware)?;
    let mut report = json!({
        "schema_version": 4, "admission": config.admission, "status": "running", "platform": "esp-idf-freertos",
        "backend": config.backend,
        "timestamp_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        "independent_boot_id": config.boot_id, "run_order_index": config.run_order,
        "reset_kind": "RTS warm-reset request; DTR deasserted",
        "serial": {"baud": 115200, "data_bits": 8, "parity": "none", "stop_bits": 1},
        "firmware_sha256": digest(&firmware), "collector_version": env!("CARGO_PKG_VERSION"),
        "collector_executable_sha256": digest(&collector), "host_rustc_at_capture": compiler,
        "provenance": provenance, "previous_nonces": config.previous_nonces,
        "hello": null, "qualification": null, "config_after": null, "benchmarks": null, "failure": null,
        "tick_scale_seconds": {"numerator": 1, "denominator": TICK_HZ},
        "limitations": [
            "Warm reset request only; nonce freshness does not establish independent statistical trials",
            "52-bit onboard SYSTIMER raw ticks at nominal 16 MHz; no external frequency calibration",
            "1000 observations per benchmark within each boot; nearest-rank percentiles",
            "Gross request/reply and observer ticks retained; no observer subtraction",
            "Blocked sleep and SDK WFI-call continuity checked against kernel ticks; no external timing reference",
            "Before/after register equality cannot exclude transient intermediate changes",
            "Device protocol cannot authenticate firmware; preserve flash evidence separately",
            "host_rustc_at_capture observes PATH, not the compiler used to build this executable"
        ]
    });
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    let mut raw = File::create(config.output.join("serial.raw"))?;
    let mut records = File::create(config.output.join("records.jsonl"))?;
    let mut boot = Boot::default();
    let result = capture_boot(&config, &mut raw, &mut records, &mut boot);
    raw.flush()?;
    records.flush()?;
    report["hello"] = serde_json::to_value(&boot.hello)?;
    report["qualification"] = serde_json::to_value(&boot.qualification)?;
    match &result {
        Ok(()) => {
            for (key, value) in boot
                .report_fields()?
                .as_object()
                .context("internal report fields must be object")?
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
        "Validated {} boot {}: {}",
        config.backend,
        config.boot_id,
        config.output.display()
    );
    Ok(())
}

fn capture_boot(
    config: &Capture<'_>,
    raw: &mut impl Write,
    records: &mut impl Write,
    boot: &mut Boot,
) -> Result<()> {
    let mut port = serialport::new(config.port, 115_200)
        .timeout(Duration::from_millis(100))
        .open()
        .context("opening serial port")?;
    port.write_data_terminal_ready(false)?;
    port.write_request_to_send(true)?;
    std::thread::sleep(Duration::from_millis(100));
    port.clear(serialport::ClearBuffer::Input)?;
    port.write_request_to_send(false)?;
    capture_stream(&mut port, raw, records, boot, config)
}

/// The same reader/guard path used by a connected port and transport fixtures.
fn capture_stream(
    input: &mut impl Read,
    raw: &mut impl Write,
    records: &mut impl Write,
    boot: &mut Boot,
    config: &Capture<'_>,
) -> Result<()> {
    let result = boot_text::capture(input, raw, config.timeout, |line| {
        if let Some(record) = parse_record(line)? {
            serde_json::to_writer(&mut *records, &record)?;
            records.write_all(b"\n")?;
            boot.accept(record, config.backend, config.previous_nonces)
        } else {
            Ok(false)
        }
    });
    if result.is_err() {
        boot.rejected = true;
    }
    result
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io;

    use super::*;
    use crate::boot_text::MAX_LINE_BYTES;

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

    pub(crate) fn transcript(backend: &str) -> Vec<Value> {
        let mut records = vec![json!({
            "event": "hello", "protocol": 3, "boot_nonce": 42, "backend": backend,
            "platform": "esp-idf-freertos", "tick_hz": TICK_HZ, "tick_width_bits": 52,
            "sample_count": 1000, "observer_count": 1000, "warmup_iterations": 8,
            "queue_capacity": 1, "message_bytes": 4, "main_priority": 1,
            "worker_priority": 5, "main_stack_bytes": 32768, "worker_stack_bytes": 8192,
            "sample_buffer_bytes": 48000, "sample_storage": "preallocated-heap",
            "rtos_tick_hz": 100, "config_before": configuration()
        })];
        for benchmark in ["timer-overhead", "message-roundtrip"] {
            for index in 0..SAMPLES {
                let observer = benchmark == "timer-overhead";
                let start_tick = if observer {
                    900_000 + u64::from(index) * 100
                } else {
                    2_000_000 + u64::from(index) * 10_000
                };
                let elapsed = if observer {
                    u64::from(index % 7)
                } else {
                    100 + u64::from(index)
                };
                records.push(json!({"event": "sample", "protocol": 3, "boot_nonce": 42,
                    "backend": backend, "benchmark": benchmark, "index": index,
                    "start_tick": start_tick, "end_tick": start_tick + elapsed,
                    "checksum": if observer { 0 } else { index ^ REPLY_MASK },
                    "phase_sequence": if observer { 0 } else { index + WARMUPS }, "blocked": !observer}));
            }
        }
        records.push(
            json!({"event": "qualification", "protocol": 3, "boot_nonce": 42,
            "backend": backend, "monotonic_reads": 1000, "backsteps": 0,
            "monotonic_first_tick": 1000, "monotonic_last_tick": 2000,
            "blocked_start_tick": 3000, "blocked_end_tick": 323000,
            "legacy_before_us": 187, "legacy_after_us": 20188,
            "blocked_wait_method": "std-thread-sleep", "wfi_verified": true,
            "debugger_attached": false, "wfi_start_tick": 324000, "wfi_end_tick": 804000,
            "wfi_rtos_tick_start": 10, "wfi_rtos_tick_end": 13, "wfi_calls": 3,
            "readiness_checks": 1008, "blocked_checks": 1008,
            "queue_empty_checks": if backend == NATIVE_BACKEND { 1008 } else { 0 },
            "empty_receive_pass": true, "full_send_pass": true, "cleanup_pass": true}),
        );
        records.push(json!({"event": "complete", "protocol": 3, "boot_nonce": 42,
            "backend": backend, "samples": 2000, "message_roundtrip_samples": 1000,
            "timer_overhead_samples": 1000, "config_after": configuration(),
            "worker_joined": true, "backend_destroyed": true}));
        records
    }

    fn encoded(records: &[Value]) -> Vec<u8> {
        let mut bytes = b"ESP-ROM:esp32c6\r\n".to_vec();
        for record in records {
            serde_json::to_writer(&mut bytes, record).unwrap();
            bytes.push(b'\n');
        }
        bytes
    }

    fn replay(records: &[Value], backend: &str) -> Result<Boot> {
        validate(io::Cursor::new(encoded(records)), backend, &[])
    }

    #[test]
    fn both_backends_keep_raw_ticks_nearest_rank_and_complete_evidence() {
        for backend in [STD_BACKEND, NATIVE_BACKEND] {
            let boot = replay(&transcript(backend), backend).unwrap();
            let report = boot.report_fields().unwrap();
            assert_eq!(report["hello"]["backend"], backend);
            assert_eq!(
                report["tick_scale_seconds"],
                json!({"numerator": 1, "denominator": 16_000_000})
            );
            let summary = boot.summary().unwrap();
            let ticks = &summary.benchmarks["message-roundtrip"];
            assert_eq!(ticks.sample_count, 1000);
            assert_eq!(
                (
                    ticks.min_ticks,
                    ticks.p50_ticks,
                    ticks.p95_ticks,
                    ticks.p99_ticks,
                    ticks.max_ticks
                ),
                (100, 599, 1049, 1089, 1099)
            );
            assert_eq!(ticks.mean_ticks, 599.5);
            assert_eq!(summary.benchmarks["timer-overhead"].min_ticks, 0);
        }
    }

    #[test]
    fn rejects_missing_duplicate_out_of_order_and_trailing_records() {
        let original = transcript(STD_BACKEND);
        for position in [0, 1, 1000, 1001, 2000, 2001, 2002] {
            let mut missing = original.clone();
            missing.remove(position);
            assert!(replay(&missing, STD_BACKEND).is_err(), "missing {position}");
            let mut duplicate = original.clone();
            duplicate.insert(position, duplicate[position].clone());
            assert!(
                replay(&duplicate, STD_BACKEND).is_err(),
                "duplicate {position}"
            );
        }
        let mut reordered = original.clone();
        reordered.swap(1, 2);
        assert!(replay(&reordered, STD_BACKEND).is_err());
        let mut rebooted = original;
        rebooted.push(rebooted[0].clone());
        assert!(replay(&rebooted, STD_BACKEND).is_err());
    }

    #[test]
    fn rejects_inconsistent_headers_unknown_fields_and_wrong_backend() {
        for (field, wrong) in [
            ("protocol", json!(2)),
            ("tick_hz", json!(1_000_000)),
            ("tick_width_bits", json!(64)),
            ("sample_count", json!(999)),
            ("observer_count", json!(999)),
            ("warmup_iterations", json!(0)),
            ("queue_capacity", json!(2)),
            ("message_bytes", json!(8)),
            ("main_priority", json!(5)),
            ("worker_priority", json!(1)),
            ("rtos_tick_hz", json!(1000)),
            ("main_stack_bytes", json!(8192)),
            ("worker_stack_bytes", json!(32768)),
            ("sample_buffer_bytes", json!(0)),
            ("sample_storage", json!("stack")),
            ("unknown", json!(true)),
        ] {
            let mut records = transcript(STD_BACKEND);
            records[0][field] = wrong;
            assert!(replay(&records, STD_BACKEND).is_err(), "field {field}");
        }
        assert!(replay(&transcript(STD_BACKEND), NATIVE_BACKEND).is_err());
        assert!(replay(&transcript(STD_BACKEND), "unknown").is_err());
        let mut missing = transcript(STD_BACKEND);
        missing[0].as_object_mut().unwrap().remove("tick_hz");
        assert!(replay(&missing, STD_BACKEND).is_err());
    }

    #[test]
    fn rejects_timer_configuration_lies_and_changed_readback() {
        for (field, wrong) in [
            ("source", json!("fosc")),
            ("enabled", json!(false)),
            ("unit0_enabled", json!(false)),
            ("unit0_stalls_cpu", json!(false)),
            ("systimer_conf", json!(1_u32 << 30)),
            ("pcr_systimer_conf", json!(3)),
            (
                "pcr_systimer_func_clk_conf",
                json!((1_u32 << 20) | (1 << 22)),
            ),
        ] {
            let mut records = transcript(STD_BACKEND);
            records[0]["config_before"]["timer"][field] = wrong;
            assert!(replay(&records, STD_BACKEND).is_err(), "field {field}");
        }
        let mut changed = transcript(STD_BACKEND);
        changed[2002]["config_after"]["timer"]["systimer_conf"] = json!(0x5000_0001_u32);
        assert!(replay(&changed, STD_BACKEND).is_err());
        let mut wrong_cpu = transcript(STD_BACKEND);
        wrong_cpu[0]["config_before"]["clocks"]["cpu_mhz"] = json!(120);
        assert!(replay(&wrong_cpu, STD_BACKEND).is_err());
        let mut missing = transcript(STD_BACKEND);
        missing[2002]
            .as_object_mut()
            .unwrap()
            .remove("config_after");
        assert!(replay(&missing, STD_BACKEND).is_err());
    }

    #[test]
    fn rejects_sequence_checksum_phase_and_timestamp_errors() {
        for (field, wrong) in [
            ("protocol", json!(2)),
            ("boot_nonce", json!(43)),
            ("backend", json!(NATIVE_BACKEND)),
            ("index", json!(1)),
            ("checksum", json!(0)),
            ("phase_sequence", json!(0)),
            ("blocked", json!(false)),
            ("benchmark", json!("unknown")),
            ("end_tick", json!(1_999_999)),
            ("end_tick", json!(2_000_000 + TICK_HZ + 1)),
            ("start_tick", json!(COUNTER_LIMIT)),
            ("unknown", json!(true)),
        ] {
            let mut records = transcript(STD_BACKEND);
            records[1001][field] = wrong;
            assert!(replay(&records, STD_BACKEND).is_err(), "field {field}");
        }
        let mut backwards = transcript(STD_BACKEND);
        backwards[2]["start_tick"] = json!(899_999);
        backwards[2]["end_tick"] = json!(900_001);
        assert!(replay(&backwards, STD_BACKEND).is_err());
    }

    #[test]
    fn qualification_requires_proved_counts_progress_scale_and_wfi() {
        for (field, wrong) in [
            ("monotonic_reads", json!(999)),
            ("backsteps", json!(1)),
            ("monotonic_last_tick", json!(999)),
            ("blocked_end_tick", json!(3000)),
            ("blocked_end_tick", json!(8_003_001)),
            ("legacy_after_us", json!(100)),
            ("legacy_before_us", json!(1000)),
            ("blocked_wait_method", json!("busy-wait")),
            ("wfi_verified", json!(false)),
            ("debugger_attached", json!(true)),
            ("wfi_calls", json!(0)),
            ("wfi_rtos_tick_end", json!(10)),
            ("wfi_end_tick", json!(325000)),
            ("readiness_checks", json!(1007)),
            ("blocked_checks", json!(1007)),
            ("queue_empty_checks", json!(1008)),
            ("empty_receive_pass", json!(false)),
            ("full_send_pass", json!(false)),
            ("cleanup_pass", json!(false)),
            ("unknown", json!(true)),
        ] {
            let mut records = transcript(STD_BACKEND);
            records[2001][field] = wrong;
            assert!(replay(&records, STD_BACKEND).is_err(), "field {field}");
        }
        let mut native_missing = transcript(NATIVE_BACKEND);
        native_missing[2001]["queue_empty_checks"] = json!(0);
        assert!(replay(&native_missing, NATIVE_BACKEND).is_err());
    }

    #[test]
    fn completion_rejects_bad_counts_or_failed_cleanup() {
        for (field, wrong) in [
            ("samples", json!(1999)),
            ("message_roundtrip_samples", json!(999)),
            ("timer_overhead_samples", json!(999)),
            ("worker_joined", json!(false)),
            ("backend_destroyed", json!(false)),
            ("unknown", json!(true)),
        ] {
            let mut records = transcript(STD_BACKEND);
            records[2002][field] = wrong;
            assert!(replay(&records, STD_BACKEND).is_err(), "field {field}");
        }
    }

    #[test]
    fn nonce_freshness_is_explicit_and_any_rejection_revokes_results() {
        let bytes = encoded(&transcript(STD_BACKEND));
        assert!(validate(io::Cursor::new(&bytes), STD_BACKEND, &[1, 42]).is_err());
        assert!(validate(io::Cursor::new(&bytes), STD_BACKEND, &[1, 41]).is_ok());
        let mut boot = replay(&transcript(STD_BACKEND), STD_BACKEND).unwrap();
        assert!(boot.summary().is_ok());
        let duplicate: Record = serde_json::from_value(transcript(STD_BACKEND).remove(0)).unwrap();
        assert!(boot.accept(duplicate, STD_BACKEND, &[]).is_err());
        assert!(boot.summary().is_err());
        assert!(
            boot.accept(
                serde_json::from_value(transcript(STD_BACKEND).remove(0)).unwrap(),
                STD_BACKEND,
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn archived_framing_rejects_malformed_overlong_and_partial_json() {
        let original = encoded(&transcript(STD_BACKEND));
        for suffix in [b"{broken\n".as_slice(), b"{\"event\":\"hello\"}".as_slice()] {
            let mut bytes = original.clone();
            bytes.extend_from_slice(suffix);
            assert!(validate(io::Cursor::new(bytes), STD_BACKEND, &[]).is_err());
        }
        for newline in [false, true] {
            let mut bytes = original.clone();
            bytes.extend(std::iter::repeat_n(b'x', MAX_LINE_BYTES + 1));
            if newline {
                bytes.push(b'\n');
            }
            assert!(validate(io::Cursor::new(bytes), STD_BACKEND, &[]).is_err());
        }
        let mut maximum_noise = vec![b'x'; MAX_LINE_BYTES];
        maximum_noise.push(b'\n');
        maximum_noise.extend_from_slice(&original);
        assert!(validate(io::Cursor::new(maximum_noise), STD_BACKEND, &[]).is_ok());
    }

    #[test]
    fn queue_capture_and_replay_reject_pre_json_crashes_and_trailing_diagnostics() {
        let valid = encoded(&transcript(NATIVE_BACKEND));
        let hello = valid.iter().position(|&byte| byte == b'{').unwrap();
        let config = Capture {
            admission: &crate::qualification::Admission::fixture_pilot(),
            port: "unused",
            output: Path::new("unused"),
            backend: NATIVE_BACKEND,
            elf: Path::new("unused"),
            provenance: Path::new("unused"),
            boot_id: "fixture",
            run_order: 1,
            timeout: Duration::from_secs(1),
            previous_nonces: &[],
        };
        for fault in [
            b"panic: before JSON\n".as_slice(),
            b"Rebooting...\n".as_slice(),
            b"ESP-ROM:esp32c6-20220919\n".as_slice(),
        ] {
            let bytes = [&valid[..hello], fault, &valid[hello..]].concat();
            assert!(validate(io::Cursor::new(&bytes), NATIVE_BACKEND, &[]).is_err());
            let mut boot = Boot::default();
            let mut raw = Vec::new();
            assert!(
                capture_stream(
                    &mut io::Cursor::new(&bytes),
                    &mut raw,
                    &mut Vec::new(),
                    &mut boot,
                    &config
                )
                .is_err()
            );
            assert!(!raw.is_empty());
            assert!(boot.summary().is_err());
        }
        let mut boot = Boot::default();
        capture_stream(
            &mut io::Cursor::new(&valid),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut boot,
            &config,
        )
        .unwrap();
        assert!(boot.summary().is_ok());
        let padding = (512 + 1024 - (valid.len() + 1) % 1024) % 1024;
        let mut bytes = vec![b'x'; padding];
        bytes.push(b'\n');
        bytes.extend_from_slice(&valid);
        bytes.extend_from_slice(b"Rebooting...");
        let mut boot = Boot::default();
        assert!(
            capture_stream(
                &mut io::Cursor::new(&bytes),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut boot,
                &config
            )
            .is_err()
        );
        assert!(boot.summary().is_err());
        assert!(validate(io::Cursor::new(bytes), NATIVE_BACKEND, &[]).is_err());
    }
}
