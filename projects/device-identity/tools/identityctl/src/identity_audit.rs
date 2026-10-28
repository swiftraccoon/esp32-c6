//! Offline, value-free relations and bridge-integrity checks on complete inputs.
//!
//! Both operations replay the complete existing USB capture first. A valid
//! native descriptor cannot rescue a failed bridge domain or receipt. Factory
//! MAC extraction follows the existing selected-field eFuse validator, including
//! metadata, display/raw byte order and integrity-annotation checks. These saved
//! representations do not authenticate hardware or establish population uniqueness.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use usb_observer::{Domain, Record};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    UsbFactoryMacRelation,
    Cp2102nConfigurationIntegrity,
}

/// A strict MAC-form native serial compared with a validated readable factory MAC.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UsbFactoryMacRelation {
    schema_version: u32,
    kind: Kind,
    whole_usb_capture_replayed: bool,
    selected_efuse_fields_validated: bool,
    native_serial_code_units: usize,
    matches_factory_mac: bool,
}

/// Saved configuration layout and checksum qualification, without its contents.
///
/// Byte four's default flag is deliberately not a lock or programming-permission
/// interpretation. Legacy GetLockValue excludes CP2102N in AN978 section 1.1.34.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BridgeConfigurationAudit {
    schema_version: u32,
    kind: Kind,
    whole_usb_capture_replayed: bool,
    configuration_bytes: usize,
    configuration_layout_version: u8,
    little_endian_size_matches_length: bool,
    checksum_coverage_bytes: usize,
    big_endian_fletcher16_valid: bool,
    update_byte_is_default_ff: bool,
    lock_state_qualified: bool,
    programming_permission_qualified: bool,
}

fn complete_record(capture: &Path) -> Result<Record, &'static str> {
    crate::usb_control::replay_record(capture)
}

fn mac_text(bytes: &[u8]) -> Result<[u8; 6], &'static str> {
    if bytes.len() != 17 || !bytes.is_ascii() {
        return Err("native serial is outside the supported MAC representation");
    }
    let mut result = [0; 6];
    for (index, octet) in result.iter_mut().enumerate() {
        let start = index * 3;
        if (index != 5 && bytes[start + 2] != b':')
            || !bytes[start..start + 2].iter().all(u8::is_ascii_hexdigit)
        {
            return Err("native serial is outside the supported MAC representation");
        }
        // ASCII and exact offsets establish valid UTF-8 slice boundaries.
        let text =
            std::str::from_utf8(&bytes[start..start + 2]).map_err(|_| "invalid saved MAC text")?;
        *octet = u8::from_str_radix(text, 16).map_err(|_| "invalid saved MAC text")?;
    }
    if result == [0; 6] || result[0] & 1 != 0 {
        return Err("saved MAC text is not a nonzero unicast address");
    }
    Ok(result)
}

fn factory_mac(bytes: &[u8]) -> Result<[u8; 6], &'static str> {
    // This validator rejects duplicate/unknown/missing fields, wrong metadata,
    // invalid (OK) annotation, and disagreement between display/raw formats.
    crate::efuse::summarize(bytes)?;
    let input: Value = serde_json::from_slice(bytes).map_err(|_| "invalid selected eFuse input")?;
    if input["MAC"]["readable"].as_bool() != Some(true) {
        return Err("factory MAC correspondence requires a readable field");
    }
    let display = input["MAC"]["value"]
        .as_str()
        .and_then(|value| value.strip_suffix(" (OK)"))
        .ok_or("invalid validated factory MAC display")?;
    mac_text(display.as_bytes())
}

fn native_serial_mac(record: &Record) -> Result<[u8; 6], &'static str> {
    let native = &record.domains[0];
    if native.domain != Domain::NativeUsbSerialJtag {
        return Err("native USB domain is absent");
    }
    // Whole replay has established the exact header-first request sequence and
    // full descriptor shape. Only the complete non-LANGID serial is extracted.
    let transfer = native
        .transfers
        .iter()
        .find(|transfer| {
            transfer.request.request_type == 0x80
                && transfer.request.request == 6
                && transfer.request.value & 0xff00 == 0x0300
                && transfer.request.value & 0xff != 0
                && transfer.request.length > 2
        })
        .ok_or("native serial is outside the supported MAC representation")?;
    let bytes = &transfer.bytes;
    if bytes.len() != 36 || bytes[0] != 36 || bytes[1] != 3 {
        return Err("native serial is outside the supported MAC representation");
    }
    let mut ascii = [0; 17];
    for (index, pair) in bytes[2..].chunks_exact(2).enumerate() {
        if pair[1] != 0 {
            return Err("native serial is outside the supported MAC representation");
        }
        ascii[index] = pair[0];
    }
    mac_text(&ascii)
}

