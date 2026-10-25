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
    Retired,
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
    #[serde(default)]
    relocations: Vec<Relocation>,
    #[serde(default)]
    directory_relocations: Vec<Relocation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Relocation {
    from: String,
    to: String,
}

/// Immutable, already checked lineage for the current tracked public files.
#[derive(Debug)]
pub(crate) struct Publication {
    schema_version: u32,
    root: PathBuf,
    entries: BTreeMap<String, Entry>,
    original_head: String,
    limitations: Vec<String>,
    relocations: BTreeMap<String, String>,
    directory_relocations: BTreeMap<String, String>,
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
            && !name.contains('\\')
            && !name.bytes().any(|byte| byte.is_ascii_control())
            && name
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != "..")
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
            && !path.starts_with(".git"),
        "publication path must be normalized and repository-relative: {name}"
    );
    Ok(())
}

fn relocation_map(rows: Vec<Relocation>) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    let mut destinations = BTreeSet::new();
    for row in rows {
        normalized(&row.from)?;
        normalized(&row.to)?;
        // The historical OS collection can move as a subtree without changing
        // its recorded names or bytes. Other evidence/policy redirects remain
        // forbidden; normal inventory, hash and original-identity checks apply.
        let os_evidence_move = (row.from == "results"
            || row.from.starts_with("results/")
            || row.from == "quality/evidence.json"
            || row.from == "quality/methods"
            || row.from.starts_with("quality/methods/"))
            && row.to == format!("projects/os-evaluation/{}", row.from);
        for name in [&row.from, &row.to] {
            let components: Vec<_> = name.split('/').collect();
            ensure!(
                os_evidence_move
                    || (!name.starts_with("results/")
                        && name != "results"
                        && !name.starts_with("quality/")
                        && name != "quality"
                        && !matches!(components.as_slice(), ["projects", _, "quality", ..])
                        && !name.contains("/results/")
                        && !name.ends_with("/results")
                        && !matches!(
                            Path::new(name).file_name().and_then(|n| n.to_str()),
                            Some("INPUTS.sha256" | "SHA256SUMS")
                        )),
                "frozen archive/policy paths cannot be relocated"
            );
        }
        ensure!(row.from != row.to, "relocation must change the path");
        ensure!(
            destinations.insert(row.to.clone()),
            "duplicate relocation destination"
        );
        ensure!(
            map.insert(row.from, row.to).is_none(),
            "duplicate relocation source"
        );
    }
    ensure!(
        map.values().all(|to| !map.contains_key(to)),
        "relocation chains/cycles are unsupported"
    );
    Ok(map)
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
            (input.schema_version == 1
                && input.kind == "source-snapshot-v1"
                && input.relocations.is_empty()
                && input.directory_relocations.is_empty())
                || (input.schema_version == 2 && input.kind == "source-snapshot-v2"),
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
        let relocations = relocation_map(input.relocations)?;
        let directory_relocations = relocation_map(input.directory_relocations)?;
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
                Status::Retired => ensure!(
                    input.schema_version == 2
                        && entry.original_sha256.is_none()
                        && entry.public_sha256.is_none()
                        && entry.reason.is_some()
                        && !actual.contains(&entry.path),
                    "retired publication addition needs schema 2, absent hashes/path and a reason: {}",
                    entry.path
                ),
            }
            if let Some(expected) = &entry.public_sha256 {
                let current = relocations.get(&entry.path).unwrap_or(&entry.path);
                ensure!(
                    current != manifest_name,
                    "publication manifest cannot hash itself"
                );
                if current != &entry.path {
                    ensure!(
                        !actual.iter().any(|name| name == &entry.path
                            || name.starts_with(&format!("{}/", entry.path))),
                        "relocated source is still tracked or reused as a directory"
                    );
                }
                ensure!(
                    actual.contains(current),
                    "public artifact is not tracked: {}",
                    entry.path
                );
                ensure!(
                    crate::digest(&fs::read(regular(&root, current)?)?) == *expected,
                    "public artifact hash mismatch: {}",
                    entry.path
                );
                ensure!(
                    present.insert(current.clone()),
                    "duplicate physical publication path"
                );
            }
            if Path::new(&entry.path)
                .file_name()
                .is_some_and(|name| name == "serial.raw" || name == "observations.jsonl")
            {
                ensure!(
                    entry.status == Status::Retained
                        || (entry.status == Status::Added
                            && Path::new(&entry.path).starts_with("projects")),
                    "historical protocol raw bytes must remain retained; new project captures must be added: {}",
                    entry.path
                );
            }
            ensure!(
                entries.insert(entry.path.clone(), entry).is_none(),
                "duplicate publication path"
            );
        }
        for from in relocations.keys() {
            ensure!(
                entries
                    .get(from)
                    .is_some_and(|entry| entry.public_sha256.is_some()),
                "relocation lacks a present source declaration"
            );
        }
        for (from, to) in &directory_relocations {
            ensure!(
                !entries.contains_key(from)
                    && !actual
                        .iter()
                        .any(|name| name == from || name.starts_with(&format!("{from}/"))),
                "directory relocation shadows a tracked path"
            );
            ensure!(
                actual
                    .iter()
                    .any(|name| name.starts_with(&format!("{to}/")))
                    && relocations
                        .iter()
                        .any(|(old, new)| old.starts_with(&format!("{from}/"))
                            && new.starts_with(&format!("{to}/"))),
                "directory relocation lacks mapped tracked descendants"
            );
        }
        ensure!(
            actual == present,
            "publication mapping does not exactly cover tracked files"
        );
        let publication = Self {
            schema_version: input.schema_version,
            root,
            entries,
            original_head: input.original_head,
            limitations: input.limitations,
            relocations,
            directory_relocations,
        };
        let mut recorded_paths = BTreeSet::new();
        for path in &actual {
            recorded_paths.insert(publication.recorded(path));
            // An explicit file alias must not hide an omission in any
            // containing directory alias, including overlapping namespaces.
            for (from, to) in &publication.directory_relocations {
                if let Some(suffix) = path.strip_prefix(&format!("{to}/")) {
                    recorded_paths.insert(format!("{from}/{suffix}"));
                }
            }
        }
        for (name, entry) in &publication.entries {
            if matches!(entry.status, Status::Omitted | Status::Retired) {
                let prefix = format!("{name}/");
                ensure!(
                    !actual
                        .iter()
                        .chain(&recorded_paths)
                        .any(|path| path == name || path.starts_with(&prefix)),
                    "omitted publication artifact shadows a tracked path: {name}"
                );
            }
        }
        Ok(publication)
    }

    /// Exact aliases are limited to publication verification, never method admission.
    pub(crate) fn resolved<'a>(&'a self, name: &'a str) -> &'a str {
        self.relocations
            .get(name)
            .or_else(|| self.directory_relocations.get(name))
            .map_or(name, String::as_str)
    }

    /// Recorded names also cover omitted siblings of a relocated archive.
    pub(crate) fn recorded(&self, name: &str) -> String {
        if let Some((from, _)) = self.relocations.iter().find(|(_, to)| to.as_str() == name) {
            return from.clone();
        }
        self.directory_relocations
            .iter()
            .filter_map(|(from, to)| {
                if name == to {
                    Some((to.len(), from.clone()))
                } else {
                    name.strip_prefix(&format!("{to}/"))
                        .map(|suffix| (to.len(), format!("{from}/{suffix}")))
                }
            })
            .max_by_key(|(length, _)| *length)
            .map_or_else(|| name.to_owned(), |(_, recorded)| recorded)
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

    /// Both omitted originals and retired additions are declared unavailable.
    pub(crate) fn is_omitted(&self, name: &str) -> bool {
        self.entries
            .get(name)
            .is_some_and(|entry| matches!(entry.status, Status::Omitted | Status::Retired))
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
        Ok(self.recorded(
            path.strip_prefix(&self.root)?
                .to_str()
                .context("non-UTF-8 archive path")?,
        ))
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
                        .join(self.resolved(name))
                        .parent()
                        .context("raw capture has no parent")?
                        .to_owned(),
                );
            }
            if name.starts_with("results/") && name.ends_with("/run.json") && !self.is_omitted(name)
            {
                let report: Value =
                    serde_json::from_slice(&fs::read(self.root.join(self.resolved(name)))?)?;
                if report.get("collector_executable_sha256").is_some()
                    || matches!(report["hello"]["protocol"].as_u64(), Some(1..=5))
                    || report["platform"] == "rtic"
                {
                    directories.insert(
                        self.root
                            .join(self.resolved(name))
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
    let ledger_name = publication.recorded(ledger.to_str().context("ledger path is not UTF-8")?);
    publication.require_retained(&ledger_name)?;
    let structure =
        crate::assurance::verify_public_repository(root, Path::new(&ledger_name), &publication)?;
    let replay = publication.replay()?;
    let mut counts = BTreeMap::new();
    for entry in publication.entries.values() {
        *counts
            .entry(format!("{:?}", entry.status).to_lowercase())
            .or_insert(0_usize) += 1;
    }
    Ok(
        json!({"schema_version":publication.schema_version, "status":"public_snapshot_consistent", "kind":format!("source-snapshot-v{}", publication.schema_version),
        "original_head_recorded":publication.original_head, "file_status_counts":counts,
        "relocated_source_files":publication.relocations.len(),
        "relocated_directories":publication.directory_relocations.len(),
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

        fn move_os_archive(&mut self) {
            fs::create_dir_all(self.root.join("projects/os-evaluation/quality")).unwrap();
            fs::rename(
                self.root.join("results"),
                self.root.join("projects/os-evaluation/results"),
            )
            .unwrap();
            fs::rename(
                self.root.join("quality/evidence.json"),
                self.root
                    .join("projects/os-evaluation/quality/evidence.json"),
            )
            .unwrap();
            let aliases: Vec<_> = self.manifest["files"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|entry| {
                    let path = entry["path"].as_str().unwrap();
                    entry["public_sha256"].is_string()
                        && (path.starts_with("results/") || path == "quality/evidence.json")
                })
                .map(|entry| {
                    let path = entry["path"].as_str().unwrap();
                    json!({"from":path,"to":format!("projects/os-evaluation/{path}")})
                })
                .collect();
            self.manifest["schema_version"] = json!(2);
            self.manifest["kind"] = json!("source-snapshot-v2");
            self.manifest["relocations"] = json!(aliases);
            self.manifest["directory_relocations"] = json!([
                {"from":"results","to":"projects/os-evaluation/results"},
                {"from":"results/control","to":"projects/os-evaluation/results/control"}
            ]);
            self.save();
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
    fn retired_addition_preserves_sealed_links_without_an_original_identity() {
        let mut fixture = Fixture::new(false);
        let document = "docs/added.md";
        let bytes = b"# Previously added document\n";
        let note = "results/control/README.md";
        let note_bytes = b"[Published addition](../../docs/added.md)\n";
        fixture.original.insert(note.into(), note_bytes.to_vec());
        fixture.write(note, note_bytes);
        let seal = "results/control/SHA256SUMS";
        fixture
            .original
            .get_mut(seal)
            .unwrap()
            .extend_from_slice(format!("{}  {note}\n", crate::digest(note_bytes)).as_bytes());
        fixture.write(seal, &fixture.original[seal]);
        fixture.reseal();
        fixture.write(document, bytes);
        fixture.manifest["schema_version"] = json!(2);
        fixture.manifest["kind"] = json!("source-snapshot-v2");
        fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "path":document, "status":"added", "original_sha256":null,
                "public_sha256":crate::digest(bytes), "reason":"Published fixture addition"
            }));
        fixture.save();
        fixture.verify().unwrap();

        fs::remove_file(fixture.root.join(document)).unwrap();
        let entry = fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap();
        entry["status"] = json!("retired");
        entry["public_sha256"] = Value::Null;
        entry["reason"] = json!(format!(
            "Withdrawn addition; last public SHA-256 {}",
            crate::digest(bytes)
        ));
        fixture.save();
        let result = fixture.verify().unwrap();
        assert_eq!(result["file_status_counts"]["retired"], 1);
        assert_eq!(
            result["ledger_structure"]["declared_omitted_markdown_targets"],
            2
        );
        assert_eq!(fs::read(fixture.root.join(note)).unwrap(), note_bytes);
        assert_eq!(
            fs::read(fixture.root.join(seal)).unwrap(),
            fixture.original[seal]
        );
        let publication =
            Publication::load(&fixture.root, Path::new("quality/publication.json")).unwrap();
        assert!(
            publication
                .original_identity(document, Some(&crate::digest(bytes)))
                .is_err()
        );
        assert!(publication.require_public(document).is_err());
        assert!(!publication.original_paths_under("docs").contains(document));
    }

    #[test]
    fn retirement_requires_schema_two_absent_hashes_and_a_nonempty_reason() {
        for fault in 0..6 {
            let mut fixture = Fixture::new(false);
            fixture.manifest["schema_version"] = json!(2);
            fixture.manifest["kind"] = json!("source-snapshot-v2");
            fs::remove_file(fixture.root.join("PUBLICATION.md")).unwrap();
            let entry = fixture.manifest["files"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["path"] == "PUBLICATION.md")
                .unwrap();
            entry["status"] = json!("retired");
            entry["public_sha256"] = Value::Null;
            entry["reason"] = json!("Withdrawn publication addition");
            match fault {
                0 => {
                    fixture.manifest["schema_version"] = json!(1);
                    fixture.manifest["kind"] = json!("source-snapshot-v1");
                }
                1 => entry["original_sha256"] = json!("a".repeat(64)),
                2 => entry["public_sha256"] = json!("b".repeat(64)),
                3 => entry["reason"] = Value::Null,
                4 => entry["reason"] = json!(""),
                5 => entry["reason"] = json!(" \n\t"),
                _ => unreachable!(),
            }
            fixture.save();
            assert!(
                fixture.verify().is_err(),
                "retirement metadata fault {fault}"
            );
        }
        let mut fixture = Fixture::new(false);
        fs::remove_file(fixture.root.join("docs/platform.md")).unwrap();
        fixture.manifest["schema_version"] = json!(2);
        fixture.manifest["kind"] = json!("source-snapshot-v2");
        let entry = fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["path"] == "docs/platform.md")
            .unwrap();
        entry["status"] = json!("retired");
        entry["public_sha256"] = Value::Null;
        entry["reason"] = json!("An original cannot become a retired addition");
        fixture.save();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("retired publication addition")
        );
    }

    #[test]
    fn retired_additions_cannot_shadow_files_directories_or_gain_file_aliases() {
        for fault in 0..3 {
            let mut fixture = Fixture::new(false);
            fixture.manifest["schema_version"] = json!(2);
            fixture.manifest["kind"] = json!("source-snapshot-v2");
            fs::remove_file(fixture.root.join("PUBLICATION.md")).unwrap();
            let entry = fixture.manifest["files"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["path"] == "PUBLICATION.md")
                .unwrap();
            entry["status"] = json!("retired");
            entry["public_sha256"] = Value::Null;
            fixture.save();
            fixture.verify().unwrap();
            match fault {
                0 => fixture.write("PUBLICATION.md", b"still present"),
                1 => {
                    let name = "PUBLICATION.md/child.md";
                    let bytes = b"# Reused retired name\n";
                    fixture.write(name, bytes);
                    fixture.manifest["files"].as_array_mut().unwrap().push(json!({
                        "path":name, "status":"added", "original_sha256":null,
                        "public_sha256":crate::digest(bytes), "reason":"Contrary directory shadow"
                    }));
                }
                2 => {
                    fixture.manifest["relocations"] = json!([
                        {"from":"PUBLICATION.md", "to":"docs/platform.md"}
                    ])
                }
                _ => unreachable!(),
            }
            fixture.save();
            assert!(fixture.verify().is_err(), "retirement shadow fault {fault}");
        }
    }

    #[test]
    fn raw_captures_and_receipts_cannot_be_retired_additions() {
        for project in [false, true] {
            for basename in ["serial.raw", "observations.jsonl"] {
                let mut fixture = Fixture::new(true);
                let name = if project {
                    format!("projects/fixture/results/probe/{basename}")
                } else {
                    format!("results/control/{basename}")
                };
                fixture.manifest["schema_version"] = json!(2);
                fixture.manifest["kind"] = json!("source-snapshot-v2");
                if project {
                    let bytes = b"Separate project protocol requires its own replay\n";
                    fixture.write(&name, bytes);
                    fixture.manifest["files"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({
                            "path":name, "status":"added", "original_sha256":null,
                            "public_sha256":crate::digest(bytes), "reason":"New project raw fixture"
                        }));
                }
                fixture.save();
                fixture.verify().unwrap();
                fs::remove_file(fixture.root.join(&name)).unwrap();
                let entry = fixture.manifest["files"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|row| row["path"] == name)
                    .unwrap();
                entry["status"] = json!("retired");
                entry["original_sha256"] = Value::Null;
                entry["public_sha256"] = Value::Null;
                entry["reason"] = json!("Attempted raw retirement");
                fixture.save();
                assert!(
                    fixture
                        .verify()
                        .unwrap_err()
                        .to_string()
                        .contains("historical protocol raw bytes must remain retained"),
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn project_archive_move_preserves_replay_and_recorded_identities() {
        for console in [false, true] {
            let mut fixture = Fixture::new(console);
            fixture.move_os_archive();
            let outcome = fixture.verify().unwrap();
            assert_eq!(
                outcome["retained_raw_replay"]["successful_raw_capture_replays"],
                usize::from(!console)
            );
            assert_eq!(
                outcome["retained_raw_replay"]["failed_captures_retained"],
                usize::from(console)
            );
            assert_eq!(
                outcome,
                verify(
                    &fixture.root,
                    Path::new("projects/os-evaluation/quality/evidence.json"),
                    Path::new("quality/publication.json"),
                )
                .unwrap()
            );
            fixture.write(
                "projects/os-evaluation/results/control/serial.raw",
                b"changed",
            );
            assert!(
                fixture
                    .verify()
                    .unwrap_err()
                    .to_string()
                    .contains("hash mismatch")
            );
        }
    }

    #[test]
    fn relocated_omissions_cannot_shadow_present_physical_files_or_directories() {
        let mut accepted = Vec::new();
        for directory in [false, true] {
            let mut fixture = Fixture::new(false);
            fixture.move_os_archive();
            let derivative = "projects/os-evaluation/results/control/task-accounting-abi.md";
            let derivative_bytes = b"A separately identified excerpt, not a frozen original.\n";
            fixture.write(derivative, derivative_bytes);
            fixture.manifest["files"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "path":derivative, "status":"added", "original_sha256":null,
                    "public_sha256":crate::digest(derivative_bytes), "reason":"New derived excerpt"
                }));
            fixture.save();
            fixture.verify().unwrap();
            let destination = if directory {
                "projects/os-evaluation/results/control/firmware.elf/child.txt"
            } else {
                "projects/os-evaluation/results/control/firmware.elf"
            };
            let bytes = b"new bytes cannot occupy an omitted historical artifact name";
            fixture.write(destination, bytes);
            fixture.manifest["files"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "path":destination, "status":"added", "original_sha256":null,
                    "public_sha256":crate::digest(bytes), "reason":"Contrary host fixture"
                }));
            fixture.save();
            if fixture.verify().is_ok() {
                accepted.push(if directory { "directory" } else { "file" });
            }
        }
        assert!(
            accepted.is_empty(),
            "omitted artifact shadows accepted: {accepted:?}"
        );
    }

    #[test]
    fn moved_archive_derivatives_can_reference_declared_omitted_siblings() {
        let mut fixture = Fixture::new(false);
        fixture.move_os_archive();
        let path = "projects/os-evaluation/results/control/availability.md";
        let bytes = b"Original unavailable input: [ELF](firmware.elf).\n";
        fixture.write(path, bytes);
        fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "path":path, "status":"added", "original_sha256":null,
                "public_sha256":crate::digest(bytes), "reason":"Current availability note"
            }));
        fixture.save();
        let result = fixture.verify().unwrap();
        assert_eq!(
            result["ledger_structure"]["declared_omitted_markdown_targets"],
            2
        );
    }

    #[test]
    fn moved_archives_reject_omission_aliases_and_raw_retirement() {
        let mut fixture = Fixture::new(false);
        fixture.move_os_archive();
        fixture.verify().unwrap();
        fixture.manifest["relocations"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "from":"results/control/firmware.elf",
                "to":"projects/os-evaluation/results/control/firmware.elf"
            }));
        fixture.save();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("relocation lacks a present source declaration")
        );

        let mut fixture = Fixture::new(true);
        fixture.move_os_archive();
        fixture.verify().unwrap();
        let raw_name = "results/control/serial.raw";
        let row = fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["path"] == raw_name)
            .unwrap();
        row["status"] = json!("omitted");
        row["public_sha256"] = Value::Null;
        row["reason"] = json!("A failed physical capture must not be retired selectively");
        fixture.manifest["relocations"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["from"] != raw_name);
        fs::remove_file(
            fixture
                .root
                .join("projects/os-evaluation/results/control/serial.raw"),
        )
        .unwrap();
        fixture.save();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("historical protocol raw bytes must remain retained")
        );
    }

    #[test]
    fn unrelated_file_alias_cannot_hide_a_directory_omission_shadow() {
        for status in ["omitted", "retired"] {
            let mut fixture = Fixture::new(false);
            let platform = "projects/fixture/docs/platform.md";
            let reused = "projects/fixture/docs/retired.md";
            let bytes = b"a different source cannot reuse an omitted alias destination";
            fixture.write(platform, &fixture.original["docs/platform.md"]);
            fs::remove_file(fixture.root.join("docs/platform.md")).unwrap();
            fixture.write(reused, bytes);
            fixture.manifest["files"].as_array_mut().unwrap().extend([
            json!({
                "path":"docs/retired.md", "status":status,
                "original_sha256":if status == "omitted" {Some(crate::digest(b"original retired document"))} else {None},
                "public_sha256":null, "reason":"Retired document"
            }),
            json!({
                "path":"different-source.md", "status":"retained",
                "original_sha256":crate::digest(bytes),
                "public_sha256":crate::digest(bytes), "reason":null
            }),
        ]);
            fixture.manifest["schema_version"] = json!(2);
            fixture.manifest["kind"] = json!("source-snapshot-v2");
            fixture.manifest["relocations"] = json!([
                {"from":"docs/platform.md", "to":platform},
                {"from":"different-source.md", "to":reused}
            ]);
            fixture.manifest["directory_relocations"] =
                json!([{"from":"docs", "to":"projects/fixture/docs"}]);
            fixture.save();
            assert!(
                fixture
                    .verify()
                    .unwrap_err()
                    .to_string()
                    .contains("omitted publication artifact shadows a tracked path")
            );
        }
    }

    #[test]
    fn archive_relative_manifests_relocate_without_enabling_full_verification_aliases() {
        let mut fixture = Fixture::new(false);
        let manifest_name = "results/control/SHA256SUMS";
        let relative = String::from_utf8(fixture.original[manifest_name].clone())
            .unwrap()
            .replace("  results/control/", "  ")
            .into_bytes();
        fixture
            .original
            .insert(manifest_name.into(), relative.clone());
        fixture.write(manifest_name, &relative);
        let ledger_name = "quality/evidence.json";
        let mut ledger: Value = serde_json::from_slice(&fixture.original[ledger_name]).unwrap();
        ledger["archives"][0]["manifest_base"] = json!("archive");
        let ledger = serde_json::to_vec_pretty(&ledger).unwrap();
        fixture.original.insert(ledger_name.into(), ledger.clone());
        fixture.write(ledger_name, &ledger);
        fixture.reseal();
        fixture.verify().unwrap();
        fixture.move_os_archive();
        fixture.verify().unwrap();
        assert!(
            crate::assurance::verify_repository(
                &fixture.root,
                Path::new("projects/os-evaluation/quality/evidence.json")
            )
            .unwrap_err()
            .to_string()
            .contains("archive must be an immediate directory")
        );
        assert!(crate::qualification::replay_archives(&fixture.root).is_err());
    }

    #[test]
    fn moved_archive_manifest_rows_cannot_escape_the_recorded_collection() {
        for outside in [
            "results/other/firmware.elf",
            "results/control-suffix/firmware.elf",
            "results/control/../other/firmware.elf",
            "/results/control/firmware.elf",
        ] {
            let mut fixture = Fixture::new(false);
            let manifest_name = "results/control/INPUTS.sha256";
            let changed = format!(
                "{}  {outside}\n",
                crate::digest(&fixture.original["results/control/firmware.elf"])
            )
            .into_bytes();
            let old_digest = crate::digest(&fixture.original[manifest_name]);
            let new_digest = crate::digest(&changed);
            fixture
                .original
                .insert(manifest_name.into(), changed.clone());
            fixture.write(manifest_name, &changed);
            let seal_name = "results/control/SHA256SUMS";
            let seal = String::from_utf8(fixture.original[seal_name].clone())
                .unwrap()
                .replace(&old_digest, &new_digest)
                .into_bytes();
            fixture.original.insert(seal_name.into(), seal.clone());
            fixture.write(seal_name, &seal);
            fixture.reseal();
            fixture.move_os_archive();
            let message = fixture.verify().unwrap_err().to_string();
            assert!(
                message.contains("checksum path leaves its archive")
                    || message.contains("publication path must be normalized"),
                "unexpected rejection for {outside}: {message}"
            );
        }
    }

    #[test]
    fn evidence_moves_cannot_change_project_or_archive_names() {
        for (from, to) in [
            (
                "results/control/serial.raw",
                "projects/other/results/control/serial.raw",
            ),
            (
                "results/control/serial.raw",
                "projects/os-evaluation/results/renamed/serial.raw",
            ),
            (
                "quality/evidence.json",
                "projects/os-evaluation/quality/other.json",
            ),
            (
                "quality/privacy-allowlist.json",
                "projects/os-evaluation/quality/privacy-allowlist.json",
            ),
        ] {
            assert!(
                relocation_map(vec![Relocation {
                    from: from.into(),
                    to: to.into()
                }])
                .is_err()
            );
        }
    }

    #[test]
    fn moved_source_keeps_original_identity_and_resolves_historical_references() {
        let mut fixture = Fixture::new(false);
        let destination = "projects/os-evaluation/docs/platform.md";
        fixture.write(destination, &fixture.original["docs/platform.md"]);
        fs::remove_file(fixture.root.join("docs/platform.md")).unwrap();
        fixture.manifest["schema_version"] = json!(2);
        fixture.manifest["kind"] = json!("source-snapshot-v2");
        fixture.manifest["relocations"] = json!([{"from":"docs/platform.md","to":destination}]);
        fixture.manifest["directory_relocations"] =
            json!([{"from":"docs","to":"projects/os-evaluation/docs"}]);
        fixture.save();
        let result = fixture.verify().unwrap();
        assert_eq!(result["relocated_source_files"], 1);
        assert_eq!(
            result["retained_raw_replay"]["successful_raw_capture_replays"],
            1
        );
        let publication =
            Publication::load(&fixture.root, Path::new("quality/publication.json")).unwrap();
        publication
            .original_identity(
                "docs/platform.md",
                Some(&crate::digest(&fixture.original["docs/platform.md"])),
            )
            .unwrap();
        assert!(publication.original_identity(destination, None).is_err());
        fixture.write(destination, b"changed without a new public hash");
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("hash mismatch")
        );
    }

    #[test]
    fn relocation_declarations_reject_aliases_escapes_missing_files_and_frozen_data() {
        let invalid = [
            json!([{"from":"docs/platform.md","to":"projects/moved.md"},{"from":"docs/platform.md","to":"projects/other.md"}]),
            json!([{"from":"docs/platform.md","to":"projects/moved.md"},{"from":"PUBLICATION.md","to":"projects/moved.md"}]),
            json!([{"from":"docs/platform.md","to":"../outside.md"}]),
            json!([{"from":"docs/platform.md","to":"projects/./moved.md"}]),
            json!([{"from":"docs/platform.md","to":"projects//moved.md"}]),
            json!([{"from":"docs/platform.md","to":"PUBLICATION.md"},{"from":"PUBLICATION.md","to":"docs/platform.md"}]),
            json!([{"from":"docs/platform.md","to":"projects/missing.md"}]),
            json!([{"from":"unknown.md","to":"PUBLICATION.md"}]),
            json!([{"from":"results/control/serial.raw","to":"projects/moved.raw"}]),
            json!([{"from":"quality/evidence.json","to":"projects/moved.json"}]),
            json!([{"from":"projects/fixture/quality/evidence.json","to":"projects/moved.json"}]),
            json!([{"from":"docs/platform.md","to":"projects/control/INPUTS.sha256"}]),
        ];
        for aliases in invalid {
            let mut fixture = Fixture::new(false);
            fixture.manifest["schema_version"] = json!(2);
            fixture.manifest["kind"] = json!("source-snapshot-v2");
            fixture.manifest["relocations"] = aliases;
            fixture.save();
            assert!(fixture.verify().is_err());
        }
        let mut fixture = Fixture::new(false);
        fixture.manifest["relocations"] =
            json!([{"from":"docs/platform.md","to":"projects/moved.md"}]);
        fixture.save();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("unsupported publication")
        );
    }

    #[cfg(unix)]
    #[test]
    fn relocated_files_cannot_use_symlink_targets_or_untracked_copies() {
        let mut fixture = Fixture::new(false);
        let destination = "projects/moved.md";
        fixture.write(destination, &fixture.original["docs/platform.md"]);
        fs::remove_file(fixture.root.join("docs/platform.md")).unwrap();
        fixture.manifest["schema_version"] = json!(2);
        fixture.manifest["kind"] = json!("source-snapshot-v2");
        fixture.manifest["relocations"] = json!([{"from":"docs/platform.md","to":destination}]);
        fixture.save();
        fixture.verify().unwrap();
        git(&fixture.root, &["rm", "--cached", destination]).unwrap();
        assert!(fixture.verify().is_err());
        fixture.save();
        fs::remove_file(fixture.root.join(destination)).unwrap();
        std::os::unix::fs::symlink("../PUBLICATION.md", fixture.root.join(destination)).unwrap();
        fixture.save();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
    }

    #[test]
    fn relocated_file_names_cannot_shadow_new_tracked_directories() {
        let mut fixture = Fixture::new(false);
        fixture.write("projects/moved.md", &fixture.original["docs/platform.md"]);
        fs::remove_file(fixture.root.join("docs/platform.md")).unwrap();
        fixture.write("docs/platform.md/child.md", b"# Reused namespace\n");
        fixture.manifest["schema_version"] = json!(2);
        fixture.manifest["kind"] = json!("source-snapshot-v2");
        fixture.manifest["relocations"] =
            json!([{"from":"docs/platform.md","to":"projects/moved.md"}]);
        fixture.manifest["files"].as_array_mut().unwrap().push(json!({"path":"docs/platform.md/child.md","status":"added","original_sha256":null,"public_sha256":crate::digest(b"# Reused namespace\n"),"reason":"Host fixture"}));
        fixture.save();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("reused as a directory")
        );
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
    fn new_project_raw_is_added_without_reclassifying_historical_captures() {
        let mut fixture = Fixture::new(false);
        let path = "projects/new-project/results/probe/serial.raw";
        let raw = b"A new project's protocol is checked by its own replay command.\n";
        fixture.write(path, raw);
        fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "path":path, "status":"added", "original_sha256":null,
                "public_sha256":crate::digest(raw), "reason":"New project functional probe"
            }));
        fixture.save();
        fixture.verify().unwrap();

        let row = fixture.manifest["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["path"] == "results/control/serial.raw")
            .unwrap();
        row["status"] = json!("added");
        row["original_sha256"] = Value::Null;
        row["reason"] = json!("Attempted historical reclassification");
        fixture.save();
        assert!(
            format!("{:#}", fixture.verify().unwrap_err())
                .contains("historical protocol raw bytes")
        );
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
