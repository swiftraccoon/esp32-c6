//! Freeze a balanced collection schedule and analyze validated paired boots.
//!
//! This is a single-board characterization. The optional order-statistic
//! interval assumes independent, identically distributed block contrasts;
//! timestamps, balancing and nonce uniqueness do not prove that assumption.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const EXPERIMENT: &str = "idf-native-queue-v1";
const ALGORITHM: &str = "splitmix64-fisher-yates-balanced-v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Backend {
    #[serde(rename = "rust-std-sync-channel")]
    Std,
    #[serde(rename = "freertos-queue")]
    Native,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Self::Std => "rust-std-sync-channel",
            Self::Native => "freertos-queue",
        }
    }

    fn short_name(self) -> &'static str {
        match self {
            Self::Std => "std",
            Self::Native => "native",
        }
    }

    fn other(self) -> Self {
        match self {
            Self::Std => Self::Native,
            Self::Native => Self::Std,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Schedule {
    schema_version: u32,
    experiment: String,
    algorithm: String,
    seed: u64,
    blocks: Vec<Block>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Block {
    block: u32,
    session: u32,
    runs: [ScheduledRun; 2],
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ScheduledRun {
    run_order: u32,
    backend: Backend,
    directory: String,
}

/// A reproducible permutation generator; it makes no cryptographic promise.
#[derive(Debug)]
struct Permutation(u64);

impl Permutation {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, exclusive_upper: u64) -> usize {
        // Rejection sampling avoids modulo bias. Callers only request 2..=4,
        // so the accepted result always fits usize on supported host targets.
        let threshold = exclusive_upper.wrapping_neg() % exclusive_upper;
        loop {
            let value = self.next();
            if value >= threshold {
                return (value % exclusive_upper) as usize;
            }
        }
    }
}

impl Schedule {
    fn generate(seed: u64) -> Self {
        let mut permutation = Permutation(seed);
        let mut blocks = Vec::with_capacity(12);
        for session in 1..=3 {
            let mut first = [Backend::Std, Backend::Std, Backend::Native, Backend::Native];
            for index in (1..first.len()).rev() {
                let selected = permutation.index((index + 1) as u64);
                first.swap(index, selected);
            }
            for backend in first {
                let block = blocks.len() as u32 + 1;
                let run = |backend: Backend, position: u32| ScheduledRun {
                    run_order: (block - 1) * 2 + position,
                    backend,
                    directory: format!(
                        "runs/s{session}-b{block:02}-{position:02}-{}",
                        backend.short_name()
                    ),
                };
                blocks.push(Block {
                    block,
                    session,
                    runs: [run(backend, 1), run(backend.other(), 2)],
                });
            }
        }
        Self {
            schema_version: 1,
            experiment: EXPERIMENT.into(),
            algorithm: ALGORITHM.into(),
            seed,
            blocks,
        }
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            *self == Self::generate(self.seed),
            "schedule differs from its seeded, balanced contract"
        );
        Ok(())
    }
}

/// Writes a new frozen schedule without replacing earlier experimental plans.
///
/// Returns errors for directory creation, an existing destination, serialization
/// or I/O failures. Collection must also freeze its image/provenance manifest.
pub(crate) fn write_schedule(seed: u64, output: &Path) -> Result<()> {
    let data = serde_json::to_vec_pretty(&Schedule::generate(seed))?;
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(&data)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

#[derive(Debug)]
struct ValidatedRun {
    nonce: u64,
    median: u64,
    configuration: Value,
    firmware_sha256: String,
    provenance: Value,
    timestamp_unix_ms: u64,
    evidence: Value,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn validated_run(
    directory: &Path,
    run: &ScheduledRun,
    previous: &[u64],
    admission: Option<&crate::qualification::QualifiedBackend>,
) -> Result<ValidatedRun> {
    let bytes = fs::read(directory.join("run.json")).context("reading run report")?;
    let report: Value = serde_json::from_slice(&bytes)?;
    crate::qualification::report_admission(&report, admission)?;
    ensure!(report["status"] == "pass", "capture did not pass");
    ensure!(
        report["run_order_index"] == run.run_order,
        "run order differs from schedule"
    );
    let replay = crate::queue_protocol::validate_file(
        &directory.join("serial.raw"),
        run.backend.name(),
        previous,
    )?;
    for field in ["hello", "qualification", "config_after", "benchmarks"] {
        ensure!(
            report[field] == replay[field],
            "capture/replay mismatch in {field}"
        );
    }
    let nonce = replay["hello"]["boot_nonce"]
        .as_u64()
        .context("missing nonce")?;
    let median = replay["benchmarks"]["message-roundtrip"]["p50_ticks"]
        .as_u64()
        .context("missing roundtrip median")?;
    let firmware = fs::read(directory.join("firmware.elf"))?;
    let firmware_sha256 = hash(&firmware);
    ensure!(
        report["firmware_sha256"] == firmware_sha256,
        "firmware hash mismatch"
    );
    Ok(ValidatedRun {
        nonce,
        median,
        configuration: replay["hello"]["config_before"].clone(),
        firmware_sha256,
        provenance: report["provenance"].clone(),
        timestamp_unix_ms: report["timestamp_unix_ms"]
            .as_u64()
            .context("missing capture timestamp")?,
        evidence: json!({
            "directory": run.directory,
            "backend": run.backend,
            "boot_nonce": nonce,
            "p50_ticks": median,
            "timestamp_unix_ms": report["timestamp_unix_ms"],
            "report_sha256": hash(&bytes),
            "serial_sha256": hash(&fs::read(directory.join("serial.raw"))?),
            "benchmarks": replay["benchmarks"]
        }),
    })
}

fn central_value(values: &[i64]) -> Option<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    match sorted.len() {
        0 => None,
        count if count % 2 == 0 => {
            let sum = i128::from(sorted[count / 2 - 1]) + i128::from(sorted[count / 2]);
            Some(sum as f64 / 2.0)
        }
        count => Some(sorted[count / 2] as f64),
    }
}

fn contrast_summary(values: &[i64]) -> Value {
    json!({"blocks": values.len(), "median_difference_ticks": central_value(values)})
}

/// Replays each scheduled raw capture before calculating paired contrasts.
///
/// Missing, failed or invalid runs remain visible and make the fixed study
/// incomplete. No complete-study interval is reported for a successful subset.
/// Schedule/report I/O or contract failures return an error; missing run files
/// are instead represented as unsuccessful scheduled attempts.
pub(crate) fn analyze(schedule: &Path, results: &Path) -> Result<Value> {
    analyze_with_root(&std::env::current_dir()?, schedule, results)
}

pub(crate) fn analyze_with_root(root: &Path, schedule: &Path, results: &Path) -> Result<Value> {
    let schedule_bytes = fs::read(schedule)?;
    let schedule: Schedule = serde_json::from_slice(&schedule_bytes)?;
    schedule.validate()?;
    let manifest_bytes =
        fs::read(results.join("study-manifest.json")).context("reading frozen study manifest")?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    ensure!(
        matches!(manifest["schema_version"].as_u64(), Some(1 | 2))
            && manifest["experiment"] == EXPERIMENT,
        "unexpected study manifest"
    );
    let method = if manifest["schema_version"] == 2 {
        let identity: crate::qualification::FileIdentity =
            serde_json::from_value(manifest["method_definition"].clone())
                .context("prospective study needs a frozen method definition")?;
        Some(crate::qualification::study_method(
            root,
            &identity,
            &manifest["images"],
        )?)
    } else {
        ensure!(
            manifest["collector_sha256"] == crate::qualification::HISTORICAL_COLLECTOR,
            "unrecognized legacy study collector; use prospective schema 2"
        );
        ensure!(
            manifest.get("method_definition").is_none(),
            "legacy manifest cannot claim prospective method admission"
        );
        None
    };
    ensure!(
        manifest["schedule_sha256"] == hash(&schedule_bytes),
        "schedule differs from frozen manifest"
    );
    let mut expected_images = Vec::with_capacity(2);
    for backend in [Backend::Std, Backend::Native] {
        let directory = results.join("artifacts").join(backend.short_name());
        let firmware_sha256 = hash(&fs::read(directory.join("firmware.elf"))?);
        let provenance_bytes = fs::read(directory.join("provenance.json"))?;
        let provenance: Value = serde_json::from_slice(&provenance_bytes)?;
        ensure!(
            manifest["images"][backend.name()]["elf_sha256"] == firmware_sha256,
            "frozen image hash mismatch"
        );
        ensure!(
            manifest["images"][backend.name()]["provenance_sha256"] == hash(&provenance_bytes),
            "frozen provenance hash mismatch"
        );
        expected_images.push((firmware_sha256, provenance));
    }
    let mut nonces = Vec::with_capacity(24);
    let mut configuration = None;
    let mut previous_timestamp = None;
    let mut blocks = Vec::with_capacity(12);
    let mut differences = Vec::with_capacity(12);
    let mut sessions: [Vec<i64>; 3] = Default::default();
    let mut orders: [Vec<i64>; 2] = Default::default();
    for block in &schedule.blocks {
        let mut medians = [None, None];
        let mut attempts = Vec::with_capacity(2);
        for run in &block.runs {
            let directory = results.join(&run.directory);
            let index = usize::from(run.backend == Backend::Native);
            let validated =
                validated_run(&directory, run, &nonces, method.as_ref().map(|v| &v[index]))
                    .and_then(|value| {
                        if let Some(expected) = &configuration {
                            ensure!(
                                expected == &value.configuration,
                                "configuration changed across study"
                            );
                        }
                        let index = usize::from(run.backend == Backend::Native);
                        ensure!(
                            expected_images[index].0 == value.firmware_sha256,
                            "capture image differs from frozen backend"
                        );
                        ensure!(
                            expected_images[index].1 == value.provenance,
                            "capture provenance differs from frozen backend"
                        );
                        if let Some(previous) = previous_timestamp {
                            ensure!(
                                value.timestamp_unix_ms > previous,
                                "capture timestamps do not follow scheduled order"
                            );
                        }
                        Ok(value)
                    });
            match validated {
                Ok(value) => {
                    let index = usize::from(run.backend == Backend::Native);
                    medians[index] = Some(value.median);
                    nonces.push(value.nonce);
                    configuration.get_or_insert(value.configuration);
                    previous_timestamp = Some(value.timestamp_unix_ms);
                    attempts.push(json!({"status": "pass", "evidence": value.evidence}));
                }
                Err(error) => {
                    let report_path = directory.join("run.json");
                    let report = fs::read(&report_path)
                        .ok()
                        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
                    let status = match report.as_ref().and_then(|value| value["status"].as_str()) {
                        Some("failed") => "failed",
                        Some("running") => "incomplete",
                        _ if !report_path.exists() => "missing",
                        _ => "invalid",
                    };
                    attempts.push(json!({
                        "status": status,
                        "directory": run.directory,
                        "backend": run.backend,
                        "reason": format!("{error:#}"),
                        "capture_failure": report.as_ref().map(|value| &value["failure"])
                    }));
                }
            }
        }
        let difference = match medians {
            [Some(std), Some(native)] => Some(i64::try_from(native)? - i64::try_from(std)?),
            _ => None,
        };
        if let Some(value) = difference {
            differences.push(value);
            sessions[(block.session - 1) as usize].push(value);
            orders[usize::from(block.runs[0].backend == Backend::Native)].push(value);
        }
        blocks.push(json!({
            "block": block.block,
            "session": block.session,
            "attempts": attempts,
            "native_minus_std_p50_ticks": difference
        }));
    }
    let complete = differences.len() == 12;
    let interval = if complete {
        let mut ordered = differences.clone();
        ordered.sort_unstable();
        json!({
            "lower_ticks": ordered[2], "upper_ticks": ordered[9],
            "coverage_continuous_iid": 0.96142578125_f64,
            "assumption": "Independent identically distributed block contrasts; ties can increase coverage",
            "estimand": "Median paired difference of boot p50 ticks, native minus std"
        })
    } else {
        Value::Null
    };
    let mut report = json!({
        "schema_version": 1, "experiment": EXPERIMENT,
        "status": if complete { "complete" } else { "incomplete" },
        "schedule_sha256": hash(&schedule_bytes), "seed": schedule.seed,
        "manifest_sha256": hash(&manifest_bytes),
        "scheduled_blocks": 12, "complete_blocks": differences.len(),
        "validated_boots": nonces.len(), "tick_hz": 16_000_000,
        "difference_direction": "native minus std; negative means smaller native-path p50",
        "descriptive_complete_pairs": contrast_summary(&differences),
        "conditional_median_interval": interval,
        "sessions": sessions.iter().map(|values| contrast_summary(values)).collect::<Vec<_>>(),
        "order_std_then_native": contrast_summary(&orders[0]),
        "order_native_then_std": contrast_summary(&orders[1]),
        "blocks": blocks,
        "limitations": [
            "One board, fixed images, warm resets; no manufacturing-population estimate",
            "Read chronological/session/order evidence before relying on IID interval assumptions",
            "Capture/replay and image hashes do not attest physical flashing or oscillator calibration",
            "Fixture invalidations and their one permitted block repeat require an explicit external attempt ledger"
        ]
    });
    if method.is_some() {
        report["admission_mode"] = json!("prospective_method_controls_verified");
        report["method_definition"] = manifest["method_definition"].clone();
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new() -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "esp32-c6-queue-study-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            write_schedule(42, &root.join("schedule.json")).unwrap();
            let schedule = Schedule::generate(42);
            let mut images = serde_json::Map::new();
            for backend in [Backend::Std, Backend::Native] {
                let directory = root.join("artifacts").join(backend.short_name());
                fs::create_dir_all(&directory).unwrap();
                let firmware = backend.name().as_bytes();
                let provenance =
                    serde_json::to_vec(&json!({"fixture": true, "backend": backend})).unwrap();
                fs::write(directory.join("firmware.elf"), firmware).unwrap();
                fs::write(directory.join("provenance.json"), &provenance).unwrap();
                images.insert(
                    backend.name().into(),
                    json!({
                        "elf_sha256": hash(firmware), "provenance_sha256": hash(&provenance)
                    }),
                );
            }
            for block in &schedule.blocks {
                for run in &block.runs {
                    let directory = root.join(&run.directory);
                    fs::create_dir_all(&directory).unwrap();
                    let mut records = crate::queue_protocol::tests::transcript(run.backend.name());
                    for record in &mut records {
                        record["boot_nonce"] = json!(run.run_order);
                        if record["event"] == "sample" && record["benchmark"] == "message-roundtrip"
                        {
                            record["end_tick"] = json!(
                                record["start_tick"].as_u64().unwrap()
                                    + if run.backend == Backend::Std {
                                        200
                                    } else {
                                        120
                                    }
                            );
                        }
                    }
                    let mut raw = b"ESP-ROM:esp32c6-20220919\r\nrst:0x1\r\n".to_vec();
                    for record in &records {
                        serde_json::to_writer(&mut raw, record).unwrap();
                        raw.push(b'\n');
                    }
                    fs::write(directory.join("serial.raw"), raw).unwrap();
                    let mut report = crate::queue_protocol::validate_file(
                        &directory.join("serial.raw"),
                        run.backend.name(),
                        &[],
                    )
                    .unwrap();
                    let firmware = run.backend.name().as_bytes();
                    report["schema_version"] = json!(3);
                    report["collector_executable_sha256"] =
                        json!(crate::qualification::HISTORICAL_COLLECTOR);
                    report["status"] = json!("pass");
                    report["run_order_index"] = json!(run.run_order);
                    report["timestamp_unix_ms"] = json!(1_000 + run.run_order);
                    report["firmware_sha256"] = json!(hash(firmware));
                    report["provenance"] = json!({"fixture": true, "backend": run.backend});
                    fs::write(directory.join("firmware.elf"), firmware).unwrap();
                    fs::write(
                        directory.join("run.json"),
                        serde_json::to_vec(&report).unwrap(),
                    )
                    .unwrap();
                }
            }
            let manifest = json!({"schema_version": 1, "experiment": EXPERIMENT,
                "collector_sha256": crate::qualification::HISTORICAL_COLLECTOR,
                "schedule_sha256": hash(&fs::read(root.join("schedule.json")).unwrap()),
                "images": images});
            fs::write(
                root.join("study-manifest.json"),
                serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
            Self(root)
        }

        fn analyze(&self) -> Value {
            analyze(&self.0.join("schedule.json"), &self.0).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // This path was generated and exclusively created by this fixture.
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn study_replays_all_boots_and_reports_only_the_prespecified_contrast() {
        let fixture = Fixture::new();
        let report = fixture.analyze();
        assert_eq!(report["status"], "complete");
        assert_eq!(report["validated_boots"], 24);
        assert_eq!(report["complete_blocks"], 12);
        assert_eq!(
            report["descriptive_complete_pairs"]["median_difference_ticks"],
            -80.0
        );
        assert_eq!(report["conditional_median_interval"]["lower_ticks"], -80);
        assert_eq!(report["conditional_median_interval"]["upper_ticks"], -80);
        assert_eq!(
            report["conditional_median_interval"]["coverage_continuous_iid"],
            0.96142578125
        );
        assert!(write_schedule(12, &fixture.0.join("schedule.json")).is_err());
    }

    #[test]
    fn failed_or_invalid_attempts_never_produce_a_success_subset_interval() {
        let fixture = Fixture::new();
        let schedule = Schedule::generate(42);
        let first = fixture
            .0
            .join(&schedule.blocks[0].runs[0].directory)
            .join("run.json");
        let original = fs::read(&first).unwrap();
        let mut failed: Value = serde_json::from_slice(&original).unwrap();
        failed["status"] = json!("failed");
        failed["failure"] = json!("wrong reply");
        fs::write(&first, serde_json::to_vec(&failed).unwrap()).unwrap();
        let report = fixture.analyze();
        assert_eq!(report["status"], "incomplete");
        assert_eq!(report["complete_blocks"], 11);
        assert!(report["conditional_median_interval"].is_null());
        assert_eq!(report["blocks"][0]["attempts"][0]["status"], "failed");
        fs::write(&first, original).unwrap();

        let second = fixture
            .0
            .join(&schedule.blocks[0].runs[1].directory)
            .join("run.json");
        let mut reordered: Value = serde_json::from_slice(&fs::read(&second).unwrap()).unwrap();
        reordered["timestamp_unix_ms"] = json!(1);
        fs::write(&second, serde_json::to_vec(&reordered).unwrap()).unwrap();
        let report = fixture.analyze();
        assert_eq!(report["status"], "incomplete");
        assert!(report["conditional_median_interval"].is_null());
        assert_eq!(report["blocks"][0]["attempts"][1]["status"], "invalid");

        fs::write(fixture.0.join("artifacts/std/firmware.elf"), b"changed").unwrap();
        assert!(analyze(&fixture.0.join("schedule.json"), &fixture.0).is_err());
    }

    #[test]
    fn hidden_pre_json_crash_or_restart_invalidates_a_study_block() {
        let fixture = Fixture::new();
        let schedule = Schedule::generate(42);
        let path = fixture
            .0
            .join(&schedule.blocks[0].runs[0].directory)
            .join("serial.raw");
        let original = fs::read(&path).unwrap();
        for prefix in [
            b"Guru Meditation Error: Core 0 panic'ed\n".as_slice(),
            b"Rebooting...\n".as_slice(),
            b"ESP-ROM:esp32c6-20220919\n".as_slice(),
        ] {
            // Keep every previously passing report/image/provenance untouched.
            // Admission must inspect text, not only compare parsed JSON fields.
            fs::write(&path, [prefix, original.as_slice()].concat()).unwrap();
            let report = fixture.analyze();
            assert_eq!(report["status"], "incomplete");
            assert_eq!(report["complete_blocks"], 11);
            assert_eq!(report["validated_boots"], 23);
            assert!(report["conditional_median_interval"].is_null());
            assert_eq!(report["blocks"][0]["attempts"][0]["status"], "invalid");
        }
    }

    #[test]
    fn a_new_qualification_capture_cannot_be_promoted_by_deleting_metadata() {
        let fixture = Fixture::new();
        let schedule = Schedule::generate(42);
        let path = fixture
            .0
            .join(&schedule.blocks[0].runs[0].directory)
            .join("run.json");
        let mut report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        report["schema_version"] = json!(4);
        report["collector_executable_sha256"] = json!("new collector identity");
        report["admission"] = json!({"purpose": "qualification", "elf_sha256": report["firmware_sha256"],
            "provenance_sha256": "0".repeat(64), "method": null});
        fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
        let result = fixture.analyze();
        assert_eq!(result["complete_blocks"], 11);
        assert!(result["conditional_median_interval"].is_null());
        assert!(
            result["blocks"][0]["attempts"][0]["reason"]
                .as_str()
                .unwrap()
                .contains("qualification pilot")
        );
        report.as_object_mut().unwrap().remove("admission");
        for version in [4, 3] {
            report["schema_version"] = json!(version);
            fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
            let result = fixture.analyze();
            assert_eq!(result["complete_blocks"], 11);
            assert!(result["conditional_median_interval"].is_null());
        }
    }

    #[test]
    fn prospective_study_checks_real_controls_and_both_receipt_paths() {
        // All data/ELFs in this graph are synthetic host fixtures. Positive
        // controls are saved separately before comparative receipts are made.
        let fixture = Fixture::new();
        let root = &fixture.0;
        let schedule = Schedule::generate(42);
        let identity = |path: &Path| {
            json!({"path": path.strip_prefix(root).unwrap().to_str().unwrap(),
            "sha256": hash(&fs::read(path).unwrap())})
        };
        for (name, text) in [
            ("source.txt", "fixture endpoints"),
            ("config.txt", "fixed fixture config"),
            ("semantics.md", "fixture semantic review"),
            ("adversarial.md", "different fixture adversarial review"),
        ] {
            fs::write(root.join(name), text).unwrap();
        }
        let mut variants = Vec::new();
        let mut images = serde_json::Map::new();
        for backend in [Backend::Std, Backend::Native] {
            let artifacts = root.join("artifacts").join(backend.short_name());
            let elf = crate::qualification::tests::elf_fixture(backend.name().as_bytes());
            fs::write(artifacts.join("firmware.elf"), &elf).unwrap();
            let provenance = fs::read(artifacts.join("provenance.json")).unwrap();
            for run in schedule
                .blocks
                .iter()
                .flat_map(|b| &b.runs)
                .filter(|r| r.backend == backend)
            {
                let directory = root.join(&run.directory);
                fs::write(directory.join("firmware.elf"), &elf).unwrap();
                let mut report: Value =
                    serde_json::from_slice(&fs::read(directory.join("run.json")).unwrap()).unwrap();
                report["firmware_sha256"] = json!(hash(&elf));
                fs::write(
                    directory.join("run.json"),
                    serde_json::to_vec(&report).unwrap(),
                )
                .unwrap();
            }
            let run = schedule
                .blocks
                .iter()
                .flat_map(|b| &b.runs)
                .find(|r| r.backend == backend)
                .unwrap();
            let control = root.join("controls").join(backend.short_name());
            fs::create_dir_all(&control).unwrap();
            fs::copy(
                root.join(&run.directory).join("serial.raw"),
                control.join("serial.raw"),
            )
            .unwrap();
            fs::copy(
                root.join(&run.directory).join("run.json"),
                control.join("run.json"),
            )
            .unwrap();
            variants.push(json!({"id": backend.short_name(), "wire": "queue", "selector": backend.name(),
                "elf": identity(&artifacts.join("firmware.elf")), "provenance": identity(&artifacts.join("provenance.json")),
                "configuration": [identity(&root.join("config.txt"))],
                "positive_raw": identity(&control.join("serial.raw")), "positive_report": identity(&control.join("run.json")),
                "required_elf_text": [backend.name()], "required_output_text": ["\"event\":\"complete\""]}));
            images.insert(
                backend.name().into(),
                json!({"elf_sha256": hash(&elf), "provenance_sha256": hash(&provenance)}),
            );
        }
        let contracts = ["message-roundtrip", "timer-overhead"].map(|metric| json!({"metric": metric,
            "begin": "fixture start", "end": "fixture end", "units_and_clock": "nominal ticks",
            "aggregation": "per-event gross", "observer_policy": "no subtraction", "correctness": "checked samples",
            "source": identity(&root.join("source.txt"))}));
        let definition = json!({"schema_version": 1, "id": "prospective-fixture", "decision_question": "admission control test",
            "qualification_timing": "synthetic offline controls", "timing_contracts": contracts, "variants": variants,
            "review": {"producer": "fixture author", "reviewer": "fixture reviewer", "semantics": identity(&root.join("semantics.md")),
                "adversarial_reproduction": identity(&root.join("adversarial.md"))}, "limitations": ["No physical evidence"]});
        fs::write(
            root.join("method.json"),
            serde_json::to_vec(&definition).unwrap(),
        )
        .unwrap();
        let method_identity = identity(&root.join("method.json"));
        let manifest = json!({"schema_version": 2, "experiment": EXPERIMENT,
            "schedule_sha256": hash(&fs::read(root.join("schedule.json")).unwrap()),
            "method_definition": method_identity, "images": images});
        fs::write(
            root.join("study-manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        for run in schedule.blocks.iter().flat_map(|b| &b.runs) {
            let path = root.join(&run.directory).join("run.json");
            let mut report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            report["schema_version"] = json!(4);
            report["collector_executable_sha256"] = json!("synthetic new collector");
            report["admission"] = json!({"purpose": "comparison", "elf_sha256": report["firmware_sha256"],
                "provenance_sha256": images[run.backend.name()]["provenance_sha256"],
                "method": {"id": "prospective-fixture", "variant": run.backend.short_name(), "definition": method_identity}});
            fs::write(path, serde_json::to_vec(&report).unwrap()).unwrap();
        }
        let analyze = || analyze_with_root(root, &root.join("schedule.json"), root).unwrap();
        let result = analyze();
        assert_eq!(result["complete_blocks"], 12);
        assert_eq!(
            result["admission_mode"],
            "prospective_method_controls_verified"
        );
        assert!(!result["conditional_median_interval"].is_null());
        let path = root
            .join(&schedule.blocks[0].runs[0].directory)
            .join("run.json");
        let original: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        for fault in ["pilot", "missing", "downgraded", "wrong_method"] {
            let mut report = original.clone();
            match fault {
                "pilot" => report["admission"]["purpose"] = json!("qualification"),
                "missing" => {
                    report.as_object_mut().unwrap().remove("admission");
                }
                "downgraded" => {
                    report["schema_version"] = json!(3);
                    report["collector_executable_sha256"] =
                        json!(crate::qualification::HISTORICAL_COLLECTOR);
                }
                _ => report["admission"]["method"]["variant"] = json!("wrong variant"),
            }
            fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
            let result = analyze();
            assert_eq!(result["complete_blocks"], 11, "{fault}");
            assert!(result["conditional_median_interval"].is_null(), "{fault}");
        }
    }

    #[test]
    fn seeded_schedule_balances_each_session_and_never_traverses() {
        for seed in [0, 1, 42, u64::MAX] {
            let schedule = Schedule::generate(seed);
            schedule.validate().unwrap();
            assert_eq!(schedule.blocks.len(), 12);
            for session in 1..=3 {
                let blocks: Vec<_> = schedule
                    .blocks
                    .iter()
                    .filter(|block| block.session == session)
                    .collect();
                assert_eq!(blocks.len(), 4);
                assert_eq!(
                    blocks
                        .iter()
                        .filter(|block| block.runs[0].backend == Backend::Std)
                        .count(),
                    2
                );
                for block in blocks {
                    assert_ne!(block.runs[0].backend, block.runs[1].backend);
                    for run in &block.runs {
                        assert!(run.directory.starts_with("runs/"));
                        assert!(!run.directory.contains(".."));
                    }
                }
            }
        }
    }

    #[test]
    fn schedule_tampering_is_rejected_before_any_capture_is_read() {
        let mut schedule = Schedule::generate(42);
        schedule.blocks[0].runs[0].directory = "../outside".into();
        assert!(schedule.validate().is_err());
        let mut schedule = Schedule::generate(42);
        schedule.blocks[0].runs[1].backend = schedule.blocks[0].runs[0].backend;
        assert!(schedule.validate().is_err());
    }

    #[test]
    fn contrast_median_handles_signed_even_and_empty_samples() {
        assert_eq!(central_value(&[]), None);
        assert_eq!(central_value(&[-12, -5, 3]), Some(-5.0));
        assert_eq!(central_value(&[2, -3]), Some(-0.5));
        assert_eq!(central_value(&[i64::MIN, i64::MAX]), Some(-0.5));
    }

    #[test]
    fn permutation_stream_is_reproducible() {
        let mut random = Permutation(0);
        assert_eq!(random.next(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(random.next(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(Schedule::generate(17), Schedule::generate(17));
    }
}
