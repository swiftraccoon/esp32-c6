//! Bounded planner, strict saved-transfer replay and nonidentifying reports.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Version of the complete raw record and sanitized reports.
pub const FORMAT_VERSION: u32 = 2;
/// Upper bound on serialized private capture input.
pub const MAX_RECORD_BYTES: usize = 64 * 1024;
/// Exact CP2102N configuration length for the qualified layout.
pub const CONFIGURATION_BYTES: u16 = 678;

/// Device ownership domain; a C6 override cannot configure its separate bridge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// ESP32-C6 native USB Serial/JTAG controller.
    NativeUsbSerialJtag,
    /// Silicon Labs CP2102N USB-to-UART bridge.
    Cp2102nBridge,
}

impl Domain {
    /// Fixed domain ordering in every complete capture.
    pub const ALL: [Self; 2] = [Self::NativeUsbSerialJtag, Self::Cp2102nBridge];

    /// Return the documented vendor and product identifiers used for selection.
    pub const fn ids(self) -> (u16, u16) {
        match self {
            Self::NativeUsbSerialJtag => (0x303a, 0x1001),
            Self::Cp2102nBridge => (0x10c4, 0xea60),
        }
    }
}

/// One exact USB setup packet with an IN direction and fixed device recipient.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// bmRequestType; the planner emits only standard IN 0x80 or vendor IN 0xc0.
    pub request_type: u8,
    /// bRequest; GET_DESCRIPTOR or the qualified bridge vendor request.
    pub request: u8,
    /// wValue, in host numeric representation; the backend uses generated ABI.
    pub value: u16,
    /// wIndex, including the chosen LANGID for a serial string.
    pub index: u16,
    /// Maximum number of returned bytes.
    pub length: u16,
}

/// One private transfer result, including failures and partial returned bytes.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Transfer {
    /// Exact requested setup packet; local validation can decline submission.
    pub request: Request,
    /// Signed IOReturn or negative local validation code; zero is success.
    ///
    /// A local pre-submission rejection does not establish an on-bus request.
    /// No foreign integer is converted into a Rust enum.
    pub status: i32,
    /// Raw actual-length report, retained even if it exceeds the allocated cap.
    pub returned_length: u32,
    /// Initialized prefix bounded by the requested size and reported length.
    ///
    /// Failed-call buffer bytes are retained without asserting they are a valid
    /// response. Overreported lengths remain explicit and fail validation.
    /// Unreturned zero padding is excluded. Raw serialization is private.
    pub bytes: Vec<u8>,
}

impl fmt::Debug for Transfer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Transfer")
            .field("request", &self.request)
            .field("status", &self.status)
            .field("returned_length", &self.returned_length)
            .field("bytes", &"<redacted>")
            .finish()
    }
}

/// Raw observations for one selected device class, including setup/cleanup failure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainRecord {
    /// Expected ownership domain, verified against the fresh device descriptor.
    pub domain: Domain,
    /// Number of cached registry matches; a unique match is required for requests.
    pub matched_count: u32,
    /// Native setup status or a negative local validation code; zero is success.
    pub setup_status: i32,
    /// Native/local resource cleanup status; zero means cleanup completed.
    pub cleanup_status: i32,
    /// Ordered requests and results, stopping after the first invalid transfer.
    pub transfers: Vec<Transfer>,
}

/// Complete two-domain private capture. Missing or repeated domains are invalid.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Raw format version; unsupported versions fail explicitly.
    pub version: u32,
    /// Fixed native-then-bridge domain order.
    pub domains: [DomainRecord; 2],
}

