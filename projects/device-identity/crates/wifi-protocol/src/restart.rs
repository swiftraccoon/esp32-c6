//! Bounded same-boot AP restart control; version one remains separate.
//!
//! The two phases intentionally share one private nonce SSID. Address changes
//! across a stopped/restarted AP do not establish hot rotation, RF silence or
//! reduced correlation. Failed sessions can still report checked cleanup using
//! [`Snapshot::first_failure`]; cleanup never erases that terminal failure.
//!
//! ```
//! use wifi_protocol::restart::{Operation, Request, MAX_REQUEST_BYTES, encode_request, decode_request};
//! let request = Request::new(1, Operation::Status, [7; 8]);
//! let mut frame = [0; MAX_REQUEST_BYTES];
//! let length = encode_request(&request, &mut frame)?;
//! assert_eq!(decode_request(&frame[..length])?, request);
//! assert_ne!(wifi_protocol::restart::Phase::First.mac(), wifi_protocol::restart::Phase::Second.mac());
//! assert!(!wifi_protocol::restart::valid_original_mac(wifi_protocol::restart::FIRST_AP_MAC));
//! # Ok::<(), wifi_protocol::FrameError>(())
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};

pub use super::{
    AP_CHANNEL, AP_WINDOW_MILLIS, FrameError, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, SSID_LEN,
    SSID_PREFIX, ssid,
};

/// Wire version for this two-phase restart protocol.
pub const WIRE_VERSION: u32 = 2;
/// First fixed synthetic locally administered unicast control.
pub const FIRST_AP_MAC: [u8; 6] = super::AP_MAC;
/// Second fixed synthetic locally administered unicast control.
pub const SECOND_AP_MAC: [u8; 6] = [2, 23, 42, 65, 83, 114];

/// The only two ordered start attempts permitted in a boot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Initial start, before any restart.
    First,
    /// Restart after checked first-phase Stop and MAC restoration.
    Second,
}

impl Phase {
    /// Return the fixed synthetic MAC for this phase; callers cannot choose one.
    pub const fn mac(self) -> [u8; 6] {
        match self {
            Self::First => FIRST_AP_MAC,
            Self::Second => SECOND_AP_MAC,
        }
    }
}

/// Fixed control operations; no arbitrary radio setting is accepted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Inspect state without initializing or starting Wi-Fi.
    Status,
    /// Consume the initial start attempt.
    StartFirst,
    /// Consume the restart attempt after a clean checked first-phase Stop.
    StartSecond,
    /// Stop and restore the once-saved original MAC, including after failure.
    Stop,
}

/// Application state, qualified by [`Snapshot::phase`] and driver readbacks.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// No start attempt has been consumed.
    Idle,
    /// The current phase reports active with its fixed control MAC.
    Active,
    /// The attempted phase is inactive with its original MAC restored.
    Stopped,
}

/// One bounded request. The live controller additionally enforces increasing
/// nonzero IDs and one session nonce. A nonce is correlation, not authentication.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    version: u32,
    /// Strictly increasing nonzero boot-local request ID.
    pub id: u32,
    /// Requested fixed operation.
    pub operation: Operation,
    /// One nonzero nonce shared across every request and both phases.
    pub nonce: [u8; 8],
}

impl Request {
    /// Construct a version-two request; see the module's framing example.
    pub const fn new(id: u32, operation: Operation, nonce: [u8; 8]) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            operation,
            nonce,
        }
    }

    /// Validate stateless bounds; sequence/nonce binding belongs to the controller.
    ///
    /// # Errors
    /// Returns a static version/invalid-frame error without private values.
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
        f.debug_struct("RestartRequest")
            .field("operation", &self.operation)
            .field("correlation", &"<redacted>")
            .finish()
    }
}

