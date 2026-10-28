//! Synthetic controls exercise the actual file/CLI/JSON output path.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
const SENTINEL: &str = "synthetic-private-sentinel";
const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;

struct Temp {
    root: PathBuf,
}

impl Temp {
    fn new() -> Self {
        let root = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "identityctl-control-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        Self { root }
    }

    fn write(&self, bytes: &[u8]) -> PathBuf {
        let path = self.root.join("input.json");
        fs::write(&path, bytes).unwrap();
        path
    }

    fn json(&self, input: &Value) -> PathBuf {
        self.write(&serde_json::to_vec(input).unwrap())
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run(path: &Path) -> Output {
    run_command("usb-summary", path)
}

fn run_command(command: &str, path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_identityctl"))
        .args([OsStr::new(command), OsStr::new("--input"), path.as_os_str()])
        .output()
        .unwrap()
}

fn efuse_input() -> Value {
    let mut fields = serde_json::Map::new();
    for (name, count, block, word, pos) in [
        ("MAC", 6, 1, 0, 0),
        ("MAC_EXT", 2, 1, 1, 16),
        ("OPTIONAL_UNIQUE_ID", 16, 2, 0, 0),
    ] {
        let mac_field = matches!(name, "MAC" | "MAC_EXT");
        let separator = if mac_field { ":" } else { " " };
        let mut value = vec!["00"; count].join(separator);
        if mac_field {
            value.push_str(" (OK)");
        }
        fields.insert(
            name.into(),
            json!({
                "name": name, "value": value, "raw_value": format!("0x{}", "00".repeat(count)),
                "readable": true, "writeable": false, "description": SENTINEL, "category": SENTINEL,
                "block": block, "word": word, "pos": pos, "efuse_type": format!("bytes:{count}"),
                "bit_len": count * 8
            }),
        );
    }
    Value::Object(fields)
}

fn expect_failure(output: Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "failure emitted partial JSON");
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.starts_with("identityctl: "));
    assert!(!error.contains(SENTINEL));
}

fn device(vendor: &str, product: &str) -> Value {
    json!({
        "USBDeviceKeyVendorID": vendor,
        "USBDeviceKeyProductID": product
    })
}

fn input(nodes: Vec<Value>) -> Value {
    json!({"SPUSBHostDataType": [{"_items": nodes}]})
}

#[test]
fn nested_matches_count_serial_states_without_copying_private_fields() {
    let temp = Temp::new();
    let mut native = device("0x303a", "0x1001");
    native["USBDeviceKeySerialNumber"] = json!(SENTINEL);
    native["_name"] = json!(SENTINEL);
    native["USBKeyLocationID"] = json!(SENTINEL);
    native["unknown"] = json!({"nested": [SENTINEL]});
    let mut bridge = device("0x10c4", "0xea60");
    bridge["USBDeviceKeySerialNumber"] = json!(" \t");
    let mut hub = device("0x1234", "0x0001");
    hub["_items"] = json!([native, bridge, device("0x10c4", "0xea60")]);
    let path = temp.json(&input(vec![hub]));
    let first = run(&path);
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(
        report,
        json!({
            "schema_version": 1,
            "source_format": "macos_sp_usb_host_data_type",
            "observer": "attached_usb_host",
            "domains": [
                {"domain": "native_usb_serial_jtag", "device_count": 1,
                 "serial_present_count": 1, "serial_empty_count": 0, "serial_absent_count": 0},
                {"domain": "cp2102n_bridge", "device_count": 2,
                 "serial_present_count": 0, "serial_empty_count": 1, "serial_absent_count": 1}
            ]
        })
    );
    assert!(!String::from_utf8_lossy(&first.stdout).contains(SENTINEL));
    assert_eq!(
        first.stdout,
        run(&path).stdout,
        "saved input replay changed summary"
    );
}

#[test]
fn wrong_products_and_no_matches_produce_zero_counts() {
    let temp = Temp::new();
    let output = run(&temp.json(&input(vec![
        device("0x303a", "0x1002"),
        device("0x10c4", "0xea61"),
    ])));
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    for domain in report["domains"].as_array().unwrap() {
        assert_eq!(domain["device_count"], 0);
    }
    assert!(
        run(&temp.json(&json!({"SPUSBHostDataType": []})))
            .status
            .success()
    );
}

#[test]
fn an_empty_controller_is_valid_but_an_unrecognized_empty_leaf_is_not() {
    let temp = Temp::new();
    let path = temp.json(&json!({"SPUSBHostDataType": [{"Driver": "synthetic-driver"}]}));
    let output = run(&path);
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["domains"][0]["device_count"], 0);
    expect_failure(run(
        &temp.json(&input(vec![json!({"Driver": "synthetic-driver"})]))
    ));
}

#[test]
fn unsupported_or_malformed_schemas_cannot_become_no_device_observations() {
    let temp = Temp::new();
    for invalid in [
        json!({}),
        json!({"SPUSBDataType": []}),
        json!({"SPUSBHostDataType": null}),
        json!({"SPUSBHostDataType": [{"new_items": []}]}),
        input(vec![json!({"USBDeviceKeyVendorID": "0x303a"})]),
        input(vec![json!({"USBDeviceKeyProductID": "0x1001"})]),
        input(vec![json!({"_items": null})]),
        input(vec![json!({"_items": {}})]),
        input(vec![json!({"_items": [SENTINEL]})]),
    ] {
        expect_failure(run(&temp.json(&invalid)));
    }
}

