//! Pure boot-local command/deadline control, tested without the radio SDK.

use wifi_protocol::{AP_WINDOW_MILLIS, Operation, Request, Response, Snapshot};

pub(crate) const PROTOCOL_FAILURE: i32 = -1001;
pub(crate) const ORDER_FAILURE: i32 = -1002;
pub(crate) const ADAPTER_FAILURE: i32 = -1003;
pub(crate) const CLOCK_FAILURE: i32 = -1004;

pub(crate) trait Radio {
    fn status(&mut self) -> Result<Snapshot, i32>;
    fn start(&mut self, nonce: [u8; 8]) -> Result<Snapshot, i32>;
    fn stop(&mut self, nonce: [u8; 8], automatic: bool) -> Result<Snapshot, i32>;
}

pub(crate) struct Controller<R> {
    radio: R,
    nonce: Option<[u8; 8]>,
    deadline_ms: Option<u64>,
    pending_error: Option<i32>,
}

fn failure(code: i32) -> i32 {
    if code == 0 { ADAPTER_FAILURE } else { code }
}

impl<R: Radio> Controller<R> {
    pub(crate) fn new(radio: R) -> Self {
        Self {
            radio,
            nonce: None,
            deadline_ms: None,
            pending_error: None,
        }
    }

    /// Deadline checks run even when no command or usable UART reply arrives.
    /// This is a software deadline before SDK calls, not an RF duration bound.
    pub(crate) fn poll(&mut self, now_ms: u64) {
        if self.deadline_ms.is_some_and(|deadline| now_ms >= deadline) {
            self.deadline_ms = None;
            if let Some(nonce) = self.nonce {
                let result = self
                    .radio
                    .stop(nonce, true)
                    .and_then(|snapshot| self.check_snapshot(Operation::Stop, nonce, snapshot));
                if let Err(code) = result {
                    self.pending_error.get_or_insert(failure(code));
                    // No UART activity is required to retry failed cleanup.
                    // Retain the first failure for the eventual transcript.
                    self.deadline_ms = Some(now_ms.saturating_add(1000));
                }
            }
        }
    }

    fn check_snapshot(
        &self,
        operation: Operation,
        nonce: [u8; 8],
        snapshot: Snapshot,
    ) -> Result<Snapshot, i32> {
        if self.nonce.is_some() && snapshot.nonce != self.nonce {
            return Err(ADAPTER_FAILURE);
        }
        Response::new(1, operation, 0, Some(snapshot))
            .validate_for(&Request::new(1, operation, nonce))
            .map_err(|_| ADAPTER_FAILURE)?;
        Ok(snapshot)
    }

    pub(crate) fn handle(&mut self, request: Request, now_ms: u64) -> Response {
        self.poll(now_ms);
        if request.validate().is_err() {
            return Response::new(request.id.max(1), request.operation, PROTOCOL_FAILURE, None);
        }
        if self.nonce.is_some_and(|nonce| nonce != request.nonce) {
            return Response::new(request.id, request.operation, ORDER_FAILURE, None);
        }
        let result = match request.operation {
            Operation::Status => match self.pending_error.take() {
                Some(code) => Err(code),
                None => self.radio.status(),
            },
            Operation::Start => {
                if self.nonce.is_some() {
                    return Response::new(request.id, request.operation, ORDER_FAILURE, None);
                }
                let Some(deadline) = now_ms.checked_add(u64::from(AP_WINDOW_MILLIS)) else {
                    return Response::new(request.id, request.operation, CLOCK_FAILURE, None);
                };
                self.nonce = Some(request.nonce);
                self.deadline_ms = Some(deadline);
                let result = self.radio.start(request.nonce).and_then(|snapshot| {
                    self.check_snapshot(Operation::Start, request.nonce, snapshot)
                });
                if result.is_err() {
                    let cleanup = self.radio.stop(request.nonce, false).and_then(|snapshot| {
                        self.check_snapshot(Operation::Stop, request.nonce, snapshot)
                    });
                    if cleanup.is_ok() {
                        self.deadline_ms = None;
                    }
                }
                result
            }
            Operation::Stop => {
                let result = self.radio.stop(request.nonce, false).and_then(|snapshot| {
                    self.check_snapshot(Operation::Stop, request.nonce, snapshot)
                });
                if result.is_ok() {
                    self.deadline_ms = None;
                }
                match self.pending_error.take() {
                    Some(code) => Err(code),
                    None => result,
                }
            }
        };
        let result = result
            .and_then(|snapshot| self.check_snapshot(request.operation, request.nonce, snapshot));
        match result {
            Ok(snapshot) => Response::new(request.id, request.operation, 0, Some(snapshot)),
            Err(code) => Response::new(request.id, request.operation, failure(code), None),
        }
    }

