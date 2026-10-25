//! Offline guide verification and explicit upstream updates. No source code is executed.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DIRECTORY: &str = "docs/standards";
const LOCK: &str = "sources.json";

#[derive(Debug, Parser)]
#[command(about = "Check local style sources offline or explicitly update their pins")]
struct Cli {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[command(subcommand)]
    command: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Verify exact file inventories, hashes, and the API review inventory; no network.
    Check,
    /// Fetch upstream HEAD (the default branch), or a supplied branch/tag/commit.
    /// Refuses changed local copies. Leaves changes uncommitted for review.
    Update {
        /// Update just rust-style, rust-api, or google-rust; otherwise update all.
        #[arg(long)]
        source: Option<String>,
        #[arg(long, default_value = "HEAD")]
        rev: String,
    },
}

#[derive(Debug)]
struct Spec {
    id: &'static str,
    repository: &'static str,
    paths: &'static [&'static str],
    required: &'static [&'static str],
}

const SPECS: &[Spec] = &[
    Spec {
        id: "rust-style",
        repository: "https://github.com/rust-lang/rust.git",
        paths: &[
            "src/doc/style-guide",
            "LICENSE-APACHE",
            "LICENSE-MIT",
            "COPYRIGHT",
        ],
        required: &[
            "src/doc/style-guide/src/README.md",
            "LICENSE-APACHE",
            "LICENSE-MIT",
            "COPYRIGHT",
        ],
    },
    Spec {
        id: "rust-api",
        repository: "https://github.com/rust-lang/api-guidelines.git",
        paths: &["src", "book.toml", "LICENSE-APACHE", "LICENSE-MIT"],
        required: &[
            "src/checklist.md",
            "src/SUMMARY.md",
            "LICENSE-APACHE",
            "LICENSE-MIT",
        ],
    },
    Spec {
        id: "google-rust",
        repository: "https://github.com/google/styleguide.git",
        paths: &["rustguide.md", "LICENSE"],
        required: &["rustguide.md", "LICENSE"],
    },
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceLock {
    schema_version: u32,
    sources: Vec<Source>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Source {
    id: String,
    repository: String,
    requested_ref: String,
    commit: String,
    files: BTreeMap<String, String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Action::Check => {
            check_sources(&cli.root.join(DIRECTORY))?;
            check_inventory(&cli.root)?;
            println!("Guide inventories, hashes and API checklist agree (offline).");
        }
        Action::Update { source, rev } => update(&cli.root, source.as_deref(), &rev)?,
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn safe_path(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && !value.contains(['\\', '\0', '\r', '\n']),
        "invalid source path"
    );
    ensure!(
        Path::new(value)
            .components()
            .all(|p| matches!(p, Component::Normal(_))),
        "source path must be relative without traversal"
    );
    ensure!(
        value
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != ".."),
        "source path is not normalized"
    );
    Ok(())
}

// Reject symlinks in ancestors too, so a local copy cannot redirect reads or writes.
fn no_links(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        if let Ok(metadata) = fs::symlink_metadata(ancestor) {
            ensure!(
                !metadata.file_type().is_symlink(),
                "symlink in guide/cache path"
            );
        }
    }
    Ok(())
}

fn snapshot(directory: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    no_links(directory)?;
    let mut files = BTreeMap::new();
    walk(directory, directory, &mut files)?;
    Ok(files)
}

fn walk(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            walk(root, &path, files)?;
        } else {
            ensure!(kind.is_file(), "guide tree contains a nonregular file");
            let name = path
                .strip_prefix(root)?
                .to_str()
                .context("non-UTF-8 source path")?
                .replace('\\', "/");
            safe_path(&name)?;
            files.insert(name, fs::read(path)?);
        }
    }
    Ok(())
}

