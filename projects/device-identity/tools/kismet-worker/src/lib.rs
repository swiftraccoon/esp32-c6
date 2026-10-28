//! Private bounded request/status wire contract shared by the host and worker.
#![forbid(unsafe_code)]

use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path};
use std::time::UNIX_EPOCH;

mod readiness;
pub use readiness::Ready;

/// Static, value-free worker protocol or credential-source failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error(&'static str);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Error {}

/// Fixed prospective sample size; no transmitter behavior is requested.
pub const PACKET_LIMIT: usize = 32;
/// Maximum retained HTTP body size.
pub const BYTE_LIMIT: usize = 8 * 1024 * 1024;
/// Hard and idle deadline in seconds.
pub const SECONDS: u64 = 120;
/// Maximum retained worker standard error, including HTTP response headers.
pub const STDERR_LIMIT: usize = 64 * 1024;
const MAGIC: &[u8] = b"KISMET-WIFI-REQUEST\x01";
const STATUS: &str = "KISMET_WIFI_STATUS_V1";

/// Approved remote file identity; its contents never enter the wire request.
#[derive(Clone)]
pub struct CredentialFile {
    /// Absolute remote path resolved during private configuration discovery.
    pub path: String,
    /// Expected size of the discovered configuration file.
    pub size: u64,
    /// Expected last-modification time, in Unix nanoseconds.
    pub modified_ns: u64,
}

/// Private request without credential values. Debug is deliberately unavailable.
#[derive(Clone)]
pub struct Request {
    /// Canonical source UUID resolved from existing saved datasource metadata.
    pub source_uuid: String,
    /// Expected existing Kismet process identifier.
    pub expected_pid: u32,
    /// Effective source file for httpd_username.
    pub username: CredentialFile,
    /// Effective source file for httpd_password.
    pub password: CredentialFile,
}

fn text(reader: &mut impl Read) -> Result<String, Error> {
    let mut size = [0; 4];
    reader
        .read_exact(&mut size)
        .map_err(|_| Error("invalid worker request"))?;
    let size = u32::from_be_bytes(size) as usize;
    if size == 0 || size > 4096 {
        return Err(Error("invalid worker request"));
    }
    let mut bytes = vec![0; size];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| Error("invalid worker request"))?;
    String::from_utf8(bytes).map_err(|_| Error("invalid worker request"))
}

fn number(reader: &mut impl Read) -> Result<u64, Error> {
    let mut value = [0; 8];
    reader
        .read_exact(&mut value)
        .map_err(|_| Error("invalid worker request"))?;
    Ok(u64::from_be_bytes(value))
}

fn valid_path(value: &str) -> bool {
    let p = Path::new(value);
    p.is_absolute()
        && value.len() <= 4096
        && !value.chars().any(char::is_control)
        && p.components()
            .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
}

impl Request {
    /// Validate the fixed source and bounded remote file references.
    pub fn validate(&self) -> Result<(), Error> {
        let uuid = self.source_uuid.as_bytes();
        if uuid.len() != 36
            || uuid.iter().enumerate().any(|(n, c)| {
                if [8, 13, 18, 23].contains(&n) {
                    *c != b'-'
                } else {
                    !c.is_ascii_digit() && !(b'a'..=b'f').contains(c)
                }
            })
            || self.expected_pid == 0
            || [&self.username, &self.password].iter().any(|file| {
                !valid_path(&file.path)
                    || file.size == 0
                    || file.size > 1024 * 1024
                    || file.modified_ns == 0
            })
        {
            return Err(Error("invalid worker request"));
        }
        Ok(())
    }

    /// Encode one bounded request, never credential contents.
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let mut bytes = MAGIC.to_vec();
        for value in [&self.source_uuid, &self.username.path, &self.password.path] {
            bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
            bytes.extend_from_slice(value.as_bytes());
        }
        for n in [
            u64::from(self.expected_pid),
            self.username.size,
            self.username.modified_ns,
            self.password.size,
            self.password.modified_ns,
        ] {
            bytes.extend_from_slice(&n.to_be_bytes());
        }
        Ok(bytes)
    }

    /// Decode exactly one request and reject trailing data.
    pub fn decode(mut reader: impl Read) -> Result<Self, Error> {
        let mut magic = [0; MAGIC.len()];
        reader
            .read_exact(&mut magic)
            .map_err(|_| Error("invalid worker request"))?;
        if magic != MAGIC {
            return Err(Error("invalid worker request"));
        }
        let source_uuid = text(&mut reader)?;
        let username_path = text(&mut reader)?;
        let password_path = text(&mut reader)?;
        let expected_pid =
            u32::try_from(number(&mut reader)?).map_err(|_| Error("invalid worker request"))?;
        let username = CredentialFile {
            path: username_path,
            size: number(&mut reader)?,
            modified_ns: number(&mut reader)?,
        };
        let password = CredentialFile {
            path: password_path,
            size: number(&mut reader)?,
            modified_ns: number(&mut reader)?,
        };
        let mut tail = [0];
        if reader
            .read(&mut tail)
            .map_err(|_| Error("invalid worker request"))?
            != 0
        {
            return Err(Error("invalid worker request"));
        }
        let request = Self {
            source_uuid,
            expected_pid,
            username,
            password,
        };
        request.validate()?;
        Ok(request)
    }
}

