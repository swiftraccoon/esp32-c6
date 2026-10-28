//! Private serial collection and read-only replay share the same consumer.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use runtime_protocol::{
    MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, Phase, Request, decode_response, encode_request,
};
use serde::{Deserialize, Serialize};

use crate::runtime::{self, Exchange};
use crate::startup;
use crate::usb;

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RAW_BYTES: usize = 64 * 1024;
const CAPTURE_VERSION: u32 = 2;
const STARTUP_DURATION: Duration = Duration::from_secs(1);
const MAX_STARTUP_BYTES: usize = startup::MAX_STARTUP_BYTES;
const FILES: [&str; 15] = [
    "capture-status.json",
    "cleanup.jsonl",
    "connection.json",
    "exchanges.jsonl",
    "requests.jsonl",
    "rx.raw",
    "startup.raw",
    "summary.json",
    "tx.raw",
    "usb-defaults.json",
    "usb-defaults.status.json",
    "usb-defaults.stderr",
    "usb-overrides.json",
    "usb-overrides.status.json",
    "usb-overrides.stderr",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: u32,
    startup: startup::Report,
    table: runtime::Report,
    usb_host_properties: usb::ComparisonReport,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    success: bool,
    error: Option<String>,
    cleanup_attempted: bool,
    cleanup_success: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfilerReceipt {
    success: bool,
    exit_code: Option<i32>,
    timed_out: bool,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ControlLinePolicy {
    DeassertDtrThenRts,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StartupPolicy {
    C6RomBootOrWhitespace,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Connection {
    schema_version: u32,
    baud: u32,
    control_lines: ControlLinePolicy,
    dtr_deassertion_succeeded: bool,
    rts_deassertion_succeeded: bool,
    startup_policy: StartupPolicy,
    startup_window_ms: u32,
    startup_limit_bytes: usize,
    startup_bytes: usize,
    startup_accepted: bool,
}

impl Connection {
    fn new() -> Self {
        Self {
            schema_version: CAPTURE_VERSION,
            baud: 115_200,
            control_lines: ControlLinePolicy::DeassertDtrThenRts,
            dtr_deassertion_succeeded: false,
            rts_deassertion_succeeded: false,
            startup_policy: StartupPolicy::C6RomBootOrWhitespace,
            startup_window_ms: STARTUP_DURATION.as_millis() as u32,
            startup_limit_bytes: MAX_STARTUP_BYTES,
            startup_bytes: 0,
            startup_accepted: false,
        }
    }
}

fn collect_startup<P: Read>(
    mut port: P,
    directory: &Path,
    duration: Duration,
) -> Result<usize, &'static str> {
    let mut file = private_file(&directory.join("startup.raw"))?;
    let deadline = Instant::now() + duration;
    let mut raw = Vec::new();
    let mut bytes = [0; 256];
    while Instant::now() < deadline {
        match port.read(&mut bytes) {
            Ok(count) if count <= bytes.len() => {
                file.write_all(&bytes[..count])
                    .map_err(|_| "cannot save startup output")?;
                raw.extend_from_slice(&bytes[..count]);
                if raw.len() > MAX_STARTUP_BYTES {
                    return Err("startup output exceeds size limit");
                }
                if count == 0 {
                    thread::yield_now();
                }
            }
            Ok(_) => return Err("startup reader returned an invalid length"),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::Interrupted
                ) =>
            {
                thread::yield_now()
            }
            Err(_) => return Err("cannot read startup output"),
        }
    }
    file.sync_all()
        .map_err(|_| "cannot synchronize startup output")?;
    startup::summarize(&raw)?;
    Ok(raw.len())
}

fn validate_connection(directory: &Path) -> Result<startup::Report, &'static str> {
    let connection: Connection =
        serde_json::from_slice(&crate::read_input(&directory.join("connection.json"))?)
            .map_err(|_| "invalid runtime connection metadata")?;
    let startup = crate::read_input(&directory.join("startup.raw"))?;
    if connection.schema_version != CAPTURE_VERSION
        || connection.baud != 115_200
        || connection.control_lines != ControlLinePolicy::DeassertDtrThenRts
        || !connection.dtr_deassertion_succeeded
        || !connection.rts_deassertion_succeeded
        || connection.startup_policy != StartupPolicy::C6RomBootOrWhitespace
        || connection.startup_window_ms != STARTUP_DURATION.as_millis() as u32
        || connection.startup_limit_bytes != MAX_STARTUP_BYTES
        || !connection.startup_accepted
        || connection.startup_bytes != startup.len()
    {
        return Err("runtime connection or startup policy was not completed");
    }
    startup::summarize(&startup)
}

pub(crate) fn private_file(path: &Path) -> Result<File, &'static str> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|_| "cannot create private capture file")
}

