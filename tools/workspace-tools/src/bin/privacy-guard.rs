//! Git privacy gate. Checks staged objects rather than their worktree copies.
//! Local hooks require a private identifier list; CI checks generic patterns.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use anyhow::{Result, bail, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const SOURCE_PATH: &str = "tools/workspace-tools/src/bin/privacy-guard.rs";
const SOURCE: &[u8] = include_bytes!("privacy-guard.rs");
const ALLOW_PATH: &str = "quality/privacy-allowlist.json";
const MAX_OBJECT: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateTerms {
    terms: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Exception {
    sha256: String,
    rule: String,
    reason: String,
}

#[derive(Debug)]
struct Policy {
    terms: Vec<String>,
    exceptions: BTreeSet<(String, String)>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decoded_text(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(&[0xff, 0xfe, 0, 0]) || bytes.starts_with(&[0, 0, 0xfe, 0xff]) {
        let little = bytes[0] == 0xff;
        let body = &bytes[4..];
        if !body.len().is_multiple_of(4) {
            return None;
        }
        return body
            .chunks_exact(4)
            .map(|chunk| {
                let word = <[u8; 4]>::try_from(chunk).ok()?;
                char::from_u32(if little {
                    u32::from_le_bytes(word)
                } else {
                    u32::from_be_bytes(word)
                })
            })
            .collect();
    }
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        let little = bytes[0] == 0xff;
        let body = &bytes[2..];
        if !body.len().is_multiple_of(2) {
            return None;
        }
        let words: Vec<_> = body
            .chunks_exact(2)
            .map(|pair| {
                if little {
                    u16::from_le_bytes([pair[0], pair[1]])
                } else {
                    u16::from_be_bytes([pair[0], pair[1]])
                }
            })
            .collect();
        return String::from_utf16(&words).ok();
    }
    std::str::from_utf8(bytes).ok().map(str::to_owned)
}

// Decode JSON string tokens within JSONL and ordinary diagnostics as well as
// complete JSON documents. Keep punctuation so escaped identifier keys retain
// their key/value relationship. Parse failures stay unchanged and never print.
fn unescape_strings(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(position) = rest.find('"') {
        output.push_str(&rest[..position]);
        rest = &rest[position..];
        let mut stream = serde_json::Deserializer::from_str(rest).into_iter::<String>();
        if let Some(Ok(value)) = stream.next() {
            output.push('"');
            output.push_str(&value);
            output.push('"');
            rest = &rest[stream.byte_offset()..];
        } else {
            output.push('"');
            rest = &rest[1..];
        }
    }
    output.push_str(rest);
    output
}

// Never forward subprocess stderr: Git errors can repeat private paths/data.
fn git(args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "Git operation failed (details withheld)"
    );
    Ok(output.stdout)
}

fn git_path(name: &str) -> Result<PathBuf> {
    let bytes = git(&["rev-parse", "--git-path", name])?;
    Ok(PathBuf::from(std::str::from_utf8(&bytes)?.trim()))
}

fn bounded_file(path: &Path, max: usize) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "regular file required"
    );
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take((max + 1) as u64)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= max, "input exceeds privacy-check limit");
    Ok(bytes)
}

