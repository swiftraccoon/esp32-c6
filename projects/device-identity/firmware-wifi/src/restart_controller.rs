//! Pure v2 controller: exactly two ordered attempts, terminal failure latch.

use wifi_protocol::restart::{AP_WINDOW_MILLIS, Operation, Phase, Request, Response, Snapshot};

pub(crate) const PROTOCOL_FAILURE: i32 = -2001;
pub(crate) const ORDER_FAILURE: i32 = -2002;
pub(crate) const ADAPTER_FAILURE: i32 = -2003;
pub(crate) const CLOCK_FAILURE: i32 = -2004;

pub(crate) trait Radio {
    fn status(&mut self) -> Result<Snapshot, i32>;
    fn start(&mut self, phase: Phase, nonce: [u8; 8]) -> Result<Snapshot, i32>;
    fn stop(&mut self, nonce: [u8; 8], automatic: bool) -> Result<Snapshot, i32>;
}

#[derive(Clone, Copy)]
struct Deadline {
    at_ms: u64,
    automatic: bool,
}

pub(crate) struct Controller<R> {
    radio: R,
    nonce: Option<[u8; 8]>,
    last_id: u32,
    phase: Option<Phase>,
    first_start_ok: bool,
    first_stop_ok: bool,
    original_mac: Option<[u8; 6]>,
    first_failure: Option<i32>,
    deadline: Option<Deadline>,
}

fn nonzero(code: i32) -> i32 {
    if code == 0 { ADAPTER_FAILURE } else { code }
}

impl<R: Radio> Controller<R> {
    pub(crate) fn new(radio: R) -> Self {
        Self {
            radio,
            nonce: None,
            last_id: 0,
            phase: None,
            first_start_ok: false,
            first_stop_ok: false,
            original_mac: None,
            first_failure: None,
            deadline: None,
        }
    }

    fn latch(&mut self, code: i32) {
        self.first_failure.get_or_insert(nonzero(code));
    }

    fn checked(
        &mut self,
        operation: Operation,
        nonce: [u8; 8],
        mut snapshot: Snapshot,
    ) -> Result<Snapshot, i32> {
        // The adapter cannot supply controller failure or phase/nonce history.
        if snapshot.first_failure.is_some()
            || snapshot.phase != self.phase
            || snapshot.nonce != self.phase.map(|_| nonce)
            || self
                .original_mac
                .is_some_and(|original| snapshot.original_mac != Some(original))
        {
            return Err(ADAPTER_FAILURE);
        }
        snapshot.first_failure = self.first_failure;
        Response::new(1, operation, 0, Some(snapshot))
            .validate_for(&Request::new(1, operation, nonce))
            .map_err(|_| ADAPTER_FAILURE)?;
        if self.original_mac.is_none() {
            self.original_mac = snapshot.original_mac;
        }
        Ok(snapshot)
    }

    fn stop(&mut self, automatic: bool, now_ms: u64) -> Result<Snapshot, i32> {
        let nonce = self.nonce.ok_or(ADAPTER_FAILURE)?;
        let result = self
            .radio
            .stop(nonce, automatic)
            .and_then(|snapshot| self.checked(Operation::Stop, nonce, snapshot));
        match result {
            Ok(snapshot) => {
                self.deadline = None;
                Ok(snapshot)
            }
            Err(code) => {
                self.latch(code);
                if self.phase.is_some() {
                    self.deadline = Some(Deadline {
                        at_ms: now_ms.saturating_add(1000),
                        automatic,
                    });
                }
                Err(nonzero(code))
            }
        }
    }

    /// Poll both the phase deadline and failed-cleanup retries without UART.
    /// SDK calls and scheduling prevent any hard on-air duration guarantee.
    pub(crate) fn poll(&mut self, now_ms: u64) {
        if let Some(deadline) = self.deadline.filter(|deadline| now_ms >= deadline.at_ms) {
            self.deadline = None;
            let _ = self.stop(deadline.automatic, now_ms);
        }
    }

    fn fail(&mut self, code: i32, now_ms: u64) {
        self.latch(code);
        if self.phase.is_some() {
            let _ = self.stop(false, now_ms);
        }
    }