fn check_sources(directory: &Path) -> Result<SourceLock> {
    no_links(&directory.join(LOCK))?;
    let lock: SourceLock = serde_json::from_slice(&fs::read(directory.join(LOCK))?)?;
    ensure!(
        lock.schema_version == 1 && lock.sources.len() == SPECS.len(),
        "unsupported/incomplete source lock"
    );
    let mut ids = BTreeSet::new();
    for source in &lock.sources {
        let spec = SPECS
            .iter()
            .find(|s| s.id == source.id)
            .context("unknown guide")?;
        ensure!(ids.insert(&source.id), "duplicate guide");
        ensure!(
            source.repository == spec.repository && hex(&source.commit, 40),
            "invalid guide provenance"
        );
        valid_revision(&source.requested_ref)?;
        for path in source.files.keys() {
            safe_path(path)?;
            ensure!(
                selected(spec, path),
                "file outside declared upstream selection"
            );
        }
        for required in spec.required {
            ensure!(
                source.files.contains_key(*required),
                "required guide/license missing: {required}"
            );
        }
        let actual = snapshot(&directory.join(&source.id))?;
        ensure!(
            actual.len() == source.files.len(),
            "guide inventory changed: {}",
            source.id
        );
        for (name, expected) in &source.files {
            let bytes = actual.get(name).context("missing guide file")?;
            ensure!(
                hex(expected, 64) && digest(bytes) == *expected,
                "guide content changed: {}/{name}",
                source.id
            );
        }
    }
    Ok(lock)
}

fn selected(spec: &Spec, path: &str) -> bool {
    spec.paths.iter().any(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|s| s.starts_with('/'))
    })
}

fn valid_revision(rev: &str) -> Result<()> {
    ensure!(
        !rev.is_empty()
            && !rev.starts_with('-')
            && rev
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._/-".contains(&b))
            && !rev.contains(".."),
        "use HEAD, a branch, a tag, or a full commit hash"
    );
    Ok(())
}

// Arguments are passed directly, never interpolated into a shell. Git diagnostics
// are not echoed: local Git configuration can contain personal filesystem paths.
fn git(directory: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    // A caller's Git environment must not redirect cache operations into its
    // working repository or an alternate object/index directory.
    for name in [
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        command.env_remove(name);
    }
    let output = command
        .args(["-c", "core.fsmonitor=false"])
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .current_dir(directory)
        .args(args)
        .output()
        .context("starting Git")?;
    ensure!(
        output.status.success(),
        "source Git operation failed ({})",
        args.first().unwrap_or(&"unknown")
    );
    Ok(output.stdout)
}

fn regular_tree(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() || metadata.is_dir(),
        "nonregular Git cache entry"
    );
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            regular_tree(&entry?.path())?;
        }
    }
    Ok(())
}

fn check_git_cache(cache: &Path) -> Result<()> {
    let metadata = cache.join(".git");
    no_links(&metadata)?;
    ensure!(
        fs::symlink_metadata(&metadata)?.is_dir(),
        "Git cache metadata must be a directory, not a gitdir redirect"
    );
    regular_tree(&metadata)?;
    ensure!(
        !metadata.join("commondir").exists() && !metadata.join("objects/info/alternates").exists(),
        "redirected Git cache storage is unsupported"
    );
    Ok(())
}

fn fetch_source(
    root: &Path,
    spec: &Spec,
    rev: &str,
) -> Result<(Source, BTreeMap<String, Vec<u8>>)> {
    let cache = root.join(".cache/standard-sources").join(spec.id);
    no_links(&cache)?;
    fs::create_dir_all(&cache)?;
    if !cache.join(".git").exists() {
        // Detect a dangling metadata symlink before clone can follow it.
        no_links(&cache.join(".git"))?;
        git(
            &cache,
            &[
                "clone",
                "--quiet",
                "--filter=blob:none",
                "--no-checkout",
                "--depth=1",
                spec.repository,
                ".",
            ],
        )?;
    }
    check_git_cache(&cache)?;
    let origin = String::from_utf8(git(&cache, &["remote", "get-url", "origin"])?)?;
    ensure!(
        origin.trim() == spec.repository,
        "source cache has an unexpected origin"
    );
    git(
        &cache,
        &[
            "fetch",
            "--quiet",
            "--depth=1",
            "--filter=blob:none",
            "--no-tags",
            "--no-recurse-submodules",
            "origin",
            rev,
        ],
    )?;
    let commit = String::from_utf8(git(
        &cache,
        &["rev-parse", "--verify", "FETCH_HEAD^{commit}"],
    )?)?
    .trim()
    .to_owned();
    ensure!(hex(&commit, 40), "unexpected upstream object format");
    let mut args = vec!["ls-tree", "-r", "-z", &commit, "--"];
    args.extend_from_slice(spec.paths);
    let tree = git(&cache, &args)?;
    let files = read_tree(&cache, spec, &tree)?;
    let source = Source {
        id: spec.id.to_owned(),
        repository: spec.repository.to_owned(),
        requested_ref: rev.to_owned(),
        commit,
        files: files
            .iter()
            .map(|(path, bytes)| (path.clone(), digest(bytes)))
            .collect(),
    };
    for required in spec.required {
        ensure!(
            source.files.contains_key(*required),
            "upstream removed required guide/license: {required}"
        );
    }
    Ok((source, files))
}

