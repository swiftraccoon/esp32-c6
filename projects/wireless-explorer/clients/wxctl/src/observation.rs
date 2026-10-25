//! Live capture and strict offline derivation use the same stream consumer.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use wireless_protocol::capture::{self, Details, Event, Packet, Radio, Record, SNAPLEN};
use wireless_protocol::{Command, Request, encode};

const MAX_RAW: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Window {
    pub(crate) id: u32,
    pub(crate) radio: Radio,
    pub(crate) channel: u8,
    pub(crate) millis: u32,
}

impl Window {
    fn request(&self) -> Request {
        Request::new(
            self.id,
            Command::Observe {
                radio: self.radio,
                channel: self.channel,
                millis: self.millis,
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u8,
    port: String,
    baud: u32,
    host_started_unix_ms: u128,
    windows: Vec<Window>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Activity {
    packets: u32,
    payload_bytes: u64,
    rssi_min_dbm: Option<i8>,
    rssi_max_dbm: Option<i8>,
    packet_types: BTreeMap<String, u32>,
    ble_ad_types: BTreeMap<u8, u32>,
    ble_manufacturers: BTreeMap<u16, u32>,
    ble_incomplete_reports: u32,
    ble_malformed_ad_data: u32,
    distinct_reported_addresses: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Summary {
    window: Window,
    start_us: u64,
    listening_us: u64,
    stopped_us: u64,
    end_us: u64,
    sdk_reports: u32,
    recorder_dropped: u32,
    invalid_reports: u32,
    truncated: u32,
    activity: Activity,
}

#[derive(Debug, Serialize)]
struct SavedPacket<'a> {
    metadata: Packet,
    payload_hex: &'a str,
}

struct Consumer<E, P> {
    window: Window,
    events: E,
    packets: P,
    line: Vec<u8>,
    start: Option<u64>,
    listening: Option<u64>,
    pending: Option<(Packet, String)>,
    last_seq: Option<u32>,
    last_packet_time: u64,
    truncations: u32,
    addresses: BTreeSet<(u8, [u8; 6])>,
    activity: Activity,
    summary: Option<Summary>,
}

impl<E: Write, P: Write> Consumer<E, P> {
    fn new(window: Window, events: E, packets: P) -> Self {
        Self {
            window,
            events,
            packets,
            line: Vec::new(),
            start: None,
            listening: None,
            pending: None,
            last_seq: None,
            last_packet_time: 0,
            truncations: 0,
            addresses: BTreeSet::new(),
            activity: Activity::default(),
            summary: None,
        }
    }

    fn feed(&mut self, bytes: &[u8]) -> Result<()> {
        for &byte in bytes {
            if byte == b'\n' {
                ensure!(!self.line.is_empty(), "empty stream line");
                let record = capture::decode(&self.line)?;
                self.accept(&record)?;
                serde_json::to_writer(&mut self.events, &record)?;
                self.events.write_all(b"\n")?;
                self.line.clear();
            } else {
                ensure!(self.line.len() < capture::MAX_LINE, "oversized stream line");
                self.line.push(byte);
            }
        }
        Ok(())
    }

    fn accept(&mut self, record: &Record) -> Result<()> {
        ensure!(record.id == self.window.id, "stream request ID mismatch");
        ensure!(self.summary.is_none(), "data after End");
        match &record.event {
            Event::Start {
                radio,
                channel,
                millis,
                at_us,
                snaplen,
            } => {
                ensure!(self.start.is_none(), "duplicate Start");
                ensure!(
                    *radio == self.window.radio
                        && *channel == self.window.channel
                        && *millis == self.window.millis,
                    "window settings mismatch"
                );
                ensure!(*snaplen == SNAPLEN as u16, "unexpected snapshot cap");
                ensure!(
                    radio.valid_window(*channel, *millis),
                    "invalid window settings"
                );
                self.start = Some(*at_us);
            }
            Event::Listening { at_us } => {
                ensure!(
                    self.start.is_some_and(|start| *at_us >= start) && self.listening.is_none(),
                    "invalid Listening transition"
                );
                self.listening = Some(*at_us);
            }
            Event::Packet(packet) => {
                ensure!(
                    self.listening.is_some() && self.pending.is_none(),
                    "packet before Listening or incomplete previous packet"
                );
                ensure!(
                    packet.details.radio() == self.window.radio
                        && packet.channel == self.window.channel,
                    "packet radio/channel mismatch"
                );
                ensure!(
                    self.last_seq.is_none_or(|last| packet.seq > last),
                    "non-increasing packet sequence"
                );
                ensure!(
                    packet.at_us >= self.start.unwrap_or(u64::MAX)
                        && packet.at_us >= self.last_packet_time,
                    "packet clock went backwards"
                );
                let metadata_only = matches!(packet.details, Details::Wifi { kind: 3, .. });
                let expected = if metadata_only {
                    0
                } else {
                    usize::from(packet.original_len).min(SNAPLEN)
                };
                ensure!(
                    usize::from(packet.captured_len) == expected,
                    "incorrect packet length/truncation"
                );
                match packet.details {
                    Details::Ble { .. } => {
                        ensure!(packet.original_len <= 251, "oversized BLE report")
                    }
                    Details::Ieee802154 { .. } => {
                        ensure!(packet.original_len <= 125, "oversized 802.15.4 frame")
                    }
                    Details::Wifi { kind, .. } => ensure!(kind <= 3, "unknown Wi-Fi packet type"),
                }
                self.last_seq = Some(packet.seq);
                self.last_packet_time = packet.at_us;
                if usize::from(packet.original_len) > SNAPLEN {
                    self.truncations += 1;
                }
                self.pending = Some((*packet, String::new()));
                self.finish_packet()?;
            }
            Event::Chunk { seq, offset, data } => {
                let (packet, payload) = self.pending.as_mut().context("chunk without packet")?;
                ensure!(
                    *seq == packet.seq && usize::from(*offset) * 2 == payload.len(),
                    "chunk sequence/offset mismatch"
                );
                ensure!(
                    !data.is_empty()
                        && data.len() % 2 == 0
                        && data
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                    "invalid hexadecimal chunk"
                );
                ensure!(
                    payload.len() + data.len() <= usize::from(packet.captured_len) * 2,
                    "payload exceeds packet size"
                );
                payload.push_str(data);
                self.finish_packet()?;
            }
            Event::End {
                at_us,
                stopped_us,
                seen,
                emitted,
                dropped,
                invalid,
                truncated,
                error,
            } => {
                ensure!(*error == 0, "device capture error {error}");
                let listening = self.listening.context("End without Listening")?;
                ensure!(self.pending.is_none(), "End with incomplete packet");
                ensure!(
                    *emitted == self.activity.packets && *truncated == self.truncations,
                    "emission/truncation count mismatch"
                );
                ensure!(
                    u64::from(*emitted) + u64::from(*dropped) + u64::from(*invalid)
                        == u64::from(*seen),
                    "report conservation failure"
                );
                ensure!(
                    self.last_seq.is_none_or(|last| last < *seen),
                    "sequence exceeds report count"
                );
                ensure!(
                    *stopped_us >= listening
                        && stopped_us - listening >= u64::from(self.window.millis) * 1000,
                    "short listening window"
                );
                ensure!(
                    *stopped_us >= self.last_packet_time && at_us >= stopped_us,
                    "invalid stop/end timestamp"
                );
                self.activity.distinct_reported_addresses = self.addresses.len();
                self.summary = Some(Summary {
                    window: self.window.clone(),
                    start_us: self.start.context("missing Start")?,
                    listening_us: listening,
                    stopped_us: *stopped_us,
                    end_us: *at_us,
                    sdk_reports: *seen,
                    recorder_dropped: *dropped,
                    invalid_reports: *invalid,
                    truncated: *truncated,
                    activity: self.activity.clone(),
                });
            }
        }
        Ok(())
    }

    fn finish_packet(&mut self) -> Result<()> {
        if self
            .pending
            .as_ref()
            .is_none_or(|(p, bytes)| bytes.len() != usize::from(p.captured_len) * 2)
        {
            return Ok(());
        }
        let (packet, payload) = self.pending.take().context("pending packet disappeared")?;
        serde_json::to_writer(
            &mut self.packets,
            &SavedPacket {
                metadata: packet,
                payload_hex: &payload,
            },
        )?;
        self.packets.write_all(b"\n")?;
        self.activity.packets += 1;
        self.activity.payload_bytes += u64::from(packet.captured_len);
        if packet.rssi != 127 {
            self.activity.rssi_min_dbm = Some(
                self.activity
                    .rssi_min_dbm
                    .map_or(packet.rssi, |v| v.min(packet.rssi)),
            );
            self.activity.rssi_max_dbm = Some(
                self.activity
                    .rssi_max_dbm
                    .map_or(packet.rssi, |v| v.max(packet.rssi)),
            );
        }
        let bytes = unhex(&payload)?;
        let kind = match packet.details {
            Details::Wifi { kind, .. } => {
                if bytes.len() >= 16 && kind != 3 {
                    let mut address = [0; 6];
                    address.copy_from_slice(&bytes[10..16]);
                    self.addresses.insert((0, address));
                }
                format!("wifi_sdk_{kind}")
            }
            Details::Ble {
                address,
                address_type,
                event_type,
                data_status,
                ..
            } => {
                if address_type != 255 {
                    self.addresses.insert((address_type, address));
                }
                if data_status == 0 {
                    self.advertisement(&bytes);
                } else {
                    self.activity.ble_incomplete_reports += 1;
                }
                format!("ble_event_{event_type}")
            }
            Details::Ieee802154 { .. } => bytes.first().map_or_else(
                || "ieee802154_empty".into(),
                |b| format!("ieee802154_type_{}", b & 7),
            ),
        };
        *self.activity.packet_types.entry(kind).or_default() += 1;
        Ok(())
    }

    fn advertisement(&mut self, mut bytes: &[u8]) {
        while let Some((&len, remaining)) = bytes.split_first() {
            if len == 0 {
                return;
            }
            let len = usize::from(len);
            if len > remaining.len() {
                self.activity.ble_malformed_ad_data += 1;
                return;
            }
            let field = &remaining[..len];
            *self.activity.ble_ad_types.entry(field[0]).or_default() += 1;
            if field[0] == 0xff && field.len() >= 3 {
                *self
                    .activity
                    .ble_manufacturers
                    .entry(u16::from_le_bytes([field[1], field[2]]))
                    .or_default() += 1;
            }
            bytes = &remaining[len..];
        }
    }

    fn finish(self) -> Result<Summary> {
        ensure!(self.line.is_empty(), "trailing partial stream line");
        self.summary.context("missing End: capture is incomplete")
    }
}

fn unhex(text: &str) -> Result<Vec<u8>> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let nibble = |b: u8| {
                char::from(b)
                    .to_digit(16)
                    .context("invalid hexadecimal digit")
            };
            Ok(((nibble(pair[0])? << 4) | nibble(pair[1])?) as u8)
        })
        .collect()
}

pub(crate) fn plan(
    radios: &[Radio],
    wifi: &[u8],
    ieee: &[u8],
    millis: u32,
    cycles: u16,
) -> Result<Vec<Window>> {
    ensure!(
        (1..=100).contains(&cycles) && !radios.is_empty(),
        "invalid cycle/radio selection"
    );
    let mut windows = Vec::new();
    for _ in 0..cycles {
        for &radio in radios {
            let channels = match radio {
                Radio::Wifi => wifi,
                Radio::Ble => &[0],
                Radio::Ieee802154 => ieee,
            };
            ensure!(!channels.is_empty(), "empty channel selection");
            let mut unique = BTreeSet::new();
            for &channel in channels {
                ensure!(
                    radio.valid_window(channel, millis) && unique.insert(channel),
                    "invalid/duplicate {radio:?} channel or duration"
                );
                windows.push(Window {
                    id: windows.len() as u32 + 1,
                    radio,
                    channel,
                    millis,
                });
            }
        }
    }
    ensure!(windows.len() <= 3000, "too many windows");
    Ok(windows)
}

pub(crate) fn capture(
    port: &str,
    baud: u32,
    output: &Path,
    windows: Vec<Window>,
) -> Result<Vec<Summary>> {
    fs::create_dir(output).context("capture directory must be new and its parent must exist")?;
    let result = collect(port, baud, output, windows);
    if let Err(error) = &result {
        fs::write(output.join("failure.txt"), format!("{error:#}\n"))?;
    }
    result
}

fn collect(port: &str, baud: u32, output: &Path, windows: Vec<Window>) -> Result<Vec<Summary>> {
    let manifest = Manifest {
        schema: 1,
        port: port.into(),
        baud,
        host_started_unix_ms: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        windows,
    };
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    let mut serial = serialport::new(port, baud)
        .timeout(Duration::from_millis(100))
        .dtr_on_open(false)
        .open()?;
    serial.write_data_terminal_ready(false)?;
    serial.write_request_to_send(false)?;
    let mut startup = File::create(output.join("startup.raw"))?;
    let mut buffer = [0; 4096];
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut startup_bytes = 0;
    while Instant::now() < deadline {
        let n = read_serial(serial.as_mut(), &mut buffer)?;
        startup.write_all(&buffer[..n])?;
        startup_bytes += n;
        ensure!(startup_bytes <= 65536, "excessive startup output");
    }
    startup.sync_all()?;
    let mut summaries = Vec::new();
    for (index, window) in manifest.windows.iter().enumerate() {
        let directory = output.join(format!("window-{index:04}"));
        fs::create_dir(&directory)?;
        let mut wire = [0; wireless_protocol::MAX_LINE_BYTES + 1];
        let len = encode(&window.request(), &mut wire)?;
        fs::write(directory.join("request.frame"), &wire[..len])?;
        let mut raw = File::create(directory.join("serial.raw"))?;
        let mut events = File::create(directory.join("events.jsonl"))?;
        let mut packets = File::create(directory.join("packets.jsonl"))?;
        let mut consumer = Consumer::new(window.clone(), &mut events, &mut packets);
        serial.write_all(&wire[..len])?;
        let deadline = Instant::now()
            + Duration::from_millis(u64::from(window.millis))
            + Duration::from_secs(20);
        let mut total = 0;
        let mut ended = None;
        loop {
            ensure!(
                Instant::now() < deadline,
                "capture deadline expired at window {index}"
            );
            let n = read_serial(serial.as_mut(), &mut buffer)?;
            raw.write_all(&buffer[..n])?;
            total += n as u64;
            ensure!(total <= MAX_RAW, "capture byte cap exceeded");
            consumer.feed(&buffer[..n])?;
            if consumer.summary.is_some() && ended.is_none() {
                ended = Some(Instant::now());
            }
            if ended.is_some_and(|at: Instant| at.elapsed() >= Duration::from_millis(150)) {
                break;
            }
        }
        let summary = consumer.finish()?;
        raw.sync_all()?;
        events.sync_all()?;
        packets.sync_all()?;
        fs::write(
            directory.join("summary.json"),
            serde_json::to_vec_pretty(&summary)?,
        )?;
        eprintln!(
            "{}/{} {:?} channel {}: {} reports, {} logged, {} dropped",
            index + 1,
            manifest.windows.len(),
            window.radio,
            window.channel,
            summary.sdk_reports,
            summary.activity.packets,
            summary.recorder_dropped
        );
        summaries.push(summary);
    }
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&summaries)?,
    )?;
    ensure!(replay(output)? == summaries, "live/replay summary mismatch");
    Ok(summaries)
}