/// Replay a complete saved USB bundle, validate the exact three selected eFuse
/// fields, and emit only a MAC correspondence boolean and fixed metadata.
/// A valid different MAC returns `matches_factory_mac=false`; malformed,
/// protected or unsupported representations fail with static diagnostics.
///
/// # Errors
///
/// Returns a static error if complete USB replay fails, the selected eFuse
/// metadata or integrity annotation is invalid, the factory field is protected,
/// or the native serial is not exactly17 colon-separated ASCII hexadecimal
/// characters representing a nonzero unicast MAC. No input values are formatted.
pub(crate) fn usb_factory_mac_relation(
    capture: &Path,
    efuse_input: &Path,
) -> Result<UsbFactoryMacRelation, &'static str> {
    let record = complete_record(capture)?;
    let factory = factory_mac(&crate::read_input(efuse_input)?)?;
    let serial = native_serial_mac(&record)?;
    Ok(UsbFactoryMacRelation {
        schema_version: 1,
        kind: Kind::UsbFactoryMacRelation,
        whole_usb_capture_replayed: true,
        selected_efuse_fields_validated: true,
        native_serial_code_units: 17,
        matches_factory_mac: factory == serial,
    })
}

fn fletcher16(bytes: &[u8]) -> u16 {
    // AN978 rev1.5 section1.1.43: initial sums ff, chunks at most20 bytes,
    // carry reductions after each chunk and once more at the end. With this
    // chunk bound both intermediate u16 sums stay below65536.
    let (mut first, mut second) = (0xff_u16, 0xff_u16);
    for chunk in bytes.chunks(20) {
        for &byte in chunk {
            first += u16::from(byte);
            second += first;
        }
        first = (first & 0xff) + (first >> 8);
        second = (second & 0xff) + (second >> 8);
    }
    first = (first & 0xff) + (first >> 8);
    second = (second & 0xff) + (second >> 8);
    (second << 8) | first
}

fn configuration(bytes: &[u8]) -> Result<BridgeConfigurationAudit, &'static str> {
    // The declared method admits this observed version-one raw encoding:
    // little-endian ConfigSize and high-byte-first checksum. AN978 gives its
    // exact offsets/coverage/algorithm but does not expressly specify byte order.
    // No fallback endianness, alternative checksum or unsupported layout passes.
    if bytes.len() != 678 || bytes[2] != 1 || u16::from_le_bytes([bytes[0], bytes[1]]) != 678 {
        return Err("unsupported CP2102N configuration size or layout");
    }
    if fletcher16(&bytes[..676]) != u16::from_be_bytes([bytes[676], bytes[677]]) {
        return Err("CP2102N configuration Fletcher16 did not validate");
    }
    Ok(BridgeConfigurationAudit {
        schema_version: 1,
        kind: Kind::Cp2102nConfigurationIntegrity,
        whole_usb_capture_replayed: true,
        configuration_bytes: 678,
        configuration_layout_version: 1,
        little_endian_size_matches_length: true,
        checksum_coverage_bytes: 676,
        big_endian_fletcher16_valid: true,
        update_byte_is_default_ff: bytes[4] == 0xff,
        lock_state_qualified: false,
        programming_permission_qualified: false,
    })
}