pub(crate) fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<(), &'static str> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|_| "cannot encode capture metadata")?;
    bytes.push(b'\n');
    let mut file = private_file(path)?;
    file.write_all(&bytes)
        .map_err(|_| "cannot save capture metadata")?;
    file.sync_all()
        .map_err(|_| "cannot synchronize capture metadata")
}

pub(crate) fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    let components: Vec<_> = path.components().collect();
    if components.len() < 3
        || components[0] != Component::Normal("private".as_ref())
        || components[1] != Component::Normal("device-identity".as_ref())
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("capture directory must be under workspace private/device-identity");
    }
    let root = Command::new("git")
        .args(["rev-parse", "--show-prefix"])
        .stderr(Stdio::null())
        .output()
        .map_err(|_| "cannot verify capture workspace")?;
    if !root.status.success() || !root.stdout.iter().all(u8::is_ascii_whitespace) {
        return Err("run capture from the workspace root");
    }
    let ignored = Command::new("git")
        .args(["check-ignore", "--quiet", "--"])
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| "cannot verify private capture storage")?;
    if !ignored.success() {
        return Err("capture directory must be ignored by Git");
    }
    let parent = path.parent().ok_or("capture directory needs a parent")?;
    let mut prefix = std::path::PathBuf::new();
    for component in parent.components() {
        prefix.push(component);
        let metadata = fs::symlink_metadata(&prefix).map_err(|_| "capture parent must exist")?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("capture parents must be directories without symlinks");
        }
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(path)
        .map_err(|_| "capture directory must be new")
}

struct Logs {
    requests: File,
    exchanges: File,
    tx: File,
    rx: File,
    cleanup: File,
    received: usize,
}

