//! Bounded existing-Kismet transport, private receipts and exact offline replay.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use kismet_worker::{BYTE_LIMIT, CredentialFile, Request, STDERR_LIMIT, Status};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wifi_observer::Summary;

use crate::capture::{create_private_directory, private_file, save_json};

const VERSION: u32 = 1;
const FILES: [&str; 7] = [
    "request.bin",
    "packets.pcapng",
    "transport.stderr",
    "transport-status.json",
    "worker-provenance.json",
    "summary.json",
    "capture-status.json",
];

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Provenance {
    schema_version: u32,
    pub(crate) binary_sha256: String,
    pub(crate) build_inputs_sha256: String,
    rustc_version: String,
    curl_version: String,
}

impl Provenance {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != VERSION
            || !digest_shape(&self.binary_sha256)
            || !digest_shape(&self.build_inputs_sha256)
            || self.rustc_version != "rustc 1.90.0 (1159e78c4 2025-09-14)"
            || self.curl_version != "8.17.0"
        {
            return Err("unsupported Wi-Fi worker provenance");
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CredentialSource {
    path: String,
    size: u64,
    modified_ns: u64,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    schema_version: u32,
    ssh_alias: String,
    worker_path: String,
    source_uuid: String,
    expected_pid: u32,
    username: CredentialSource,
    password: CredentialSource,
    pub(crate) provenance: Provenance,
}

impl Profile {
    pub(crate) fn request(&self) -> Result<Request, &'static str> {
        if self.schema_version != VERSION
            || self.ssh_alias.is_empty()
            || self.ssh_alias.starts_with('-')
            || self.ssh_alias.len() > 128
            || !self
                .ssh_alias
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || !self.worker_path.starts_with('/')
            || self.worker_path.len() > 4096
            || self.worker_path.split('/').any(|c| c == "." || c == "..")
            || !self
                .worker_path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        {
            return Err("invalid private Wi-Fi capture profile");
        }
        self.provenance.validate()?;
        let request = Request {
            source_uuid: self.source_uuid.to_ascii_lowercase(),
            expected_pid: self.expected_pid,
            username: CredentialFile {
                path: self.username.path.clone(),
                size: self.username.size,
                modified_ns: self.username.modified_ns,
            },
            password: CredentialFile {
                path: self.password.path.clone(),
                size: self.password.size,
                modified_ns: self.password.modified_ns,
            },
        };
        request
            .validate()
            .map_err(|_| "invalid private Wi-Fi capture profile")?;
        Ok(request)
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Transport {
    pub(crate) schema_version: u32,
    pub(crate) ssh_exit_code: Option<i32>,
    pub(crate) ssh_signal: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) io_failed: bool,
    pub(crate) limit_exceeded: bool,
    pub(crate) worker_status_line: Option<String>,
}

impl Transport {
    fn validate(
        &self,
        raw: &[u8],
        stderr: &[u8],
        request: &[u8],
        provenance: &Provenance,
    ) -> Result<(), &'static str> {
        let worker = Status::parse(stderr).map_err(|_| "Wi-Fi worker status did not validate")?;
        if self.schema_version != VERSION
            || self.ssh_exit_code != Some(0)
            || self.ssh_signal.is_some()
            || self.timed_out
            || self.io_failed
            || self.limit_exceeded
            || !worker.accepted()
            || worker.request_sha256 != digest(request)
            || worker.binary_sha256 != provenance.binary_sha256
            || worker.build_inputs_sha256 != provenance.build_inputs_sha256
            || worker.saved_bytes != raw.len()
            || self.worker_status_line.as_deref() != Some(worker.line().as_str())
        {
            return Err("Wi-Fi transport did not complete its fixed sample");
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    file: String,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    success: bool,
    error: Option<String>,
    artifacts: Vec<Artifact>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest_shape(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>, &'static str> {
    let mut b = serde_json::to_vec_pretty(v).map_err(|_| "cannot encode Wi-Fi capture metadata")?;
    b.push(b'\n');
    Ok(b)
}

fn copy_bounded(
    mut source: impl Read,
    destination: &mut impl Write,
    cap: usize,
    stop: &AtomicBool,
) -> (bool, bool) {
    let mut total = 0;
    let mut bytes = [0; 8192];
    loop {
        match source.read(&mut bytes) {
            Ok(0) => return (false, false),
            Ok(n) => {
                let retain = n.min(cap - total);
                if destination.write_all(&bytes[..retain]).is_err() {
                    stop.store(true, Ordering::Relaxed);
                    return (true, false);
                }
                total += retain;
                if n > retain {
                    stop.store(true, Ordering::Relaxed);
                    return (false, true);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                stop.store(true, Ordering::Relaxed);
                return (true, false);
            }
        }
    }
}

pub(crate) fn ssh_command(profile: &Profile) -> Command {
    let mut command = Command::new("ssh");
    command.args([
        "-oBatchMode=yes",
        "-oStrictHostKeyChecking=yes",
        "-oConnectTimeout=8",
        "-oClearAllForwardings=yes",
        "-oRequestTTY=no",
        "--",
        &profile.ssh_alias,
        "sudo",
        "-n",
        &profile.worker_path,
    ]);
    command
}

fn ssh(
    profile: &Profile,
    request: Option<&[u8]>,
    stdout: &mut (impl Write + Send),
    stderr: &mut (impl Write + Send),
) -> Result<Transport, &'static str> {
    let mut command = ssh_command(profile);
    if request.is_none() {
        command.arg("--provenance");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "cannot start private Wi-Fi transport")?;
    let mut transport = Transport {
        schema_version: VERSION,
        ssh_exit_code: None,
        ssh_signal: None,
        timed_out: false,
        io_failed: false,
        limit_exceeded: false,
        worker_status_line: None,
    };
    if let Some(mut input) = child.stdin.take()
        && let Some(bytes) = request
        && input.write_all(bytes).is_err()
    {
        transport.io_failed = true;
    }
    let child_out = child
        .stdout
        .take()
        .ok_or("cannot acquire Wi-Fi transport output")?;
    let child_err = child
        .stderr
        .take()
        .ok_or("cannot acquire Wi-Fi transport status")?;
    let stop = AtomicBool::new(false);
    thread::scope(|scope| {
        let out = scope.spawn(|| {
            copy_bounded(
                child_out,
                stdout,
                if request.is_some() { BYTE_LIMIT } else { 4096 },
                &stop,
            )
        });
        let err = scope.spawn(|| copy_bounded(child_err, stderr, STDERR_LIMIT + 4096, &stop));
        let deadline =
            Instant::now() + Duration::from_secs(if request.is_some() { 132 } else { 10 });
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(_) => {
                    transport.io_failed = true;
                    let _ = child.kill();
                    break;
                }
            }
            if stop.load(Ordering::Relaxed) || transport.io_failed {
                let _ = child.kill();
                break;
            }
            if Instant::now() >= deadline {
                transport.timed_out = true;
                let _ = child.kill();
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        match child.wait() {
            Ok(exit) => {
                transport.ssh_exit_code = exit.code();
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    transport.ssh_signal = exit.signal();
                }
            }
            Err(_) => transport.io_failed = true,
        }
        for joined in [out.join(), err.join()] {
            match joined {
                Ok((failed, limit)) => {
                    transport.io_failed |= failed;
                    transport.limit_exceeded |= limit;
                }
                Err(_) => transport.io_failed = true,
            }
        }
    });
    Ok(transport)
}

fn capture_with<P, F>(
    directory: &Path,
    request: &Request,
    expected: &Provenance,
    provenance: P,
    observe: F,
) -> Result<Summary, &'static str>
where
    P: FnOnce() -> Result<Provenance, &'static str>,
    F: FnOnce(&[u8], &mut File, &mut File) -> Result<Transport, &'static str>,
{
    // Both packet and status destinations exist before the first remote request.
    let mut raw = private_file(&directory.join("packets.pcapng"))?;
    let mut stderr = private_file(&directory.join("transport.stderr"))?;
    let mut receipt = Receipt {
        schema_version: VERSION,
        success: false,
        error: Some("storage".into()),
        artifacts: Vec::new(),
    };
    let result = (|| {
        let encoded = request
            .encode()
            .map_err(|_| "invalid private Wi-Fi request")?;
        let mut file = private_file(&directory.join("request.bin"))?;
        file.write_all(&encoded)
            .map_err(|_| "cannot save Wi-Fi request")?;
        file.sync_all()
            .map_err(|_| "cannot synchronize Wi-Fi request")?;
        receipt.error = Some("provenance".into());
        let actual = provenance()?;
        actual.validate()?;
        save_json(&directory.join("worker-provenance.json"), &actual)?;
        if actual != *expected {
            return Err("remote Wi-Fi worker differs from approved provenance");
        }
        receipt.error = Some("transport".into());
        let mut status = observe(&encoded, &mut raw, &mut stderr)?;
        raw.sync_all()
            .map_err(|_| "cannot synchronize private Wi-Fi packets")?;
        stderr
            .sync_all()
            .map_err(|_| "cannot synchronize private Wi-Fi status")?;
        let packets = crate::read_input(&directory.join("packets.pcapng"))?;
        let errors = crate::read_input(&directory.join("transport.stderr"))?;
        status.worker_status_line = Status::parse(&errors).ok().map(|status| status.line());
        save_json(&directory.join("transport-status.json"), &status)?;
        status.validate(&packets, &errors, &encoded, &actual)?;
        receipt.error = Some("observation".into());
        let summary = wifi_observer::analyze(&packets)
            .map_err(|_| "Wi-Fi packet observation did not validate")?;
        if summary.packet_count != kismet_worker::PACKET_LIMIT as u64 {
            return Err("Wi-Fi packet count disagrees with fixed sample");
        }
        save_json(&directory.join("summary.json"), &summary)?;
        receipt.success = true;
        receipt.error = None;
        Ok(summary)
    })();
    for name in &FILES[..FILES.len() - 1] {
        let path = directory.join(name);
        if path.is_file() {
            let bytes = crate::read_input(&path)?;
            receipt.artifacts.push(Artifact {
                file: (*name).into(),
                sha256: digest(&bytes),
            });
        }
    }
    save_json(&directory.join("capture-status.json"), &receipt)?;
    result
}

pub(crate) fn load_profile(profile: &Path) -> Result<Profile, &'static str> {
    let profile: Profile = serde_json::from_slice(&crate::read_input(profile)?)
        .map_err(|_| "invalid private Wi-Fi capture profile")?;
    profile.request()?;
    Ok(profile)
}

pub(crate) fn fetch_provenance(profile: &Profile) -> Result<Provenance, &'static str> {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let status = ssh(profile, None, &mut out, &mut err)?;
    if status.ssh_exit_code != Some(0)
        || status.ssh_signal.is_some()
        || status.io_failed
        || status.limit_exceeded
        || status.timed_out
        || !err.is_empty()
    {
        return Err("remote Wi-Fi provenance request failed");
    }
    let provenance: Provenance =
        serde_json::from_slice(&out).map_err(|_| "invalid remote Wi-Fi provenance")?;
    provenance.validate()?;
    if provenance != profile.provenance {
        return Err("Wi-Fi worker provenance changed");
    }
    Ok(provenance)
}

pub(crate) fn capture_using<F>(
    profile: &Profile,
    directory: &Path,
    observe: F,
) -> Result<Summary, &'static str>
where
    F: FnOnce(&[u8], &mut File, &mut File) -> Result<Transport, &'static str>,
{
    let request = profile.request()?;
    create_private_directory(directory)?;
    capture_with(
        directory,
        &request,
        &profile.provenance,
        || fetch_provenance(profile),
        observe,
    )
}

pub(crate) fn capture(profile: &Path, directory: &Path) -> Result<Summary, &'static str> {
    let profile = load_profile(profile)?;
    capture_using(&profile, directory, |request, out, err| {
        ssh(&profile, Some(request), out, err)
    })
}