/// Static failure category; diagnostics never carry private bytes or names.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    /// Unsupported record version, invalid JSON or unexpected fields.
    Schema,
    /// No unique device could be selected, or setup failed.
    Selection,
    /// Resource cleanup did not complete successfully.
    Cleanup,
    /// A request is missing, extra or different from the fixed planner's request.
    Sequence,
    /// A native transfer failed or reported a contradictory length.
    Transfer,
    /// Fresh device descriptor length/type/layout or VID/PID is invalid.
    DeviceDescriptor,
    /// LANGID table has malformed or duplicate entries.
    LanguageDescriptor,
    /// Serial string descriptor shape or UTF-16 encoding is invalid.
    SerialDescriptor,
    /// The bridge part code is outside the qualified CP2102N set.
    BridgePart,
    /// Bridge configuration has an unsupported length/layout version.
    BridgeConfiguration,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Schema => "invalid USB observer record schema",
            Self::Selection => "USB observer needs one successfully selected device per domain",
            Self::Cleanup => "USB observer resource cleanup failed",
            Self::Sequence => "USB observer transfer sequence does not match the planner",
            Self::Transfer => "USB observer transfer failed or returned an invalid length",
            Self::DeviceDescriptor => "invalid USB device descriptor",
            Self::LanguageDescriptor => "invalid USB language descriptor",
            Self::SerialDescriptor => "invalid USB serial descriptor",
            Self::BridgePart => "unsupported USB bridge part code",
            Self::BridgeConfiguration => "unsupported USB bridge configuration layout",
        })
    }
}

impl std::error::Error for Error {}

/// Whether a serial string index/content was observed, without its value.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialPresence {
    /// Fresh device descriptor has iSerialNumber equal to zero.
    Absent,
    /// The selected serial descriptor decodes to empty or whitespace-only text.
    Empty,
    /// The selected serial descriptor decodes to nonempty text.
    Present,
}

/// The read-only layout observation does not qualify bridge lock state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LockState {
    /// No write/lock test or independently qualified lock interpretation occurred.
    Unknown,
}

/// Source-qualified checksum coverage; configuration bytes are not silently trusted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumQualification {
    /// No primary-source checksum algorithm is implemented by this method.
    Unqualified,
}

/// Fixed-field bridge configuration metadata, never its raw contents.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeSummary {
    /// Qualified layout version, observed at byte two.
    pub configuration_version: u8,
    /// Whether byte four equals the documented default; this is not lock state.
    pub update_byte_is_default_ff: bool,
    /// Explicitly unqualified lock state.
    pub lock_state: LockState,
    /// Explicit checksum qualification limit.
    pub checksum: ChecksumQualification,
}

/// Sanitized observations for one device class.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainSummary {
    /// Native controller or bridge domain.
    pub domain: Domain,
    /// Exactly one cached registry match was selected.
    pub matched_count: u32,
    /// Number of successful, shape-validated transfers.
    pub transfer_count: usize,
    /// Presence of the fresh serial string without its contents.
    pub serial: SerialPresence,
    /// UTF-16 code units, absent when the device advertises no serial index.
    pub serial_code_units: Option<usize>,
    /// Bridge-only fixed layout metadata; absent for the native controller.
    pub bridge: Option<BridgeSummary>,
}

/// Complete sanitized report derived from the exact recorded request sequence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    /// Sanitized report version.
    pub schema_version: u32,
    /// Fixed native-then-bridge summaries.
    pub domains: [DomainSummary; 2],
}

const DEVICE: Request = Request {
    request_type: 0x80,
    request: 0x06,
    value: 0x0100,
    index: 0,
    length: 18,
};

const LANGUAGES: Request = Request {
    request_type: 0x80,
    request: 0x06,
    value: 0x0300,
    index: 0,
    length: 4,
};

const PART: Request = Request {
    request_type: 0xc0,
    request: 0xff,
    value: 0x370b,
    index: 0,
    length: 1,
};

const CONFIGURATION: Request = Request {
    request_type: 0xc0,
    request: 0xff,
    value: 0x000e,
    index: 0,
    length: CONFIGURATION_BYTES,
};

struct ParsedDomain {
    summary: DomainSummary,
    // No Debug implementation: decoded raw serial contents stay private.
    serial_value: Option<String>,
}

fn bytes(transfer: &Transfer, expected: Request) -> Result<&[u8], Error> {
    if transfer.request != expected {
        return Err(Error::Sequence);
    }
    if transfer.status != 0
        || transfer.returned_length > u32::from(expected.length)
        || usize::try_from(transfer.returned_length).ok() != Some(transfer.bytes.len())
    {
        return Err(Error::Transfer);
    }
    Ok(&transfer.bytes)
}

fn device(bytes: &[u8], domain: Domain) -> Result<u8, Error> {
    if bytes.len() != 18
        || bytes[0] != 18
        || bytes[1] != 1
        || !matches!(bytes[7], 8 | 16 | 32 | 64)
        || bytes[17] == 0
        || (
            u16::from_le_bytes([bytes[8], bytes[9]]),
            u16::from_le_bytes([bytes[10], bytes[11]]),
        ) != domain.ids()
    {
        return Err(Error::DeviceDescriptor);
    }
    Ok(bytes[16])
}

