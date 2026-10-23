//! Host-side capture, validation, and ELF inspection for the initial C6 trials.
//!
//! This binary has no public Rust library API. Its single host thread performs
//! blocking file/serial I/O and allocates outside all device timing regions.
//! Protocol lines are bounded; raw bytes and failed captures remain on disk.
//! Protocols 1/2 require 64 compute/timer observations; version 2 scheduler
//! adapters additionally require 1,000 validated message roundtrips. Protocol 3
//! handles queue-path comparisons; protocols 4/5 qualify accounting/finite load.

#![forbid(unsafe_code)]

mod accounting;
mod assurance;
mod boot_text;
mod load;
mod protocol;
mod qualification;
mod queue_protocol;
mod queue_study;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use object::{Object, ObjectSection};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::protocol::{Boot, Record};
use crate::qualification::{Admission, CapturePurpose, Wire};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Capture finite offered-load protocol 5 for qualification only.
    CaptureLoad {
        #[arg(long, value_enum)]
        purpose: CapturePurpose,
        #[arg(long, value_enum)]
        profile: load::Profile,
        #[arg(long)]
        port: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        elf: PathBuf,
        #[arg(long)]
        provenance: PathBuf,
        #[arg(long)]
        boot_id: String,
        #[arg(long)]
        run_order: u32,
        #[arg(long, default_value_t = 120)]
        timeout_seconds: u64,
    },
    /// Replay finite-load qualification without touching a device.
    ValidateLoad {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, value_enum)]
        profile: load::Profile,
    },
    /// Capture protocol 4 accounting controls for qualification only.
    CaptureAccounting {
        #[arg(long, value_enum)]
        purpose: CapturePurpose,
        #[arg(long)]
        port: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        elf: PathBuf,
        #[arg(long)]
        provenance: PathBuf,
        #[arg(long)]
        boot_id: String,
        #[arg(long)]
        run_order: u32,
        #[arg(long, default_value_t = 30)]
        timeout_seconds: u64,
    },
    /// Replay accounting qualification controls without touching a device.
    ValidateAccounting {
        #[arg(long)]
        input: PathBuf,
    },
    /// Verify archive identities, evidence classifications and local references.
    VerifyEvidence {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value = "quality/evidence.json")]
        ledger: PathBuf,
    },
    /// Execute a method's artifact, positive and negative controls offline.
    QualifyMethod {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        method: PathBuf,
    },
    /// Reproduce archived protocol/study outputs without changing any result.
    ReplayArchives {
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Capture the protocol 3 raw-tick queue comparison, retaining failed trials.
    CaptureQueue {
        #[arg(long, value_enum)]
        purpose: CapturePurpose,
        #[arg(long)]
        method: Option<PathBuf>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        port: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        backend: String,
        #[arg(long)]
        elf: PathBuf,
        #[arg(long)]
        provenance: PathBuf,
        #[arg(long)]
        boot_id: String,
        #[arg(long)]
        run_order: u32,
        #[arg(long, default_value_t = 180)]
        timeout_seconds: u64,
        /// Previously observed device nonces; repeat this flag for all prior boots.
        #[arg(long)]
        previous_nonce: Vec<u64>,
    },
    /// Replay protocol 3 raw-tick evidence without touching a device.
    ValidateQueue {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        backend: String,
        #[arg(long)]
        previous_nonce: Vec<u64>,
    },
    /// Freeze twelve balanced paired blocks before collecting queue results.
    QueueSchedule {
        #[arg(long)]
        seed: u64,
        #[arg(long)]
        output: PathBuf,
    },
    /// Analyze a frozen queue schedule with archived raw validation and failures.
    AnalyzeQueue {
        #[arg(long)]
        schedule: PathBuf,
        #[arg(long)]
        results: PathBuf,
    },
    /// Reset the connected board and preserve one complete, validated boot.
    Capture {
        #[arg(long, value_enum)]
        purpose: CapturePurpose,
        #[arg(long)]
        method: Option<PathBuf>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        port: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        platform: String,
        #[arg(long)]
        elf: PathBuf,
        /// JSON build provenance, copied into the result.
        #[arg(long)]
        provenance: PathBuf,
        #[arg(long)]
        boot_id: String,
        #[arg(long)]
        run_order: u32,
        #[arg(long, default_value_t = 30)]
        timeout_seconds: u64,
    },
    /// Validate an archived transcript without touching a device.
    Validate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        platform: String,
    },
    /// List allocated ELF sections without calling ELF file size flash usage.
    Inspect {
        #[arg(long)]
        elf: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Action::CaptureLoad {
            purpose,
            profile,
            port,
            output,
            elf,
            provenance,
            boot_id,
            run_order,
            timeout_seconds,
        } => load::capture(load::Capture {
            purpose,
            profile,
            port: &port,
            output: &output,
            elf: &elf,
            provenance: &provenance,
            boot_id: &boot_id,
            run_order,
            timeout: Duration::from_secs(timeout_seconds),
        }),
        Action::ValidateLoad { input, profile } => {
            write_json(io::stdout().lock(), &load::validate_file(&input, profile)?)
        }
        Action::CaptureAccounting {
            purpose,
            port,
            output,
            elf,
            provenance,
            boot_id,
            run_order,
            timeout_seconds,
        } => accounting::capture(accounting::Capture {
            purpose,
            port: &port,
            output: &output,
            elf: &elf,
            provenance: &provenance,
            boot_id: &boot_id,
            run_order,
            timeout: Duration::from_secs(timeout_seconds),
        }),
        Action::ValidateAccounting { input } => {
            write_json(io::stdout().lock(), &accounting::validate_file(&input)?)
        }
        Action::VerifyEvidence { root, ledger } => write_json(
            io::stdout().lock(),
            &assurance::verify_repository(&root, &ledger)?,
        ),
        Action::QualifyMethod { root, method } => write_json(
            io::stdout().lock(),
            &qualification::qualify(&root, &method)?,
        ),
        Action::ReplayArchives { root } => {
            write_json(io::stdout().lock(), &qualification::replay_archives(&root)?)
        }
        Action::CaptureQueue {
            purpose,
            method,
            root,
            port,
            output,
            backend,
            elf,
            provenance,
            boot_id,
            run_order,
            timeout_seconds,
            previous_nonce,
        } => {
            let admission = qualification::admit(
                &root,
                purpose,
                method.as_deref(),
                Wire::Queue,
                &backend,
                &elf,
                &provenance,
            )?;
            queue_protocol::capture(queue_protocol::Capture {
                admission: &admission,
                port: &port,
                output: &output,
                backend: &backend,
                elf: &elf,
                provenance: &provenance,
                boot_id: &boot_id,
                run_order,
                timeout: Duration::from_secs(timeout_seconds),
                previous_nonces: &previous_nonce,
            })
        }
        Action::ValidateQueue {
            input,
            backend,
            previous_nonce,
        } => write_json(
            io::stdout().lock(),
            &queue_protocol::validate_file(&input, &backend, &previous_nonce)?,
        ),
        Action::QueueSchedule { seed, output } => queue_study::write_schedule(seed, &output),
        Action::AnalyzeQueue { schedule, results } => write_json(
            io::stdout().lock(),
            &queue_study::analyze(&schedule, &results)?,
        ),
        Action::Capture {
            purpose,
            method,
            root,
            port,
            output,
            platform,
            elf,
            provenance,
            boot_id,
            run_order,
            timeout_seconds,
        } => {
            let admission = qualification::admit(
                &root,
                purpose,
                method.as_deref(),
                Wire::Legacy,
                &platform,
                &elf,
                &provenance,
            )?;
            capture(Capture {
                admission: &admission,
                port: &port,
                output: &output,
                platform: &platform,
                elf: &elf,
                provenance: &provenance,
                boot_id: &boot_id,
                run_order,
                timeout: Duration::from_secs(timeout_seconds),
            })
        }
        Action::Validate { input, platform } => {
            let boot = validate(BufReader::new(File::open(input)?), &platform)?;
            write_json(
                io::stdout().lock(),
                &json!({"hello": boot.hello(), "benchmarks": boot.summaries()?}),
            )
        }
        Action::Inspect { elf, output } => {
            let report = inspect(&elf)?;
            match output {
                Some(path) => write_json(File::create(path)?, &report),
                None => write_json(io::stdout().lock(), &report),
            }
        }
    }
}

