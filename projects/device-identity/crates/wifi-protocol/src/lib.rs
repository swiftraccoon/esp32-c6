//! Bounded private control for one pre-start SoftAP MAC override per boot.
//!
//! The nonce, original MAC and SSID belong in private captures. Debug redacts
//! those values. Successful driver readback does not establish transmitted
//! identity; a separate receiver must observe the expected beacon.
//!
//! ```
//! use wifi_protocol::{Operation, Request, MAX_REQUEST_BYTES, encode_request, decode_request};
//! let request = Request::new(1, Operation::Status, [7; 8]);
//! let mut bytes = [0; MAX_REQUEST_BYTES];
//! let length = encode_request(&request, &mut bytes)?;
//! assert_eq!(decode_request(&bytes[..length])?, request);
//! # Ok::<(), wifi_protocol::FrameError>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

use serde::{Deserialize, Serialize};

pub mod restart;

#[cfg(test)]
extern crate self as wifi_protocol;
#[cfg(test)]
#[path = "../../../firmware-wifi/src/controller.rs"]
mod firmware_controller;

/// Supported private control format.
pub const WIRE_VERSION: u32 = 1;
/// Maximum request size including LF.
pub const MAX_REQUEST_BYTES: usize = 256;
/// Maximum response size including LF.
pub const MAX_RESPONSE_BYTES: usize = 1024;
/// Fixed synthetic locally administered unicast AP control, not a unique ID.
pub const AP_MAC: [u8; 6] = [2, 23, 42, 65, 83, 113];
/// Fixed receiver/transmitter channel for this presence-witness method.
pub const AP_CHANNEL: u8 = 1;
/// Requested firmware auto-stop deadline; not a measured on-air duration.
pub const AP_WINDOW_MILLIS: u32 = 8000;
/// Nonunique ASCII prefix for the private nonce-bearing SSID.
pub const SSID_PREFIX: &[u8; 6] = b"di-ap-";
/// Complete SSID size, including sixteen lower-case hexadecimal nonce digits.
pub const SSID_LEN: usize = 22;

/// Construct the fixed-length private trial SSID without allocation.
pub fn ssid(nonce: [u8; 8]) -> [u8; SSID_LEN] {
    let mut result = [0; SSID_LEN];
    result[..SSID_PREFIX.len()].copy_from_slice(SSID_PREFIX);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (index, byte) in nonce.into_iter().enumerate() {
        result[SSID_PREFIX.len() + index * 2] = HEX[usize::from(byte >> 4)];
        result[SSID_PREFIX.len() + index * 2 + 1] = HEX[usize::from(byte & 15)];
    }
    result
}

/// Fixed operations; callers cannot supply an arbitrary MAC or radio channel.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Inspect application/driver state without starting Wi-Fi.
    Status,
    /// Perform the single allowed AP start with the fixed control MAC.
    Start,
    /// Stop and restore the saved driver MAC; repeatable for recovery.
    Stop,
}

/// Host request; its nonce is a correlation marker, not authentication.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    version: u32,
    /// Nonzero request correlation identifier.
    pub id: u32,
    /// Fixed operation.
    pub operation: Operation,
    /// One fresh nonzero nonce shared by all requests of the same trial.
    pub nonce: [u8; 8],
}

impl Request {
    /// Construct a request using the supported version; validate before use.
    pub const fn new(id: u32, operation: Operation, nonce: [u8; 8]) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            operation,
            nonce,
        }
    }

    /// Validate version and nonzero correlation fields.
    ///
    /// # Errors
    /// Returns a static error without copying request contents.
    pub fn validate(&self) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION {
            return Err(FrameError::Version);
        }
        if self.id == 0 || self.nonce == [0; 8] {
            return Err(FrameError::Invalid);
        }
        Ok(())
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("operation", &self.operation)
            .field("correlation", &"<redacted>")
            .finish()
    }
}

/// Application state, distinct from underlying driver initialization.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// No Wi-Fi initialization or start has been attempted.
    Idle,
    /// The single AP start completed and the driver reports running.
    Active,
    /// The AP is inactive; a second Start remains forbidden until reset.
    Stopped,
}

/// Private driver readback; initialization can remain true after a valid Stop.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    /// Boot-local application state.
    pub state: State,
    /// Latched trial nonce; absent only before a Start attempt.
    pub nonce: Option<[u8; 8]>,
    /// Whether an owned initialized driver remains present.
    pub initialized: bool,
    /// Event-backed SDK running-state observation.
    pub running: bool,
    /// Current driver AP MAC, absent before initialization.
    pub mac: Option<[u8; 6]>,
    /// Original driver AP MAC saved before its override.
    pub original_mac: Option<[u8; 6]>,
    /// Whether the firmware's requested auto-stop deadline ran.
    pub auto_stopped: bool,
}

fn valid_mac(value: [u8; 6]) -> bool {
    value != [0; 6] && value[0] & 1 == 0
}