impl Policy {
    fn load(public_only: bool) -> Result<Self> {
        let terms = if public_only {
            Vec::new()
        } else {
            let bytes = bounded_file(&git_path("privacy/terms.json")?, 65_536).map_err(|_| {
                anyhow::anyhow!("private terms missing/unreadable; run privacy-guard install")
            })?;
            let input: PrivateTerms = serde_json::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("invalid private terms configuration"))?;
            ensure!(
                !input.terms.is_empty(),
                "private identifier list must not be empty"
            );
            ensure!(
                input
                    .terms
                    .iter()
                    .all(|term| term.trim() == term && term.len() >= 3),
                "invalid private identifier entry"
            );
            input
                .terms
                .into_iter()
                .map(|term| term.to_lowercase())
                .collect()
        };
        let bytes = git(&["show", &format!(":{ALLOW_PATH}")])?;
        let entries: Vec<Exception> = serde_json::from_slice(&bytes)?;
        let mut exceptions = BTreeSet::new();
        for entry in entries {
            ensure!(
                matches!(entry.rule.as_str(), "opaque-artifact" | "raw-artifact"),
                "personal-data rules cannot be excepted"
            );
            ensure!(
                entry.sha256.len() == 64
                    && entry.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                    && !entry.reason.trim().is_empty(),
                "invalid privacy exception"
            );
            ensure!(
                exceptions.insert((entry.sha256, entry.rule)),
                "duplicate privacy exception"
            );
        }
        Ok(Self { terms, exceptions })
    }

    fn findings(&self, bytes: &[u8]) -> BTreeSet<&'static str> {
        // Unsupported binary/encoded input fails closed unless its exact bytes
        // were reviewed as a historical artifact. Inspect ASCII in it as well.
        let nonnul: Vec<u8> = bytes.iter().copied().filter(|b| *b != 0).collect();
        let decoded = decoded_text(bytes);
        let original = decoded
            .as_deref()
            .map(|s| s.replace('\0', ""))
            .unwrap_or_else(|| String::from_utf8_lossy(&nonnul).into_owned());
        let text = format!("{original}\n{}", unescape_strings(&original)).to_lowercase();
        let mut rules = BTreeSet::new();
        if decoded.is_none() {
            rules.insert("opaque-artifact");
        }
        if self.terms.iter().any(|term| text.contains(term)) {
            rules.insert("private-identifier");
        }
        for prefix in ["/users/", "/home/", "\\users\\"] {
            for (position, tail) in text
                .match_indices(prefix)
                .map(|(pos, _)| (pos, &text[pos + prefix.len()..]))
            {
                if prefix == "/users/" && text[..position].ends_with("https://api.github.com") {
                    continue;
                }
                let user: String = tail
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
                    .collect();
                if !user.is_empty() && !matches!(user.as_str(), "builduser" | "runner") {
                    rules.insert("personal-home-path");
                }
            }
        }
        for (position, _) in text.match_indices('@') {
            let before = text[..position].chars().next_back();
            let domain: String = text[position + 1..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
                .collect();
            let domain = domain.trim_end_matches('.');
            let valid_domain = domain.rsplit_once('.').is_some_and(|(host, tld)| {
                !host.is_empty() && tld.len() >= 2 && tld.bytes().all(|b| b.is_ascii_alphabetic())
            });
            if before.is_some_and(|c| c.is_ascii_alphanumeric())
                && valid_domain
                && domain.contains('.')
                && !matches!(
                    domain,
                    "example.com"
                        | "example.invalid"
                        | "noreply.github.com"
                        | "users.noreply.github.com"
                )
            {
                rules.insert("email-address");
            }
        }
        let raw = text.as_bytes();
        for window in raw.windows(17) {
            let separator = window[2];
            if matches!(separator, b':' | b'-')
                && (0..17).all(|i| {
                    if i % 3 == 2 {
                        window[i] == separator
                    } else {
                        window[i].is_ascii_hexdigit()
                    }
                })
            {
                rules.insert("hardware-address");
                break;
            }
        }
        for prefix in ["/dev/cu.usb", "/dev/tty.usb"] {
            for (position, _) in text.match_indices(prefix) {
                let suffix = &text[position + prefix.len()..];
                let token: String = suffix
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
                    .collect();
                if token.chars().any(|c| c.is_ascii_digit()) {
                    rules.insert("serial-device-path");
                }
            }
        }
        for key in [
            "ssid",
            "bssid",
            "mac",
            "serial_number",
            "serialnumber",
            "imei",
            "iccid",
            "eui64",
            "address",
            "direct_address",
            "payload_hex",
            "ssid_hex",
            "device_id",
            "hostname",
        ] {
            let needle = format!("\"{key}\"");
            for (position, _) in text.match_indices(&needle) {
                if text[position + needle.len()..]
                    .trim_start()
                    .strip_prefix(':')
                    .is_some_and(|tail| {
                        tail.trim_start().starts_with('[')
                            || (key != "address"
                                && tail
                                    .trim_start()
                                    .as_bytes()
                                    .first()
                                    .is_some_and(u8::is_ascii_digit))
                    })
                {
                    rules.insert("device-identifier-field");
                }
                if let Some(value) = text[position + needle.len()..]
                    .trim_start()
                    .strip_prefix(':')
                    .and_then(|tail| tail.trim_start().strip_prefix('"'))
                    .and_then(|tail| tail.split('"').next())
                    && !value.is_empty()
                    && !value.starts_with('<')
                    && value != "redacted"
                    && !(key == "address"
                        && value.strip_prefix("0x").is_some_and(|hex| {
                            !hex.is_empty()
                                && hex.len() <= 8
                                && hex.bytes().all(|b| b.is_ascii_hexdigit())
                        }))
                {
                    rules.insert("device-identifier-field");
                }
            }
        }
        if text
            .lines()
            .any(|line| line.starts_with("wc ") && line.contains('{'))
        {
            rules.insert("radio-capture");
        }
        if original
            .lines()
            .filter(|line| line.trim_start().starts_with('{'))
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .any(|value| {
                value
                    .get("chunk")
                    .is_some_and(|chunk| chunk.get("data").is_some() && chunk.get("seq").is_some())
            })
        {
            rules.insert("radio-capture");
        }
        let key_marker = ["-----begin ", "private key-----"].concat();
        let rsa_marker = ["-----begin rsa ", "private key-----"].concat();
        let openssh_marker = ["-----begin openssh ", "private key-----"].concat();
        if [key_marker, rsa_marker, openssh_marker]
            .iter()
            .any(|marker| text.contains(marker))
            || text.lines().any(|line| {
                line.trim_start().starts_with("-----begin ")
                    && line.trim_end().ends_with(" private key-----")
            })
        {
            rules.insert("private-key");
        }
        if bytes.starts_with(&[0x1f, 0x8b])
            || bytes.starts_with(b"PK\x03\x04")
            || bytes.starts_with(b"\x7fELF")
            || bytes.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
            || bytes.starts_with(b"MZ")
            || bytes.starts_with(b"%PDF-")
            || bytes.starts_with(b"GIF87a")
            || bytes.starts_with(b"GIF89a")
            || bytes.get(257..262) == Some(b"ustar")
        {
            rules.insert("opaque-artifact");
        }
        // Only the two artifact categories can use byte-exact exceptions.
        let hash = digest(bytes);
        rules.retain(|rule| {
            !(self
                .exceptions
                .contains(&(hash.clone(), (*rule).to_owned()))
                || (*rule == "opaque-artifact"
                    && self
                        .exceptions
                        .contains(&(hash.clone(), "raw-artifact".to_owned()))))
        });
        rules
    }

    fn path_findings(&self, path: &str, bytes: &[u8]) -> BTreeSet<&'static str> {
        let mut rules = self.findings(path.as_bytes());
        let parts: Vec<_> = path.split('/').collect();
        if parts.iter().any(|part| {
            matches!(*part, "private" | "backups" | ".cache" | ".env") || part.starts_with(".env.")
        }) {
            rules.insert("private-storage-path");
        }
        if path.ends_with(".raw")
            || path.ends_with(".frame")
            || path.ends_with(".bin")
            || path.ends_with(".elf")
            || path.ends_with(".pcap")
            || path.ends_with(".pcapng")
        {
            let key = (digest(bytes), "raw-artifact".to_owned());
            if !self.exceptions.contains(&key) {
                rules.insert("raw-artifact");
            }
        }
        rules
    }
}

