//! Read-only evidence, archive and coverage consistency checks.
//!
//! Role-tagged, content-addressed records make unsupported promotions visible.
//! They do not establish semantic truth, authenticated authorship or independent
//! physical measurement. No command from a ledger or Markdown file is executed.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    schema_version: u32,
    archives: Vec<Archive>,
    claims: Vec<Claim>,
    platforms: Vec<NamedReference>,
    subsystems: Vec<Named>,
    coverage_defaults: Defaults,
    coverage: Vec<Coverage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Archive {
    id: String,
    path: String,
    state: ArchiveState,
    manifest: Option<String>,
    manifest_base: ManifestBase,
    inputs: Option<FrozenInputs>,
    limitations: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ArchiveState {
    Finalized,
    LegacyUnsealed,
    Collecting,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ManifestBase {
    Repository,
    Archive,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenInputs {
    path: String,
    freeze_commit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    id: String,
    statement: String,
    level: Level,
    outcome: Outcome,
    scope: String,
    producer: String,
    reproduced_by: Option<String>,
    evidence: Vec<Evidence>,
    limitations: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Level {
    SourceSupported,
    BuildChecked,
    Observed,
    OperatorReported,
    IndependentlyReproduced,
    Failed,
    Unmeasured,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Partial,
    Qualified,
    Failed,
    NotRun,
    Blocked,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    path: String,
    role: Role,
    sha256: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Role {
    Source,
    BuildLog,
    RawCapture,
    OperatorRecord,
    Reproduction,
    Review,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedReference {
    id: String,
    label: String,
    reference: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Named {
    id: String,
    label: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defaults {
    driver: String,
    rust_api: String,
    pins: String,
    fixture: String,
    outcome: Outcome,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Coverage {
    platform: String,
    subsystem: String,
    driver: Driver,
    rust_api: RustApi,
    pins: Pins,
    fixture: FixtureState,
    outcome: Outcome,
    scope: String,
    claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Driver {
    Upstream,
    Community,
    Missing,
    Unverified,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum RustApi {
    Available,
    Missing,
    BuildFailed,
    Unverified,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Pins {
    Known,
    Unknown,
    NotRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum FixtureState {
    Available,
    Missing,
    Unverified,
    NotRequired,
}

#[derive(Debug)]
struct Repository<'a> {
    root: PathBuf,
    tracked: BTreeSet<String>,
    publication: Option<&'a crate::public_snapshot::Publication>,
    omitted_links: Cell<usize>,
    relocated_links: Cell<usize>,
}

impl Repository<'_> {
    fn open(root: &Path) -> Result<Self> {
        let root = root.canonicalize().context("resolving repository root")?;
        let top = git(&root, &["rev-parse", "--show-toplevel"])?;
        let top = Path::new(std::str::from_utf8(&top)?.trim()).canonicalize()?;
        ensure!(root == top, "--root must be the Git repository root");
        let files = git(&root, &["ls-files", "--cached", "-z"])?;
        let tracked = files
            .split(|byte| *byte == 0)
            .filter(|name| !name.is_empty())
            .map(|name| String::from_utf8(name.to_vec()).context("Git path is not UTF-8"))
            .collect::<Result<_>>()?;
        Ok(Self {
            root,
            tracked,
            publication: None,
            omitted_links: Cell::new(0),
            relocated_links: Cell::new(0),
        })
    }

    fn path(&self, name: &str) -> Result<PathBuf> {
        let name = self
            .publication
            .map_or(name, |publication| publication.resolved(name));
        let relative = Path::new(name);
        ensure!(
            !name.is_empty()
                && relative
                    .components()
                    .all(|part| matches!(part, Component::Normal(_))),
            "path must be a normalized repository-relative path: {name}"
        );
        let path = self
            .root
            .join(relative)
            .canonicalize()
            .with_context(|| format!("missing referenced path: {name}"))?;
        ensure!(
            path.starts_with(&self.root),
            "path escapes repository: {name}"
        );
        let mut component_path = self.root.clone();
        for component in relative.components() {
            component_path.push(component.as_os_str());
            ensure!(
                !fs::symlink_metadata(&component_path)?
                    .file_type()
                    .is_symlink(),
                "referenced path contains a symlink: {name}"
            );
        }
        Ok(path)
    }

    fn file(&self, name: &str) -> Result<PathBuf> {
        let name = self
            .publication
            .map_or(name, |publication| publication.resolved(name));
        let path = self.path(name)?;
        ensure!(path.is_file(), "expected a regular file: {name}");
        ensure!(
            self.tracked.contains(name),
            "file is not Git-tracked or staged: {name}"
        );
        Ok(path)
    }

    fn identity(&self, name: &str, expected: Option<&str>) -> Result<()> {
        if let Some(publication) = self.publication {
            publication.original_identity(name, expected)
        } else {
            let path = self.file(name)?;
            if let Some(expected) = expected {
                ensure!(digest(&path)? == expected, "evidence hash mismatch: {name}");
            }
            Ok(())
        }
    }
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("running Git for repository identity")?;
    ensure!(
        output.status.success(),
        "Git check failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

fn nonempty(value: &str, label: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "{label} must not be empty");
    Ok(())
}

fn limitations(values: &[String], label: &str) -> Result<()> {
    ensure!(!values.is_empty(), "{label} needs explicit limitations");
    for value in values {
        nonempty(value, label)?;
    }
    Ok(())
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

/// Verifies a typed evidence ledger without changing files or opening a device.
///
/// Errors identify malformed, missing, untracked, escaping, changed or
/// structurally unsupported records. A successful result is consistency evidence,
/// not a semantic or physical qualification decision.
pub(crate) fn verify_repository(root: &Path, ledger_path: &Path) -> Result<Value> {
    verify_repository_inner(root, ledger_path, None)
}

/// Checks public derivative structure without claiming original-artifact access.
pub(crate) fn verify_public_repository(
    root: &Path,
    ledger_path: &Path,
    publication: &crate::public_snapshot::Publication,
) -> Result<Value> {
    verify_repository_inner(root, ledger_path, Some(publication))
}

fn verify_repository_inner(
    root: &Path,
    ledger_path: &Path,
    publication: Option<&crate::public_snapshot::Publication>,
) -> Result<Value> {
    let mut repo = Repository::open(root)?;
    repo.publication = publication;
    let ledger_name = ledger_path.to_str().context("ledger path is not UTF-8")?;
    let components: Vec<_> = ledger_name.split('/').collect();
    let results_root = if components.first() == Some(&"projects") {
        ensure!(
            components.len() == 4 && components[2] == "quality",
            "project ledger must be projects/<project>/quality/<file>"
        );
        format!("projects/{}/results", components[1])
    } else {
        "results".to_owned()
    };
    let ledger: Ledger = serde_json::from_reader(File::open(repo.file(ledger_name)?)?)
        .context("parsing typed evidence ledger")?;
    ensure!(
        ledger.schema_version == 1,
        "unsupported evidence-ledger schema"
    );
    let mut archive_ids = BTreeSet::new();
    let mut archive_paths = BTreeSet::new();
    let mut sealed = 0_usize;
    let mut manifest_entries = 0_usize;
    let mut warnings = Vec::new();
    for archive in &ledger.archives {
        nonempty(&archive.id, "archive id")?;
        ensure!(
            archive_ids.insert(&archive.id),
            "duplicate archive id: {}",
            archive.id
        );
        ensure!(
            archive_paths.insert(archive.path.clone()),
            "duplicate archive path: {}",
            archive.path
        );
        ensure!(
            Path::new(&archive.path).parent() == Some(Path::new(&results_root)),
            "archive must be an immediate directory under its ledger's results root: {}",
            archive.path
        );
        let directory = repo.path(&archive.path)?;
        ensure!(
            directory.is_dir(),
            "archive is not a directory: {}",
            archive.path
        );
        limitations(&archive.limitations, "archive")?;
        match archive.state {
            ArchiveState::Finalized => {
                let required = format!("{}/SHA256SUMS", archive.path);
                ensure!(
                    archive.manifest.as_deref() == Some(required.as_str()),
                    "finalized archive needs its SHA256SUMS: {}",
                    archive.path
                );
                let entries = verify_manifest(&repo, &required, &directory, archive.manifest_base)?;
                let actual = if let Some(publication) = publication {
                    publication.original_paths_under(&archive.path)
                } else {
                    inventory(&directory, &repo.root)?
                };
                let required_files: BTreeSet<_> = actual
                    .into_iter()
                    .filter(|name| name != &required)
                    .collect();
                ensure!(
                    entries == required_files,
                    "archive manifest coverage differs (unlisted or missing artifacts): {}",
                    archive.path
                );
                manifest_entries += entries.len();
                sealed += 1;
            }
            ArchiveState::LegacyUnsealed => {
                ensure!(
                    archive.manifest.is_none(),
                    "legacy-unsealed archive must not claim a manifest"
                );
                warnings.push(format!(
                    "historical collection has no finalized checksum seal: {}",
                    archive.path
                ));
            }
            ArchiveState::Collecting => {
                ensure!(
                    archive.manifest.is_none(),
                    "collecting archive must not claim finalized SHA256SUMS"
                );
                warnings.push(format!("collection is unfinished: {}", archive.path));
            }
        }
        let expected_inputs = format!("{}/INPUTS.sha256", archive.path);
        if directory.join("INPUTS.sha256").exists() {
            let inputs = archive
                .inputs
                .as_ref()
                .context("existing INPUTS.sha256 must be registered with its real freeze commit")?;
            ensure!(
                inputs.path == expected_inputs,
                "unexpected INPUTS manifest path"
            );
            let path = repo.file(&inputs.path)?;
            if let Some(publication) = publication {
                publication.require_retained(&inputs.path)?;
            }
            if let Some(commit) = &inputs.freeze_commit {
                ensure!(
                    hex(commit, 40) || hex(commit, 64),
                    "freeze commit must be a full hexadecimal object id"
                );
                if publication.is_none() {
                    let frozen = git(&repo.root, &["show", &format!("{commit}:{}", inputs.path)])?;
                    ensure!(
                        frozen == fs::read(path)?,
                        "INPUTS manifest changed since freeze: {}",
                        inputs.path
                    );
                } else {
                    warnings.push(format!("Historical freeze commit recorded but unavailable for public verification: {commit}"));
                }
            } else {
                ensure!(
                    archive.state == ArchiveState::Collecting,
                    "only collecting inputs may lack a freeze commit"
                );
                warnings.push(format!(
                    "collection inputs are not frozen: {}",
                    archive.path
                ));
            }
            verify_manifest(&repo, &inputs.path, &directory, ManifestBase::Repository)?;
        } else {
            ensure!(
                archive.inputs.is_none(),
                "registered input manifest is missing: {}",
                archive.path
            );
        }
    }
    let actual_archives = fs::read_dir(repo.path(&results_root)?)?
        .map(|entry| -> Result<Option<String>> {
            let entry = entry?;
            ensure!(
                !entry.file_type()?.is_symlink(),
                "results directory contains a symlink"
            );
            Ok(entry
                .file_type()?
                .is_dir()
                .then(|| format!("{results_root}/{}", entry.file_name().to_string_lossy())))
        })
        .filter_map(|entry| entry.transpose())
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(
        archive_paths == actual_archives,
        "registered archives differ from actual results directories"
    );

    let mut claims = BTreeMap::new();
    for claim in &ledger.claims {
        nonempty(&claim.id, "claim id")?;
        ensure!(
            claims.insert(claim.id.as_str(), claim).is_none(),
            "duplicate claim id: {}",
            claim.id
        );
        check_claim(&repo, claim).with_context(|| format!("claim {}", claim.id))?;
        if claim.outcome == Outcome::Qualified {
            for evidence in &claim.evidence {
                ensure!(
                    !ledger
                        .archives
                        .iter()
                        .any(|archive| archive.state != ArchiveState::Finalized
                            && evidence.path.starts_with(&format!("{}/", archive.path))),
                    "qualified claim depends on an unfinished/unsealed archive: {}",
                    claim.id
                );
            }
        }
    }
    let mut platforms = BTreeSet::new();
    for platform in &ledger.platforms {
        nonempty(&platform.id, "platform id")?;
        nonempty(&platform.label, "platform label")?;
        ensure!(
            platforms.insert(platform.id.as_str()),
            "duplicate platform id"
        );
        repo.file(&platform.reference)?;
    }
    let mut subsystems = BTreeSet::new();
    for subsystem in &ledger.subsystems {
        nonempty(&subsystem.id, "subsystem id")?;
        nonempty(&subsystem.label, "subsystem label")?;
        ensure!(
            subsystems.insert(subsystem.id.as_str()),
            "duplicate subsystem id"
        );
    }
    ensure!(
        !platforms.is_empty() && !subsystems.is_empty(),
        "coverage dimensions must not be empty"
    );
    let defaults = &ledger.coverage_defaults;
    ensure!(
        defaults.driver == "unverified"
            && defaults.rust_api == "unverified"
            && defaults.pins == "unverified"
            && defaults.fixture == "unverified"
            && defaults.outcome == Outcome::NotRun,
        "coverage defaults must remain unknown/not_run"
    );
    let mut pairs = BTreeSet::new();
    let mut outcomes = BTreeMap::new();
    for coverage in &ledger.coverage {
        ensure!(
            platforms.contains(coverage.platform.as_str())
                && subsystems.contains(coverage.subsystem.as_str()),
            "coverage references an unknown platform/subsystem"
        );
        ensure!(
            pairs.insert((coverage.platform.as_str(), coverage.subsystem.as_str())),
            "duplicate coverage pair"
        );
        check_coverage(coverage, &claims)?;
        *outcomes
            .entry(outcome_name(coverage.outcome))
            .or_insert(0_usize) += 1;
    }
    let total = platforms
        .len()
        .checked_mul(subsystems.len())
        .context("coverage size overflow")?;
    let unknown = total
        .checked_sub(pairs.len())
        .context("coverage overrides exceed dimensions")?;
    let links = check_markdown(&repo)?;
    let mut result = json!({
        "schema_version": 1, "status": "consistent", "archives": ledger.archives.len(),
        "finalized_archives": sealed, "verified_manifest_entries": manifest_entries,
        "claims": claims.len(), "platforms": platforms.len(), "domains": subsystems.len(),
        "coverage_pairs": total, "explicit_pairs": pairs.len(), "default_unknown_not_run_pairs": unknown,
        "explicit_outcomes": outcomes, "checked_local_markdown_links": links, "warnings": warnings,
        "limits": ["Hashes and roles check identity/structure, not semantic support or authenticated independence", "Unknown coverage is not unsupported hardware; consistency is not project completion"]
    });
    if publication.is_some() {
        let fields = result
            .as_object_mut()
            .context("verification result must be an object")?;
        fields.remove("finalized_archives");
        fields.remove("verified_manifest_entries");
        result["status"] = json!("public_derivative_structure_consistent");
        result["historical_finalized_archive_declarations"] = json!(sealed);
        result["checked_original_hash_declarations"] = json!(manifest_entries);
        result["declared_omitted_markdown_targets"] = json!(repo.omitted_links.get());
        result["declared_relocated_markdown_targets"] = json!(repo.relocated_links.get());
        result["limits"] = json!([
            "Public bytes, declared lineage and ledger structure only; original artifacts and freeze ancestry are not verified",
            "Historical claim classifications/outcomes are preserved declarations, not new physical qualification",
            "Links to explicitly omitted artifacts identify unavailable original evidence, not downloadable public artifacts"
        ]);
    }
    Ok(result)
}

fn verify_manifest(
    repo: &Repository,
    name: &str,
    archive: &Path,
    base: ManifestBase,
) -> Result<BTreeSet<String>> {
    let archive_name = archive
        .strip_prefix(&repo.root)?
        .to_str()
        .context("archive path is not UTF-8")?;
    let archive_name = repo.publication.map_or_else(
        || archive_name.to_owned(),
        |publication| publication.recorded(archive_name),
    );
    let contents = fs::read_to_string(repo.file(name)?)?;
    if let Some(publication) = repo.publication {
        publication.require_retained(name)?;
    }
    ensure!(!contents.is_empty(), "empty checksum manifest: {name}");
    let mut entries = BTreeSet::new();
    for (index, line) in contents.lines().enumerate() {
        let (expected, path) = line
            .split_once("  ")
            .with_context(|| format!("invalid checksum row {} in {name}", index + 1))?;
        ensure!(hex(expected, 64), "invalid SHA-256 in {name}");
        ensure!(!path.is_empty(), "empty checksum path in {name}");
        let path = match base {
            ManifestBase::Repository => path.to_owned(),
            ManifestBase::Archive => format!("{archive_name}/{path}"),
        };
        ensure!(
            entries.insert(path.clone()),
            "duplicate checksum path in {name}: {path}"
        );
        ensure!(
            Path::new(&path).starts_with(&archive_name),
            "checksum path leaves its archive: {path}"
        );
        if repo.publication.is_some() {
            repo.identity(&path, Some(expected))?;
        } else {
            ensure!(
                digest(&repo.file(&path)?)? == expected,
                "checksum mismatch: {path}"
            );
        }
    }
    Ok(entries)
}

fn inventory(directory: &Path, root: &Path) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "archive contains a symlink: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            paths.extend(inventory(&entry.path(), root)?);
        } else {
            ensure!(kind.is_file(), "archive contains a non-regular artifact");
            paths.insert(
                entry
                    .path()
                    .strip_prefix(root)?
                    .to_str()
                    .context("artifact path is not UTF-8")?
                    .to_owned(),
            );
        }
    }
    Ok(paths)
}

fn check_claim(repo: &Repository, claim: &Claim) -> Result<()> {
    nonempty(&claim.statement, "statement")?;
    nonempty(&claim.scope, "scope")?;
    nonempty(&claim.producer, "producer")?;
    limitations(&claim.limitations, "claim")?;
    if claim.level == Level::Unmeasured {
        ensure!(
            matches!(claim.outcome, Outcome::NotRun | Outcome::Blocked),
            "unmeasured claim cannot have a measured outcome"
        );
    } else {
        ensure!(
            !claim.evidence.is_empty(),
            "measured/source/failure claim needs evidence"
        );
    }
    if claim.level == Level::Failed {
        ensure!(
            matches!(claim.outcome, Outcome::Failed | Outcome::Blocked),
            "failed claim cannot become favorable coverage"
        );
    }
    if claim.outcome == Outcome::Qualified {
        ensure!(
            matches!(
                claim.level,
                Level::Observed | Level::IndependentlyReproduced
            ),
            "source/build/operator evidence cannot qualify physical behavior"
        );
    }
    let mut paths = BTreeSet::new();
    for evidence in &claim.evidence {
        ensure!(
            paths.insert(&evidence.path),
            "duplicate evidence path in claim"
        );
        if claim.level != Level::Unmeasured {
            ensure!(
                evidence.sha256.is_some(),
                "non-unmeasured evidence needs SHA-256"
            );
        }
        if let Some(expected) = &evidence.sha256 {
            ensure!(hex(expected, 64), "invalid evidence SHA-256");
        }
        repo.identity(&evidence.path, evidence.sha256.as_deref())?;
        if evidence.role == Role::RawCapture
            && let Some(publication) = repo.publication
        {
            publication.require_public(&evidence.path)?;
        }
    }
    let required_role = match claim.level {
        Level::SourceSupported => Some(Role::Source),
        Level::BuildChecked => Some(Role::BuildLog),
        Level::Observed => Some(Role::RawCapture),
        Level::OperatorReported => Some(Role::OperatorRecord),
        _ => None,
    };
    if let Some(role) = required_role {
        ensure!(
            claim.evidence.iter().any(|evidence| evidence.role == role),
            "claim lacks its required evidence role"
        );
    }
    if claim.level == Level::IndependentlyReproduced {
        let reproducer = claim
            .reproduced_by
            .as_deref()
            .context("independent claim needs reproduced_by")?;
        nonempty(reproducer, "reproduced_by")?;
        ensure!(
            normalize_actor(reproducer) != normalize_actor(&claim.producer),
            "producer and reproducer must differ"
        );
        ensure!(
            claim
                .evidence
                .iter()
                .any(|evidence| evidence.role == Role::Reproduction)
                && claim
                    .evidence
                    .iter()
                    .any(|evidence| evidence.role == Role::Review),
            "independent claim needs distinct reproduction and review artifacts"
        );
    } else {
        ensure!(
            claim.reproduced_by.is_none(),
            "only independently_reproduced claims use reproduced_by"
        );
    }
    Ok(())
}

fn check_coverage(coverage: &Coverage, claims: &BTreeMap<&str, &Claim>) -> Result<()> {
    nonempty(&coverage.scope, "coverage scope")?;
    let mut ids = BTreeSet::new();
    let mut support = Vec::new();
    for id in &coverage.claims {
        ensure!(ids.insert(id), "duplicate supporting claim");
        support.push(
            *claims
                .get(id.as_str())
                .with_context(|| format!("unknown supporting claim: {id}"))?,
        );
    }
    if coverage.outcome != Outcome::NotRun
        || coverage.driver != Driver::Unverified
        || coverage.rust_api != RustApi::Unverified
        || coverage.pins == Pins::Known
        || coverage.fixture == FixtureState::Available
    {
        ensure!(
            !support.is_empty(),
            "coverage override needs supporting claims"
        );
    }
    if matches!(coverage.driver, Driver::Upstream | Driver::Community)
        || coverage.rust_api == RustApi::Available
    {
        ensure!(
            support.iter().any(|claim| matches!(
                claim.level,
                Level::SourceSupported
                    | Level::BuildChecked
                    | Level::Observed
                    | Level::IndependentlyReproduced
            ) && !matches!(
                claim.outcome,
                Outcome::Failed | Outcome::Blocked
            )),
            "driver/Rust API availability needs nonfailed source/build/observed/reproduced support"
        );
    }
    if coverage.pins == Pins::Known || coverage.fixture == FixtureState::Available {
        ensure!(
            support
                .iter()
                .any(|claim| !matches!(claim.level, Level::Failed | Level::Unmeasured)),
            "availability needs nonfailed known evidence"
        );
    }
    if coverage.outcome == Outcome::Qualified {
        ensure!(
            support
                .iter()
                .any(|claim| claim.outcome == Outcome::Qualified
                    && matches!(
                        claim.level,
                        Level::Observed | Level::IndependentlyReproduced
                    )),
            "qualified coverage needs a qualifying physical/reproduction claim"
        );
        ensure!(
            support
                .iter()
                .any(|claim| claim.outcome == Outcome::Qualified
                    && matches!(
                        claim.level,
                        Level::Observed | Level::IndependentlyReproduced
                    )
                    && claim
                        .evidence
                        .iter()
                        .any(|evidence| evidence.role == Role::RawCapture)),
            "qualified physical coverage needs raw_capture on its qualifying claim"
        );
        ensure!(
            matches!(coverage.driver, Driver::Upstream | Driver::Community)
                && matches!(coverage.pins, Pins::Known | Pins::NotRequired)
                && matches!(
                    coverage.fixture,
                    FixtureState::Available | FixtureState::NotRequired
                ),
            "qualified coverage has unestablished driver/pins/fixture"
        );
    }
    Ok(())
}

fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Partial => "partial",
        Outcome::Qualified => "qualified",
        Outcome::Failed => "failed",
        Outcome::NotRun => "not_run",
        Outcome::Blocked => "blocked",
    }
}

fn normalize_actor(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn check_markdown(repo: &Repository) -> Result<usize> {
    let mut count = 0;
    for name in &repo.tracked {
        if !name.ends_with(".md") {
            continue;
        }
        let path = repo.file(name)?;
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading Markdown: {name}"))?;
        let mut fence: Option<(u8, usize)> = None;
        for line in text.lines() {
            let trimmed = line.trim_start();
            let bytes = trimmed.as_bytes();
            let run = bytes.first().map_or(0, |first| {
                bytes.iter().take_while(|byte| *byte == first).count()
            });
            if let Some((marker, length)) = fence {
                if bytes.first() == Some(&marker) && run >= length {
                    fence = None;
                }
                continue;
            }
            if run >= 3 && matches!(bytes.first(), Some(b'`' | b'~')) {
                fence = Some((bytes[0], run));
                continue;
            }
            if line.starts_with("    ") || line.starts_with('\t') {
                continue;
            }
            let line = without_inline_code(line);
            for target in markdown_targets(&line) {
                if local_target(repo, name, &target)
                    .with_context(|| format!("local link in {name}: {target}"))?
                {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

fn without_inline_code(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            output.push(bytes[i]);
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        let length = i - start;
        let mut end = i;
        let mut closing = None;
        while end < bytes.len() {
            if bytes[end] != b'`' {
                end += 1;
                continue;
            }
            let mark = end;
            while end < bytes.len() && bytes[end] == b'`' {
                end += 1;
            }
            if end - mark == length {
                closing = Some(end);
                break;
            }
        }
        if let Some(end) = closing {
            output.resize(output.len() + end - start, b' ');
            i = end;
        } else {
            output.extend_from_slice(&bytes[start..]);
            break;
        }
    }
    // Only spans bounded by ASCII delimiters are replaced; UTF-8 outside is intact.
    String::from_utf8(output).expect("replacing complete UTF-8 code spans preserves UTF-8")
}

fn markdown_targets(line: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let trimmed = line.trim_start();
    if trimmed.starts_with('[')
        && let Some(end) = trimmed.find("]:")
    {
        targets.push(trimmed[end + 2..].trim().to_owned());
    }
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if &bytes[i..i + 2] != b"](" {
            i += 1;
            continue;
        }
        let Some(opening) = line[..i].rfind('[') else {
            i += 1;
            continue;
        };
        if bytes[..opening]
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count()
            % 2
            != 0
        {
            i += 1;
            continue;
        }
        let start = i + 2;
        let mut j = start;
        let mut depth = 1;
        let mut angle = false;
        while j < bytes.len() {
            match bytes[j] {
                b'\\' => {
                    j = (j + 2).min(bytes.len());
                    continue;
                }
                b'<' => angle = true,
                b'>' => angle = false,
                b'(' if !angle => depth += 1,
                b')' if !angle => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        if depth == 0 {
            targets.push(line[start..j].trim().to_owned());
            i = j + 1;
        } else {
            break;
        }
    }
    targets
}

fn local_target(repo: &Repository, source: &str, target: &str) -> Result<bool> {
    let target = if let Some(rest) = target.strip_prefix('<') {
        rest.split_once('>').map_or(rest, |(path, _)| path)
    } else {
        target.split_ascii_whitespace().next().unwrap_or("")
    };
    let target = target.split(['#', '?']).next().unwrap_or("");
    if target.is_empty() {
        return Ok(false);
    }
    if let Some((scheme, _)) = target.split_once(':')
        && !scheme.is_empty()
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte))
    {
        return Ok(false);
    }
    let mut decoded = Vec::new();
    let mut bytes = target.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let a = bytes.next().context("incomplete percent escape")?;
            let b = bytes.next().context("incomplete percent escape")?;
            let digits = [a, b];
            decoded.push(
                u8::from_str_radix(std::str::from_utf8(&digits)?, 16)
                    .context("invalid percent escape")?,
            );
        } else {
            decoded.push(byte);
        }
    }
    let decoded = String::from_utf8(decoded)
        .context("local target is not UTF-8")?
        .replace("\\(", "(")
        .replace("\\)", ")");
    // Unmodified mdBook sources link to rendered HTML. Verify the corresponding
    // checked-in chapter, only inside these two pinned upstream source trees.
    let style_book = source.starts_with("docs/standards/rust-style/src/doc/style-guide/src/");
    let api_book = source.starts_with("docs/standards/rust-api/src/");
    let decoded = if (style_book || api_book)
        && !decoded.contains('/')
        && let Some(chapter) = decoded.strip_suffix(".html")
    {
        format!(
            "{}.md",
            if style_book && chapter == "index" {
                "README"
            } else {
                chapter
            }
        )
    } else {
        decoded
    };
    ensure!(
        !Path::new(&decoded).is_absolute(),
        "local Markdown target must not be host-absolute"
    );
    let mut relative = Path::new(source)
        .parent()
        .context("Markdown source has no parent")?
        .to_path_buf();
    for part in Path::new(&decoded).components() {
        match part {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                ensure!(relative.pop(), "Markdown link escapes repository");
            }
            _ => bail!("unsupported local Markdown path"),
        }
    }
    let name = relative.to_str().context("local target is not UTF-8")?;
    if name.is_empty() {
        ensure!(
            !repo.tracked.is_empty(),
            "root directory link has no tracked target"
        );
        return Ok(true);
    }
    if repo
        .publication
        .is_some_and(|publication| publication.is_omitted(&publication.recorded(name)))
    {
        repo.omitted_links.set(
            repo.omitted_links
                .get()
                .checked_add(1)
                .context("omitted link count overflow")?,
        );
        return Ok(true);
    }
    let resolved = repo
        .publication
        .map_or(name, |publication| publication.resolved(name));
    if resolved != name {
        repo.relocated_links.set(
            repo.relocated_links
                .get()
                .checked_add(1)
                .context("relocated link count overflow")?,
        );
    }
    let path = repo.path(name)?;
    if path.is_file() {
        repo.file(name)?;
    } else {
        ensure!(
            path.is_dir()
                && repo
                    .tracked
                    .iter()
                    .any(|entry| entry.starts_with(&format!("{resolved}/"))),
            "local directory link has no tracked target"
        );
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    #[derive(Debug)]
    struct Fixture {
        root: PathBuf,
        ledger: Value,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "c6-assurance-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let mut fixture = Self {
                root,
                ledger: json!({}),
            };
            fixture.command(&["init", "--quiet"]);
            fixture.write("README.md", "[Platform](docs/platform.md)\n");
            fixture.write("docs/platform.md", "# Platform\nScope remains limited.\n");
            fixture.write(
                "source.rs",
                "// Fixture input, not an authored firmware implementation.\n",
            );
            fixture.write(
                "results/control/report.md",
                "[Repository](../../README.md)\n",
            );
            fixture.write("results/control/capture.raw", "retained fixture bytes\r\n");
            fixture.seal();
            fixture.ledger = json!({
                "schema_version":1,
                "archives":[{"id":"control","path":"results/control","state":"finalized","manifest":"results/control/SHA256SUMS","manifest_base":"repository","inputs":null,"limitations":["A test artifact, not physical evidence."]}],
                "claims":[{"id":"source","statement":"A source record exists.","level":"source_supported","outcome":"partial","scope":"Fixture-only source statement.","producer":"original","reproduced_by":null,"evidence":[{"path":"source.rs","role":"source","sha256":digest(&fixture.root.join("source.rs")).unwrap()}],"limitations":["No physical behavior is established."]}],
                "platforms":[{"id":"test","label":"Test route","reference":"docs/platform.md"}],
                "subsystems":[{"id":"cpu","label":"CPU"}],
                "coverage_defaults":{"driver":"unverified","rust_api":"unverified","pins":"unverified","fixture":"unverified","outcome":"not_run"},
                "coverage":[{"platform":"test","subsystem":"cpu","driver":"upstream","rust_api":"available","pins":"not_required","fixture":"available","outcome":"partial","scope":"Source-only partial fixture record.","claims":["source"]}]
            });
            fixture.save();
            fixture
        }

        fn command(&self, args: &[&str]) -> Vec<u8> {
            git(&self.root, args).unwrap()
        }

        fn write(&self, path: &str, contents: &str) {
            let path = self.root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }

        fn seal(&self) {
            let directory = self.root.join("results/control");
            let rows = inventory(&directory, &self.root)
                .unwrap()
                .into_iter()
                .filter(|name| name != "results/control/SHA256SUMS")
                .map(|name| format!("{}  {name}\n", digest(&self.root.join(&name)).unwrap()))
                .collect::<String>();
            self.write("results/control/SHA256SUMS", &rows);
        }

        fn save(&self) {
            self.write(
                "quality/evidence.json",
                &serde_json::to_string_pretty(&self.ledger).unwrap(),
            );
            self.command(&["add", "--all"]);
        }

        fn verify(&self) -> Result<Value> {
            verify_repository(&self.root, Path::new("quality/evidence.json"))
        }

        fn rejected(&self, reason: &str) {
            let error = self.verify().unwrap_err();
            assert!(
                format!("{error:#}").contains(reason),
                "unexpected rejection: {error:#}"
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only this test's exclusively created temporary repository is removed.
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn valid_staged_ledger_checks_complete_inventory_and_scoped_coverage() {
        let fixture = Fixture::new();
        let result = fixture.verify().unwrap();
        assert_eq!(result["finalized_archives"], 1);
        assert_eq!(result["verified_manifest_entries"], 2);
        assert_eq!(result["coverage_pairs"], 1);
        assert_eq!(result["explicit_outcomes"]["partial"], 1);
        assert_eq!(result["checked_local_markdown_links"], 2);
    }

    #[test]
    fn historical_archive_relative_manifests_use_only_the_explicit_registered_base() {
        let mut fixture = Fixture::new();
        let path = fixture.root.join("results/control/SHA256SUMS");
        let rows = fs::read_to_string(&path)
            .unwrap()
            .replace("  results/control/", "  ");
        fs::write(&path, rows).unwrap();
        fixture.ledger["archives"][0]["manifest_base"] = json!("archive");
        fixture.save();
        fixture.verify().unwrap();
        fixture.ledger["archives"][0]["manifest_base"] = json!("repository");
        fixture.save();
        assert!(
            fixture.verify().is_err(),
            "wrong base must not be guessed from existing files"
        );
    }

    #[test]
    fn archive_mutations_are_rejected_instead_of_being_resealed_implicitly() {
        for mutation in [
            "edited",
            "deleted",
            "unlisted",
            "manifest",
            "duplicate",
            "escape",
            "unicode",
        ] {
            let fixture = Fixture::new();
            match mutation {
                "edited" => fixture.write("results/control/capture.raw", "silently edited data"),
                "deleted" => {
                    fs::remove_file(fixture.root.join("results/control/capture.raw")).unwrap()
                }
                "unlisted" => fixture.write("results/control/extra.json", "{}"),
                "manifest" => {
                    fs::remove_file(fixture.root.join("results/control/SHA256SUMS")).unwrap()
                }
                "duplicate" => {
                    let manifest =
                        fs::read_to_string(fixture.root.join("results/control/SHA256SUMS"))
                            .unwrap();
                    fixture.write(
                        "results/control/SHA256SUMS",
                        &(manifest.clone() + &manifest),
                    );
                }
                "escape" => fixture.write(
                    "results/control/SHA256SUMS",
                    &format!("{}  ../outside\n", "0".repeat(64)),
                ),
                "unicode" => fixture.write(
                    "results/control/SHA256SUMS",
                    &format!("{}  results/control/capture.raw\n", "é".repeat(40)),
                ),
                _ => unreachable!(),
            }
            assert!(
                fixture.verify().is_err(),
                "mutation was admitted: {mutation}"
            );
        }
    }

    #[test]
    fn unregistered_collections_and_missing_support_are_visible() {
        let fixture = Fixture::new();
        fixture.write("results/omitted/report.md", "not registered");
        fixture.rejected("registered archives differ");
        let mut fixture = Fixture::new();
        fixture.ledger["claims"][0]["evidence"][0]["sha256"] = Value::Null;
        fixture.save();
        fixture.rejected("needs SHA-256");
        let fixture = Fixture::new();
        fixture.write("source.rs", "changed support bytes");
        fixture.rejected("evidence hash mismatch");
    }

    #[test]
    fn operator_and_source_evidence_cannot_be_promoted_by_only_changing_labels() {
        let mut fixture = Fixture::new();
        fixture.ledger["claims"][0]["level"] = json!("operator_reported");
        fixture.ledger["claims"][0]["evidence"][0]["role"] = json!("operator_record");
        fixture.ledger["coverage"][0]["driver"] = json!("unverified");
        fixture.ledger["coverage"][0]["rust_api"] = json!("unverified");
        fixture.save();
        fixture.verify().unwrap();
        fixture.ledger["claims"][0]["level"] = json!("independently_reproduced");
        fixture.save();
        fixture.rejected("needs reproduced_by");

        let mut fixture = Fixture::new();
        fixture.ledger["claims"][0]["outcome"] = json!("qualified");
        fixture.save();
        fixture.rejected("cannot qualify physical behavior");
    }

    #[test]
    fn operator_inventory_does_not_establish_driver_or_rust_api_availability() {
        let mut fixture = Fixture::new();
        fixture.ledger["claims"][0]["level"] = json!("operator_reported");
        fixture.ledger["claims"][0]["evidence"][0]["role"] = json!("operator_record");
        fixture.save();
        fixture.rejected("driver/Rust API availability needs");
        fixture.ledger["coverage"][0]["driver"] = json!("unverified");
        fixture.ledger["coverage"][0]["rust_api"] = json!("unverified");
        fixture.ledger["coverage"][0]["pins"] = json!("known");
        fixture.save();
        fixture.verify().unwrap();
    }

    #[test]
    fn reproduction_requires_different_recorded_actors_and_distinct_hashed_roles() {
        let mut fixture = Fixture::new();
        fixture.ledger["claims"][0]["level"] = json!("independently_reproduced");
        fixture.ledger["claims"][0]["reproduced_by"] = json!("reviewer");
        fixture.ledger["claims"][0]["evidence"] = json!([
            {"path":"source.rs","role":"reproduction","sha256":digest(&fixture.root.join("source.rs")).unwrap()},
            {"path":"README.md","role":"review","sha256":digest(&fixture.root.join("README.md")).unwrap()}
        ]);
        fixture.save();
        fixture.verify().unwrap();
        fixture.ledger["claims"][0]["producer"] = json!("same author");
        fixture.ledger["claims"][0]["reproduced_by"] = json!("  SAME   AUTHOR ");
        fixture.save();
        fixture.rejected("producer and reproducer must differ");
        fixture.ledger["claims"][0]["producer"] = json!("original");
        fixture.ledger["claims"][0]["reproduced_by"] = json!("reviewer");
        fixture.ledger["claims"][0]["evidence"][1]["path"] = json!("source.rs");
        fixture.save();
        fixture.rejected("duplicate evidence path");
    }

    #[test]
    fn defaults_and_source_only_overrides_cannot_qualify_all_hardware() {
        let mut fixture = Fixture::new();
        fixture.ledger["coverage_defaults"]["outcome"] = json!("qualified");
        fixture.save();
        fixture.rejected("defaults must remain unknown/not_run");
        let mut fixture = Fixture::new();
        fixture.ledger["coverage"][0]["outcome"] = json!("qualified");
        fixture.save();
        fixture.rejected("qualified coverage needs a qualifying");
        fixture.ledger["coverage"][0]["claims"] = json!(["imagined"]);
        fixture.save();
        fixture.rejected("unknown supporting claim");
    }

    #[test]
    fn offline_reproduction_alone_does_not_fully_qualify_a_physical_domain() {
        let mut fixture = Fixture::new();
        fixture.ledger["claims"][0]["level"] = json!("independently_reproduced");
        fixture.ledger["claims"][0]["outcome"] = json!("qualified");
        fixture.ledger["claims"][0]["reproduced_by"] = json!("reviewer");
        fixture.ledger["claims"][0]["evidence"] = json!([
            {"path":"source.rs","role":"reproduction","sha256":digest(&fixture.root.join("source.rs")).unwrap()},
            {"path":"README.md","role":"review","sha256":digest(&fixture.root.join("README.md")).unwrap()}
        ]);
        fixture.ledger["coverage"][0]["outcome"] = json!("qualified");
        fixture.save();
        fixture.rejected("qualified physical coverage needs raw_capture");
        fixture.ledger["claims"][0]["evidence"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "path":"results/control/capture.raw","role":"raw_capture",
                "sha256":digest(&fixture.root.join("results/control/capture.raw")).unwrap()
            }));
        fixture.save();
        fixture.verify().unwrap();
    }

    #[test]
    fn changing_frozen_inputs_and_resealing_the_archive_does_not_hide_drift() {
        let mut fixture = Fixture::new();
        let raw = "results/control/capture.raw";
        fixture.write(
            "results/control/INPUTS.sha256",
            &format!("{}  {raw}\n", digest(&fixture.root.join(raw)).unwrap()),
        );
        fixture.seal();
        fixture.save();
        fixture.command(&[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            "fixture freeze",
        ]);
        let commit = String::from_utf8(fixture.command(&["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        fixture.ledger["archives"][0]["inputs"] =
            json!({"path":"results/control/INPUTS.sha256","freeze_commit":commit});
        fixture.save();
        fixture.verify().unwrap();
        fixture.write(raw, "new bytes hidden behind an updated manifest");
        fixture.write(
            "results/control/INPUTS.sha256",
            &format!("{}  {raw}\n", digest(&fixture.root.join(raw)).unwrap()),
        );
        fixture.seal();
        fixture.save();
        fixture.rejected("INPUTS manifest changed since freeze");
    }

    #[test]
    fn collecting_inputs_are_checked_without_pretending_they_are_frozen_or_qualified() {
        let mut fixture = Fixture::new();
        let raw = "results/control/capture.raw";
        fixture.write(
            "results/control/INPUTS.sha256",
            &format!("{}  {raw}\n", digest(&fixture.root.join(raw)).unwrap()),
        );
        fixture.ledger["archives"][0]["state"] = json!("collecting");
        fixture.ledger["archives"][0]["manifest"] = Value::Null;
        fixture.ledger["archives"][0]["inputs"] =
            json!({"path":"results/control/INPUTS.sha256","freeze_commit":null});
        fixture.save();
        let report = fixture.verify().unwrap();
        assert_eq!(report["finalized_archives"], 0);
        assert_eq!(report["warnings"].as_array().unwrap().len(), 2);
        fixture.ledger["claims"][0]["level"] = json!("observed");
        fixture.ledger["claims"][0]["outcome"] = json!("qualified");
        fixture.ledger["claims"][0]["evidence"] = json!([{"path":raw,"role":"raw_capture","sha256":digest(&fixture.root.join(raw)).unwrap()}]);
        fixture.save();
        fixture.rejected("qualified claim depends on an unfinished");
        fixture.ledger["archives"][0]["state"] = json!("finalized");
        fixture.ledger["archives"][0]["manifest"] = json!("results/control/SHA256SUMS");
        fixture.seal();
        fixture.save();
        fixture.rejected("only collecting inputs may lack a freeze commit");
    }

    #[test]
    fn markdown_checks_real_targets_but_ignores_literal_code_and_external_links() {
        let fixture = Fixture::new();
        fixture.write("README.md", "```text\n[example](not-a-file)\n```\n`[literal](also-not-a-file)`\n\\[escaped](not-a-file)\n[external](https://example.invalid/missing)\n[real](docs/platform.md#heading)\n[space](<docs/a%20b.md>)\n");
        fixture.write("docs/a b.md", "# Space\n");
        fixture.save();
        fixture.verify().unwrap();
        fixture.write("README.md", "[broken](docs/missing.md)\n");
        fixture.rejected("missing referenced path");
        fixture.write("README.md", "[escape](../outside.md)\n");
        fixture.rejected("Markdown link escapes repository");
    }

    #[test]
    fn upstream_book_links_require_real_chapters_without_exempting_authored_docs() {
        let fixture = Fixture::new();
        fixture.write(
            "docs/standards/rust-api/src/about.md",
            "[checklist](checklist.html#rules)\n",
        );
        fixture.write("docs/standards/rust-api/src/checklist.md", "# Rules\n");
        fixture.write(
            "docs/standards/rust-style/src/doc/style-guide/src/items.md",
            "[intro](index.html)\n",
        );
        fixture.write(
            "docs/standards/rust-style/src/doc/style-guide/src/README.md",
            "# Introduction\n",
        );
        fixture.save();
        fixture.verify().unwrap();
        fixture.write(
            "docs/standards/rust-api/src/about.md",
            "[missing](missing.html)\n",
        );
        fixture.rejected("missing referenced path");
        fixture.write(
            "docs/standards/rust-api/src/about.md",
            "[checklist](checklist.html)\n",
        );
        fixture.write("docs/authored.md", "[chapter](platform.html)\n");
        fixture.save();
        fixture.rejected("missing referenced path");
    }

    #[test]
    fn project_ledger_discovers_only_its_own_archive_root() {
        let fixture = Fixture::new();
        fixture.save();
        let old = "results/control";
        let new = "projects/fixture/results/control";
        fs::create_dir_all(fixture.root.join("projects/fixture/results")).unwrap();
        fs::rename(fixture.root.join(old), fixture.root.join(new)).unwrap();
        let seal = fs::read_to_string(fixture.root.join(new).join("SHA256SUMS")).unwrap();
        fixture.write(&format!("{new}/SHA256SUMS"), &seal.replace(old, new));
        let ledger = serde_json::to_string(&fixture.ledger)
            .unwrap()
            .replace(old, new);
        fixture.write("projects/fixture/quality/evidence.json", &ledger);
        fixture.write("projects/fixture/README.md", "# Fixture project\n");
        fixture.write(
            "results/unrelated/report.md",
            "# Unrelated historical archive\n",
        );
        git(&fixture.root, &["add", "--all"]).unwrap();
        verify_repository(
            &fixture.root,
            Path::new("projects/fixture/quality/evidence.json"),
        )
        .unwrap();
        fixture.write(
            "projects/fixture/results/unregistered/report.md",
            "# Missing registration\n",
        );
        git(&fixture.root, &["add", "--all"]).unwrap();
        assert!(
            verify_repository(
                &fixture.root,
                Path::new("projects/fixture/quality/evidence.json")
            )
            .unwrap_err()
            .to_string()
            .contains("registered archives differ")
        );
        fixture.write(
            "projects/fixture/quality/evidence.json",
            &ledger.replace(new, "projects/other/results/control"),
        );
        git(&fixture.root, &["add", "--all"]).unwrap();
        assert!(
            verify_repository(
                &fixture.root,
                Path::new("projects/fixture/quality/evidence.json")
            )
            .unwrap_err()
            .to_string()
            .contains("results root")
        );
    }

    #[test]
    fn full_verification_does_not_resolve_moved_platform_sources() {
        let fixture = Fixture::new();
        fixture.save();
        fixture.verify().unwrap();
        fs::create_dir_all(fixture.root.join("projects/fixture/docs")).unwrap();
        fs::rename(
            fixture.root.join("docs/platform.md"),
            fixture.root.join("projects/fixture/docs/platform.md"),
        )
        .unwrap();
        fixture.write("quality/publication.json", "{\"relocations\":[{\"from\":\"docs/platform.md\",\"to\":\"projects/fixture/docs/platform.md\"}]}\n");
        git(&fixture.root, &["add", "--all"]).unwrap();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("missing referenced path")
        );
    }

    #[cfg(unix)]
    #[test]
    fn even_an_empty_project_archive_root_must_not_be_a_symlink() {
        let fixture = Fixture::new();
        fixture.save();
        let mut ledger = fixture.ledger.clone();
        ledger["archives"] = json!([]);
        ledger["claims"] = json!([]);
        fixture.write(
            "projects/fixture/quality/evidence.json",
            &serde_json::to_string(&ledger).unwrap(),
        );
        fs::create_dir_all(fixture.root.join(".cache/empty")).unwrap();
        std::os::unix::fs::symlink(
            fixture.root.join(".cache/empty"),
            fixture.root.join("projects/fixture/results"),
        )
        .unwrap();
        git(&fixture.root, &["add", "--all"]).unwrap();
        assert!(
            verify_repository(
                &fixture.root,
                Path::new("projects/fixture/quality/evidence.json")
            )
            .unwrap_err()
            .to_string()
            .contains("symlink")
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_tracked_symlink_cannot_smuggle_external_evidence_into_the_repository() {
        let mut fixture = Fixture::new();
        let outside = Fixture::new();
        std::os::unix::fs::symlink(
            outside.root.join("source.rs"),
            fixture.root.join("external.rs"),
        )
        .unwrap();
        fixture.ledger["claims"][0]["evidence"][0]["path"] = json!("external.rs");
        fixture.save();
        fixture.rejected("path escapes repository");
    }

    #[cfg(unix)]
    #[test]
    fn a_tracked_symlink_cannot_promote_an_ignored_internal_file_into_evidence() {
        let mut fixture = Fixture::new();
        fixture.write(".gitignore", ".cache/\n");
        fixture.write(
            ".cache/support.rs",
            &fs::read_to_string(fixture.root.join("source.rs")).unwrap(),
        );
        std::os::unix::fs::symlink(".cache/support.rs", fixture.root.join("internal.rs")).unwrap();
        fixture.ledger["claims"][0]["evidence"][0]["path"] = json!("internal.rs");
        fixture.save();
        fixture.rejected("referenced path contains a symlink");
        fixture.ledger["claims"][0]["evidence"][0]["path"] = json!("source.rs");
        fixture.write("README.md", "[not tracked evidence](internal.rs)\n");
        fixture.save();
        fixture.rejected("referenced path contains a symlink");
    }

    #[cfg(unix)]
    #[test]
    fn a_cached_child_path_cannot_hide_an_ignored_symlink_ancestor() {
        let mut fixture = Fixture::new();
        fixture.write(".gitignore", ".cache/\n");
        fixture.write(
            ".cache/alternate/platform.md",
            &fs::read_to_string(fixture.root.join("docs/platform.md")).unwrap(),
        );
        fixture.ledger["claims"][0]["evidence"][0]["path"] = json!("docs/platform.md");
        fixture.ledger["claims"][0]["evidence"][0]["sha256"] =
            json!(digest(&fixture.root.join("docs/platform.md")).unwrap());
        fixture.save();
        // Preserve the indexed child path while only the fixture working tree
        // replaces its parent with a link to untracked, ignored content.
        fs::remove_dir_all(fixture.root.join("docs")).unwrap();
        std::os::unix::fs::symlink(".cache/alternate", fixture.root.join("docs")).unwrap();
        fixture.rejected("referenced path contains a symlink");
    }
}
