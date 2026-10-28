//! Controlled AP beacon-presence witness, with receiver readiness before start.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use kismet_worker::{BYTE_LIMIT, Ready, STDERR_LIMIT};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wifi_protocol::{
    AP_MAC, AP_WINDOW_MILLIS, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, Operation, Request, Response,
    State, decode_response, encode_request, ssid,
};

use crate::capture::{create_private_directory, private_file, save_json};
use crate::{startup, wifi_capture};

const VERSION: u32 = 1;
const UART_LIMIT: usize = 64 * 1024;
const READY_TIMEOUT: Duration = Duration::from_secs(30);
const UART_TIMEOUT: Duration = Duration::from_secs(5);
const ROOT_FILES: [&str; 12] = [
    "trial.json",
    "connection.json",
    "startup.raw",
    "tx.raw",
    "rx.raw",
    "requests.jsonl",
    "exchanges.jsonl",
    "cleanup.jsonl",
    "events.jsonl",
    "receiver-ready.json",
    "summary.json",
    "capture-status.json",
];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|_| "cannot encode AP trial metadata")?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Trial {
    schema_version: u32,
    nonce: [u8; 8],
    first_id: u32,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Connection {
    schema_version: u32,
    baud: u32,
    dtr_deassertion_succeeded: bool,
    rts_deassertion_succeeded: bool,
    startup_accepted: bool,
    cleanup_stop_attempted: bool,
    cleanup_stop_confirmed: bool,
    recovery_required: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EventName {
    Status,
    ReceiverReady,
    ReceiverAlive,
    StartRequested,
    Start,
    Stop,
    CleanupStop,
    ReceiverComplete,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Event {
    sequence: u32,
    elapsed_us: u64,
    event: EventName,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadyReceipt {
    pub(super) line: String,
    pub(super) local_prefix_bytes: usize,
    pub(super) local_prefix_sha256: String,
}

/// Value-free result; raw nonce, UART MACs and PCAPNG remain private.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: u32,
    receiver: wifi_observer::Summary,
    witness: wifi_observer::WitnessSummary,
    uart_commands: u32,
    stop_confirmed: bool,
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

struct Gate {
    expected_request: String,
    expected_binary: String,
    expected_build: String,
    prefix: Vec<u8>,
    prefix_length: Option<usize>,
    stderr_line: Vec<u8>,
    ready: Option<Ready>,
    announced: bool,
    failed: bool,
    notify: mpsc::Sender<Result<ReadyReceipt, &'static str>>,
    raw_sync: File,
}

impl Gate {
    fn fail(&mut self) {
        if !self.failed {
            self.failed = true;
            let _ = self.notify.send(Err("receiver readiness did not validate"));
        }
    }
    fn body(&mut self, bytes: &[u8]) {
        if self.failed || self.announced {
            return;
        }
        let needed = self
            .prefix_length
            .unwrap_or(12)
            .saturating_sub(self.prefix.len());
        self.prefix
            .extend_from_slice(&bytes[..bytes.len().min(needed)]);
        if self.prefix_length.is_none() && self.prefix.len() >= 12 {
            let len = match self.prefix[8..12] {
                [0x4d, 0x3c, 0x2b, 0x1a] => {
                    u32::from_le_bytes(self.prefix[4..8].try_into().expect("fixed header"))
                }
                [0x1a, 0x2b, 0x3c, 0x4d] => {
                    u32::from_be_bytes(self.prefix[4..8].try_into().expect("fixed header"))
                }
                _ => {
                    self.fail();
                    return;
                }
            } as usize;
            if !(28..=BYTE_LIMIT).contains(&len) || !len.is_multiple_of(4) {
                self.fail();
                return;
            }
            self.prefix_length = Some(len);
        }
        if let Some(length) = self.prefix_length {
            let consumed = needed.min(bytes.len());
            let remaining = &bytes[consumed..];
            let retain = remaining.len().min(length - self.prefix.len());
            self.prefix.extend_from_slice(&remaining[..retain]);
        }
        self.check();
    }
    fn stderr(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.stderr_line.push(*byte);
            if self.stderr_line.len() > STDERR_LIMIT + 4096 {
                self.fail();
                return;
            }
            if *byte == b'\n' {
                if self.stderr_line.starts_with(b"KISMET_WIFI_READY") {
                    if self.ready.is_some() {
                        self.fail();
                        return;
                    }
                    match Ready::parse_line(&self.stderr_line) {
                        Ok(ready) => self.ready = Some(ready),
                        Err(_) => {
                            self.fail();
                            return;
                        }
                    }
                }
                self.stderr_line.clear();
            }
        }
        self.check();
    }
    fn check(&mut self) {
        if self.announced || self.failed {
            return;
        }
        let Some(ready) = self.ready.as_ref() else {
            return;
        };
        let Some(length) = self.prefix_length else {
            return;
        };
        if self.prefix.len() != length {
            return;
        }
        if ready.prefix_bytes != length
            || ready.prefix_sha256 != digest(&self.prefix)
            || ready.request_sha256 != self.expected_request
            || ready.binary_sha256 != self.expected_binary
            || ready.build_inputs_sha256 != self.expected_build
            || wifi_observer::validate_section_prefix(&self.prefix).is_err()
            || self.raw_sync.sync_all().is_err()
        {
            self.fail();
            return;
        }
        self.announced = true;
        let _ = self.notify.send(Ok(ReadyReceipt {
            line: ready.line(),
            local_prefix_bytes: length,
            local_prefix_sha256: digest(&self.prefix),
        }));
    }
}

#[derive(Default)]
struct Control {
    child: Mutex<Option<Child>>,
    cancel: AtomicBool,
}
impl Control {
    fn alive(&self) -> bool {
        // A detected readiness failure invalidates the receiver immediately,
        // before its still-running SSH child has finished shutting down.
        if self.cancel.load(Ordering::Acquire) {
            return false;
        }
        let alive = self
            .child
            .lock()
            .ok()
            .and_then(|mut lock| {
                lock.as_mut()
                    .map(|child| matches!(child.try_wait(), Ok(None)))
            })
            .unwrap_or(false);
        alive && !self.cancel.load(Ordering::Acquire)
    }
    fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Ok(mut child) = self.child.lock()
            && let Some(child) = child.as_mut()
        {
            let _ = child.kill();
        }
    }
}

fn pump(
    mut reader: impl Read,
    writer: &mut File,
    cap: usize,
    gate: &Mutex<Gate>,
    body: bool,
    cancel: &AtomicBool,
) -> (bool, bool) {
    let mut total = 0;
    let mut bytes = [0; 8192];
    loop {
        match reader.read(&mut bytes) {
            Ok(0) => return (false, false),
            Ok(n) => {
                let keep = n.min(cap - total);
                if writer.write_all(&bytes[..keep]).is_err() {
                    cancel.store(true, Ordering::Release);
                    return (true, false);
                }
                total += keep;
                if let Ok(mut gate) = gate.lock() {
                    if body {
                        gate.body(&bytes[..keep]);
                    } else {
                        gate.stderr(&bytes[..keep]);
                    }
                    if gate.failed {
                        cancel.store(true, Ordering::Release);
                    }
                } else {
                    cancel.store(true, Ordering::Release);
                    return (true, false);
                }
                if keep < n {
                    cancel.store(true, Ordering::Release);
                    return (false, true);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                cancel.store(true, Ordering::Release);
                return (true, false);
            }
        }
    }
}

fn receiver_transport(
    profile: &wifi_capture::Profile,
    request: &[u8],
    out: &mut File,
    err: &mut File,
    gate: &Mutex<Gate>,
    control: &Control,
) -> Result<wifi_capture::Transport, &'static str> {
    let mut child = wifi_capture::ssh_command(profile)
        .arg("--ready")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "cannot start AP witness receiver")?;
    let sent = child
        .stdin
        .take()
        .is_some_and(|mut stdin| stdin.write_all(request).is_ok());
    if !sent {
        let _ = child.kill();
        let _ = child.wait();
        return Err("cannot send AP witness receiver request");
    }
    let stdout = child
        .stdout
        .take()
        .ok_or("missing AP receiver packet pipe")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("missing AP receiver status pipe")?;
    *control
        .child
        .lock()
        .map_err(|_| "cannot own AP receiver process")? = Some(child);
    let mut result = wifi_capture::Transport {
        schema_version: 1,
        ssh_exit_code: None,
        ssh_signal: None,
        timed_out: false,
        io_failed: false,
        limit_exceeded: false,
        worker_status_line: None,
    };
    thread::scope(|scope| {
        let body = scope.spawn(|| pump(stdout, out, BYTE_LIMIT, gate, true, &control.cancel));
        let status = scope.spawn(|| {
            pump(
                stderr,
                err,
                STDERR_LIMIT + 4096,
                gate,
                false,
                &control.cancel,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(132);
        loop {
            let exited = match control.child.lock() {
                Ok(mut lock) => match lock.as_mut().map(Child::try_wait) {
                    Some(Ok(Some(_))) => true,
                    Some(Ok(None)) => false,
                    _ => {
                        result.io_failed = true;
                        true
                    }
                },
                Err(_) => {
                    result.io_failed = true;
                    true
                }
            };
            if exited {
                break;
            }
            if control.cancel.load(Ordering::Acquire) {
                control.cancel();
                break;
            }
            if Instant::now() >= deadline {
                result.timed_out = true;
                control.cancel();
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        if let Ok(mut lock) = control.child.lock() {
            if let Some(child) = lock.as_mut() {
                match child.wait() {
                    Ok(exit) => {
                        result.ssh_exit_code = exit.code();
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            result.ssh_signal = exit.signal();
                        }
                    }
                    Err(_) => result.io_failed = true,
                }
            }
        } else {
            result.io_failed = true;
        }
        for joined in [body.join(), status.join()] {
            match joined {
                Ok((failed, limit)) => {
                    result.io_failed |= failed;
                    result.limit_exceeded |= limit;
                }
                Err(_) => result.io_failed = true,
            }
        }
    });
    Ok(result)
}

pub(super) struct Receiver {
    control: Arc<Control>,
    ready: mpsc::Receiver<Result<ReadyReceipt, &'static str>>,
    finished: mpsc::Receiver<Result<wifi_observer::Summary, &'static str>>,
    outcome: Option<Result<wifi_observer::Summary, &'static str>>,
    join: Option<thread::JoinHandle<()>>,
}
impl Receiver {
    pub(super) fn start(
        profile: wifi_capture::Profile,
        directory: PathBuf,
    ) -> Result<Self, &'static str> {
        let encoded = profile
            .request()?
            .encode()
            .map_err(|_| "invalid AP receiver request")?;
        let (notify, ready) = mpsc::channel();
        let (finish, finished) = mpsc::channel();
        let control = Arc::new(Control::default());
        let owned = control.clone();
        let join = thread::spawn(move || {
            let result = wifi_capture::capture_using(&profile, &directory, |request, out, err| {
                let gate = Mutex::new(Gate {
                    expected_request: digest(&encoded),
                    expected_binary: profile.provenance.binary_sha256.clone(),
                    expected_build: profile.provenance.build_inputs_sha256.clone(),
                    prefix: Vec::new(),
                    prefix_length: None,
                    stderr_line: Vec::new(),
                    ready: None,
                    announced: false,
                    failed: false,
                    notify,
                    raw_sync: out
                        .try_clone()
                        .map_err(|_| "cannot synchronize AP receiver prefix")?,
                });
                receiver_transport(&profile, request, out, err, &gate, &owned)
            });
            let _ = finish.send(result);
        });
        Ok(Self {
            control,
            ready,
            finished,
            outcome: None,
            join: Some(join),
        })
    }
    pub(super) fn poll_ready(&mut self) -> Result<Option<ReadyReceipt>, &'static str> {
        match self.ready.try_recv() {
            Ok(result) => result.map(Some),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err("AP receiver readiness ended"),
        }
    }
    fn failed(&mut self) -> bool {
        if self.outcome.is_none() {
            match self.finished.try_recv() {
                Ok(result) => self.outcome = Some(result),
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.outcome = Some(Err("AP receiver controller ended without status"))
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        self.outcome.as_ref().is_some_and(Result::is_err)
    }
    fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str> {
        let result = match self.outcome.take() {
            Some(result) => result,
            None => self
                .finished
                .recv_timeout(Duration::from_secs(140))
                .map_err(|_| "AP receiver did not finish")?,
        };
        if let Some(join) = self.join.take() {
            join.join().map_err(|_| "AP receiver controller failed")?;
        }
        result
    }
}
impl Drop for Receiver {
    fn drop(&mut self) {
        if self.join.is_some() {
            self.control.cancel();
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }
}

struct Logs {
    tx: File,
    rx: File,
    requests: File,
    exchanges: File,
    cleanup: File,
    events: File,
    started: Instant,
    sequence: u32,
    received: usize,
    original: Option<[u8; 6]>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Exchange {
    request: Request,
    response: Response,
}
impl Logs {
    fn new(directory: &Path) -> Result<Self, &'static str> {
        Ok(Self {
            tx: private_file(&directory.join("tx.raw"))?,
            rx: private_file(&directory.join("rx.raw"))?,
            requests: private_file(&directory.join("requests.jsonl"))?,
            exchanges: private_file(&directory.join("exchanges.jsonl"))?,
            cleanup: private_file(&directory.join("cleanup.jsonl"))?,
            events: private_file(&directory.join("events.jsonl"))?,
            started: Instant::now(),
            sequence: 0,
            received: 0,
            original: None,
        })
    }
    fn event(&mut self, event: EventName) -> Result<(), &'static str> {
        self.sequence += 1;
        let event = Event {
            sequence: self.sequence,
            elapsed_us: u64::try_from(self.started.elapsed().as_micros())
                .map_err(|_| "AP trial clock overflow")?,
            event,
        };
        serde_json::to_writer(&mut self.events, &event).map_err(|_| "cannot save AP event")?;
        self.events
            .write_all(b"\n")
            .map_err(|_| "cannot save AP event")
    }
    fn byte(&mut self, port: &mut impl Read) -> Result<Option<u8>, &'static str> {
        let mut byte = [0];
        match port.read(&mut byte) {
            Ok(0) => Ok(None),
            Ok(1) => {
                self.rx
                    .write_all(&byte)
                    .map_err(|_| "cannot save AP UART input")?;
                self.received += 1;
                if self.received > UART_LIMIT {
                    return Err("AP UART input exceeds limit");
                }
                Ok(Some(byte[0]))
            }
            Ok(_) => Err("invalid AP UART read length"),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::Interrupted
                ) =>
            {
                Ok(None)
            }
            Err(_) => Err("cannot read AP UART"),
        }
    }
    fn exchange(
        &mut self,
        port: &mut (impl Read + Write),
        request: Request,
        cleanup: bool,
    ) -> Result<Response, &'static str> {
        let mut bytes = [0; MAX_REQUEST_BYTES];
        let length =
            encode_request(&request, &mut bytes).map_err(|_| "cannot encode AP UART request")?;
        self.requests
            .write_all(&bytes[..length])
            .map_err(|_| "cannot save AP UART request")?;
        let deadline = Instant::now() + UART_TIMEOUT;
        let mut sent = 0;
        while sent < length {
            if Instant::now() >= deadline {
                return Err("AP UART write timed out");
            }
            match port.write(&bytes[sent..length]) {
                Ok(0) => thread::yield_now(),
                Ok(n) if n <= length - sent => {
                    self.tx
                        .write_all(&bytes[sent..sent + n])
                        .map_err(|_| "cannot save AP UART output")?;
                    sent += n;
                }
                Ok(_) => return Err("invalid AP UART write length"),
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::WouldBlock
                            | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err("cannot write AP UART"),
            }
        }
        let mut line = Vec::new();
        while Instant::now() < deadline {
            if let Some(byte) = self.byte(port)? {
                line.push(byte);
                if line.len() > MAX_RESPONSE_BYTES {
                    return Err("AP UART line exceeds limit");
                }
                if byte == b'\n' {
                    if line.iter().all(u8::is_ascii_whitespace) {
                        line.clear();
                        continue;
                    }
                    let response =
                        decode_response(&line).map_err(|_| "invalid AP UART response")?;
                    let exchange = Exchange { request, response };
                    let target = if cleanup {
                        &mut self.cleanup
                    } else {
                        &mut self.exchanges
                    };
                    serde_json::to_writer(&mut *target, &exchange)
                        .map_err(|_| "cannot save AP UART exchange")?;
                    target
                        .write_all(b"\n")
                        .map_err(|_| "cannot save AP UART exchange")?;
                    response
                        .validate_for(&request)
                        .map_err(|_| "AP operation did not validate")?;
                    return Ok(response);
                }
            }
        }
        Err("AP UART response timed out")
    }
    fn sync(&self) -> Result<(), &'static str> {
        for file in [
            &self.tx,
            &self.rx,
            &self.requests,
            &self.exchanges,
            &self.cleanup,
            &self.events,
        ] {
            file.sync_all()
                .map_err(|_| "cannot synchronize AP UART logs")?;
        }
        Ok(())
    }
}

fn initial(response: &Response) -> Result<(), &'static str> {
    let s = response
        .snapshot
        .as_ref()
        .ok_or("missing initial AP snapshot")?;
    if s.state != State::Idle
        || s.initialized
        || s.running
        || s.mac.is_some()
        || s.original_mac.is_some()
        || s.nonce.is_some()
    {
        return Err("AP trial did not start idle");
    }
    Ok(())
}
fn active(response: &Response, nonce: [u8; 8]) -> Result<[u8; 6], &'static str> {
    let s = response
        .snapshot
        .as_ref()
        .ok_or("missing active AP snapshot")?;
    let original = s.original_mac.ok_or("missing AP original address")?;
    if s.state != State::Active
        || !s.initialized
        || !s.running
        || s.mac != Some(AP_MAC)
        || s.nonce != Some(nonce)
        || original == AP_MAC
    {
        return Err("AP start state did not validate");
    }
    Ok(original)
}
fn stopped(response: &Response, nonce: [u8; 8], original: [u8; 6]) -> Result<(), &'static str> {
    let s = response
        .snapshot
        .as_ref()
        .ok_or("missing stopped AP snapshot")?;
    if s.state != State::Stopped
        || !s.initialized
        || s.running
        || s.nonce != Some(nonce)
        || s.mac != Some(original)
        || s.original_mac != Some(original)
    {
        return Err("AP stop state did not validate");
    }
    Ok(())
}
fn cleanup_stopped(response: &Response, nonce: [u8; 8]) -> bool {
    response.snapshot.as_ref().is_some_and(|s| {
        !s.running
            && match s.state {
                State::Idle => !s.initialized && s.nonce.is_none(),
                State::Stopped => {
                    s.initialized
                        && s.nonce == Some(nonce)
                        && s.mac.is_some()
                        && s.mac == s.original_mac
                }
                State::Active => false,
            }
    })
}

