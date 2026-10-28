//! Bounded commands and private snapshots for a radio-uninitialized MAC-table probe.
//!
//! Frames are one JSON object followed by LF. Actual identifiers occur in
//! [`Snapshot`] serialization and must stay in private capture storage. Debug
//! formatting deliberately redacts snapshot values.
//!
//! ```
//! use runtime_protocol::{Phase, Request, MAX_REQUEST_BYTES, decode_request, encode_request};
//! let request = Request::new(42, Phase::Defaults);
//! let mut line = [0; MAX_REQUEST_BYTES];
//! let length = encode_request(&request, &mut line)?;
//! assert_eq!(decode_request(&line[..length])?, request);
//! # Ok::<(), runtime_protocol::FrameError>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

use serde::{Deserialize, Serialize};

#[cfg(test)]
extern crate self as runtime_protocol;
#[cfg(test)]
#[path = "../../../firmware/src/probe.rs"]
mod firmware_probe;

/// Supported wire version; newer versions fail instead of being guessed.
pub const WIRE_VERSION: u32 = 1;
/// Maximum request size, including LF.
pub const MAX_REQUEST_BYTES: usize = 128;
/// Maximum response size, including LF.
pub const MAX_RESPONSE_BYTES: usize = 2048;

/// Synthetic locally administered unicast base used only for table testing.
pub const BASE_OVERRIDE: [u8; 6] = [2, 23, 42, 65, 83, 96];
/// Synthetic station table value; this probe never initializes Wi-Fi.
pub const WIFI_STATION_OVERRIDE: [u8; 6] = [2, 23, 42, 65, 83, 112];
/// Synthetic SoftAP table value; this probe never starts an access point.
pub const WIFI_AP_OVERRIDE: [u8; 6] = [2, 23, 42, 65, 83, 113];
/// Synthetic Bluetooth table value; the controller remains uninitialized.
pub const BLUETOOTH_OVERRIDE: [u8; 6] = [2, 23, 42, 65, 83, 114];
/// Synthetic IEEE 802.15.4 table value; no radio driver is enabled.
pub const IEEE802154_OVERRIDE: [u8; 8] = [2, 23, 42, 65, 83, 96, 112, 128];

/// Ordered operations for one probe boot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Read factory/base/interface values, populating the SDK's interface cache.
    Defaults,
    /// Set only BASE; previously cached interface entries must remain unchanged.
    BaseOverride,
    /// Set all four interface table entries explicitly.
    InterfaceOverride,
    /// Restore the saved base and interface table entries in RAM.
    Restored,
}

impl Phase {
    /// Required command order. Reset starts a new sequence.
    pub const ALL: [Self; 4] = [
        Self::Defaults,
        Self::BaseOverride,
        Self::InterfaceOverride,
        Self::Restored,
    ];
}

/// One versioned fixed operation; no caller-supplied address is accepted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    version: u32,
    /// Caller-selected correlation token; not authentication or a board identity.
    pub id: u32,
    /// Desired phase; see [`Phase::ALL`] for the boot-local order.
    pub phase: Phase,
}

impl Request {
    /// Construct a command using the supported wire version.
    pub const fn new(id: u32, phase: Phase) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            phase,
        }
    }

    /// Check that the supported wire version was received.
    ///
    /// # Errors
    /// Returns [`FrameError::Version`] for an unsupported version.
    pub fn validate(&self) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION {
            return Err(FrameError::Version);
        }
        Ok(())
    }
}

/// Private raw values returned by the selected SDK APIs.
///
/// `factory` is eight bytes because the C6 `esp_efuse_mac_get_default` API
/// inserts MAC_EXT into the factory MAC. BASE and ordinary interface getters
/// return six bytes; the IEEE getter returns eight. Serialization exposes the
/// values and belongs only in private storage. Debug formatting is redacted.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    /// Direct factory MAC and extension from eFuse, in EUI-64 layout.
    pub factory: [u8; 8],
    /// Current software base table entry.
    pub base: [u8; 6],
    /// Current Wi-Fi station software table entry.
    pub wifi_station: [u8; 6],
    /// Current Wi-Fi SoftAP software table entry.
    pub wifi_ap: [u8; 6],
    /// Current Bluetooth software table entry, independent of controller startup.
    pub bluetooth: [u8; 6],
    /// Current IEEE 802.15.4 software table entry.
    pub ieee802154: [u8; 8],
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Snapshot")
            .field("identity_values", &"<redacted>")
            .finish()
    }
}