pub(crate) fn replay(directory: &Path) -> Result<Summary, &'static str> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect Wi-Fi capture directory")? {
        let entry = entry.map_err(|_| "cannot inspect Wi-Fi capture entry")?;
        if !entry
            .file_type()
            .map_err(|_| "cannot inspect Wi-Fi capture file")?
            .is_file()
        {
            return Err("Wi-Fi capture requires regular files");
        }
        names.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid Wi-Fi capture filename")?,
        );
    }
    if names != FILES.into_iter().map(str::to_owned).collect() {
        return Err("Wi-Fi capture inventory is incomplete or unexpected");
    }
    let receipt: Receipt =
        serde_json::from_slice(&crate::read_input(&directory.join("capture-status.json"))?)
            .map_err(|_| "invalid Wi-Fi capture receipt")?;
    if receipt.schema_version != VERSION
        || !receipt.success
        || receipt.error.is_some()
        || receipt.artifacts.len() != 6
    {
        return Err("Wi-Fi capture records a failed or incomplete attempt");
    }
    for (artifact, name) in receipt.artifacts.iter().zip(&FILES[..6]) {
        if artifact.file != *name
            || !digest_shape(&artifact.sha256)
            || artifact.sha256 != digest(&crate::read_input(&directory.join(name))?)
        {
            return Err("Wi-Fi capture bytes disagree with completion receipt");
        }
    }
    let encoded = crate::read_input(&directory.join("request.bin"))?;
    Request::decode(&*encoded).map_err(|_| "invalid saved Wi-Fi request")?;
    let provenance: Provenance = serde_json::from_slice(&crate::read_input(
        &directory.join("worker-provenance.json"),
    )?)
    .map_err(|_| "invalid saved Wi-Fi provenance")?;
    provenance.validate()?;
    let packets = crate::read_input(&directory.join("packets.pcapng"))?;
    let errors = crate::read_input(&directory.join("transport.stderr"))?;
    let status: Transport = serde_json::from_slice(&crate::read_input(
        &directory.join("transport-status.json"),
    )?)
    .map_err(|_| "invalid saved Wi-Fi transport status")?;
    status.validate(&packets, &errors, &encoded, &provenance)?;
    let summary = wifi_observer::analyze(&packets)
        .map_err(|_| "Wi-Fi packet observation did not validate")?;
    if summary.packet_count != kismet_worker::PACKET_LIMIT as u64 {
        return Err("Wi-Fi packet count disagrees with fixed sample");
    }
    if json(&summary)? != crate::read_input(&directory.join("summary.json"))? {
        return Err("Wi-Fi saved summary disagrees with complete replay");
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kismet_worker::Stop;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "wifi-capture-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn request() -> Request {
        Request {
            source_uuid: "00000000-0000-0000-0000-000000000001".into(),
            expected_pid: 1,
            username: CredentialFile {
                path: "/synthetic/config".into(),
                size: 1,
                modified_ns: 1,
            },
            password: CredentialFile {
                path: "/synthetic/config".into(),
                size: 1,
                modified_ns: 1,
            },
        }
    }
    fn inputs() -> Provenance {
        Provenance {
            schema_version: 1,
            binary_sha256: "a".repeat(64),
            build_inputs_sha256: "b".repeat(64),
            rustc_version: "rustc 1.90.0 (1159e78c4 2025-09-14)".into(),
            curl_version: "8.17.0".into(),
        }
    }
    #[test]
    fn profile_uuid_case_normalizes_to_the_same_strict_request() {
        let canonical = ["abcdef12", "ab34", "cd56", "ef78", "abcdef123456"].join("-");
        let mut profile = Profile {
            schema_version: VERSION,
            ssh_alias: "synthetic-receiver".into(),
            worker_path: "/synthetic/kismet-worker".into(),
            source_uuid: canonical.clone(),
            expected_pid: 1,
            username: CredentialSource {
                path: "/synthetic/config".into(),
                size: 1,
                modified_ns: 1,
            },
            password: CredentialSource {
                path: "/synthetic/config".into(),
                size: 1,
                modified_ns: 1,
            },
            provenance: inputs(),
        };
        let lower = profile.request().unwrap();
        assert_eq!(lower.source_uuid, canonical);
        let encoded = lower.encode().unwrap();
        let mixed: String = canonical
            .chars()
            .enumerate()
            .map(|(index, c)| {
                if index % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();
        for source in [canonical.to_ascii_uppercase(), mixed, canonical.clone()] {
            profile.source_uuid = source;
            let request = profile.request().unwrap();
            assert_eq!(request.source_uuid, canonical);
            assert_eq!(request.encode().unwrap(), encoded);
        }
        let mut direct = lower;
        direct.source_uuid = canonical.to_ascii_uppercase();
        assert!(direct.validate().is_err());
    }
    fn packets(count: usize) -> Vec<u8> {
        let mut b = vec![
            0x0a, 0x0d, 0x0d, 0x0a, 28, 0, 0, 0, 0x4d, 0x3c, 0x2b, 0x1a, 1, 0, 0, 0,
        ];
        b.extend_from_slice(&u64::MAX.to_le_bytes());
        b.extend_from_slice(&28_u32.to_le_bytes());
        b.extend_from_slice(&[
            1, 0, 0, 0, 20, 0, 0, 0, 127, 0, 0, 0, 255, 255, 0, 0, 20, 0, 0, 0,
        ]);
        for n in 0..count {
            b.extend_from_slice(&[6, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            b.extend_from_slice(&(n as u32).to_le_bytes());
            b.extend_from_slice(&[
                10, 0, 0, 0, 10, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 4, 0, 0, 0, 44, 0, 0, 0,
            ]);
        }
        b
    }
    fn fake(directory: &Path, http: u16) -> Result<Summary, &'static str> {
        let provenance = inputs();
        let raw = packets(32);
        capture_with(
            directory,
            &request(),
            &provenance,
            || Ok(inputs()),
            |request, out, err| {
                out.write_all(&raw).unwrap();
                let status = Status {
                    stop: Stop::Boundary,
                    saved_bytes: raw.len(),
                    observed_bytes: raw.len(),
                    epbs: 32,
                    http_status: http,
                    curl_exit: 0,
                    curl_signal: 0,
                    intentional_kill: false,
                    cleanup_success: true,
                    stderr_truncated: false,
                    request_sha256: digest(request),
                    binary_sha256: provenance.binary_sha256.clone(),
                    build_inputs_sha256: provenance.build_inputs_sha256.clone(),
                };
                err.write_all(status.line().as_bytes()).unwrap();
                Ok(Transport {
                    schema_version: 1,
                    ssh_exit_code: Some(0),
                    ssh_signal: None,
                    timed_out: false,
                    io_failed: false,
                    limit_exceeded: false,
                    worker_status_line: None,
                })
            },
        )
    }
    fn rehash(directory: &Path) {
        let mut receipt: Receipt =
            serde_json::from_slice(&fs::read(directory.join("capture-status.json")).unwrap())
                .unwrap();
        for artifact in &mut receipt.artifacts {
            artifact.sha256 = digest(&fs::read(directory.join(&artifact.file)).unwrap());
        }
        fs::write(
            directory.join("capture-status.json"),
            json(&receipt).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn actual_save_replay_and_failed_http_preserve_partial() {
        let t = Temp::new();
        let summary = fake(&t.0, 200).unwrap();
        assert_eq!(summary.packet_count, 32);
        assert_eq!(replay(&t.0).unwrap(), summary);
        assert_eq!(fs::read_dir(&t.0).unwrap().count(), 7);
        let t = Temp::new();
        assert!(fake(&t.0, 401).is_err());
        assert_eq!(fs::read(t.0.join("packets.pcapng")).unwrap(), packets(32));
        assert!(!t.0.join("summary.json").exists());
        assert!(replay(&t.0).is_err());
        let receipt: Receipt =
            serde_json::from_slice(&fs::read(t.0.join("capture-status.json")).unwrap()).unwrap();
        assert!(!receipt.success);
    }
    #[test]
    fn rehashed_request_provenance_summary_and_packet_count_tampering_rejected() {
        for mutation in 0..4 {
            let t = Temp::new();
            fake(&t.0, 200).unwrap();
            match mutation {
                0 => {
                    let mut changed = request();
                    changed.expected_pid = 2;
                    fs::write(t.0.join("request.bin"), changed.encode().unwrap()).unwrap();
                }
                1 => {
                    let mut changed = inputs();
                    changed.binary_sha256 = "c".repeat(64);
                    fs::write(t.0.join("worker-provenance.json"), json(&changed).unwrap()).unwrap();
                }
                2 => {
                    let mut summary = fs::read(t.0.join("summary.json")).unwrap();
                    summary.push(b' ');
                    fs::write(t.0.join("summary.json"), summary).unwrap();
                }
                3 => {
                    let raw = packets(31);
                    fs::write(t.0.join("packets.pcapng"), &raw).unwrap();
                    let errors = fs::read(t.0.join("transport.stderr")).unwrap();
                    let mut status = Status::parse(&errors).unwrap();
                    status.saved_bytes = raw.len();
                    status.observed_bytes = raw.len();
                    fs::write(t.0.join("transport.stderr"), status.line()).unwrap();
                    let mut transport: Transport = serde_json::from_slice(
                        &fs::read(t.0.join("transport-status.json")).unwrap(),
                    )
                    .unwrap();
                    transport.worker_status_line = Some(status.line());
                    fs::write(t.0.join("transport-status.json"), json(&transport).unwrap())
                        .unwrap();
                    let summary = wifi_observer::analyze(&raw).unwrap();
                    fs::write(t.0.join("summary.json"), json(&summary).unwrap()).unwrap();
                }
                _ => unreachable!(),
            }
            rehash(&t.0);
            assert!(replay(&t.0).is_err());
        }
    }
    #[test]
    fn preexisting_raw_file_prevents_remote_call() {
        let t = Temp::new();
        fs::write(t.0.join("packets.pcapng"), b"existing").unwrap();
        assert!(
            capture_with(
                &t.0,
                &request(),
                &inputs(),
                || panic!("remote provenance must not run"),
                |_, _, _| panic!("remote capture must not run")
            )
            .is_err()
        );
    }
}