    /// Stop promptly after transport failure while retaining the deadline when
    /// that cleanup fails. A later command reports the failure before admission.
    pub(crate) fn transport_failed(&mut self, code: i32) {
        if let Some(nonce) = self.nonce {
            let result = self
                .radio
                .stop(nonce, false)
                .and_then(|snapshot| self.check_snapshot(Operation::Stop, nonce, snapshot));
            match result {
                Ok(_) => {
                    self.deadline_ms = None;
                    self.pending_error.get_or_insert(failure(code));
                }
                Err(stop_error) => {
                    self.pending_error.get_or_insert(failure(stop_error));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wifi_protocol::{AP_MAC, State};

    struct Fake {
        snapshot: Snapshot,
        starts: u32,
        stops: u32,
        start_error: Option<i32>,
        stop_error: Option<i32>,
        invalid_stop_snapshot: bool,
    }

    impl Fake {
        fn idle() -> Self {
            Self {
                snapshot: Snapshot {
                    state: State::Idle,
                    nonce: None,
                    initialized: false,
                    running: false,
                    mac: None,
                    original_mac: None,
                    auto_stopped: false,
                },
                starts: 0,
                stops: 0,
                start_error: None,
                stop_error: None,
                invalid_stop_snapshot: false,
            }
        }
    }

    impl Radio for Fake {
        fn status(&mut self) -> Result<Snapshot, i32> {
            Ok(self.snapshot)
        }
        fn start(&mut self, nonce: [u8; 8]) -> Result<Snapshot, i32> {
            self.starts += 1;
            self.snapshot = Snapshot {
                state: State::Active,
                nonce: Some(nonce),
                initialized: true,
                running: true,
                mac: Some(AP_MAC),
                original_mac: Some([48; 6]),
                auto_stopped: false,
            };
            self.start_error.map_or(Ok(self.snapshot), Err)
        }
        fn stop(&mut self, _nonce: [u8; 8], automatic: bool) -> Result<Snapshot, i32> {
            self.stops += 1;
            if let Some(error) = self.stop_error {
                return Err(error);
            }
            if self.snapshot.state != State::Idle {
                self.snapshot.state = State::Stopped;
                self.snapshot.running = false;
                self.snapshot.mac = self.snapshot.original_mac;
                self.snapshot.auto_stopped |= automatic;
            }
            if self.invalid_stop_snapshot {
                self.snapshot.running = true;
            }
            Ok(self.snapshot)
        }
    }

    fn request(operation: Operation) -> Request {
        Request::new(1, operation, [7; 8])
    }

    #[test]
    fn idle_single_start_nonce_binding_stop_and_no_restart() {
        let mut controller = Controller::new(Fake::idle());
        let idle = controller.handle(request(Operation::Status), 0);
        assert_eq!(idle.snapshot.unwrap().state, State::Idle);
        assert_eq!(controller.radio.starts, 0);
        assert_eq!(controller.handle(request(Operation::Start), 0).status, 0);
        assert_ne!(
            controller
                .handle(Request::new(2, Operation::Status, [8; 8]), 1)
                .status,
            0
        );
        assert_ne!(controller.handle(request(Operation::Start), 1).status, 0);
        for _ in 0..2 {
            assert_eq!(controller.handle(request(Operation::Stop), 2).status, 0);
        }
        assert_ne!(controller.handle(request(Operation::Start), 3).status, 0);
        assert_eq!(controller.radio.starts, 1);
    }

    #[test]
    fn invalid_request_and_clock_overflow_never_start() {
        let mut controller = Controller::new(Fake::idle());
        for request in [
            Request::new(0, Operation::Start, [7; 8]),
            Request::new(1, Operation::Start, [0; 8]),
        ] {
            assert_ne!(controller.handle(request, 0).status, 0);
        }
        assert_eq!(
            controller
                .handle(request(Operation::Start), u64::MAX)
                .status,
            CLOCK_FAILURE
        );
        assert_eq!(controller.radio.starts, 0);
    }

    #[test]
    fn deadline_stops_without_uart_and_stop_after_deadline_is_idempotent() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(Operation::Start), 10);
        controller.poll(8009);
        assert_eq!(controller.radio.stops, 0);
        controller.poll(8010);
        controller.poll(8011);
        assert_eq!(controller.radio.stops, 1);
        let stopped = controller
            .handle(request(Operation::Stop), 8020)
            .snapshot
            .unwrap();
        assert!(!stopped.running && stopped.auto_stopped);
        assert_eq!(stopped.mac, stopped.original_mac);
    }

    #[test]
    fn partial_start_failure_runs_cleanup_and_remains_nonrestartable() {
        let mut radio = Fake::idle();
        radio.start_error = Some(55);
        let mut controller = Controller::new(radio);
        assert_eq!(controller.handle(request(Operation::Start), 0).status, 55);
        assert_eq!(controller.radio.stops, 1);
        assert_eq!(controller.radio.snapshot.state, State::Stopped);
        assert_ne!(controller.handle(request(Operation::Start), 1).status, 0);
    }

    #[test]
    fn failed_auto_stop_is_reported_and_cleanup_can_be_retried() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(Operation::Start), 0);
        controller.radio.stop_error = Some(66);
        controller.poll(8000);
        controller.radio.stop_error = None;
        assert_eq!(controller.handle(request(Operation::Stop), 8001).status, 66);
        assert_eq!(
            controller
                .handle(request(Operation::Status), 8002)
                .snapshot
                .unwrap()
                .state,
            State::Stopped
        );
    }