/// One private reply. A success contains exactly one complete snapshot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    version: u32,
    /// Exact request correlation token.
    pub id: u32,
    /// Echoed operation; validates correlation without a board identifier.
    pub phase: Phase,
    /// SDK status integer, zero for success; negative protocol codes are retained.
    pub status: i32,
    /// Complete raw snapshot on success, absent on failure.
    pub snapshot: Option<Snapshot>,
}

impl Response {
    /// Construct a reply using the supported version.
    ///
    /// Call [`Self::validate_for`] or use [`encode_response`] before admission.
    pub const fn new(id: u32, phase: Phase, status: i32, snapshot: Option<Snapshot>) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            phase,
            status,
            snapshot,
        }
    }

    /// Validate version, requested phase, status/payload consistency and success.
    ///
    /// # Errors
    /// Returns a static [`FrameError`] for unsupported versions, a mismatched
    /// phase, inconsistent status/payload, or a reported device failure.
    pub fn validate_for(&self, request: &Request) -> Result<(), FrameError> {
        request.validate()?;
        self.validate_shape()?;
        if self.id != request.id || self.phase != request.phase {
            return Err(FrameError::Mismatch);
        }
        if self.status != 0 {
            return Err(FrameError::Device);
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION {
            return Err(FrameError::Version);
        }
        if (self.status == 0) != self.snapshot.is_some() {
            return Err(FrameError::Invalid);
        }
        Ok(())
    }
}

/// Static diagnostics deliberately contain no private input or value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FrameError {
    /// Frame exceeds its explicit byte bound or output capacity.
    Overflow,
    /// Input JSON, metadata or status/payload shape is invalid.
    Invalid,
    /// Wire version is unsupported.
    Version,
    /// Reply phase does not match the outstanding request.
    Mismatch,
    /// Device explicitly reported failure; its raw status stays in the reply.
    Device,
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Overflow => "runtime frame exceeds byte limit",
            Self::Invalid => "invalid runtime frame",
            Self::Version => "unsupported runtime protocol version",
            Self::Mismatch => "runtime response phase mismatch",
            Self::Device => "runtime device operation failed",
        })
    }
}

impl core::error::Error for FrameError {}

/// Encode a validated request with a terminating LF into caller-owned storage.
///
/// # Errors
/// Returns a static error for an unsupported version or insufficient capacity.
pub fn encode_request(request: &Request, bytes: &mut [u8]) -> Result<usize, FrameError> {
    request.validate()?;
    encode(request, bytes, MAX_REQUEST_BYTES)
}

/// Encode a shape-validated private reply with LF into caller-owned storage.
///
/// # Errors
/// Returns a static error for invalid version/status/payload or capacity.
pub fn encode_response(response: &Response, bytes: &mut [u8]) -> Result<usize, FrameError> {
    response.validate_shape()?;
    encode(response, bytes, MAX_RESPONSE_BYTES)
}

fn encode<T: Serialize>(value: &T, bytes: &mut [u8], limit: usize) -> Result<usize, FrameError> {
    let capacity = bytes.len().min(limit);
    if capacity == 0 {
        return Err(FrameError::Overflow);
    }
    let length = serde_json_core::to_slice(value, &mut bytes[..capacity - 1])
        .map_err(|_| FrameError::Overflow)?;
    bytes[length] = b'\n';
    Ok(length + 1)
}

/// Decode one complete bounded request; trailing JSON or non-whitespace fails.
///
/// # Errors
/// Returns a static error for malformed, duplicate/unknown-field, oversized or
/// unsupported-version data. The trailing LF may be included or omitted.
pub fn decode_request(bytes: &[u8]) -> Result<Request, FrameError> {
    let request: Request = decode(bytes, MAX_REQUEST_BYTES)?;
    request.validate()?;
    Ok(request)
}

/// Decode one bounded private reply without authenticating its origin.
///
/// # Errors
/// Returns a static error for malformed, oversized, unsupported-version or
/// inconsistent status/payload data. A valid device failure remains a reply;
/// [`Response::validate_for`] rejects its admission as a successful observation.
pub fn decode_response(bytes: &[u8]) -> Result<Response, FrameError> {
    let response: Response = decode(bytes, MAX_RESPONSE_BYTES)?;
    response.validate_shape()?;
    Ok(response)
}

fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8], limit: usize) -> Result<T, FrameError> {
    if bytes.len() > limit {
        return Err(FrameError::Overflow);
    }
    let (value, consumed) = serde_json_core::from_slice(bytes).map_err(|_| FrameError::Invalid)?;
    if !bytes[consumed..].iter().all(u8::is_ascii_whitespace) {
        return Err(FrameError::Invalid);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot {
            factory: [79; 8],
            base: [83; 6],
            wifi_station: [89; 6],
            wifi_ap: [97; 6],
            bluetooth: [101; 6],
            ieee802154: [103; 8],
        }
    }

    #[test]
    fn rejects_unknown_version_fields_and_unconsumed_suffixes() {
        for bytes in [
            br#"{"version":2,"id":1,"phase":"defaults"}"#.as_slice(),
            br#"{"version":1,"id":1,"phase":"defaults","ignored":true}"#,
            br#"{"version":1,"id":1,"phase":"defaults","phase":"restored"}"#,
            br#"{"version":1,"id":1,"phase":"defaults"} {}"#,
        ] {
            assert!(decode_request(bytes).is_err());
        }
    }

    #[test]
    fn rejects_failure_admission_mismatch_and_partial_success() {
        let request = Request::new(42, Phase::Defaults);
        let failed = Response::new(42, Phase::Defaults, 257, None);
        assert_eq!(failed.validate_for(&request), Err(FrameError::Device));
        let mismatch = Response::new(42, Phase::Restored, 0, Some(snapshot()));
        assert_eq!(mismatch.validate_for(&request), Err(FrameError::Mismatch));
        assert_eq!(
            Response::new(42, Phase::Defaults, 0, None).validate_for(&request),
            Err(FrameError::Invalid)
        );
        assert_eq!(
            Response::new(42, Phase::Defaults, -1, Some(snapshot())).validate_for(&request),
            Err(FrameError::Invalid)
        );
    }

    #[test]
    fn debug_redacts_private_values_but_wire_roundtrip_preserves_them() {
        let response = Response::new(42, Phase::Defaults, 0, Some(snapshot()));
        let debug = format!("{response:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("79"));
        assert!(!debug.contains("103"));
        let mut bytes = [0; MAX_RESPONSE_BYTES];
        let length = encode_response(&response, &mut bytes).unwrap();
        assert_eq!(decode_response(&bytes[..length]).unwrap(), response);
    }

    #[test]
    fn encoder_capacity_and_decoder_bound_fail_without_panicking() {
        let request = Request::new(42, Phase::Defaults);
        for capacity in [0, 1, 12] {
            assert_eq!(
                encode_request(&request, &mut [0; 12][..capacity]),
                Err(FrameError::Overflow)
            );
        }
        assert_eq!(
            decode_request(&[b' '; MAX_REQUEST_BYTES + 1]),
            Err(FrameError::Overflow)
        );
    }

    #[test]
    fn actual_probe_restores_saved_state_after_injected_partial_operation_failure() {
        let saved = snapshot();
        let mut probe = firmware_probe::Probe::new();
        let first = probe.handle(Request::new(11, Phase::Defaults), |phase, baseline| {
            assert_eq!(phase, Phase::Defaults);
            assert!(baseline.is_none());
            Ok(saved)
        });
        assert_eq!(first.snapshot, Some(saved));
        let failed = probe.handle(Request::new(12, Phase::BaseOverride), |_, baseline| {
            assert_eq!(baseline.copied(), Some(saved));
            Err(257)
        });
        assert_eq!(failed.status, 257);
        let rejected = probe.handle(Request::new(13, Phase::InterfaceOverride), |_, _| {
            panic!("a failed sequence must not run another apply operation")
        });
        assert_eq!(rejected.status, firmware_probe::PHASE_FAILURE);
        for id in [14, 15] {
            let restored = probe.handle(Request::new(id, Phase::Restored), |phase, baseline| {
                assert_eq!(phase, Phase::Restored);
                Ok(*baseline.unwrap())
            });
            assert_eq!(restored.snapshot, Some(saved));
        }
    }

    #[test]
    fn actual_probe_rejects_stale_phase_without_losing_cleanup_after_lost_reply() {
        let saved = snapshot();
        let mut probe = firmware_probe::Probe::new();
        let invalid = probe.handle(Request::new(20, Phase::InterfaceOverride), |_, _| {
            panic!("out-of-order operation must not execute")
        });
        assert_eq!(invalid.status, firmware_probe::PHASE_FAILURE);
        // Simulate the defaults response being lost after its operation ran.
        let _lost = probe.handle(Request::new(21, Phase::Defaults), |_, _| Ok(saved));
        let repeated = probe.handle(Request::new(21, Phase::Defaults), |_, _| {
            panic!("repeated defaults must not replace the saved baseline")
        });
        assert_eq!(repeated.status, firmware_probe::PHASE_FAILURE);
        let cleanup = probe.handle(Request::new(22, Phase::Restored), |_, baseline| {
            Ok(*baseline.unwrap())
        });
        assert_eq!(cleanup.snapshot, Some(saved));
    }
}
