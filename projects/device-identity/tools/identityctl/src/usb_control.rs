//! Fixed USB IN-only capture, private receipts and portable complete replay.

use std::collections::BTreeSet;
use std::fs;
#[cfg(any(target_os = "macos", test))]
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use usb_observer::{Record, Summary};

#[cfg(target_os = "macos")]
use crate::capture::create_private_directory;
#[cfg(any(target_os = "macos", test))]
use crate::capture::{private_file, save_json};

const CAPTURE_VERSION: u32 = 2;
const FILES: [&str; 4] = [
    "capture-status.json",
    "native_provenance.json",
    "record.json",
    "summary.json",
];
const HEADER_PATHS: [&str; 8] = [
    "IOKit.framework/Headers/usb/IOUSBLib.h",
    "IOKit.framework/Headers/usb/USB.h",
    "IOKit.framework/Headers/IOCFPlugIn.h",
    "IOKit.framework/Headers/IOKitLib.h",
    "CoreFoundation.framework/Headers/CFPlugInCOM.h",
    "CoreFoundation.framework/Headers/CFUUID.h",
    "CoreFoundation.framework/Headers/CFNumber.h",
    "CoreFoundation.framework/Headers/CFString.h",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeaderInput {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    sdk_version: String,
    bindgen: String,
    libclang: String,
    headers: Vec<HeaderInput>,
    bindings_sha256: String,
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn provenance(bytes: &[u8]) -> Result<(), &'static str> {
    let value: Provenance =
        serde_json::from_slice(bytes).map_err(|_| "invalid native USB input provenance")?;
    if value.sdk_version != "27.0"
        || value.bindgen != "0.72.1"
        || value.libclang != "21.1.8"
        || value.headers.len() != HEADER_PATHS.len()
        || !valid_digest(&value.bindings_sha256)
        || value
            .headers
            .iter()
            .zip(HEADER_PATHS)
            .any(|(header, expected)| header.path != expected || !valid_digest(&header.sha256))
    {
        return Err("unsupported native USB input provenance");
    }
    Ok(())
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Failure {
    Observation(usb_observer::Error),
    Storage,
    Provenance,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    success: bool,
    error: Option<Failure>,
    record_sha256: Option<String>,
    provenance_sha256: Option<String>,
    summary_sha256: Option<String>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|_| "cannot encode USB capture record")?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(any(target_os = "macos", test))]
fn capture_with<F>(directory: &Path, inputs: &[u8], observe: F) -> Result<Summary, &'static str>
where
    F: FnOnce() -> Record,
{
    // Acquire the raw destination before calling the observer. An unwritable or
    // already existing record must never trigger a hardware request.
    let mut raw = private_file(&directory.join("record.json"))?;
    let mut receipt = Receipt {
        schema_version: CAPTURE_VERSION,
        success: false,
        error: Some(Failure::Storage),
        record_sha256: None,
        provenance_sha256: None,
        summary_sha256: None,
    };
    let result = (|| {
        provenance(inputs).map_err(|_| {
            receipt.error = Some(Failure::Provenance);
            "native USB provenance did not validate"
        })?;
        let mut source = private_file(&directory.join("native_provenance.json"))?;
        source
            .write_all(inputs)
            .map_err(|_| "cannot save USB provenance")?;
        source
            .sync_all()
            .map_err(|_| "cannot synchronize USB provenance")?;
        receipt.provenance_sha256 = Some(digest(inputs));

        let observed = observe();
        let bytes = json(&observed)?;
        raw.write_all(&bytes)
            .map_err(|_| "cannot save raw USB record")?;
        raw.sync_all()
            .map_err(|_| "cannot synchronize raw USB record")?;
        receipt.record_sha256 = Some(digest(&bytes));
        let report = usb_observer::summarize(&observed).map_err(|error| {
            receipt.error = Some(Failure::Observation(error));
            "USB control observation did not validate"
        })?;
        let summary = json(&report)?;
        let mut destination = private_file(&directory.join("summary.json"))?;
        destination
            .write_all(&summary)
            .map_err(|_| "cannot save USB summary")?;
        destination
            .sync_all()
            .map_err(|_| "cannot synchronize USB summary")?;
        receipt.summary_sha256 = Some(digest(&summary));
        receipt.success = true;
        receipt.error = None;
        Ok(report)
    })();
    save_json(&directory.join("capture-status.json"), &receipt)?;
    result
}

/// The production observer is never called by the fake-stream controls below.
#[cfg(target_os = "macos")]
pub(crate) fn capture(directory: &Path) -> Result<Summary, &'static str> {
    create_private_directory(directory)?;
    capture_with(
        directory,
        usb_observer::native_provenance().as_bytes(),
        usb_observer::observe,
    )
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn capture(_: &Path) -> Result<Summary, &'static str> {
    Err("native USB control capture requires macOS")
}