#[derive(Debug)]
struct Objects {
    child: std::process::Child,
    input: std::process::ChildStdin,
    output: BufReader<std::process::ChildStdout>,
}

impl Objects {
    fn open() -> Result<Self> {
        let mut child = Command::new("git")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("missing Git input pipe"))?;
        let output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("missing Git output pipe"))?,
        );
        Ok(Self {
            child,
            input,
            output,
        })
    }

    fn read(&mut self, oid: &str) -> Result<(String, Vec<u8>)> {
        ensure!(
            oid.len() == 40 && oid.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid object identity"
        );
        writeln!(self.input, "{oid}")?;
        self.input.flush()?;
        let mut header = String::new();
        self.output.read_line(&mut header)?;
        let fields: Vec<_> = header.split_whitespace().collect();
        ensure!(
            fields.len() == 3 && fields[0] == oid,
            "unexpected Git object response"
        );
        let size: usize = fields[2].parse()?;
        ensure!(size <= MAX_OBJECT, "object exceeds privacy-check limit");
        let mut bytes = vec![0; size];
        self.output.read_exact(&mut bytes)?;
        let mut newline = [0];
        self.output.read_exact(&mut newline)?;
        ensure!(newline == [b'\n'], "invalid Git object framing");
        Ok((fields[1].to_owned(), bytes))
    }
}