fn read_serial(serial: &mut dyn serialport::SerialPort, buffer: &mut [u8]) -> Result<usize> {
    match serial.read(buffer) {
        Ok(n) => Ok(n),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
            ) =>
        {
            Ok(0)
        }
        Err(error) => Err(error.into()),
    }
}

struct Compare<R>(R);
impl<R: Read> Write for Compare<R> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut expected = vec![0; bytes.len()];
        self.0.read_exact(&mut expected)?;
        if bytes != expected {
            return Err(std::io::Error::other("saved JSONL differs from raw stream"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<R: Read> Compare<R> {
    fn finish(&mut self) -> Result<()> {
        ensure!(self.0.read(&mut [0])? == 0, "extra saved JSONL data");
        Ok(())
    }
}

fn ordinary_file(path: &Path) -> Result<File> {
    ensure!(
        fs::symlink_metadata(path)?.is_file(),
        "not a regular file: {}",
        path.display()
    );
    Ok(File::open(path)?)
}

fn bounded_file(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    ordinary_file(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "file exceeds byte limit: {}",
        path.display()
    );
    Ok(bytes)
}

fn json_file<T: serde::de::DeserializeOwned>(path: &Path, limit: u64) -> Result<T> {
    Ok(serde_json::from_slice(&bounded_file(path, limit)?)?)
}

fn inventory(path: &Path, names: BTreeSet<String>) -> Result<()> {
    let actual: BTreeSet<String> = fs::read_dir(path)?
        .map(|entry| {
            entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("non-UTF8 capture filename"))
        })
        .collect::<Result<_>>()?;
    ensure!(
        actual == names,
        "capture file inventory mismatch: {}",
        path.display()
    );
    Ok(())
}

pub(crate) fn replay(output: &Path) -> Result<Vec<Summary>> {
    ensure!(
        fs::symlink_metadata(output)?.is_dir(),
        "capture root is not an ordinary directory"
    );
    match fs::symlink_metadata(output.join("failure.txt")) {
        Ok(_) => bail!("capture retains failure marker"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let manifest: Manifest = json_file(&output.join("manifest.json"), 1024 * 1024)?;
    ensure!(
        manifest.schema == 1 && !manifest.windows.is_empty() && manifest.windows.len() <= 3000,
        "invalid capture manifest"
    );
    bounded_file(&output.join("startup.raw"), 65536)?;
    let mut names = BTreeSet::from([
        "manifest.json".into(),
        "summary.json".into(),
        "startup.raw".into(),
    ]);
    names.extend((0..manifest.windows.len()).map(|index| format!("window-{index:04}")));
    inventory(output, names)?;
    let mut summaries = Vec::new();
    let mut last_end = None;
    for (index, window) in manifest.windows.into_iter().enumerate() {
        ensure!(
            window.id == index as u32 + 1
                && window.radio.valid_window(window.channel, window.millis),
            "invalid manifest window"
        );
        let directory = output.join(format!("window-{index:04}"));
        ensure!(
            fs::symlink_metadata(&directory)?.is_dir(),
            "window directory is a symlink/non-directory"
        );
        inventory(
            &directory,
            [
                "request.frame",
                "serial.raw",
                "events.jsonl",
                "packets.jsonl",
                "summary.json",
            ]
            .map(String::from)
            .into(),
        )?;
        let request = bounded_file(&directory.join("request.frame"), 513)?;
        ensure!(
            super::transcript::request_frame(&request)? == window.request(),
            "saved request differs from plan"
        );
        let mut events = Compare(BufReader::new(ordinary_file(
            &directory.join("events.jsonl"),
        )?));
        let mut packets = Compare(BufReader::new(ordinary_file(
            &directory.join("packets.jsonl"),
        )?));
        let mut consumer = Consumer::new(window, &mut events, &mut packets);
        let mut raw = BufReader::new(ordinary_file(&directory.join("serial.raw"))?);
        let mut total = 0;
        loop {
            let bytes = raw.fill_buf()?;
            if bytes.is_empty() {
                break;
            }
            total += bytes.len() as u64;
            ensure!(total <= MAX_RAW, "raw byte limit exceeded");
            consumer.feed(bytes)?;
            let len = bytes.len();
            raw.consume(len);
        }
        let summary = consumer.finish()?;
        events.finish()?;
        packets.finish()?;
        ensure!(
            last_end.is_none_or(|end| summary.start_us >= end),
            "reset/overlap between windows"
        );
        last_end = Some(summary.end_us);
        let saved: Summary = json_file(&directory.join("summary.json"), 65536)?;
        ensure!(summary == saved, "window summary differs from raw stream");
        summaries.push(summary);
    }
    let saved: Vec<Summary> = json_file(&output.join("summary.json"), 16 * 1024 * 1024)?;
    ensure!(
        summaries == saved,
        "session summary differs from raw stream"
    );
    Ok(summaries)
}

#[cfg(test)]
mod tests;
