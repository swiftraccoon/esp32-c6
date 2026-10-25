use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use serde_json::{Value, json};

fn n(value: &Value, key: &str) -> u64 {
    value[key].as_u64().expect("missing unsigned field")
}

fn check(path: &Path) -> Value {
    let manifest: Value = serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).expect("manifest JSON");
    let saved: Value = serde_json::from_slice(&fs::read(path.join("summary.json")).unwrap()).expect("summary JSON");
    let windows = manifest["windows"].as_array().expect("window array");
    assert_eq!(saved.as_array().unwrap().len(), windows.len());
    let mut totals: BTreeMap<String, [u64; 9]> = BTreeMap::new();
    let mut kinds: BTreeMap<String, u64> = BTreeMap::new();
    let mut previous_end = 0;
    let mut result_windows = Vec::new();
    for (index, window) in windows.iter().enumerate() {
        let radio = window["radio"].as_str().unwrap();
        assert!(matches!(radio, "wifi" | "ble" | "ieee802154"));
        assert_eq!(n(window, "id"), index as u64 + 1);
        let dir = path.join(format!("window-{index:04}"));
        let bytes = fs::read(dir.join("serial.raw")).unwrap();
        assert!(bytes.ends_with(b"\n"));
        let mut packet_count = 0;
        let mut payload_bytes = 0;
        let mut truncated = 0;
        let mut pending: Option<(u64, usize, usize)> = None;
        let mut last_seq = None;
        let mut last_time = 0;
        let mut start = None;
        let mut listening = None;
        let mut end = None;
        let mut rssi_min = None;
        let mut rssi_max = None;
        let mut metadata_values: BTreeMap<String, u64> = BTreeMap::new();
        for line in bytes[..bytes.len() - 1].split(|byte| *byte == b'\n') {
            assert!(line.starts_with(b"WC "));
            let record: Value = serde_json::from_slice(&line[3..]).expect("raw record JSON");
            assert_eq!(n(&record, "version"), 1);
            assert_eq!(n(&record, "id"), n(window, "id"));
            assert!(end.is_none(), "record after End");
            let event = record["event"].as_object().unwrap();
            assert_eq!(event.len(), 1);
            let (name, data) = event.iter().next().unwrap();
            match name.as_str() {
                "start" => {
                    assert!(start.is_none());
                    for field in ["radio", "channel", "millis"] { assert_eq!(data[field], window[field]); }
                    assert_eq!(n(data, "snaplen"), 4096);
                    start = Some(n(data, "at_us"));
                    assert!(start.unwrap() >= previous_end);
                },
                "listening" => {
                    assert!(start.is_some() && listening.is_none());
                    listening = Some(n(data, "at_us"));
                    assert!(listening.unwrap() >= start.unwrap());
                },
                "packet" => {
                    assert!(listening.is_some() && pending.is_none());
                    let seq = n(data, "seq");
                    assert!(last_seq.is_none_or(|previous| seq > previous));
                    last_seq = Some(seq);
                    let at = n(data, "at_us");
                    assert!(at >= start.unwrap() && at >= last_time);
                    last_time = at;
                    assert_eq!(n(data, "channel"), n(window, "channel"));
                    let details = data["details"].as_object().unwrap();
                    assert_eq!(details.len(), 1);
                    let (kind, metadata) = details.iter().next().unwrap();
                    assert_eq!(kind, radio);
                    let original = n(data, "original_len");
                    let cap = n(data, "captured_len");
                    let misc = radio == "wifi" && n(metadata, "kind") == 3;
                    assert_eq!(cap, if misc { 0 } else { original.min(4096) });
                    match radio {
                        "wifi" => {
                            assert!(n(metadata, "kind") <= 3);
                            assert!(original <= 16383);
                            for field in ["kind", "rx_state", "phy", "rate"] {
                                *metadata_values.entry(format!("{field}:{}", n(metadata, field))).or_default() += 1;
                            }
                        },
                        "ble" => {
                            assert!(original <= 251);
                            for field in ["event_type", "primary_phy", "secondary_phy", "data_status", "address_type"] {
                                *metadata_values.entry(format!("{field}:{}", n(metadata, field))).or_default() += 1;
                            }
                            for field in ["address", "direct_address"] {
                                let address = metadata[field].as_array().unwrap();
                                assert_eq!(address.len(), 6);
                                assert!(address.iter().all(|part| part.as_u64().is_some_and(|part| part <= 255)));
                            }
                        },
                        "ieee802154" => { assert!(original <= 125); assert!(n(metadata, "lqi") <= 255); },
                        _ => unreachable!(),
                    }
                    let rssi = data["rssi"].as_i64().unwrap();
                    assert!((-128..=127).contains(&rssi));
                    if rssi != 127 {
                        rssi_min = Some(rssi_min.map_or(rssi, |old: i64| old.min(rssi)));
                        rssi_max = Some(rssi_max.map_or(rssi, |old: i64| old.max(rssi)));
                    }
                    let category = match radio {
                        "wifi" => format!("wifi:{}", n(metadata, "kind")),
                        "ble" => format!("ble:{}", n(metadata, "event_type")),
                        _ => String::from("ieee802154"),
                    };
                    *kinds.entry(category).or_default() += 1;
                    truncated += u64::from(original > 4096);
                    packet_count += 1;
                    payload_bytes += cap;
                    if cap != 0 { pending = Some((seq, cap as usize, 0)); }
                },
                "chunk" => {
                    let (seq, cap, used) = pending.as_mut().expect("chunk without pending packet");
                    assert_eq!(n(data, "seq"), *seq);
                    assert_eq!(n(data, "offset"), *used as u64);
                    let hex = data["data"].as_str().unwrap();
                    assert!(!hex.is_empty() && hex.len() <= 256 && hex.len() % 2 == 0);
                    assert!(hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
                    *used += hex.len() / 2;
                    assert!(*used <= *cap);
                    if used == cap { pending = None; }
                },
                "end" => {
                    assert!(pending.is_none());
                    assert_eq!(data["error"].as_i64().unwrap(), 0);
                    assert_eq!(n(data, "emitted"), packet_count);
                    assert_eq!(n(data, "truncated"), truncated);
                    assert_eq!(n(data, "seen"), packet_count + n(data, "dropped") + n(data, "invalid"));
                    assert!(last_seq.is_none_or(|seq| seq < n(data, "seen")));
                    assert!(n(data, "stopped_us") >= last_time);
                    assert!(n(data, "stopped_us") >= listening.unwrap() + n(window, "millis") * 1000);
                    assert!(n(data, "at_us") >= n(data, "stopped_us"));
                    end = Some(data.clone());
                },
                _ => panic!("unknown event"),
            }
        }
        let end = end.expect("missing End");
        let summary: Value = serde_json::from_slice(&fs::read(dir.join("summary.json")).unwrap()).unwrap();
        assert_eq!(summary, saved[index]);
        assert_eq!(summary["window"], *window);
        for (sum_key, expected) in [
            ("sdk_reports", n(&end, "seen")), ("recorder_dropped", n(&end, "dropped")),
            ("invalid_reports", n(&end, "invalid")), ("truncated", truncated),
            ("start_us", start.unwrap()), ("listening_us", listening.unwrap()),
            ("stopped_us", n(&end, "stopped_us")), ("end_us", n(&end, "at_us")),
        ] { assert_eq!(n(&summary, sum_key), expected); }
        assert_eq!(n(&summary["activity"], "packets"), packet_count);
        assert_eq!(n(&summary["activity"], "payload_bytes"), payload_bytes);
        assert_eq!(summary["activity"]["rssi_min_dbm"].as_i64(), rssi_min);
        assert_eq!(summary["activity"]["rssi_max_dbm"].as_i64(), rssi_max);
        let packets = fs::read(dir.join("packets.jsonl")).unwrap();
        assert_eq!(packets.iter().filter(|byte| **byte == b'\n').count() as u64, packet_count);
        let count = totals.entry(radio.into()).or_default();
        let values = [1, n(&end, "seen"), packet_count, n(&end, "dropped"), n(&end, "invalid"), truncated, payload_bytes, n(&end, "stopped_us") - listening.unwrap(), n(window, "millis") * 1000];
        for (sum, value) in count.iter_mut().zip(values) { *sum += value; }
        previous_end = n(&end, "at_us");
        result_windows.push(json!({"index":index,"radio":radio,"channel":n(window,"channel"),"seen":values[1],"logged":packet_count,"dropped":values[3],"invalid":values[4],"truncated":truncated,"payload_bytes":payload_bytes,"requested_us":values[8],"listening_us":values[7],"rssi_min":rssi_min,"rssi_max":rssi_max,"metadata_counts":metadata_values}));
    }
    json!({"windows":windows.len(),"aggregate_fields":["windows","sdk_reports","logged","dropped","invalid","truncated","payload_bytes","listening_us","requested_us"],"aggregates":totals,"packet_types":kinds,"per_window":result_windows})
}

fn main() {
    let path = std::env::args_os().nth(1).expect("capture path required");
    println!("{}", serde_json::to_string_pretty(&check(Path::new(&path))).unwrap());
}
