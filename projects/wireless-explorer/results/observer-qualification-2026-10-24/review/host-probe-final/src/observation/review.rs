use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn sample() -> (Window, Vec<Record>) {
    let window = Window { id: 1, radio: Radio::Ieee802154, channel: 15, millis: 100 };
    let packet = Packet {
        seq: 0, at_us: 150, channel: 15, rssi: -42,
        original_len: 4, captured_len: 4,
        details: Details::Ieee802154 { lqi: 200, sdk_time: 99 },
    };
    let records = vec![
        Record::new(1, Event::Start { radio: window.radio, channel: 15, millis: 100, at_us: 100, snaplen: SNAPLEN as u16 }),
        Record::new(1, Event::Listening { at_us: 200 }),
        Record::new(1, Event::Packet(packet)),
        Record::new(1, Event::Chunk { seq: 0, offset: 0, data: "01020304".try_into().unwrap() }),
        Record::new(1, Event::End { at_us: 100_300, stopped_us: 100_200, seen: 1, emitted: 1, dropped: 0, invalid: 0, truncated: 0, error: 0 }),
    ];
    (window, records)
}

fn wire(records: &[Record]) -> Vec<u8> {
    records.iter().flat_map(|record| {
        let mut out = [0; capture::MAX_LINE + 1];
        let len = capture::encode(record, &mut out).unwrap();
        out[..len].to_vec()
    }).collect()
}

fn check(window: Window, raw: &[u8], chunk: usize) -> Result<Summary> {
    let mut consumer = Consumer::new(window, std::io::sink(), std::io::sink());
    for bytes in raw.chunks(chunk) { consumer.feed(bytes)?; }
    consumer.finish()
}

#[test]
fn independent_byte_fragmentation_and_setup_timestamp_positive_controls() {
    let (window, records) = sample();
    for chunk in [1, 2, 7, 1024, 10000] {
        let summary = check(window.clone(), &wire(&records), chunk).unwrap();
        assert_eq!(summary.activity.packets, 1);
        assert_eq!(summary.activity.payload_bytes, 4);
        assert_eq!(summary.activity.rssi_min_dbm, Some(-42));
    }
    // Reports can arrive during radio setup and must retain that earlier clock.
    assert!(matches!(records[2].event, Event::Packet(Packet { at_us: 150, .. })));
}

#[test]
fn independent_transition_counter_length_and_timestamp_controls() {
    let (window, original) = sample();
    let mut negatives = Vec::new();
    for name in ["id", "channel", "radio", "captured", "original", "before_start", "after_stop", "sequence"] {
        let mut changed = original.clone();
        if name == "id" { changed[2].id = 77; }
        let Event::Packet(packet) = &mut changed[2].event else { unreachable!() };
        match name {
            "id" => {}, "channel" => packet.channel = 26,
            "radio" => packet.details = Details::Wifi { kind: 0, rx_state: 0, sdk_time: 0, phy: 0, rate: 0 },
            "captured" => packet.captured_len = 3,
            "original" => packet.original_len = 126,
            "before_start" => packet.at_us = 99,
            "after_stop" => packet.at_us = 100_201,
            "sequence" => packet.seq = 1,
            _ => unreachable!(),
        }
        negatives.push((name.to_string(), wire(&changed)));
    }
    for name in ["seen", "emitted", "dropped", "invalid", "truncated", "error", "short", "end_before_stop"] {
        let mut changed = original.clone();
        let Event::End { at_us, stopped_us, seen, emitted, dropped, invalid, truncated, error } = &mut changed[4].event else { unreachable!() };
        match name {
            "seen" => *seen = u32::MAX,
            "emitted" => *emitted = 2,
            "dropped" => *dropped = 1,
            "invalid" => *invalid = 1,
            "truncated" => *truncated = 1,
            "error" => *error = -7,
            "short" => *stopped_us = 100_199,
            "end_before_stop" => *at_us = 100_199,
            _ => unreachable!(),
        }
        negatives.push((name.to_string(), wire(&changed)));
    }
    for name in ["no_listening", "extra_listening", "packet_interleave", "extra_chunk", "trailing_record", "double_start"] {
        let mut changed = original.clone();
        match name {
            "no_listening" => { changed.remove(1); },
            "extra_listening" => changed.insert(2, changed[1].clone()),
            "packet_interleave" => changed.insert(3, changed[2].clone()),
            "extra_chunk" => changed.insert(4, changed[3].clone()),
            "trailing_record" => changed.push(changed[1].clone()),
            "double_start" => changed.insert(1, changed[0].clone()),
            _ => unreachable!(),
        }
        negatives.push((name.to_string(), wire(&changed)));
    }
    for (name, raw) in negatives {
        assert!(check(window.clone(), &raw, 13).is_err(), "admitted {name}");
    }
}

