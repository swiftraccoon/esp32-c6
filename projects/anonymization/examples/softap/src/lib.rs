//! Example-local private UART records and deadline admission.
//!
//! This is a trusted attached-host experiment protocol, not authentication.
//! The target adapter owns SDK transitions; this module only bounds requests,
//! orders the two attempts and retains transport/deadline failures.
#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

use serde::{Deserialize, Serialize};

/// Complete LF-terminated request bound.
pub const REQUEST_BYTES: usize = 256;
/// Complete LF-terminated reply bound.
pub const REPLY_BYTES: usize = 1024;
/// Requested stop deadline, without a hard real-time or RF-duration guarantee.
pub const WINDOW_MS: u64 = 8000;
/// Fixed protocol/transport failure.
pub const PROTOCOL_ERROR: i32 = -4101;
/// Fixed identity-adapter failure; SDK details remain in the adapter error.
pub const ADAPTER_ERROR: i32 = -4102;
/// Fixed elapsed-deadline failure.
pub const DEADLINE_ERROR: i32 = -4103;

/// Fixed example operations.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Read the prepared identities and current SDK state.
    Status,
    /// Start the first prepared identity once.
    StartFirst,
    /// Request checked inactive state; repeatable for recovery.
    Stop,
    /// Start the second identity after an explicit checked Stop.
    StartSecond,
}

/// Private host request, validated before any action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Required namespace `anon_softap`.
    pub domain: Domain,
    /// Required format version one.
    pub version: u32,
    /// Increasing nonzero request identifier.
    pub id: u32,
    /// Fixed requested operation.
    pub operation: Operation,
}

/// Namespace prevents admitting another experiment's request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Domain {
    /// This example's only namespace.
    #[serde(rename = "anon_softap")]
    SoftAp,
}

/// Decode one request without LF; require the entire bounded frame.
///
/// # Errors
/// Returns a fixed error for oversized, malformed, trailing, wrong-version or
/// zero-ID records. Unknown and duplicate fields are rejected by serde.
pub fn decode(bytes: &[u8]) -> Result<Request, i32> {
    if bytes.is_empty() || bytes.len() >= REQUEST_BYTES {
        return Err(PROTOCOL_ERROR);
    }
    let (request, consumed): (Request, usize) =
        serde_json_core::from_slice(bytes).map_err(|_| PROTOCOL_ERROR)?;
    if consumed != bytes.len() || request.version != 1 || request.id == 0 {
        return Err(PROTOCOL_ERROR);
    }
    Ok(request)
}

/// Private expected identity retained by the attached host.
#[derive(Clone, Copy, Serialize)]
pub struct Expected {
    /// Explicitly requested AP address.
    pub mac: [u8; 6],
    /// Exactly 21 ASCII SSID bytes.
    pub ssid: [u8; 21],
}

impl fmt::Debug for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expected")
            .field("contents", &"redacted")
            .finish()
    }
}

/// Private UART reply. Serialize only into ignored attached-host captures.
#[derive(Serialize)]
pub struct Reply {
    /// Fixed namespace.
    pub domain: Domain,
    /// Format version one.
    pub version: u32,
    /// Request correlation, zero only for an uncorrelatable malformed frame.
    pub id: u32,
    /// Requested operation.
    pub operation: Operation,
    /// Immediate operation status.
    pub status: i32,
    /// Sticky first session error, including errors followed by recovery.
    pub failure: Option<i32>,
    /// SDK state/readback observation, not RF attestation.
    pub active: bool,
    /// Current driver address readback, if available.
    pub readback: Option<[u8; 6]>,
    /// First prepared expected identity.
    pub first: Expected,
    /// Second prepared expected identity.
    pub second: Expected,
}

impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reply")
            .field("operation", &self.operation)
            .field("status", &self.status)
            .field("failure", &self.failure)
            .field("active", &self.active)
            .finish_non_exhaustive()
    }
}