    #[test]
    fn invalid_cleanup_readback_keeps_the_autonomous_deadline() {
        for partial_start in [false, true] {
            let mut radio = Fake::idle();
            radio.invalid_stop_snapshot = true;
            radio.start_error = partial_start.then_some(55);
            let mut controller = Controller::new(radio);
            let start = controller.handle(request(Operation::Start), 0);
            assert_eq!(start.status, if partial_start { 55 } else { 0 });
            if !partial_start {
                assert_eq!(
                    controller.handle(request(Operation::Stop), 1).status,
                    ADAPTER_FAILURE
                );
            }
            controller.radio.invalid_stop_snapshot = false;
            let previous_stops = controller.radio.stops;
            controller.poll(8000);
            assert_eq!(controller.radio.stops, previous_stops + 1);
            assert!(!controller.radio.snapshot.running);
        }
    }

    #[test]
    fn failed_auto_stop_retries_without_uart_and_retains_first_failure() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(Operation::Start), 0);
        controller.radio.stop_error = Some(66);
        controller.poll(8000);
        assert_eq!(controller.radio.stops, 1);
        controller.radio.stop_error = None;
        controller.poll(8999);
        assert_eq!(controller.radio.stops, 1);
        controller.poll(9000);
        assert_eq!(controller.radio.stops, 2);
        assert_eq!(controller.radio.snapshot.state, State::Stopped);
        assert_eq!(
            controller.handle(request(Operation::Status), 9001).status,
            66
        );
    }

    #[test]
    fn transport_error_does_not_erase_an_unreported_auto_stop_failure() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(Operation::Start), 0);
        controller.radio.stop_error = Some(66);
        controller.poll(8000);
        controller.radio.stop_error = None;
        controller.transport_failed(77);
        assert_eq!(
            controller.handle(request(Operation::Status), 8001).status,
            66
        );
        assert!(!controller.radio.snapshot.running);
    }

    #[test]
    fn transport_failure_stops_and_retains_the_error() {
        let mut controller = Controller::new(Fake::idle());
        controller.handle(request(Operation::Start), 0);
        controller.transport_failed(77);
        assert_eq!(controller.radio.snapshot.state, State::Stopped);
        assert_eq!(controller.handle(request(Operation::Status), 1).status, 77);
        assert_eq!(controller.handle(request(Operation::Stop), 2).status, 0);
    }
}