fn string_body(bytes: &[u8], error: Error) -> Result<&[u8], Error> {
    if bytes.len() < 2
        || bytes[0] as usize != bytes.len()
        || bytes[1] != 3
        || !bytes.len().is_multiple_of(2)
    {
        return Err(error);
    }
    Ok(&bytes[2..])
}

fn declared_length(
    prefix: &[u8],
    prefix_length: usize,
    minimum: usize,
    error: Error,
) -> Result<usize, Error> {
    if prefix.len() < 2 || prefix.len() != prefix_length || prefix[1] != 3 {
        return Err(error);
    }
    let declared = usize::from(prefix[0]);
    if !(minimum..=254).contains(&declared) || !declared.is_multiple_of(2) {
        return Err(error);
    }
    Ok(declared)
}

fn language(bytes: &[u8]) -> Result<u16, Error> {
    let body = string_body(bytes, Error::LanguageDescriptor)?;
    if body.is_empty() {
        return Err(Error::LanguageDescriptor);
    }
    let mut codes = Vec::new();
    for pair in body.chunks_exact(2) {
        let code = u16::from_le_bytes([pair[0], pair[1]]);
        if code == 0 || codes.contains(&code) {
            return Err(Error::LanguageDescriptor);
        }
        codes.push(code);
    }
    // Select the first device-advertised language without guessing a LANGID.
    Ok(codes[0])
}

fn serial(bytes: &[u8]) -> Result<(String, usize), Error> {
    let body = string_body(bytes, Error::SerialDescriptor)?;
    let units: Vec<_> = body
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let text = String::from_utf16(&units).map_err(|_| Error::SerialDescriptor)?;
    if text.chars().any(char::is_control) {
        return Err(Error::SerialDescriptor);
    }
    Ok((text, units.len()))
}

fn execute<F>(domain: Domain, mut read: F) -> Result<ParsedDomain, Error>
where
    F: FnMut(Request) -> Result<Transfer, Error>,
{
    let first = read(DEVICE)?;
    let serial_index = device(bytes(&first, DEVICE)?, domain)?;
    let mut count = 1;
    let (serial_value, serial_presence, serial_code_units) = if serial_index == 0 {
        (None, SerialPresence::Absent, None)
    } else {
        let languages = read(LANGUAGES)?;
        let prefix = bytes(&languages, LANGUAGES)?;
        let declared = declared_length(prefix, 4, 4, Error::LanguageDescriptor)?;
        if u16::from_le_bytes([prefix[2], prefix[3]]) == 0 {
            return Err(Error::LanguageDescriptor);
        }
        count += 1;
        let langid = if declared == prefix.len() {
            language(prefix)?
        } else {
            let request = Request {
                length: declared as u16,
                ..LANGUAGES
            };
            let complete = read(request)?;
            let body = bytes(&complete, request)?;
            if !body.starts_with(prefix) {
                return Err(Error::LanguageDescriptor);
            }
            count += 1;
            language(body)?
        };
        let request = Request {
            value: 0x0300 | u16::from(serial_index),
            index: langid,
            length: 2,
            ..LANGUAGES
        };
        let transfer = read(request)?;
        let prefix = bytes(&transfer, request)?;
        let declared = declared_length(prefix, 2, 2, Error::SerialDescriptor)?;
        count += 1;
        let (value, units) = if declared == prefix.len() {
            serial(prefix)?
        } else {
            let request = Request {
                length: declared as u16,
                ..request
            };
            let complete = read(request)?;
            let body = bytes(&complete, request)?;
            if !body.starts_with(prefix) {
                return Err(Error::SerialDescriptor);
            }
            count += 1;
            serial(body)?
        };
        let presence = if value.trim().is_empty() {
            SerialPresence::Empty
        } else {
            SerialPresence::Present
        };
        (Some(value), presence, Some(units))
    };
    let bridge = if domain == Domain::Cp2102nBridge {
        let part = read(PART)?;
        if !matches!(bytes(&part, PART)?, [0x20..=0x22]) {
            return Err(Error::BridgePart);
        }
        let configuration = read(CONFIGURATION)?;
        let body = bytes(&configuration, CONFIGURATION)?;
        if body.len() != usize::from(CONFIGURATION_BYTES) || body[2] != 1 {
            return Err(Error::BridgeConfiguration);
        }
        count += 2;
        Some(BridgeSummary {
            configuration_version: body[2],
            update_byte_is_default_ff: body[4] == 0xff,
            lock_state: LockState::Unknown,
            checksum: ChecksumQualification::Unqualified,
        })
    } else {
        None
    };
    Ok(ParsedDomain {
        summary: DomainSummary {
            domain,
            matched_count: 1,
            transfer_count: count,
            serial: serial_presence,
            serial_code_units,
            bridge,
        },
        serial_value,
    })
}