#[derive(Debug)]
struct Capture<'a> {
    admission: &'a Admission,
    port: &'a str,
    output: &'a Path,
    platform: &'a str,
    elf: &'a Path,
    provenance: &'a Path,
    boot_id: &'a str,
    run_order: u32,
    timeout: Duration,
}

/// Captures one requested reset without overwriting existing trial evidence.
///
/// Returns an error for invalid paths/provenance, failed I/O or reset requests,
/// timeout, or a rejected protocol. A started trial keeps partial evidence and
/// is marked failed when its final report can be written.
fn capture(config: Capture<'_>) -> Result<()> {
    ensure!(
        !config.timeout.is_zero(),
        "capture timeout must be positive"
    );
    ensure!(
        !config.output.exists(),
        "result directory already exists; refusing to overwrite it"
    );
    // Check artifacts before changing the target's reset lines.
    let firmware = fs::read(config.elf).context("reading firmware ELF")?;
    let provenance_bytes = fs::read(config.provenance)?;
    config
        .admission
        .check_inputs(&firmware, &provenance_bytes)?;
    let provenance: Value = serde_json::from_slice(&provenance_bytes)?;
    ensure!(
        provenance.is_object(),
        "build provenance must be a JSON object"
    );
    let collector = fs::read(std::env::current_exe()?).context("reading collector executable")?;
    if let Some(parent) = config.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(config.output)?;
    fs::write(config.output.join("firmware.elf"), &firmware)?;
    let mut report = json!({
        "schema_version": 4,
        "admission": config.admission,
        "status": "running",
        "platform": config.platform,
        "timestamp_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        "independent_boot_id": config.boot_id,
        "run_order_index": config.run_order,
        "reset_kind": "RTS warm-reset request; DTR deasserted",
        "serial": {"baud": 115200, "data_bits": 8, "parity": "none", "stop_bits": 1},
        "firmware_sha256": digest(&firmware),
        "collector_version": env!("CARGO_PKG_VERSION"),
        "collector_executable_sha256": digest(&collector),
        "host_rustc_at_capture": compiler_version()?,
        "provenance": provenance,
        "hello": null,
        "benchmarks": null,
        "failure": null,
        "limitations": [
            "Warm reset only; no externally referenced boot timing",
            "Internal microsecond timer; no external IRQ or power instrument",
            "64 samples per benchmark in each boot; nearest-rank percentiles",
            "Compute includes timer observer overhead; no subtraction",
            "host_rustc_at_capture observes PATH at collection time, not the compiler used to build this executable"
        ]
    });
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    let mut raw = File::create(config.output.join("serial.raw"))?;
    let mut records = File::create(config.output.join("records.jsonl"))?;
    let mut boot = Boot::default();
    let result = capture_boot(&config, &mut raw, &mut records, &mut boot);
    raw.flush()?;
    records.flush()?;
    report["hello"] = serde_json::to_value(boot.hello())?;
    if let Some(clocks_after) = boot.clocks_after() {
        report["clocks_after"] = serde_json::to_value(clocks_after)?;
        report["limitations"][2] = json!(
            "64 compute/calibration observations per boot; scheduler adapters additionally record 1000 message roundtrips; nearest-rank percentiles"
        );
        report["limitations"].as_array_mut().context("internal limitations must be an array")?.push(json!("Clock evidence decodes register readback before/after timing; no external frequency measurement"));
    }
    match &result {
        Ok(()) => {
            report["status"] = json!("pass");
            report["benchmarks"] = serde_json::to_value(boot.summaries()?)?;
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
        config.platform,
        config.boot_id,
        config.output.display()
    );
    Ok(())
}

/// Blocks while requesting reset and consuming the first complete boot.
///
/// Reset, serial, parse, validation and output errors abort the capture; they
/// never resume at a later hello record. This function uses no async context.
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
    capture_stream(
        &mut port,
        raw,
        records,
        boot,
        config.platform,
        config.timeout,
    )
}

