//! Offline method controls and artifact-bound admission before serial access.
//!
//! These checks establish consistency and reproduce selected rejection controls.
//! Review records and printed tokens do not authenticate hardware or prove that
//! the stated endpoints, independence or physical accuracy are correct.

use std::collections::BTreeSet;
use std::fs;
use std::io::Cursor;
use std::path::{Component, Path};

use anyhow::{Context, Result, ensure};
use clap::ValueEnum;
use object::Object;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Original, explicitly historical queue collector, never produced schema 4.
pub(crate) const HISTORICAL_COLLECTOR: &str =
    "1b5e71a6648086369ba8f4af53e7e1622a08d990fab24448a46126abcfd4277a";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CapturePurpose {
    Qualification,
    Comparison,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Wire {
    Legacy,
    Queue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FileIdentity {
    pub(crate) path: String,
    pub(crate) sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TimingContract {
    metric: String,
    begin: String,
    end: String,
    units_and_clock: String,
    aggregation: String,
    observer_policy: String,
    correctness: String,
    source: FileIdentity,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Variant {
    id: String,
    wire: Wire,
    selector: String,
    elf: FileIdentity,
    provenance: FileIdentity,
    configuration: Vec<FileIdentity>,
    positive_raw: FileIdentity,
    positive_report: FileIdentity,
    required_elf_text: Vec<String>,
    required_output_text: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Review {
    producer: String,
    reviewer: String,
    semantics: FileIdentity,
    adversarial_reproduction: FileIdentity,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Method {
    schema_version: u32,
    id: String,
    decision_question: String,
    qualification_timing: String,
    timing_contracts: Vec<TimingContract>,
    variants: Vec<Variant>,
    review: Review,
    limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MethodProof {
    pub(crate) id: String,
    pub(crate) variant: String,
    pub(crate) definition: FileIdentity,
}

/// New capture-report metadata. A pilot has no comparative method proof.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Admission {
    pub(crate) purpose: CapturePurpose,
    pub(crate) elf_sha256: String,
    pub(crate) provenance_sha256: String,
    pub(crate) method: Option<MethodProof>,
}

fn nonempty(value: &str, field: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "empty method field: {field}");
    Ok(())
}

fn actor(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn relative_file(root: &Path, path: &str) -> Result<std::path::PathBuf> {
    let path = Path::new(path);
    ensure!(
        !path.as_os_str().is_empty()
            && path.components().all(|c| matches!(c, Component::Normal(_))),
        "method evidence must be a repository-relative file"
    );
    let root = root.canonicalize()?;
    let target = root
        .join(path)
        .canonicalize()
        .context("finding method evidence")?;
    ensure!(
        target.starts_with(&root),
        "method evidence escapes repository"
    );
    ensure!(target.is_file(), "method evidence is not a regular file");
    Ok(target)
}

fn read_identity(root: &Path, identity: &FileIdentity) -> Result<Vec<u8>> {
    ensure!(
        identity.sha256.len() == 64
            && identity
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "invalid method SHA-256 for {}",
        identity.path
    );
    let bytes = fs::read(relative_file(root, &identity.path)?)?;
    ensure!(
        crate::digest(&bytes) == identity.sha256,
        "stale method evidence: {}",
        identity.path
    );
    Ok(bytes)
}

fn contains(bytes: &[u8], text: &str) -> bool {
    !text.is_empty()
        && bytes
            .windows(text.len())
            .any(|window| window == text.as_bytes())
}

fn replay(bytes: &[u8], wire: Wire, selector: &str) -> Result<Value> {
    match wire {
        Wire::Queue => crate::queue_protocol::validate_bytes(bytes, selector, &[]),
        Wire::Legacy => {
            let boot = crate::validate(Cursor::new(bytes), selector)?;
            Ok(json!({"hello": boot.hello(), "benchmarks": boot.summaries()?}))
        }
    }
}

fn corrupt_checksum(raw: &[u8]) -> Result<Vec<u8>> {
    let mut changed = false;
    let mut output = Vec::with_capacity(raw.len());
    for line in raw.split_inclusive(|&b| b == b'\n') {
        if !changed && line.trim_ascii().starts_with(b"{") {
            let mut record: Value = serde_json::from_slice(line)?;
            if record["event"] == "sample"
                && let Some(checksum) = record["checksum"].as_u64().filter(|&v| v != 0)
            {
                record["checksum"] = json!(checksum ^ 1);
                serde_json::to_writer(&mut output, &record)?;
                output.push(b'\n');
                changed = true;
                continue;
            }
        }
        output.extend_from_slice(line);
    }
    ensure!(
        changed,
        "positive control lacks a nonzero sample checksum to challenge"
    );
    Ok(output)
}

/// Executes controls through the actual replay validators, not a second parser.
fn negative_controls(raw: &[u8], wire: Wire, selector: &str) -> Result<Vec<&'static str>> {
    let mut passed = Vec::new();
    for (name, prefix, reason) in [
        (
            "panic_before_json",
            "Guru Meditation Error: Core 0 panic'ed\n",
            "fatal or reboot",
        ),
        ("reboot_before_json", "Rebooting...\n", "fatal or reboot"),
        (
            "additional_startup",
            "ESP-ROM:esp32c6-20220919\n",
            "additional ROM",
        ),
    ] {
        let altered = [prefix.as_bytes(), raw].concat();
        let error = replay(&altered, wire, selector)
            .err()
            .with_context(|| format!("negative control unexpectedly passed: {name}"))?;
        ensure!(
            format!("{error:#}").contains(reason),
            "{name} failed for an unrelated reason: {error:#}"
        );
        passed.push(name);
    }
    // Remove the completion record, not an arbitrary byte in a valid payload.
    let last_record = raw
        .split_inclusive(|&b| b == b'\n')
        .scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line))
        })
        .filter(|(_, line)| line.trim_ascii().starts_with(b"{"))
        .last()
        .context("positive control has no protocol record")?
        .0;
    let error = replay(&raw[..last_record], wire, selector)
        .err()
        .context("missing-completion negative control unexpectedly passed")?;
    let message = format!("{error:#}").to_ascii_lowercase();
    ensure!(
        message.contains("complete") || message.contains("completion"),
        "missing completion failed for an unrelated reason: {error:#}"
    );
    passed.push("missing_completion");
    let error = replay(&corrupt_checksum(raw)?, wire, selector)
        .err()
        .context("checksum negative control unexpectedly passed")?;
    ensure!(
        format!("{error:#}").contains("checksum"),
        "checksum control failed for an unrelated reason: {error:#}"
    );
    passed.push("wrong_checksum");
    Ok(passed)
}

fn inspect_method(root: &Path, definition: &FileIdentity) -> Result<(Method, Value)> {
    let method: Method = serde_json::from_slice(&read_identity(root, definition)?)?;
    ensure!(
        method.schema_version == 1,
        "unsupported method definition version"
    );
    for (value, field) in [
        (&method.id, "id"),
        (&method.decision_question, "decision_question"),
        (&method.qualification_timing, "qualification_timing"),
        (&method.review.producer, "review producer"),
        (&method.review.reviewer, "reviewer"),
    ] {
        nonempty(value, field)?;
    }
    ensure!(
        actor(&method.review.producer) != actor(&method.review.reviewer),
        "method needs distinct producer/reviewer attribution"
    );
    ensure!(
        method.review.semantics.sha256 != method.review.adversarial_reproduction.sha256,
        "semantic and adversarial reviews need distinct artifact identities"
    );
    ensure!(
        !method.limitations.is_empty() && method.limitations.iter().all(|v| !v.trim().is_empty()),
        "method limitations must be explicit"
    );
    read_identity(root, &method.review.semantics)?;
    read_identity(root, &method.review.adversarial_reproduction)?;
    ensure!(
        !method.timing_contracts.is_empty(),
        "method lacks timing contracts"
    );
    let mut metric_ids = BTreeSet::new();
    for metric in &method.timing_contracts {
        ensure!(
            metric_ids.insert(&metric.metric),
            "duplicate metric contract"
        );
        for (value, field) in [
            (&metric.metric, "metric"),
            (&metric.begin, "begin"),
            (&metric.end, "end"),
            (&metric.units_and_clock, "units_and_clock"),
            (&metric.aggregation, "aggregation"),
            (&metric.observer_policy, "observer_policy"),
            (&metric.correctness, "correctness"),
        ] {
            nonempty(value, field)?;
        }
        read_identity(root, &metric.source)?;
    }
    ensure!(!method.variants.is_empty(), "method has no variants");
    let mut variant_ids = BTreeSet::new();
    let mut selectors = BTreeSet::new();
    let mut evidence = Vec::new();
    for variant in &method.variants {
        nonempty(&variant.id, "variant id")?;
        nonempty(&variant.selector, "variant selector")?;
        ensure!(
            variant_ids.insert(&variant.id)
                && selectors.insert((format!("{:?}", variant.wire), &variant.selector)),
            "duplicate method variant"
        );
        let elf = read_identity(root, &variant.elf)?;
        let object = object::File::parse(elf.as_slice()).context("reading method ELF")?;
        ensure!(
            object.format() == object::BinaryFormat::Elf
                && object.architecture() == object::Architecture::Riscv32
                && object.kind() == object::ObjectKind::Executable
                && object.segments().next().is_some(),
            "method needs a loadable RISC-V32 executable ELF"
        );
        let provenance: Value = serde_json::from_slice(&read_identity(root, &variant.provenance)?)?;
        ensure!(provenance.is_object(), "method provenance is not an object");
        ensure!(
            !variant.configuration.is_empty(),
            "missing configuration identity"
        );
        for file in &variant.configuration {
            read_identity(root, file)?;
        }
        ensure!(
            !variant.required_elf_text.is_empty() && !variant.required_output_text.is_empty(),
            "method requires compiled and observed output controls"
        );
        for text in &variant.required_elf_text {
            ensure!(
                contains(&elf, text),
                "required text absent from compiled ELF: {text}"
            );
        }
        let raw = read_identity(root, &variant.positive_raw)?;
        for text in &variant.required_output_text {
            ensure!(
                contains(&raw, text),
                "required text absent from positive capture: {text}"
            );
        }
        let replayed = replay(&raw, variant.wire, &variant.selector)
            .with_context(|| format!("positive control rejected: {}", variant.id))?;
        let actual_metrics = replayed["benchmarks"]
            .as_object()
            .context("positive control has no benchmark map")?
            .keys()
            .collect::<BTreeSet<_>>();
        ensure!(
            actual_metrics == metric_ids,
            "timing contracts do not exactly cover positive-control metrics"
        );
        let report: Value =
            serde_json::from_slice(&read_identity(root, &variant.positive_report)?)?;
        ensure!(
            report["status"] == "pass",
            "positive control was not a passing capture"
        );
        ensure!(
            report["firmware_sha256"] == variant.elf.sha256,
            "positive control ELF mismatch"
        );
        ensure!(
            report["provenance"] == provenance,
            "positive control provenance mismatch"
        );
        for field in ["hello", "benchmarks"] {
            ensure!(
                report[field] == replayed[field],
                "positive capture/replay mismatch: {field}"
            );
        }
        if variant.wire == Wire::Queue {
            for field in ["qualification", "config_after"] {
                ensure!(
                    report[field] == replayed[field],
                    "positive capture/replay mismatch: {field}"
                );
            }
        }
        evidence.push(json!({"variant": variant.id, "positive_control": "pass", "negative_controls_rejected": negative_controls(&raw, variant.wire, &variant.selector)?}));
    }
    let output = json!({
        "schema_version": 1, "method_id": method.id, "definition": definition,
        "status": "controls_passed", "variants": evidence,
        "limits": "Artifact/control consistency only. Labels, ELF strings and transcripts do not establish semantic correctness, independence, physical authenticity or absolute accuracy."
    });
    Ok((method, output))
}

fn definition_identity(root: &Path, path: &Path) -> Result<FileIdentity> {
    let root = root.canonicalize()?;
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
    .canonicalize()?;
    ensure!(
        path.starts_with(&root) && path.is_file(),
        "method definition escapes repository or is not a file"
    );
    Ok(FileIdentity {
        path: path
            .strip_prefix(&root)?
            .to_str()
            .context("non-UTF8 method path")?
            .to_owned(),
        sha256: crate::digest(&fs::read(path)?),
    })
}

/// Verifies exact artifacts, a passing transcript and five named rejection controls.
pub(crate) fn qualify(root: &Path, path: &Path) -> Result<Value> {
    inspect_method(root, &definition_identity(root, path)?).map(|(_, report)| report)
}

/// Replays retained successful protocols and compares complete archived outputs.
/// Failed captures remain failures; no failed raw trace is promoted by replay.
pub(crate) fn replay_archives(root: &Path) -> Result<Value> {
    fn collect(path: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "archive replay refuses symlink traversal"
            );
            if kind.is_dir() {
                collect(&entry.path(), files)?;
            } else if kind.is_file() {
                files.push(entry.path());
            }
        }
        Ok(())
    }
    let root = root.canonicalize()?;
    let mut files = Vec::new();
    collect(&root.join("results"), &mut files)?;
    files.sort();
    let mut directories = BTreeSet::new();
    for path in &files {
        if path.file_name().is_some_and(|n| n == "serial.raw") {
            directories.insert(
                path.parent()
                    .context("raw file has no parent")?
                    .to_path_buf(),
            );
        } else if path.file_name().is_some_and(|n| n == "run.json") {
            let report: Value = serde_json::from_slice(&fs::read(path)?)?;
            if report.get("collector_executable_sha256").is_some()
                || matches!(report["hello"]["protocol"].as_u64(), Some(1..=4))
            {
                directories.insert(path.parent().context("report has no parent")?.to_path_buf());
            }
        }
    }
    let mut successful = 0_usize;
    let mut failed = 0_usize;
    let mut studies = 0_usize;
    for directory in directories {
        let path = directory.join("serial.raw");
        let report: Value = serde_json::from_slice(&fs::read(directory.join("run.json"))?)?;
        let raw = fs::read(&path)
            .with_context(|| format!("capture lacks retained raw bytes: {}", path.display()))?;
        let accounting = report["hello"]["protocol"] == 4
            || report["experiment"] == crate::accounting::EXPERIMENT;
        if accounting {
            let admission: Admission = serde_json::from_value(report["admission"].clone())?;
            ensure!(
                report["schema_version"] == 4
                    && report["experiment"] == crate::accounting::EXPERIMENT
                    && admission.purpose == CapturePurpose::Qualification
                    && admission.method.is_none()
                    && report["benchmarks"].is_null()
                    && report["firmware_sha256"] == admission.elf_sha256,
                "accounting archive must retain qualification-only admission"
            );
            let provenance_bytes = fs::read(directory.join("provenance.json"))?;
            admission.check_inputs(
                &fs::read(directory.join("firmware.elf"))?,
                &provenance_bytes,
            )?;
            ensure!(
                report["provenance"] == serde_json::from_slice::<Value>(&provenance_bytes)?,
                "accounting provenance differs from retained input bytes"
            );
        }
        if report["status"] == "failed" {
            ensure!(
                report["failure"]
                    .as_str()
                    .is_some_and(|v| !v.trim().is_empty())
                    && report["benchmarks"].is_null()
                    && (!accounting || report["qualification"].is_null()),
                "failed capture must retain a failure and no favorable summary"
            );
            failed += 1;
            continue;
        }
        ensure!(
            report["status"] == "pass",
            "unfinished capture cannot be presented as a finalized replay: {}",
            path.display()
        );
        ensure!(
            report["firmware_sha256"] == crate::digest(&fs::read(directory.join("firmware.elf"))?),
            "capture ELF differs from supplied artifact: {}",
            path.display()
        );
        let value = if accounting {
            let value = crate::accounting::validate_bytes(&raw)?;
            for (key, field) in value
                .as_object()
                .context("accounting replay must be an object")?
            {
                ensure!(
                    report[key] == *field,
                    "accounting capture report differs from replay: {key}"
                );
            }
            value
        } else {
            let wire = if report["hello"]["protocol"] == 3 {
                Wire::Queue
            } else {
                Wire::Legacy
            };
            let selector = if wire == Wire::Queue {
                report["backend"].as_str()
            } else {
                report["platform"].as_str()
            }
            .context("capture lacks protocol/backend identity")?;
            replay(&raw, wire, selector)?
        };
        let mut output = Vec::new();
        crate::write_json(&mut output, &value)?;
        ensure!(
            output == fs::read(directory.join("replay.json"))?,
            "archived replay differs: {}",
            path.display()
        );
        successful += 1;
    }
    for path in files
        .iter()
        .filter(|p| p.file_name().is_some_and(|n| n == "study-manifest.json"))
    {
        let manifest: Value = serde_json::from_slice(&fs::read(path)?)?;
        if manifest["experiment"] != "idf-native-queue-v1" {
            continue;
        }
        let directory = path.parent().context("study manifest has no parent")?;
        let value = crate::queue_study::analyze_with_root(
            &root,
            &directory.join("schedule.json"),
            directory,
        )?;
        let mut output = Vec::new();
        crate::write_json(&mut output, &value)?;
        ensure!(
            output == fs::read(directory.join("summary.json"))?,
            "archived study summary differs: {}",
            path.display()
        );
        studies += 1;
    }
    Ok(
        json!({"status": "replays_consistent", "successful_protocol_transcripts": successful,
        "failed_captures_retained": failed, "study_summaries": studies,
        "limits": "Reproduces retained protocol outputs only; does not qualify native console suites, physical accuracy or unrecorded events."}),
    )
}