pub(super) fn startup_capture(port: &mut impl Read, file: &mut File) -> Result<(), &'static str> {
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut raw = Vec::new();
    let mut bytes = [0; 256];
    while Instant::now() < deadline {
        match port.read(&mut bytes) {
            Ok(n) if n <= bytes.len() => {
                file.write_all(&bytes[..n])
                    .map_err(|_| "cannot save AP startup input")?;
                raw.extend_from_slice(&bytes[..n]);
                if raw.len() > UART_LIMIT {
                    return Err("AP startup input exceeds limit");
                }
            }
            Ok(_) => return Err("invalid AP startup read length"),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return Err("cannot read AP startup input"),
        }
    }
    file.sync_all()
        .map_err(|_| "cannot synchronize AP startup input")?;
    startup::summarize(&raw)?;
    Ok(())
}

fn fresh_trial() -> Result<Trial, &'static str> {
    let mut random = File::open("/dev/urandom").map_err(|_| "cannot obtain AP trial randomness")?;
    for _ in 0..8 {
        let mut nonce = [0; 8];
        let mut id = [0; 4];
        random
            .read_exact(&mut nonce)
            .and_then(|()| random.read_exact(&mut id))
            .map_err(|_| "cannot read AP trial randomness")?;
        let first_id = u32::from_ne_bytes(id);
        if nonce != [0; 8] && first_id != 0 && first_id <= u32::MAX - 4 {
            return Ok(Trial {
                schema_version: VERSION,
                nonce,
                first_id,
            });
        }
    }
    Err("cannot choose AP trial correlation")
}

