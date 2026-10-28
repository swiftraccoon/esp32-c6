//! Synthetic CLI controls for private packet summaries and late corruption.

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static NEXT: AtomicU64 = AtomicU64::new(0);
const SENTINEL: &[u8] = b"synthetic-private-wifi-label";

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "identity-wifi-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn summarize(&self, bytes: &[u8]) -> Output {
        let path = self.0.join("packets.pcapng");
        fs::write(&path, bytes).unwrap();
        Command::new(env!("CARGO_BIN_EXE_identityctl"))
            .args([
                OsStr::new("wifi-summary"),
                OsStr::new("--input"),
                path.as_os_str(),
            ])
            .output()
            .unwrap()
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn block(kind: u32, body: &[u8]) -> Vec<u8> {
    assert_eq!(body.len() % 4, 0);
    let length = u32::try_from(body.len() + 12).unwrap();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&kind.to_le_bytes());
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(body);
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes
}

fn option(code: u16, value: &[u8]) -> Vec<u8> {
    let mut bytes = code.to_le_bytes().to_vec();
    bytes.extend_from_slice(&u16::try_from(value.len()).unwrap().to_le_bytes());
    bytes.extend_from_slice(value);
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    bytes
}

fn fixture() -> Vec<u8> {
    let mut section = 0x1a2b3c4du32.to_le_bytes().to_vec();
    section.extend_from_slice(&1u16.to_le_bytes());
    section.extend_from_slice(&0u16.to_le_bytes());
    section.extend_from_slice(&u64::MAX.to_le_bytes());
    let mut bytes = block(0x0a0d0d0a, &section);
    let mut interface = 127u16.to_le_bytes().to_vec();
    interface.extend_from_slice(&0u16.to_le_bytes());
    interface.extend_from_slice(&65535u32.to_le_bytes());
    interface.extend(option(2, SENTINEL));
    interface.extend(option(0, &[]));
    bytes.extend(block(1, &interface));

    let mut packet = vec![0, 0, 8, 0, 0, 0, 0, 0];
    packet.extend_from_slice(&[0x80, 0, 0, 0]);
    packet.extend_from_slice(&[0xff; 6]);
    let synthetic_address = [2, 17, 34, 51, 68, 85];
    packet.extend_from_slice(&synthetic_address);
    packet.extend_from_slice(&synthetic_address);
    packet.extend_from_slice(&[0, 0]);
    packet.extend_from_slice(&[0; 8]);
    packet.extend_from_slice(&100u16.to_le_bytes());
    packet.extend_from_slice(&1u16.to_le_bytes());
    packet.extend_from_slice(&[0, u8::try_from(SENTINEL.len()).unwrap()]);
    packet.extend_from_slice(SENTINEL);
    packet.extend_from_slice(&[1, 1, 0x82, 3, 1, 1, 5, 4, 0, 1, 0, 0]);
    let packet_length = u32::try_from(packet.len()).unwrap();
    for timestamp in 1..=32u32 {
        let mut body = 0u32.to_le_bytes().to_vec();
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&timestamp.to_le_bytes());
        body.extend_from_slice(&packet_length.to_le_bytes());
        body.extend_from_slice(&packet_length.to_le_bytes());
        body.extend_from_slice(&packet);
        body.resize(body.len().next_multiple_of(4), 0);
        bytes.extend(block(6, &body));
    }
    bytes
}

#[test]
fn wifi_summary_decodes_beacons_without_publishing_private_contents() {
    let temp = Temp::new();
    let first = temp.summarize(&fixture());
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["packet_count"], 32);
    assert_eq!(report["beacon_count"], 32);
    assert_eq!(report["visible_ssid_beacon_count"], 32);
    assert_eq!(report["channel_ie_beacon_count"], 32);
    assert_eq!(report["tim_ie_beacon_count"], 32);
    assert!(!first.stdout.windows(SENTINEL.len()).any(|v| v == SENTINEL));
    assert_eq!(first.stdout, temp.summarize(&fixture()).stdout);
}

#[test]
fn late_wifi_container_corruption_emits_no_partial_summary() {
    let temp = Temp::new();
    let valid = fixture();
    let mut bad_trailer = valid.clone();
    *bad_trailer.last_mut().unwrap() ^= 1;
    let mut extra = valid.clone();
    extra.extend_from_slice(SENTINEL);
    for bytes in [&valid[..valid.len() - 1], &bad_trailer, &extra] {
        let output = temp.summarize(bytes);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.windows(SENTINEL.len()).any(|v| v == SENTINEL));
    }
}

#[test]
fn wifi_help_paths_use_static_binary_metadata() {
    for command in [
        "wifi-summary",
        "wifi-capture",
        "wifi-replay",
        "wifi-ap-trial",
        "wifi-ap-replay",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_identityctl"))
            .args([command, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("identityctl")
        );
    }
}
