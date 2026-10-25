use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(radio: Radio, length: usize) -> (Window, Vec<Record>) {
    let channel = match radio {
        Radio::Wifi => 6,
        Radio::Ble => 0,
        Radio::Ieee802154 => 15,
    };
    let window = Window {
        id: 1,
        radio,
        channel,
        millis: 100,
    };
    let details = match radio {
        Radio::Wifi => Details::Wifi {
            kind: 0,
            rx_state: 0,
            sdk_time: 500,
            phy: 0,
            rate: 0,
        },
        Radio::Ble => Details::Ble {
            address: [2, 0, 0, 0, 0, 1],
            address_type: 1,
            event_type: 0,
            primary_phy: 1,
            secondary_phy: 0,
            sid: 255,
            data_status: 0,
            tx_power: 127,
            periodic_interval: 0,
            direct_address: [0; 6],
            direct_address_type: 0,
        },
        Radio::Ieee802154 => Details::Ieee802154 {
            lqi: 100,
            sdk_time: 123,
        },
    };
    let retained = length.min(SNAPLEN);
    let mut records = vec![
        Record::new(
            1,
            Event::Start {
                radio,
                channel,
                millis: 100,
                at_us: 100,
                snaplen: SNAPLEN as u16,
            },
        ),
        Record::new(1, Event::Listening { at_us: 200 }),
        Record::new(
            1,
            Event::Packet(Packet {
                seq: 1,
                at_us: 201,
                channel,
                rssi: -50,
                original_len: length as u16,
                captured_len: retained as u16,
                details,
            }),
        ),
    ];
    let data = (0..retained).map(|n| n as u8).collect::<Vec<_>>();
    for (index, chunk) in data.chunks(capture::CHUNK_BYTES).enumerate() {
        records.push(Record::new(
            1,
            Event::Chunk {
                seq: 1,
                offset: (index * capture::CHUNK_BYTES) as u16,
                data: capture::hex(chunk).unwrap(),
            },
        ));
    }
    records.push(Record::new(
        1,
        Event::End {
            at_us: 100_202,
            stopped_us: 100_201,
            seen: 3,
            emitted: 1,
            dropped: 1,
            invalid: 1,
            truncated: u32::from(length > SNAPLEN),
            error: 0,
        },
    ));
    (window, records)
}

fn wire(records: &[Record]) -> Vec<u8> {
    let mut result = Vec::new();
    for record in records {
        let mut buffer = [0; capture::MAX_LINE + 1];
        let len = capture::encode(record, &mut buffer).unwrap();
        result.extend_from_slice(&buffer[..len]);
    }
    result
}

fn derive(window: Window, raw: &[u8]) -> Result<Summary> {
    let mut consumer = Consumer::new(window, std::io::sink(), std::io::sink());
    for chunk in raw.chunks(17) {
        consumer.feed(chunk)?;
    }
    consumer.finish()
}

#[test]
fn full_packets_fragments_caps_and_loss_conservation() -> Result<()> {
    for (radio, length) in [
        (Radio::Wifi, 5000),
        (Radio::Ble, 251),
        (Radio::Ieee802154, 125),
    ] {
        let (window, records) = fixture(radio, length);
        let summary = derive(window, &wire(&records))?;
        assert_eq!(summary.activity.payload_bytes, length.min(SNAPLEN) as u64);
        assert_eq!(summary.sdk_reports, 3);
        assert_eq!(summary.recorder_dropped, 1);
    }
    assert!(capture::hex(&[0; 129]).is_err());
    Ok(())
}