/// Terminal capture outcome; only Boundary can qualify a completed sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stop {
    Boundary,
    Deadline,
    Idle,
    ByteLimit,
    Framing,
    Read,
    Write,
    Eof,
    Setup,
}

impl Stop {
    fn name(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::Deadline => "deadline",
            Self::Idle => "idle",
            Self::ByteLimit => "byte_limit",
            Self::Framing => "framing",
            Self::Read => "read",
            Self::Write => "write",
            Self::Eof => "eof",
            Self::Setup => "setup",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "boundary" => Self::Boundary,
            "deadline" => Self::Deadline,
            "idle" => Self::Idle,
            "byte_limit" => Self::ByteLimit,
            "framing" => Self::Framing,
            "read" => Self::Read,
            "write" => Self::Write,
            "eof" => Self::Eof,
            "setup" => Self::Setup,
            _ => return None,
        })
    }
}

/// Fixed terminal status separate from raw curl standard error and HTTP status.
#[derive(Clone, Eq, PartialEq)]
pub struct Status {
    pub stop: Stop,
    pub saved_bytes: usize,
    pub observed_bytes: usize,
    pub epbs: usize,
    pub http_status: u16,
    pub curl_exit: i32,
    pub curl_signal: i32,
    pub intentional_kill: bool,
    pub cleanup_success: bool,
    pub stderr_truncated: bool,
    pub request_sha256: String,
    pub binary_sha256: String,
    pub build_inputs_sha256: String,
}

impl Status {
    /// Strict ASCII terminal line appended after private raw standard error.
    pub fn line(&self) -> String {
        format!(
            "\n{STATUS} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
            self.stop.name(),
            self.saved_bytes,
            self.observed_bytes,
            self.epbs,
            self.http_status,
            self.curl_exit,
            self.curl_signal,
            u8::from(self.intentional_kill),
            u8::from(self.cleanup_success),
            u8::from(self.stderr_truncated),
            self.request_sha256,
            self.binary_sha256,
            self.build_inputs_sha256
        )
    }
    /// Decode exactly one final terminal line; reject duplicates or missing status.
    pub fn parse(stderr: &[u8]) -> Result<Self, Error> {
        let text = std::str::from_utf8(stderr).map_err(|_| Error("invalid worker status"))?;
        let mut found = text.lines().filter(|line| line.starts_with(STATUS));
        let line = found.next().ok_or(Error("missing worker status"))?;
        if found.next().is_some() || text.lines().last() != Some(line) {
            return Err(Error("invalid worker status"));
        }
        let f: Vec<_> = line.split_ascii_whitespace().collect();
        if f.len() != 14
            || f[0] != STATUS
            || f[11..].iter().any(|hash| {
                hash.len() != 64
                    || !hash
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(Error("invalid worker status"));
        }
        fn n<T: std::str::FromStr>(s: &str) -> Result<T, Error> {
            s.parse().map_err(|_| Error("invalid worker status"))
        }
        fn b(s: &str) -> Result<bool, Error> {
            match s {
                "0" => Ok(false),
                "1" => Ok(true),
                _ => Err(Error("invalid worker status")),
            }
        }
        Ok(Self {
            stop: Stop::parse(f[1]).ok_or(Error("invalid worker status"))?,
            saved_bytes: n(f[2])?,
            observed_bytes: n(f[3])?,
            epbs: n(f[4])?,
            http_status: n(f[5])?,
            curl_exit: n(f[6])?,
            curl_signal: n(f[7])?,
            intentional_kill: b(f[8])?,
            cleanup_success: b(f[9])?,
            stderr_truncated: b(f[10])?,
            request_sha256: f[11].into(),
            binary_sha256: f[12].into(),
            build_inputs_sha256: f[13].into(),
        })
    }
    /// Whether transport reached its intentional block boundary with HTTP 200.
    pub fn accepted(&self) -> bool {
        self.stop == Stop::Boundary
            && self.epbs == PACKET_LIMIT
            && self.saved_bytes <= BYTE_LIMIT
            && self.saved_bytes > 0
            && self.observed_bytes >= self.saved_bytes
            && self.http_status == 200
            && self.cleanup_success
            && !self.stderr_truncated
            && ((self.curl_exit == 0 && self.curl_signal == 0)
                || (self.intentional_kill && self.curl_signal == 9))
    }
}

/// Read one known effective credential field after verifying unchanged file metadata.
/// Values are never returned in errors or process arguments.
pub fn credential(file: &CredentialFile, key: &str) -> Result<String, Error> {
    if !valid_path(&file.path) {
        return Err(Error("invalid credential source"));
    }
    let mut prefix = std::path::PathBuf::new();
    for part in Path::new(&file.path).components() {
        prefix.push(part);
        if fs::symlink_metadata(&prefix)
            .map_err(|_| Error("credential source unavailable"))?
            .file_type()
            .is_symlink()
        {
            return Err(Error("credential source changed"));
        }
    }
    let meta = fs::metadata(&file.path).map_err(|_| Error("credential source unavailable"))?;
    let modified = meta
        .modified()
        .map_err(|_| Error("credential source unavailable"))?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error("credential source unavailable"))?
        .as_nanos();
    if !meta.is_file() || meta.len() != file.size || modified != u128::from(file.modified_ns) {
        return Err(Error("credential source changed"));
    }
    let contents =
        fs::read_to_string(&file.path).map_err(|_| Error("credential source unavailable"))?;
    let mut value = None;
    for line in contents
        .lines()
        .map(str::trim)
        .filter(|s| !s.starts_with('#'))
    {
        if let Some((name, content)) = line.split_once('=')
            && name.trim().eq_ignore_ascii_case(key)
        {
            if value.is_some() {
                return Err(Error("ambiguous credential field"));
            }
            value = Some(content.trim().to_owned());
        }
    }
    value
        .filter(|s| !s.is_empty() && !s.chars().any(char::is_control))
        .ok_or(Error("credential field unavailable"))
}