#[test]
fn independent_parser_and_framing_negative_controls() {
    let (window, records) = sample();
    let good = wire(&records);
    let text = String::from_utf8(good.clone()).unwrap();
    let cases = [
        ("unknown version", text.replacen("\"version\":1", "\"version\":2", 1).into_bytes()),
        ("duplicate version", text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1).into_bytes()),
        ("unknown field", text.replacen("\"version\":1", "\"version\":1,\"extra\":true", 1).into_bytes()),
        ("uppercase hex", text.replace("01020304", "aA020304").into_bytes()),
        ("odd hex", text.replace("01020304", "1020304").into_bytes()),
        ("bad utf8", [vec![0xff, b'\n'], good.clone()].concat()),
        ("long line", [vec![b'x'; capture::MAX_LINE + 1], vec![b'\n'], good.clone()].concat()),
        ("blank line", [vec![b'\n'], good.clone()].concat()),
        ("missing newline", good[..good.len() - 1].to_vec()),
        ("ROM restart", [b"ESP-ROM:esp32c6\n".to_vec(), good.clone()].concat()),
        ("panic suffix", [good.clone(), b"Guru Meditation Error\n".to_vec()].concat()),
    ];
    for (name, raw) in cases {
        assert!(check(window.clone(), &raw, 1).is_err(), "admitted {name}");
    }
}

static NEXT: AtomicUsize = AtomicUsize::new(0);
fn archive() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../archives/case-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir_all(path.join("window-0000")).unwrap();
    let (window, records) = sample();
    let raw = wire(&records);
    let mut events = Vec::new();
    let mut packets = Vec::new();
    let mut consumer = Consumer::new(window.clone(), &mut events, &mut packets);
    consumer.feed(&raw).unwrap();
    let summary = consumer.finish().unwrap();
    let dir = path.join("window-0000");
    fs::write(dir.join("serial.raw"), raw).unwrap();
    fs::write(dir.join("events.jsonl"), events).unwrap();
    fs::write(dir.join("packets.jsonl"), packets).unwrap();
    fs::write(dir.join("summary.json"), serde_json::to_vec(&summary).unwrap()).unwrap();
    let mut request = [0; 513];
    let len = encode(&window.request(), &mut request).unwrap();
    fs::write(dir.join("request.frame"), &request[..len]).unwrap();
    fs::write(path.join("summary.json"), serde_json::to_vec(&[summary]).unwrap()).unwrap();
    fs::write(path.join("manifest.json"), serde_json::to_vec(&Manifest { schema: 1, port: "synthetic-review-fixture".into(), baud: 921600, host_started_unix_ms: 0, windows: vec![window] }).unwrap()).unwrap();
    fs::write(path.join("startup.raw"), b"synthetic pre-command startup\n").unwrap();
    assert!(replay(&path).is_ok());
    path
}

#[test]
fn independent_archive_tamper_symlink_and_failure_controls() {
    for name in ["events suffix", "packets suffix", "request suffix", "raw suffix", "summary change", "dangling failure", "window symlink", "raw symlink"] {
        let path = archive();
        match name {
            "events suffix" | "packets suffix" | "request suffix" | "raw suffix" => {
                let relative = match name { "events suffix" => "events.jsonl", "packets suffix" => "packets.jsonl", "request suffix" => "request.frame", _ => "serial.raw" };
                let file = path.join("window-0000").join(relative);
                let mut bytes = fs::read(&file).unwrap();
                bytes.extend_from_slice(b"extraneous data\n");
                fs::write(file, bytes).unwrap();
            },
            "summary change" => fs::write(path.join("summary.json"), b"[]").unwrap(),
            "dangling failure" => std::os::unix::fs::symlink("absent", path.join("failure.txt")).unwrap(),
            "window symlink" => {
                fs::rename(path.join("window-0000"), path.join("saved-window")).unwrap();
                std::os::unix::fs::symlink("saved-window", path.join("window-0000")).unwrap();
            },
            "raw symlink" => {
                fs::rename(path.join("window-0000/serial.raw"), path.join("saved.raw")).unwrap();
                std::os::unix::fs::symlink("../saved.raw", path.join("window-0000/serial.raw")).unwrap();
            },
            _ => unreachable!(),
        }
        assert!(replay(&path).is_err(), "admitted {name}");
    }
}