/// Private driver readback plus the controller's terminal failure latch.
///
/// Status/Stop can succeed with `first_failure=Some` to verify cleanup. Such a
/// session remains failed and no later Start can be authorized. Debug redacts
/// nonce and addresses; raw serialization belongs only in private captures.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    /// Current application state.
    pub state: State,
    /// Last consumed start attempt; absent before the first attempt.
    pub phase: Option<Phase>,
    /// Bound nonce after a start attempt; absent in Idle.
    pub nonce: Option<[u8; 8]>,
    /// Whether an initialized owned driver remains retained until reset.
    pub initialized: bool,
    /// Event-backed AP running-state observation.
    pub running: bool,
    /// Current AP driver MAC, absent before initialization.
    pub mac: Option<[u8; 6]>,
    /// Original AP MAC saved once, never replaced between phases.
    pub original_mac: Option<[u8; 6]>,
    /// An autonomous phase deadline requested Stop for this phase.
    pub auto_stopped: bool,
    /// First nonzero protocol, transport, adapter or automatic-stop failure.
    pub first_failure: Option<i32>,
}

/// Whether an original driver address is valid and distinct from both controls.
/// See the module's example; this does not establish population uniqueness.
pub fn valid_original_mac(mac: [u8; 6]) -> bool {
    mac != [0; 6] && mac[0] & 1 == 0 && mac != FIRST_AP_MAC && mac != SECOND_AP_MAC
}

impl Snapshot {
    fn validate(&self) -> Result<(), FrameError> {
        if self.first_failure == Some(0) {
            return Err(FrameError::State);
        }
        let attempted = self.phase.is_some() && self.nonce.is_some_and(|nonce| nonce != [0; 8]);
        let original = self.original_mac.is_some_and(valid_original_mac);
        let valid = match self.state {
            State::Idle => {
                self.phase.is_none()
                    && self.nonce.is_none()
                    && !self.initialized
                    && !self.running
                    && self.mac.is_none()
                    && self.original_mac.is_none()
                    && !self.auto_stopped
            }
            State::Active => {
                attempted
                    && self.initialized
                    && self.running
                    && original
                    && self.mac == self.phase.map(Phase::mac)
                    && (!self.auto_stopped || self.first_failure.is_some())
            }
            State::Stopped => {
                attempted
                    && !self.running
                    && if self.initialized {
                        original && self.mac == self.original_mac
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
        f.debug_struct("RestartSnapshot")
            .field("state", &self.state)
            .field("phase", &self.phase)
            .field("initialized", &self.initialized)
            .field("running", &self.running)
            .field("auto_stopped", &self.auto_stopped)
            .field("first_failure", &self.first_failure)
            .field("private_values", &"<redacted>")
            .finish()
    }
}

/// Correlated response; a current operation error contains no partial snapshot.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    version: u32,
    /// Echoed request ID.
    pub id: u32,
    /// Echoed operation.
    pub operation: Operation,
    /// Zero for a currently successful operation; nonzero raw SDK/application error.
    pub status: i32,
    /// Complete readback on current success, possibly retaining terminal failure.
    pub snapshot: Option<Snapshot>,
}

impl Response {
    /// Construct a version-two response; validate before admission.
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

    /// Validate correlation and operation state. A successful cleanup with a
    /// latched failure remains decodable; trial admission must reject that latch.
    ///
    /// # Errors
    /// Returns a static framing, device, correlation or inconsistent-state error.
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
            Operation::Stop if matches!(snapshot.state, State::Idle | State::Stopped) => Ok(()),
            Operation::StartFirst | Operation::StartSecond
                if snapshot.state == State::Active
                    && snapshot.first_failure.is_none()
                    && !snapshot.auto_stopped
                    && snapshot.phase
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

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestartResponse")
            .field("operation", &self.operation)
            .field("status", &self.status)
            .field("snapshot", &self.snapshot)
            .field("correlation", &"<redacted>")
            .finish()
    }
}

/// Encode one validated LF-terminated request; see the module example.
///
/// # Errors
/// Returns a static invalid-frame/version/capacity error.
pub fn encode_request(request: &Request, bytes: &mut [u8]) -> Result<usize, FrameError> {
    request.validate()?;
    super::encode(request, bytes, MAX_REQUEST_BYTES)
}

/// Encode one shape-validated LF-terminated response.
///
/// # Errors
/// Returns a static invalid-state/version/capacity error.
pub fn encode_response(response: &Response, bytes: &mut [u8]) -> Result<usize, FrameError> {
    response.validate_shape()?;
    super::encode(response, bytes, MAX_RESPONSE_BYTES)
}

/// Decode a bounded request, rejecting unknown fields and trailing data.
///
/// # Errors
/// Returns a static invalid-frame/version/capacity error.
pub fn decode_request(bytes: &[u8]) -> Result<Request, FrameError> {
    let value: Request = super::decode(bytes, MAX_REQUEST_BYTES)?;
    value.validate()?;
    Ok(value)
}

/// Decode response shape; source authenticity and live continuity are unproved.
///
/// # Errors
/// Returns a static invalid-frame/state/version/capacity error.
pub fn decode_response(bytes: &[u8]) -> Result<Response, FrameError> {
    let value: Response = super::decode(bytes, MAX_RESPONSE_BYTES)?;
    value.validate_shape()?;
    Ok(value)
}

#[cfg(test)]
#[path = "../../../firmware-wifi/src/restart_controller.rs"]
mod firmware_controller;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::format;

