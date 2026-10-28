//! Parse the explicit macOS USB host schema and emit only fixed-field counts.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a present string while distinguishing missing fields from null.
fn present_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(Some)
}

fn present_nodes<'de, D>(deserializer: D) -> Result<Option<Vec<Node>>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
struct Input {
    #[serde(rename = "SPUSBHostDataType")]
    hosts: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    #[serde(rename = "Driver", default, deserialize_with = "present_string")]
    controller_driver: Option<String>,
    #[serde(
        rename = "USBDeviceKeyVendorID",
        default,
        deserialize_with = "present_string"
    )]
    vendor: Option<String>,
    #[serde(
        rename = "USBDeviceKeyProductID",
        default,
        deserialize_with = "present_string"
    )]
    product: Option<String>,
    #[serde(
        rename = "USBDeviceKeySerialNumber",
        default,
        deserialize_with = "present_string"
    )]
    serial: Option<String>,
    #[serde(rename = "_items", default, deserialize_with = "present_nodes")]
    children: Option<Vec<Node>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Domain {
    NativeUsbSerialJtag,
    Cp2102nBridge,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Counts {
    domain: Domain,
    device_count: u64,
    serial_present_count: u64,
    serial_empty_count: u64,
    serial_absent_count: u64,
}

// Holds at most one private serial per domain; never implements Debug.
struct Observation {
    counts: Counts,
    sole_serial: Option<String>,
}

impl Observation {
    fn new(domain: Domain) -> Self {
        Self {
            counts: Counts::new(domain),
            sole_serial: None,
        }
    }

    fn record(&mut self, serial: Option<String>) {
        self.counts.record(serial.as_deref());
        self.sole_serial = if self.counts.device_count == 1 {
            serial
        } else {
            None
        };
    }
}

impl Counts {
    fn new(domain: Domain) -> Self {
        Self {
            domain,
            device_count: 0,
            serial_present_count: 0,
            serial_empty_count: 0,
            serial_absent_count: 0,
        }
    }

    fn record(&mut self, serial: Option<&str>) {
        self.device_count += 1;
        match serial {
            Some(value) if value.trim().is_empty() => self.serial_empty_count += 1,
            Some(_) => self.serial_present_count += 1,
            None => self.serial_absent_count += 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum SourceFormat {
    #[serde(rename = "macos_sp_usb_host_data_type")]
    MacosSpUsbHostDataType,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Observer {
    AttachedUsbHost,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Report {
    schema_version: u32,
    source_format: SourceFormat,
    observer: Observer,
    domains: [Counts; 2],
}

/// Only exactly four hexadecimal digits after `0x` are accepted.
fn usb_id(input: &str) -> Result<u16, &'static str> {
    let digits = input
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 4 && digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or("invalid USB identifier metadata")?;
    u16::from_str_radix(digits, 16).map_err(|_| "invalid USB identifier metadata")
}

#[derive(Clone, Copy)]
enum Position {
    Root,
    Nested,
}

fn visit(
    node: Node,
    domains: &mut [Observation; 2],
    position: Position,
) -> Result<(), &'static str> {
    match (node.vendor, node.product) {
        (Some(vendor), Some(product)) => {
            let index = match (usb_id(&vendor)?, usb_id(&product)?) {
                (0x303a, 0x1001) => Some(0),
                (0x10c4, 0xea60) => Some(1),
                _ => None,
            };
            if let Some(index) = index {
                domains[index].record(node.serial);
            }
        }
        (None, None)
            if node.serial.is_none()
                && (node.children.is_some()
                    || (matches!(position, Position::Root)
                        && node.controller_driver.is_some())) => {}
        _ => return Err("incomplete USB identifier metadata"),
    }
    if let Some(children) = node.children {
        for child in children {
            visit(child, domains, Position::Nested)?;
        }
    }
    Ok(())
}

fn parse(bytes: &[u8]) -> Result<[Observation; 2], &'static str> {
    let input: Input = serde_json::from_slice(bytes).map_err(|_| "invalid USB host JSON schema")?;
    let mut domains = [
        Observation::new(Domain::NativeUsbSerialJtag),
        Observation::new(Domain::Cp2102nBridge),
    ];
    for host in input.hosts {
        visit(host, &mut domains, Position::Root)?;
    }
    Ok(domains)
}

pub(crate) fn summarize(bytes: &[u8]) -> Result<Report, &'static str> {
    Ok(Report {
        schema_version: 1,
        source_format: SourceFormat::MacosSpUsbHostDataType,
        observer: Observer::AttachedUsbHost,
        domains: parse(bytes)?.map(|observation| observation.counts),
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ComparisonState {
    Unchanged,
    Changed,
    Unavailable,
    Ambiguous,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct DomainComparison {
    domain: Domain,
    before_count: u64,
    after_count: u64,
    serial_property: ComparisonState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ComparisonReport {
    schema_version: u32,
    source_format: SourceFormat,
    observer: Observer,
    domains: [DomainComparison; 2],
}

fn compare_domain(before: Observation, after: Observation) -> DomainComparison {
    let before_count = before.counts.device_count;
    let after_count = after.counts.device_count;
    let serial_property = if before_count > 1 || after_count > 1 {
        ComparisonState::Ambiguous
    } else {
        match (before.sole_serial, after.sole_serial) {
            (Some(first), Some(second))
                if !first.trim().is_empty() && !second.trim().is_empty() =>
            {
                if first == second {
                    ComparisonState::Unchanged
                } else {
                    ComparisonState::Changed
                }
            }
            _ => ComparisonState::Unavailable,
        }
    };
    DomainComparison {
        domain: before.counts.domain,
        before_count,
        after_count,
        serial_property,
    }
}

/// Compare host property contents without publishing the values.
///
/// Multiple matches cannot be attributed to the same physical board. These are
/// saved host properties, which can be cached, rather than fresh USB transfers.
pub(crate) fn compare(before: &[u8], after: &[u8]) -> Result<ComparisonReport, &'static str> {
    let [before_native, before_bridge] = parse(before)?;
    let [after_native, after_bridge] = parse(after)?;
    Ok(ComparisonReport {
        schema_version: 1,
        source_format: SourceFormat::MacosSpUsbHostDataType,
        observer: Observer::AttachedUsbHost,
        domains: [
            compare_domain(before_native, after_native),
            compare_domain(before_bridge, after_bridge),
        ],
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{ComparisonState, compare};

    fn capture(serials: &[Option<&str>]) -> Vec<u8> {
        let devices: Vec<_> = serials
            .iter()
            .map(|serial| {
                let mut device = json!({
                    "USBDeviceKeyVendorID": "0x303a",
                    "USBDeviceKeyProductID": "0x1001",
                    "_name": "private unrelated metadata",
                });
                if let Some(serial) = serial {
                    device["USBDeviceKeySerialNumber"] = json!(serial);
                }
                device
            })
            .collect();
        serde_json::to_vec(&json!({"SPUSBHostDataType": [{"_items": devices}]})).unwrap()
    }

    #[test]
    fn equal_counts_do_not_hide_changed_private_values() {
        let first = ["private", "before", "sentinel"].join("-");
        let second = ["private", "after", "sentinel"].join("-");
        let before = capture(&[Some(&first)]);
        let after = capture(&[Some(&second)]);
        let unchanged = compare(&before, &before).unwrap();
        let changed = compare(&before, &after).unwrap();
        assert_eq!(
            unchanged.domains[0].serial_property,
            ComparisonState::Unchanged
        );
        assert_eq!(changed.domains[0].serial_property, ComparisonState::Changed);
        assert_eq!(
            changed.domains[1].serial_property,
            ComparisonState::Unavailable
        );
        for report in [unchanged, changed] {
            let output = serde_json::to_string(&report).unwrap();
            assert!(!output.contains(&first));
            assert!(!output.contains(&second));
            assert!(!output.contains("private unrelated metadata"));
        }
    }

    #[test]
    fn empty_absent_or_multiple_matches_remain_unqualified() {
        let one = capture(&[Some("synthetic-single")]);
        for after in [capture(&[]), capture(&[None]), capture(&[Some(" \t ")])] {
            assert_eq!(
                compare(&one, &after).unwrap().domains[0].serial_property,
                ComparisonState::Unavailable
            );
        }
        let repeated = capture(&[Some("synthetic-single"), Some("synthetic-single")]);
        for (before, after) in [(&one, &repeated), (&repeated, &one)] {
            assert_eq!(
                compare(before, after).unwrap().domains[0].serial_property,
                ComparisonState::Ambiguous
            );
        }
    }

    #[test]
    fn comparison_validates_both_complete_inputs() {
        let good = capture(&[Some("synthetic-single")]);
        let mut bad: Value = serde_json::from_slice(&good).unwrap();
        bad["SPUSBHostDataType"][0]["_items"][0]["USBDeviceKeySerialNumber"] = json!(null);
        let bad = serde_json::to_vec(&bad).unwrap();
        assert!(compare(&bad, &good).is_err());
        assert!(compare(&good, &bad).is_err());
        let mut suffix = good.clone();
        suffix.extend_from_slice(b"false");
        assert!(compare(&good, &suffix).is_err());
    }
}