/// Uses the actual capture reader path without coupling framing to port setup.
fn capture_stream(
    input: &mut impl Read,
    raw: &mut impl Write,
    records: &mut impl Write,
    boot: &mut Boot,
    platform: &str,
    timeout: Duration,
) -> Result<()> {
    let result = boot_text::capture(input, raw, timeout, |line| {
        if let Some(record) = parse_record(line)? {
            // Preserve parseable records even when protocol validation rejects one.
            serde_json::to_writer(&mut *records, &record)?;
            records.write_all(b"\n")?;
            boot.accept(record, platform)
        } else {
            Ok(false)
        }
    });
    if result.is_err() {
        boot.reject();
    }
    result
}

fn parse_record(line: &[u8]) -> Result<Option<Record>> {
    let trimmed = line.trim_ascii();
    if !trimmed.starts_with(b"{") {
        return Ok(None);
    }
    serde_json::from_slice(trimmed)
        .context("malformed protocol record")
        .map(Some)
}

/// Validates an entire archived transcript without touching a device.
///
/// Malformed, overlong, inconsistent, incomplete or trailing protocol records
/// return errors. Ordinary startup text is admitted only within one guarded boot.
fn validate(input: impl BufRead, platform: &str) -> Result<Boot> {
    let mut boot = Boot::default();
    boot_text::replay(input, |line| {
        if let Some(record) = parse_record(line)? {
            boot.accept(record, platform)
        } else {
            Ok(false)
        }
    })?;
    boot.summaries()?;
    Ok(boot)
}

