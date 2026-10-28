//! One retained UART session: checked AP A stop/restoration before AP B starts.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wifi_protocol::restart::{
    AP_WINDOW_MILLIS, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, Operation, Phase, Request, Response,
    State, decode_response, encode_request, ssid,
};

use crate::capture::{create_private_directory, private_file, save_json};
use crate::wifi_trial::{self, ReadyReceipt, Receiver, TrialReceiver};
use crate::{startup, wifi_capture};

const VERSION: u32 = 2;
const UART_LIMIT: usize = 64 * 1024;
const UART_TIMEOUT: Duration = Duration::from_secs(5);
const READY_TIMEOUT: Duration = Duration::from_secs(30);
const ROOT_FILES: [&str; 13] = [
    "restart.json",
    "connection.json",
    "startup.raw",
    "tx.raw",
    "rx.raw",
    "requests.jsonl",
    "exchanges.jsonl",
    "cleanup.jsonl",
    "events.jsonl",
    "ready-first.json",
    "ready-second.json",
    "summary.json",
    "capture-status.json",
];
const PHASES: [Phase; 2] = [Phase::First, Phase::Second];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    let mut b =
        serde_json::to_vec_pretty(value).map_err(|_| "cannot encode AP restart metadata")?;
    b.push(b'\n');
    Ok(b)
}
fn paths(phase: Phase) -> (&'static str, &'static str) {
    match phase {
        Phase::First => ("receiver-first", "ready-first.json"),
        Phase::Second => ("receiver-second", "ready-second.json"),
    }
}
fn start_operation(phase: Phase) -> Operation {
    match phase {
        Phase::First => Operation::StartFirst,
        Phase::Second => Operation::StartSecond,
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema_version: u32,
    nonce: [u8; 8],
    first_id: u32,
}
impl Session {
    fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != VERSION
            || self.nonce == [0; 8]
            || self.first_id == 0
            || self.first_id > u32::MAX - 5
        {
            return Err("invalid AP restart identity");
        }
        Ok(())
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Connection {
    schema_version: u32,
    baud: u32,
    uart_open_count: u32,
    control_release_count: u32,
    startup_window_count: u32,
    dtr_deassertion_succeeded: bool,
    rts_deassertion_succeeded: bool,
    startup_accepted: bool,
    cleanup_stop_attempted: bool,
    cleanup_stop_confirmed: bool,
    recovery_required: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EventKind {
    Status,
    ReceiverOpened,
    ReceiverReady,
    ReceiverAlive,
    StartRequested,
    Start,
    Stop,
    ReceiverComplete,
    PhaseAdmitted,
    CleanupStop,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Event {
    sequence: u32,
    elapsed_us: u64,
    phase: Option<Phase>,
    event: EventKind,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Exchange {
    request: Request,
    response: Response,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PhaseReport {
    phase: Phase,
    receiver: wifi_observer::Summary,
    witness: wifi_observer::WitnessSummary,
    stop_confirmed: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum UnqualifiedReason {
    CapturesNotEntirelyCheckedTargets,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Collateral {
    Qualified(wifi_observer::ProfileComparison),
    Unqualified(UnqualifiedReason),
}
/// Only fixed counts and checked relationships; identifiers remain private.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: u32,
    phases: [PhaseReport; 2],
    uart_commands: u32,
    one_uart_connection: bool,
    target_addresses_differ: bool,
    same_ssid: bool,
    collateral: Collateral,
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

// Production timers retain the real monotonic clock. Offline transport models
// explicitly advance their own time, independent of filesystem/scheduler load.
enum Clock {
    Real(Instant),
    #[cfg(test)]
    Manual(std::rc::Rc<std::cell::Cell<Duration>>),
}
impl Clock {
    fn now(&self) -> Duration {
        match self {
            Self::Real(started) => started.elapsed(),
            #[cfg(test)]
            Self::Manual(now) => now.get(),
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
    clock: Clock,
    sequence: u32,
    received: usize,
    original: Option<[u8; 6]>,
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
            clock: Clock::Real(Instant::now()),
            sequence: 0,
            received: 0,
            original: None,
        })
    }
    fn deadline(&self, timeout: Duration) -> Result<Duration, &'static str> {
        self.clock
            .now()
            .checked_add(timeout)
            .ok_or("AP restart deadline overflow")
    }
    fn expired(&self, deadline: Duration) -> bool {
        self.clock.now() >= deadline
    }
    #[cfg(test)]
    fn with_manual_clock(
        directory: &Path,
        now: std::rc::Rc<std::cell::Cell<Duration>>,
    ) -> Result<Self, &'static str> {
        let mut logs = Self::new(directory)?;
        logs.clock = Clock::Manual(now);
        Ok(logs)
    }
    fn event(&mut self, phase: Option<Phase>, event: EventKind) -> Result<(), &'static str> {
        self.sequence += 1;
        let record = Event {
            sequence: self.sequence,
            elapsed_us: u64::try_from(self.clock.now().as_micros())
                .map_err(|_| "AP restart clock overflow")?,
            phase,
            event,
        };
        serde_json::to_writer(&mut self.events, &record)
            .map_err(|_| "cannot save AP restart event")?;
        self.events
            .write_all(b"\n")
            .map_err(|_| "cannot save AP restart event")
    }
    fn byte(&mut self, port: &mut impl Read) -> Result<Option<u8>, &'static str> {
        let mut byte = [0];
        match port.read(&mut byte) {
            Ok(0) => Ok(None),
            Ok(1) => {
                self.rx
                    .write_all(&byte)
                    .map_err(|_| "cannot save AP restart UART input")?;
                self.received += 1;
                if self.received > UART_LIMIT {
                    return Err("AP restart UART input exceeds limit");
                }
                Ok(Some(byte[0]))
            }
            Ok(_) => Err("invalid AP restart UART read length"),
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
            Err(_) => Err("cannot read AP restart UART"),
        }
    }
    fn quiet(&mut self, port: &mut impl Read) -> Result<(), &'static str> {
        while let Some(byte) = self.byte(port)? {
            if !byte.is_ascii_whitespace() {
                return Err("unexpected AP restart UART output");
            }
        }
        Ok(())
    }
    fn exchange(
        &mut self,
        port: &mut (impl Read + Write),
        request: Request,
        cleanup: bool,
        timeout: Duration,
    ) -> Result<Response, &'static str> {
        let mut encoded = [0; MAX_REQUEST_BYTES];
        let length = encode_request(&request, &mut encoded)
            .map_err(|_| "cannot encode AP restart request")?;
        self.requests
            .write_all(&encoded[..length])
            .map_err(|_| "cannot save AP restart request")?;
        let deadline = self.deadline(timeout)?;
        let mut sent = 0;
        while sent < length {
            if self.expired(deadline) {
                return Err("AP restart UART write timed out");
            }
            match port.write(&encoded[sent..length]) {
                Ok(0) => thread::yield_now(),
                Ok(n) if n <= length - sent => {
                    self.tx
                        .write_all(&encoded[sent..sent + n])
                        .map_err(|_| "cannot save AP restart UART output")?;
                    sent += n;
                }
                Ok(_) => return Err("invalid AP restart UART write length"),
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::Interrupted
                            | std::io::ErrorKind::WouldBlock
                    ) => {}
                Err(_) => return Err("cannot write AP restart UART"),
            }
        }
        let mut line = Vec::new();
        while !self.expired(deadline) {
            if let Some(byte) = self.byte(port)? {
                // Retain even a late final LF, but do not admit an ACK whose
                // read/raw save completed at or beyond the exchange deadline.
                if self.expired(deadline) {
                    return Err("AP restart UART response timed out");
                }
                line.push(byte);
                if line.len() > MAX_RESPONSE_BYTES {
                    return Err("AP restart UART line exceeds limit");
                }
                if byte == b'\n' {
                    if line.iter().all(u8::is_ascii_whitespace) {
                        line.clear();
                        continue;
                    }
                    let response =
                        decode_response(&line).map_err(|_| "invalid AP restart response")?;
                    let record = Exchange { request, response };
                    let output = if cleanup {
                        &mut self.cleanup
                    } else {
                        &mut self.exchanges
                    };
                    serde_json::to_writer(&mut *output, &record)
                        .map_err(|_| "cannot save AP restart exchange")?;
                    output
                        .write_all(b"\n")
                        .map_err(|_| "cannot save AP restart exchange")?;
                    response
                        .validate_for(&request)
                        .map_err(|_| "AP restart operation did not validate")?;
                    return Ok(response);
                }
            }
        }
        Err("AP restart UART response timed out")
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
                .map_err(|_| "cannot synchronize AP restart UART logs")?;
        }
        Ok(())
    }
}

