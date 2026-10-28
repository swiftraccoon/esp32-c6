//! Domain-separated, bounded private JSON exchanges; diagnostics redact values.
//!
//! ```
//! use device_identity_ble_firmware::protocol::{Request,Operation,MAX_REQUEST_BYTES,encode_request,decode_request};
//! let request=Request::new(1,Operation::Status,[7;8]);
//! let mut bytes=[0;MAX_REQUEST_BYTES];
//! let n=encode_request(&request,&mut bytes)?;
//! assert_eq!(decode_request(&bytes[..n])?,request);
//! # Ok::<(),device_identity_ble_firmware::protocol::FrameError>(())
//! ```

use core::fmt;
use serde::{Deserialize, Serialize};

/// BLE advertising wire version; distinct from the Wi-Fi protocols.
pub const WIRE_VERSION: u32 = 1;
/// Maximum request including its optional terminal LF.
pub const MAX_REQUEST_BYTES: usize = 256;
/// Maximum response including its terminal LF.
pub const MAX_RESPONSE_BYTES: usize = 1024;
/// Independent software stop request deadline per consumed phase.
pub const WINDOW_MILLIS: u32 = 8000;
/// Fixed first static-random control, in conventional MSB-first order.
pub const FIRST_ADDRESS: [u8; 6] = [0xc2, 0x17, 0x2a, 0x41, 0x53, 0x71];
/// Fixed second static-random control; it is not an RPA.
pub const SECOND_ADDRESS: [u8; 6] = [0xc2, 0x17, 0x2a, 0x41, 0x53, 0x72];
/// Custom application service UUID in its little-endian AD encoding.
pub const SERVICE_UUID_LE: [u8; 16] = [
    0xc6, 0x61, 0xf3, 0xd0, 0x5c, 0x2a, 0x28, 0xa2, 0x7b, 0x45, 0x0e, 0x82, 0x92, 0x25, 0x77, 0x87,
];
/// Construct one complete 128-bit Service Data AD containing the session nonce.
pub fn nonce_ad(nonce: [u8; 8]) -> [u8; 26] {
    let mut out = [0; 26];
    out[0] = 25;
    out[1] = 0x21;
    out[2..18].copy_from_slice(&SERVICE_UUID_LE);
    out[18..].copy_from_slice(&nonce);
    out
}

/// Static failure, containing no private values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    /// Frame encoding/decoding exceeds its bound.
    Overflow,
    /// JSON, fields or scalar bounds are invalid.
    Invalid,
    /// Wire version is outside this protocol.
    Version,
    /// Response state is contradictory.
    State,
    /// Request/response correlation differs.
    Mismatch,
    /// Current device operation failed.
    Device,
}
impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Overflow => "BLE frame exceeds capacity",
            Self::Invalid => "invalid BLE control frame",
            Self::Version => "unsupported BLE control version",
            Self::State => "inconsistent BLE control state",
            Self::Mismatch => "BLE response correlation mismatch",
            Self::Device => "BLE device operation failed",
        })
    }
}
impl core::error::Error for FrameError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Domain {
    BleAdvertising,
}
/// Ordered stopped-state advertising attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// First advertising attempt.
    First,
    /// Second attempt after an explicit successful first Stop.
    Second,
}
impl Phase {
    /// Fixed conventional static-random address for this phase.
    pub const fn address(self) -> [u8; 6] {
        match self {
            Self::First => FIRST_ADDRESS,
            Self::Second => SECOND_ADDRESS,
        }
    }
}
/// The complete command surface; no arbitrary radio settings are accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Inspect without initializing Bluetooth.
    Status,
    /// Consume the first advertising attempt.
    StartFirst,
    /// Consume the second attempt after checked Stop.
    StartSecond,
    /// Request checked advertising stop, retaining random RAM state.
    Stop,
}
/// Application state, qualified by completion events and readbacks.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// No consumed Start.
    Idle,
    /// Advertising start completion and address readback succeeded.
    Active,
    /// SDK host-stack Stop completion succeeded, without controller/RF proof.
    Stopped,
}
/// Actual used-address type returned by the SDK.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressType {
    /// Public address type.
    Public,
    /// Random address type; no RPA interpretation.
    Random,
}