impl Logs {
    fn new(directory: &Path) -> Result<Self, &'static str> {
        Ok(Self {
            requests: private_file(&directory.join("requests.jsonl"))?,
            exchanges: private_file(&directory.join("exchanges.jsonl"))?,
            tx: private_file(&directory.join("tx.raw"))?,
            rx: private_file(&directory.join("rx.raw"))?,
            cleanup: private_file(&directory.join("cleanup.jsonl"))?,
            received: 0,
        })
    }

    fn receive_byte<P: Read>(&mut self, mut port: P) -> Result<Option<u8>, &'static str> {
        let mut byte = [0];
        match port.read(&mut byte) {
            Ok(0) => Ok(None),
            Ok(1) => {
                self.rx
                    .write_all(&byte)
                    .map_err(|_| "cannot save serial input")?;
                self.received += 1;
                if self.received > MAX_RAW_BYTES {
                    return Err("runtime raw input exceeds size limit");
                }
                Ok(Some(byte[0]))
            }
            Ok(_) => Err("serial reader returned an invalid length"),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::Interrupted
                ) =>
            {
                Ok(None)
            }
            Err(_) => Err("cannot read runtime serial response"),
        }
    }

    fn exchange<P: Read + Write>(
        &mut self,
        mut port: P,
        request: Request,
        timeout: Duration,
    ) -> Result<Exchange, &'static str> {
        let mut bytes = [0; MAX_REQUEST_BYTES];
        let length =
            encode_request(&request, &mut bytes).map_err(|_| "cannot encode runtime request")?;
        self.requests
            .write_all(&bytes[..length])
            .map_err(|_| "cannot save runtime request")?;
        let deadline = Instant::now() + timeout;
        let mut sent = 0;
        while sent < length {
            if Instant::now() >= deadline {
                return Err("runtime serial request timed out");
            }
            match port.write(&bytes[sent..length]) {
                Ok(0) => thread::yield_now(),
                Ok(count) if count <= length - sent => {
                    self.tx
                        .write_all(&bytes[sent..sent + count])
                        .map_err(|_| "cannot save serial output")?;
                    sent += count;
                }
                Ok(_) => return Err("serial writer returned an invalid length"),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::Interrupted
                            | std::io::ErrorKind::WouldBlock
                    ) => {}
                Err(_) => return Err("cannot send runtime request"),
            }
        }
        let mut line = Vec::new();
        while Instant::now() < deadline {
            let Some(byte) = self.receive_byte(&mut port)? else {
                thread::yield_now();
                continue;
            };
            line.push(byte);
            if line.len() > MAX_RESPONSE_BYTES {
                return Err("runtime serial line exceeds size limit");
            }
            if byte == b'\n' {
                if line.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'{') {
                    let response =
                        decode_response(&line).map_err(|_| "invalid runtime response frame")?;
                    response
                        .validate_for(&request)
                        .map_err(|_| "runtime response validation failed")?;
                    return Ok(Exchange { request, response });
                }
                if line.iter().any(|byte| !byte.is_ascii_whitespace()) {
                    return Err("unexpected non-JSON serial output during runtime capture");
                }
                line.clear();
            }
        }
        Err("runtime serial response timed out")
    }

    fn save_exchange(&mut self, exchange: &Exchange, cleanup: bool) -> Result<(), &'static str> {
        let file = if cleanup {
            &mut self.cleanup
        } else {
            &mut self.exchanges
        };
        serde_json::to_writer(&mut *file, exchange).map_err(|_| "cannot save runtime exchange")?;
        file.write_all(b"\n")
            .map_err(|_| "cannot finish runtime exchange")
    }

    fn drain_final<P: Read>(
        &mut self,
        mut port: P,
        duration: Duration,
    ) -> Result<(), &'static str> {
        let deadline = Instant::now() + duration;
        let mut unexpected = false;
        while Instant::now() < deadline {
            if let Some(byte) = self.receive_byte(&mut port)? {
                unexpected |= !byte.is_ascii_whitespace();
            } else {
                thread::yield_now();
            }
        }
        if unexpected {
            return Err("unexpected serial suffix after runtime restoration");
        }
        Ok(())
    }

    fn synchronize(&self) -> Result<(), &'static str> {
        for file in [
            &self.requests,
            &self.exchanges,
            &self.tx,
            &self.rx,
            &self.cleanup,
        ] {
            file.sync_all()
                .map_err(|_| "cannot synchronize raw capture files")?;
        }
        Ok(())
    }
}