#[test]
fn malformed_ids_and_serial_types_fail_without_echoing_values() {
    let temp = Temp::new();
    for invalid_id in [
        json!(SENTINEL),
        json!("303a"),
        json!("0x303aa"),
        json!("0x303g"),
        json!("0x303a trailing"),
        json!(123),
        Value::Null,
    ] {
        let mut node = device("0x303a", "0x1001");
        node["USBDeviceKeyVendorID"] = invalid_id;
        expect_failure(run(&temp.json(&input(vec![node]))));
    }
    for invalid_serial in [
        json!(123),
        json!([SENTINEL]),
        json!({"value": SENTINEL}),
        Value::Null,
    ] {
        let mut node = device("0x303a", "0x1001");
        node["USBDeviceKeySerialNumber"] = invalid_serial;
        expect_failure(run(&temp.json(&input(vec![node]))));
    }
}

#[test]
fn duplicate_metadata_trailing_bytes_and_invalid_utf8_are_rejected() {
    let temp = Temp::new();
    for invalid in [
        br#"{"SPUSBHostDataType": [], "SPUSBHostDataType": []}"#.as_slice(),
        br#"{"SPUSBHostDataType":[{"USBDeviceKeyVendorID":"0x303a","USBDeviceKeyVendorID":"0x10c4","USBDeviceKeyProductID":"0x1001"}]}"#,
        br#"{"SPUSBHostDataType": []} synthetic-private-sentinel"#,
        b"\xffsynthetic-private-sentinel",
    ] {
        expect_failure(run(&temp.write(invalid)));
    }
}

#[test]
fn exact_file_cap_is_accepted_and_an_extra_byte_is_rejected() {
    let temp = Temp::new();
    let mut bytes = br#"{"SPUSBHostDataType": []}"#.to_vec();
    bytes.resize(MAX_INPUT_BYTES, b' ');
    assert!(run(&temp.write(&bytes)).status.success());
    bytes.push(b' ');
    expect_failure(run(&temp.write(&bytes)));
}

#[test]
fn non_regular_missing_and_parent_paths_are_rejected_without_path_echo() {
    let temp = Temp::new();
    expect_failure(run(&temp.root));
    expect_failure(run(&temp.root.join(SENTINEL)));
    expect_failure(run(&temp.root.join("..").join(SENTINEL)));
}

#[cfg(unix)]
#[test]
fn symlink_file_and_parent_are_rejected() {
    use std::os::unix::fs::symlink;

    let temp = Temp::new();
    let source = temp.json(&input(Vec::new()));
    let link = temp.root.join("link");
    symlink(&source, &link).unwrap();
    expect_failure(run(&link));
    let directory = temp.root.join("linked-directory");
    symlink(&temp.root, &directory).unwrap();
    expect_failure(run(&directory.join("input.json")));
}

#[test]
fn argument_failures_do_not_copy_unknown_flags_or_path_values() {
    for args in [
        vec![SENTINEL],
        vec!["usb-summary", "--input", SENTINEL, SENTINEL],
        vec!["usb-summary", "--input"],
    ] {
        expect_failure(
            Command::new(env!("CARGO_BIN_EXE_identityctl"))
                .args(args)
                .output()
                .unwrap(),
        );
    }
}

#[cfg(unix)]
#[test]
fn help_and_version_do_not_copy_an_alternate_argv_zero() {
    use std::os::unix::process::CommandExt;

    for arguments in [
        vec!["--help"],
        vec!["--version"],
        vec!["usb-summary", "--help"],
        vec!["efuse-summary", "--help"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_identityctl"))
            .arg0(SENTINEL)
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("identityctl"));
        assert!(!text.contains(SENTINEL));
    }
}

#[test]
fn efuse_cli_save_and_replay_produce_only_sanitized_field_states() {
    let temp = Temp::new();
    let path = temp.json(&efuse_input());
    let first = run_command("efuse-summary", &path);
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&first.stdout).contains(SENTINEL));
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["kind"], "efuse_summary");
    assert_eq!(report["observer"], "physical_debug");
    assert_eq!(report["fields"].as_array().unwrap().len(), 3);
    assert!(report["fields"].as_array().unwrap().iter().all(|field| {
        field["content"] == "all_zero" && field["readable"] == true && field["writeable"] == false
    }));
    assert_eq!(first.stdout, run_command("efuse-summary", &path).stdout);
}

#[test]
fn efuse_cli_errors_never_emit_partial_json_or_private_data() {
    let temp = Temp::new();
    for input in [json!({}), json!({"unexpected": SENTINEL})] {
        expect_failure(run_command("efuse-summary", &temp.json(&input)));
    }
    let mut bad = efuse_input();
    bad["OPTIONAL_UNIQUE_ID"]["value"] = json!(SENTINEL);
    expect_failure(run_command("efuse-summary", &temp.json(&bad)));
    let mut bad = efuse_input();
    bad["MAC_EXT"]["bit_len"] = json!(17);
    expect_failure(run_command("efuse-summary", &temp.json(&bad)));
    let mut bad = efuse_input();
    bad["MAC"]["value"] = json!(SENTINEL);
    expect_failure(run_command("efuse-summary", &temp.json(&bad)));
    let mut bad = efuse_input();
    bad["MAC_EXT"]["value"] = json!("synthetic failed integrity annotation");
    expect_failure(run_command("efuse-summary", &temp.json(&bad)));
}
