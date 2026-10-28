//! Synthetic end-to-end controls for runtime and host-property summaries.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use runtime_protocol::{
    BASE_OVERRIDE, BLUETOOTH_OVERRIDE, IEEE802154_OVERRIDE, Phase, Request, Response, Snapshot,
    WIFI_AP_OVERRIDE, WIFI_STATION_OVERRIDE,
};
use serde_json::{Value, json};

static NEXT: AtomicU64 = AtomicU64::new(0);
const SENTINEL: &str = "synthetic-runtime-private-sentinel";

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "identity-runtime-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(command: &str, input: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_identityctl"))
        .args([
            OsStr::new(command),
            OsStr::new("--input"),
            input.as_os_str(),
        ])
        .output()
        .unwrap()
}

fn failure(output: Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.starts_with("identityctl: "));
    assert!(!diagnostic.contains(SENTINEL));
}

fn sequence() -> Vec<Value> {
    let initial = Snapshot {
        factory: [48, 52, 56, 60, 64, 68, 72, 76],
        base: [48, 52, 56, 68, 72, 76],
        wifi_station: [48, 52, 56, 68, 72, 76],
        wifi_ap: [48, 52, 56, 68, 72, 77],
        bluetooth: [48, 52, 56, 68, 72, 78],
        ieee802154: [48, 52, 56, 60, 64, 68, 72, 76],
    };
    let base = Snapshot {
        base: BASE_OVERRIDE,
        ..initial
    };
    let custom = Snapshot {
        wifi_station: WIFI_STATION_OVERRIDE,
        wifi_ap: WIFI_AP_OVERRIDE,
        bluetooth: BLUETOOTH_OVERRIDE,
        ieee802154: IEEE802154_OVERRIDE,
        ..base
    };
    let snapshots = [initial, base, custom, initial];
    Phase::ALL
        .into_iter()
        .enumerate()
        .map(|(index, phase)| {
            let request = Request::new(256 + index as u32, phase);
            let response = Response::new(request.id, phase, 0, Some(snapshots[index]));
            json!({"request":request,"response":response})
        })
        .collect()
}

fn encode(entries: &[Value]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for entry in entries {
        serde_json::to_writer(&mut bytes, entry).unwrap();
        bytes.push(b'\n');
    }
    bytes
}

#[test]
fn complete_runtime_cli_replays_with_only_fixed_findings() {
    let temp = Temp::new();
    let entries = sequence();
    let path = temp.write("exchanges.jsonl", &encode(&entries));
    let first = run("runtime-summary", &path);
    let second = run("runtime-summary", &path);
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["phase_count"], 4);
    assert_eq!(
        report["cached_interfaces_unchanged_after_base_override"],
        true
    );
    assert_eq!(report["initial_table_state_restored"], true);
    let output = String::from_utf8(first.stdout).unwrap();
    assert!(!output.contains("snapshot"));
    assert!(!output.contains(&serde_json::to_string(&entries[0]["response"]["snapshot"]).unwrap()));
}

#[test]
fn runtime_cli_rejects_late_corruption_unknown_private_fields_and_incomplete_records() {
    let temp = Temp::new();
    let mut variants = Vec::new();
    let mut entries = sequence();
    entries[3]["response"]["snapshot"]["base"][0] = json!(98);
    variants.push(encode(&entries));
    let mut entries = sequence();
    entries[3]["response"]["unexpected"] = json!(SENTINEL);
    variants.push(encode(&entries));
    let mut entries = sequence();
    entries[3]["response"]["id"] = json!(999);
    variants.push(encode(&entries));
    let mut entries = sequence();
    entries[3]["response"]["status"] = json!(-2);
    entries[3]["response"]["snapshot"] = Value::Null;
    variants.push(encode(&entries));
    let mut bytes = encode(&sequence());
    bytes.pop();
    variants.push(bytes);
    variants.push(vec![b'x'; 16 * 1024 + 1]);
    for (index, bytes) in variants.iter().enumerate() {
        let path = temp.write(&format!("{SENTINEL}-{index}.jsonl"), bytes);
        failure(run("runtime-summary", &path));
    }
}

#[test]
fn usb_comparison_compares_contents_and_omits_them_through_cli() {
    let temp = Temp::new();
    let before = json!({"SPUSBHostDataType":[{"_items":[{
        "USBDeviceKeyVendorID":"0x303a","USBDeviceKeyProductID":"0x1001",
        "USBDeviceKeySerialNumber":SENTINEL
    }]}]});
    let mut after = before.clone();
    after["SPUSBHostDataType"][0]["_items"][0]["USBDeviceKeySerialNumber"] =
        json!(format!("{SENTINEL}-different"));
    let first = temp.write("before.json", &serde_json::to_vec(&before).unwrap());
    let second = temp.write("after.json", &serde_json::to_vec(&after).unwrap());
    let command = || {
        Command::new(env!("CARGO_BIN_EXE_identityctl"))
            .args([
                OsStr::new("usb-compare"),
                OsStr::new("--before"),
                first.as_os_str(),
                OsStr::new("--after"),
                second.as_os_str(),
            ])
            .output()
            .unwrap()
    };
    let output = command();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["domains"][0]["serial_property"], "changed");
    assert!(!String::from_utf8(output.stdout).unwrap().contains(SENTINEL));
    fs::write(&second, format!("{{\"{SENTINEL}\":null}}")).unwrap();
    failure(command());
}

#[test]
fn capture_rejects_public_or_absolute_output_before_opening_a_device() {
    for output in [
        "projects/device-identity/should-not-exist",
        "/tmp/should-not-exist",
        "private/device-identity/../should-not-exist",
    ] {
        let response = Command::new(env!("CARGO_BIN_EXE_identityctl"))
            .args(["runtime-capture", "--port", SENTINEL, "--output", output])
            .output()
            .unwrap();
        failure(response);
    }
}

#[cfg(unix)]
#[test]
fn all_new_help_paths_keep_a_static_binary_name() {
    use std::os::unix::process::CommandExt;
    for operation in [
        "runtime-capture",
        "runtime-summary",
        "runtime-replay",
        "startup-summary",
        "usb-compare",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_identityctl"))
            .arg0(SENTINEL)
            .args([operation, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!String::from_utf8(output.stdout).unwrap().contains(SENTINEL));
    }
}