fn profiler(directory: &Path, phase: Phase) -> Result<(), &'static str> {
    let stem = match phase {
        Phase::Defaults => "usb-defaults",
        Phase::InterfaceOverride => "usb-overrides",
        _ => return Err("invalid USB observation phase"),
    };
    let stdout = private_file(&directory.join(format!("{stem}.json")))?;
    let stderr = private_file(&directory.join(format!("{stem}.stderr")))?;
    let mut child = Command::new("/usr/sbin/system_profiler")
        .args(["SPUSBHostDataType", "-json", "-timeout", "15"])
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .map_err(|_| "cannot start USB host observer")?;
    let deadline = Instant::now() + Duration::from_secs(20);
    let (status, timed_out) = loop {
        match child
            .try_wait()
            .map_err(|_| "cannot wait for USB host observer")?
        {
            Some(status) => break (status, false),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let status = child.wait().map_err(|_| "cannot stop USB host observer")?;
                break (status, true);
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    let receipt = ProfilerReceipt {
        success: status.success() && !timed_out,
        exit_code: status.code(),
        timed_out,
    };
    save_json(&directory.join(format!("{stem}.status.json")), &receipt)?;
    if !receipt.success {
        return Err("USB host observer failed");
    }
    let bytes = crate::read_input(&directory.join(format!("{stem}.json")))?;
    usb::summarize(&bytes)?;
    Ok(())
}

fn validate_serial_artifacts(directory: &Path, transcript: &[u8]) -> Result<(), &'static str> {
    let exchanges = runtime::decode(transcript)?;
    let mut expected_tx = Vec::new();
    for exchange in &exchanges {
        let mut bytes = [0; MAX_REQUEST_BYTES];
        let length = encode_request(&exchange.request, &mut bytes)
            .map_err(|_| "cannot reconstruct runtime request")?;
        expected_tx.extend_from_slice(&bytes[..length]);
    }
    if crate::read_input(&directory.join("requests.jsonl"))? != expected_tx
        || crate::read_input(&directory.join("tx.raw"))? != expected_tx
    {
        return Err("runtime saved request bytes disagree");
    }
    let raw = crate::read_input(&directory.join("rx.raw"))?;
    if raw.len() > MAX_RAW_BYTES {
        return Err("runtime raw response is oversized or incomplete");
    }
    let mut received = Vec::new();
    for line in raw.split_inclusive(|&byte| byte == b'\n') {
        if line.len() > MAX_RESPONSE_BYTES {
            return Err("runtime raw response line exceeds size limit");
        }
        if !line.ends_with(b"\n") && line.iter().any(|byte| !byte.is_ascii_whitespace()) {
            return Err("runtime raw response is incomplete");
        }
        if line.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'{') {
            received.push(decode_response(line).map_err(|_| "invalid saved runtime response")?);
        } else if line.iter().any(|byte| !byte.is_ascii_whitespace()) {
            return Err("unexpected non-JSON saved runtime output");
        }
    }
    if received != exchanges.map(|exchange| exchange.response) {
        return Err("runtime raw responses disagree with saved exchanges");
    }
    Ok(())
}

fn complete_report(directory: &Path) -> Result<Report, &'static str> {
    let startup = validate_connection(directory)?;
    let transcript = crate::read_input(&directory.join("exchanges.jsonl"))?;
    validate_serial_artifacts(directory, &transcript)?;
    let before = crate::read_input(&directory.join("usb-defaults.json"))?;
    let after = crate::read_input(&directory.join("usb-overrides.json"))?;
    Ok(Report {
        schema_version: CAPTURE_VERSION,
        startup,
        table: runtime::summarize(&transcript)?,
        usb_host_properties: usb::compare(&before, &after)?,
    })
}

fn session<P, F>(
    mut port: P,
    directory: &Path,
    first_id: u32,
    timeout: Duration,
    drain: Duration,
    mut observe_usb: F,
) -> Result<Report, &'static str>
where
    P: Read + Write,
    F: FnMut(&Path, Phase) -> Result<(), &'static str>,
{
    let mut logs = Logs::new(directory)?;
    let mut initial_snapshot = None;
    let result: Result<Report, &'static str> = (|| {
        for (index, phase) in Phase::ALL.into_iter().enumerate() {
            let id = first_id
                .checked_add(index as u32)
                .ok_or("runtime correlation token overflow")?;
            let exchange = logs.exchange(&mut port, Request::new(id, phase), timeout)?;
            if phase == Phase::Defaults {
                initial_snapshot = exchange.response.snapshot;
            }
            logs.save_exchange(&exchange, false)?;
            if matches!(phase, Phase::Defaults | Phase::InterfaceOverride) {
                observe_usb(directory, phase)?;
            }
        }
        logs.drain_final(&mut port, drain)?;
        logs.synchronize()?;
        let report = complete_report(directory)?;
        save_json(&directory.join("summary.json"), &report)?;
        Ok(report)
    })();
    let (cleanup_attempted, cleanup_success) = if result.is_err() {
        let cleanup = first_id
            .checked_add(4)
            .ok_or("runtime correlation token overflow")
            .and_then(|id| logs.exchange(&mut port, Request::new(id, Phase::Restored), timeout));
        let success = match cleanup {
            Ok(exchange) => {
                let saved = logs.save_exchange(&exchange, true).is_ok();
                saved
                    && initial_snapshot.is_some()
                    && exchange.response.snapshot == initial_snapshot
            }
            Err(_) => false,
        };
        (true, success)
    } else {
        (false, false)
    };
    let synchronize = logs.synchronize();
    let receipt = Receipt {
        schema_version: CAPTURE_VERSION,
        success: result.is_ok() && synchronize.is_ok(),
        error: result
            .as_ref()
            .err()
            .copied()
            .or(synchronize.err())
            .map(str::to_owned),
        cleanup_attempted,
        cleanup_success,
    };
    save_json(&directory.join("capture-status.json"), &receipt)?;
    synchronize?;
    result
}