fn artifacts(directory: &Path) -> Result<Vec<Artifact>, &'static str> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect AP trial artifacts")? {
        let entry = entry.map_err(|_| "cannot inspect AP trial artifact")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid AP trial artifact name")?;
        if name == "capture-status.json" {
            continue;
        }
        let kind = entry
            .file_type()
            .map_err(|_| "cannot inspect AP trial artifact")?;
        if kind.is_file() {
            paths.push(name);
        } else if kind.is_dir() && name == "receiver" {
            for nested in
                fs::read_dir(entry.path()).map_err(|_| "cannot inspect AP receiver artifacts")?
            {
                let nested = nested.map_err(|_| "cannot inspect AP receiver artifact")?;
                if !nested
                    .file_type()
                    .map_err(|_| "cannot inspect AP receiver artifact")?
                    .is_file()
                {
                    return Err("AP receiver artifacts must be regular files");
                }
                let name = nested
                    .file_name()
                    .into_string()
                    .map_err(|_| "invalid AP receiver artifact name")?;
                paths.push(format!("receiver/{name}"));
            }
        } else {
            return Err("unexpected AP trial artifact");
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|file| {
            Ok(Artifact {
                sha256: digest(&crate::read_input(&directory.join(&file))?),
                file,
            })
        })
        .collect()
}