fn initial(response: &Response) -> Result<(), &'static str> {
    let s = response
        .snapshot
        .as_ref()
        .ok_or("missing AP restart initial state")?;
    if s.state != State::Idle
        || s.phase.is_some()
        || s.nonce.is_some()
        || s.initialized
        || s.running
        || s.mac.is_some()
        || s.original_mac.is_some()
        || s.first_failure.is_some()
    {
        return Err("AP restart did not begin idle");
    }
    Ok(())
}
fn active(
    response: &Response,
    phase: Phase,
    nonce: [u8; 8],
    saved: Option<[u8; 6]>,
) -> Result<[u8; 6], &'static str> {
    let s = response
        .snapshot
        .as_ref()
        .ok_or("missing AP restart active state")?;
    let original = s
        .original_mac
        .ok_or("missing AP restart original address")?;
    if s.state != State::Active
        || s.phase != Some(phase)
        || s.nonce != Some(nonce)
        || !s.initialized
        || !s.running
        || s.mac != Some(phase.mac())
        || s.first_failure.is_some()
        || PHASES.iter().any(|phase| phase.mac() == original)
        || saved.is_some_and(|saved| saved != original)
    {
        return Err("AP restart active state did not validate");
    }
    Ok(original)
}
fn stopped(
    response: &Response,
    phase: Phase,
    nonce: [u8; 8],
    original: [u8; 6],
) -> Result<(), &'static str> {
    let s = response
        .snapshot
        .as_ref()
        .ok_or("missing AP restart stopped state")?;
    if s.state != State::Stopped
        || s.phase != Some(phase)
        || s.nonce != Some(nonce)
        || !s.initialized
        || s.running
        || s.mac != Some(original)
        || s.original_mac != Some(original)
        || s.first_failure.is_some()
    {
        return Err("AP restart Stop did not restore the original address");
    }
    Ok(())
}
fn cleanup_stopped(response: &Response, nonce: [u8; 8], original: Option<[u8; 6]>) -> bool {
    response.snapshot.as_ref().is_some_and(|s| {
        !s.running
            && match s.state {
                State::Idle => !s.initialized && s.nonce.is_none() && s.phase.is_none(),
                State::Stopped => {
                    s.nonce == Some(nonce)
                        && s.phase.is_some()
                        && if let Some(original) = original {
                            s.initialized
                                && s.mac == Some(original)
                                && s.original_mac == Some(original)
                        } else {
                            !s.initialized && s.mac.is_none() && s.original_mac.is_none()
                                || s.initialized && s.mac.is_some() && s.mac == s.original_mac
                        }
                }
                State::Active => false,
            }
    })
}

trait PhaseReceiver {
    fn poll_ready(&mut self) -> Result<Option<ReadyReceipt>, &'static str>;
    fn alive(&self) -> bool;
    fn failed(&mut self) -> bool;
    fn completed(&mut self) -> bool;
    fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str>;
}
impl PhaseReceiver for Receiver {
    fn poll_ready(&mut self) -> Result<Option<ReadyReceipt>, &'static str> {
        Receiver::poll_ready(self)
    }
    fn alive(&self) -> bool {
        TrialReceiver::alive(self)
    }
    fn failed(&mut self) -> bool {
        TrialReceiver::failed(self)
    }
    fn completed(&mut self) -> bool {
        TrialReceiver::completed(self)
    }
    fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str> {
        TrialReceiver::finish(self)
    }
}

#[derive(Clone, Copy)]
struct Timing {
    exchange: Duration,
    ready: Duration,
    window: Duration,
    suffix: Duration,
}
impl Timing {
    fn production() -> Self {
        Self {
            exchange: UART_TIMEOUT,
            ready: READY_TIMEOUT,
            window: Duration::from_millis(u64::from(AP_WINDOW_MILLIS)),
            suffix: Duration::from_millis(200),
        }
    }
}

fn phase_report(
    directory: &Path,
    session: &Session,
    phase: Phase,
) -> Result<(PhaseReport, Vec<u8>), &'static str> {
    let (capture, ready) = paths(phase);
    let ready: ReadyReceipt = serde_json::from_slice(&crate::read_input(&directory.join(ready))?)
        .map_err(|_| "invalid AP restart readiness receipt")?;
    let (receiver, raw) = wifi_trial::replay_receiver(&directory.join(capture), &ready)?;
    let witness = wifi_observer::beacon_witness(&raw, &ssid(session.nonce), phase.mac())
        .map_err(|_| "AP restart beacon witness did not validate")?;
    if witness.matching_beacon_count == 0 {
        return Err("AP restart phase has no qualified matching beacon");
    }
    Ok((
        PhaseReport {
            phase,
            receiver,
            witness,
            stop_confirmed: true,
        },
        raw,
    ))
}