/// Binds a new capture's purpose and input bytes before any serial access.
pub(crate) fn admit(
    root: &Path,
    purpose: CapturePurpose,
    method_path: Option<&Path>,
    wire: Wire,
    selector: &str,
    elf: &Path,
    provenance: &Path,
) -> Result<Admission> {
    let elf_sha256 = crate::digest(&fs::read(elf).context("reading requested ELF")?);
    let provenance_sha256 =
        crate::digest(&fs::read(provenance).context("reading requested provenance")?);
    let proof = match purpose {
        CapturePurpose::Qualification => {
            ensure!(
                method_path.is_none(),
                "qualification pilot must not carry comparative admission"
            );
            None
        }
        CapturePurpose::Comparison => {
            let path = method_path.context("comparison capture requires --method qualification")?;
            let definition = definition_identity(root, path)?;
            let (method, _) = inspect_method(root, &definition)?;
            let variant = method
                .variants
                .iter()
                .find(|v| v.wire == wire && v.selector == selector)
                .context("method has no matching wire/backend variant")?;
            ensure!(
                variant.elf.sha256 == elf_sha256 && variant.provenance.sha256 == provenance_sha256,
                "capture differs from qualified ELF/provenance; qualify new artifacts first"
            );
            Some(MethodProof {
                id: method.id,
                variant: variant.id.clone(),
                definition,
            })
        }
    };
    Ok(Admission {
        purpose,
        elf_sha256,
        provenance_sha256,
        method: proof,
    })
}