pub(super) trait TrialReceiver {
    fn ready(&mut self) -> Result<ReadyReceipt, &'static str>;
    fn alive(&self) -> bool;
    fn failed(&mut self) -> bool;
    fn completed(&mut self) -> bool;
    fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str>;
}
impl TrialReceiver for Receiver {
    fn ready(&mut self) -> Result<ReadyReceipt, &'static str> {
        self.ready
            .recv_timeout(READY_TIMEOUT)
            .map_err(|_| "AP receiver readiness timed out")?
    }
    fn alive(&self) -> bool {
        self.control.alive()
    }
    fn failed(&mut self) -> bool {
        Receiver::failed(self)
    }
    fn completed(&mut self) -> bool {
        Receiver::failed(self);
        self.outcome.is_some()
    }
    fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str> {
        Receiver::finish(self)
    }
}
fn run_phases<P, R, F>(
    port: &mut P,
    directory: &Path,
    trial: &Trial,
    logs: &mut Logs,
    receiver: &mut Option<R>,
    start_receiver: F,
    window: Duration,
) -> Result<(), &'static str>
where
    P: Read + Write,
    R: TrialReceiver,
    F: FnOnce() -> Result<R, &'static str>,
{
    let status = logs.exchange(
        port,
        Request::new(trial.first_id, Operation::Status, trial.nonce),
        false,
    )?;
    initial(&status)?;
    logs.event(EventName::Status)?;
    *receiver = Some(start_receiver()?);
    let current = receiver.as_mut().ok_or("missing AP receiver")?;
    let ready = current.ready()?;
    save_json(&directory.join("receiver-ready.json"), &ready)?;
    logs.event(EventName::ReceiverReady)?;
    if current.failed() || !current.alive() {
        return Err("AP receiver ended before transmission");
    }
    logs.event(EventName::ReceiverAlive)?;
    logs.event(EventName::StartRequested)?;
    if current.failed() || !current.alive() {
        return Err("AP receiver failed before start request");
    }
    let start = logs.exchange(
        port,
        Request::new(trial.first_id + 1, Operation::Start, trial.nonce),
        false,
    )?;
    let original = active(&start, trial.nonce)?;
    logs.original = Some(original);
    logs.event(EventName::Start)?;
    let until = Instant::now() + window;
    while Instant::now() < until {
        if current.failed() {
            return Err("AP receiver failed during transmission");
        }
        if let Some(byte) = logs.byte(port)?
            && !byte.is_ascii_whitespace()
        {
            return Err("unexpected AP UART output during transmission");
        }
    }
    let stop = logs.exchange(
        port,
        Request::new(trial.first_id + 2, Operation::Stop, trial.nonce),
        false,
    )?;
    stopped(&stop, trial.nonce, original)?;
    logs.event(EventName::Stop)?;
    let receive_deadline = Instant::now() + Duration::from_secs(140);
    while !current.completed() {
        if Instant::now() >= receive_deadline {
            return Err("AP receiver completion timed out");
        }
        if let Some(byte) = logs.byte(port)?
            && !byte.is_ascii_whitespace()
        {
            return Err("unexpected AP UART suffix");
        }
    }
    current.finish()?;
    let drain_until = Instant::now() + Duration::from_millis(200);
    while Instant::now() < drain_until {
        if let Some(byte) = logs.byte(port)?
            && !byte.is_ascii_whitespace()
        {
            return Err("unexpected AP UART suffix");
        }
    }
    logs.event(EventName::ReceiverComplete)?;
    Ok(())
}

pub(crate) fn capture(
    port_file: &Path,
    profile: &Path,
    directory: &Path,
) -> Result<Report, &'static str> {
    let port = crate::read_input(port_file)?;
    let port = std::str::from_utf8(&port)
        .map_err(|_| "invalid private AP bridge port")?
        .trim();
    if !port.starts_with("/dev/cu.")
        || port.len() > 256
        || !port
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    {
        return Err("invalid private AP bridge port");
    }
    let profile = wifi_capture::load_profile(profile)?;
    create_private_directory(directory)?;
    let trial = fresh_trial()?;
    save_json(&directory.join("trial.json"), &trial)?;
    let mut logs = Logs::new(directory)?;
    let mut startup_file = private_file(&directory.join("startup.raw"))?;
    let mut connection = Connection {
        schema_version: VERSION,
        baud: 115200,
        ..Connection::default()
    };
    let mut serial: Option<Box<dyn serialport::SerialPort>> = None;
    let mut receiver: Option<Receiver> = None;
    let result = (|| {
        // Opening this CP2102N callout can assert RTS and affect reset/boot.
        // Release DTR before RTS as in the qualified runtime transport. The
        // separate experiment controller owns full-flash recovery afterward.
        let mut opened = serialport::new(port, 115200)
            .timeout(Duration::from_millis(100))
            .dtr_on_open(false)
            .open()
            .map_err(|_| "cannot open AP trial UART")?;
        connection.dtr_deassertion_succeeded = opened.write_data_terminal_ready(false).is_ok();
        connection.rts_deassertion_succeeded = opened.write_request_to_send(false).is_ok();
        serial = Some(opened);
        if !connection.dtr_deassertion_succeeded || !connection.rts_deassertion_succeeded {
            return Err("cannot release AP UART control lines");
        }
        let port = serial.as_mut().ok_or("missing AP UART handle")?;
        startup_capture(port, &mut startup_file)?;
        connection.startup_accepted = true;
        run_phases(
            port,
            directory,
            &trial,
            &mut logs,
            &mut receiver,
            || Receiver::start(profile, directory.join("receiver")),
            Duration::from_millis(u64::from(AP_WINDOW_MILLIS)),
        )?;
        Ok(())
    })();
    if result.is_err() {
        if let Some(port) = serial.as_mut() {
            connection.cleanup_stop_attempted = true;
            connection.cleanup_stop_confirmed = logs
                .exchange(
                    port,
                    Request::new(trial.first_id + 3, Operation::Stop, trial.nonce),
                    true,
                )
                .is_ok_and(|response| match logs.original {
                    Some(original) => stopped(&response, trial.nonce, original).is_ok(),
                    None => cleanup_stopped(&response, trial.nonce),
                });
            let _ = logs.event(EventName::CleanupStop);
        }
        connection.recovery_required = true;
        if let Some(receiver) = receiver.as_ref() {
            receiver.control.cancel();
        }
    }
    drop(receiver);
    finish_capture(directory, &logs, &connection, result)
}

fn finish_capture(
    directory: &Path,
    logs: &Logs,
    connection: &Connection,
    result: Result<(), &'static str>,
) -> Result<Report, &'static str> {
    logs.sync()?;
    save_json(&directory.join("connection.json"), &connection)?;
    let result = result.and_then(|()| derive_report(directory));
    let result = match result {
        Ok(report) => {
            save_json(&directory.join("summary.json"), &report)?;
            Ok(report)
        }
        Err(error) => Err(error),
    };
    let receipt = Receipt {
        schema_version: VERSION,
        success: result.is_ok(),
        error: result.as_ref().err().map(|error| (*error).into()),
        artifacts: artifacts(directory)?,
    };
    save_json(&directory.join("capture-status.json"), &receipt)?;
    result
}

fn lines<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<Vec<T>, &'static str> {
    let mut records = Vec::new();
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        if !line.ends_with(b"\n") || line.len() > 4096 {
            return Err("invalid AP trial record framing");
        }
        records.push(serde_json::from_slice(line).map_err(|_| "invalid AP trial record")?);
    }
    Ok(records)
}

pub(super) fn replay_receiver(
    directory: &Path,
    receipt: &ReadyReceipt,
) -> Result<(wifi_observer::Summary, Vec<u8>), &'static str> {
    let ready =
        Ready::parse_line(receipt.line.as_bytes()).map_err(|_| "invalid AP receiver readiness")?;
    let receiver = wifi_capture::replay(directory)?;
    let raw = crate::read_input(&directory.join("packets.pcapng"))?;
    let prefix = raw
        .get(..ready.prefix_bytes)
        .ok_or("AP receiver prefix is truncated")?;
    wifi_observer::validate_section_prefix(prefix)
        .map_err(|_| "invalid AP receiver section prefix")?;
    let provenance: wifi_capture::Provenance = serde_json::from_slice(&crate::read_input(
        &directory.join("worker-provenance.json"),
    )?)
    .map_err(|_| "invalid AP receiver provenance")?;
    let stderr = crate::read_input(&directory.join("transport.stderr"))?;
    let ready_lines: Vec<_> = stderr
        .split_inclusive(|b| *b == b'\n')
        .filter(|line| line.starts_with(b"KISMET_WIFI_READY"))
        .collect();
    if ready_lines.len() != 1
        || Ready::parse_line(ready_lines[0]).map_err(|_| "invalid saved AP receiver readiness")?
            != ready
        || ready.prefix_sha256 != digest(prefix)
        || receipt.local_prefix_bytes != ready.prefix_bytes
        || receipt.local_prefix_sha256 != ready.prefix_sha256
        || ready.request_sha256 != digest(&crate::read_input(&directory.join("request.bin"))?)
        || ready.binary_sha256 != provenance.binary_sha256
        || ready.build_inputs_sha256 != provenance.build_inputs_sha256
    {
        return Err("AP receiver readiness bindings disagree");
    }
    Ok((receiver, raw))
}