impl Snapshot {
    fn validate(&self) -> Result<(), FrameError> {
        let valid = match self.state {
            State::Idle => {
                self.nonce.is_none()
                    && !self.initialized
                    && !self.running
                    && self.mac.is_none()
                    && self.original_mac.is_none()
                    && !self.auto_stopped
            }
            State::Active => {
                self.nonce.is_some_and(|n| n != [0; 8])
                    && self.initialized
                    && self.running
                    && !self.auto_stopped
                    && self.mac == Some(AP_MAC)
                    && self
                        .original_mac
                        .is_some_and(|m| valid_mac(m) && m != AP_MAC)
            }
            State::Stopped => {
                self.nonce.is_some_and(|n| n != [0; 8])
                    && !self.running
                    && if self.initialized {
                        self.original_mac
                            .is_some_and(|m| valid_mac(m) && m != AP_MAC)
                            && self.mac == self.original_mac
                    } else {
                        self.mac.is_none() && self.original_mac.is_none()
                    }
            }
        };
        if valid {
            Ok(())
        } else {
            Err(FrameError::State)
        }
    }
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("state", &self.state)
            .field("initialized", &self.initialized)
            .field("running", &self.running)
            .field("auto_stopped", &self.auto_stopped)
            .field("private_values", &"<redacted>")
            .finish()
    }
}

/// Correlated driver response; failed SDK operations have no successful snapshot.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    version: u32,
    /// Echoed request identifier.
    pub id: u32,
    /// Echoed operation.
    pub operation: Operation,
    /// Zero for success; raw SDK/application failures stay in private records.
    pub status: i32,
    /// Complete successful snapshot, absent on failure.
    pub snapshot: Option<Snapshot>,
}

impl Response {
    /// Construct a supported response; validate before admission.
    pub const fn new(
        id: u32,
        operation: Operation,
        status: i32,
        snapshot: Option<Snapshot>,
    ) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            operation,
            status,
            snapshot,
        }
    }

    fn validate_shape(&self) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION {
            return Err(FrameError::Version);
        }
        if self.id == 0 || ((self.status == 0) != self.snapshot.is_some()) {
            return Err(FrameError::Invalid);
        }
        if let Some(snapshot) = self.snapshot {
            snapshot.validate()?;
        }
        Ok(())
    }

    /// Check complete shape, nonce/operation correlation and operation success.
    ///
    /// # Errors
    /// Returns a static framing, correlation, state or device error.
    pub fn validate_for(&self, request: &Request) -> Result<(), FrameError> {
        request.validate()?;
        self.validate_shape()?;
        if self.id != request.id || self.operation != request.operation {
            return Err(FrameError::Mismatch);
        }
        if self.status != 0 {
            return Err(FrameError::Device);
        }
        let snapshot = self.snapshot.ok_or(FrameError::Invalid)?;
        if snapshot.nonce.is_some_and(|nonce| nonce != request.nonce) {
            return Err(FrameError::Mismatch);
        }
        match request.operation {
            Operation::Status => Ok(()),
            Operation::Start if snapshot.state == State::Active => Ok(()),
            Operation::Stop if matches!(snapshot.state, State::Idle | State::Stopped) => Ok(()),
            _ => Err(FrameError::State),
        }
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("operation", &self.operation)
            .field("status", &self.status)
            .field("snapshot", &self.snapshot)
            .field("correlation", &"<redacted>")
            .finish()
    }
}

/// Value-free wire and state failures.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FrameError {
    /// Input or output exceeds the fixed frame capacity.
    Overflow,
    /// Malformed/unknown JSON fields or inconsistent status/payload.
    Invalid,
    /// Unsupported wire version.
    Version,
    /// Request identifier, operation or session nonce mismatch.
    Mismatch,
    /// Explicit device operation failure.
    Device,
    /// Inconsistent application/driver state.
    State,
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Overflow => "Wi-Fi control frame exceeds byte limit",
            Self::Invalid => "invalid Wi-Fi control frame",
            Self::Version => "unsupported Wi-Fi control version",
            Self::Mismatch => "Wi-Fi control correlation mismatch",
            Self::Device => "Wi-Fi device operation failed",
            Self::State => "inconsistent Wi-Fi control state",
        })
    }
}
impl core::error::Error for FrameError {}

