//! Pure two-attempt sequencing and independent cleanup deadlines.
use crate::protocol::{Operation, Phase, Request, Response, Snapshot, WINDOW_MILLIS};

/// Static protocol/framing failure.
pub const PROTOCOL_FAILURE: i32 = -3001;
/// Monotonic ID, nonce or phase ordering failure.
pub const ORDER_FAILURE: i32 = -3002;
/// Contradictory adapter result.
pub const ADAPTER_FAILURE: i32 = -3003;
/// Deadline arithmetic failure.
pub const CLOCK_FAILURE: i32 = -3004;

/// Require a selected advertising Start at the hardware boundary.
///
/// This checks the operation only; the controller separately enforces attempt
/// ordering, nonce binding, deadlines and checked Stop. It opens no device.
///
/// ```
/// use device_identity_ble_firmware::controller::check_advertise_operation;
/// use device_identity_ble_firmware::protocol::Operation;
/// assert!(check_advertise_operation(Operation::Status).is_err());
/// assert!(check_advertise_operation(Operation::StartFirst).is_ok());
/// ```
///
/// # Errors
/// Returns a static adapter failure for Status or Stop.
pub fn check_advertise_operation(operation: Operation) -> Result<(), i32> {
    if matches!(operation, Operation::StartFirst | Operation::StartSecond) {
        Ok(())
    } else {
        Err(ADAPTER_FAILURE)
    }
}