fn run_session<P, R, F>(
    port: &mut P,
    directory: &Path,
    session: &Session,
    logs: &mut Logs,
    receivers: &mut Vec<R>,
    mut start_receiver: F,
    timing: Timing,
) -> Result<(), &'static str>
where
    P: Read + Write,
    R: PhaseReceiver,
    F: FnMut(Phase, PathBuf) -> Result<R, &'static str>,
{
    let status = logs.exchange(
        port,
        Request::new(session.first_id, Operation::Status, session.nonce),
        false,
        timing.exchange,
    )?;
    initial(&status)?;
    logs.event(None, EventKind::Status)?;
    let mut validated_raw = Vec::with_capacity(2);
    for (index, phase) in PHASES.into_iter().enumerate() {
        logs.event(Some(phase), EventKind::ReceiverOpened)?;
        receivers.push(start_receiver(phase, directory.join(paths(phase).0))?);
        let current = receivers.last_mut().ok_or("missing AP restart receiver")?;
        let until = logs.deadline(timing.ready)?;
        let ready = loop {
            if logs.expired(until) {
                return Err("AP restart receiver readiness timed out");
            }
            logs.quiet(port)?;
            if logs.expired(until) {
                return Err("AP restart receiver readiness timed out");
            }
            if current.failed() {
                return Err("AP restart receiver failed before readiness");
            }
            if let Some(ready) = current.poll_ready()? {
                break ready;
            }
        };
        save_json(&directory.join(paths(phase).1), &ready)?;
        logs.event(Some(phase), EventKind::ReceiverReady)?;
        logs.quiet(port)?;
        if current.failed() || !current.alive() {
            return Err("AP restart receiver ended before Start");
        }
        logs.event(Some(phase), EventKind::ReceiverAlive)?;
        logs.event(Some(phase), EventKind::StartRequested)?;
        if current.failed() || !current.alive() {
            return Err("AP restart receiver failed before Start request");
        }
        let start = logs.exchange(
            port,
            Request::new(
                session.first_id + 1 + index as u32 * 2,
                start_operation(phase),
                session.nonce,
            ),
            false,
            timing.exchange,
        )?;
        let original = active(&start, phase, session.nonce, logs.original)?;
        logs.original = Some(original);
        logs.event(Some(phase), EventKind::Start)?;
        let until = logs.deadline(timing.window)?;
        while !logs.expired(until) {
            if current.failed() {
                return Err("AP restart receiver failed while AP active");
            }
            logs.quiet(port)?;
        }
        let stop = logs.exchange(
            port,
            Request::new(
                session.first_id + 2 + index as u32 * 2,
                Operation::Stop,
                session.nonce,
            ),
            false,
            timing.exchange,
        )?;
        stopped(&stop, phase, session.nonce, original)?;
        logs.event(Some(phase), EventKind::Stop)?;
        let deadline = logs.deadline(Duration::from_secs(140))?;
        while !current.completed() {
            logs.quiet(port)?;
            if logs.expired(deadline) {
                return Err("AP restart receiver completion timed out");
            }
        }
        current.finish()?;
        let until = logs.deadline(timing.suffix)?;
        while !logs.expired(until) {
            logs.quiet(port)?;
        }
        logs.event(Some(phase), EventKind::ReceiverComplete)?;
        let (_, raw) = phase_report(directory, session, phase)?;
        validated_raw.push(raw);
        logs.event(Some(phase), EventKind::PhaseAdmitted)?;
    }
    let [first, second] = validated_raw.as_slice() else {
        return Err("AP restart phase captures are incomplete");
    };
    collateral(first, second, session.nonce)?;
    Ok(())
}

fn fresh_session() -> Result<Session, &'static str> {
    let mut random =
        File::open("/dev/urandom").map_err(|_| "cannot obtain AP restart randomness")?;
    for _ in 0..8 {
        let mut nonce = [0; 8];
        let mut id = [0; 4];
        random
            .read_exact(&mut nonce)
            .and_then(|()| random.read_exact(&mut id))
            .map_err(|_| "cannot read AP restart randomness")?;
        let first_id = u32::from_ne_bytes(id);
        let session = Session {
            schema_version: VERSION,
            nonce,
            first_id,
        };
        if session.validate().is_ok() {
            return Ok(session);
        }
    }
    Err("cannot choose AP restart correlation")
}

pub(crate) fn capture(
    port_file: &Path,
    profile: &Path,
    directory: &Path,
) -> Result<Report, &'static str> {
    let port = crate::read_input(port_file)?;
    let port = std::str::from_utf8(&port)
        .map_err(|_| "invalid private AP restart bridge")?
        .trim();
    if !port.starts_with("/dev/cu.")
        || port.len() > 256
        || !port
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    {
        return Err("invalid private AP restart bridge");
    }
    let profile = wifi_capture::load_profile(profile)?;
    create_private_directory(directory)?;
    let session = fresh_session()?;
    save_json(&directory.join("restart.json"), &session)?;
    let mut logs = Logs::new(directory)?;
    let mut startup = private_file(&directory.join("startup.raw"))?;
    let mut connection = Connection {
        schema_version: VERSION,
        baud: 115200,
        ..Connection::default()
    };
    let mut serial: Option<Box<dyn serialport::SerialPort>> = None;
    let mut receivers = Vec::new();
    let result = (|| {
        // This is the only open and DTR/RTS release in the retained session.
        let mut opened = serialport::new(port, 115200)
            .timeout(Duration::from_millis(100))
            .dtr_on_open(false)
            .open()
            .map_err(|_| "cannot open AP restart UART")?;
        connection.uart_open_count = 1;
        connection.control_release_count = 1;
        connection.dtr_deassertion_succeeded = opened.write_data_terminal_ready(false).is_ok();
        connection.rts_deassertion_succeeded = opened.write_request_to_send(false).is_ok();
        serial = Some(opened);
        if !connection.dtr_deassertion_succeeded || !connection.rts_deassertion_succeeded {
            return Err("cannot release AP restart UART control lines");
        }
        let port = serial.as_mut().ok_or("missing AP restart UART handle")?;
        connection.startup_window_count = 1;
        wifi_trial::startup_capture(port, &mut startup)?;
        connection.startup_accepted = true;
        run_session(
            port,
            directory,
            &session,
            &mut logs,
            &mut receivers,
            |_, path| Receiver::start(profile.clone(), path),
            Timing::production(),
        )
    })();
    if result.is_err() {
        if let Some(port) = serial.as_mut() {
            connection.cleanup_stop_attempted = true;
            connection.cleanup_stop_confirmed = logs
                .exchange(
                    port,
                    Request::new(session.first_id + 5, Operation::Stop, session.nonce),
                    true,
                    UART_TIMEOUT,
                )
                .is_ok_and(|response| cleanup_stopped(&response, session.nonce, logs.original));
            let _ = logs.event(None, EventKind::CleanupStop);
        }
        connection.recovery_required = true;
    }
    drop(receivers);
    finish_capture(directory, &logs, &connection, result)
}

fn artifacts(directory: &Path) -> Result<Vec<Artifact>, &'static str> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect AP restart artifacts")? {
        let entry = entry.map_err(|_| "cannot inspect AP restart artifact")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid AP restart artifact name")?;
        if name == "capture-status.json" {
            continue;
        }
        let kind = entry
            .file_type()
            .map_err(|_| "cannot inspect AP restart artifact")?;
        if kind.is_file() {
            names.push(name);
        } else if kind.is_dir() && ["receiver-first", "receiver-second"].contains(&name.as_str()) {
            for child in
                fs::read_dir(entry.path()).map_err(|_| "cannot inspect AP restart receiver")?
            {
                let child = child.map_err(|_| "cannot inspect AP restart receiver artifact")?;
                if !child
                    .file_type()
                    .map_err(|_| "cannot inspect AP restart receiver artifact")?
                    .is_file()
                {
                    return Err("AP restart artifacts must be regular files");
                }
                let child = child
                    .file_name()
                    .into_string()
                    .map_err(|_| "invalid AP restart receiver filename")?;
                names.push(format!("{name}/{child}"));
            }
        } else {
            return Err("unexpected AP restart artifact");
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|file| {
            Ok(Artifact {
                sha256: digest(&crate::read_input(&directory.join(&file))?),
                file,
            })
        })
        .collect()
}

fn finish_capture(
    directory: &Path,
    logs: &Logs,
    connection: &Connection,
    result: Result<(), &'static str>,
) -> Result<Report, &'static str> {
    logs.sync()?;
    save_json(&directory.join("connection.json"), connection)?;
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
            return Err("invalid AP restart record framing");
        }
        records.push(serde_json::from_slice(line).map_err(|_| "invalid AP restart record")?);
    }
    Ok(records)
}