#[test]
fn stream_negative_controls() {
    let (window, records) = fixture(Radio::Wifi, 200);
    let good = wire(&records);
    let mut cases = vec![
        Vec::new(),
        good[..good.len() - 1].to_vec(),
        [b"ESP-ROM: reset\n".as_slice(), &good].concat(),
        [good.as_slice(), b"panic\n"].concat(),
    ];
    for index in [0, 1, 2, 3, records.len() - 1] {
        let mut modified = records.clone();
        modified.remove(index);
        cases.push(wire(&modified));
        let mut modified = records.clone();
        modified.insert(index, modified[index].clone());
        cases.push(wire(&modified));
    }
    let corruptions = [
        (
            2,
            Event::Packet(Packet {
                seq: 9,
                ..match records[2].event {
                    Event::Packet(p) => p,
                    _ => unreachable!(),
                }
            }),
        ),
        (
            3,
            Event::Chunk {
                seq: 1,
                offset: 1,
                data: capture::hex(&[0]).unwrap(),
            },
        ),
        (
            3,
            Event::Chunk {
                seq: 1,
                offset: 0,
                data: "gg".try_into().unwrap(),
            },
        ),
        (
            3,
            Event::Chunk {
                seq: 1,
                offset: 0,
                data: "f".try_into().unwrap(),
            },
        ),
        (1, Event::Listening { at_us: 0 }),
    ];
    for (index, event) in corruptions {
        let mut modified = records.clone();
        modified[index].event = event;
        cases.push(wire(&modified));
    }
    for field in [
        "seen",
        "emitted",
        "dropped",
        "invalid",
        "truncated",
        "error",
        "stopped_us",
        "at_us",
    ] {
        let mut value = serde_json::to_value(records.last().unwrap()).unwrap();
        value["event"]["end"][field] = serde_json::json!(999);
        let mut modified = records.clone();
        *modified.last_mut().unwrap() = serde_json::from_value(value).unwrap();
        cases.push(wire(&modified));
    }
    for (index, bad) in cases.into_iter().enumerate() {
        assert!(
            derive(window.clone(), &bad).is_err(),
            "accepted negative control {index}"
        );
    }
}

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Archive(PathBuf);
impl Drop for Archive {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn archive() -> Result<Archive> {
    let path = std::env::temp_dir().join(format!(
        "wx-observation-{}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path)?;
    let dir = path.join("window-0000");
    fs::create_dir(&dir)?;
    let (window, records) = fixture(Radio::Ble, 200);
    let raw = wire(&records);
    let mut events = Vec::new();
    let mut packets = Vec::new();
    let mut consumer = Consumer::new(window.clone(), &mut events, &mut packets);
    consumer.feed(&raw)?;
    let summary = consumer.finish()?;
    fs::write(dir.join("serial.raw"), raw)?;
    fs::write(dir.join("events.jsonl"), events)?;
    fs::write(dir.join("packets.jsonl"), packets)?;
    fs::write(dir.join("summary.json"), serde_json::to_vec(&summary)?)?;
    let mut request = [0; 513];
    let n = encode(&window.request(), &mut request)?;
    fs::write(dir.join("request.frame"), &request[..n])?;
    fs::write(
        path.join("summary.json"),
        serde_json::to_vec(&vec![summary])?,
    )?;
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec(&Manifest {
            schema: 1,
            port: "synthetic".into(),
            baud: 921600,
            host_started_unix_ms: 0,
            windows: vec![window],
        })?,
    )?;
    Ok(Archive(path))
}

#[test]
fn archive_roundtrip_rejects_modified_outputs_and_failure_markers() -> Result<()> {
    let capture = archive()?;
    assert_eq!(replay(&capture.0)?.len(), 1);
    for relative in [
        "summary.json",
        "window-0000/summary.json",
        "window-0000/packets.jsonl",
        "window-0000/events.jsonl",
        "window-0000/request.frame",
        "window-0000/serial.raw",
    ] {
        let path = capture.0.join(relative);
        let original = fs::read(&path)?;
        fs::write(&path, b"{}\n")?;
        assert!(replay(&capture.0).is_err(), "admitted modified {relative}");
        fs::write(path, original)?;
    }
    fs::write(capture.0.join("failure.txt"), "retained failed attempt")?;
    assert!(replay(&capture.0).is_err());
    fs::remove_file(capture.0.join("failure.txt"))?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("absent", capture.0.join("failure.txt"))?;
        assert!(replay(&capture.0).is_err());
        fs::remove_file(capture.0.join("failure.txt"))?;
        let path = capture.0.join("window-0000/packets.jsonl");
        let bytes = fs::read(&path)?;
        fs::remove_file(&path)?;
        std::os::unix::fs::symlink("../summary.json", &path)?;
        assert!(replay(&capture.0).is_err());
        fs::remove_file(&path)?;
        fs::write(path, bytes)?;
    }
    assert!(replay(&capture.0).is_ok());
    Ok(())
}

#[test]
fn plan_rejects_bad_channels_and_duplicates_before_serial_access() -> Result<()> {
    assert_eq!(
        plan(
            &[Radio::Wifi, Radio::Ble, Radio::Ieee802154],
            &[1, 6, 11],
            &[11, 26],
            1000,
            2
        )?
        .len(),
        12
    );
    for channels in [&[0][..], &[14], &[1, 1], &[]] {
        assert!(plan(&[Radio::Wifi], channels, &[], 100, 1).is_err());
    }
    assert!(plan(&[Radio::Ble], &[], &[], 99, 1).is_err());
    Ok(())
}
