//! Artifact-bound, single-boot qualification for the unmodified RTIC C6 example.
//!
//! Raw console bytes alone cannot establish elapsed silence. Capture and replay
//! therefore share bounded framing and require a separate host receipt stream.
//! Host pacing observations are neither device clock calibration nor benchmarks.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use object::{Object, ObjectSegment};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::boot_text::{BootTextGuard, MAX_LINE_BYTES};
use crate::qualification::{Admission, CapturePurpose};
use crate::{compiler_version, digest, write_json};

pub(crate) const EXPERIMENT: &str = "rtic-monotonic-qualification-v1";
const CEILING_NS: u64 = 45_000_000_000;
const GUARD_NS: u64 = 12_000_000_000;
const NAMES: [&str; 7] = [
    "init",
    "hello from foo",
    "hello from bar",
    "hello from baz",
    "bye from foo",
    "bye from bar",
    "bye from baz",
];
const TASKS: [&str; 3] = ["foo", "bar", "baz"];
const PACING_NS: [(u64, u64); 3] = [
    (1_500_000_000, 3_500_000_000),
    (2_500_000_000, 4_500_000_000),
    (3_500_000_000, 5_500_000_000),
];

/// Nanoseconds are measured from the host RTS-release request, not device reset.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EndReason {
    GuardElapsed,
    Eof,
    ReadError,
    Timeout,
    Rejected,
}

/// Every read advances a cumulative raw-byte boundary; polls do not.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
enum Receipt {
    Start {
        at_ns: u64,
        raw_end: u64,
    },
    Read {
        at_ns: u64,
        raw_end: u64,
    },
    Poll {
        at_ns: u64,
        raw_end: u64,
    },
    End {
        at_ns: u64,
        raw_end: u64,
        reason: EndReason,
    },
}