fn read_tree(cache: &Path, spec: &Spec, tree: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    let mut total = 0_usize;
    for record in tree.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let (metadata, path) = std::str::from_utf8(record)?
            .split_once('\t')
            .context("invalid Git tree")?;
        let fields: Vec<_> = metadata.split_whitespace().collect();
        ensure!(
            fields.len() == 3
                && matches!(fields[0], "100644" | "100755")
                && fields[1] == "blob"
                && hex(fields[2], 40),
            "nonregular upstream guide entry"
        );
        safe_path(path)?;
        ensure!(selected(spec, path), "unexpected upstream guide path");
        let size: usize = String::from_utf8(git(cache, &["cat-file", "-s", fields[2]])?)?
            .trim()
            .parse()?;
        ensure!(
            size <= 16 * 1024 * 1024 && total + size <= 64 * 1024 * 1024 && files.len() < 10_000,
            "guide selection exceeds size limit"
        );
        total += size;
        let bytes = git(cache, &["cat-file", "blob", fields[2]])?;
        ensure!(bytes.len() == size, "Git blob size changed");
        ensure!(
            files.insert(path.to_owned(), bytes).is_none(),
            "duplicate source path"
        );
    }
    Ok(files)
}

fn write_files(directory: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    for (name, bytes) in files {
        safe_path(name)?;
        let path = directory.join(name);
        fs::create_dir_all(path.parent().context("source has no parent")?)?;
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn update(root: &Path, source: Option<&str>, rev: &str) -> Result<()> {
    valid_revision(rev)?;
    ensure!(
        source.is_none_or(|id| SPECS.iter().any(|s| s.id == id)),
        "unknown guide"
    );
    let cache = root.join(".cache/style-sources");
    no_links(&cache)?;
    fs::create_dir_all(&cache)?;
    let mutex = cache.join("update.lock");
    let lock = fs::OpenOptions::new().write(true).create_new(true).open(&mutex)
        .context("another update is active; after a crashed update inspect .cache/style-sources before removing update.lock")?;
    let outcome = update_locked(root, source, rev);
    drop(lock);
    fs::remove_file(mutex).context("removing update lock")?;
    outcome
}

fn update_locked(root: &Path, source: Option<&str>, rev: &str) -> Result<()> {
    let directory = root.join(DIRECTORY);
    let mut lock = check_sources(&directory)?;
    let original = snapshot(&directory)?;
    // All downloads and checks happen before replacing any tracked directory.
    let mut replacements = Vec::new();
    for spec in SPECS.iter().filter(|s| source.is_none_or(|id| id == s.id)) {
        let (next, bytes) =
            fetch_source(root, spec, rev).with_context(|| format!("updating {}", spec.id))?;
        println!("{}: {}", spec.id, next.commit);
        replacements.push((next, bytes));
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let transaction = root.join(format!(
        ".cache/style-sources/update-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&transaction)?;
    let candidate = transaction.join("next");
    write_files(&candidate, &original)?;
    for (next, bytes) in replacements {
        fs::remove_dir_all(candidate.join(&next.id))?;
        write_files(&candidate.join(&next.id), &bytes)?;
        let current = lock
            .sources
            .iter_mut()
            .find(|s| s.id == next.id)
            .context("source absent from lock")?;
        *current = next;
    }
    let mut encoded = serde_json::to_vec_pretty(&lock)?;
    encoded.push(b'\n');
    fs::write(candidate.join(LOCK), encoded)?;
    check_sources(&candidate)?;
    ensure!(
        snapshot(&directory)? == original,
        "local sources changed during fetch; nothing installed"
    );
    let previous = transaction.join("previous");
    fs::rename(&directory, &previous)?;
    if let Err(error) = fs::rename(&candidate, &directory) {
        fs::rename(&previous, &directory).context(
            "install and rollback failed; previous copy retained in .cache/style-sources",
        )?;
        return Err(error).context("install failed; previous sources restored");
    }
    println!(
        "Updated local pins; review the diff and reconcile the coding standard/API review before committing. Previous copies remain in .cache/style-sources."
    );
    if check_inventory(root).is_err() {
        bail!(
            "sources installed; API inventory differs, reconcile docs/rust-api-checklist.md before check/commit"
        );
    }
    Ok(())
}

fn checklist_ids(text: &str, prefix: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for line in text
        .lines()
        .map(str::trim_start)
        .filter(|l| l.starts_with(prefix))
    {
        if line.starts_with("[C-")
            && line
                .split_once(']')
                .is_some_and(|(_, rest)| rest.trim_start().starts_with(':'))
        {
            continue;
        }
        let mut remaining = line;
        while let Some(start) = remaining.find("[C-") {
            remaining = &remaining[start + 1..];
            let Some(end) = remaining.find(']') else {
                break;
            };
            ids.insert(remaining[..end].to_owned());
            remaining = &remaining[end + 1..];
        }
    }
    ids
}

fn check_inventory(root: &Path) -> Result<()> {
    let upstream = fs::read_to_string(root.join(DIRECTORY).join("rust-api/src/checklist.md"))?;
    let inventory = fs::read_to_string(root.join("docs/rust-api-checklist.md"))?;
    // Checked boxes still describe active guidelines. Reference definitions
    // alone do not: upstream retains some obsolete C-* link definitions.
    let ids = checklist_ids(&upstream, "");
    ensure!(
        !ids.is_empty() && ids == checklist_ids(&inventory, "| [C-"),
        "API review inventory differs from adopted checklist"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temporary() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "style-sources-{}-{stamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        // macOS commonly exposes its temporary directory through /var -> /private/var.
        path.canonicalize().unwrap()
    }

    #[test]
    fn rejects_path_and_revision_injection() {
        for path in [
            "../escape",
            "/escape",
            "a/../b",
            "a//b",
            "a/./b",
            "a\\b",
            "a\nb",
            "",
        ] {
            assert!(safe_path(path).is_err(), "{path:?}");
        }
        for rev in [
            "--upload-pack=evil",
            "HEAD:evil",
            "HEAD~1",
            "a..b",
            "a\nb",
            "",
        ] {
            assert!(valid_revision(rev).is_err());
        }
        for rev in ["HEAD", "main", "master", "gh-pages", "refs/tags/v1.0"] {
            valid_revision(rev).unwrap();
        }
    }

    #[test]
    fn offline_check_rejects_edits_extra_missing_files_and_bad_provenance() {
        let directory = temporary();
        let mut lock = SourceLock {
            schema_version: 1,
            sources: Vec::new(),
        };
        for spec in SPECS {
            let files: BTreeMap<_, _> = spec
                .required
                .iter()
                .map(|s| (s.to_string(), b"fixture\n".to_vec()))
                .collect();
            write_files(&directory.join(spec.id), &files).unwrap();
            lock.sources.push(Source {
                id: spec.id.to_owned(),
                repository: spec.repository.to_owned(),
                requested_ref: "HEAD".to_owned(),
                commit: "a".repeat(40),
                files: files.iter().map(|(p, b)| (p.clone(), digest(b))).collect(),
            });
        }
        let save = |lock: &SourceLock| {
            fs::write(directory.join(LOCK), serde_json::to_vec(lock).unwrap()).unwrap()
        };
        save(&lock);
        check_sources(&directory).unwrap();
        let guide = directory.join("google-rust/rustguide.md");
        fs::write(&guide, "edit").unwrap();
        assert!(check_sources(&directory).is_err());
        fs::write(&guide, b"fixture\n").unwrap();
        let extra = directory.join("google-rust/extra");
        fs::write(&extra, "extra").unwrap();
        assert!(check_sources(&directory).is_err());
        fs::remove_file(extra).unwrap();
        fs::remove_file(&guide).unwrap();
        assert!(check_sources(&directory).is_err());
        fs::write(&guide, b"fixture\n").unwrap();
        lock.sources[0].repository = "https://example.invalid/changed".to_owned();
        save(&lock);
        assert!(check_sources(&directory).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reads_git_blob_bytes_and_rejects_symlink_entries() {
        let root = temporary();
        git(&root, &["init", "--quiet"]).unwrap();
        fs::write(root.join("rustguide.md"), b"exact\r\nbytes\n").unwrap();
        fs::write(root.join("LICENSE"), b"license\n").unwrap();
        git(&root, &["add", "."]).unwrap();
        let tree_id = String::from_utf8(git(&root, &["write-tree"]).unwrap()).unwrap();
        let tree = git(&root, &["ls-tree", "-r", "-z", tree_id.trim()]).unwrap();
        let files = read_tree(&root, &SPECS[2], &tree).unwrap();
        assert_eq!(files["rustguide.md"], b"exact\r\nbytes\n");
        assert!(
            read_tree(
                &root,
                &SPECS[2],
                format!("120000 blob {}\trustguide.md\0", "a".repeat(40)).as_bytes()
            )
            .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_local_symlinks_and_linked_ancestors() {
        let root = temporary();
        fs::create_dir(root.join("real")).unwrap();
        std::os::unix::fs::symlink(root.join("real"), root.join("link")).unwrap();
        assert!(snapshot(&root.join("link")).is_err());
        assert!(snapshot(&root).is_err());
        assert!(no_links(&root.join("link/new")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_redirected_git_metadata_before_fetch() {
        let root = temporary();
        let outside = temporary();
        git(&outside, &["init", "--quiet"]).unwrap();
        std::os::unix::fs::symlink(outside.join(".git"), root.join(".git")).unwrap();
        assert!(check_git_cache(&root).is_err());
        fs::remove_file(root.join(".git")).unwrap();
        fs::write(root.join(".git"), "gitdir: ../elsewhere\n").unwrap();
        assert!(check_git_cache(&root).is_err());
        fs::remove_file(root.join(".git")).unwrap();
        git(&root, &["init", "--quiet"]).unwrap();
        check_git_cache(&root).unwrap();
        fs::write(root.join(".git/commondir"), "../elsewhere\n").unwrap();
        assert!(check_git_cache(&root).is_err());
        fs::remove_file(root.join(".git/commondir")).unwrap();
        std::os::unix::fs::symlink(
            outside.join(".git/objects"),
            root.join(".git/objects/redirect"),
        )
        .unwrap();
        assert!(check_git_cache(&root).is_err());
        assert!(!outside.join(".git/FETCH_HEAD").exists());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn active_inventory_ignores_unused_reference_definitions() {
        let upstream = "  - [ ] Rule ([C-ONE])\n[C-OLD]: old.html\n  * [x] Rule ([C-TWO])\n";
        let authored = "| [C-ONE](one.md) | Applies |\n| [C-TWO](two.md) | Pending |\n";
        assert_eq!(
            checklist_ids(upstream, ""),
            checklist_ids(authored, "| [C-")
        );
        assert_ne!(
            checklist_ids(upstream, ""),
            checklist_ids("| [C-ONE](one.md) |", "| [C-")
        );
        assert_eq!(
            checklist_ids("[C-ONE] rule with [C-TWO]\n[C-OLD]: old.html\n", ""),
            checklist_ids(upstream, "")
        );
    }
}