/// Drive the exact IN-only plan and preserve each raw result before checking it.
///
/// Zero/ambiguous selection or setup failure sends no request. A failed or
/// malformed transfer is retained as the last transfer and ends that domain.
/// Callers can set [`DomainRecord::cleanup_status`] after releasing native
/// resources. All serialized raw records belong in private storage.
pub fn observe_domain<F>(
    domain: Domain,
    matched_count: u32,
    setup_status: i32,
    mut read: F,
) -> DomainRecord
where
    F: FnMut(Request) -> Transfer,
{
    let mut record = DomainRecord {
        domain,
        matched_count,
        setup_status,
        cleanup_status: 0,
        transfers: Vec::new(),
    };
    if matched_count == 1 && setup_status == 0 {
        let _ = execute(domain, |request| {
            let transfer = read(request);
            record.transfers.push(transfer.clone());
            Ok(transfer)
        });
    }
    record
}

fn replay(record: &DomainRecord) -> Result<ParsedDomain, Error> {
    if record.matched_count != 1 || record.setup_status != 0 {
        return if record.transfers.is_empty() {
            Err(Error::Selection)
        } else {
            Err(Error::Sequence)
        };
    }
    let mut cursor = 0;
    let result = execute(record.domain, |expected| {
        let transfer = record.transfers.get(cursor).ok_or(Error::Sequence)?;
        if transfer.request != expected {
            return Err(Error::Sequence);
        }
        cursor += 1;
        Ok(transfer.clone())
    });
    if cursor != record.transfers.len() {
        return Err(Error::Sequence);
    }
    if record.cleanup_status != 0 {
        return Err(Error::Cleanup);
    }
    result
}

/// Decode bounded private JSON without echoing parser details or raw contents.
///
/// # Errors
/// Returns [`Error::Schema`] for invalid JSON, unknown/duplicate fields,
/// oversized input or unsupported versions. Semantic transfer validation is
/// performed by [`summarize`].
pub fn parse(bytes: &[u8]) -> Result<Record, Error> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(Error::Schema);
    }
    let record: Record = serde_json::from_slice(bytes).map_err(|_| Error::Schema)?;
    if record.version != FORMAT_VERSION {
        return Err(Error::Schema);
    }
    Ok(record)
}

/// Replay both exact request sequences and return only nonidentifying metadata.
///
/// # Errors
/// Returns a static failure for wrong domain order/version, ambiguous/missing
/// selection, setup/cleanup failure, extra/truncated/wrong requests, failed
/// transfers, malformed descriptors or unsupported bridge layout. No private
/// transfer bytes, raw serials or configuration values occur in diagnostics.
pub fn summarize(record: &Record) -> Result<Summary, Error> {
    if record.version != FORMAT_VERSION
        || record.domains.each_ref().map(|entry| entry.domain) != Domain::ALL
    {
        return Err(Error::Schema);
    }
    let [native, bridge] = record.domains.each_ref().map(replay);
    Ok(Summary {
        schema_version: FORMAT_VERSION,
        domains: [native?.summary, bridge?.summary],
    })
}

/// Whether two unique fresh serial observations have equal decoded contents.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialComparison {
    /// Both nonempty UTF-16 strings decode to identical contents.
    Unchanged,
    /// Both nonempty UTF-16 strings decode to different contents.
    Changed,
    /// At least one descriptor has no serial index or has empty content.
    Unavailable,
}

