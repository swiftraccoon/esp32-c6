//! Limited verification of a source and text snapshot.
//!
//! Original hashes and Git IDs are historical declarations here. This path does
//! not mint method proofs, admit captures, inspect omitted ELFs, or establish
//! pre-capture Git ancestry. Complete-artifact commands remain separate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Deserializer};
use serde_json::{Value, json};

use crate::qualification::{Admission, CapturePurpose};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Retained,
    Modified,
    Omitted,
    Added,
}

fn nullable<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    Option::deserialize(deserializer)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    status: Status,
    #[serde(deserialize_with = "nullable")]
    original_sha256: Option<String>,
    #[serde(deserialize_with = "nullable")]
    public_sha256: Option<String>,
    #[serde(deserialize_with = "nullable")]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    kind: String,
    original_head: String,
    files: Vec<Entry>,
    limitations: Vec<String>,
}

/// Immutable, already checked lineage for the current tracked public files.
#[derive(Debug)]
pub(crate) struct Publication {
    root: PathBuf,
    entries: BTreeMap<String, Entry>,
    original_head: String,
    limitations: Vec<String>,
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn normalized(name: &str) -> Result<()> {
    let path = Path::new(name);
    ensure!(
        !name.is_empty()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
            && !path.starts_with(".git"),
        "publication path must be normalized and repository-relative: {name}"
    );
    Ok(())
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "Git publication inventory failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

fn tracked(root: &Path) -> Result<BTreeSet<String>> {
    git(root, &["ls-files", "--cached", "-z"])?
        .split(|&byte| byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8(name.to_vec()).context("non-UTF-8 publication path"))
        .collect()
}

fn regular(root: &Path, name: &str) -> Result<PathBuf> {
    normalized(name)?;
    let mut path = root.to_path_buf();
    for component in Path::new(name).components() {
        path.push(component);
        ensure!(
            !fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "publication path contains symlink: {name}"
        );
    }
    ensure!(path.is_file(), "publication needs a regular file: {name}");
    Ok(path)
}

impl Publication {
    fn load(root: &Path, manifest: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let top = git(&root, &["rev-parse", "--show-toplevel"])?;
        ensure!(
            Path::new(std::str::from_utf8(&top)?.trim()).canonicalize()? == root,
            "--root must be the Git root"
        );
        let manifest_name = manifest
            .to_str()
            .context("non-UTF-8 publication manifest path")?;
        let actual = tracked(&root)?;
        ensure!(
            actual.contains(manifest_name),
            "publication manifest must be tracked or staged"
        );
        let input: Manifest = serde_json::from_slice(&fs::read(regular(&root, manifest_name)?)?)?;
        ensure!(
            input.schema_version == 1 && input.kind == "source-snapshot-v1",
            "unsupported publication manifest"
        );
        ensure!(
            hex(&input.original_head, 40),
            "publication original_head must be a full historical Git ID"
        );
        ensure!(
            !input.limitations.is_empty()
                && input
                    .limitations
                    .iter()
                    .all(|value| !value.trim().is_empty()),
            "publication needs explicit limitations"
        );
        let mut entries = BTreeMap::new();
        let mut present = BTreeSet::from([manifest_name.to_owned()]);
        for entry in input.files {
            normalized(&entry.path)?;
            ensure!(
                entry.path != manifest_name,
                "publication manifest cannot hash itself"
            );
            for hash in [&entry.original_sha256, &entry.public_sha256]
                .into_iter()
                .flatten()
            {
                ensure!(hex(hash, 64), "invalid publication SHA-256: {}", entry.path);
            }
            if let Some(reason) = &entry.reason {
                ensure!(!reason.trim().is_empty(), "empty publication reason");
            }
            match entry.status {
                Status::Retained => ensure!(
                    entry.original_sha256.is_some() && entry.original_sha256 == entry.public_sha256,
                    "retained publication hashes must be equal: {}",
                    entry.path
                ),
                Status::Modified => ensure!(
                    entry.original_sha256.is_some()
                        && entry.public_sha256.is_some()
                        && entry.original_sha256 != entry.public_sha256
                        && entry.reason.is_some(),
                    "changed publication needs distinct hashes and reason: {}",
                    entry.path
                ),
                Status::Omitted => ensure!(
                    entry.original_sha256.is_some()
                        && entry.public_sha256.is_none()
                        && entry.reason.is_some()
                        && !actual.contains(&entry.path),
                    "omitted publication artifact must not be tracked and needs original identity/reason: {}",
                    entry.path
                ),
                Status::Added => ensure!(
                    entry.original_sha256.is_none()
                        && entry.public_sha256.is_some()
                        && entry.reason.is_some(),
                    "added publication file cannot assert an original identity: {}",
                    entry.path
                ),
            }
            if let Some(expected) = &entry.public_sha256 {
                ensure!(
                    actual.contains(&entry.path),
                    "public artifact is not tracked: {}",
                    entry.path
                );
                ensure!(
                    crate::digest(&fs::read(regular(&root, &entry.path)?)?) == *expected,
                    "public artifact hash mismatch: {}",
                    entry.path
                );
                present.insert(entry.path.clone());
            }
            if Path::new(&entry.path)
                .file_name()
                .is_some_and(|name| name == "serial.raw" || name == "observations.jsonl")
            {
                ensure!(
                    entry.status == Status::Retained,
                    "protocol raw bytes and timing receipts must remain retained: {}",
                    entry.path
                );
            }
            ensure!(
                entries.insert(entry.path.clone(), entry).is_none(),
                "duplicate publication path"
            );
        }
        ensure!(
            actual == present,
            "publication mapping does not exactly cover tracked files"
        );
        Ok(Self {
            root,
            entries,
            original_head: input.original_head,
            limitations: input.limitations,
        })
    }

    pub(crate) fn original_identity(&self, name: &str, expected: Option<&str>) -> Result<()> {
        normalized(name)?;
        let entry = self
            .entries
            .get(name)
            .with_context(|| format!("unmapped original evidence: {name}"))?;
        if let Some(expected) = expected {
            ensure!(
                hex(expected, 64) && entry.original_sha256.as_deref() == Some(expected),
                "recorded original hash declaration differs: {name}"
            );
        }
        Ok(())
    }

    pub(crate) fn require_retained(&self, name: &str) -> Result<()> {
        ensure!(
            self.entries
                .get(name)
                .is_some_and(|entry| entry.status == Status::Retained),
            "historical manifest/measurement must remain byte-identical: {name}"
        );
        Ok(())
    }

    pub(crate) fn require_public(&self, name: &str) -> Result<()> {
        ensure!(
            self.entries
                .get(name)
                .is_some_and(|entry| entry.public_sha256.is_some()),
            "raw evidence must remain public: {name}"
        );
        Ok(())
    }

    pub(crate) fn is_omitted(&self, name: &str) -> bool {
        self.entries
            .get(name)
            .is_some_and(|entry| entry.status == Status::Omitted)
    }

    pub(crate) fn original_paths_under(&self, directory: &str) -> BTreeSet<String> {
        self.entries
            .iter()
            .filter(|(name, entry)| {
                entry.original_sha256.is_some() && name.starts_with(&format!("{directory}/"))
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn relative(&self, path: &Path) -> Result<String> {
        Ok(path
            .strip_prefix(&self.root)?
            .to_str()
            .context("non-UTF-8 archive path")?
            .to_owned())
    }

    fn report_inputs(&self, directory: &Path, report: &Value) -> Result<()> {
        let elf = self.relative(&directory.join("firmware.elf"))?;
        let firmware = report["firmware_sha256"]
            .as_str()
            .context("capture lacks original firmware identity")?;
        self.original_identity(&elf, Some(firmware))?;
        if report["schema_version"] == 4 {
            let admission: Admission = serde_json::from_value(report["admission"].clone())?;
            ensure!(
                admission.elf_sha256 == firmware,
                "capture firmware/admission identity differs"
            );
            let diagnostic = matches!(report["hello"]["protocol"].as_u64(), Some(4 | 5))
                || report["platform"] == "rtic"
                || report["experiment"] == crate::load::EXPERIMENT
                || report["experiment"] == crate::accounting::EXPERIMENT;
            if diagnostic {
                ensure!(
                    admission.purpose == CapturePurpose::Qualification
                        && admission.method.is_none()
                        && report["benchmarks"].is_null()
                        && report["comparative_score"].is_null(),
                    "diagnostic capture cannot be promoted to comparison"
                );
                let name = self.relative(&directory.join("provenance.json"))?;
                self.original_identity(&name, Some(&admission.provenance_sha256))?;
                self.require_public(&name)?;
                ensure!(
                    report["provenance"]
                        == serde_json::from_slice::<Value>(&fs::read(
                            directory.join("provenance.json")
                        )?)?,
                    "public provenance copies differ"
                );
            }
        }
        Ok(())
    }

    fn replay(&self) -> Result<Value> {
        let mut directories = BTreeSet::new();
        let mut unavailable_studies = Vec::new();
        for name in self.entries.keys() {
            if name.starts_with("results/") && name.ends_with("/serial.raw") {
                directories.insert(
                    self.root
                        .join(name)
                        .parent()
                        .context("raw capture has no parent")?
                        .to_owned(),
                );
            }
            if name.starts_with("results/") && name.ends_with("/run.json") && !self.is_omitted(name)
            {
                let report: Value = serde_json::from_slice(&fs::read(self.root.join(name))?)?;
                if report.get("collector_executable_sha256").is_some()
                    || matches!(report["hello"]["protocol"].as_u64(), Some(1..=5))
                    || report["platform"] == "rtic"
                {
                    directories.insert(
                        self.root
                            .join(name)
                            .parent()
                            .context("report has no parent")?
                            .to_owned(),
                    );
                }
            }
            if name.ends_with("/study-manifest.json") && !self.is_omitted(name) {
                unavailable_studies.push(name);
            }
        }
        let mut successful = 0_usize;
        let mut failed = 0_usize;
        for directory in directories {
            let raw_name = self.relative(&directory.join("serial.raw"))?;
            let report_name = self.relative(&directory.join("run.json"))?;
            self.require_retained(&raw_name)?;
            self.require_public(&report_name)?;
            let raw = fs::read(directory.join("serial.raw"))?;
            let report: Value = serde_json::from_slice(&fs::read(directory.join("run.json"))?)?;
            self.report_inputs(&directory, &report)?;
            let console = report["platform"] == "rtic"
                || report["experiment"] == crate::rtic_console::EXPERIMENT
                || self
                    .entries
                    .contains_key(&self.relative(&directory.join("observations.jsonl"))?);
            let replayed = if console {
                let name = self.relative(&directory.join("observations.jsonl"))?;
                self.require_retained(&name)?;
                ensure!(
                    report["schema_version"] == 4
                        && report["experiment"] == crate::rtic_console::EXPERIMENT
                        && report["platform"] == "rtic",
                    "RTIC capture identity was changed"
                );
                ensure!(
                    report["serial_sha256"] == crate::digest(&raw)
                        && report["observations_sha256"]
                            == crate::digest(&fs::read(directory.join("observations.jsonl"))?),
                    "RTIC retained-byte hashes differ"
                );
                crate::rtic_console::validate_bytes(
                    &raw,
                    &fs::read(directory.join("observations.jsonl"))?,
                )
            } else if report["hello"]["protocol"] == 5
                || report["experiment"] == crate::load::EXPERIMENT
            {
                ensure!(
                    report["experiment"] == crate::load::EXPERIMENT,
                    "load experiment changed"
                );
                crate::load::validate_bytes(
                    &raw,
                    serde_json::from_value(report["profile"].clone())?,
                )
            } else if report["hello"]["protocol"] == 4
                || report["experiment"] == crate::accounting::EXPERIMENT
            {
                crate::accounting::validate_bytes(&raw)
            } else if report["hello"]["protocol"] == 3 {
                crate::queue_protocol::validate_bytes(
                    &raw,
                    report["backend"]
                        .as_str()
                        .context("queue backend missing")?,
                    &[],
                )
            } else {
                let boot = crate::validate(
                    std::io::Cursor::new(&raw),
                    report["platform"].as_str().context("platform missing")?,
                );
                boot.and_then(|boot| {
                    Ok(json!({"hello":boot.hello(), "benchmarks":boot.summaries()?}))
                })
            };
            if report["status"] == "failed" {
                ensure!(
                    report["failure"]
                        .as_str()
                        .is_some_and(|reason| !reason.trim().is_empty())
                        && report["benchmarks"].is_null(),
                    "failed capture retains scores or lacks reason"
                );
                if report["experiment"] == crate::load::EXPERIMENT
                    || report["experiment"] == crate::accounting::EXPERIMENT
                {
                    ensure!(
                        report["qualification"].is_null() && report["comparative_score"].is_null(),
                        "failed diagnostic retains favorable qualification"
                    );
                    if report["experiment"] == crate::load::EXPERIMENT {
                        ensure!(
                            report["phases"].is_null(),
                            "failed load retains phase summary"
                        );
                    }
                }
                if console {
                    for key in [
                        "qualification",
                        "console",
                        "summary",
                        "benchmarks",
                        "comparative_score",
                    ] {
                        ensure!(
                            report.get(key) == Some(&Value::Null),
                            "failed RTIC capture retains favorable field: {key}"
                        );
                    }
                    ensure!(
                        replayed.is_err(),
                        "failed RTIC capture cannot become a successful raw/receipt replay"
                    );
                }
                failed += 1;
                continue;
            }
            ensure!(report["status"] == "pass", "unfinished public capture");
            let value = replayed?;
            for (key, field) in value.as_object().context("replay must return object")? {
                ensure!(
                    report[key] == *field,
                    "public capture report differs from replay: {key}"
                );
            }
            let replay_name = self.relative(&directory.join("replay.json"))?;
            self.require_retained(&replay_name)?;
            let mut output = Vec::new();
            crate::write_json(&mut output, &value)?;
            ensure!(
                fs::read(directory.join("replay.json"))? == output,
                "public retained replay differs: {replay_name}"
            );
            successful += 1;
        }
        Ok(
            json!({"successful_raw_capture_replays":successful, "failed_captures_retained":failed,
            "exact_artifact_study_replays":null, "unavailable_study_manifests":unavailable_studies,
            "limits":"Per-boot retained raw/receipt derivation only; original ELF identity, method admission and study qualification are not reproduced"}),
        )
    }
}

/// Verifies a source snapshot without relaxing complete-artifact commands.
pub(crate) fn verify(root: &Path, ledger: &Path, manifest: &Path) -> Result<Value> {
    let publication = Publication::load(root, manifest)?;
    let ledger_name = ledger.to_str().context("ledger path is not UTF-8")?;
    publication.require_retained(ledger_name)?;
    let structure = crate::assurance::verify_public_repository(root, ledger, &publication)?;
    let replay = publication.replay()?;
    let mut counts = BTreeMap::new();
    for entry in publication.entries.values() {
        *counts
            .entry(format!("{:?}", entry.status).to_lowercase())
            .or_insert(0_usize) += 1;
    }
    Ok(
        json!({"schema_version":1, "status":"public_snapshot_consistent", "kind":"source-snapshot-v1",
        "original_head_recorded":publication.original_head, "file_status_counts":counts,
        "ledger_structure":structure, "retained_raw_replay":replay,
        "unavailable_checks":["Original pre-capture Git ancestry and frozen-input commit verification",
            "Omitted original binary/ELF/executable inspection and identity verification",
            "Exact-artifact method qualification and comparative study replay",
            "Native console semantic validation beyond existing supported per-boot adapters"],
        "limitations":publication.limitations}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[derive(Debug)]
    struct Fixture {
        root: PathBuf,
        original: BTreeMap<String, Vec<u8>>,
        manifest: Value,
    }

    impl Fixture {
        fn new(console: bool) -> Self {
            let root = std::env::temp_dir().join(format!(
                "c6-public-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            git(&root, &["init", "--quiet"]).unwrap();
            let original_elf = b"synthetic original ELF identity, never executable or flashed";
            let provenance = b"{\"scope\":\"original fixture\"}\n";
            let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let raw = fs::read(source_root.join(if console {
                "results/rtic-qualification-2026-10-23/runs/01/serial.raw"
            } else {
                "results/idf-load-qualification-2026-10-23/runs/off-01/serial.raw"
            }))
            .unwrap();
            let mut original: BTreeMap<String, Vec<u8>> = BTreeMap::new();
            original.insert(
                "README.md".into(),
                b"[Platform](docs/platform.md)\n[Omitted original](results/control/firmware.elf)\n"
                    .to_vec(),
            );
            original.insert("docs/platform.md".into(), b"# Fixture platform\n".to_vec());
            original.insert("results/control/firmware.elf".into(), original_elf.to_vec());
            original.insert(
                "results/control/provenance.json".into(),
                provenance.to_vec(),
            );
            original.insert("results/control/serial.raw".into(), raw.clone());
            let mut report = json!({"schema_version":4, "status":if console {"failed"} else {"pass"},
                "platform":if console {"rtic"} else {"esp-idf-freertos"},
                "experiment":if console {crate::rtic_console::EXPERIMENT} else {crate::load::EXPERIMENT},
                "profile":"runtime-stats-disabled", "firmware_sha256":crate::digest(original_elf),
                "admission":{"purpose":"qualification","elf_sha256":crate::digest(original_elf),"provenance_sha256":crate::digest(provenance),"method":null},
                "provenance":serde_json::from_slice::<Value>(provenance).unwrap(),
                "collector_executable_sha256":"synthetic fixture", "benchmarks":null, "comparative_score":null,
                "qualification":null,"console":null,"summary":null,"failure":if console {Some("additional ROM startup")} else {None}});
            if console {
                let observations = fs::read(
                    source_root
                        .join("results/rtic-qualification-2026-10-23/runs/01/observations.jsonl"),
                )
                .unwrap();
                report["serial_sha256"] = json!(crate::digest(&raw));
                report["observations_sha256"] = json!(crate::digest(&observations));
                original.insert("results/control/observations.jsonl".into(), observations);
            } else {
                let replay =
                    crate::load::validate_bytes(&raw, crate::load::Profile::RuntimeStatsDisabled)
                        .unwrap();
                for (key, value) in replay.as_object().unwrap() {
                    report[key] = value.clone();
                }
                let mut bytes = Vec::new();
                crate::write_json(&mut bytes, &replay).unwrap();
                original.insert("results/control/replay.json".into(), bytes);
            }
            original.insert(
                "results/control/run.json".into(),
                serde_json::to_vec_pretty(&report).unwrap(),
            );
            original.insert(
                "results/control/INPUTS.sha256".into(),
                format!(
                    "{}  results/control/firmware.elf\n",
                    crate::digest(original_elf)
                )
                .into_bytes(),
            );
            let checksums = original
                .iter()
                .filter(|(name, _)| name.starts_with("results/control/"))
                .map(|(name, bytes)| format!("{}  {name}\n", crate::digest(bytes)))
                .collect::<String>();
            original.insert("results/control/SHA256SUMS".into(), checksums.into_bytes());
            let ledger = json!({"schema_version":1,
                "archives":[{"id":"fixture","path":"results/control","state":"finalized","manifest":"results/control/SHA256SUMS","manifest_base":"repository",
                    "inputs":{"path":"results/control/INPUTS.sha256","freeze_commit":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},"limitations":["Synthetic metadata; no physical claim."]}],
                "claims":[{"id":"raw","statement":"A retained host fixture.","level":if console {"failed"} else {"observed"},"outcome":if console {"failed"} else {"partial"},
                    "scope":"Host parser fixture only.","producer":"fixture","reproduced_by":null,
                    "evidence":[{"path":"results/control/serial.raw","role":"raw_capture","sha256":crate::digest(&raw)}],"limitations":["Not a new physical result."]}],
                "platforms":[{"id":"fixture","label":"fixture","reference":"docs/platform.md"}],"subsystems":[{"id":"timer","label":"timer"}],
                "coverage_defaults":{"driver":"unverified","rust_api":"unverified","pins":"unverified","fixture":"unverified","outcome":"not_run"},"coverage":[]});
            original.insert(
                "quality/evidence.json".into(),
                serde_json::to_vec_pretty(&ledger).unwrap(),
            );
            let mut fixture = Self {
                root,
                original,
                manifest: json!({}),
            };
            for (name, bytes) in &fixture.original {
                if !name.ends_with("firmware.elf") {
                    fixture.write(name, bytes);
                }
            }
            fixture.write("PUBLICATION.md", b"# Source snapshot fixture\n");
            fixture.manifest = json!({"schema_version":1,"kind":"source-snapshot-v1","original_head":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","files":[],"limitations":["Original bytes/Git ancestry unavailable."]});
            fixture.reseal();
            fixture
        }

        fn write(&self, name: &str, bytes: &[u8]) {
            let path = self.root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        fn reseal(&mut self) {
            let names = self
                .original
                .keys()
                .cloned()
                .chain(["PUBLICATION.md".into()]);
            let mut rows = Vec::new();
            for name in names {
                let original = self.original.get(&name).map(|bytes| crate::digest(bytes));
                let public = fs::read(self.root.join(&name))
                    .ok()
                    .map(|bytes| crate::digest(&bytes));
                let status = if public.is_none() {
                    "omitted"
                } else if original.is_none() {
                    "added"
                } else if original == public {
                    "retained"
                } else {
                    "modified"
                };
                rows.push(json!({"path":name,"status":status,"original_sha256":original,"public_sha256":public,
                    "reason":if status=="retained" {None} else {Some("Explicit host fixture derivative")}}));
            }
            self.manifest["files"] = json!(rows);
            self.save();
        }

        fn save(&self) {
            self.write(
                "quality/publication.json",
                &serde_json::to_vec_pretty(&self.manifest).unwrap(),
            );
            git(&self.root, &["add", "--all"]).unwrap();
        }

        fn verify(&self) -> Result<Value> {
            verify(
                &self.root,
                Path::new("quality/evidence.json"),
                Path::new("quality/publication.json"),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn source_snapshot_replays_raw_without_complete_artifact_proofs() {
        let fixture = Fixture::new(false);
        let result = fixture.verify().unwrap();
        assert_eq!(
            result["retained_raw_replay"]["successful_raw_capture_replays"],
            1
        );
        assert!(result["retained_raw_replay"]["exact_artifact_study_replays"].is_null());
        assert_eq!(
            result["ledger_structure"]["declared_omitted_markdown_targets"],
            1
        );
        assert!(
            git(
                &fixture.root,
                &["cat-file", "-e", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]
            )
            .is_err()
        );
        assert!(
            crate::assurance::verify_repository(&fixture.root, Path::new("quality/evidence.json"))
                .is_err()
        );
        assert!(crate::qualification::replay_archives(&fixture.root).is_err());
    }

    #[test]
    fn publication_status_inventory_and_identity_mutations_reject() {
        for fault in 0..10 {
            let mut fixture = Fixture::new(false);
            let rows = fixture.manifest["files"].as_array_mut().unwrap();
            match fault {
                0 => {rows.pop();}
                1 => rows.push(rows[0].clone()),
                2 => rows[0]["path"]=json!("../escape"),
                3 => rows[0]["public_sha256"]=json!("0".repeat(64)),
                4 => {let row=rows.iter_mut().find(|row|row["status"]=="omitted").unwrap(); row["reason"]=Value::Null;}
                5 => {let row=rows.iter_mut().find(|row|row["status"]=="added").unwrap(); row["original_sha256"]=json!("0".repeat(64));}
                6 => rows.push(json!({"path":"quality/publication.json","status":"added","original_sha256":null,"public_sha256":"0".repeat(64),"reason":"self"})),
                7 => {rows[0].as_object_mut().unwrap().remove("public_sha256");}
                8 => fixture.write("unlisted.txt",b"not mapped"),
                9 => fixture.write("results/control/firmware.elf",b"falsely omitted"),
                _=>unreachable!()
            }
            fixture.save();
            assert!(fixture.verify().is_err(), "mutation{fault}");
        }
    }

    #[test]
    fn modifying_historical_ledger_or_manifest_is_not_a_public_freeze() {
        for name in [
            "quality/evidence.json",
            "results/control/INPUTS.sha256",
            "results/control/SHA256SUMS",
        ] {
            let mut fixture = Fixture::new(false);
            let mut bytes = fs::read(fixture.root.join(name)).unwrap();
            bytes.push(b'\n');
            fixture.write(name, &bytes);
            fixture.reseal();
            let error = fixture.verify().unwrap_err();
            assert!(
                format!("{error:#}").contains("byte-identical"),
                "{name}: {error:#}"
            );
        }
    }

    #[test]
    fn numeric_report_tampering_fails_after_updating_public_hash() {
        let mut fixture = Fixture::new(false);
        let path = fixture.root.join("results/control/run.json");
        let mut report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let mean = &mut report["phases"][5]["conditional_latencies"]["send_entry_to_completion_us"]
            ["mean_us"];
        let original = mean.as_f64().unwrap();
        *mean = json!(f64::from_bits(original.to_bits() + 1));
        fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        fixture.reseal();
        assert!(format!("{:#}", fixture.verify().unwrap_err()).contains("differs from replay"));
    }

    #[test]
    fn failed_rtic_remains_failed_and_cannot_gain_favorable_fields() {
        let mut fixture = Fixture::new(true);
        assert_eq!(
            fixture.verify().unwrap()["retained_raw_replay"]["failed_captures_retained"],
            1
        );
        let path = fixture.root.join("results/control/run.json");
        let original: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        for key in ["qualification", "console", "summary", "status"] {
            let mut report = original.clone();
            report[key] = if key == "status" {
                json!("pass")
            } else {
                json!({"passed":true})
            };
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            fixture.reseal();
            assert!(fixture.verify().is_err(), "{key}");
        }
    }

    #[test]
    fn only_exact_declared_omission_links_are_permitted() {
        let mut fixture = Fixture::new(false);
        fixture.write(
            "README.md",
            b"[Not declared omitted](results/control/unlisted.elf)\n",
        );
        fixture.reseal();
        assert!(fixture.verify().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn mapped_symlink_cannot_escape_the_snapshot() {
        let fixture = Fixture::new(false);
        fs::remove_file(fixture.root.join("docs/platform.md")).unwrap();
        std::os::unix::fs::symlink(
            fixture.root.join("README.md"),
            fixture.root.join("docs/platform.md"),
        )
        .unwrap();
        assert!(fixture.verify().is_err());
    }
}