/// Exact readback availability; private addresses never occur in Debug.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(into = "ReadbackWire", try_from = "ReadbackWire")]
pub enum AddressReadback {
    /// No query before initialization/first configuration.
    NotRequested,
    /// Successful generated-binding used-address query.
    Available {
        /// Conventional MSB-first returned address.
        address: [u8; 6],
        /// Returned public/random type.
        address_type: AddressType,
    },
    /// SDK did not provide an address at this point.
    Unavailable {
        /// Nonzero SDK status, retained only in private exchanges.
        status: i32,
    },
}

// A flat fixed record keeps serde-json-core decoding allocation-free; internally
// tagged enum deserialization otherwise requires serde's allocated Content.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Availability {
    NotRequested,
    Available,
    Unavailable,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadbackWire {
    availability: Availability,
    address: Option<[u8; 6]>,
    address_type: Option<AddressType>,
    status: Option<i32>,
}
impl From<AddressReadback> for ReadbackWire {
    fn from(value: AddressReadback) -> Self {
        match value {
            AddressReadback::NotRequested => Self {
                availability: Availability::NotRequested,
                address: None,
                address_type: None,
                status: None,
            },
            AddressReadback::Available {
                address,
                address_type,
            } => Self {
                availability: Availability::Available,
                address: Some(address),
                address_type: Some(address_type),
                status: Some(0),
            },
            AddressReadback::Unavailable { status } => Self {
                availability: Availability::Unavailable,
                address: None,
                address_type: None,
                status: Some(status),
            },
        }
    }
}
impl TryFrom<ReadbackWire> for AddressReadback {
    type Error = FrameError;
    fn try_from(value: ReadbackWire) -> Result<Self, Self::Error> {
        match (
            value.availability,
            value.address,
            value.address_type,
            value.status,
        ) {
            (Availability::NotRequested, None, None, None) => Ok(Self::NotRequested),
            (Availability::Available, Some(address), Some(address_type), Some(0)) => {
                Ok(Self::Available {
                    address,
                    address_type,
                })
            }
            (Availability::Unavailable, None, None, Some(status)) if status != 0 => {
                Ok(Self::Unavailable { status })
            }
            _ => Err(FrameError::State),
        }
    }
}
impl fmt::Debug for AddressReadback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotRequested => "AddressReadback::NotRequested",
            Self::Available { .. } => "AddressReadback::Available(<redacted>)",
            Self::Unavailable { .. } => "AddressReadback::Unavailable(<redacted>)",
        })
    }
}
impl AddressReadback {
    fn valid(self) -> bool {
        match self {
            Self::NotRequested => true,
            Self::Unavailable { status } => status != 0,
            Self::Available { address, .. } => address != [0; 6] && address != [255; 6],
        }
    }
    /// Whether this is the exact observed random control for a phase.
    pub fn matches(self, phase: Phase) -> bool {
        self == Self::Available {
            address: phase.address(),
            address_type: AddressType::Random,
        }
    }
}

/// One nonce-correlated command; the live controller enforces monotonic IDs.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    domain: Domain,
    version: u32,
    /// Nonzero monotonically increasing boot-local ID.
    pub id: u32,
    /// Fixed command.
    pub operation: Operation,
    /// One fresh nonzero session nonce, private correlation only.
    pub nonce: [u8; 8],
}
impl Request {
    /// Construct a domain-tagged request.
    pub const fn new(id: u32, operation: Operation, nonce: [u8; 8]) -> Self {
        Self {
            domain: Domain::BleAdvertising,
            version: WIRE_VERSION,
            id,
            operation,
            nonce,
        }
    }
    /// Check stateless shape; sequence and freshness are separate gates.
    ///
    /// # Errors
    /// Returns a static version or invalid-scalar category.
    pub fn validate(&self) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION {
            Err(FrameError::Version)
        } else if self.id == 0 || self.nonce == [0; 8] {
            Err(FrameError::Invalid)
        } else {
            Ok(())
        }
    }
}
impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BleRequest")
            .field("operation", &self.operation)
            .field("private_values", &"<redacted>")
            .finish()
    }
}