/// Serialize one bounded private reply including its LF.
///
/// # Errors
/// Returns a fixed error when caller storage is too small.
pub fn encode(reply: &Reply, bytes: &mut [u8]) -> Result<usize, i32> {
    let capacity = bytes.len().min(REPLY_BYTES);
    let payload = capacity.checked_sub(1).ok_or(PROTOCOL_ERROR)?;
    let length =
        serde_json_core::to_slice(reply, &mut bytes[..payload]).map_err(|_| PROTOCOL_ERROR)?;
    bytes[length] = b'\n';
    Ok(length + 1)
}

/// Boot-local admission and deadline state. No hardware calls or allocation.
#[derive(Debug, Default)]
pub struct Session {
    last_id: u32,
    first_attempted: bool,
    explicit_stop: bool,
    second_attempted: bool,
    failure: Option<i32>,
    deadline: Option<u64>,
    next_cleanup: u64,
    cleanup_pending: bool,
}

impl Session {
    /// Admit before calling the adapter; attempts and deadlines are prearmed.
    ///
    /// # Errors
    /// Rejects replayed IDs, reordered/repeated Starts or a Start after failure.
    pub fn admit(&mut self, request: Request, now_ms: u64) -> Result<(), i32> {
        if request.version != 1 || request.id == 0 || request.id <= self.last_id {
            self.fail(PROTOCOL_ERROR);
            return Err(PROTOCOL_ERROR);
        }
        self.last_id = request.id;
        match request.operation {
            Operation::Status | Operation::Stop => return Ok(()),
            Operation::StartFirst if !self.first_attempted && self.failure.is_none() => {
                self.first_attempted = true;
            }
            Operation::StartSecond
                if self.first_attempted
                    && self.explicit_stop
                    && !self.second_attempted
                    && self.failure.is_none() =>
            {
                self.second_attempted = true;
            }
            _ => {
                self.fail(PROTOCOL_ERROR);
                return Err(PROTOCOL_ERROR);
            }
        }
        self.explicit_stop = false;
        self.deadline = now_ms.checked_add(WINDOW_MS);
        self.next_cleanup = 0;
        self.cleanup_pending = false;
        if self.deadline.is_none() {
            self.fail(DEADLINE_ERROR);
            return Err(DEADLINE_ERROR);
        }
        Ok(())
    }

    /// Retain the first nonzero failure permanently.
    pub fn fail(&mut self, code: i32) {
        if code != 0 && self.failure.is_none() {
            self.failure = Some(code);
        }
    }

    /// First retained session error, without private identity data.
    pub const fn failure(&self) -> Option<i32> {
        self.failure
    }

    /// Reject a successful SDK Start that completed at or after its deadline.
    ///
    /// # Errors
    /// Returns a fixed error and preserves it as the first session failure.
    pub fn start_completed(&mut self, now_ms: u64) -> Result<(), i32> {
        if self.deadline.is_some_and(|deadline| now_ms < deadline) {
            Ok(())
        } else {
            self.fail(DEADLINE_ERROR);
            Err(DEADLINE_ERROR)
        }
    }

    /// Request immediate checked cleanup after a transport or SDK failure.
    pub fn request_cleanup(&mut self, now_ms: u64) {
        if !self.cleanup_pending {
            self.deadline = Some(now_ms);
            self.next_cleanup = 0;
            self.cleanup_pending = true;
        }
    }

    /// A due deadline requests cleanup and throttles later retries to one second.
    pub fn cleanup_due(&mut self, now_ms: u64) -> bool {
        if self.deadline.is_some_and(|deadline| now_ms >= deadline) && now_ms >= self.next_cleanup {
            self.cleanup_pending = true;
            self.next_cleanup = now_ms.saturating_add(1000);
            return true;
        }
        false
    }