fn correlation_token() -> Result<u32, &'static str> {
    let mut random =
        File::open("/dev/urandom").map_err(|_| "cannot obtain capture correlation token")?;
    for _ in 0..8 {
        let mut bytes = [0; 4];
        random
            .read_exact(&mut bytes)
            .map_err(|_| "cannot read capture correlation token")?;
        let token = u32::from_ne_bytes(bytes);
        if token != 0 && token <= u32::MAX - 4 {
            return Ok(token);
        }
    }
    Err("cannot select capture correlation token")
}

pub(crate) fn capture(port: &str, directory: &Path) -> Result<Report, &'static str> {
    if !cfg!(target_os = "macos") {
        return Err("runtime capture requires the macOS USB observer");
    }
    create_private_directory(directory)?;
    let mut connection = Connection::new();
    let result = (|| {
        let token = correlation_token()?;
        // Opening this CP2102N tty may assert RTS and hold EN low. Release DTR
        // before RTS, checking both operations. These transitions can release
        // reset/start the application; no previous RAM state is presumed.
        let mut serial = serialport::new(port, 115_200)
            .timeout(Duration::from_millis(100))
            .dtr_on_open(false)
            .open()
            .map_err(|_| "cannot open runtime UART bridge")?;
        connection.dtr_deassertion_succeeded = serial.write_data_terminal_ready(false).is_ok();
        connection.rts_deassertion_succeeded = serial.write_request_to_send(false).is_ok();
        if !connection.dtr_deassertion_succeeded || !connection.rts_deassertion_succeeded {
            return Err("cannot release runtime UART control lines");
        }
        connection.startup_bytes = collect_startup(&mut serial, directory, STARTUP_DURATION)?;
        connection.startup_accepted = true;
        save_json(&directory.join("connection.json"), &connection)?;
        session(
            &mut serial,
            directory,
            token,
            RESPONSE_TIMEOUT,
            Duration::from_millis(200),
            profiler,
        )
    })();
    if !directory.join("connection.json").exists() {
        connection.startup_bytes = fs::metadata(directory.join("startup.raw"))
            .ok()
            .and_then(|metadata| usize::try_from(metadata.len()).ok())
            .unwrap_or(0);
        save_json(&directory.join("connection.json"), &connection)?;
    }
    if result.is_err() && !directory.join("capture-status.json").exists() {
        save_json(
            &directory.join("capture-status.json"),
            &Receipt {
                schema_version: CAPTURE_VERSION,
                success: false,
                error: result.as_ref().err().copied().map(str::to_owned),
                cleanup_attempted: false,
                cleanup_success: false,
            },
        )?;
    }
    result
}