fn derive_report(directory: &Path) -> Result<Report, &'static str> {
    let trial: Trial = serde_json::from_slice(&crate::read_input(&directory.join("trial.json"))?)
        .map_err(|_| "invalid AP trial identity")?;
    if trial.schema_version != VERSION
        || trial.nonce == [0; 8]
        || trial.first_id == 0
        || trial.first_id > u32::MAX - 4
    {
        return Err("invalid AP trial identity");
    }
    let connection: Connection =
        serde_json::from_slice(&crate::read_input(&directory.join("connection.json"))?)
            .map_err(|_| "invalid AP trial connection")?;
    if connection.schema_version != VERSION
        || connection.baud != 115200
        || !connection.dtr_deassertion_succeeded
        || !connection.rts_deassertion_succeeded
        || !connection.startup_accepted
        || connection.cleanup_stop_attempted
        || connection.cleanup_stop_confirmed
        || connection.recovery_required
    {
        return Err("AP trial connection did not complete");
    }
    startup::summarize(&crate::read_input(&directory.join("startup.raw"))?)?;
    if !crate::read_input(&directory.join("cleanup.jsonl"))?.is_empty() {
        return Err("AP trial includes failure cleanup");
    }
    let exchanges: Vec<Exchange> = lines(&crate::read_input(&directory.join("exchanges.jsonl"))?)?;
    if exchanges.len() != 3 {
        return Err("AP trial command sequence is incomplete");
    }
    let mut expected_tx = Vec::new();
    for (index, operation) in [Operation::Status, Operation::Start, Operation::Stop]
        .into_iter()
        .enumerate()
    {
        let expected = Request::new(trial.first_id + index as u32, operation, trial.nonce);
        if exchanges[index].request != expected {
            return Err("AP trial command order mismatch");
        }
        exchanges[index]
            .response
            .validate_for(&expected)
            .map_err(|_| "AP trial device response did not validate")?;
        let mut encoded = [0; MAX_REQUEST_BYTES];
        let length = encode_request(&expected, &mut encoded)
            .map_err(|_| "cannot reconstruct AP trial command")?;
        expected_tx.extend_from_slice(&encoded[..length]);
    }
    if crate::read_input(&directory.join("requests.jsonl"))? != expected_tx
        || crate::read_input(&directory.join("tx.raw"))? != expected_tx
    {
        return Err("AP trial UART requests disagree");
    }
    let raw = crate::read_input(&directory.join("rx.raw"))?;
    if raw.len() > UART_LIMIT {
        return Err("AP UART raw input exceeds limit");
    }
    let mut responses = Vec::new();
    for line in raw.split_inclusive(|b| *b == b'\n') {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if !line.ends_with(b"\n") || line.len() > MAX_RESPONSE_BYTES {
            return Err("AP UART raw input is incomplete");
        }
        responses.push(decode_response(line).map_err(|_| "invalid raw AP UART response")?);
    }
    if responses != exchanges.iter().map(|e| e.response).collect::<Vec<_>>() {
        return Err("AP trial UART responses disagree");
    }
    initial(&exchanges[0].response)?;
    let original = active(&exchanges[1].response, trial.nonce)?;
    stopped(&exchanges[2].response, trial.nonce, original)?;
    let events: Vec<Event> = lines(&crate::read_input(&directory.join("events.jsonl"))?)?;
    let required = [
        EventName::Status,
        EventName::ReceiverReady,
        EventName::ReceiverAlive,
        EventName::StartRequested,
        EventName::Start,
        EventName::Stop,
        EventName::ReceiverComplete,
    ];
    if events.len() != required.len()
        || events
            .iter()
            .zip(required)
            .enumerate()
            .any(|(n, (e, required))| {
                e.event != required
                    || e.sequence != n as u32 + 1
                    || (n > 0 && e.elapsed_us < events[n - 1].elapsed_us)
            })
    {
        return Err("AP trial readiness or command ordering failed");
    }
    let receipt: ReadyReceipt =
        serde_json::from_slice(&crate::read_input(&directory.join("receiver-ready.json"))?)
            .map_err(|_| "invalid AP receiver readiness receipt")?;
    let (receiver, raw) = replay_receiver(&directory.join("receiver"), &receipt)?;
    let witness = wifi_observer::beacon_witness(&raw, &ssid(trial.nonce), AP_MAC)
        .map_err(|_| "AP beacon witness did not validate")?;
    if witness.matching_beacon_count == 0 {
        return Err("AP trial has no qualified matching beacon");
    }
    Ok(Report {
        schema_version: VERSION,
        receiver,
        witness,
        uart_commands: 3,
        stop_confirmed: true,
    })
}

pub(crate) fn replay(directory: &Path) -> Result<Report, &'static str> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect AP trial directory")? {
        let entry = entry.map_err(|_| "cannot inspect AP trial entry")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid AP trial filename")?;
        let kind = entry
            .file_type()
            .map_err(|_| "cannot inspect AP trial entry")?;
        if !(kind.is_file() || kind.is_dir() && name == "receiver") {
            return Err("AP trial requires regular artifacts");
        }
        names.insert(name);
    }
    let mut expected: BTreeSet<_> = ROOT_FILES.into_iter().map(str::to_owned).collect();
    expected.insert("receiver".into());
    if names != expected {
        return Err("AP trial inventory is incomplete or unexpected");
    }
    let receipt: Receipt =
        serde_json::from_slice(&crate::read_input(&directory.join("capture-status.json"))?)
            .map_err(|_| "invalid AP trial receipt")?;
    if receipt.schema_version != VERSION || !receipt.success || receipt.error.is_some() {
        return Err("AP trial records a failed attempt");
    }
    let actual = artifacts(directory)?;
    if receipt.artifacts.len() != actual.len()
        || receipt
            .artifacts
            .iter()
            .zip(actual)
            .any(|(a, b)| a.file != b.file || a.sha256 != b.sha256)
    {
        return Err("AP trial artifacts disagree with completion receipt");
    }
    let report = derive_report(directory)?;
    if json(&report)? != crate::read_input(&directory.join("summary.json"))? {
        return Err("AP trial summary disagrees with complete replay");
    }
    Ok(report)
}