    fn active(phase: Phase) -> Snapshot {
        Snapshot {
            state: State::Active,
            phase: Some(phase),
            nonce: Some([7; 8]),
            initialized: true,
            running: true,
            mac: Some(phase.mac()),
            original_mac: Some([48; 6]),
            auto_stopped: false,
            first_failure: None,
        }
    }

    #[test]
    fn wire_phase_correlations_failure_latch_and_legacy_separation() {
        for (phase, operation) in [
            (Phase::First, Operation::StartFirst),
            (Phase::Second, Operation::StartSecond),
        ] {
            let request = Request::new(1, operation, [7; 8]);
            let response = Response::new(1, operation, 0, Some(active(phase)));
            let mut bytes = [0; MAX_RESPONSE_BYTES];
            let n = encode_response(&response, &mut bytes).unwrap();
            assert_eq!(decode_response(&bytes[..n]).unwrap(), response);
            assert_eq!(response.validate_for(&request), Ok(()));
            assert!(super::super::decode_response(&bytes[..n]).is_err());
            let wrong = Response::new(
                1,
                operation,
                0,
                Some(active(if phase == Phase::First {
                    Phase::Second
                } else {
                    Phase::First
                })),
            );
            assert!(wrong.validate_for(&request).is_err());
            let failed = Snapshot {
                first_failure: Some(55),
                ..active(phase)
            };
            assert!(
                Response::new(1, operation, 0, Some(failed))
                    .validate_for(&request)
                    .is_err()
            );
            let stopped = Snapshot {
                state: State::Stopped,
                running: false,
                mac: failed.original_mac,
                ..failed
            };
            assert!(
                Response::new(2, Operation::Stop, 0, Some(stopped))
                    .validate_for(&Request::new(2, Operation::Stop, [7; 8]))
                    .is_ok()
            );
        }
    }

    #[test]
    fn strict_bounds_private_debug_and_invalid_originals() {
        for mac in [[0; 6], [1; 6], FIRST_AP_MAC, SECOND_AP_MAC] {
            assert!(!valid_original_mac(mac));
        }
        let request = Request::new(1, Operation::Status, [7; 8]);
        assert!(!format!("{request:?} {:?}", active(Phase::First)).contains("[7, 7"));
        assert_eq!(encode_request(&request, &mut []), Err(FrameError::Overflow));
        assert!(
            decode_request(
                b"{\"version\":1,\"id\":1,\"operation\":\"status\",\"nonce\":[7,7,7,7,7,7,7,7]}"
            )
            .is_err()
        );
        assert!(decode_request(b"{\"version\":2,\"id\":1,\"operation\":\"status\",\"nonce\":[7,7,7,7,7,7,7,7],\"unknown\":0}").is_err());
    }
}