    fn failure_response(&mut self, request: Request, code: i32, now_ms: u64) -> Response {
        self.fail(code, now_ms);
        Response::new(request.id.max(1), request.operation, nonzero(code), None)
    }

    /// Every valid decoded request consumes a strictly greater ID. A failure
    /// permanently inhibits both Starts even when later checked cleanup works.
    pub(crate) fn handle(&mut self, request: Request, now_ms: u64) -> Response {
        self.poll(now_ms);
        if request.validate().is_err() {
            return self.failure_response(request, PROTOCOL_FAILURE, now_ms);
        }
        if request.id <= self.last_id {
            return self.failure_response(request, ORDER_FAILURE, now_ms);
        }
        self.last_id = request.id;
        match self.nonce {
            Some(nonce) if nonce != request.nonce => {
                return self.failure_response(request, ORDER_FAILURE, now_ms);
            }
            None => self.nonce = Some(request.nonce),
            _ => (),
        }
        let result = match request.operation {
            Operation::Status => self
                .radio
                .status()
                .and_then(|snapshot| self.checked(Operation::Status, request.nonce, snapshot)),
            Operation::Stop => {
                let result = self.stop(false, now_ms);
                if result.is_ok()
                    && self.first_failure.is_none()
                    && self.first_start_ok
                    && self.phase == Some(Phase::First)
                {
                    // A successful automatic Stop or Status is insufficient.
                    self.first_stop_ok = true;
                }
                result
            }
            Operation::StartFirst | Operation::StartSecond => {
                let phase = if request.operation == Operation::StartFirst {
                    Phase::First
                } else {
                    Phase::Second
                };
                let allowed = self.first_failure.is_none()
                    && match phase {
                        Phase::First => self.phase.is_none(),
                        Phase::Second => self.phase == Some(Phase::First) && self.first_stop_ok,
                    };
                if !allowed {
                    return self.failure_response(request, ORDER_FAILURE, now_ms);
                }
                let Some(deadline) = now_ms.checked_add(u64::from(AP_WINDOW_MILLIS)) else {
                    return self.failure_response(request, CLOCK_FAILURE, now_ms);
                };
                // Consume the attempt and arm cleanup before fallible setup.
                self.phase = Some(phase);
                self.first_stop_ok = false;
                self.deadline = Some(Deadline {
                    at_ms: deadline,
                    automatic: true,
                });
                let result = self
                    .radio
                    .start(phase, request.nonce)
                    .and_then(|snapshot| self.checked(request.operation, request.nonce, snapshot));
                if result.is_ok() && phase == Phase::First {
                    self.first_start_ok = true;
                }
                result
            }
        };
        match result {
            Ok(snapshot) => Response::new(request.id, request.operation, 0, Some(snapshot)),
            // stop() already latched the error and armed its bounded retry.
            Err(code) if request.operation == Operation::Stop => {
                Response::new(request.id, request.operation, nonzero(code), None)
            }
            Err(code) => self.failure_response(request, code, now_ms),
        }
    }