impl Drop for Objects {
    fn drop(&mut self) {
        // Never leave the helper waiting for another request, including errors.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn report(oid: &str, rules: &BTreeSet<&str>) -> bool {
    for rule in rules {
        eprintln!("privacy blocked object {}: {rule}", &oid[..12]);
    }
    !rules.is_empty()
}

fn check_index(policy: &Policy, require_source: bool) -> Result<()> {
    let entries = git(&["ls-files", "--stage", "-z"])?;
    let mut objects = Objects::open()?;
    let mut blocked = false;
    let mut source_matches = false;
    let mut count = 0;
    for entry in entries.split(|b| *b == 0).filter(|entry| !entry.is_empty()) {
        let (metadata, path) = entry.split_at(
            entry
                .iter()
                .position(|b| *b == b'\t')
                .ok_or_else(|| anyhow::anyhow!("invalid index entry"))?,
        );
        let fields: Vec<_> = std::str::from_utf8(metadata)?.split_whitespace().collect();
        ensure!(
            fields.len() == 3 && fields[2] == "0",
            "unmerged index rejected"
        );
        ensure!(
            matches!(fields[0], "100644" | "100755"),
            "symlinks/submodules require explicit privacy review"
        );
        let path = std::str::from_utf8(&path[1..])?;
        let (kind, bytes) = objects.read(fields[1])?;
        ensure!(kind == "blob", "index entry is not a blob");
        let mut rules = policy.findings(&bytes);
        rules.extend(policy.path_findings(path, &bytes));
        blocked |= report(fields[1], &rules);
        if path == SOURCE_PATH {
            source_matches = bytes == SOURCE;
        }
        count += 1;
    }
    ensure!(
        !require_source || source_matches,
        "privacy hook is stale or source is missing; rebuild and reinstall it"
    );
    ensure!(
        !blocked,
        "staged privacy check failed; matched contents were withheld"
    );
    eprintln!("privacy staged check passed: {count} files");
    Ok(())
}

fn check_metadata(policy: &Policy) -> Result<()> {
    for name in ["GIT_AUTHOR_IDENT", "GIT_COMMITTER_IDENT"] {
        ensure!(
            policy.findings(&git(&["var", name])?).is_empty(),
            "commit identity violates privacy policy (details withheld)"
        );
    }
    Ok(())
}

fn check_history(policy: &Policy, extra: &[String]) -> Result<()> {
    ensure!(
        git(&["rev-parse", "--is-shallow-repository"])? == b"false\n",
        "complete history is required for a push audit"
    );
    let refs = git(&["for-each-ref", "--format=%(refname)"])?;
    ensure!(
        policy.findings(&refs).is_empty(),
        "reference name violates privacy policy"
    );
    let mut args = vec!["rev-list", "--objects", "--all", "--no-object-names"];
    args.extend(extra.iter().map(String::as_str));
    let inventory = git(&args)?;
    let mut ids: BTreeSet<String> = std::str::from_utf8(&inventory)?
        .lines()
        .map(str::to_owned)
        .collect();
    // Include annotated tag messages, which rev-list dereferences to commits.
    let tags = git(&["for-each-ref", "--format=%(objectname)", "refs/tags"])?;
    ids.extend(std::str::from_utf8(&tags)?.lines().map(str::to_owned));
    ids.extend(extra.iter().cloned());
    let mut objects = Objects::open()?;
    let mut blocked = false;
    for oid in &ids {
        let (kind, bytes) = objects.read(oid)?;
        ensure!(
            matches!(kind.as_str(), "blob" | "tree" | "commit" | "tag"),
            "unsupported Git object type"
        );
        if kind != "tree" {
            blocked |= report(oid, &policy.findings(&bytes));
        }
        // Tree records include raw object IDs; inspect each path and retrieve
        // the corresponding blob for extension-based restrictions too.
        if kind == "tree" {
            let entries = git(&["ls-tree", "-z", oid])?;
            for entry in entries.split(|b| *b == 0).filter(|entry| !entry.is_empty()) {
                let split = entry
                    .iter()
                    .position(|b| *b == b'\t')
                    .ok_or_else(|| anyhow::anyhow!("invalid tree entry"))?;
                let fields: Vec<_> = std::str::from_utf8(&entry[..split])?
                    .split_whitespace()
                    .collect();
                ensure!(
                    fields.len() == 3 && matches!(fields[1], "blob" | "tree"),
                    "submodule privacy cannot be checked"
                );
                let (_, contents) = objects.read(fields[2])?;
                blocked |= report(
                    fields[2],
                    &policy.path_findings(std::str::from_utf8(&entry[split + 1..])?, &contents),
                );
            }
        }
    }
    ensure!(
        !blocked,
        "history privacy check failed; matched contents were withheld"
    );
    eprintln!("privacy history check passed: {} objects", ids.len());
    Ok(())
}

#[cfg(unix)]
fn install() -> Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
    let directory = git_path("privacy")?;
    fs::create_dir_all(&directory)?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    let hooks = directory.join("hooks");
    fs::create_dir_all(&hooks)?;
    let hooks = hooks.canonicalize()?;
    let configured = Command::new("git")
        .args(["config", "--get", "core.hooksPath"])
        .output()?;
    if configured.status.success() {
        ensure!(
            Path::new(std::str::from_utf8(&configured.stdout)?.trim()).canonicalize()? == hooks,
            "existing hooksPath requires integration; it will not be overwritten"
        );
    } else {
        for name in ["pre-commit", "prepare-commit-msg", "commit-msg", "pre-push"] {
            ensure!(
                !git_path(&format!("hooks/{name}"))?.try_exists()?,
                "existing hook requires integration; it will not be overwritten"
            );
        }
    }
    let terms_path = directory.join("terms.json");
    if !terms_path.try_exists()? {
        let mut terms = BTreeSet::new();
        if let Some(home) = env::var_os("HOME")
            && let Some(name) = Path::new(&home)
                .file_name()
                .and_then(|value| value.to_str())
            && name.len() >= 3
        {
            terms.insert(name.to_lowercase());
        }
        if let Ok(user) = env::var("USER")
            && user.len() >= 3
        {
            terms.insert(user.to_lowercase());
        }
        #[cfg(target_os = "macos")]
        if let Ok(output) = Command::new("id").arg("-F").output()
            && output.status.success()
        {
            let name = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            if name.len() >= 3 {
                terms.insert(name.clone());
                terms.extend(
                    name.split_whitespace()
                        .filter(|part| part.len() >= 4)
                        .map(str::to_owned),
                );
            }
        }
        ensure!(
            !terms.is_empty(),
            "cannot infer private identifiers; configure terms.json locally"
        );
        let bytes = serde_json::to_vec_pretty(&serde_json::json!({ "terms": terms }))?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&terms_path)?;
        file.write_all(&bytes)?;
    }
    let executable = directory.join("privacy-guard");
    fs::copy(env::current_exe()?, &executable)?;
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))?;
    for name in ["pre-commit", "prepare-commit-msg", "commit-msg", "pre-push"] {
        let path = hooks.join(name);
        if fs::symlink_metadata(&path).is_ok() {
            ensure!(
                fs::read_link(&path)? == Path::new("../privacy-guard"),
                "unexpected existing managed hook"
            );
        } else {
            symlink("../privacy-guard", path)?;
        }
    }
    git(&[
        "config",
        "--local",
        "core.hooksPath",
        hooks
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-UTF-8 hook directory"))?,
    ])?;
    eprintln!("privacy hooks installed; personal terms stay in private Git metadata");
    Ok(())
}