    /// Record checked cleanup; only an explicit request authorizes phase two.
    pub fn stopped(&mut self, result: Result<(), i32>, explicit: bool) {
        match result {
            Ok(()) => {
                self.deadline = None;
                self.cleanup_pending = false;
                if explicit && self.first_attempted && !self.second_attempted {
                    self.explicit_stop = true;
                }
            }
            Err(code) => self.fail(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: u32, operation: Operation) -> Request {
        Request {
            domain: Domain::SoftAp,
            version: 1,
            id,
            operation,
        }
    }

    #[test]
    fn malformed_foreign_and_duplicate_records_reject() {
        let good = br#"{"domain":"anon_softap","version":1,"id":1,"operation":"status"}"#;
        assert_eq!(decode(good), Ok(request(1, Operation::Status)));
        for bad in [
            br#"{"domain":"other","version":1,"id":1,"operation":"status"}"#.as_slice(),
            br#"{"domain":"anon_softap","version":1,"id":1,"id":2,"operation":"status"}"#,
            br#"{"domain":"anon_softap","version":1,"id":1,"operation":"status","extra":0}"#,
            br#"{"domain":"anon_softap","version":2,"id":1,"operation":"status"}"#,
        ] {
            assert!(decode(bad).is_err());
        }
        assert!(decode(&[b' '; REQUEST_BYTES]).is_err());
    }

    #[test]
    fn automatic_stop_does_not_authorize_second_attempt() {
        let mut session = Session::default();
        session.admit(request(1, Operation::StartFirst), 5).unwrap();
        assert!(!session.cleanup_due(8004));
        assert!(session.cleanup_due(8005));
        session.stopped(Ok(()), false);
        assert!(
            session
                .admit(request(2, Operation::StartSecond), 9000)
                .is_err()
        );
        session.stopped(Ok(()), true);
        assert!(
            session
                .admit(request(3, Operation::StartSecond), 9001)
                .is_err()
        );
    }

    #[test]
    fn checked_sequence_and_failed_cleanup_are_sticky() {
        let mut session = Session::default();
        session.admit(request(1, Operation::StartFirst), 0).unwrap();
        session.stopped(Ok(()), true);
        session
            .admit(request(2, Operation::StartSecond), 100)
            .unwrap();
        assert!(session.cleanup_due(8100));
        session.stopped(Err(-99), false);
        assert!(!session.cleanup_due(9099));
        assert!(session.cleanup_due(9100));
        session.stopped(Ok(()), true);
        assert_eq!(session.failure(), Some(-99));
        assert!(
            session
                .admit(request(3, Operation::StartSecond), 9200)
                .is_err()
        );
    }

    #[test]
    fn duplicate_id_and_deadline_overflow_cannot_start() {
        let mut session = Session::default();
        session.admit(request(1, Operation::Status), 0).unwrap();
        assert!(session.admit(request(1, Operation::StartFirst), 0).is_err());
        let mut overflow = Session::default();
        assert!(
            overflow
                .admit(request(1, Operation::StartFirst), u64::MAX)
                .is_err()
        );
    }

    #[test]
    fn late_start_and_transport_cleanup_cannot_be_rehabilitated() {
        let mut session = Session::default();
        session.admit(request(1, Operation::StartFirst), 0).unwrap();
        assert!(session.start_completed(7999).is_ok());
        assert_eq!(session.start_completed(8000), Err(DEADLINE_ERROR));
        session.request_cleanup(8000);
        assert!(session.cleanup_due(8000));
        session.stopped(Ok(()), true);
        assert_eq!(session.failure(), Some(DEADLINE_ERROR));
        assert!(
            session
                .admit(request(2, Operation::StartSecond), 8001)
                .is_err()
        );
    }

    #[test]
    fn repeated_transport_errors_preserve_cleanup_retry_throttle() {
        let mut session = Session::default();
        session.admit(request(1, Operation::StartFirst), 0).unwrap();
        session.fail(PROTOCOL_ERROR);
        session.request_cleanup(10);
        assert!(session.cleanup_due(10));
        session.stopped(Err(-99), false);
        for now in [11, 100, 1009] {
            session.request_cleanup(now);
            assert!(!session.cleanup_due(now));
        }
        assert!(session.cleanup_due(1010));
        assert_eq!(session.failure(), Some(PROTOCOL_ERROR));
    }
}
