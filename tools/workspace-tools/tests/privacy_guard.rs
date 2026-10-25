//! End-to-end Git controls for the privacy hooks; no network or hardware I/O.

#![cfg(unix)]
#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Repository(PathBuf);

impl Repository {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "c6-privacy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let repo = Self(path);
        repo.ok(&["init", "--quiet"]);
        repo.ok(&["config", "user.name", "Fixture"]);
        repo.ok(&["config", "user.email", "fixture@example.invalid"]);
        repo.ok(&["config", "commit.gpgsign", "false"]);
        repo.write(
            ".git/privacy/terms.json",
            br#"{"terms":["fixture-private-identity"]}"#,
        );
        repo.write("quality/privacy-allowlist.json", b"[]\n");
        repo.write(
            "tools/workspace-tools/src/bin/privacy-guard.rs",
            include_bytes!("../src/bin/privacy-guard.rs"),
        );
        repo.ok(&["add", "."]);
        assert!(repo.guard("install").status.success());
        repo.ok(&["commit", "--quiet", "-m", "test: initialize safe fixture"]);
        repo
    }

    fn write(&self, path: &str, bytes: &[u8]) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn git(&self, args: &[&str]) -> Output {
        Command::new("git")
            .current_dir(&self.0)
            .args(args)
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> Output {
        let output = self.git(args);
        assert!(output.status.success(), "fixture operation failed");
        output
    }

    fn guard(&self, action: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_privacy-guard"))
            .current_dir(&self.0)
            .arg(action)
            .output()
            .unwrap()
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn staged_binary_leak_blocks_even_when_worktree_is_clean() {
    let repo = Repository::new();
    let before = repo.ok(&["rev-parse", "HEAD"]).stdout;
    repo.write(
        "build.log",
        b"compiler\0fixture-private-identity\0diagnostic\n",
    );
    repo.write(".gitattributes", b"*.log binary\n");
    repo.ok(&["add", "build.log", ".gitattributes"]);
    repo.write("build.log", b"clean working copy\n");
    for extra in [false, true] {
        let mut args = vec![
            "commit",
            "--quiet",
            "-m",
            "test: reject staged personal data",
        ];
        if extra {
            args.push("--no-verify");
        }
        let output = repo.git(&args);
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fixture-private-identity"));
        assert_eq!(repo.ok(&["rev-parse", "HEAD"]).stdout, before);
    }
    repo.ok(&["add", "build.log"]);
    repo.ok(&["commit", "--quiet", "-m", "test: accept sanitized bytes"]);
}

#[test]
fn missing_policy_stale_binary_and_private_commit_message_block() {
    let repo = Repository::new();
    repo.write("safe.txt", b"safe\n");
    repo.ok(&["add", "safe.txt"]);
    assert!(
        !repo
            .git(&["commit", "--quiet", "-m", "test: fixture-private-identity"])
            .status
            .success()
    );
    fs::remove_file(repo.0.join(".git/privacy/terms.json")).unwrap();
    assert!(
        !repo
            .git(&["commit", "--quiet", "-m", "test: missing policy"])
            .status
            .success()
    );
    repo.write(
        ".git/privacy/terms.json",
        br#"{"terms":["fixture-private-identity"]}"#,
    );
    repo.write(
        "tools/workspace-tools/src/bin/privacy-guard.rs",
        b"// stale source\n",
    );
    repo.ok(&["add", "tools/workspace-tools/src/bin/privacy-guard.rs"]);
    assert!(
        !repo
            .git(&["commit", "--quiet", "-m", "test: stale hook"])
            .status
            .success()
    );
}

#[test]
fn push_rejects_a_leak_hidden_in_an_earlier_commit() {
    let repo = Repository::new();
    repo.write("leak.log", b"fixture-private-identity\n");
    repo.ok(&["add", "leak.log"]);
    // Deliberately create contrary history in this disposable repository.
    repo.ok(&[
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "--quiet",
        "-m",
        "test: unsafe historical fixture",
    ]);
    repo.write("leak.log", b"sanitized\n");
    repo.ok(&["add", "leak.log"]);
    repo.ok(&[
        "commit",
        "--quiet",
        "-m",
        "test: safe tip with unsafe ancestor",
    ]);
    assert!(!repo.guard("history").status.success());
    let remote = repo.0.join("local-remote.git");
    repo.ok(&["init", "--bare", "--quiet", remote.to_str().unwrap()]);
    let output = repo.git(&["push", remote.to_str().unwrap(), "HEAD:refs/heads/test"]);
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("fixture-private-identity"));
    assert!(
        repo.ok(&["ls-remote", remote.to_str().unwrap()])
            .stdout
            .is_empty()
    );
}

#[test]
fn derived_radio_records_and_replacement_objects_cannot_hide_a_leak() {
    let repo = Repository::new();
    for contents in [
        format!("{{\"{}\":{}}}", "address", "[1,2,3,4,5,6]"),
        format!("{{\"{}\":\"{}\"}}", "payload_hex", "1234"),
        format!("{{\"{}\":{}}}", "direct_address", "[1,2,3,4,5,6]"),
    ] {
        repo.write("renamed.jsonl", contents.as_bytes());
        repo.ok(&["add", "renamed.jsonl"]);
        assert!(!repo.guard("check").status.success());
    }
    repo.write("renamed.jsonl", b"fixture-private-identity\n");
    repo.ok(&["add", "renamed.jsonl"]);
    let leaked = repo.ok(&["rev-parse", ":renamed.jsonl"]);
    repo.write("replacement.txt", b"clean replacement\n");
    let clean = repo.ok(&["hash-object", "-w", "replacement.txt"]);
    repo.ok(&[
        "replace",
        String::from_utf8_lossy(&leaked.stdout).trim(),
        String::from_utf8_lossy(&clean.stdout).trim(),
    ]);
    assert!(!repo.guard("check").status.success());
}

#[test]
fn public_tip_cannot_be_pushed_under_a_private_reference_name() {
    let repo = Repository::new();
    let remote = repo.0.join("local-remote.git");
    repo.ok(&["init", "--bare", "--quiet", remote.to_str().unwrap()]);
    assert!(
        !repo
            .git(&[
                "push",
                remote.to_str().unwrap(),
                "HEAD:refs/heads/fixture-private-identity"
            ])
            .status
            .success()
    );
    assert!(
        repo.ok(&["ls-remote", remote.to_str().unwrap()])
            .stdout
            .is_empty()
    );
    repo.ok(&[
        "push",
        "--quiet",
        remote.to_str().unwrap(),
        "HEAD:refs/heads/clean",
    ]);
    repo.ok(&["branch", "fixture-private-identity"]);
    assert!(!repo.guard("history").status.success());
}