fn collateral(first: &[u8], second: &[u8], nonce: [u8; 8]) -> Result<Collateral, &'static str> {
    let target = ssid(nonce);
    let result = (|| {
        let first = wifi_observer::BeaconInput::new(first, &target, Phase::First.mac())?;
        let second = wifi_observer::BeaconInput::new(second, &target, Phase::Second.mac())?;
        wifi_observer::compare_beacon_profiles(first, second)
    })();
    match result {
        Ok(report) => {
            if !report.expected_ssids_equal || report.expected_macs_equal {
                return Err("AP restart collateral targets disagree");
            }
            Ok(Collateral::Qualified(report))
        }
        Err(wifi_observer::Error::ProfileCapture) => Ok(Collateral::Unqualified(
            UnqualifiedReason::CapturesNotEntirelyCheckedTargets,
        )),
        Err(_) => Err("AP restart collateral comparison did not validate"),
    }
}

fn derive_report(directory: &Path) -> Result<Report, &'static str> {
    let session: Session =
        serde_json::from_slice(&crate::read_input(&directory.join("restart.json"))?)
            .map_err(|_| "invalid AP restart identity")?;
    session.validate()?;
    let connection: Connection =
        serde_json::from_slice(&crate::read_input(&directory.join("connection.json"))?)
            .map_err(|_| "invalid AP restart connection")?;
    if connection.schema_version != VERSION
        || connection.baud != 115200
        || connection.uart_open_count != 1
        || connection.control_release_count != 1
        || connection.startup_window_count != 1
        || !connection.dtr_deassertion_succeeded
        || !connection.rts_deassertion_succeeded
        || !connection.startup_accepted
        || connection.cleanup_stop_attempted
        || connection.cleanup_stop_confirmed
        || connection.recovery_required
    {
        return Err("AP restart did not retain one qualified connection");
    }
    startup::summarize(&crate::read_input(&directory.join("startup.raw"))?)?;
    if !crate::read_input(&directory.join("cleanup.jsonl"))?.is_empty() {
        return Err("AP restart includes failure cleanup");
    }
    let exchanges: Vec<Exchange> = lines(&crate::read_input(&directory.join("exchanges.jsonl"))?)?;
    if exchanges.len() != 5 {
        return Err("AP restart command sequence is incomplete");
    }
    let operations = [
        Operation::Status,
        Operation::StartFirst,
        Operation::Stop,
        Operation::StartSecond,
        Operation::Stop,
    ];
    let mut expected_tx = Vec::new();
    for (index, operation) in operations.into_iter().enumerate() {
        let expected = Request::new(session.first_id + index as u32, operation, session.nonce);
        if exchanges[index].request != expected {
            return Err("AP restart command order mismatch");
        }
        exchanges[index]
            .response
            .validate_for(&expected)
            .map_err(|_| "AP restart response did not validate")?;
        if exchanges[index]
            .response
            .snapshot
            .as_ref()
            .is_none_or(|s| s.first_failure.is_some())
        {
            return Err("AP restart has a recorded firmware failure");
        }
        let mut encoded = [0; MAX_REQUEST_BYTES];
        let len = encode_request(&expected, &mut encoded)
            .map_err(|_| "cannot reconstruct AP restart request")?;
        expected_tx.extend_from_slice(&encoded[..len]);
    }
    if crate::read_input(&directory.join("requests.jsonl"))? != expected_tx
        || crate::read_input(&directory.join("tx.raw"))? != expected_tx
    {
        return Err("AP restart UART requests disagree");
    }
    let raw = crate::read_input(&directory.join("rx.raw"))?;
    if raw.len() > UART_LIMIT {
        return Err("AP restart raw UART exceeds limit");
    }
    let mut responses = Vec::new();
    for line in raw.split_inclusive(|b| *b == b'\n') {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if !line.ends_with(b"\n") || line.len() > MAX_RESPONSE_BYTES {
            return Err("AP restart raw UART is incomplete");
        }
        responses.push(decode_response(line).map_err(|_| "invalid raw AP restart response")?);
    }
    if responses
        != exchanges
            .iter()
            .map(|exchange| exchange.response)
            .collect::<Vec<_>>()
    {
        return Err("AP restart raw UART responses disagree");
    }
    initial(&exchanges[0].response)?;
    let original = active(&exchanges[1].response, Phase::First, session.nonce, None)?;
    stopped(
        &exchanges[2].response,
        Phase::First,
        session.nonce,
        original,
    )?;
    active(
        &exchanges[3].response,
        Phase::Second,
        session.nonce,
        Some(original),
    )?;
    stopped(
        &exchanges[4].response,
        Phase::Second,
        session.nonce,
        original,
    )?;
    let events: Vec<Event> = lines(&crate::read_input(&directory.join("events.jsonl"))?)?;
    let mut required = vec![(None, EventKind::Status)];
    for phase in PHASES {
        for event in [
            EventKind::ReceiverOpened,
            EventKind::ReceiverReady,
            EventKind::ReceiverAlive,
            EventKind::StartRequested,
            EventKind::Start,
            EventKind::Stop,
            EventKind::ReceiverComplete,
            EventKind::PhaseAdmitted,
        ] {
            required.push((Some(phase), event));
        }
    }
    if events.len() != required.len()
        || events
            .iter()
            .zip(required)
            .enumerate()
            .any(|(index, (actual, (phase, event)))| {
                actual.phase != phase
                    || actual.event != event
                    || actual.sequence != index as u32 + 1
                    || (index > 0 && actual.elapsed_us < events[index - 1].elapsed_us)
            })
    {
        return Err("AP restart phase or readiness event order failed");
    }
    let (first, first_raw) = phase_report(directory, &session, Phase::First)?;
    let (second, second_raw) = phase_report(directory, &session, Phase::Second)?;
    if crate::read_input(&directory.join("receiver-first/request.bin"))?
        != crate::read_input(&directory.join("receiver-second/request.bin"))?
    {
        return Err("AP restart receiver selection changed");
    }
    let first_provenance: wifi_capture::Provenance = serde_json::from_slice(&crate::read_input(
        &directory.join("receiver-first/worker-provenance.json"),
    )?)
    .map_err(|_| "invalid AP restart receiver provenance")?;
    let second_provenance: wifi_capture::Provenance = serde_json::from_slice(&crate::read_input(
        &directory.join("receiver-second/worker-provenance.json"),
    )?)
    .map_err(|_| "invalid AP restart receiver provenance")?;
    if first_provenance != second_provenance {
        return Err("AP restart receiver provenance changed");
    }
    let collateral = collateral(&first_raw, &second_raw, session.nonce)?;
    Ok(Report {
        schema_version: VERSION,
        phases: [first, second],
        uart_commands: 5,
        one_uart_connection: true,
        target_addresses_differ: Phase::First.mac() != Phase::Second.mac(),
        same_ssid: true,
        collateral,
    })
}