    /// Framing/write/read failure also inhibits every later Start. Keep the
    /// first raw error while promptly attempting cleanup and autonomous retries.
    pub(crate) fn transport_failed(&mut self, code: i32, now_ms: u64) {
        self.fail(code, now_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wifi_protocol::restart::State;

    struct Fake {
        snapshot: Snapshot,
        initializations: u32,
        starts: u32,
        stops: u32,
        start_error: Option<i32>,
        stop_error: Option<i32>,
        invalid_stop: bool,
    }

    impl Fake {
        fn idle() -> Self {
            Self {
                snapshot: Snapshot {
                    state: State::Idle,
                    phase: None,
                    nonce: None,
                    initialized: false,
                    running: false,
                    mac: None,
                    original_mac: None,
                    auto_stopped: false,
                    first_failure: None,
                },
                initializations: 0,
                starts: 0,
                stops: 0,
                start_error: None,
                stop_error: None,
                invalid_stop: false,
            }
        }
    }

    impl Radio for Fake {
        fn status(&mut self) -> Result<Snapshot, i32> {
            Ok(self.snapshot)
        }
        fn start(&mut self, phase: Phase, nonce: [u8; 8]) -> Result<Snapshot, i32> {
            if !self.snapshot.initialized {
                self.initializations += 1;
            }
            self.starts += 1;
            self.snapshot = Snapshot {
                state: State::Active,
                phase: Some(phase),
                nonce: Some(nonce),
                initialized: true,
                running: true,
                mac: Some(phase.mac()),
                original_mac: Some([48; 6]),
                auto_stopped: false,
                first_failure: None,
            };
            self.start_error.map_or(Ok(self.snapshot), Err)
        }
        fn stop(&mut self, _: [u8; 8], automatic: bool) -> Result<Snapshot, i32> {
            self.stops += 1;
            if let Some(code) = self.stop_error {
                return Err(code);
            }
            if self.snapshot.phase.is_some() {
                self.snapshot.state = State::Stopped;
                self.snapshot.running = self.invalid_stop;
                self.snapshot.mac = self.snapshot.original_mac;
                self.snapshot.auto_stopped |= automatic;
            }
            Ok(self.snapshot)
        }
    }

    fn request(id: u32, operation: Operation) -> Request {
        Request::new(id, operation, [7; 8])
    }

    fn first_stopped(controller: &mut Controller<Fake>) {
        assert_eq!(
            controller.handle(request(1, Operation::Status), 0).status,
            0
        );
        assert_eq!(
            controller
                .handle(request(2, Operation::StartFirst), 10)
                .status,
            0
        );
        assert_eq!(controller.handle(request(3, Operation::Stop), 20).status, 0);
    }

    #[test]
    fn same_boot_two_ordered_attempts_original_unchanged_and_no_third_start() {
        let mut controller = Controller::new(Fake::idle());
        first_stopped(&mut controller);
        let second = controller.handle(request(4, Operation::StartSecond), 30);
        assert_eq!(second.snapshot.unwrap().mac, Some(Phase::Second.mac()));
        let stopped = controller
            .handle(request(5, Operation::Stop), 40)
            .snapshot
            .unwrap();
        assert_eq!(stopped.mac, Some([48; 6]));
        assert_eq!(stopped.first_failure, None);
        assert_eq!(controller.radio.initializations, 1);
        assert_eq!(controller.radio.starts, 2);
        assert_ne!(
            controller
                .handle(request(6, Operation::StartSecond), 50)
                .status,
            0
        );
        assert_eq!(controller.radio.starts, 2);
    }

    #[test]
    fn auto_stop_and_status_do_not_authorize_second_start_or_reset_continuation() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(1, Operation::StartFirst), 0);
        controller.poll(8000);
        assert_eq!(
            controller
                .handle(request(2, Operation::Status), 8001)
                .status,
            0
        );
        assert_ne!(
            controller
                .handle(request(3, Operation::StartSecond), 8002)
                .status,
            0
        );
        assert_eq!(controller.radio.starts, 1);
        let mut rebooted = Controller::new(Fake::idle());
        assert_ne!(
            rebooted
                .handle(request(4, Operation::StartSecond), 0)
                .status,
            0
        );
        assert_eq!(rebooted.radio.initializations, 0);
    }