/// Reports allocated ELF sections without equating them to peak RAM or flash.
///
/// Returns I/O, malformed/unsupported input, name decoding or size-overflow
/// errors instead of producing a partial footprint total.
fn inspect(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    let object = object::File::parse(bytes.as_slice()).context("parsing ELF")?;
    ensure!(
        object.format() == object::BinaryFormat::Elf,
        "input is not ELF"
    );
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    let mut sections = Vec::new();
    for section in object.sections() {
        let object::SectionFlags::Elf { sh_flags, .. } = section.flags() else {
            continue;
        };
        if !sh_flags.contains(object::elf::SHF_ALLOC) {
            continue;
        }
        let kind = format!("{:?}", section.kind());
        let total = totals.entry(kind.clone()).or_default();
        *total = total
            .checked_add(section.size())
            .context("allocated section total overflows u64")?;
        sections.push(json!({
            "name": section.name()?,
            "kind": kind,
            "address": section.address(),
            "size_bytes": section.size()
        }));
    }
    Ok(json!({
        "elf_sha256": digest(&bytes),
        "elf_file_bytes": bytes.len(),
        "allocated_section_bytes_by_kind": totals,
        "allocated_sections": sections,
        "limitations": "ELF file bytes include non-flashed data; sections are not peak RAM or complete flash-image footprint."
    }))
}