pub(crate) fn replay(directory: &Path) -> Result<Report, &'static str> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect AP restart directory")? {
        let entry = entry.map_err(|_| "cannot inspect AP restart entry")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid AP restart filename")?;
        let kind = entry
            .file_type()
            .map_err(|_| "cannot inspect AP restart entry")?;
        if !(kind.is_file()
            || kind.is_dir() && ["receiver-first", "receiver-second"].contains(&name.as_str()))
        {
            return Err("AP restart requires regular artifacts");
        }
        names.insert(name);
    }
    let mut expected: BTreeSet<_> = ROOT_FILES.into_iter().map(str::to_owned).collect();
    expected.extend(["receiver-first".into(), "receiver-second".into()]);
    if names != expected {
        return Err("AP restart inventory is incomplete or unexpected");
    }
    let receipt: Receipt =
        serde_json::from_slice(&crate::read_input(&directory.join("capture-status.json"))?)
            .map_err(|_| "invalid AP restart completion receipt")?;
    if receipt.schema_version != VERSION || !receipt.success || receipt.error.is_some() {
        return Err("AP restart records a failed attempt");
    }
    let actual = artifacts(directory)?;
    if actual.len() != receipt.artifacts.len()
        || actual
            .iter()
            .zip(receipt.artifacts)
            .any(|(actual, saved)| actual.file != saved.file || actual.sha256 != saved.sha256)
    {
        return Err("AP restart artifacts disagree with completion receipt");
    }
    let report = derive_report(directory)?;
    if json(&report)? != crate::read_input(&directory.join("summary.json"))? {
        return Err("AP restart summary disagrees with complete replay");
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kismet_worker::{CredentialFile, Ready, Status, Stop};
    use serde_json::json;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use wifi_protocol::restart::{Snapshot, decode_request, encode_response};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const ORIGINAL: [u8; 6] = [14, 43, 44, 45, 46, 47];
    const OTHER: [u8; 6] = [18, 53, 54, 55, 56, 57];
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "ap-restart-{}-{}",
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
    fn session() -> Session {
        Session {
            schema_version: VERSION,
            nonce: [7, 11, 13, 17, 19, 23, 29, 31],
            first_id: 61,
        }
    }
    fn timing() -> Timing {
        Timing {
            exchange: UART_TIMEOUT,
            ready: Duration::from_secs(1),
            window: Duration::from_millis(2),
            suffix: Duration::from_millis(2),
        }
    }
    fn write_json(path: &Path, value: &impl Serialize) {
        fs::write(path, super::json(value).unwrap()).unwrap();
    }
    fn crc(bytes: &[u8]) -> u32 {
        let mut value = !0_u32;
        for byte in bytes {
            value ^= u32::from(*byte);
            for _ in 0..8 {
                value = (value >> 1) ^ (0xedb88320_u32.wrapping_mul(value & 1));
            }
        }
        !value
    }
    fn packet_capture(nonce: [u8; 8], phase: Phase, mixed: bool, no_target: bool) -> Vec<u8> {
        let mut out = vec![
            0x0a, 0x0d, 0x0d, 0x0a, 28, 0, 0, 0, 0x4d, 0x3c, 0x2b, 0x1a, 1, 0, 0, 0,
        ];
        out.extend_from_slice(&u64::MAX.to_le_bytes());
        out.extend_from_slice(&28_u32.to_le_bytes());
        out.extend_from_slice(&[
            1, 0, 0, 0, 20, 0, 0, 0, 127, 0, 0, 0, 255, 255, 0, 0, 20, 0, 0, 0,
        ]);
        for index in 0..32_u32 {
            let mut target = nonce;
            if no_target {
                target[0] ^= 1;
            }
            let mut frame = vec![128, 0, 0, 0];
            frame.extend_from_slice(&[255; 6]);
            frame.extend_from_slice(&phase.mac());
            frame.extend_from_slice(&phase.mac());
            frame.extend_from_slice(&[0; 10]);
            frame.extend_from_slice(&[100, 0, 1, 0, 0, 22]);
            frame.extend_from_slice(&ssid(target));
            frame.extend_from_slice(&[3, 1, 1]);
            let mut checksum = crc(&frame);
            if mixed && index == 31 {
                checksum ^= 1;
            }
            frame.extend_from_slice(&checksum.to_le_bytes());
            let mut packet = vec![0, 0, 9, 0, 2, 0, 0, 0, 16];
            packet.extend_from_slice(&frame);
            let padded = (packet.len() + 3) & !3;
            let len = 32 + padded;
            out.extend_from_slice(&6_u32.to_le_bytes());
            out.extend_from_slice(&(len as u32).to_le_bytes());
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&index.to_le_bytes());
            out.extend_from_slice(&(packet.len() as u32).to_le_bytes());
            out.extend_from_slice(&(packet.len() as u32).to_le_bytes());
            out.extend_from_slice(&packet);
            out.resize(out.len() + padded - packet.len(), 0);
            out.extend_from_slice(&(len as u32).to_le_bytes());
        }
        out
    }
    fn receiver_fixture(
        directory: &Path,
        phase: Phase,
        nonce: [u8; 8],
        mixed: bool,
        no_target: bool,
    ) -> (ReadyReceipt, wifi_observer::Summary) {
        fs::create_dir(directory).unwrap();
        let raw = packet_capture(nonce, phase, mixed, no_target);
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
        let summary = wifi_observer::analyze(&raw).unwrap();
        fs::write(directory.join("request.bin"), request).unwrap();
        fs::write(directory.join("packets.pcapng"), raw).unwrap();
        fs::write(
            directory.join("transport.stderr"),
            format!("{}{}", ready.line(), status.line()),
        )
        .unwrap();
        write_json(
            &directory.join("worker-provenance.json"),
            &json!({"schema_version":1,"binary_sha256":"b".repeat(64),"build_inputs_sha256":"c".repeat(64),"rustc_version":"rustc 1.90.0 (1159e78c4 2025-09-14)","curl_version":"8.17.0"}),
        );
        write_json(
            &directory.join("transport-status.json"),
            &json!({"schema_version":1,"ssh_exit_code":0,"ssh_signal":null,"timed_out":false,"io_failed":false,"limit_exceeded":false,"worker_status_line":status.line()}),
        );
        write_json(&directory.join("summary.json"), &summary);
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
        (
            ReadyReceipt {
                line: ready.line(),
                local_prefix_bytes: 28,
                local_prefix_sha256: ready.prefix_sha256,
            },
            summary,
        )
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Scenario {
        Good,
        MixedFirst,
        NoTargetFirst,
        ReadyFirstFailure,
        ReadySecondFailure,
        CancelledSecond,
        ReceiverSecondFailure,
        LostFirstStartAck,
        LostSecondStartAck,
        LostFirstStopAck,
        PartialFirstWrite,
        StickyFirstStop,
        WrongSecondPhase,
        WrongSecondNonce,
        WrongSecondOriginal,
        ResetDuringSecondReady,
        BadCleanup,
    }
    struct Wire {
        rx: VecDeque<u8>,
        pending: Vec<u8>,
        requests: Vec<Request>,
        phase: Option<Phase>,
        running: bool,
        original: [u8; 6],
        latch: Option<i32>,
        scenario: Scenario,
        write_fault: bool,
        clock: Rc<Cell<Duration>>,
        fault_hits: Vec<(usize, Operation)>,
    }
    struct Port(Rc<RefCell<Wire>>);
    impl Port {
        fn new(scenario: Scenario) -> Self {
            Self(Rc::new(RefCell::new(Wire {
                rx: VecDeque::new(),
                pending: Vec::new(),
                requests: Vec::new(),
                phase: None,
                running: false,
                original: ORIGINAL,
                latch: None,
                scenario,
                write_fault: false,
                clock: Rc::new(Cell::new(Duration::ZERO)),
                fault_hits: Vec::new(),
            })))
        }
    }
    impl Read for Port {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            let byte = self.0.borrow_mut().rx.pop_front();
            if let Some(byte) = byte {
                bytes[0] = byte;
                Ok(1)
            } else {
                let clock = self.0.borrow().clock.clone();
                clock.set(clock.get() + Duration::from_millis(1));
                Err(std::io::ErrorKind::TimedOut.into())
            }
        }
    }
    impl Write for Port {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut wire = self.0.borrow_mut();
            if wire.write_fault {
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            if wire.scenario == Scenario::PartialFirstWrite
                && wire.requests.len() == 1
                && wire.pending.len() >= 14
            {
                wire.write_fault = true;
                wire.fault_hits.push((1, Operation::StartFirst));
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            let n = bytes.len().min(7);
            wire.pending.extend_from_slice(&bytes[..n]);
            if !wire.pending.ends_with(b"\n") {
                return Ok(n);
            }
            let request = decode_request(&wire.pending).unwrap();
            wire.pending.clear();
            let ordinal = wire.requests.len();
            wire.requests.push(request);
            if matches!(
                request.operation,
                Operation::StartFirst | Operation::StartSecond
            ) {
                wire.phase = Some(if request.operation == Operation::StartFirst {
                    Phase::First
                } else {
                    Phase::Second
                });
                wire.running = true;
                if wire.scenario == Scenario::WrongSecondOriginal
                    && request.operation == Operation::StartSecond
                {
                    wire.original = OTHER;
                }
            }
            if request.operation == Operation::Stop {
                wire.running = false;
                if wire.scenario == Scenario::StickyFirstStop {
                    wire.latch = Some(257);
                }
            }
            let cleanup = ordinal > 4 || request.id == session().first_id + 5;
            let failed = wire.scenario == Scenario::BadCleanup
                && (request.operation == Operation::StartFirst || cleanup);
            if failed {
                wire.latch = Some(257);
            }
            let snapshot = match wire.phase {
                None => Snapshot {
                    state: State::Idle,
                    phase: None,
                    nonce: None,
                    initialized: false,
                    running: false,
                    mac: None,
                    original_mac: None,
                    auto_stopped: false,
                    first_failure: wire.latch,
                },
                Some(mut phase) => {
                    if wire.scenario == Scenario::WrongSecondPhase
                        && request.operation == Operation::StartSecond
                    {
                        phase = Phase::First;
                    }
                    let mut nonce = request.nonce;
                    if wire.scenario == Scenario::WrongSecondNonce
                        && request.operation == Operation::StartSecond
                    {
                        nonce[0] ^= 1;
                    }
                    Snapshot {
                        state: if wire.running {
                            State::Active
                        } else {
                            State::Stopped
                        },
                        phase: Some(phase),
                        nonce: Some(nonce),
                        initialized: true,
                        running: wire.running,
                        mac: Some(if wire.running {
                            phase.mac()
                        } else {
                            wire.original
                        }),
                        original_mac: Some(wire.original),
                        auto_stopped: !wire.running,
                        first_failure: wire.latch,
                    }
                }
            };
            let response = Response::new(
                request.id,
                request.operation,
                if failed { 257 } else { 0 },
                if failed { None } else { Some(snapshot) },
            );
            if wire.scenario == Scenario::LostFirstStartAck && ordinal == 1
                || wire.scenario == Scenario::LostSecondStartAck && ordinal == 3
                || wire.scenario == Scenario::LostFirstStopAck && ordinal == 2
            {
                wire.fault_hits.push((ordinal, request.operation));
                return Ok(n);
            }
            let mut encoded = [0; MAX_RESPONSE_BYTES];
            let len = encode_response(&response, &mut encoded).unwrap();
            wire.rx.extend(&encoded[..len]);
            Ok(n)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    struct FakeReceiver {
        ready: Option<ReadyReceipt>,
        summary: wifi_observer::Summary,
        wire: Rc<RefCell<Wire>>,
        phase: Phase,
    }
    impl PhaseReceiver for FakeReceiver {
        fn poll_ready(&mut self) -> Result<Option<ReadyReceipt>, &'static str> {
            let scenario = self.wire.borrow().scenario;
            if scenario == Scenario::ReadyFirstFailure && self.phase == Phase::First
                || scenario == Scenario::ReadySecondFailure && self.phase == Phase::Second
            {
                return Err("synthetic restart readiness failure");
            }
            Ok(self.ready.take())
        }
        fn alive(&self) -> bool {
            !(self.wire.borrow().scenario == Scenario::CancelledSecond
                && self.phase == Phase::Second)
        }
        fn failed(&mut self) -> bool {
            let wire = self.wire.borrow();
            wire.scenario == Scenario::ReceiverSecondFailure
                && wire.running
                && self.phase == Phase::Second
        }
        fn completed(&mut self) -> bool {
            true
        }
        fn finish(&mut self) -> Result<wifi_observer::Summary, &'static str> {
            Ok(self.summary.clone())
        }
    }
    fn fake_capture(
        directory: &Path,
        scenario: Scenario,
    ) -> (Result<Report, &'static str>, Vec<Request>, Vec<Phase>) {
        let session = session();
        write_json(&directory.join("restart.json"), &session);
        fs::write(directory.join("startup.raw"), b"").unwrap();
        let mut port = Port::new(scenario);
        let mut logs = Logs::with_manual_clock(directory, port.0.borrow().clock.clone()).unwrap();
        let wire = port.0.clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let tracked = calls.clone();
        let mut receivers = Vec::new();
        let result = run_session(
            &mut port,
            directory,
            &session,
            &mut logs,
            &mut receivers,
            |phase, path| {
                tracked.borrow_mut().push(phase);
                let (ready, summary) = receiver_fixture(
                    &path,
                    phase,
                    session.nonce,
                    scenario == Scenario::MixedFirst && phase == Phase::First,
                    scenario == Scenario::NoTargetFirst && phase == Phase::First,
                );
                if scenario == Scenario::ResetDuringSecondReady && phase == Phase::Second {
                    let mut wire = wire.borrow_mut();
                    wire.rx.extend(b" \t \r\n  PRIVATE_RESET_SENTINEL\n");
                    wire.phase = None;
                    wire.running = false;
                }
                Ok(FakeReceiver {
                    ready: Some(ready),
                    summary,
                    wire: wire.clone(),
                    phase,
                })
            },
            timing(),
        );
        let mut connection = Connection {
            schema_version: VERSION,
            baud: 115200,
            uart_open_count: 1,
            control_release_count: 1,
            startup_window_count: 1,
            dtr_deassertion_succeeded: true,
            rts_deassertion_succeeded: true,
            startup_accepted: true,
            ..Connection::default()
        };
        if result.is_err() {
            connection.cleanup_stop_attempted = true;
            connection.cleanup_stop_confirmed = logs
                .exchange(
                    &mut port,
                    Request::new(session.first_id + 5, Operation::Stop, session.nonce),
                    true,
                    timing().exchange,
                )
                .is_ok_and(|r| cleanup_stopped(&r, session.nonce, logs.original));
            logs.event(None, EventKind::CleanupStop).unwrap();
            connection.recovery_required = true;
        }
        let result = finish_capture(directory, &logs, &connection, result);
        let expected_fault = match scenario {
            Scenario::LostFirstStartAck => Some((1, Operation::StartFirst)),
            Scenario::LostSecondStartAck => Some((3, Operation::StartSecond)),
            Scenario::LostFirstStopAck => Some((2, Operation::Stop)),
            Scenario::PartialFirstWrite => Some((1, Operation::StartFirst)),
            _ => None,
        };
        if let Some(expected) = expected_fault {
            assert!(wire.borrow().fault_hits.contains(&expected));
        }
        if scenario == Scenario::PartialFirstWrite {
            assert!(wire.borrow().write_fault);
        }
        let requests = wire.borrow().requests.clone();
        let phases = calls.borrow().clone();
        (result, requests, phases)
    }
    fn rehash(directory: &Path) {
        write_json(
            &directory.join("capture-status.json"),
            &Receipt {
                schema_version: VERSION,
                success: true,
                error: None,
                artifacts: artifacts(directory).unwrap(),
            },
        );
    }

    #[test]
    fn retained_session_uses_five_commands_two_receivers_and_exact_save_replay() {
        let t = Temp::new();
        let (report, requests, phases) = fake_capture(&t.0, Scenario::Good);
        let report = report.unwrap();
        assert_eq!(
            requests.iter().map(|r| r.operation).collect::<Vec<_>>(),
            [
                Operation::Status,
                Operation::StartFirst,
                Operation::Stop,
                Operation::StartSecond,
                Operation::Stop
            ]
        );
        assert!(requests.windows(2).all(|r| r[0].id < r[1].id));
        assert_eq!(phases, PHASES);
        assert_eq!(report.phases[0].witness.matching_beacon_count, 32);
        assert_eq!(report.phases[1].witness.matching_beacon_count, 32);
        assert!(report.same_ssid && report.target_addresses_differ && report.one_uart_connection);
        assert!(matches!(report.collateral, Collateral::Qualified(_)));
        assert_eq!(artifacts(&t.0).unwrap().len() + 1, 27);
        assert!(replay(&t.0).is_ok());
        let output = String::from_utf8(super::json(&report).unwrap()).unwrap();
        assert!(!output.contains("di-ap-"));
        assert!(!output.contains("original_mac"));
    }

    #[test]
    fn presence_does_not_require_profile_qualification_but_a_missing_target_never_opens_b() {
        let t = Temp::new();
        let (report, _, phases) = fake_capture(&t.0, Scenario::MixedFirst);
        let report = report.unwrap();
        assert_eq!(phases, PHASES);
        assert_eq!(report.phases[0].witness.matching_beacon_count, 31);
        assert!(matches!(
            report.collateral,
            Collateral::Unqualified(UnqualifiedReason::CapturesNotEntirelyCheckedTargets)
        ));
        assert!(replay(&t.0).is_ok());
        let t = Temp::new();
        let (result, requests, phases) = fake_capture(&t.0, Scenario::NoTargetFirst);
        assert!(result.is_err());
        assert_eq!(phases, [Phase::First]);
        assert!(
            !requests
                .iter()
                .any(|r| r.operation == Operation::StartSecond)
        );
    }

    #[test]
    fn readiness_cancel_reset_and_failure_latch_suppress_second_start() {
        for scenario in [
            Scenario::ReadyFirstFailure,
            Scenario::ReadySecondFailure,
            Scenario::CancelledSecond,
            Scenario::ResetDuringSecondReady,
            Scenario::StickyFirstStop,
        ] {
            let t = Temp::new();
            let (result, requests, _) = fake_capture(&t.0, scenario);
            assert!(result.is_err());
            let expected_error = match scenario {
                Scenario::ReadyFirstFailure | Scenario::ReadySecondFailure => {
                    "synthetic restart readiness failure"
                }
                Scenario::CancelledSecond => "AP restart receiver ended before Start",
                Scenario::ResetDuringSecondReady => "unexpected AP restart UART output",
                Scenario::StickyFirstStop => "AP restart Stop did not restore the original address",
                _ => unreachable!(),
            };
            assert_eq!(result.as_ref().err().copied(), Some(expected_error));
            assert!(
                !requests
                    .iter()
                    .any(|r| r.operation == Operation::StartSecond)
            );
            let receipt: Receipt =
                serde_json::from_slice(&fs::read(t.0.join("capture-status.json")).unwrap())
                    .unwrap();
            assert!(!receipt.success);
            assert!(replay(&t.0).is_err());
            if scenario == Scenario::ReadyFirstFailure {
                assert!(
                    !requests
                        .iter()
                        .any(|r| r.operation == Operation::StartFirst)
                );
            }
        }
    }

    #[test]
    fn lost_acks_partial_uart_wrong_phase_nonce_original_and_cleanup_never_resume() {
        for scenario in [
            Scenario::LostFirstStartAck,
            Scenario::LostSecondStartAck,
            Scenario::LostFirstStopAck,
            Scenario::PartialFirstWrite,
            Scenario::WrongSecondPhase,
            Scenario::WrongSecondNonce,
            Scenario::WrongSecondOriginal,
            Scenario::BadCleanup,
            Scenario::ReceiverSecondFailure,
        ] {
            let t = Temp::new();
            let (result, requests, _) = fake_capture(&t.0, scenario);
            assert!(result.is_err());
            let expected_error = match scenario {
                Scenario::LostFirstStartAck
                | Scenario::LostSecondStartAck
                | Scenario::LostFirstStopAck => "AP restart UART response timed out",
                Scenario::PartialFirstWrite => "cannot write AP restart UART",
                Scenario::WrongSecondPhase | Scenario::WrongSecondNonce | Scenario::BadCleanup => {
                    "AP restart operation did not validate"
                }
                Scenario::WrongSecondOriginal => "AP restart active state did not validate",
                Scenario::ReceiverSecondFailure => "AP restart receiver failed while AP active",
                _ => unreachable!(),
            };
            assert_eq!(result.as_ref().err().copied(), Some(expected_error));
            let expected_prefix: &[Operation] = match scenario {
                Scenario::LostFirstStartAck | Scenario::BadCleanup => {
                    &[Operation::Status, Operation::StartFirst]
                }
                Scenario::LostFirstStopAck => {
                    &[Operation::Status, Operation::StartFirst, Operation::Stop]
                }
                Scenario::PartialFirstWrite => &[Operation::Status],
                _ => &[
                    Operation::Status,
                    Operation::StartFirst,
                    Operation::Stop,
                    Operation::StartSecond,
                ],
            };
            assert!(requests.len() >= expected_prefix.len());
            assert_eq!(
                requests[..expected_prefix.len()]
                    .iter()
                    .map(|r| r.operation)
                    .collect::<Vec<_>>(),
                expected_prefix
            );
            assert!(
                requests
                    .iter()
                    .filter(|r| r.operation == Operation::StartFirst)
                    .count()
                    <= 1
            );
            assert!(
                requests
                    .iter()
                    .filter(|r| r.operation == Operation::StartSecond)
                    .count()
                    <= 1
            );
            if [
                Scenario::LostFirstStartAck,
                Scenario::LostFirstStopAck,
                Scenario::PartialFirstWrite,
                Scenario::BadCleanup,
            ]
            .contains(&scenario)
            {
                assert!(
                    !requests
                        .iter()
                        .any(|r| r.operation == Operation::StartSecond)
                );
            }
            let connection: Connection =
                serde_json::from_slice(&fs::read(t.0.join("connection.json")).unwrap()).unwrap();
            assert!(connection.cleanup_stop_attempted && connection.recovery_required);
            if [
                Scenario::BadCleanup,
                Scenario::PartialFirstWrite,
                Scenario::WrongSecondOriginal,
            ]
            .contains(&scenario)
            {
                assert!(!connection.cleanup_stop_confirmed);
            }
            assert!(replay(&t.0).is_err());
        }
    }

    #[test]
    fn rehashed_session_semantic_corruption_is_rejected() {
        for mutation in 0..7 {
            let t = Temp::new();
            fake_capture(&t.0, Scenario::Good).0.unwrap();
            match mutation {
                0 => {
                    let path = t.0.join("connection.json");
                    let mut c: Connection =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    c.uart_open_count = 2;
                    write_json(&path, &c);
                }
                1 => {
                    let path = t.0.join("events.jsonl");
                    let mut events: Vec<Event> = lines(&fs::read(&path).unwrap()).unwrap();
                    events.swap(8, 9);
                    let bytes: Vec<_> = events
                        .iter()
                        .flat_map(|event| {
                            let mut bytes = serde_json::to_vec(event).unwrap();
                            bytes.push(b'\n');
                            bytes
                        })
                        .collect();
                    fs::write(path, bytes).unwrap();
                }
                2 => {
                    let path = t.0.join("ready-second.json");
                    let mut ready: ReadyReceipt =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    ready.local_prefix_sha256 = "f".repeat(64);
                    write_json(&path, &ready);
                }
                3 => {
                    fs::write(t.0.join("rx.raw"), b"PRIVATE_RESET_SENTINEL\n").unwrap();
                }
                4 => {
                    let path = t.0.join("exchanges.jsonl");
                    let mut exchanges: Vec<Exchange> = lines(&fs::read(&path).unwrap()).unwrap();
                    exchanges[3].request = Request::new(
                        session().first_id + 3,
                        Operation::StartFirst,
                        session().nonce,
                    );
                    let bytes: Vec<_> = exchanges
                        .iter()
                        .flat_map(|e| {
                            let mut b = serde_json::to_vec(e).unwrap();
                            b.push(b'\n');
                            b
                        })
                        .collect();
                    fs::write(path, bytes).unwrap();
                }
                5 => {
                    fs::write(t.0.join("cleanup.jsonl"), b"{}\n").unwrap();
                }
                6 => {
                    let mut report = derive_report(&t.0).unwrap();
                    report.same_ssid = false;
                    write_json(&t.0.join("summary.json"), &report);
                }
                _ => unreachable!(),
            };
            rehash(&t.0);
            assert!(replay(&t.0).is_err());
        }
    }

    #[test]
    fn malformed_collateral_is_an_error_instead_of_an_unqualified_result() {
        assert!(collateral(b"bad", b"bad", session().nonce).is_err());
    }

    #[test]
    fn expired_readiness_deadline_never_consumes_a_start_attempt() {
        let t = Temp::new();
        let session = session();
        let mut port = Port::new(Scenario::Good);
        let mut logs = Logs::with_manual_clock(&t.0, port.0.borrow().clock.clone()).unwrap();
        let wire = port.0.clone();
        let mut receivers = Vec::new();
        let mut timing = timing();
        timing.ready = Duration::ZERO;
        let result = run_session(
            &mut port,
            &t.0,
            &session,
            &mut logs,
            &mut receivers,
            |phase, path| {
                let (ready, summary) = receiver_fixture(&path, phase, session.nonce, false, false);
                Ok(FakeReceiver {
                    ready: Some(ready),
                    summary,
                    wire: wire.clone(),
                    phase,
                })
            },
            timing,
        );
        assert_eq!(
            result.err(),
            Some("AP restart receiver readiness timed out")
        );
        assert_eq!(
            wire.borrow()
                .requests
                .iter()
                .map(|r| r.operation)
                .collect::<Vec<_>>(),
            [Operation::Status]
        );
    }

    struct ModelDelay {
        port: Port,
        delay: Duration,
        delayed: bool,
        last_byte: bool,
    }
    impl Write for ModelDelay {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.port.write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.port.flush()
        }
    }
    impl Read for ModelDelay {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            if !self.delayed && (!self.last_byte || self.port.0.borrow().rx.front() == Some(&b'\n'))
            {
                assert!(!self.port.0.borrow().rx.is_empty());
                let clock = self.port.0.borrow().clock.clone();
                clock.set(clock.get() + self.delay);
                self.delayed = true;
            }
            self.port.read(bytes)
        }
    }
    #[test]
    fn modeled_transport_delay_checks_the_exact_exchange_boundary() {
        for last_byte in [false, true] {
            for delay in [
                UART_TIMEOUT - Duration::from_millis(1),
                UART_TIMEOUT,
                UART_TIMEOUT + Duration::from_millis(1),
            ] {
                let t = Temp::new();
                let port = Port::new(Scenario::Good);
                let wire = port.0.clone();
                let clock = wire.borrow().clock.clone();
                let mut logs = Logs::with_manual_clock(&t.0, clock.clone()).unwrap();
                let mut port = ModelDelay {
                    port,
                    delay,
                    delayed: false,
                    last_byte,
                };
                let result = logs.exchange(
                    &mut port,
                    Request::new(session().first_id, Operation::Status, session().nonce),
                    false,
                    UART_TIMEOUT,
                );
                assert!(port.delayed);
                assert_eq!(clock.get(), delay);
                assert_eq!(wire.borrow().requests.len(), 1);
                assert_eq!(wire.borrow().requests[0].operation, Operation::Status);
                if delay >= UART_TIMEOUT {
                    assert_eq!(result.err(), Some("AP restart UART response timed out"));
                    assert!(fs::read(t.0.join("exchanges.jsonl")).unwrap().is_empty());
                    if last_byte {
                        let raw = fs::read(t.0.join("rx.raw")).unwrap();
                        assert_eq!(raw.last(), Some(&b'\n'));
                        assert!(decode_response(&raw).is_ok());
                        assert!(wire.borrow().rx.is_empty());
                    } else {
                        assert!(!wire.borrow().rx.is_empty());
                    }
                } else {
                    assert!(result.is_ok());
                    assert!(wire.borrow().rx.is_empty());
                }
            }
        }
    }

    struct QueuedRealDelay {
        rx: VecDeque<u8>,
        tx: Vec<u8>,
        delayed: bool,
    }
    impl Write for QueuedRealDelay {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.tx.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl Read for QueuedRealDelay {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            if !self.delayed {
                assert!(!self.rx.is_empty());
                self.delayed = true;
                thread::sleep(Duration::from_millis(150));
            }
            if let Some(byte) = self.rx.pop_front() {
                bytes[0] = byte;
                Ok(1)
            } else {
                Err(std::io::ErrorKind::TimedOut.into())
            }
        }
    }
    #[test]
    fn real_clock_preemption_still_times_out_a_prequeued_response() {
        for short in [true, false] {
            let t = Temp::new();
            let mut logs = Logs::new(&t.0).unwrap();
            let request = Request::new(session().first_id, Operation::Status, session().nonce);
            let response = Response::new(
                request.id,
                request.operation,
                0,
                Some(Snapshot {
                    state: State::Idle,
                    phase: None,
                    nonce: None,
                    initialized: false,
                    running: false,
                    mac: None,
                    original_mac: None,
                    auto_stopped: false,
                    first_failure: None,
                }),
            );
            let mut encoded = [0; MAX_RESPONSE_BYTES];
            let len = encode_response(&response, &mut encoded).unwrap();
            let mut port = QueuedRealDelay {
                rx: encoded[..len].iter().copied().collect(),
                tx: Vec::new(),
                delayed: false,
            };
            let result = logs.exchange(
                &mut port,
                request,
                false,
                if short {
                    Duration::from_millis(100)
                } else {
                    UART_TIMEOUT
                },
            );
            assert_eq!(decode_request(&port.tx).unwrap(), request);
            assert!(port.delayed);
            if short {
                assert_eq!(result.err(), Some("AP restart UART response timed out"));
                assert!(!port.rx.is_empty());
            } else {
                assert!(result.is_ok());
                assert!(port.rx.is_empty());
            }
        }
    }
}