/// Replay the complete USB bundle and validate exactly678 CP2102N version-one
/// bytes. AN978 Fletcher16 covers bytes0..676, excluding its two checksum bytes.
/// Header size is little-endian and the stored checksum is big-endian under the
/// declared observed layout; neither lock state nor write permission is inferred.
///
/// # Errors
///
/// Returns a static error for a failed complete USB replay, absent configuration
/// transfer, unsupported size/version/size-header encoding, or checksum mismatch.
pub(crate) fn bridge_configuration_audit(
    capture: &Path,
) -> Result<BridgeConfigurationAudit, &'static str> {
    let record = complete_record(capture)?;
    let bridge = &record.domains[1];
    if bridge.domain != Domain::Cp2102nBridge {
        return Err("USB bridge domain is absent");
    }
    let transfer = bridge
        .transfers
        .iter()
        .find(|transfer| {
            transfer.request.request_type == 0xc0
                && transfer.request.request == 0xff
                && transfer.request.value == 0x000e
                && transfer.request.index == 0
        })
        .ok_or("CP2102N configuration transfer is absent")?;
    configuration(&transfer.bytes)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use usb_observer::{Domain, FORMAT_VERSION, Record, Transfer, observe_domain};

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "identity-audit-{}-{}",
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

    fn encoded(value: &impl Serialize) -> Vec<u8> {
        let mut bytes = serde_json::to_vec_pretty(value).unwrap();
        bytes.push(b'\n');
        bytes
    }
    fn hash(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
    fn serial(text: &str) -> Vec<u8> {
        let mut bytes = vec![2 + text.len() as u8 * 2, 3];
        for unit in text.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes
    }
    fn config() -> Vec<u8> {
        let mut bytes = vec![0; 678];
        bytes[..2].copy_from_slice(&678_u16.to_le_bytes());
        bytes[2] = 1;
        bytes[4] = 255;
        let sum = independent_fletcher(&bytes[..676]);
        bytes[676..].copy_from_slice(&sum.to_be_bytes());
        bytes
    }
    fn independent_fletcher(bytes: &[u8]) -> u16 {
        let (mut one, mut two) = (0_u32, 0_u32);
        for byte in bytes {
            one = (one + u32::from(*byte)) % 255;
            two = (two + one) % 255;
        }
        let one = if one == 0 { 255 } else { one };
        let two = if two == 0 { 255 } else { two };
        ((two << 8) | one) as u16
    }
    fn fixture(native_text: &str) -> Record {
        Record {
            version: FORMAT_VERSION,
            domains: Domain::ALL.map(|domain| {
                observe_domain(domain, 1, 0, |request| {
                    let value = if request.value == 0x0100 {
                        let (vid, pid) = domain.ids();
                        let mut bytes = vec![18, 1, 0, 2, 0, 0, 0, 64];
                        bytes.extend_from_slice(&vid.to_le_bytes());
                        bytes.extend_from_slice(&pid.to_le_bytes());
                        bytes.extend_from_slice(&[0, 1, 1, 2, 3, 1]);
                        bytes
                    } else if request.value == 0x0300 {
                        vec![4, 3, 9, 4]
                    } else if request.request == 6 {
                        let text = if domain == Domain::NativeUsbSerialJtag {
                            native_text
                        } else {
                            "synthetic-bridge"
                        };
                        serial(text)[..usize::from(request.length)].to_vec()
                    } else if request.value == 0x370b {
                        vec![0x21]
                    } else {
                        config()
                    };
                    Transfer {
                        request,
                        status: 0,
                        returned_length: value.len() as u32,
                        bytes: value,
                    }
                })
            }),
        }
    }
    fn save_capture(path: &Path, record: &Record) {
        let headers = [
            "IOKit.framework/Headers/usb/IOUSBLib.h",
            "IOKit.framework/Headers/usb/USB.h",
            "IOKit.framework/Headers/IOCFPlugIn.h",
            "IOKit.framework/Headers/IOKitLib.h",
            "CoreFoundation.framework/Headers/CFPlugInCOM.h",
            "CoreFoundation.framework/Headers/CFUUID.h",
            "CoreFoundation.framework/Headers/CFNumber.h",
            "CoreFoundation.framework/Headers/CFString.h",
        ];
        let inputs = encoded(
            &json!({"sdk_version":"27.0","bindgen":"0.72.1","libclang":"21.1.8","headers":headers.map(|path|json!({"path":path,"sha256":"0".repeat(64)})),"bindings_sha256":"0".repeat(64)}),
        );
        let raw = encoded(record);
        let summary = encoded(&usb_observer::summarize(record).unwrap());
        fs::write(path.join("record.json"), &raw).unwrap();
        fs::write(path.join("native_provenance.json"), &inputs).unwrap();
        fs::write(path.join("summary.json"), &summary).unwrap();
        fs::write(path.join("capture-status.json"),encoded(&json!({"schema_version":2,"success":true,"error":null,"record_sha256":hash(&raw),"provenance_sha256":hash(&inputs),"summary_sha256":hash(&summary)}))).unwrap();
    }
    fn efuse() -> Value {
        let mut object = serde_json::Map::new();
        for (name, count, block, word, pos) in [
            ("MAC", 6, 1, 0, 0),
            ("MAC_EXT", 2, 1, 1, 16),
            ("OPTIONAL_UNIQUE_ID", 16, 2, 0, 0),
        ] {
            let raw = (1..=count).map(|n| n as u8).collect::<Vec<_>>();
            let is_mac = name != "OPTIONAL_UNIQUE_ID";
            let mut display = raw.clone();
            if is_mac {
                display.reverse();
            }
            let separator = if is_mac { ":" } else { " " };
            let mut value = display
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(separator);
            if is_mac {
                value.push_str(" (OK)");
            }
            object.insert(name.into(),json!({"name":name,"value":value,"raw_value":format!("0x{}",raw.iter().map(|b|format!("{b:02x}")).collect::<String>()),"readable":true,"writeable":false,"description":"synthetic","category":"synthetic","block":block,"word":word,"pos":pos,"efuse_type":format!("bytes:{count}"),"bit_len":count*8}));
        }
        Value::Object(object)
    }
    fn native_text() -> String {
        [6, 5, 4, 3, 2, 1]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(":")
    }

    #[test]
    fn complete_replay_and_efuse_validation_gate_the_relation() {
        let directory = Temp::new();
        let capture = directory.0.join("capture");
        fs::create_dir(&capture).unwrap();
        save_capture(&capture, &fixture(&native_text()));
        let efuse_path = directory.0.join("efuse.json");
        fs::write(&efuse_path, encoded(&efuse())).unwrap();
        let mut protected = efuse();
        protected["MAC"]["readable"] = json!(false);
        fs::write(&efuse_path, encoded(&protected)).unwrap();
        assert!(usb_factory_mac_relation(&capture, &efuse_path).is_err());
        fs::write(&efuse_path, encoded(&efuse())).unwrap();
        assert!(
            usb_factory_mac_relation(&capture, &efuse_path)
                .unwrap()
                .matches_factory_mac
        );
        let other = [8, 5, 4, 3, 2, 1]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":");
        save_capture(&capture, &fixture(&other));
        assert!(
            !usb_factory_mac_relation(&capture, &efuse_path)
                .unwrap()
                .matches_factory_mac
        );
        let mut malformed = efuse();
        malformed["MAC"]["raw_value"] = json!("0x000000000000");
        fs::write(&efuse_path, encoded(&malformed)).unwrap();
        assert!(usb_factory_mac_relation(&capture, &efuse_path).is_err());
        fs::write(&efuse_path, encoded(&efuse())).unwrap();
        let mut receipt: Value =
            serde_json::from_slice(&fs::read(capture.join("capture-status.json")).unwrap())
                .unwrap();
        receipt["success"] = json!(false);
        fs::write(capture.join("capture-status.json"), encoded(&receipt)).unwrap();
        assert!(usb_factory_mac_relation(&capture, &efuse_path).is_err());
    }

    #[test]
    fn exact_checksum_coverage_order_size_and_layout_controls() {
        let good = config();
        assert!(configuration(&good).unwrap().big_endian_fletcher16_valid);
        for position in 0..678 {
            for bit in 0..8 {
                let mut changed = good.clone();
                changed[position] ^= 1 << bit;
                assert!(configuration(&changed).is_err());
            }
        }
        for size in [0, 677, 679] {
            assert!(configuration(&vec![0; size]).is_err());
        }
        let mut changed = good.clone();
        changed.swap(0, 1);
        let sum = independent_fletcher(&changed[..676]);
        changed[676..].copy_from_slice(&sum.to_be_bytes());
        assert!(configuration(&changed).is_err());
        let mut changed = good.clone();
        changed[2] = 2;
        let sum = independent_fletcher(&changed[..676]);
        changed[676..].copy_from_slice(&sum.to_be_bytes());
        assert!(configuration(&changed).is_err());
        let mut changed = good;
        changed.swap(676, 677);
        assert!(configuration(&changed).is_err());
        for length in [0, 1, 19, 20, 21, 675, 676] {
            let data = (0..length).map(|n| (n * 73 + 91) as u8).collect::<Vec<_>>();
            assert_eq!(fletcher16(&data), independent_fletcher(&data));
        }
    }

    #[test]
    fn whole_capture_bridge_audit_keeps_lock_unknown_and_values_private() {
        let directory = Temp::new();
        save_capture(&directory.0, &fixture(&native_text()));
        let report = bridge_configuration_audit(&directory.0).unwrap();
        assert!(!report.lock_state_qualified && !report.programming_permission_qualified);
        let output = String::from_utf8(encoded(&report)).unwrap();
        assert!(!output.contains(&native_text()));
        fs::write(directory.0.join("record.json"), b"{}").unwrap();
        assert!(bridge_configuration_audit(&directory.0).is_err());
        assert!(mac_text(b"synthetic-private").is_err());
        let zero = [0_u8; 6]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":");
        assert!(mac_text(zero.as_bytes()).is_err());
    }
}