/// Compare only complete admitted trials; raw PCAPNG is never an admission path.
pub(crate) fn compare_profiles(
    before: &Path,
    after: &Path,
) -> Result<wifi_observer::ProfileComparison, &'static str> {
    replay(before)?;
    replay(after)?;

    let before_trial: Trial =
        serde_json::from_slice(&crate::read_input(&before.join("trial.json"))?)
            .map_err(|_| "invalid AP trial comparison target")?;
    let after_trial: Trial = serde_json::from_slice(&crate::read_input(&after.join("trial.json"))?)
        .map_err(|_| "invalid AP trial comparison target")?;
    let before_raw = crate::read_input(&before.join("receiver/packets.pcapng"))?;
    let after_raw = crate::read_input(&after.join("receiver/packets.pcapng"))?;
    let before_ssid = ssid(before_trial.nonce);
    let after_ssid = ssid(after_trial.nonce);
    let before_input = wifi_observer::BeaconInput::new(&before_raw, &before_ssid, AP_MAC)
        .map_err(|_| "AP beacon profile comparison input did not validate")?;
    let after_input = wifi_observer::BeaconInput::new(&after_raw, &after_ssid, AP_MAC)
        .map_err(|_| "AP beacon profile comparison input did not validate")?;
    wifi_observer::compare_beacon_profiles(before_input, after_input)
        .map_err(|_| "AP beacon profile comparison did not validate")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kismet_worker::{CredentialFile, Status, Stop};
    use serde_json::json;
    use std::collections::VecDeque;
    use std::sync::atomic::AtomicU64;
    use wifi_protocol::{Snapshot, decode_request, encode_response};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const ORIGINAL: [u8; 6] = [14, 42, 43, 44, 45, 46];
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "ap-trial-{}-{}",
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
    fn trial() -> Trial {
        Trial {
            schema_version: VERSION,
            nonce: [7, 11, 13, 17, 19, 23, 29, 31],
            first_id: 41,
        }
    }
    fn crc(bytes: &[u8]) -> u32 {
        let mut crc = !0_u32;
        for b in bytes {
            crc ^= u32::from(*b);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320_u32.wrapping_mul(crc & 1));
            }
        }
        !crc
    }
    fn raw(nonce: [u8; 8]) -> Vec<u8> {
        let mut out = vec![
            0x0a, 0x0d, 0x0d, 0x0a, 28, 0, 0, 0, 0x4d, 0x3c, 0x2b, 0x1a, 1, 0, 0, 0,
        ];
        out.extend_from_slice(&u64::MAX.to_le_bytes());
        out.extend_from_slice(&28_u32.to_le_bytes());
        out.extend_from_slice(&[
            1, 0, 0, 0, 20, 0, 0, 0, 127, 0, 0, 0, 255, 255, 0, 0, 20, 0, 0, 0,
        ]);
        let mut frame = vec![0x80, 0, 0, 0];
        frame.extend_from_slice(&[255; 6]);
        frame.extend_from_slice(&AP_MAC);
        frame.extend_from_slice(&AP_MAC);
        frame.extend_from_slice(&[0; 10]);
        frame.extend_from_slice(&[100, 0, 1, 0]);
        frame.extend_from_slice(&[0, 22]);
        frame.extend_from_slice(&ssid(nonce));
        frame.extend_from_slice(&[3, 1, 1]);
        let checksum = crc(&frame);
        frame.extend_from_slice(&checksum.to_le_bytes());
        let mut packet = vec![0, 0, 9, 0, 2, 0, 0, 0, 16];
        packet.extend_from_slice(&frame);
        let padded = (packet.len() + 3) & !3;
        for timestamp in 0..32_u32 {
            let len = 32 + padded;
            out.extend_from_slice(&6_u32.to_le_bytes());
            out.extend_from_slice(&(len as u32).to_le_bytes());
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&timestamp.to_le_bytes());
            out.extend_from_slice(&(packet.len() as u32).to_le_bytes());
            out.extend_from_slice(&(packet.len() as u32).to_le_bytes());
            out.extend_from_slice(&packet);
            out.resize(out.len() + padded - packet.len(), 0);
            out.extend_from_slice(&(len as u32).to_le_bytes());
        }
        out
    }
    fn write_json(path: &Path, value: &impl Serialize) {
        fs::write(path, super::json(value).unwrap()).unwrap();
    }
    fn receiver_artifacts(directory: &Path, nonce: [u8; 8]) -> ReadyReceipt {
        fs::create_dir(directory).unwrap();
        let raw = raw(nonce);
        let request = kismet_worker::Request {
            source_uuid: ["00000000", "0000", "0000", "0000", "000000000001"].join("-"),
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
        .encode()
        .unwrap();
        let provenance = json!({"schema_version":1,"binary_sha256":"b".repeat(64),"build_inputs_sha256":"c".repeat(64),"rustc_version":"rustc 1.90.0 (1159e78c4 2025-09-14)","curl_version":"8.17.0"});
        let ready = Ready {
            http_status: 200,
            prefix_bytes: 28,
            prefix_sha256: digest(&raw[..28]),
            request_sha256: digest(&request),
            binary_sha256: "b".repeat(64),
            build_inputs_sha256: "c".repeat(64),
        };
        let status = Status {
            stop: Stop::Boundary,
            saved_bytes: raw.len(),
            observed_bytes: raw.len(),
            epbs: 32,
            http_status: 200,
            curl_exit: 0,
            curl_signal: 0,
            intentional_kill: false,
            cleanup_success: true,
            stderr_truncated: false,
            request_sha256: digest(&request),
            binary_sha256: "b".repeat(64),
            build_inputs_sha256: "c".repeat(64),
        };
        fs::write(directory.join("request.bin"), request).unwrap();
        fs::write(directory.join("packets.pcapng"), raw).unwrap();
        fs::write(
            directory.join("transport.stderr"),
            format!("{}{}", ready.line(), status.line()),
        )
        .unwrap();
        write_json(&directory.join("worker-provenance.json"), &provenance);
        write_json(
            &directory.join("transport-status.json"),
            &json!({"schema_version":1,"ssh_exit_code":0,"ssh_signal":null,"timed_out":false,"io_failed":false,"limit_exceeded":false,"worker_status_line":status.line()}),
        );
        let summary =
            wifi_observer::analyze(&fs::read(directory.join("packets.pcapng")).unwrap()).unwrap();
        write_json(&directory.join("summary.json"), &summary);
        rehash_receiver(directory);
        ReadyReceipt {
            line: ready.line(),
            local_prefix_bytes: 28,
            local_prefix_sha256: ready.prefix_sha256,
        }
    }
    fn rehash_receiver(directory: &Path) {
        let names = [
            "request.bin",
            "packets.pcapng",
            "transport.stderr",
            "transport-status.json",
            "worker-provenance.json",
            "summary.json",
        ];
        let artifacts:Vec<_>=names.into_iter().map(|file|json!({"file":file,"sha256":digest(&fs::read(directory.join(file)).unwrap())})).collect();
        write_json(
            &directory.join("capture-status.json"),
            &json!({"schema_version":1,"success":true,"error":null,"artifacts":artifacts}),
        );
    }
    struct FakeReceiver {
        ready: Option<ReadyReceipt>,
        alive: bool,
        fail_ready: bool,
        fail_outcome: bool,
        summary: wifi_observer::Summary,
    }
    impl TrialReceiver for FakeReceiver {
        fn ready(&mut self) -> Result<ReadyReceipt, &'static str> {
            if self.fail_ready {
                Err("synthetic readiness failure")
            } else {
                self.ready.take().ok_or("missing synthetic readiness")
            }
        }
        fn alive(&self) -> bool {
            self.alive
        }
        fn failed(&mut self) -> bool {
            self.fail_outcome
        }
        fn completed(&mut self) -> bool {
            true
        }
        fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str> {
            Ok(self.summary.clone())
        }
    }
    struct FakePort {
        rx: VecDeque<u8>,
        pending: Vec<u8>,
        operations: Vec<Operation>,
        fail_start: bool,
        wrong_stop: bool,
        late_suffix: bool,
    }
    impl FakePort {
        fn new() -> Self {
            Self {
                rx: VecDeque::new(),
                pending: Vec::new(),
                operations: Vec::new(),
                fail_start: false,
                wrong_stop: false,
                late_suffix: false,
            }
        }
    }
    impl Write for FakePort {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let count = bytes.len().min(7);
            self.pending.extend_from_slice(&bytes[..count]);
            if self.pending.ends_with(b"\n") {
                let request = decode_request(&self.pending).unwrap();
                self.pending.clear();
                self.operations.push(request.operation);
                let original = if self.wrong_stop && request.operation == Operation::Stop {
                    [18, 31, 32, 33, 34, 35]
                } else {
                    ORIGINAL
                };
                let snapshot = match request.operation {
                    Operation::Status => Snapshot {
                        state: State::Idle,
                        nonce: None,
                        initialized: false,
                        running: false,
                        mac: None,
                        original_mac: None,
                        auto_stopped: false,
                    },
                    Operation::Start => Snapshot {
                        state: State::Active,
                        nonce: Some(request.nonce),
                        initialized: true,
                        running: true,
                        mac: Some(AP_MAC),
                        original_mac: Some(original),
                        auto_stopped: false,
                    },
                    Operation::Stop => Snapshot {
                        state: State::Stopped,
                        nonce: Some(request.nonce),
                        initialized: true,
                        running: false,
                        mac: Some(original),
                        original_mac: Some(original),
                        auto_stopped: true,
                    },
                };
                let failed = self.fail_start && request.operation == Operation::Start;
                let response = Response::new(
                    request.id,
                    request.operation,
                    if failed { 257 } else { 0 },
                    if failed { None } else { Some(snapshot) },
                );
                let mut bytes = [0; MAX_RESPONSE_BYTES];
                let len = encode_response(&response, &mut bytes).unwrap();
                self.rx.extend(&bytes[..len]);
                if self.late_suffix && request.operation == Operation::Stop {
                    self.rx.extend(b"PRIVATE_PANIC_SENTINEL\n");
                }
            }
            Ok(count)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl Read for FakePort {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            if let Some(byte) = self.rx.pop_front() {
                bytes[0] = byte;
                Ok(1)
            } else {
                Err(std::io::ErrorKind::TimedOut.into())
            }
        }
    }
    fn complete(directory: &Path) -> Report {
        complete_trial(directory, trial())
    }
    fn complete_trial(directory: &Path, trial: Trial) -> Report {
        write_json(&directory.join("trial.json"), &trial);
        fs::write(directory.join("startup.raw"), b"").unwrap();
        let ready = receiver_artifacts(&directory.join("receiver"), trial.nonce);
        let summary = wifi_capture::replay(&directory.join("receiver")).unwrap();
        let mut logs = Logs::new(directory).unwrap();
        let mut receiver = None;
        let mut port = FakePort::new();
        run_phases(
            &mut port,
            directory,
            &trial,
            &mut logs,
            &mut receiver,
            || {
                Ok(FakeReceiver {
                    ready: Some(ready),
                    alive: true,
                    fail_ready: false,
                    fail_outcome: false,
                    summary,
                })
            },
            Duration::ZERO,
        )
        .unwrap();
        let connection = Connection {
            schema_version: 1,
            baud: 115200,
            dtr_deassertion_succeeded: true,
            rts_deassertion_succeeded: true,
            startup_accepted: true,
            ..Connection::default()
        };
        finish_capture(directory, &logs, &connection, Ok(())).unwrap()
    }

    fn rehash(directory: &Path) {
        write_json(
            &directory.join("capture-status.json"),
            &Receipt {
                schema_version: 1,
                success: true,
                error: None,
                artifacts: artifacts(directory).unwrap(),
            },
        );
    }

    #[test]
    fn actual_uart_framing_save_and_replay_produces_only_presence_counts() {
        let t = Temp::new();
        let report = complete(&t.0);
        assert_eq!(report.witness.matching_beacon_count, 32);
        assert!(replay(&t.0).is_ok());
        let text = String::from_utf8(super::json(&report).unwrap()).unwrap();
        assert!(!text.contains("nonce"));
        assert!(!text.contains("original_mac"));
        assert!(!text.contains("di-ap-"));
    }

    #[test]
    fn receiver_not_ready_or_dead_never_sends_start_and_failed_start_remains_recoverable() {
        for mode in 0..4 {
            let t = Temp::new();
            let trial = trial();
            write_json(&t.0.join("trial.json"), &trial);
            fs::write(t.0.join("startup.raw"), b"").unwrap();
            let ready = receiver_artifacts(&t.0.join("receiver"), trial.nonce);
            let summary = wifi_capture::replay(&t.0.join("receiver")).unwrap();
            let mut port = FakePort::new();
            port.fail_start = mode == 2;
            let mut logs = Logs::new(&t.0).unwrap();
            let mut receiver = None;
            let result = run_phases(
                &mut port,
                &t.0,
                &trial,
                &mut logs,
                &mut receiver,
                || {
                    Ok(FakeReceiver {
                        ready: Some(ready),
                        alive: mode != 1,
                        fail_ready: mode == 0,
                        fail_outcome: mode == 3,
                        summary,
                    })
                },
                Duration::ZERO,
            );
            assert!(result.is_err());
            if mode < 2 || mode == 3 {
                assert_eq!(port.operations, [Operation::Status]);
            } else {
                assert_eq!(port.operations, [Operation::Status, Operation::Start]);
            }
            let response = logs
                .exchange(
                    &mut port,
                    Request::new(trial.first_id + 3, Operation::Stop, trial.nonce),
                    true,
                )
                .unwrap();
            assert!(cleanup_stopped(&response, trial.nonce));
            assert_eq!(port.operations.last(), Some(&Operation::Stop));
            let connection = Connection {
                schema_version: 1,
                baud: 115200,
                dtr_deassertion_succeeded: true,
                rts_deassertion_succeeded: true,
                startup_accepted: true,
                cleanup_stop_attempted: true,
                cleanup_stop_confirmed: true,
                recovery_required: true,
            };
            assert!(finish_capture(&t.0, &logs, &connection, result).is_err());
            let receipt: Receipt =
                serde_json::from_slice(&fs::read(t.0.join("capture-status.json")).unwrap())
                    .unwrap();
            assert!(!receipt.success);
            assert!(receipt.error.is_some());
            assert!(!t.0.join("summary.json").exists());
            assert!(!fs::read(t.0.join("rx.raw")).unwrap().is_empty());
            assert!(replay(&t.0).is_err());
        }
    }

    #[test]
    fn cross_phase_wrong_restoration_and_late_uart_suffix_are_rejected() {
        for mode in 0..2 {
            let t = Temp::new();
            let trial = trial();
            let ready = receiver_artifacts(&t.0.join("receiver"), trial.nonce);
            let summary = wifi_capture::replay(&t.0.join("receiver")).unwrap();
            let mut port = FakePort::new();
            port.wrong_stop = mode == 0;
            port.late_suffix = mode == 1;
            let mut logs = Logs::new(&t.0).unwrap();
            let mut receiver = None;
            assert!(
                run_phases(
                    &mut port,
                    &t.0,
                    &trial,
                    &mut logs,
                    &mut receiver,
                    || Ok(FakeReceiver {
                        ready: Some(ready),
                        alive: true,
                        fail_ready: false,
                        fail_outcome: false,
                        summary
                    }),
                    Duration::ZERO
                )
                .is_err()
            );
        }
    }

    #[test]
    fn rehashed_ready_order_nonce_and_uart_mutations_cannot_pass() {
        for mutation in 0..5 {
            let t = Temp::new();
            complete(&t.0);
            match mutation {
                0 => {
                    let path = t.0.join("receiver-ready.json");
                    let mut v: ReadyReceipt =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    let mut ready = Ready::parse_line(v.line.as_bytes()).unwrap();
                    ready.prefix_sha256 = "f".repeat(64);
                    v.line = ready.line();
                    v.local_prefix_sha256 = ready.prefix_sha256;
                    write_json(&path, &v);
                }
                1 => {
                    let path = t.0.join("events.jsonl");
                    let mut events: Vec<Event> = lines(&fs::read(&path).unwrap()).unwrap();
                    events.swap(1, 3);
                    let bytes: Vec<_> = events
                        .iter()
                        .flat_map(|e| {
                            let mut b = serde_json::to_vec(e).unwrap();
                            b.push(b'\n');
                            b
                        })
                        .collect();
                    fs::write(path, bytes).unwrap();
                }
                2 => {
                    fs::write(t.0.join("rx.raw"), b"PRIVATE_PANIC_SENTINEL\n").unwrap();
                }
                3 => {
                    let mut changed = trial().nonce;
                    changed[0] ^= 1;
                    let raw = raw(changed);
                    fs::write(t.0.join("receiver/packets.pcapng"), &raw).unwrap();
                    write_json(
                        &t.0.join("receiver/summary.json"),
                        &wifi_observer::analyze(&raw).unwrap(),
                    );
                    rehash_receiver(&t.0.join("receiver"));
                }
                4 => {
                    let path = t.0.join("receiver/transport.stderr");
                    let bytes = fs::read(&path).unwrap();
                    let first = bytes.split_inclusive(|b| *b == b'\n').next().unwrap();
                    let mut altered = first.to_vec();
                    altered.extend_from_slice(&bytes);
                    fs::write(path, altered).unwrap();
                    rehash_receiver(&t.0.join("receiver"));
                }
                _ => unreachable!(),
            };
            rehash(&t.0);
            assert!(replay(&t.0).is_err());
        }
    }

    #[test]
    fn fragmented_local_gate_requires_exact_prefix_and_all_bindings() {
        for bad in [false, true] {
            let t = Temp::new();
            let raw = raw(trial().nonce);
            let prefix = &raw[..28];
            let mut file = private_file(&t.0.join("raw")).unwrap();
            file.write_all(prefix).unwrap();
            let (tx, rx) = mpsc::channel();
            let mut gate = Gate {
                expected_request: "a".repeat(64),
                expected_binary: "b".repeat(64),
                expected_build: "c".repeat(64),
                prefix: Vec::new(),
                prefix_length: None,
                stderr_line: Vec::new(),
                ready: None,
                announced: false,
                failed: false,
                notify: tx,
                raw_sync: file,
            };
            let ready = Ready {
                http_status: 200,
                prefix_bytes: 28,
                prefix_sha256: if bad { "d".repeat(64) } else { digest(prefix) },
                request_sha256: "a".repeat(64),
                binary_sha256: "b".repeat(64),
                build_inputs_sha256: "c".repeat(64),
            };
            for chunk in ready.line().as_bytes().chunks(3) {
                gate.stderr(chunk);
            }
            assert!(rx.try_recv().is_err());
            for chunk in prefix.chunks(5) {
                gate.body(chunk);
            }
            assert_eq!(
                rx.recv_timeout(Duration::from_secs(1)).unwrap().is_ok(),
                !bad
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn cancelled_live_receiver_does_not_authorize_start() {
        let child = std::process::Command::new("/bin/sleep")
            .arg("5")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let control = Control {
            child: Mutex::new(Some(child)),
            cancel: AtomicBool::new(false),
        };
        let initially_alive = control.alive();
        // Reproduce the shutdown interval without killing the child yet.
        control.cancel.store(true, Ordering::Release);
        let cancelled_authorizes_start = control.alive();
        let mut child = control.child.lock().unwrap().take().unwrap();
        let child_still_running = child.try_wait().unwrap().is_none();
        let _ = child.kill();
        child.wait().unwrap();
        assert!(initially_alive);
        assert!(child_still_running);
        assert!(!cancelled_authorizes_start);
    }

    #[test]
    fn profiles_compare_bound_trial_targets_and_allow_the_same_capture() {
        let before = Temp::new();
        complete(&before.0);
        let after = Temp::new();
        let mut changed = trial();
        changed.nonce[0] ^= 128;
        complete_trial(&after.0, changed);
        let comparison = compare_profiles(&before.0, &after.0).unwrap();
        assert_eq!(comparison.before_target_packet_count, 32);
        assert_eq!(comparison.after_target_packet_count, 32);
        assert!(!comparison.expected_ssids_equal);
        assert!(comparison.expected_macs_equal);
        assert!(comparison.beacon_interval.identical_across_all_beacons);
        assert!(comparison.ordered_non_ssid_ies.identical_across_all_beacons);
        assert!(
            comparison
                .elements
                .iter()
                .all(|element| element.element_id != 0)
        );
        let bytes = super::json(&comparison).unwrap();
        assert!(!bytes.windows(b"di-ap-".len()).any(|w| w == b"di-ap-"));
        assert!(
            !bytes
                .windows(b"original_mac".len())
                .any(|w| w == b"original_mac")
        );
        assert!(
            compare_profiles(&before.0, &before.0)
                .unwrap()
                .expected_ssids_equal
        );
    }

    #[test]
    fn profile_comparison_rejects_raw_paths_and_invalid_full_bundles_on_either_side() {
        let good = Temp::new();
        complete(&good.0);
        let raw_path = good.0.join("receiver/packets.pcapng");
        assert!(compare_profiles(&raw_path, &good.0).is_err());
        assert!(compare_profiles(&good.0, &raw_path).is_err());
        for mutation in 0..4 {
            let bad = Temp::new();
            complete(&bad.0);
            match mutation {
                0 => {
                    let path = bad.0.join("receiver/packets.pcapng");
                    let mut bytes = fs::read(&path).unwrap();
                    *bytes.last_mut().unwrap() ^= 1;
                    fs::write(path, bytes).unwrap();
                }
                1 => {
                    let path = bad.0.join("exchanges.jsonl");
                    let mut exchanges: Vec<Exchange> = lines(&fs::read(&path).unwrap()).unwrap();
                    exchanges[0].request =
                        Request::new(trial().first_id, Operation::Start, trial().nonce);
                    let bytes: Vec<_> = exchanges
                        .iter()
                        .flat_map(|e| {
                            let mut b = serde_json::to_vec(e).unwrap();
                            b.push(b'\n');
                            b
                        })
                        .collect();
                    fs::write(path, bytes).unwrap();
                    rehash(&bad.0);
                }
                2 => {
                    let path = bad.0.join("capture-status.json");
                    let mut receipt: Receipt =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    receipt.success = false;
                    receipt.error = Some("PRIVATE_PROFILE_SENTINEL".into());
                    write_json(&path, &receipt);
                }
                3 => {
                    fs::remove_file(bad.0.join("summary.json")).unwrap();
                }
                _ => unreachable!(),
            }
            for (before, after) in [(&bad.0, &good.0), (&good.0, &bad.0)] {
                let error = compare_profiles(before, after).err().unwrap();
                assert!(!error.contains("PRIVATE_PROFILE_SENTINEL"));
            }
        }
    }

    #[test]
    fn admitted_presence_trial_with_one_background_or_bad_fcs_packet_cannot_be_a_profile_input() {
        let good = Temp::new();
        complete(&good.0);
        for bad_fcs in [false, true] {
            let partial = Temp::new();
            complete(&partial.0);
            let mut bytes = raw(trial().nonce);
            let block_length =
                u32::from_le_bytes(bytes[bytes.len() - 4..].try_into().unwrap()) as usize;
            let start = bytes.len() - block_length;
            if bad_fcs {
                let captured =
                    u32::from_le_bytes(bytes[start + 20..start + 24].try_into().unwrap()) as usize;
                bytes[start + 28 + captured - 1] ^= 1;
            } else {
                let mut nonce = trial().nonce;
                nonce[0] ^= 1;
                let other = raw(nonce);
                bytes[start..].copy_from_slice(&other[start..]);
            }
            fs::write(partial.0.join("receiver/packets.pcapng"), &bytes).unwrap();
            write_json(
                &partial.0.join("receiver/summary.json"),
                &wifi_observer::analyze(&bytes).unwrap(),
            );
            rehash_receiver(&partial.0.join("receiver"));
            let report = derive_report(&partial.0).unwrap();
            assert_eq!(report.witness.matching_beacon_count, 31);
            write_json(&partial.0.join("summary.json"), &report);
            rehash(&partial.0);
            assert!(replay(&partial.0).is_ok());
            assert!(compare_profiles(&good.0, &partial.0).is_err());
            assert!(compare_profiles(&partial.0, &good.0).is_err());
        }
    }
}