impl Admission {
    pub(crate) fn check_inputs(&self, elf: &[u8], provenance: &[u8]) -> Result<()> {
        ensure!(
            self.elf_sha256 == crate::digest(elf)
                && self.provenance_sha256 == crate::digest(provenance),
            "capture inputs changed after admission"
        );
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn fixture_pilot() -> Self {
        Self {
            purpose: CapturePurpose::Qualification,
            elf_sha256: String::new(),
            provenance_sha256: String::new(),
            method: None,
        }
    }
}

/// Legacy permission is narrowly tied to the retained pre-admission collector.
pub(crate) fn report_admission(report: &Value, expected: Option<&QualifiedBackend>) -> Result<()> {
    if report["schema_version"] == 3 {
        ensure!(
            expected.is_none()
                && report["collector_executable_sha256"] == HISTORICAL_COLLECTOR
                && report.get("admission").is_none(),
            "unrecognized legacy report; cannot omit new admission metadata"
        );
        return Ok(());
    }
    ensure!(
        report["schema_version"] == 4,
        "unsupported capture-report version"
    );
    let admission: Admission = serde_json::from_value(report["admission"].clone())?;
    ensure!(
        admission.purpose == CapturePurpose::Comparison,
        "qualification pilot is excluded from comparative analysis"
    );
    let proof = admission
        .method
        .context("comparison report lacks method qualification")?;
    let expected = expected.context("comparison report needs a prospective frozen study method")?;
    ensure!(
        proof == expected.proof,
        "method differs from frozen study qualification"
    );
    ensure!(
        report["firmware_sha256"] == admission.elf_sha256,
        "admitted ELF differs from capture"
    );
    ensure!(
        admission.elf_sha256 == expected.elf_sha256
            && admission.provenance_sha256 == expected.provenance_sha256,
        "report input identities differ from frozen method"
    );
    Ok(())
}

/// Rechecks a prospective study's frozen method and backend artifact identities.
#[derive(Debug)]
pub(crate) struct QualifiedBackend {
    proof: MethodProof,
    elf_sha256: String,
    provenance_sha256: String,
}

pub(crate) fn study_method(
    root: &Path,
    identity: &FileIdentity,
    images: &Value,
) -> Result<[QualifiedBackend; 2]> {
    let (method, _) = inspect_method(root, identity)?;
    let backend = |backend: &str| -> Result<QualifiedBackend> {
        let variant = method
            .variants
            .iter()
            .find(|v| v.wire == Wire::Queue && v.selector == backend)
            .context("study method lacks a required backend")?;
        ensure!(
            images[backend]["elf_sha256"] == variant.elf.sha256
                && images[backend]["provenance_sha256"] == variant.provenance.sha256,
            "study artifacts differ from qualified backend"
        );
        Ok(QualifiedBackend {
            proof: MethodProof {
                id: method.id.clone(),
                variant: variant.id.clone(),
                definition: identity.clone(),
            },
            elf_sha256: variant.elf.sha256.clone(),
            provenance_sha256: variant.provenance.sha256.clone(),
        })
    };
    Ok([
        backend("rust-std-sync-channel")?,
        backend("freertos-queue")?,
    ])
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Debug)]
    struct Fixture(std::path::PathBuf);

    impl Fixture {
        fn new() -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "c6-method-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }

        fn file(&self, name: &str, bytes: &[u8]) -> FileIdentity {
            fs::write(self.0.join(name), bytes).unwrap();
            FileIdentity {
                path: name.into(),
                sha256: crate::digest(bytes),
            }
        }

        fn method(&self) -> Method {
            // This synthetic ELF identity is only a file/control test. It is
            // never described as a linked firmware or physical qualification.
            let elf = self.file("firmware.elf", &elf_fixture(b"complete-protocol"));
            let provenance_value = json!({"fixture": true});
            let provenance = self.file(
                "provenance.json",
                &serde_json::to_vec(&provenance_value).unwrap(),
            );
            let mut raw = b"ESP-ROM:esp32c6-20220919\nrst:0x1\n".to_vec();
            for record in crate::queue_protocol::tests::transcript("freertos-queue") {
                serde_json::to_writer(&mut raw, &record).unwrap();
                raw.push(b'\n');
            }
            let mut report = replay(&raw, Wire::Queue, "freertos-queue").unwrap();
            report["status"] = json!("pass");
            report["firmware_sha256"] = json!(elf.sha256);
            report["provenance"] = provenance_value;
            let positive_report = self.file("run.json", &serde_json::to_vec(&report).unwrap());
            let source = self.file("source.txt", b"fixture timing description");
            Method {
                schema_version: 1,
                id: "fixture".into(),
                decision_question: "Can the controls reject invalid captures?".into(),
                qualification_timing: "synthetic host test; not physical evidence".into(),
                timing_contracts: vec![
                    TimingContract {
                        metric: "message-roundtrip".into(),
                        begin: "before send".into(),
                        end: "after checked reply".into(),
                        units_and_clock: "nominal ticks".into(),
                        aggregation: "per-event gross".into(),
                        observer_policy: "no subtraction".into(),
                        correctness: "sequence checksum".into(),
                        source: source.clone(),
                    },
                    TimingContract {
                        metric: "timer-overhead".into(),
                        begin: "first read".into(),
                        end: "second read".into(),
                        units_and_clock: "nominal ticks".into(),
                        aggregation: "per-event gross".into(),
                        observer_policy: "not subtracted".into(),
                        correctness: "ordered reads".into(),
                        source,
                    },
                ],
                variants: vec![Variant {
                    id: "native".into(),
                    wire: Wire::Queue,
                    selector: "freertos-queue".into(),
                    elf,
                    provenance,
                    configuration: vec![
                        self.file("configuration.txt", b"fixture fixed configuration"),
                    ],
                    positive_raw: self.file("serial.raw", &raw),
                    positive_report,
                    required_elf_text: vec!["complete-protocol".into()],
                    required_output_text: vec!["\"event\":\"complete\"".into()],
                }],
                review: Review {
                    producer: "test author".into(),
                    reviewer: "test reviewer".into(),
                    semantics: self.file("semantics.md", b"semantic test record"),
                    adversarial_reproduction: self
                        .file("adversarial.md", b"adversarial test record"),
                },
                limitations: vec!["Host fixtures are not hardware evidence".into()],
            }
        }

        fn save(&self, method: &Method) -> FileIdentity {
            self.file("method.json", &serde_json::to_vec(method).unwrap())
        }
    }

    pub(crate) fn elf_fixture(text: &[u8]) -> Vec<u8> {
        // Minimal structural ELF fixture, not executable C6 workload evidence.
        let mut bytes = vec![0_u8; 84];
        bytes[..7].copy_from_slice(b"\x7fELF\x01\x01\x01");
        bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
        bytes[18..20].copy_from_slice(&243_u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&0x4200_0000_u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&52_u32.to_le_bytes());
        bytes[40..42].copy_from_slice(&52_u16.to_le_bytes());
        bytes[42..44].copy_from_slice(&32_u16.to_le_bytes());
        bytes[44..46].copy_from_slice(&1_u16.to_le_bytes());
        bytes[46..48].copy_from_slice(&40_u16.to_le_bytes());
        bytes[52..56].copy_from_slice(&1_u32.to_le_bytes());
        bytes[60..64].copy_from_slice(&0x4200_0000_u32.to_le_bytes());
        bytes[76..80].copy_from_slice(&5_u32.to_le_bytes());
        bytes[80..84].copy_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(text);
        let size = u32::try_from(bytes.len()).unwrap();
        bytes[68..72].copy_from_slice(&size.to_le_bytes());
        bytes[72..76].copy_from_slice(&size.to_le_bytes());
        bytes
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // Exclusively created, generated temporary fixture directory.
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn method_runs_a_positive_and_all_named_negative_controls() {
        let fixture = Fixture::new();
        let method = fixture.method();
        let identity = fixture.save(&method);
        let (_, report) = inspect_method(&fixture.0, &identity).unwrap();
        assert_eq!(report["status"], "controls_passed");
        assert_eq!(
            report["variants"][0]["negative_controls_rejected"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
        let admission = admit(
            &fixture.0,
            CapturePurpose::Comparison,
            Some(Path::new("method.json")),
            Wire::Queue,
            "freertos-queue",
            &fixture.0.join("firmware.elf"),
            &fixture.0.join("provenance.json"),
        )
        .unwrap();
        assert_eq!(admission.method.unwrap().definition, identity);
    }

    #[test]
    fn missing_compiled_output_cannot_be_hidden_by_a_passing_transcript() {
        let fixture = Fixture::new();
        let mut method = fixture.method();
        // Hashes and the supplied report are made self-consistent. The test
        // must fail on the missing ELF literal rather than a stale checksum.
        method.variants[0].elf = fixture.file("firmware.elf", &elf_fixture(b"different-output"));
        let path = fixture.0.join("run.json");
        let mut report: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        report["firmware_sha256"] = json!(method.variants[0].elf.sha256);
        method.variants[0].positive_report =
            fixture.file("run.json", &serde_json::to_vec(&report).unwrap());
        let identity = fixture.save(&method);
        assert!(
            format!("{:#}", inspect_method(&fixture.0, &identity).unwrap_err())
                .contains("absent from compiled ELF")
        );
    }

    #[test]
    fn bad_positive_stale_configuration_and_changed_capture_artifacts_fail() {
        let fixture = Fixture::new();
        let mut method = fixture.method();
        method.variants[0].positive_raw =
            fixture.file("serial.raw", b"ESP-ROM:esp32c6\nRebooting...\n");
        let identity = fixture.save(&method);
        assert!(inspect_method(&fixture.0, &identity).is_err());
        let method = fixture.method();
        let identity = fixture.save(&method);
        fs::write(
            fixture.0.join("configuration.txt"),
            b"changed configuration",
        )
        .unwrap();
        assert!(
            format!("{:#}", inspect_method(&fixture.0, &identity).unwrap_err())
                .contains("stale method evidence")
        );
        let method = fixture.method();
        fixture.save(&method);
        let admission = admit(
            &fixture.0,
            CapturePurpose::Comparison,
            Some(Path::new("method.json")),
            Wire::Queue,
            "freertos-queue",
            &fixture.0.join("firmware.elf"),
            &fixture.0.join("provenance.json"),
        )
        .unwrap();
        assert!(
            admission
                .check_inputs(b"changed ELF", b"changed provenance")
                .is_err()
        );
        fs::write(fixture.0.join("firmware.elf"), b"new ELF").unwrap();
        assert!(
            admit(
                &fixture.0,
                CapturePurpose::Comparison,
                Some(Path::new("method.json")),
                Wire::Queue,
                "freertos-queue",
                &fixture.0.join("firmware.elf"),
                &fixture.0.join("provenance.json")
            )
            .is_err()
        );
    }

    #[test]
    fn purpose_and_missing_metadata_cannot_promote_a_pilot() {
        let fixture = Fixture::new();
        let method = fixture.method();
        let identity = fixture.save(&method);
        let expected = QualifiedBackend {
            proof: MethodProof {
                id: method.id,
                variant: "native".into(),
                definition: identity,
            },
            elf_sha256: method.variants[0].elf.sha256.clone(),
            provenance_sha256: method.variants[0].provenance.sha256.clone(),
        };
        let mut admission = Admission {
            purpose: CapturePurpose::Qualification,
            elf_sha256: expected.elf_sha256.clone(),
            provenance_sha256: expected.provenance_sha256.clone(),
            method: None,
        };
        let mut report = json!({"schema_version": 4, "firmware_sha256": admission.elf_sha256, "admission": admission,
            "collector_executable_sha256": "new collector"});
        assert!(
            report_admission(&report, Some(&expected))
                .unwrap_err()
                .to_string()
                .contains("qualification pilot")
        );
        report.as_object_mut().unwrap().remove("admission");
        assert!(report_admission(&report, Some(&expected)).is_err());
        report["schema_version"] = json!(3);
        assert!(report_admission(&report, None).is_err());
        report["schema_version"] = json!(4);
        admission.purpose = CapturePurpose::Comparison;
        admission.method = Some(expected.proof.clone());
        report["admission"] = serde_json::to_value(&admission).unwrap();
        report_admission(&report, Some(&expected)).unwrap();
        report["admission"]["method"]["variant"] = json!("different variant");
        assert!(report_admission(&report, Some(&expected)).is_err());
    }

    #[test]
    fn comparison_requires_a_method_and_pilot_cannot_claim_one() {
        let fixture = Fixture::new();
        let method = fixture.method();
        fixture.save(&method);
        for (purpose, method) in [
            (CapturePurpose::Comparison, None),
            (
                CapturePurpose::Qualification,
                Some(Path::new("method.json")),
            ),
        ] {
            assert!(
                admit(
                    &fixture.0,
                    purpose,
                    method,
                    Wire::Queue,
                    "freertos-queue",
                    &fixture.0.join("firmware.elf"),
                    &fixture.0.join("provenance.json")
                )
                .is_err()
            );
        }
        assert!(relative_file(&fixture.0, "../outside").is_err());
        assert!(relative_file(&fixture.0, "/absolute").is_err());
    }

    #[test]
    fn incomplete_metric_mapping_or_relabelled_same_review_cannot_qualify() {
        let fixture = Fixture::new();
        let mut method = fixture.method();
        method.timing_contracts.pop();
        assert!(
            format!(
                "{:#}",
                inspect_method(&fixture.0, &fixture.save(&method)).unwrap_err()
            )
            .contains("exactly cover")
        );
        let mut method = fixture.method();
        method.review.reviewer = " TEST   AUTHOR ".into();
        assert!(
            format!(
                "{:#}",
                inspect_method(&fixture.0, &fixture.save(&method)).unwrap_err()
            )
            .contains("distinct producer")
        );
        method.review.reviewer = "TESTAUTHOR".into();
        assert!(
            format!(
                "{:#}",
                inspect_method(&fixture.0, &fixture.save(&method)).unwrap_err()
            )
            .contains("distinct producer")
        );
        let mut method = fixture.method();
        method.review.adversarial_reproduction = method.review.semantics.clone();
        assert!(
            format!(
                "{:#}",
                inspect_method(&fixture.0, &fixture.save(&method)).unwrap_err()
            )
            .contains("distinct artifact")
        );
    }

    #[test]
    fn archival_replay_checks_report_inventory_and_elf_identity() {
        let fixture = Fixture::new();
        let method = fixture.method();
        let directory = fixture.0.join("results/fixture/runs/01");
        fs::create_dir_all(&directory).unwrap();
        let mut report: Value =
            serde_json::from_slice(&fs::read(fixture.0.join("run.json")).unwrap()).unwrap();
        report["platform"] = json!("esp-idf-freertos");
        report["backend"] = json!("freertos-queue");
        report["collector_executable_sha256"] = json!("fixture only");
        fs::write(
            directory.join("run.json"),
            serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
        let raw = fs::read(fixture.0.join("serial.raw")).unwrap();
        fs::write(directory.join("serial.raw"), &raw).unwrap();
        fs::copy(
            fixture.0.join(&method.variants[0].elf.path),
            directory.join("firmware.elf"),
        )
        .unwrap();
        let mut output = Vec::new();
        crate::write_json(
            &mut output,
            &replay(&raw, Wire::Queue, "freertos-queue").unwrap(),
        )
        .unwrap();
        fs::write(directory.join("replay.json"), output).unwrap();
        assert_eq!(
            replay_archives(&fixture.0).unwrap()["successful_protocol_transcripts"],
            1
        );
        fs::remove_file(directory.join("serial.raw")).unwrap();
        assert!(
            replay_archives(&fixture.0).is_err(),
            "a pass report cannot disappear when its raw file is missing"
        );
        fs::write(directory.join("serial.raw"), &raw).unwrap();
        fs::write(directory.join("firmware.elf"), b"different ELF").unwrap();
        assert!(format!("{:#}", replay_archives(&fixture.0).unwrap_err()).contains("ELF differs"));
    }
}