    #[test]
    fn phase_deadlines_rearm_and_explicit_stop_after_auto_stop_authorizes_once() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(1, Operation::StartFirst), 10);
        controller.poll(8009);
        assert_eq!(controller.radio.stops, 0);
        controller.poll(8010);
        assert!(
            controller
                .handle(request(2, Operation::Stop), 8020)
                .snapshot
                .unwrap()
                .auto_stopped
        );
        assert_eq!(
            controller
                .handle(request(3, Operation::StartSecond), 10000)
                .status,
            0
        );
        controller.poll(17999);
        assert!(controller.radio.snapshot.running);
        controller.poll(18000);
        assert!(!controller.radio.snapshot.running);
        assert_eq!(controller.radio.snapshot.phase, Some(Phase::Second));
    }

    #[test]
    fn all_uncertainty_is_terminal_for_starts_but_cleanup_remains_verifiable() {
        for failure in 0..4 {
            let mut controller = Controller::new(Fake::idle());
            first_stopped(&mut controller);
            match failure {
                0 => {
                    controller.transport_failed(77, 21);
                }
                1 => {
                    controller.handle(request(3, Operation::Status), 21);
                }
                2 => {
                    controller.handle(Request::new(4, Operation::Status, [8; 8]), 21);
                }
                _ => {
                    controller.handle(Request::new(4, Operation::Status, [0; 8]), 21);
                }
            }
            assert_ne!(
                controller
                    .handle(request(5, Operation::StartSecond), 30)
                    .status,
                0
            );
            let cleanup = controller
                .handle(request(6, Operation::Stop), 40)
                .snapshot
                .unwrap();
            assert!(cleanup.first_failure.is_some());
            assert!(!cleanup.running);
            assert_eq!(cleanup.mac, cleanup.original_mac);
            assert_eq!(controller.radio.starts, 1);
        }
    }

    #[test]
    fn partial_start_and_incorrect_restoration_never_authorize_second() {
        for partial_start in [false, true] {
            let mut radio = Fake::idle();
            if partial_start {
                radio.start_error = Some(55);
            }
            let mut controller = Controller::new(radio);
            controller.handle(request(1, Operation::StartFirst), 0);
            if !partial_start {
                controller.radio.invalid_stop = true;
            }
            let stop = controller.handle(request(2, Operation::Stop), 1);
            if partial_start {
                assert_eq!(stop.status, 0);
                assert_eq!(stop.snapshot.unwrap().first_failure, Some(55));
            } else {
                assert_eq!(stop.status, ADAPTER_FAILURE);
            }
            controller.radio.invalid_stop = false;
            assert!(
                controller
                    .handle(request(3, Operation::Stop), 2)
                    .snapshot
                    .unwrap()
                    .first_failure
                    .is_some()
            );
            assert_ne!(
                controller
                    .handle(request(4, Operation::StartSecond), 3)
                    .status,
                0
            );
            assert_eq!(controller.radio.starts, 1);
        }
    }

    #[test]
    fn failed_auto_stop_retries_without_uart_and_retains_first_error_forever() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(1, Operation::StartFirst), 0);
        controller.radio.stop_error = Some(66);
        controller.poll(8000);
        assert_eq!(controller.radio.stops, 1);
        controller.radio.stop_error = None;
        controller.poll(8999);
        assert_eq!(controller.radio.stops, 1);
        controller.poll(9000);
        assert_eq!(controller.radio.stops, 2);
        controller.transport_failed(77, 9001);
        let cleaned = controller
            .handle(request(2, Operation::Stop), 9002)
            .snapshot
            .unwrap();
        assert_eq!(cleaned.first_failure, Some(66));
        assert_ne!(
            controller
                .handle(request(3, Operation::StartSecond), 9003)
                .status,
            0
        );
        assert_eq!(controller.first_failure, Some(66));
    }

    #[test]
    fn partial_second_start_consumes_attempt_and_later_cleanup_keeps_failure() {
        let mut controller = Controller::new(Fake::idle());
        first_stopped(&mut controller);
        controller.radio.start_error = Some(55);
        assert_eq!(
            controller
                .handle(request(4, Operation::StartSecond), 30)
                .status,
            55
        );
        let cleaned = controller
            .handle(request(5, Operation::Stop), 40)
            .snapshot
            .unwrap();
        assert_eq!(cleaned.phase, Some(Phase::Second));
        assert_eq!(cleaned.first_failure, Some(55));
        assert_eq!(cleaned.mac, Some([48; 6]));
        assert_eq!(controller.radio.initializations, 1);
        assert_ne!(
            controller
                .handle(request(6, Operation::StartSecond), 50)
                .status,
            0
        );
        assert_eq!(controller.radio.starts, 2);
    }

    #[test]
    fn invalid_initial_framing_and_clock_overflow_never_initialize() {
        let mut controller = Controller::new(Fake::idle());
        controller.transport_failed(PROTOCOL_FAILURE, 0);
        assert_ne!(
            controller
                .handle(request(1, Operation::StartFirst), 1)
                .status,
            0
        );
        assert_eq!(controller.radio.initializations, 0);
        let mut controller = Controller::new(Fake::idle());
        assert_eq!(
            controller
                .handle(request(1, Operation::StartFirst), u64::MAX)
                .status,
            CLOCK_FAILURE
        );
        assert_eq!(controller.radio.starts, 0);
    }
}