impl Receipt {
    fn coordinates(self) -> (u64, u64) {
        match self {
            Self::Start { at_ns, raw_end }
            | Self::Read { at_ns, raw_end }
            | Self::Poll { at_ns, raw_end }
            | Self::End { at_ns, raw_end, .. } => (at_ns, raw_end),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Marker {
    name: &'static str,
    at_ns: u64,
    raw_end: u64,
}

/// One owned stream per attempt. No error permits resynchronization.
#[derive(Debug, Default)]
struct Console {
    guard: BootTextGuard,
    line: Vec<u8>,
    bytes: u64,
    initialized: bool,
    hello: [bool; 3],
    hello_order: Vec<&'static str>,
    byes: usize,
    markers: Vec<Marker>,
    rejected: bool,
}

impl Console {
    fn push(&mut self, bytes: &[u8], at_ns: u64) -> Result<()> {
        ensure!(!self.rejected, "RTIC console attempt was already rejected");
        let result = (|| {
            for &byte in bytes {
                self.bytes = self.bytes.checked_add(1).context("raw offset overflow")?;
                if byte == b'\n' {
                    self.accept_line(at_ns)?;
                    self.line.clear();
                } else {
                    ensure!(
                        self.line.len() < MAX_LINE_BYTES,
                        "RTIC console line exceeds limit"
                    );
                    self.line.push(byte);
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.rejected = true;
        }
        result
    }

    fn accept_line(&mut self, at_ns: u64) -> Result<()> {
        self.guard.observe(&self.line)?;
        let line = std::str::from_utf8(&self.line)
            .context("RTIC console contains invalid UTF-8")?
            .trim_ascii();
        if line.is_empty() {
            return Ok(());
        }
        ensure!(
            !line.starts_with(['{', '[']),
            "JSON is not the RTIC console format"
        );
        if !self.initialized {
            if line == NAMES[0] {
                self.guard.start_console()?;
                self.initialized = true;
                self.markers.push(Marker {
                    name: NAMES[0],
                    at_ns,
                    raw_end: self.bytes,
                });
                return Ok(());
            }
            ensure!(
                allowed_startup(line),
                "unexpected pre-init RTIC text: {line}"
            );
            return Ok(());
        }
        if let Some(index) = NAMES[1..4].iter().position(|&name| line == name) {
            ensure!(self.byes == 0, "RTIC hello appeared after first bye");
            ensure!(!self.hello[index], "duplicate RTIC hello");
            self.hello[index] = true;
            self.hello_order.push(TASKS[index]);
            self.markers.push(Marker {
                name: NAMES[index + 1],
                at_ns,
                raw_end: self.bytes,
            });
            return Ok(());
        }
        ensure!(
            self.byes < 3 && line == NAMES[4 + self.byes],
            "unexpected RTIC application line or bye order: {line}"
        );
        ensure!(
            self.hello.iter().all(|&seen| seen),
            "RTIC bye preceded all hellos"
        );
        let index = self.byes;
        let hello = self
            .markers
            .iter()
            .find(|marker| marker.name == NAMES[index + 1])
            .context("missing own RTIC hello")?;
        let elapsed = at_ns
            .checked_sub(hello.at_ns)
            .context("RTIC pacing time moved backwards")?;
        let (minimum, maximum) = PACING_NS[index];
        ensure!(
            (minimum..=maximum).contains(&elapsed),
            "RTIC {} host pacing outside inclusive budget: {elapsed} ns",
            TASKS[index]
        );
        self.markers.push(Marker {
            name: NAMES[index + 4],
            at_ns,
            raw_end: self.bytes,
        });
        self.byes += 1;
        Ok(())
    }

    fn final_time(&self) -> Option<u64> {
        (self.byes == 3)
            .then(|| self.markers.last().map(|marker| marker.at_ns))
            .flatten()
    }

    fn guard_elapsed_at(&self, at_ns: u64) -> bool {
        self.final_time().is_some_and(|final_ns| {
            at_ns
                .checked_sub(final_ns)
                .is_some_and(|span| span >= GUARD_NS)
        })
    }

    fn finish(&self, end_ns: u64, receipt_events: usize) -> Result<Value> {
        ensure!(!self.rejected, "RTIC console attempt was already rejected");
        std::str::from_utf8(&self.line).context("invalid trailing RTIC UTF-8")?;
        ensure!(
            self.line.trim_ascii().is_empty(),
            "unterminated trailing RTIC text"
        );
        self.guard.finish()?;
        let final_ns = self
            .final_time()
            .context("incomplete RTIC marker sequence")?;
        let guard_ns = end_ns
            .checked_sub(final_ns)
            .context("RTIC guard time moved backwards")?;
        ensure!(
            end_ns <= CEILING_NS,
            "RTIC observation exceeded host ceiling"
        );
        ensure!(guard_ns >= GUARD_NS, "RTIC post-marker guard is too short");
        let mut pacing = serde_json::Map::new();
        for (index, &task) in TASKS.iter().enumerate() {
            let entry = self
                .markers
                .iter()
                .find(|marker| marker.name == NAMES[index + 1])
                .context("missing RTIC entry")?;
            let exit = self
                .markers
                .iter()
                .find(|marker| marker.name == NAMES[index + 4])
                .context("missing RTIC exit")?;
            pacing.insert(
                task.into(),
                json!({"observed": exit.at_ns-entry.at_ns,
                "minimum": PACING_NS[index].0, "maximum": PACING_NS[index].1}),
            );
        }
        Ok(json!({
            "console": {"format": "rtic-monotonic-v1", "application_lines": self.markers.len(),
                "hello_order": self.hello_order, "markers": self.markers, "host_pacing_ns": pacing,
                "final_marker_at_ns": final_ns, "observation_end_at_ns": end_ns,
                "guard_observed_ns": guard_ns, "raw_bytes": self.bytes, "receipt_events": receipt_events},
            "qualification": {"passed": true,
                "checks": ["single_c6_startup", "source_marker_order", "host_pacing_budgets", "post_marker_guard"],
                "scope": "Unmodified RTIC C6 monotonic example startup and timer-dependent resumption",
                "limits": ["Host receipt pacing is not device timer accuracy or interrupt latency",
                    "Priority-zero polling does not qualify positive-priority task preemption",
                    "Quiet UART does not prove liveness, sleep residency, power or CPU efficiency",
                    "One warm-reset observation does not authenticate firmware or establish independent trials"]}
        }))
    }
}

fn allowed_startup(line: &str) -> bool {
    [
        "ESP-ROM:",
        "Build:",
        "rst:",
        "Saved PC:",
        "SPIWP:",
        "mode:",
        "load:",
        "entry ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
        || line
            .strip_prefix("I (")
            .and_then(|rest| rest.split_once(") "))
            .is_some_and(|(time, message)| {
                !time.is_empty()
                    && time.bytes().all(|byte| byte.is_ascii_digit())
                    && message.split_once(": ").is_some_and(|(tag, text)| {
                        matches!(tag, "boot" | "boot.esp32c6" | "esp_image") && !text.is_empty()
                    })
            })
}

/// Reconstructs newline receipt times and checks the entire twelve-second guard.
///
/// Requires the exact saved raw/receipt bytes. A failed terminal reason cannot
/// become a success, even when all seven application markers precede it.
pub(crate) fn validate_bytes(raw: &[u8], observations: &[u8]) -> Result<Value> {
    ensure!(
        !observations.is_empty(),
        "missing RTIC observation receipts"
    );
    ensure!(
        observations.ends_with(b"\n"),
        "unterminated RTIC observation receipt"
    );
    let mut console = Console::default();
    let mut last_time = None;
    let mut raw_end = 0_usize;
    let mut events = 0_usize;
    let mut terminal = None;
    let mut last_poll = None;
    for line in observations.split_inclusive(|&byte| byte == b'\n') {
        ensure!(terminal.is_none(), "RTIC receipt after terminal event");
        ensure!(
            line.len() <= MAX_LINE_BYTES,
            "RTIC receipt exceeds line limit"
        );
        let receipt: Receipt =
            serde_json::from_slice(line).context("invalid RTIC observation receipt")?;
        let (at_ns, end) = receipt.coordinates();
        if events == 0 {
            ensure!(
                receipt
                    == Receipt::Start {
                        at_ns: 0,
                        raw_end: 0
                    },
                "RTIC receipts must start at time/offset zero"
            );
        } else {
            ensure!(
                !matches!(receipt, Receipt::Start { .. }),
                "duplicate RTIC start receipt"
            );
            let adjacent_poll_end = matches!(
                receipt,
                Receipt::End {
                    reason: EndReason::GuardElapsed,
                    ..
                }
            ) && last_poll == Some(at_ns);
            ensure!(
                last_time.is_some_and(|last| at_ns > last || (at_ns == last && adjacent_poll_end)),
                "RTIC receipt times must strictly increase except adjacent poll/end equality"
            );
        }
        ensure!(
            at_ns <= CEILING_NS,
            "RTIC observation exceeded host ceiling"
        );
        let end = usize::try_from(end).context("RTIC raw offset exceeds host usize")?;
        ensure!(end <= raw.len(), "RTIC receipt exceeds raw length");
        match receipt {
            Receipt::Start { .. } => {}
            Receipt::Read { .. } => {
                ensure!(end > raw_end, "RTIC read receipt must advance raw offset");
                console.push(&raw[raw_end..end], at_ns)?;
                last_poll = None;
            }
            Receipt::Poll { .. } | Receipt::End { .. } => {
                ensure!(end == raw_end, "RTIC poll/end receipt advanced raw offset");
                if matches!(receipt, Receipt::Poll { .. }) {
                    last_poll = Some(at_ns);
                }
            }
        }
        if let Receipt::End { reason, .. } = receipt {
            ensure!(
                reason == EndReason::GuardElapsed,
                "RTIC terminal reason is not guard_elapsed: {reason:?}"
            );
            ensure!(
                end == raw.len(),
                "RTIC terminal receipt does not cover all raw bytes"
            );
            let poll = last_poll.context("RTIC guard needs an actual final timeout poll")?;
            ensure!(
                console.guard_elapsed_at(poll),
                "RTIC final timeout poll preceded full guard interval"
            );
            terminal = Some(poll);
        }
        raw_end = end;
        last_time = Some(at_ns);
        events = events
            .checked_add(1)
            .context("RTIC receipt count overflow")?;
    }
    console.finish(terminal.context("missing RTIC terminal receipt")?, events)
}

pub(crate) fn validate_file(raw: &Path, observations: &Path) -> Result<Value> {
    validate_bytes(&fs::read(raw)?, &fs::read(observations)?)
}

fn write_receipt(mut output: impl Write, receipt: Receipt) -> Result<()> {
    serde_json::to_writer(&mut output, &receipt)?;
    output.write_all(b"\n")?;
    Ok(())
}

/// The injected clock/reader exercises actual capture without sleeping in tests.
/// Entire received chunks are written before either receipt or text admission.
fn capture_stream(
    mut input: impl Read,
    mut raw: impl Write,
    mut observations: impl Write,
    mut elapsed_ns: impl FnMut() -> Result<u64>,
) -> Result<()> {
    let mut console = Console::default();
    let mut raw_end = 0_u64;
    write_receipt(&mut observations, Receipt::Start { at_ns: 0, raw_end })?;
    let mut chunk = [0_u8; 1024];
    loop {
        let now = elapsed_ns()?;
        if now >= CEILING_NS {
            write_receipt(
                &mut observations,
                Receipt::End {
                    at_ns: now,
                    raw_end,
                    reason: EndReason::Timeout,
                },
            )?;
            bail!("RTIC capture exceeded 45-second host ceiling");
        }
        match input.read(&mut chunk) {
            Ok(0) => {
                write_receipt(
                    &mut observations,
                    Receipt::End {
                        at_ns: elapsed_ns()?,
                        raw_end,
                        reason: EndReason::Eof,
                    },
                )?;
                bail!("RTIC serial EOF before complete observation guard");
            }
            Ok(count) => {
                let at_ns = elapsed_ns()?;
                raw.write_all(&chunk[..count])
                    .context("retaining RTIC serial read")?;
                raw_end = raw_end
                    .checked_add(u64::try_from(count)?)
                    .context("RTIC capture offset overflow")?;
                write_receipt(&mut observations, Receipt::Read { at_ns, raw_end })?;
                let result = (|| {
                    ensure!(
                        at_ns < CEILING_NS,
                        "RTIC capture read exceeded host ceiling"
                    );
                    console.push(&chunk[..count], at_ns)
                })();
                if let Err(error) = result {
                    write_receipt(
                        &mut observations,
                        Receipt::End {
                            at_ns: elapsed_ns()?,
                            raw_end,
                            reason: EndReason::Rejected,
                        },
                    )?;
                    return Err(error);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {
                let at_ns = elapsed_ns()?;
                write_receipt(&mut observations, Receipt::Poll { at_ns, raw_end })?;
                if console.guard_elapsed_at(at_ns) {
                    let result = console.finish(at_ns, 0);
                    write_receipt(
                        &mut observations,
                        Receipt::End {
                            at_ns,
                            raw_end,
                            reason: if result.is_ok() {
                                EndReason::GuardElapsed
                            } else {
                                EndReason::Rejected
                            },
                        },
                    )?;
                    result?;
                    return Ok(());
                }
            }
            Err(error) => {
                write_receipt(
                    &mut observations,
                    Receipt::End {
                        at_ns: elapsed_ns()?,
                        raw_end,
                        reason: EndReason::ReadError,
                    },
                )?;
                return Err(error).context("reading RTIC target output");
            }
        }
    }
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
}

fn inspect_inputs(firmware: &[u8], provenance: &[u8]) -> Result<Value> {
    let file = object::File::parse(firmware).context("reading RTIC ELF")?;
    ensure!(
        file.format() == object::BinaryFormat::Elf
            && file.architecture() == object::Architecture::Riscv32
            && file.is_little_endian()
            && file.kind() == object::ObjectKind::Executable
            && file
                .segments()
                .any(|segment| segment.size() > 0
                    && segment.data().is_ok_and(|bytes| !bytes.is_empty())),
        "RTIC requires a loadable RISC-V32 executable ELF"
    );
    for name in NAMES {
        ensure!(
            file.segments()
                .any(|segment| segment.data().is_ok_and(|bytes| bytes
                    .windows(name.len())
                    .any(|window| window == name.as_bytes()))),
            "required RTIC marker absent from loadable ELF: {name}"
        );
    }
    let provenance: Value =
        serde_json::from_slice(provenance).context("reading RTIC build provenance")?;
    ensure!(
        provenance.is_object(),
        "RTIC build provenance must be an object"
    );
    Ok(provenance)
}

fn record_outcome(report: &mut Value, result: &Result<Value>) {
    match result {
        Ok(fields) => {
            report["qualification"] = fields["qualification"].clone();
            report["console"] = fields["console"].clone();
            report["status"] = json!("pass");
        }
        Err(error) => {
            report["status"] = json!("failed");
            report["failure"] = json!(format!("{error:#}"));
            for key in [
                "qualification",
                "console",
                "summary",
                "benchmarks",
                "comparative_score",
            ] {
                report[key] = Value::Null;
            }
        }
    }
}

/// Retains one fixed, qualification-only attempt; preflight performs no reset.
///
/// Files and serial access block on the single host thread. A disk/process
/// failure can leave incomplete evidence, which cannot count as a passing boot.
pub(crate) fn capture(config: Capture<'_>) -> Result<()> {
    ensure!(
        config.purpose == CapturePurpose::Qualification,
        "RTIC capture is qualification only; comparison is unsupported"
    );
    ensure!(
        !config.output.exists(),
        "result directory already exists; refusing overwrite"
    );
    ensure!(
        !config.boot_id.trim().is_empty(),
        "RTIC boot ID must be nonempty"
    );
    ensure!(
        (1..=3).contains(&config.run_order),
        "RTIC run order must be 1 through 3"
    );
    let firmware = fs::read(config.elf).context("reading RTIC firmware ELF")?;
    let provenance_bytes = fs::read(config.provenance).context("reading RTIC provenance")?;
    let provenance = inspect_inputs(&firmware, &provenance_bytes)?;
    let admission = Admission {
        purpose: CapturePurpose::Qualification,
        elf_sha256: digest(&firmware),
        provenance_sha256: digest(&provenance_bytes),
        method: None,
    };
    let collector = fs::read(std::env::current_exe()?)?;
    let compiler = compiler_version()?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    if let Some(parent) = config
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(config.output)?;
    fs::write(config.output.join("firmware.elf"), &firmware)?;
    fs::write(config.output.join("provenance.json"), &provenance_bytes)?;
    let mut report = json!({"schema_version": 4, "experiment": EXPERIMENT, "platform": "rtic",
        "admission": admission, "status": "running", "timestamp_unix_ms": timestamp,
        "independent_boot_id": config.boot_id, "run_order_index": config.run_order,
        "reset_kind": "RTS warm-reset request; DTR deasserted",
        "serial": {"baud": 115200, "data_bits": 8, "parity": "none", "stop_bits": 1},
        "firmware_sha256": admission.elf_sha256, "provenance": provenance,
        "serial_sha256": digest(b""), "observations_sha256": digest(b""),
        "collector_version": env!("CARGO_PKG_VERSION"), "collector_executable_sha256": digest(&collector),
        "host_rustc_at_capture": compiler, "qualification": null, "console": null,
        "summary": null, "benchmarks": null, "comparative_score": null, "failure": null,
        "limitations": ["Qualification only; no JSON benchmark protocol or comparative score",
            "Host receipt pacing does not establish device clock accuracy or interrupt latency",
            "A twelve-second quiet console guard does not prove continuing liveness",
            "Warm-reset metadata does not authenticate hardware or establish independent trials",
            "host_rustc_at_capture observes PATH, not the compiler that built this executable"]});
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    let mut result = (|| {
        let mut raw = File::create(config.output.join("serial.raw"))?;
        let mut observations = File::create(config.output.join("observations.jsonl"))?;
        let captured = (|| {
            admission.check_inputs(&fs::read(config.elf)?, &fs::read(config.provenance)?)?;
            let mut port = serialport::new(config.port, 115_200)
                .timeout(Duration::from_millis(100))
                .open()
                .context("opening RTIC serial port")?;
            port.write_data_terminal_ready(false)?;
            port.write_request_to_send(true)?;
            std::thread::sleep(Duration::from_millis(100));
            port.clear(serialport::ClearBuffer::Input)?;
            let start = Instant::now();
            port.write_request_to_send(false)?;
            capture_stream(&mut port, &mut raw, &mut observations, || {
                u64::try_from(start.elapsed().as_nanos()).context("RTIC host clock overflow")
            })
        })();
        raw.flush().context("flushing RTIC raw evidence")?;
        observations
            .flush()
            .context("flushing RTIC timing evidence")?;
        captured?;
        validate_file(
            &config.output.join("serial.raw"),
            &config.output.join("observations.jsonl"),
        )
    })();
    // Retained bytes, rather than an in-memory parser, bind the final result.
    for (name, key) in [
        ("serial.raw", "serial_sha256"),
        ("observations.jsonl", "observations_sha256"),
    ] {
        match fs::read(config.output.join(name)) {
            Ok(bytes) => report[key] = json!(digest(&bytes)),
            Err(error) if result.is_ok() => {
                result = Err(error).with_context(|| format!("hashing retained RTIC {name}"));
            }
            Err(_) => {} // Preserve the original failure; missing files cannot finalize.
        }
    }
    record_outcome(&mut report, &result);
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    result?;
    println!(
        "Validated RTIC qualification boot {}: {}",
        config.boot_id,
        config.output.display()
    );
    Ok(())
}

/// Checks retained admission and byte identities before archive replay.
pub(crate) fn verify_archive_inputs(directory: &Path, report: &Value) -> Result<()> {
    let admission: Admission = serde_json::from_value(report["admission"].clone())?;
    ensure!(
        report["schema_version"] == 4
            && report["experiment"] == EXPERIMENT
            && report["platform"] == "rtic"
            && admission.purpose == CapturePurpose::Qualification
            && admission.method.is_none()
            && report["firmware_sha256"] == admission.elf_sha256,
        "RTIC archive must preserve schema-4 qualification-only admission"
    );
    ensure!(
        ["benchmarks", "comparative_score", "summary"]
            .iter()
            .all(|key| report.get(key) == Some(&Value::Null)),
        "RTIC archive cannot contain benchmark or comparative scores"
    );
    ensure!(
        report["independent_boot_id"]
            .as_str()
            .is_some_and(|id| !id.trim().is_empty())
            && report["run_order_index"]
                .as_u64()
                .is_some_and(|order| (1..=3).contains(&order)),
        "RTIC archive has invalid boot/order metadata"
    );
    let firmware = fs::read(directory.join("firmware.elf"))?;
    let provenance = fs::read(directory.join("provenance.json"))?;
    admission.check_inputs(&firmware, &provenance)?;
    ensure!(
        report["provenance"] == inspect_inputs(&firmware, &provenance)?,
        "RTIC provenance differs from retained original"
    );
    for (name, key) in [
        ("serial.raw", "serial_sha256"),
        ("observations.jsonl", "observations_sha256"),
    ] {
        ensure!(
            report[key] == digest(&fs::read(directory.join(name))?),
            "RTIC archive {name} hash differs"
        );
    }
    match report["status"].as_str() {
        Some("pass") => ensure!(
            report.get("failure") == Some(&Value::Null)
                && report["qualification"].is_object()
                && report["console"].is_object(),
            "RTIC passing report lacks derived fields or contains failure"
        ),
        Some("failed") => ensure!(
            report.get("qualification") == Some(&Value::Null)
                && report.get("console") == Some(&Value::Null)
                && report["failure"]
                    .as_str()
                    .is_some_and(|failure| !failure.is_empty()),
            "failed RTIC report retains favorable fields or lacks failure"
        ),
        _ => bail!("RTIC archived attempt is neither pass nor failed"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn chunks(order: [usize; 3]) -> Vec<(u64, Vec<u8>)> {
        let mut entry = b"init\r\n".to_vec();
        for index in order {
            entry.extend_from_slice(format!("{}\r\n", NAMES[index + 1]).as_bytes());
        }
        vec![
            (100_000_000, b"ESP-ROM:esp32c6-20220919\r\nBuild:Sep 19 2022\r\nrst:0x1 (POWERON)\r\nSaved PC:0x0\r\nSPIWP:0xee\r\nmode:DIO, clock div:1\r\nload:0x40800000,len:4\r\nentry 0x40800000\r\nI (20) boot: fixture\r\nI (21) boot.esp32c6: fixture\r\nI (22) esp_image: fixture\r\n".to_vec()),
            (200_000_000, entry),
            (2_200_000_000, b"bye from foo\r\n".to_vec()),
            (3_200_000_000, b"bye from bar\r\n".to_vec()),
            (4_200_000_000, b"bye from baz\r\n".to_vec()),
        ]
    }

    fn encoded(receipts: &[Receipt]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for &receipt in receipts {
            write_receipt(&mut bytes, receipt).unwrap();
        }
        bytes
    }

    fn evidence(chunks: &[(u64, Vec<u8>)]) -> (Vec<u8>, Vec<Receipt>) {
        let mut raw = Vec::new();
        let mut receipts = vec![Receipt::Start {
            at_ns: 0,
            raw_end: 0,
        }];
        for (at_ns, bytes) in chunks {
            raw.extend_from_slice(bytes);
            receipts.push(Receipt::Read {
                at_ns: *at_ns,
                raw_end: raw.len() as u64,
            });
        }
        let at_ns = chunks.last().unwrap().0 + GUARD_NS;
        let raw_end = raw.len() as u64;
        receipts.push(Receipt::Poll { at_ns, raw_end });
        receipts.push(Receipt::End {
            at_ns,
            raw_end,
            reason: EndReason::GuardElapsed,
        });
        (raw, receipts)
    }

    #[derive(Debug)]
    enum Step {
        Data(u64, VecDeque<u8>),
        Error(u64, io::ErrorKind),
        Eof(u64),
    }

    #[derive(Debug)]
    struct Reader {
        steps: VecDeque<Step>,
        clock: Rc<Cell<u64>>,
        width: usize,
    }

    impl Read for Reader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let step = self
                .steps
                .front_mut()
                .expect("fake script must include a terminal observation");
            match step {
                Step::Data(at_ns, bytes) => {
                    self.clock.set((*at_ns).max(self.clock.get() + 1));
                    let count = output.len().min(bytes.len()).min(self.width);
                    for byte in &mut output[..count] {
                        *byte = bytes.pop_front().unwrap();
                    }
                    if bytes.is_empty() {
                        self.steps.pop_front();
                    }
                    Ok(count)
                }
                Step::Error(at_ns, kind) => {
                    self.clock.set((*at_ns).max(self.clock.get() + 1));
                    let error = io::Error::new(*kind, "injected transport result");
                    self.steps.pop_front();
                    Err(error)
                }
                Step::Eof(at_ns) => {
                    self.clock.set((*at_ns).max(self.clock.get() + 1));
                    self.steps.pop_front();
                    Ok(0)
                }
            }
        }
    }

    fn steps(chunks: Vec<(u64, Vec<u8>)>) -> VecDeque<Step> {
        chunks
            .into_iter()
            .map(|(at, bytes)| Step::Data(at, bytes.into()))
            .collect()
    }

    fn collect(mut script: VecDeque<Step>, width: usize) -> (Result<()>, Vec<u8>, Vec<u8>) {
        // A final empty poll is a real read outcome in this fake transport,
        // rather than a clock jump standing in for observed silence.
        if script.is_empty() {
            script.push_back(Step::Eof(1));
        }
        let clock = Rc::new(Cell::new(1));
        let reader = Reader {
            steps: script,
            clock: clock.clone(),
            width,
        };
        let mut raw = Vec::new();
        let mut observations = Vec::new();
        let result = capture_stream(reader, &mut raw, &mut observations, || {
            let now = clock.get();
            clock.set(now + 1);
            Ok(now)
        });
        (result, raw, observations)
    }

    #[test]
    fn all_entry_orders_pass_capture_and_replay_without_device_timing_claims() {
        for order in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            let (raw, receipts) = evidence(&chunks(order));
            let fields = validate_bytes(&raw, &encoded(&receipts)).unwrap();
            assert_eq!(fields["console"]["application_lines"], 7);
            assert_eq!(fields["console"]["guard_observed_ns"], GUARD_NS);
            assert!(fields.get("benchmarks").is_none());
            for width in [1, 7, 1024] {
                let mut script = steps(chunks(order));
                script.push_back(Step::Error(
                    4_200_000_000 + GUARD_NS + 10_000,
                    io::ErrorKind::TimedOut,
                ));
                let (result, saved, observations) = collect(script, width);
                result.unwrap();
                assert_eq!(saved, raw);
                let replayed = validate_bytes(&saved, &observations).unwrap();
                assert_eq!(replayed["qualification"], fields["qualification"]);
                assert!(replayed["console"]["guard_observed_ns"].as_u64().unwrap() >= GUARD_NS);
            }
        }
    }

    #[test]
    fn pacing_and_guard_bounds_are_inclusive_and_observation_ends_at_poll() {
        for boundary in [0, 1] {
            let mut input = chunks([0, 1, 2]);
            for index in 0..3 {
                input[index + 2].0 = 200_000_000
                    + if boundary == 0 {
                        PACING_NS[index].0
                    } else {
                        PACING_NS[index].1
                    };
            }
            let (raw, mut receipts) = evidence(&input);
            let fields = validate_bytes(&raw, &encoded(&receipts)).unwrap();
            assert_eq!(fields["console"]["guard_observed_ns"], GUARD_NS);
            if let Receipt::End { at_ns, .. } = receipts.last_mut().unwrap() {
                *at_ns += 3_000_000_000; // Administrative delay is not observed silence.
            }
            let late = validate_bytes(&raw, &encoded(&receipts)).unwrap();
            assert_eq!(late, fields);
            for offset in [-1_i64, 1] {
                let mut bad = input.clone();
                bad[2].0 = 200_000_000
                    + if offset < 0 {
                        PACING_NS[0].0 - 1
                    } else {
                        PACING_NS[0].1 + 1
                    };
                if offset > 0 {
                    bad[3].0 = bad[2].0 + 1; // Preserve receipt order to exercise pacing.
                }
                let (raw, receipts) = evidence(&bad);
                assert!(
                    format!(
                        "{:#}",
                        validate_bytes(&raw, &encoded(&receipts)).unwrap_err()
                    )
                    .contains("pacing")
                );
            }
        }
    }

    #[test]
    fn missing_io_witness_cannot_turn_a_host_stall_into_a_guard() {
        let (raw, mut receipts) = evidence(&chunks([0, 1, 2]));
        receipts.remove(receipts.len() - 2);
        assert!(
            format!(
                "{:#}",
                validate_bytes(&raw, &encoded(&receipts)).unwrap_err()
            )
            .contains("timeout poll")
        );
        let mut script = steps(chunks([0, 1, 2]));
        // If a host stall moves time past the guard, a pending diagnostic must
        // still be read before any successful timeout witness is possible.
        script.push_back(Step::Data(
            4_200_000_000 + GUARD_NS + 1,
            b"PANIC: pending after host stall\n".to_vec().into(),
        ));
        let clock = Rc::new(Cell::new(1));
        let reader = Reader {
            steps: script,
            clock: clock.clone(),
            width: 1024,
        };
        let mut saved = Vec::new();
        let mut observations = Vec::new();
        let result = capture_stream(reader, &mut saved, &mut observations, || {
            let now = clock.get();
            // Receipt of the final bye is stamped at4.2s. Simulate a host
            // stall immediately afterward, before the next serial read.
            clock.set(if now == 4_200_000_000 {
                4_200_000_000 + GUARD_NS + 1
            } else {
                now + 1
            });
            Ok(now)
        });
        assert!(result.is_err());
        assert!(saved.ends_with(b"PANIC: pending after host stall\n"));
        assert!(validate_bytes(&saved, &observations).is_err());
    }

    #[test]
    fn fatal_boot_and_order_failures_are_permanent_across_read_boundaries() {
        for fault in [
            "panic: bad",
            "Rebooting...",
            "ESP-ROM:esp32c6",
            "rst:0xc",
            "Exception 'illegal instruction'",
            "Stack overflow detected",
            "{\"event\":\"complete\"}",
            "unexpected application output",
        ] {
            for width in [1, 7, 1024] {
                let mut input = chunks([0, 1, 2]);
                input[0]
                    .1
                    .extend_from_slice(format!("{fault}\n").as_bytes());
                let (result, saved, observations) = collect(steps(input), width);
                assert!(result.is_err(), "{fault} width{width}");
                assert!(validate_bytes(&saved, &observations).is_err());
            }
            let mut input = chunks([0, 1, 2]);
            input.push((4_300_000_000, format!("{fault}\n").into_bytes()));
            let (raw, receipts) = evidence(&input);
            assert!(
                validate_bytes(&raw, &encoded(&receipts)).is_err(),
                "post-completion {fault}"
            );
        }
        let mut console = Console::default();
        assert!(console.push(b"ESP-ROM:esp32c6\npanic: bad\n", 1).is_err());
        assert!(console.push(b"init\n", 2).is_err());
    }

    #[test]
    fn malformed_truncated_duplicate_and_unexpected_text_do_not_pass() {
        for change in 0..9 {
            let mut input = chunks([0, 1, 2]);
            match change {
                0 => input[0].1.extend_from_slice(b"ordinary startup noise\n"),
                1 => input[0]
                    .1
                    .extend_from_slice(b"I (12) application: unapproved tag\n"),
                2 => input[1].1.extend_from_slice(b"init\n"),
                3 => {
                    input[1].1 = b"init\nhello from foo\nhello from foo\nhello from baz\n".to_vec()
                }
                4 => input[2].1 = b"bye from bar\n".to_vec(),
                5 => input[1].1 = b"init\nhello from foo\nhello from bar\n".to_vec(),
                6 => input[4].1 = b"bye from baz".to_vec(),
                7 => input[0].1.extend_from_slice(b"\xff\n"),
                8 => input[4].1.extend_from_slice(b"partial trailing failure"),
                _ => unreachable!(),
            }
            let (raw, receipts) = evidence(&input);
            assert!(
                validate_bytes(&raw, &encoded(&receipts)).is_err(),
                "mutation{change}"
            );
        }
        let mut input = chunks([0, 1, 2]);
        input[0]
            .1
            .extend_from_slice(&vec![b'x'; MAX_LINE_BYTES + 1]);
        input[0].1.push(b'\n');
        let (raw, receipts) = evidence(&input);
        assert!(
            format!(
                "{:#}",
                validate_bytes(&raw, &encoded(&receipts)).unwrap_err()
            )
            .contains("line exceeds")
        );
    }

    #[test]
    fn entire_faulty_read_chunk_is_retained_before_admission() {
        let input = b"ESP-ROM:esp32c6\npanic: failure\ninit\nhello from foo\n".to_vec();
        let script = steps(vec![(1_000, input.clone())]);
        let (result, saved, observations) = collect(script, 1024);
        assert!(result.is_err());
        assert_eq!(saved, input);
        let receipts: Vec<Receipt> = observations
            .split_inclusive(|&byte| byte == b'\n')
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect();
        assert!(matches!(
            receipts.last(),
            Some(Receipt::End {
                reason: EndReason::Rejected,
                ..
            })
        ));
    }

    #[test]
    fn eof_read_error_interrupt_and_timeout_after_markers_remain_failures() {
        for terminal in [
            Step::Eof(4_300_000_000),
            Step::Error(4_300_000_000, io::ErrorKind::Other),
            Step::Error(4_300_000_000 + GUARD_NS, io::ErrorKind::Interrupted),
            Step::Error(CEILING_NS + 1, io::ErrorKind::TimedOut),
        ] {
            let mut script = steps(chunks([0, 1, 2]));
            script.push_back(terminal);
            let (result, raw, observations) = collect(script, 1024);
            assert!(result.is_err());
            assert!(validate_bytes(&raw, &observations).is_err());
        }
    }

    #[test]
    fn receipt_types_offsets_order_and_finality_are_strict() {
        let (raw, original) = evidence(&chunks([0, 1, 2]));
        for change in 0..10 {
            let mut receipts = original.clone();
            match change {
                0 => {
                    receipts.remove(0);
                }
                1 => {
                    receipts.pop();
                }
                2 => {
                    receipts[1] = Receipt::Read {
                        at_ns: 0,
                        raw_end: 1,
                    }
                }
                3 => {
                    receipts[1] = Receipt::Poll {
                        at_ns: 1,
                        raw_end: 1,
                    }
                }
                4 => {
                    receipts[2] = Receipt::Read {
                        at_ns: 1,
                        raw_end: raw.len() as u64 + 1,
                    }
                }
                5 => {
                    receipts[2] = Receipt::Read {
                        at_ns: 1,
                        raw_end: 0,
                    }
                }
                6 => {
                    receipts.push(*receipts.last().unwrap());
                }
                7 => {
                    receipts[1] = Receipt::Start {
                        at_ns: 1,
                        raw_end: 0,
                    }
                }
                8 => {
                    *receipts.last_mut().unwrap() = Receipt::End {
                        at_ns: 4_300_000_000 + GUARD_NS,
                        raw_end: raw.len() as u64,
                        reason: EndReason::ReadError,
                    }
                }
                9 => {
                    receipts[6] = Receipt::Poll {
                        at_ns: 4_200_000_000 + GUARD_NS - 1,
                        raw_end: raw.len() as u64,
                    }
                }
                _ => unreachable!(),
            }
            assert!(
                validate_bytes(&raw, &encoded(&receipts)).is_err(),
                "receipt mutation{change}"
            );
        }
        for bytes in [
            b"{\"event\":\"start\",\"at_ns\":0,\"raw_end\":0,\"extra\":1}\n".as_slice(),
            b"{\"event\":\"start\",\"at_ns\":0.0,\"raw_end\":0}\n",
            b"{\"event\":\"start\",\"at_ns\":0,\"raw_end\":-1}\n",
            b"{\"event\":\"start\",\"at_ns\":0,\"raw_end\":0}",
        ] {
            assert!(validate_bytes(&raw, bytes).is_err());
        }
    }

    /// A synthetic loadable ELF container for host preflight tests only. It has
    /// marker bytes, not executable firmware and is never sent to a board.
    fn elf() -> Vec<u8> {
        let payload = NAMES.join("\n").into_bytes();
        let mut bytes = vec![0_u8; 84];
        bytes[..7].copy_from_slice(b"\x7fELF\x01\x01\x01");
        bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
        bytes[18..20].copy_from_slice(&243_u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&0x4080_0000_u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&52_u32.to_le_bytes());
        bytes[40..42].copy_from_slice(&52_u16.to_le_bytes());
        bytes[42..44].copy_from_slice(&32_u16.to_le_bytes());
        bytes[44..46].copy_from_slice(&1_u16.to_le_bytes());
        for (offset, value) in [
            (52, 1_u32),
            (56, 84),
            (60, 0x4080_0000),
            (64, 0x4080_0000),
            (68, payload.len() as u32),
            (72, payload.len() as u32),
            (76, 5),
            (80, 4),
        ] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&payload);
        bytes
    }

    static TEMPORARY_ID: AtomicU64 = AtomicU64::new(0);

    #[derive(Debug)]
    struct Temporary(PathBuf);

    impl Temporary {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "esp32-c6-rtic-host-{}-{}",
                std::process::id(),
                TEMPORARY_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn inputs(&self) -> (PathBuf, PathBuf) {
            let firmware = self.0.join("input.elf");
            let provenance = self.0.join("input-provenance.json");
            fs::write(&firmware, elf()).unwrap();
            fs::write(&provenance, b"{\"fixture_only\":true}\n").unwrap();
            (firmware, provenance)
        }
    }

    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn invalid_preflight_never_creates_output_or_opens_port() {
        let temporary = Temporary::new();
        let (firmware, provenance) = temporary.inputs();
        for fault in 0..5 {
            let output = temporary.0.join(format!("rejected{fault}"));
            if fault == 1 {
                fs::write(&firmware, b"not ELF").unwrap();
            }
            if fault == 2 {
                fs::write(&firmware, elf()).unwrap();
                fs::write(&provenance, b"[]").unwrap();
            }
            if fault == 3 {
                fs::write(&provenance, b"{}").unwrap();
            }
            let error = capture(Capture {
                purpose: if fault == 0 {
                    CapturePurpose::Comparison
                } else {
                    CapturePurpose::Qualification
                },
                port: "/does/not/exist/rtic-fixture",
                output: &output,
                elf: &firmware,
                provenance: &provenance,
                boot_id: if fault == 3 { "" } else { "fixture" },
                run_order: if fault == 4 { 4 } else { 1 },
            })
            .unwrap_err();
            assert!(!format!("{error:#}").contains("opening RTIC serial port"));
            assert!(!output.exists());
        }
        let mut wrong_machine = elf();
        wrong_machine[18..20].copy_from_slice(&62_u16.to_le_bytes());
        assert!(inspect_inputs(&wrong_machine, b"{}").is_err());
        let mut missing_marker = elf();
        let index = missing_marker
            .windows(NAMES[6].len())
            .position(|bytes| bytes == NAMES[6].as_bytes())
            .unwrap();
        missing_marker[index] = b'x';
        assert!(
            format!("{:#}", inspect_inputs(&missing_marker, b"{}").unwrap_err())
                .contains("marker absent")
        );
    }

    #[test]
    fn failed_port_attempt_retains_input_hashes_and_cannot_be_promoted() {
        let temporary = Temporary::new();
        let (firmware, provenance) = temporary.inputs();
        let output = temporary.0.join("failed");
        let error = capture(Capture {
            purpose: CapturePurpose::Qualification,
            port: "/does/not/exist/rtic-fixture",
            output: &output,
            elf: &firmware,
            provenance: &provenance,
            boot_id: "failed-fixture",
            run_order: 1,
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains("opening RTIC serial port"));
        let mut report: Value =
            serde_json::from_slice(&fs::read(output.join("run.json")).unwrap()).unwrap();
        assert_eq!(report["status"], "failed");
        verify_archive_inputs(&output, &report).unwrap();
        for key in [
            "qualification",
            "console",
            "summary",
            "benchmarks",
            "comparative_score",
        ] {
            assert!(report[key].is_null());
            let original = report.clone();
            report[key] = json!({"claimed_pass": true});
            assert!(verify_archive_inputs(&output, &report).is_err());
            report = original.clone();
            report.as_object_mut().unwrap().remove(key);
            assert!(verify_archive_inputs(&output, &report).is_err());
            report = original;
        }
        fs::write(output.join("serial.raw"), b"changed").unwrap();
        assert!(verify_archive_inputs(&output, &report).is_err());
        assert!(
            validate_file(
                &output.join("serial.raw"),
                &output.join("observations.jsonl")
            )
            .is_err()
        );
    }

    #[test]
    fn archive_identity_checks_reject_retagged_admission_and_changed_inputs() {
        let temporary = Temporary::new();
        let (firmware, provenance) = temporary.inputs();
        let (raw, receipts) = evidence(&chunks([0, 1, 2]));
        let observation = encoded(&receipts);
        fs::copy(&firmware, temporary.0.join("firmware.elf")).unwrap();
        fs::copy(&provenance, temporary.0.join("provenance.json")).unwrap();
        fs::write(temporary.0.join("serial.raw"), &raw).unwrap();
        fs::write(temporary.0.join("observations.jsonl"), &observation).unwrap();
        let fields = validate_bytes(&raw, &observation).unwrap();
        let original = json!({"schema_version": 4, "experiment": EXPERIMENT, "platform": "rtic", "status": "pass",
            "admission": {"purpose": "qualification", "elf_sha256": digest(&elf()), "provenance_sha256": digest(&fs::read(provenance).unwrap()), "method": null},
            "firmware_sha256": digest(&elf()), "provenance": {"fixture_only": true},
            "serial_sha256": digest(&raw), "observations_sha256": digest(&observation), "independent_boot_id": "fixture", "run_order_index": 1,
            "qualification": fields["qualification"], "console": fields["console"], "failure": null, "summary": null, "benchmarks": null, "comparative_score": null});
        verify_archive_inputs(&temporary.0, &original).unwrap();
        for fault in 0..5 {
            let mut report = original.clone();
            match fault {
                0 => report["admission"]["purpose"] = json!("comparison"),
                1 => report["provenance"] = json!({"different": true}),
                2 => report["observations_sha256"] = json!(digest(b"changed")),
                3 => report["status"] = json!("running"),
                4 => report["failure"] = json!("contradictory failure"),
                _ => unreachable!(),
            }
            assert!(verify_archive_inputs(&temporary.0, &report).is_err());
        }
        fs::write(temporary.0.join("firmware.elf"), b"changed").unwrap();
        assert!(verify_archive_inputs(&temporary.0, &original).is_err());
    }
}