#[test]
fn oversized_metadata_with_hidden_suffix_must_reject() {
    let mut admitted = Vec::new();
    for (relative, limit) in [("manifest.json", 1024 * 1024), ("window-0000/summary.json", 65536), ("summary.json", 16 * 1024 * 1024)] {
        let path = archive();
        let file = path.join(relative);
        let mut bytes = fs::read(&file).unwrap();
        bytes.resize(limit, b' ');
        bytes.extend_from_slice(b"THIS IS INVALID JSON BEYOND THE READER CAP");
        fs::write(file, bytes).unwrap();
        if replay(&path).is_ok() { admitted.push(relative); }
    }
    assert!(admitted.is_empty(), "admitted capped JSON prefixes with hidden suffixes: {admitted:?}");
}

#[test]
fn startup_and_inventory_boundaries() {
    for name in ["missing startup", "oversized startup", "symlink startup", "extra root file", "extra window file", "extra window"] {
        let path = archive();
        match name {
            "missing startup" => fs::remove_file(path.join("startup.raw")).unwrap(),
            "oversized startup" => fs::write(path.join("startup.raw"), vec![0; 65537]).unwrap(),
            "symlink startup" => {
                fs::remove_file(path.join("startup.raw")).unwrap();
                std::os::unix::fs::symlink("window-0000/serial.raw", path.join("startup.raw")).unwrap();
            },
            "extra root file" => fs::write(path.join("ignored.txt"), b"extra").unwrap(),
            "extra window file" => fs::write(path.join("window-0000/ignored.txt"), b"extra").unwrap(),
            "extra window" => fs::create_dir(path.join("window-0001")).unwrap(),
            _ => unreachable!(),
        }
        assert!(replay(&path).is_err(), "admitted {name}");
    }
}

#[test]
fn multi_window_clock_overlap_control() {
    for (shift, admitted) in [(100_200, true), (100_199, false), (0, false)] {
        let path = archive();
        let (mut window, mut records) = sample();
        window.id = 2;
        for record in &mut records {
            record.id = 2;
            match &mut record.event {
                Event::Start { at_us, .. } | Event::Listening { at_us } => *at_us += shift,
                Event::Packet(packet) => packet.at_us += shift,
                Event::Chunk { .. } => {},
                Event::End { at_us, stopped_us, .. } => { *at_us += shift; *stopped_us += shift; },
            }
        }
        let raw = wire(&records);
        let mut events = Vec::new();
        let mut packets = Vec::new();
        let mut consumer = Consumer::new(window.clone(), &mut events, &mut packets);
        consumer.feed(&raw).unwrap();
        let summary = consumer.finish().unwrap();
        let dir = path.join("window-0001");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("serial.raw"), raw).unwrap();
        fs::write(dir.join("events.jsonl"), events).unwrap();
        fs::write(dir.join("packets.jsonl"), packets).unwrap();
        fs::write(dir.join("summary.json"), serde_json::to_vec(&summary).unwrap()).unwrap();
        let mut request = [0; 513];
        let len = encode(&window.request(), &mut request).unwrap();
        fs::write(dir.join("request.frame"), &request[..len]).unwrap();
        let first = serde_json::from_slice::<Summary>(&fs::read(path.join("window-0000/summary.json")).unwrap()).unwrap();
        fs::write(path.join("summary.json"), serde_json::to_vec(&[first, summary]).unwrap()).unwrap();
        let mut manifest: Manifest = serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).unwrap();
        manifest.windows.push(window);
        fs::write(path.join("manifest.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert_eq!(replay(&path).is_ok(), admitted, "cross-window shift {shift}");
    }
}