/// Curl configuration quoting; never invoke a shell with a credential value.
pub fn curl_quote(value: &str) -> Result<String, Error> {
    if value.chars().any(char::is_control) {
        return Err(Error("invalid credential content"));
    }
    Ok(format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

/// Write the fixed request to a child's private stdin and close it.
pub fn write_request(mut writer: impl Write, request: &Request) -> Result<(), Error> {
    writer
        .write_all(&request.encode()?)
        .map_err(|_| Error("cannot send worker request"))
}

impl fmt::Debug for CredentialFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialFile")
            .field("contents", &"<redacted>")
            .finish()
    }
}
impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("private_fields", &"<redacted>")
            .finish()
    }
}
impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Status")
            .field("stop", &self.stop)
            .field("saved_bytes", &self.saved_bytes)
            .field("epbs", &self.epbs)
            .field("http_status", &self.http_status)
            .field("private_bindings", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    #[test]
    fn request_round_trip_and_malformed_controls() {
        let r = request();
        let encoded = r.encode().unwrap();
        assert_eq!(
            Request::decode(&*encoded).unwrap().source_uuid,
            r.source_uuid
        );
        for end in 0..encoded.len() {
            assert!(Request::decode(&encoded[..end]).is_err());
        }
        let mut extra = encoded;
        extra.push(0);
        assert!(Request::decode(&*extra).is_err());
        let mut r = request();
        r.source_uuid = "../source".into();
        assert!(r.encode().is_err());
        let mut r = request();
        r.password.path = "/a/../b".into();
        assert!(r.encode().is_err());
        assert_eq!(curl_quote("a\\\"b").unwrap(), "\"a\\\\\\\"b\"");
        assert!(curl_quote("secret\nvalue").is_err());
    }
    #[test]
    fn status_round_trip_and_termination_controls() {
        let s = Status {
            stop: Stop::Boundary,
            saved_bytes: 100,
            observed_bytes: 120,
            epbs: 32,
            http_status: 200,
            curl_exit: -1,
            curl_signal: 9,
            intentional_kill: true,
            cleanup_success: true,
            stderr_truncated: false,
            request_sha256: "a".repeat(64),
            binary_sha256: "b".repeat(64),
            build_inputs_sha256: "c".repeat(64),
        };
        assert_eq!(Status::parse(s.line().as_bytes()).unwrap(), s);
        assert!(s.accepted());
        for other in [
            Status {
                http_status: 401,
                ..s.clone()
            },
            Status {
                stop: Stop::Deadline,
                ..s.clone()
            },
            Status {
                epbs: 31,
                ..s.clone()
            },
            Status {
                cleanup_success: false,
                ..s.clone()
            },
            Status {
                stderr_truncated: true,
                ..s.clone()
            },
        ] {
            assert!(!other.accepted());
        }
        assert!(Status::parse(format!("{}{}", s.line(), s.line()).as_bytes()).is_err());
        assert!(Status::parse(format!("{}tail", s.line()).as_bytes()).is_err());
        assert!(
            Status::parse(
                s.line()
                    .replace(STATUS, "KISMET_WIFI_STATUS_V1FUTURE")
                    .as_bytes()
            )
            .is_err()
        );
        assert!(
            Status::parse(s.line().replace(STATUS, "KISMET_WIFI_STATUS_V2").as_bytes()).is_err()
        );
    }
}