pub(crate) fn record(path: &Path) -> Result<Record, &'static str> {
    usb_observer::parse(&crate::read_input(path)?).map_err(|_| "invalid private USB control record")
}

pub(crate) fn replay(directory: &Path) -> Result<Summary, &'static str> {
    replay_validated(directory).map(|(_, summary)| summary)
}

/// Return the owned record whose bytes, receipt, provenance, complete request
/// sequence and saved summary were validated together. Offline derivatives must
/// use this record rather than rereading an unvalidated file after replay.
///
/// # Errors
///
/// Returns the same static diagnostics as [`replay`] for any inventory, receipt,
/// provenance, record or saved-summary disagreement. This validates saved bytes,
/// not their hardware origin or concurrent filesystem writers.
pub(crate) fn replay_record(directory: &Path) -> Result<Record, &'static str> {
    replay_validated(directory).map(|(record, _)| record)
}

fn replay_validated(directory: &Path) -> Result<(Record, Summary), &'static str> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|_| "cannot inspect USB capture directory")? {
        let entry = entry.map_err(|_| "cannot inspect USB capture entry")?;
        if !entry
            .file_type()
            .map_err(|_| "cannot inspect USB capture file")?
            .is_file()
        {
            return Err("USB capture requires regular files");
        }
        names.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid USB capture filename")?,
        );
    }
    if names != FILES.into_iter().map(str::to_owned).collect() {
        return Err("USB capture inventory is incomplete or unexpected");
    }
    let receipt: Receipt =
        serde_json::from_slice(&crate::read_input(&directory.join("capture-status.json"))?)
            .map_err(|_| "invalid USB capture receipt")?;
    if receipt.schema_version != CAPTURE_VERSION || !receipt.success || receipt.error.is_some() {
        return Err("USB capture records a failed or incomplete attempt");
    }
    let raw = crate::read_input(&directory.join("record.json"))?;
    let inputs = crate::read_input(&directory.join("native_provenance.json"))?;
    let summary = crate::read_input(&directory.join("summary.json"))?;
    provenance(&inputs)?;
    if receipt.record_sha256.as_deref() != Some(digest(&raw).as_str())
        || receipt.provenance_sha256.as_deref() != Some(digest(&inputs).as_str())
        || receipt.summary_sha256.as_deref() != Some(digest(&summary).as_str())
    {
        return Err("USB capture bytes disagree with their completion receipt");
    }
    let observed = usb_observer::parse(&raw).map_err(|_| "invalid private USB control record")?;
    let report =
        usb_observer::summarize(&observed).map_err(|_| "USB control record did not validate")?;
    if json(&report)? != summary {
        return Err("USB saved summary disagrees with complete replay");
    }
    Ok((observed, report))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::json;
    use usb_observer::{Domain, FORMAT_VERSION, Request, Transfer, observe_domain};

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Self {
            let path = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "usb-control-capture-{}-{}",
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

    fn inputs() -> Vec<u8> {
        serde_json::to_vec(&json!({
            "sdk_version":"27.0", "bindgen":"0.72.1", "libclang":"21.1.8",
            "headers": HEADER_PATHS.map(|path| json!({"path":path,"sha256":"a".repeat(64)})),
            "bindings_sha256":"b".repeat(64),
        }))
        .unwrap()
    }

    fn synthetic() -> Record {
        let domains = Domain::ALL.map(|domain| {
            observe_domain(domain, 1, 0, |request: Request| {
                let mut bytes = match request.value {
                    0x0100 => {
                        let (vendor, product) = domain.ids();
                        let mut bytes =
                            vec![18, 1, 0, 2, 0, 0, 0, 64, 0, 0, 0, 0, 0, 1, 1, 2, 3, 1];
                        bytes[8..10].copy_from_slice(&vendor.to_le_bytes());
                        bytes[10..12].copy_from_slice(&product.to_le_bytes());
                        bytes
                    }
                    0x0300 => vec![4, 3, 9, 4],
                    0x0303 => {
                        let units: Vec<_> = "private-synthetic-sentinel".encode_utf16().collect();
                        let mut bytes = vec![u8::try_from(2 + units.len() * 2).unwrap(), 3];
                        for unit in units {
                            bytes.extend_from_slice(&unit.to_le_bytes());
                        }
                        bytes
                    }
                    0x370b => vec![0x20],
                    0x000e => {
                        let mut bytes = vec![0; 678];
                        bytes[2] = 1;
                        bytes[4] = 0xff;
                        bytes
                    }
                    _ => panic!("unexpected synthetic request"),
                };
                bytes.truncate(usize::from(request.length));
                Transfer {
                    request,
                    status: 0,
                    returned_length: bytes.len() as u32,
                    bytes,
                }
            })
        });
        Record {
            version: FORMAT_VERSION,
            domains,
        }
    }

    #[test]
    fn fake_observer_uses_actual_save_and_complete_replay_path() {
        let temp = Temp::new();
        let report = capture_with(&temp.0, &inputs(), synthetic).unwrap();
        assert_eq!(replay(&temp.0).unwrap(), report);
        let raw = record(&temp.0.join("record.json")).unwrap();
        assert_eq!(raw, synthetic());
        let output = serde_json::to_string(&report).unwrap();
        assert!(!output.contains("private-synthetic-sentinel"));
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 4);
    }

    #[test]
    fn failed_native_result_remains_failed_even_with_a_complete_record_replacement() {
        let temp = Temp::new();
        assert!(
            capture_with(&temp.0, &inputs(), || {
                let mut observed = synthetic();
                observed.domains[1].cleanup_status = -9;
                observed
            })
            .is_err()
        );
        assert!(temp.0.join("record.json").exists());
        let status: Receipt =
            serde_json::from_slice(&fs::read(temp.0.join("capture-status.json")).unwrap()).unwrap();
        assert!(!status.success);
        assert!(replay(&temp.0).is_err());
        fs::write(temp.0.join("record.json"), json(&synthetic()).unwrap()).unwrap();
        save_json(
            &temp.0.join("summary.json"),
            &usb_observer::summarize(&synthetic()).unwrap(),
        )
        .unwrap();
        assert!(replay(&temp.0).is_err());
    }

    #[test]
    fn raw_provenance_summary_and_inventory_tampering_are_rejected() {
        for name in [
            "record.json",
            "native_provenance.json",
            "summary.json",
            "capture-status.json",
        ] {
            let temp = Temp::new();
            capture_with(&temp.0, &inputs(), synthetic).unwrap();
            let path = temp.0.join(name);
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            match name {
                "record.json" => {
                    value["domains"][0]["transfers"][3]["bytes"][2] = json!(88);
                }
                "native_provenance.json" => {
                    value["headers"][0]["sha256"] = json!("c".repeat(64));
                }
                "summary.json" => {
                    value["domains"][0]["serial_code_units"] = json!(1);
                }
                _ => {
                    value["success"] = json!(false);
                }
            }
            fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(replay(&temp.0).is_err(), "{name}");
        }
        let temp = Temp::new();
        capture_with(&temp.0, &inputs(), synthetic).unwrap();
        fs::write(temp.0.join("unexpected.txt"), "private marker").unwrap();
        assert!(replay(&temp.0).is_err());
    }

    #[test]
    fn raw_destination_and_provenance_fail_before_observation() {
        let temp = Temp::new();
        fs::write(temp.0.join("record.json"), "preserve").unwrap();
        let called = Cell::new(false);
        assert!(
            capture_with(&temp.0, &inputs(), || {
                called.set(true);
                synthetic()
            })
            .is_err()
        );
        assert!(!called.get());
        assert_eq!(fs::read(temp.0.join("record.json")).unwrap(), b"preserve");
        let other = Temp::new();
        assert!(
            capture_with(&other.0, b"invalid private provenance", || {
                called.set(true);
                synthetic()
            })
            .is_err()
        );
        assert!(!called.get());
        assert!(replay(&other.0).is_err());
    }

    #[test]
    fn version_one_completion_is_never_silently_upgraded() {
        let temp = Temp::new();
        capture_with(&temp.0, &inputs(), synthetic).unwrap();
        let path = temp.0.join("capture-status.json");
        let mut receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        receipt["schema_version"] = json!(1);
        fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
        assert!(replay(&temp.0).is_err());
    }
}