/// Encode one validated LF-terminated request into caller storage.
pub fn encode_request(request: &Request, bytes: &mut [u8]) -> Result<usize, FrameError> {
    request.validate()?;
    encode(request, bytes, MAX_REQUEST_BYTES)
}
/// Encode one shape-validated LF-terminated response into caller storage.
pub fn encode_response(response: &Response, bytes: &mut [u8]) -> Result<usize, FrameError> {
    response.validate_shape()?;
    encode(response, bytes, MAX_RESPONSE_BYTES)
}
fn encode<T: Serialize>(value: &T, bytes: &mut [u8], limit: usize) -> Result<usize, FrameError> {
    let cap = bytes.len().min(limit);
    if cap == 0 {
        return Err(FrameError::Overflow);
    }
    let length = serde_json_core::to_slice(value, &mut bytes[..cap - 1])
        .map_err(|_| FrameError::Overflow)?;
    bytes[length] = b'\n';
    Ok(length + 1)
}
/// Decode a bounded request and reject unknown fields and trailing non-whitespace.
pub fn decode_request(bytes: &[u8]) -> Result<Request, FrameError> {
    let value: Request = decode(bytes, MAX_REQUEST_BYTES)?;
    value.validate()?;
    Ok(value)
}
/// Decode a response without authenticating its source or admitting SDK failure.
pub fn decode_response(bytes: &[u8]) -> Result<Response, FrameError> {
    let value: Response = decode(bytes, MAX_RESPONSE_BYTES)?;
    value.validate_shape()?;
    Ok(value)
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
    use super::*;
    use std::format;

    fn active() -> Snapshot {
        Snapshot {
            state: State::Active,
            nonce: Some([7; 8]),
            initialized: true,
            running: true,
            mac: Some(AP_MAC),
            original_mac: Some([48; 6]),
            auto_stopped: false,
        }
    }

    #[test]
    fn wire_roundtrip_and_all_operation_state_relationships() {
        let request = Request::new(1, Operation::Start, [7; 8]);
        let response = Response::new(1, Operation::Start, 0, Some(active()));
        let mut bytes = [0; MAX_RESPONSE_BYTES];
        let n = encode_response(&response, &mut bytes).unwrap();
        assert_eq!(decode_response(&bytes[..n]).unwrap(), response);
        assert_eq!(response.validate_for(&request), Ok(()));
        let stopped = Snapshot {
            state: State::Stopped,
            running: false,
            mac: active().original_mac,
            auto_stopped: true,
            ..active()
        };
        let request = Request::new(2, Operation::Stop, [7; 8]);
        assert!(
            Response::new(2, Operation::Stop, 0, Some(stopped))
                .validate_for(&request)
                .is_ok()
        );
        assert!(
            Response::new(2, Operation::Stop, 0, Some(active()))
                .validate_for(&request)
                .is_err()
        );
        let idle = Snapshot {
            state: State::Idle,
            nonce: None,
            initialized: false,
            running: false,
            mac: None,
            original_mac: None,
            auto_stopped: false,
        };
        assert!(
            Response::new(2, Operation::Stop, 0, Some(idle))
                .validate_for(&request)
                .is_ok()
        );
    }

    #[test]
    fn corruption_stale_nonce_wrong_mac_and_missing_restoration_reject() {
        let request = Request::new(1, Operation::Start, [7; 8]);
        for snapshot in [
            Snapshot {
                nonce: Some([8; 8]),
                ..active()
            },
            Snapshot {
                mac: Some([4; 6]),
                ..active()
            },
            Snapshot {
                initialized: false,
                ..active()
            },
            Snapshot {
                original_mac: Some(AP_MAC),
                ..active()
            },
        ] {
            assert!(
                Response::new(1, Operation::Start, 0, Some(snapshot))
                    .validate_for(&request)
                    .is_err()
            );
        }
        assert_eq!(
            Response::new(1, Operation::Start, 5, None).validate_for(&request),
            Err(FrameError::Device)
        );
        assert!(
            Request::new(1, Operation::Start, [0; 8])
                .validate()
                .is_err()
        );
        assert!(
            Request::new(0, Operation::Start, [7; 8])
                .validate()
                .is_err()
        );
        let bad_stop = Snapshot {
            state: State::Stopped,
            running: false,
            ..active()
        };
        assert!(
            encode_response(
                &Response::new(2, Operation::Stop, 0, Some(bad_stop)),
                &mut [0; MAX_RESPONSE_BYTES]
            )
            .is_err()
        );
    }

    #[test]
    fn schema_bounds_and_debug_keep_private_values_out() {
        let request = Request::new(1, Operation::Status, [7; 8]);
        let mut bytes = [0; MAX_REQUEST_BYTES];
        let n = encode_request(&request, &mut bytes).unwrap();
        assert_eq!(decode_request(&bytes[..n]).unwrap(), request);
        assert!(encode_request(&request, &mut []).is_err());
        assert!(decode_request(&[b' '; MAX_REQUEST_BYTES + 1]).is_err());
        assert!(decode_request(br#"{"version":1,"id":1,"operation":"status","nonce":[7,7,7,7,7,7,7,7],"extra":true}"#).is_err());
        assert!(
            decode_request(
                br#"{"version":1,"id":1,"id":2,"operation":"status","nonce":[7,7,7,7,7,7,7,7]}"#
            )
            .is_err()
        );
        let text = format!("{:?}", active());
        assert!(text.contains("<redacted>"));
        assert!(!text.contains("48"));
        assert!(!text.contains("[7"));
        let label = ssid([0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        assert_eq!(&label[..SSID_PREFIX.len()], SSID_PREFIX);
        assert_eq!(&label[SSID_PREFIX.len()..], b"1023456789abcdef");
    }
}