pub(crate) fn replay(directory: &Path) -> Result<Report, &'static str> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect runtime capture directory")? {
        let entry = entry.map_err(|_| "cannot inspect runtime capture entry")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid runtime capture filename")?;
        if !entry
            .file_type()
            .map_err(|_| "cannot inspect runtime capture file")?
            .is_file()
        {
            return Err("runtime capture must contain only regular files");
        }
        names.insert(name);
        if names.len() > FILES.len() {
            return Err("runtime capture inventory contains too many entries");
        }
    }
    if names != FILES.into_iter().map(str::to_owned).collect() {
        return Err("runtime capture inventory is incomplete or unexpected");
    }
    let status: Receipt =
        serde_json::from_slice(&crate::read_input(&directory.join("capture-status.json"))?)
            .map_err(|_| "invalid runtime capture receipt")?;
    if status.schema_version != CAPTURE_VERSION
        || !status.success
        || status.error.is_some()
        || status.cleanup_attempted
        || status.cleanup_success
        || !crate::read_input(&directory.join("cleanup.jsonl"))?.is_empty()
    {
        return Err("runtime capture records a failed or incomplete attempt");
    }
    for stem in ["usb-defaults", "usb-overrides"] {
        let receipt: ProfilerReceipt = serde_json::from_slice(&crate::read_input(
            &directory.join(format!("{stem}.status.json")),
        )?)
        .map_err(|_| "invalid USB host observer receipt")?;
        if !receipt.success || receipt.exit_code != Some(0) || receipt.timed_out {
            return Err("USB host observer did not complete successfully");
        }
        // Inspect every retained regular file even when diagnostics are empty;
        // all files share the same bounded, no-symlink loader.
        crate::read_input(&directory.join(format!("{stem}.stderr")))?;
    }
    let report = complete_report(directory)?;
    let mut expected =
        serde_json::to_vec_pretty(&report).map_err(|_| "cannot encode runtime replay summary")?;
    expected.push(b'\n');
    if crate::read_input(&directory.join("summary.json"))? != expected {
        return Err("runtime saved summary disagrees with complete replay");
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use runtime_protocol::{Response, decode_request, encode_response};

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Self {
            let path = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "identity-runtime-control-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct MockPort {
        pending: Vec<u8>,
        incoming: VecDeque<u8>,
        fail_interface: bool,
        omit_base_reply: bool,
        final_suffix: bool,
        panic_before_response: Option<Phase>,
    }

    impl MockPort {
        fn new() -> Self {
            Self {
                pending: Vec::new(),
                incoming: VecDeque::from(b"\r\n".to_vec()),
                fail_interface: false,
                omit_base_reply: false,
                final_suffix: false,
                panic_before_response: None,
            }
        }
    }

    impl Read for MockPort {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            if let Some(byte) = self.incoming.pop_front() {
                bytes[0] = byte;
                Ok(1)
            } else {
                Err(io::ErrorKind::TimedOut.into())
            }
        }
    }

    impl Write for MockPort {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let length = bytes.len().min(7);
            self.pending.extend_from_slice(&bytes[..length]);
            if self.pending.ends_with(b"\n") {
                let request = decode_request(&self.pending).unwrap();
                self.pending.clear();
                if !(self.omit_base_reply && request.phase == Phase::BaseOverride) {
                    if self.panic_before_response == Some(request.phase) {
                        self.incoming.extend(b"Guru Meditation Error\n");
                    }
                    let source = runtime::synthetic_exchanges()
                        .into_iter()
                        .find(|exchange| exchange.request.phase == request.phase)
                        .unwrap();
                    let response =
                        if self.fail_interface && request.phase == Phase::InterfaceOverride {
                            Response::new(request.id, request.phase, -2, None)
                        } else {
                            Response::new(request.id, request.phase, 0, source.response.snapshot)
                        };
                    let mut encoded = [0; MAX_RESPONSE_BYTES];
                    let length = encode_response(&response, &mut encoded).unwrap();
                    self.incoming.extend(&encoded[..length]);
                    if self.final_suffix && request.phase == Phase::Restored {
                        self.incoming.extend(b"synthetic-private-suffix\n");
                    }
                }
            }
            Ok(length)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn synthetic_usb(directory: &Path, phase: Phase) -> Result<(), &'static str> {
        let stem = if phase == Phase::Defaults {
            "usb-defaults"
        } else {
            "usb-overrides"
        };
        save_json(
            &directory.join(format!("{stem}.json")),
            &serde_json::json!({
                "SPUSBHostDataType": [{"_items": [{
                    "USBDeviceKeyVendorID": "0x303a",
                    "USBDeviceKeyProductID": "0x1001",
                    "USBDeviceKeySerialNumber": "synthetic-private-property"
                }, {
                    "USBDeviceKeyVendorID": "0x10c4",
                    "USBDeviceKeyProductID": "0xea60",
                    "USBDeviceKeySerialNumber": "synthetic-private-property"
                }]}]
            }),
        )?;
        private_file(&directory.join(format!("{stem}.stderr")))?;
        save_json(
            &directory.join(format!("{stem}.status.json")),
            &ProfilerReceipt {
                success: true,
                exit_code: Some(0),
                timed_out: false,
            },
        )
    }

    fn collect(temp: &Temp, port: &mut MockPort) -> Result<Report, &'static str> {
        prepare_startup(temp, &mut *port)?;
        session(
            port,
            &temp.0,
            42,
            Duration::from_secs(1),
            Duration::from_millis(50),
            synthetic_usb,
        )
    }

    fn prepare_startup(temp: &Temp, port: &mut MockPort) -> Result<(), &'static str> {
        let mut connection = Connection::new();
        connection.dtr_deassertion_succeeded = true;
        connection.rts_deassertion_succeeded = true;
        connection.startup_bytes = collect_startup(port, &temp.0, Duration::from_millis(100))?;
        connection.startup_accepted = true;
        save_json(&temp.0.join("connection.json"), &connection)
    }

    #[test]
    fn fragmented_live_save_and_complete_replay_agree_without_values() {
        let temp = Temp::new();
        let live = collect(&temp, &mut MockPort::new()).unwrap();
        assert_eq!(live, replay(&temp.0).unwrap());
        assert_eq!(live, replay(&temp.0).unwrap());
        let encoded = serde_json::to_string(&live).unwrap();
        assert!(!encoded.contains("synthetic-private"));
        assert_eq!(fs::read(temp.0.join("startup.raw")).unwrap(), b"\r\n");
        assert!(fs::read(temp.0.join("rx.raw")).unwrap().starts_with(b"{"));
        assert_eq!(live.schema_version, CAPTURE_VERSION);
        assert_eq!(
            fs::read(temp.0.join("tx.raw")).unwrap(),
            fs::read(temp.0.join("requests.jsonl")).unwrap()
        );
    }

    #[test]
    fn device_failures_timeouts_and_usb_failure_remain_failed_after_cleanup() {
        for mode in 0..3 {
            let temp = Temp::new();
            let mut port = MockPort::new();
            port.fail_interface = mode == 0;
            port.omit_base_reply = mode == 1;
            prepare_startup(&temp, &mut port).unwrap();
            let result = session(
                &mut port,
                &temp.0,
                42,
                Duration::from_secs(1),
                Duration::ZERO,
                |directory, phase| {
                    if mode == 2 && phase == Phase::InterfaceOverride {
                        Err("synthetic USB observer failure")
                    } else {
                        synthetic_usb(directory, phase)
                    }
                },
            );
            assert!(result.is_err());
            let receipt: Receipt =
                serde_json::from_slice(&fs::read(temp.0.join("capture-status.json")).unwrap())
                    .unwrap();
            assert!(!receipt.success);
            assert!(receipt.cleanup_attempted);
            assert!(receipt.cleanup_success, "mode {mode}");
            assert!(
                runtime::summarize(&fs::read(temp.0.join("exchanges.jsonl")).unwrap()).is_err()
            );
            assert!(replay(&temp.0).is_err());
        }
    }

    #[test]
    fn final_suffix_is_retained_and_rejected() {
        let temp = Temp::new();
        let mut port = MockPort::new();
        port.final_suffix = true;
        assert!(collect(&temp, &mut port).is_err());
        let raw = fs::read(temp.0.join("rx.raw")).unwrap();
        assert!(
            raw.windows(b"synthetic-private-suffix".len())
                .any(|window| window == b"synthetic-private-suffix")
        );
        assert!(replay(&temp.0).is_err());
    }

    #[test]
    fn panic_or_other_non_json_text_cannot_be_dropped_before_or_between_replies() {
        for phase in [
            Phase::Defaults,
            Phase::BaseOverride,
            Phase::InterfaceOverride,
        ] {
            let temp = Temp::new();
            let mut port = MockPort::new();
            port.panic_before_response = Some(phase);
            assert!(collect(&temp, &mut port).is_err());
            assert!(replay(&temp.0).is_err());
        }
        let temp = Temp::new();
        collect(&temp, &mut MockPort::new()).unwrap();
        let path = temp.0.join("rx.raw");
        let mut raw = b"Guru Meditation Error\n".to_vec();
        raw.extend(fs::read(&path).unwrap());
        fs::write(path, raw).unwrap();
        assert!(replay(&temp.0).is_err());
    }

    #[test]
    fn receipts_raw_bytes_inventory_and_saved_summary_are_all_checked() {
        for mutation in 0..10 {
            let temp = Temp::new();
            collect(&temp, &mut MockPort::new()).unwrap();
            match mutation {
                0 => {
                    let path = temp.0.join("capture-status.json");
                    let mut receipt: Receipt =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    receipt.success = false;
                    fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
                }
                1 => fs::write(
                    temp.0.join("usb-overrides.status.json"),
                    b"{\"success\":false,\"exit_code\":1,\"timed_out\":false}",
                )
                .unwrap(),
                2 => fs::write(temp.0.join("tx.raw"), b"corrupted").unwrap(),
                3 => {
                    let path = temp.0.join("rx.raw");
                    let mut raw = fs::read(&path).unwrap();
                    raw.pop();
                    fs::write(path, raw).unwrap();
                }
                4 => fs::write(temp.0.join("summary.json"), b"{}").unwrap(),
                5 => fs::write(temp.0.join("unexpected"), b"synthetic").unwrap(),
                6 => {
                    let path = temp.0.join("rx.raw");
                    let mut raw = fs::read(&path).unwrap();
                    raw.extend(vec![b' '; 4096]);
                    fs::write(path, raw).unwrap();
                    // The actual live admission path uses this same validator.
                    assert!(complete_report(&temp.0).is_err());
                }
                7 | 8 => {
                    let path = temp.0.join("connection.json");
                    let mut connection: Connection =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    if mutation == 7 {
                        connection.dtr_deassertion_succeeded = false;
                    } else {
                        connection.startup_bytes += 1;
                    }
                    fs::write(path, serde_json::to_vec(&connection).unwrap()).unwrap();
                    assert!(complete_report(&temp.0).is_err());
                }
                _ => {
                    let path = temp.0.join("capture-status.json");
                    let mut receipt: Receipt =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    receipt.schema_version = 1;
                    fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
                }
            }
            assert!(replay(&temp.0).is_err(), "mutation {mutation}");
        }
    }

    #[test]
    fn recognized_startup_is_saved_separately_before_the_runtime_sequence() {
        let temp = Temp::new();
        let mut port = MockPort::new();
        port.incoming = VecDeque::from(b"ESP-ROM:esp32c6-20220919\r\nBuild:Sep 19 2022\r\nrst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)\r\nSPIWP:0xee\r\nmode:DIO, clock div:2\r\nload:0x40800000,len:0x10\r\nentry 0x40800000\r\n".to_vec());
        let report = collect(&temp, &mut port).unwrap();
        assert_eq!(report, replay(&temp.0).unwrap());
        assert!(
            fs::read(temp.0.join("startup.raw"))
                .unwrap()
                .starts_with(b"ESP-ROM:")
        );
        assert!(fs::read(temp.0.join("rx.raw")).unwrap().starts_with(b"{"));
    }

    #[test]
    fn startup_panics_or_unrecognized_text_are_retained_without_sending_a_request() {
        for startup in [
            b"Guru Meditation Error\n".as_slice(),
            b"entry panic\n",
            b"synthetic arbitrary startup\n",
        ] {
            let temp = Temp::new();
            let mut port = MockPort::new();
            port.incoming = VecDeque::from(startup.to_vec());
            assert!(collect(&temp, &mut port).is_err());
            assert_eq!(fs::read(temp.0.join("startup.raw")).unwrap(), startup);
            assert!(port.pending.is_empty());
            assert!(!temp.0.join("requests.jsonl").exists());
            assert!(replay(&temp.0).is_err());
        }
    }
}