/// Observes the current PATH-selected rustc, not this executable's build compiler.
fn compiler_version() -> Result<String> {
    let result = Command::new("rustc").arg("-Vv").output()?;
    ensure!(
        result.status.success(),
        "cannot record collector compiler version"
    );
    Ok(String::from_utf8(result.stdout)?)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_json(mut output: impl Write, value: &Value) -> Result<()> {
    serde_json::to_writer_pretty(&mut output, value)?;
    output.write_all(b"\n")?;
    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boot_text::MAX_LINE_BYTES;

    #[test]
    fn preserves_boot_noise_but_rejects_corrupt_json() {
        assert!(parse_record(b"ESP-ROM:esp32c6\r\n").unwrap().is_none());
        assert!(parse_record(b"{broken\n").is_err());
        assert!(validate(io::Cursor::new(b"boot noise\n"), "test").is_err());
    }

    #[test]
    fn rejects_unbounded_archived_lines() {
        let input = vec![b'x'; 8192];
        assert!(validate(io::Cursor::new(input), "test").is_err());
    }

    fn complete_transcript() -> Vec<u8> {
        use bench_core::{
            DEFAULT_CHECKSUM, DEFAULT_ROUNDS, DEFAULT_SAMPLES, DEFAULT_SEED, WORKLOAD_ID,
        };

        let mut output = b"ESP-ROM:esp32c6\r\n".to_vec();
        let hello = json!({
            "event": "hello", "protocol": 1, "platform": "test", "cpu_mhz": 160,
            "sample_count": DEFAULT_SAMPLES, "seed": DEFAULT_SEED, "rounds": DEFAULT_ROUNDS,
            "workload": WORKLOAD_ID, "warmup_iterations": 8, "timer_resolution_ns": 1_000
        });
        serde_json::to_writer(&mut output, &hello).unwrap();
        output.extend_from_slice(b"\r\n");
        for benchmark in ["compute", "timer-overhead"] {
            for index in 0..DEFAULT_SAMPLES {
                let (iterations, elapsed_ns, checksum) = if benchmark == "compute" {
                    (
                        DEFAULT_ROUNDS,
                        u64::from(index + 1) * 1_000,
                        DEFAULT_CHECKSUM,
                    )
                } else {
                    (0, 0, 0)
                };
                let sample = json!({
                    "event": "sample", "protocol": 1, "benchmark": benchmark,
                    "index": index, "iterations": iterations, "elapsed_ns": elapsed_ns,
                    "checksum": checksum
                });
                serde_json::to_writer(&mut output, &sample).unwrap();
                output.push(b'\n');
            }
        }
        output.extend_from_slice(b"{\"event\":\"complete\",\"protocol\":1}\n");
        output
    }

    #[test]
    fn archived_transcript_validates_every_record_before_reporting() {
        let transcript = complete_transcript();
        let boot = validate(io::Cursor::new(&transcript), "test").unwrap();
        let summary = serde_json::to_value(boot.summaries().unwrap()).unwrap();
        assert_eq!(summary["compute"]["p95_ns"], 61_000);
        assert_eq!(summary["timer-overhead"]["mean_ns"], 0.0);
        assert!(validate(io::Cursor::new(&transcript), "wrong-platform").is_err());

        let mut trailing_reboot = transcript;
        trailing_reboot.extend_from_slice(b"{\"event\":\"complete\",\"protocol\":1}\n");
        assert!(validate(io::Cursor::new(trailing_reboot), "test").is_err());
    }

    #[test]
    fn archived_line_limit_is_the_same_with_and_without_newline() {
        let mut maximum_noise = vec![b'x'; MAX_LINE_BYTES];
        maximum_noise.push(b'\n');
        maximum_noise.extend_from_slice(&complete_transcript());
        assert!(validate(io::Cursor::new(maximum_noise), "test").is_ok());

        for newline in [false, true] {
            let mut overlong = complete_transcript();
            overlong.extend(std::iter::repeat_n(b'x', MAX_LINE_BYTES + 1));
            if newline {
                overlong.push(b'\n');
            }
            assert!(validate(io::Cursor::new(overlong), "test").is_err());
        }
    }

    #[test]
    fn legacy_capture_and_replay_reject_hidden_failures_before_or_after_json() {
        let valid = complete_transcript();
        let hello = valid.iter().position(|&byte| byte == b'{').unwrap();
        for fault in [
            b"Guru Meditation Error: Core 0 panic'ed\n".as_slice(),
            b"Rebooting...\n".as_slice(),
            b"ESP-ROM:esp32c6-20220919\n".as_slice(),
        ] {
            let bytes = [&valid[..hello], fault, &valid[hello..]].concat();
            assert!(validate(io::Cursor::new(&bytes), "test").is_err());
            let mut boot = Boot::default();
            let mut raw = Vec::new();
            let mut records = Vec::new();
            assert!(
                capture_stream(
                    &mut io::Cursor::new(&bytes),
                    &mut raw,
                    &mut records,
                    &mut boot,
                    "test",
                    Duration::from_secs(1)
                )
                .is_err()
            );
            assert!(!raw.is_empty());
            assert!(boot.summaries().is_err());
        }
        let mut boot = Boot::default();
        capture_stream(
            &mut io::Cursor::new(&valid),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut boot,
            "test",
            Duration::from_secs(1),
        )
        .unwrap();
        assert!(boot.summaries().is_ok());

        // Place completion and its already-received diagnostic in one final
        // chunk; future bytes after a completed capture returns are out of scope.
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
                "test",
                Duration::from_secs(1)
            )
            .is_err()
        );
        assert!(
            boot.summaries().is_err(),
            "text failure must revoke a previously complete Boot"
        );
        assert!(validate(io::Cursor::new(bytes), "test").is_err());
    }

    #[test]
    fn legal_json_diagnostic_metadata_is_not_a_fatal_console_line() {
        let original = complete_transcript();
        let mut bytes = Vec::new();
        for line in original.split_inclusive(|&byte| byte == b'\n') {
            if line.starts_with(b"{") {
                let mut record: Value = serde_json::from_slice(line).unwrap();
                if record["event"] == "hello" {
                    record["panic"] = json!("abort");
                    record["diagnostic_note"] = json!("Rebooting and panic are words in metadata");
                }
                serde_json::to_writer(&mut bytes, &record).unwrap();
                bytes.push(b'\n');
            } else {
                bytes.extend_from_slice(line);
            }
        }
        assert!(validate(io::Cursor::new(&bytes), "test").is_ok());
        let mut boot = Boot::default();
        capture_stream(
            &mut io::Cursor::new(bytes),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut boot,
            "test",
            Duration::from_secs(1),
        )
        .unwrap();
        assert!(boot.summaries().is_ok());
    }
}