/// Private source observations. Stop retains driver/random state until reset.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    /// Current application state.
    pub state: State,
    /// Last consumed start attempt.
    pub phase: Option<Phase>,
    /// Nonce after a consumed Start; absent in Idle.
    pub nonce: Option<[u8; 8]>,
    /// A retained initialized Bluetooth driver exists.
    pub initialized: bool,
    /// Last successful SDK host-stack start/stop report, without RF proof.
    pub advertising: bool,
    /// Raw-data configuration completion for the session succeeded.
    pub data_configured: bool,
    /// Current phase's random-address setter completion succeeded.
    pub address_set_completed: bool,
    /// Query after setter completion but before advertising start.
    pub prestart_readback: AddressReadback,
    /// Current query, required to match the active phase after start.
    pub used_readback: AddressReadback,
    /// A software phase deadline requested Stop.
    pub auto_stopped: bool,
    /// Permanent first protocol, transport, completion or adapter failure.
    pub first_failure: Option<i32>,
}
impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BleSnapshot")
            .field("state", &self.state)
            .field("phase", &self.phase)
            .field("initialized", &self.initialized)
            .field("advertising", &self.advertising)
            .field("private_values", &"<redacted>")
            .finish()
    }
}
impl Snapshot {
    fn validate(&self) -> Result<(), FrameError> {
        if self.first_failure == Some(0)
            || !self.prestart_readback.valid()
            || !self.used_readback.valid()
        {
            return Err(FrameError::State);
        }
        let attempted = self.phase.is_some() && self.nonce.is_some_and(|n| n != [0; 8]);
        let valid = match self.state {
            State::Idle => {
                self.phase.is_none()
                    && self.nonce.is_none()
                    && !self.initialized
                    && !self.advertising
                    && !self.data_configured
                    && !self.address_set_completed
                    && self.prestart_readback == AddressReadback::NotRequested
                    && self.used_readback == AddressReadback::NotRequested
                    && !self.auto_stopped
            }
            State::Active => {
                attempted
                    && self.initialized
                    && self.advertising
                    && self.data_configured
                    && self.address_set_completed
                    && self.prestart_readback != AddressReadback::NotRequested
                    && self.phase.is_some_and(|p| self.used_readback.matches(p))
                    && (!self.auto_stopped || self.first_failure.is_some())
            }
            State::Stopped => {
                attempted
                    && !self.advertising
                    && if self.initialized {
                        self.used_readback != AddressReadback::NotRequested
                    } else {
                        !self.data_configured
                            && !self.address_set_completed
                            && self.prestart_readback == AddressReadback::NotRequested
                            && self.used_readback == AddressReadback::NotRequested
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

/// A correlated response; failed operations carry no partial snapshot.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    domain: Domain,
    version: u32,
    /// Echoed request ID.
    pub id: u32,
    /// Echoed operation.
    pub operation: Operation,
    /// Zero for this operation's success; otherwise raw/static failure.
    pub status: i32,
    /// Private checked state; only present when status is zero.
    pub snapshot: Option<Snapshot>,
}
impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BleResponse")
            .field("operation", &self.operation)
            .field("operation_succeeded", &(self.status == 0))
            .field("snapshot", &self.snapshot)
            .finish()
    }
}
impl Response {
    /// Construct a response; encoding checks shape before returning bytes.
    pub const fn new(
        id: u32,
        operation: Operation,
        status: i32,
        snapshot: Option<Snapshot>,
    ) -> Self {
        Self {
            domain: Domain::BleAdvertising,
            version: WIRE_VERSION,
            id,
            operation,
            status,
            snapshot,
        }
    }
    fn validate(&self) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION {
            return Err(FrameError::Version);
        }
        if self.id == 0 || (self.status == 0) != self.snapshot.is_some() {
            return Err(FrameError::Invalid);
        }
        if let Some(s) = self.snapshot {
            s.validate()?;
        }
        Ok(())
    }
    /// Check correlation and operation state; cleanup may retain a failed latch.
    ///
    /// # Errors
    /// Returns a static shape, device, correlation or state category.
    pub fn validate_for(&self, request: &Request) -> Result<(), FrameError> {
        request.validate()?;
        self.validate()?;
        if self.id != request.id || self.operation != request.operation {
            return Err(FrameError::Mismatch);
        }
        if self.status != 0 {
            return Err(FrameError::Device);
        }
        let s = self.snapshot.ok_or(FrameError::Invalid)?;
        if s.nonce.is_some_and(|n| n != request.nonce) {
            return Err(FrameError::Mismatch);
        }
        match request.operation {
            Operation::Status => Ok(()),
            Operation::Stop if matches!(s.state, State::Idle | State::Stopped) => Ok(()),
            Operation::StartFirst | Operation::StartSecond
                if s.state == State::Active
                    && s.first_failure.is_none()
                    && !s.auto_stopped
                    && s.phase
                        == Some(if request.operation == Operation::StartFirst {
                            Phase::First
                        } else {
                            Phase::Second
                        }) =>
            {
                Ok(())
            }
            _ => Err(FrameError::State),
        }
    }
}
fn encode<T: Serialize>(value: &T, bytes: &mut [u8], limit: usize) -> Result<usize, FrameError> {
    let cap = bytes.len().min(limit);
    if cap == 0 {
        return Err(FrameError::Overflow);
    }
    let n = serde_json_core::to_slice(value, &mut bytes[..cap - 1])
        .map_err(|_| FrameError::Overflow)?;
    bytes[n] = b'\n';
    Ok(n + 1)
}
fn decode<'a, T: Deserialize<'a>>(bytes: &'a [u8], limit: usize) -> Result<T, FrameError> {
    if bytes.len() > limit {
        return Err(FrameError::Overflow);
    }
    let (v, n) = serde_json_core::from_slice(bytes).map_err(|_| FrameError::Invalid)?;
    if !bytes[n..].iter().all(u8::is_ascii_whitespace) {
        return Err(FrameError::Invalid);
    }
    Ok(v)
}
/// Encode a validated bounded request plus LF.
///
/// # Errors
/// Returns a static request-validation or output-capacity category.
pub fn encode_request(request: &Request, bytes: &mut [u8]) -> Result<usize, FrameError> {
    request.validate()?;
    encode(request, bytes, MAX_REQUEST_BYTES)
}
/// Decode a whole bounded request, rejecting unknown fields and suffix data.
///
/// # Errors
/// Returns a static framing, capacity, version or scalar category.
pub fn decode_request(bytes: &[u8]) -> Result<Request, FrameError> {
    let r: Request = decode(bytes, MAX_REQUEST_BYTES)?;
    r.validate()?;
    Ok(r)
}
/// Encode a shape-validated private response plus LF.
///
/// # Errors
/// Returns a static response-state or output-capacity category.
pub fn encode_response(response: &Response, bytes: &mut [u8]) -> Result<usize, FrameError> {
    response.validate()?;
    encode(response, bytes, MAX_RESPONSE_BYTES)
}
/// Decode a whole bounded private response; this does not authenticate origin.
///
/// # Errors
/// Returns a static framing, capacity, version or response-state category.
pub fn decode_response(bytes: &[u8]) -> Result<Response, FrameError> {
    let r: Response = decode(bytes, MAX_RESPONSE_BYTES)?;
    r.validate()?;
    Ok(r)
}