/// Complete fixed-field serial comparison without serial values or raw hashes.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    /// Comparison format version.
    pub schema_version: u32,
    /// Fixed native-then-bridge comparison order.
    pub domains: [DomainComparison; 2],
}

/// Serial equality result for one device domain.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainComparison {
    /// Compared device class.
    pub domain: Domain,
    /// Decoded serial equality; selection errors remain errors, not comparisons.
    pub serial: SerialComparison,
    /// Exact equality of the complete bridge configuration response, when applicable.
    ///
    /// Equality does not qualify checksum integrity, meaning or lock state.
    pub configuration_bytes_unchanged: Option<bool>,
}

/// Compare two complete validated capture records without publishing identifiers.
///
/// # Errors
/// Returns the same semantic failures as [`summarize`] for either input.
pub fn compare(before: &Record, after: &Record) -> Result<Comparison, Error> {
    summarize(before)?;
    summarize(after)?;
    let mut results = Vec::new();
    for (first, second) in before.domains.iter().zip(&after.domains) {
        let left = replay(first)?;
        let right = replay(second)?;
        let serial = match (left.serial_value, right.serial_value) {
            (Some(left), Some(right)) if !left.trim().is_empty() && !right.trim().is_empty() => {
                if left == right {
                    SerialComparison::Unchanged
                } else {
                    SerialComparison::Changed
                }
            }
            _ => SerialComparison::Unavailable,
        };
        let configuration_bytes_unchanged = if first.domain == Domain::Cp2102nBridge {
            // Both complete records were already validated, so each bridge's
            // final transfer is its exact configuration read.
            Some(
                first.transfers.last().map(|transfer| &transfer.bytes)
                    == second.transfers.last().map(|transfer| &transfer.bytes),
            )
        } else {
            None
        };
        results.push(DomainComparison {
            domain: first.domain,
            serial,
            configuration_bytes_unchanged,
        });
    }
    let domains = results.try_into().map_err(|_| Error::Schema)?;
    Ok(Comparison {
        schema_version: FORMAT_VERSION,
        domains,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(domain: Domain, serial_index: u8) -> Vec<u8> {
        let (vendor, product) = domain.ids();
        let mut body = vec![
            18,
            1,
            0,
            2,
            0,
            0,
            0,
            64,
            0,
            0,
            0,
            0,
            0,
            1,
            1,
            2,
            serial_index,
            1,
        ];
        body[8..10].copy_from_slice(&vendor.to_le_bytes());
        body[10..12].copy_from_slice(&product.to_le_bytes());
        body
    }

    fn text_descriptor(text: &str) -> Vec<u8> {
        let units: Vec<_> = text.encode_utf16().collect();
        let mut body = vec![u8::try_from(2 + units.len() * 2).unwrap(), 3];
        for unit in units {
            body.extend_from_slice(&unit.to_le_bytes());
        }
        body
    }

    fn success(request: Request, bytes: Vec<u8>) -> Transfer {
        Transfer {
            request,
            status: 0,
            returned_length: u32::try_from(bytes.len()).unwrap(),
            bytes,
        }
    }

    fn response(
        domain: Domain,
        text: &str,
        serial_index: u8,
        languages: &[u8],
        request: Request,
    ) -> Transfer {
        let mut body = match request.value {
            0x0100 => descriptor(domain, serial_index),
            0x0300 => languages.to_vec(),
            0x370b => vec![0x20],
            0x000e => {
                let mut body = vec![0; usize::from(CONFIGURATION_BYTES)];
                body[2] = 1;
                body[4] = 0xff;
                // Arbitrary private contents are not configuration integrity.
                body[100..106].copy_from_slice(b"secret");
                body
            }
            _ => {
                assert_eq!(request.request_type, 0x80);
                assert_eq!(request.request, 0x06);
                assert_eq!(request.value, 0x0300 | u16::from(serial_index));
                assert_eq!(request.index, 0x0409);
                text_descriptor(text)
            }
        };
        body.truncate(usize::from(request.length));
        success(request, body)
    }

    fn fixture_domain_languages(
        domain: Domain,
        text: &str,
        serial_index: u8,
        languages: &[u8],
    ) -> DomainRecord {
        observe_domain(domain, 1, 0, |request| {
            response(domain, text, serial_index, languages, request)
        })
    }

    fn fixture_domain(domain: Domain, text: &str, serial_index: u8) -> DomainRecord {
        fixture_domain_languages(domain, text, serial_index, &[6, 3, 9, 4, 17, 4])
    }

    fn fixture() -> Record {
        Record {
            version: FORMAT_VERSION,
            domains: [
                fixture_domain(Domain::NativeUsbSerialJtag, "synthetic-native-marker", 3),
                fixture_domain(Domain::Cp2102nBridge, "synthetic-bridge-marker", 3),
            ],
        }
    }

    #[test]
    fn complete_capture_replays_exact_plan_without_public_values() {
        let record = fixture();
        let encoded = serde_json::to_vec(&record).unwrap();
        assert_eq!(parse(&encoded).unwrap(), record);
        let summary = summarize(&record).unwrap();
        assert_eq!(summary.domains[0].transfer_count, 5);
        assert_eq!(summary.domains[1].transfer_count, 7);
        assert_eq!(summary.domains[0].serial, SerialPresence::Present);
        let bridge = summary.domains[1].bridge.as_ref().unwrap();
        assert_eq!(bridge.configuration_version, 1);
        assert!(bridge.update_byte_is_default_ff);
        assert_eq!(bridge.lock_state, LockState::Unknown);
        assert_eq!(bridge.checksum, ChecksumQualification::Unqualified);
        let output = serde_json::to_string(&summary).unwrap();
        assert!(!output.contains("synthetic-native-marker"));
        assert!(!output.contains("synthetic-bridge-marker"));
        assert!(!output.contains("secret"));
        let debug = format!("{record:?}");
        assert!(!debug.contains("synthetic-native-marker"));
        assert!(!debug.contains("secret"));
        assert!(debug.contains("<redacted>"));
        for entry in &record.domains {
            assert!(
                entry
                    .transfers
                    .iter()
                    .all(|transfer| transfer.request.request_type & 0x80 != 0)
            );
        }
    }

    #[test]
    fn absent_serial_skips_language_and_string_transfers() {
        let record = Record {
            version: FORMAT_VERSION,
            domains: [
                fixture_domain(Domain::NativeUsbSerialJtag, "unused", 0),
                fixture_domain(Domain::Cp2102nBridge, "unused", 0),
            ],
        };
        let summary = summarize(&record).unwrap();
        assert_eq!(summary.domains[0].transfer_count, 1);
        assert_eq!(summary.domains[1].transfer_count, 3);
        assert!(summary.domains.iter().all(
            |entry| entry.serial == SerialPresence::Absent && entry.serial_code_units.is_none()
        ));
    }

    #[test]
    fn short_prefixes_then_exact_lengths_form_the_complete_plan() {
        let record = Record {
            version: FORMAT_VERSION,
            domains: Domain::ALL
                .map(|domain| fixture_domain_languages(domain, "X", 3, &[4, 3, 9, 4])),
        };
        let header = Request {
            value: 0x0303,
            index: 0x0409,
            length: 2,
            ..LANGUAGES
        };
        let expected = [
            DEVICE,
            LANGUAGES,
            header,
            Request {
                length: 4,
                ..header
            },
        ];
        assert_eq!(
            record.domains[0]
                .transfers
                .iter()
                .map(|entry| entry.request)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(summarize(&record).unwrap().domains[0].transfer_count, 4);
        assert_eq!(summarize(&record).unwrap().domains[1].transfer_count, 6);
        let empty = fixture_domain_languages(Domain::NativeUsbSerialJtag, "", 3, &[4, 3, 9, 4]);
        assert_eq!(empty.transfers.len(), 3);
        assert_eq!(
            replay(&empty).unwrap().summary.serial,
            SerialPresence::Empty
        );
        // More than one LANGID requires its full declared table, rather than
        // admitting the first four bytes as the complete language list.
        let multiple = fixture();
        assert_eq!(multiple.domains[0].transfers[1].request.length, 4);
        assert_eq!(multiple.domains[0].transfers[2].request.length, 6);
        assert!(
            multiple.domains[0].transfers[2]
                .bytes
                .starts_with(&multiple.domains[0].transfers[1].bytes)
        );
    }

    #[test]
    fn malformed_prefixes_changed_headers_and_short_bodies_are_rejected() {
        for mutation in 0..17 {
            let mut record = fixture();
            let entries = &mut record.domains[0].transfers;
            match mutation {
                0 => entries[1].bytes[0] = 0,
                1 => entries[1].bytes[0] = 3,
                2 => entries[1].bytes[1] = 2,
                3 => {
                    entries[1].bytes.pop();
                    entries[1].returned_length -= 1;
                }
                4 => entries[1].bytes[0] = 255,
                5 => entries[2].bytes[2] ^= 1,
                6 => entries[2].bytes[0] -= 2,
                7 => {
                    entries[2].bytes.pop();
                    entries[2].returned_length -= 1;
                }
                8 => entries[3].bytes[0] = 1,
                9 => entries[3].bytes[0] = 3,
                10 => entries[3].bytes[0] = 255,
                11 => entries[3].bytes[1] = 2,
                12 => {
                    entries[3].bytes.pop();
                    entries[3].returned_length -= 1;
                }
                13 => entries[4].bytes[0] -= 2,
                14 => entries[4].bytes[1] = 2,
                15 => {
                    entries[4].bytes.pop();
                    entries[4].returned_length -= 1;
                }
                _ => entries[1].request.length = 255,
            }
            assert!(
                summarize(&record).is_err(),
                "prefix/body mutation {mutation}"
            );
        }
    }

    #[test]
    fn invalid_prefix_or_repeated_header_stops_before_later_requests() {
        for bad_call in 2..=5 {
            let mut calls = 0;
            let record = observe_domain(Domain::Cp2102nBridge, 1, 0, |request| {
                calls += 1;
                let mut transfer =
                    response(Domain::Cp2102nBridge, "X", 3, &[6, 3, 9, 4, 17, 4], request);
                if calls == bad_call {
                    if bad_call == 3 {
                        // Full language body changed its previously read first LANGID.
                        transfer.bytes[2] ^= 1;
                    } else {
                        transfer.bytes[1] = 2;
                    }
                }
                transfer
            });
            assert_eq!(calls, bad_call);
            assert_eq!(record.transfers.len(), bad_call);
            assert!(replay(&record).is_err());
            assert!(
                record
                    .transfers
                    .iter()
                    .all(|entry| entry.request != PART && entry.request != CONFIGURATION)
            );
        }
    }

    #[test]
    fn missing_or_ambiguous_selection_sends_no_requests() {
        for (count, status) in [(0, 0), (2, 0), (1, -7)] {
            let entry = observe_domain(Domain::NativeUsbSerialJtag, count, status, |_| {
                panic!("selection must not transfer")
            });
            assert!(entry.transfers.is_empty());
            assert!(replay(&entry).is_err());
        }
    }

    #[test]
    fn failed_transfer_and_invalid_device_stop_after_preserving_result() {
        let failed = observe_domain(Domain::NativeUsbSerialJtag, 1, 0, |request| Transfer {
            request,
            status: -1,
            returned_length: 2,
            bytes: vec![18, 1],
        });
        assert_eq!(failed.transfers.len(), 1);
        assert_eq!(failed.transfers[0].bytes, [18, 1]);
        assert!(matches!(replay(&failed), Err(Error::Transfer)));
        let wrong = observe_domain(Domain::NativeUsbSerialJtag, 1, 0, |request| {
            let body = descriptor(Domain::Cp2102nBridge, 3);
            success(request, body)
        });
        assert_eq!(wrong.transfers.len(), 1);
        assert!(matches!(replay(&wrong), Err(Error::DeviceDescriptor)));
    }

    #[test]
    fn exact_shapes_order_and_status_are_required() {
        for mutation in 0..12 {
            let mut record = fixture();
            match mutation {
                0 => {
                    record.version = 1;
                }
                1 => {
                    record.domains.swap(0, 1);
                }
                2 => {
                    record.domains[0].transfers.pop();
                }
                3 => {
                    record.domains[0]
                        .transfers
                        .push(record.domains[0].transfers[0].clone());
                }
                4 => {
                    record.domains[0].transfers.swap(0, 1);
                }
                5 => {
                    record.domains[0].transfers[0].request.request_type = 0;
                }
                6 => {
                    record.domains[0].transfers[0].returned_length -= 1;
                }
                7 => {
                    record.domains[0].transfers[0].bytes[1] = 2;
                }
                8 => {
                    record.domains[1].transfers[5].bytes[0] = 0x01;
                }
                9 => {
                    record.domains[1].transfers[6].bytes[2] = 2;
                }
                10 => {
                    record.domains[0].cleanup_status = -9;
                }
                _ => {
                    record.domains[1].transfers[6].status = -4;
                }
            }
            assert!(summarize(&record).is_err(), "mutation {mutation}");
        }
        for index in [0, 1, 2, 3, 4, 6] {
            let mut record = fixture();
            let transfer = &mut record.domains[1].transfers[index];
            transfer.bytes.pop();
            transfer.returned_length -= 1;
            assert!(summarize(&record).is_err());
        }
    }

    #[test]
    fn languages_utf16_and_descriptor_lengths_are_validated() {
        let mut unicode = fixture();
        unicode.domains[0] = fixture_domain(Domain::NativeUsbSerialJtag, "synthetic-unicode-🧪", 3);
        assert_eq!(
            summarize(&unicode).unwrap().domains[0].serial,
            SerialPresence::Present
        );
        for body in [vec![2, 3], vec![5, 3, 9, 4, 0], vec![4, 3, 0, 0]] {
            let mut record = fixture();
            record.domains[0].transfers[1] = success(LANGUAGES, body);
            assert!(summarize(&record).is_err());
        }
        let mut duplicates = fixture();
        let duplicate_request = duplicates.domains[0].transfers[2].request;
        duplicates.domains[0].transfers[2] = success(duplicate_request, vec![6, 3, 9, 4, 9, 4]);
        assert!(summarize(&duplicates).is_err());
        for body in [
            vec![4, 3, 0, 0xd8],
            vec![4, 3, 0, 0xdc],
            vec![3, 3, 1],
            vec![4, 2, 1, 0],
        ] {
            let mut record = fixture();
            record.domains[0] = fixture_domain(Domain::NativeUsbSerialJtag, "X", 3);
            let request = record.domains[0].transfers[4].request;
            record.domains[0].transfers[4] = success(request, body);
            assert!(summarize(&record).is_err());
        }
        let mut record = fixture();
        record.domains[0] = fixture_domain(Domain::NativeUsbSerialJtag, "", 3);
        assert_eq!(
            summarize(&record).unwrap().domains[0].serial,
            SerialPresence::Empty
        );
    }

    #[test]
    fn comparison_uses_contents_and_keeps_bridge_bytes_private() {
        let before = fixture();
        let mut after = fixture();
        after.domains[0] =
            fixture_domain(Domain::NativeUsbSerialJtag, "different-private-marker", 3);
        after.domains[1].transfers[6].bytes[101] ^= 1;
        let report = compare(&before, &after).unwrap();
        assert_eq!(report.domains[0].serial, SerialComparison::Changed);
        assert_eq!(report.domains[1].serial, SerialComparison::Unchanged);
        assert_eq!(report.domains[0].configuration_bytes_unchanged, None);
        assert_eq!(report.domains[1].configuration_bytes_unchanged, Some(false));
        let same = compare(&before, &before).unwrap();
        assert!(
            same.domains
                .iter()
                .all(|entry| entry.serial == SerialComparison::Unchanged)
        );
        let output = serde_json::to_string(&report).unwrap();
        assert!(!output.contains("private-marker"));
        assert!(!output.contains("secret"));
    }

    #[test]
    fn raw_schema_caps_duplicates_and_extra_fields_fail() {
        let good = serde_json::to_string(&fixture()).unwrap();
        let mut duplicate = good.clone();
        duplicate.pop();
        duplicate.push_str(",\"version\":1}");
        assert!(parse(duplicate.as_bytes()).is_err());
        let mut extra = serde_json::to_value(fixture()).unwrap();
        extra["private_unknown_field"] = serde_json::json!("do-not-publish");
        assert!(parse(&serde_json::to_vec(&extra).unwrap()).is_err());
        assert!(parse(&vec![b' '; MAX_RECORD_BYTES + 1]).is_err());
        assert!(parse(&good.as_bytes()[..good.len() - 1]).is_err());
        assert!(parse(&[0xff]).is_err());
        let mut legacy = fixture();
        legacy.version = 1;
        assert!(parse(&serde_json::to_vec(&legacy).unwrap()).is_err());
        assert!(summarize(&legacy).is_err());
    }
}