/// Exclusive main-task radio adapter; callbacks must not call this trait.
pub trait Radio {
    /// Inspect without implicitly initializing.
    ///
    /// # Errors
    /// Returns a private nonzero SDK/static adapter status on failure.
    fn status(&mut self) -> Result<Snapshot, i32>;
    /// Consume one phase, with its prearmed absolute epoch-relative deadline.
    ///
    /// # Errors
    /// Returns a private nonzero SDK/static status; cleanup must still be tried.
    fn start(&mut self, phase: Phase, nonce: [u8; 8], deadline_ms: u64) -> Result<Snapshot, i32>;
    /// Checked stop; random state/driver can remain retained until reset.
    ///
    /// # Errors
    /// Returns a private nonzero SDK/static status; failed cleanup stays failed.
    fn stop(&mut self, nonce: [u8; 8], automatic: bool) -> Result<Snapshot, i32>;
    /// Permanent asynchronous callback failure, if observed.
    fn fault(&self) -> Option<i32> {
        None
    }
    /// Late start/unknown activity after failed cleanup needs another Stop.
    fn cleanup_needed(&self) -> bool {
        false
    }
}
#[derive(Clone, Copy)]
struct Deadline {
    at: u64,
    automatic: bool,
}
/// One retained adapter and exactly two consumed attempts, never reset in place.
pub struct Controller<R> {
    radio: R,
    nonce: Option<[u8; 8]>,
    last_id: u32,
    phase: Option<Phase>,
    first_start_ok: bool,
    first_stop_ok: bool,
    initialized_seen: bool,
    first_failure: Option<i32>,
    deadline: Option<Deadline>,
}
impl<R> core::fmt::Debug for Controller<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BleController { <redacted> }")
    }
}
fn nonzero(code: i32) -> i32 {
    if code == 0 { ADAPTER_FAILURE } else { code }
}
impl<R: Radio> Controller<R> {
    /// Retain an idle adapter without initializing the radio.
    pub fn new(radio: R) -> Self {
        Self {
            radio,
            nonce: None,
            last_id: 0,
            phase: None,
            first_start_ok: false,
            first_stop_ok: false,
            initialized_seen: false,
            first_failure: None,
            deadline: None,
        }
    }
    fn latch(&mut self, code: i32) {
        self.first_failure.get_or_insert(nonzero(code));
    }
    fn checked(&mut self, op: Operation, nonce: [u8; 8], mut s: Snapshot) -> Result<Snapshot, i32> {
        if s.first_failure.is_some()
            || s.phase != self.phase
            || s.nonce != self.phase.map(|_| nonce)
            || (self.initialized_seen && !s.initialized)
        {
            return Err(ADAPTER_FAILURE);
        }
        if matches!(op, Operation::StartFirst | Operation::StartSecond)
            && self.radio.fault().is_some()
        {
            return Err(self.radio.fault().unwrap_or(ADAPTER_FAILURE));
        }
        s.first_failure = self.first_failure;
        Response::new(1, op, 0, Some(s))
            .validate_for(&Request::new(1, op, nonce))
            .map_err(|_| ADAPTER_FAILURE)?;
        self.initialized_seen |= s.initialized;
        Ok(s)
    }
    fn stop(&mut self, automatic: bool, now: u64) -> Result<Snapshot, i32> {
        let nonce = self.nonce.ok_or(ADAPTER_FAILURE)?;
        let result = self
            .radio
            .stop(nonce, automatic)
            .and_then(|s| self.checked(Operation::Stop, nonce, s));
        match result {
            Ok(s) => {
                self.deadline = None;
                Ok(s)
            }
            Err(code) => {
                self.latch(code);
                if self.phase.is_some() {
                    self.deadline = Some(Deadline {
                        at: now.saturating_add(1000),
                        automatic,
                    });
                }
                Err(nonzero(code))
            }
        }
    }
    fn fail(&mut self, code: i32, now: u64) {
        self.latch(code);
        if self.phase.is_some() {
            let _ = self.stop(false, now);
        }
    }
    /// Poll unsolicited callback failures, independent stop deadlines and failed
    /// cleanup retries, including while UART is idle or backpressured.
    pub fn poll(&mut self, now: u64) {
        if self.first_failure.is_none()
            && let Some(code) = self.radio.fault()
        {
            self.fail(code, now);
        }
        if self.first_failure.is_some()
            && self.phase.is_some()
            && self.deadline.is_none()
            && self.radio.cleanup_needed()
        {
            let _ = self.stop(false, now);
        }
        if let Some(d) = self.deadline.filter(|d| now >= d.at) {
            self.deadline = None;
            let _ = self.stop(d.automatic, now);
        }
    }
    fn failed(&mut self, r: Request, code: i32, now: u64) -> Response {
        self.fail(code, now);
        Response::new(r.id.max(1), r.operation, nonzero(code), None)
    }
    /// Enforce domain/version/ID/nonce, ordered explicit Stop and sticky failure.
    pub fn handle(&mut self, r: Request, now: u64) -> Response {
        self.poll(now);
        if r.validate().is_err() {
            return self.failed(r, PROTOCOL_FAILURE, now);
        }
        if r.id <= self.last_id {
            return self.failed(r, ORDER_FAILURE, now);
        }
        self.last_id = r.id;
        match self.nonce {
            Some(n) if n != r.nonce => return self.failed(r, ORDER_FAILURE, now),
            None => self.nonce = Some(r.nonce),
            _ => {}
        }
        let result = match r.operation {
            Operation::Status => self
                .radio
                .status()
                .and_then(|s| self.checked(Operation::Status, r.nonce, s)),
            Operation::Stop => {
                let result = self.stop(false, now);
                if result.is_ok()
                    && self.first_failure.is_none()
                    && self.first_start_ok
                    && self.phase == Some(Phase::First)
                {
                    self.first_stop_ok = true;
                }
                result
            }
            Operation::StartFirst | Operation::StartSecond => {
                let phase = if r.operation == Operation::StartFirst {
                    Phase::First
                } else {
                    Phase::Second
                };
                if self.first_failure.is_some()
                    || !match phase {
                        Phase::First => self.phase.is_none(),
                        Phase::Second => self.phase == Some(Phase::First) && self.first_stop_ok,
                    }
                {
                    return self.failed(r, ORDER_FAILURE, now);
                }
                let Some(at) = now.checked_add(u64::from(WINDOW_MILLIS)) else {
                    return self.failed(r, CLOCK_FAILURE, now);
                };
                self.phase = Some(phase);
                self.first_stop_ok = false;
                self.deadline = Some(Deadline {
                    at,
                    automatic: true,
                });
                let result = self
                    .radio
                    .start(phase, r.nonce, at)
                    .and_then(|s| self.checked(r.operation, r.nonce, s));
                if result.is_ok() && phase == Phase::First {
                    self.first_start_ok = true;
                }
                result
            }
        };
        match result {
            Ok(s) => Response::new(r.id, r.operation, 0, Some(s)),
            Err(code) if r.operation == Operation::Stop => {
                Response::new(r.id, r.operation, nonzero(code), None)
            }
            Err(code) => self.failed(r, code, now),
        }
    }
    /// A framing/transport failure permanently blocks Starts and attempts Stop.
    pub fn transport_failed(&mut self, code: i32, now: u64) {
        self.fail(code, now);
    }
}

#[cfg(test)]
mod tests;