#[cfg(not(unix))]
fn install() -> Result<()> {
    bail!("hook installation currently supports Unix hosts only")
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let name = Path::new(
        args.first()
            .ok_or_else(|| anyhow::anyhow!("missing executable name"))?,
    )
    .file_name()
    .and_then(|value| value.to_str())
    .unwrap_or("");
    let hook = matches!(
        name,
        "pre-commit" | "prepare-commit-msg" | "commit-msg" | "pre-push"
    );
    let action = if hook {
        name
    } else {
        args.get(1).map(String::as_str).unwrap_or("check")
    };
    if action == "install" {
        return install();
    }
    let public_only = !hook && args.iter().any(|arg| arg == "--public-only");
    let policy = Policy::load(public_only)?;
    match action {
        "check" | "pre-commit" => {
            check_index(&policy, hook)?;
            if !public_only {
                check_metadata(&policy)?;
            }
        }
        "prepare-commit-msg" | "commit-msg" => {
            check_index(&policy, true)?;
            check_metadata(&policy)?;
            let path = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("missing commit message path"))?;
            let bytes = bounded_file(Path::new(path), 1024 * 1024)?;
            ensure!(
                policy.findings(&bytes).is_empty(),
                "commit message violates privacy policy (details withheld)"
            );
        }
        "history" => check_history(&policy, &[])?,
        "pre-push" => {
            check_index(&policy, true)?;
            let mut refs = Vec::new();
            for line in io::stdin().lock().lines() {
                let line = line?;
                let fields: Vec<_> = line.split_whitespace().collect();
                ensure!(fields.len() == 4, "malformed push update");
                ensure!(
                    policy.findings(fields[0].as_bytes()).is_empty()
                        && policy.findings(fields[2].as_bytes()).is_empty(),
                    "push reference violates privacy policy"
                );
                if fields[1] != "0".repeat(40) {
                    ensure!(
                        fields[1].len() == 40 && fields[1].bytes().all(|b| b.is_ascii_hexdigit()),
                        "invalid push identity"
                    );
                    refs.push(fields[1].to_owned());
                }
            }
            check_history(&policy, &refs)?;
        }
        _ => {
            bail!("usage: privacy-guard install | check [--public-only] | history [--public-only]")
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            // I/O/JSON errors can contain private paths or input fragments.
            eprintln!(
                "privacy gate failed; fix reported rules or check the installation and policy files"
            );
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Policy {
        Policy {
            terms: vec!["sensitive-person".into()],
            exceptions: BTreeSet::new(),
        }
    }

    #[test]
    fn detects_personal_data_in_binary_and_utf16_without_printing_it() {
        let home = ["/Users/", "private-person/file.log"].concat();
        let wide: Vec<u8> = home.encode_utf16().flat_map(u16::to_le_bytes).collect();
        for bytes in [
            home.into_bytes(),
            wide,
            b"binary\0sensitive-person\0".to_vec(),
        ] {
            assert!(!policy().findings(&bytes).is_empty());
        }
        assert!(policy().findings(b"/Users/builduser/project").is_empty());
        assert!(policy().findings(b"relative/project").is_empty());
    }

    #[test]
    fn identifiers_and_payloads_reject_with_narrow_artifact_exceptions() {
        for value in [
            ["person@", "private.test"].concat(),
            ["12:34:56:", "78:9a:bc"].concat(),
            ["/dev/cu.usbserial-", "98765"].concat(),
            format!("{{\"{}\":\"{}\"}}", "ssid", "private-network"),
            ["WC ", "{\"packet\":1}"].concat(),
        ] {
            assert!(!policy().findings(value.as_bytes()).is_empty());
        }
        let mut policy = policy();
        let bytes = b"\x1f\x8bsynthetic opaque fixture";
        assert!(policy.findings(bytes).contains("opaque-artifact"));
        policy
            .exceptions
            .insert((digest(bytes), "opaque-artifact".into()));
        assert!(policy.findings(bytes).is_empty());
        assert!(!policy.findings(b"\x1f\x8bchanged fixture").is_empty());
        assert!(
            policy
                .path_findings("private/output.json", b"{}")
                .contains("private-storage-path")
        );
        assert!(
            policy
                .path_findings("results/serial.raw", b"{}")
                .contains("raw-artifact")
        );
    }

    #[test]
    fn encoded_identifiers_and_renamed_binary_formats_reject() {
        let policy = Policy {
            terms: vec!["private-é".into()],
            exceptions: BTreeSet::new(),
        };
        for little in [false, true] {
            let mut bytes = if little {
                vec![0xff, 0xfe]
            } else {
                vec![0xfe, 0xff]
            };
            for word in "private-é".encode_utf16() {
                bytes.extend(if little {
                    word.to_le_bytes()
                } else {
                    word.to_be_bytes()
                });
            }
            assert!(policy.findings(&bytes).contains("private-identifier"));
        }
        let escaped = ["\"private-", "\\u00e9\""].concat();
        assert!(
            policy
                .findings(escaped.as_bytes())
                .contains("private-identifier")
        );
        let numeric = format!("{{\"{}\":{}}}", "imei", 123456789);
        assert!(
            policy
                .findings(numeric.as_bytes())
                .contains("device-identifier-field")
        );
        for bytes in [
            b"\x89PNG\r\n\x1a\n".as_slice(),
            b"\xfd7zXZ\0",
            b"7z\xbc\xaf\x27\x1c",
        ] {
            assert!(policy.findings(bytes).contains("opaque-artifact"));
        }
    }
}
