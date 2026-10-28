use super::*;
use crate::completion::{Kind, Mailbox, UNEXPECTED};
use crate::protocol::*;
use std::{format, vec, vec::Vec};

struct Fake {
    s: Snapshot,
    starts: Vec<Phase>,
    stops: u32,
    init: u32,
    start_error: Option<i32>,
    stop_error: Option<i32>,
    fault: Option<i32>,
    wrong_used: bool,
    deadlines: Vec<u64>,
}
impl Fake {
    fn idle() -> Self {
        Self {
            s: Snapshot {
                state: State::Idle,
                phase: None,
                nonce: None,
                initialized: false,
                advertising: false,
                data_configured: false,
                address_set_completed: false,
                prestart_readback: AddressReadback::NotRequested,
                used_readback: AddressReadback::NotRequested,
                auto_stopped: false,
                first_failure: None,
            },
            starts: vec![],
            stops: 0,
            init: 0,
            start_error: None,
            stop_error: None,
            fault: None,
            wrong_used: false,
            deadlines: vec![],
        }
    }
}
impl Radio for Fake {
    fn status(&mut self) -> Result<Snapshot, i32> {
        Ok(self.s)
    }
    fn start(&mut self, phase: Phase, nonce: [u8; 8], deadline: u64) -> Result<Snapshot, i32> {
        self.starts.push(phase);
        self.deadlines.push(deadline);
        if phase == Phase::First {
            self.init += 1;
        }
        self.s.phase = Some(phase);
        self.s.nonce = Some(nonce);
        self.s.state = State::Stopped;
        if let Some(e) = self.start_error {
            return Err(e);
        }
        let prestart = if self.s.initialized {
            self.s.used_readback
        } else {
            AddressReadback::Available {
                address: [48; 6],
                address_type: AddressType::Public,
            }
        };
        self.s.initialized = true;
        self.s.data_configured = true;
        self.s.address_set_completed = true;
        self.s.prestart_readback = prestart;
        self.s.auto_stopped = false;
        self.s.advertising = true;
        self.s.state = State::Active;
        self.s.used_readback = AddressReadback::Available {
            address: if self.wrong_used {
                SECOND_ADDRESS
            } else {
                phase.address()
            },
            address_type: AddressType::Random,
        };
        Ok(self.s)
    }
    fn stop(&mut self, _nonce: [u8; 8], auto: bool) -> Result<Snapshot, i32> {
        self.stops += 1;
        if let Some(e) = self.stop_error {
            return Err(e);
        }
        if self.s.phase.is_some() {
            self.s.state = State::Stopped;
            self.s.advertising = false;
            self.s.auto_stopped |= auto;
        }
        Ok(self.s)
    }
    fn fault(&self) -> Option<i32> {
        self.fault
    }
    fn cleanup_needed(&self) -> bool {
        self.s.advertising
    }
}
fn request(id: u32, op: Operation) -> Request {
    Request::new(id, op, [7; 8])
}
fn handle(c: &mut Controller<Fake>, id: u32, op: Operation, now: u64) -> Snapshot {
    let r = request(id, op);
    let response = c.handle(r, now);
    response.validate_for(&r).unwrap();
    response.snapshot.unwrap()
}

#[test]
fn ordered_five_commands_retain_driver_nonce_and_random_stop_state() {
    let mut c = Controller::new(Fake::idle());
    assert!(!handle(&mut c, 1, Operation::Status, 0).initialized);
    let a = handle(&mut c, 2, Operation::StartFirst, 10);
    assert!(a.used_readback.matches(Phase::First));
    let stop = handle(&mut c, 3, Operation::Stop, 20);
    assert!(!stop.advertising);
    assert!(stop.used_readback.matches(Phase::First));
    let b = handle(&mut c, 4, Operation::StartSecond, 30);
    assert!(b.used_readback.matches(Phase::Second));
    assert_eq!(b.prestart_readback, a.used_readback);
    handle(&mut c, 5, Operation::Stop, 40);
    assert_eq!(c.radio.init, 1);
    assert_eq!(c.radio.starts, [Phase::First, Phase::Second]);
    assert_eq!(c.radio.deadlines, [8010, 8030]);
    assert_eq!(nonce_ad([7; 8]), nonce_ad([7; 8]));
    assert_ne!(FIRST_ADDRESS, SECOND_ADDRESS);
    assert_eq!(FIRST_ADDRESS[0] & 0xc0, 0xc0);
}
#[test]
fn second_without_explicit_stop_or_after_failed_start_never_runs() {
    for first in [false, true] {
        let mut c = Controller::new(Fake::idle());
        if first {
            handle(&mut c, 1, Operation::StartFirst, 0);
        }
        assert_ne!(c.handle(request(2, Operation::StartSecond), 1).status, 0);
        assert!(!c.radio.starts.contains(&Phase::Second));
    }
    let mut f = Fake::idle();
    f.start_error = Some(77);
    let mut c = Controller::new(f);
    assert_eq!(c.handle(request(1, Operation::StartFirst), 0).status, 77);
    assert_eq!(c.radio.starts, [Phase::First]);
    assert_eq!(
        handle(&mut c, 2, Operation::Stop, 1).first_failure,
        Some(77)
    );
    assert_ne!(c.handle(request(3, Operation::StartSecond), 2).status, 0);
    assert_eq!(c.radio.init, 1);
}
#[test]
fn exact_deadline_auto_stop_is_independent_but_not_second_authorization() {
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 1, Operation::StartFirst, 100);
    c.poll(8099);
    assert_eq!(c.radio.stops, 0);
    c.poll(8100);
    assert_eq!(c.radio.stops, 1);
    assert!(c.radio.s.auto_stopped);
    assert_ne!(c.handle(request(2, Operation::StartSecond), 8101).status, 0);
    assert_eq!(c.radio.starts, [Phase::First]);
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 1, Operation::StartFirst, 100);
    c.poll(8100);
    handle(&mut c, 2, Operation::Stop, 8101);
    handle(&mut c, 3, Operation::StartSecond, 8102);
}
#[test]
fn failed_auto_cleanup_retries_without_uart_and_stays_failed() {
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 1, Operation::StartFirst, 0);
    c.radio.stop_error = Some(91);
    c.poll(8000);
    assert_eq!(c.first_failure, Some(91));
    assert_eq!(c.radio.stops, 1);
    c.poll(8999);
    assert_eq!(c.radio.stops, 1);
    c.radio.stop_error = None;
    c.poll(9000);
    assert_eq!(c.radio.stops, 2);
    assert_eq!(
        handle(&mut c, 2, Operation::Stop, 9001).first_failure,
        Some(91)
    );
    assert_ne!(c.handle(request(3, Operation::StartSecond), 9002).status, 0);
}
#[test]
fn wrong_actual_used_address_and_transport_error_are_terminal() {
    let mut f = Fake::idle();
    f.wrong_used = true;
    let mut c = Controller::new(f);
    assert_eq!(
        c.handle(request(1, Operation::StartFirst), 0).status,
        ADAPTER_FAILURE
    );
    assert_eq!(c.radio.starts, [Phase::First]);
    assert_eq!(c.radio.stops, 1);
    c.transport_failed(88, 1);
    assert_eq!(c.first_failure, Some(ADAPTER_FAILURE));
    assert_ne!(c.handle(request(2, Operation::StartFirst), 2).status, 0);
}
#[test]
fn stale_id_nonce_changed_and_repeated_attempts_fail_closed() {
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 4, Operation::Status, 0);
    assert_eq!(
        c.handle(request(4, Operation::StartFirst), 1).status,
        ORDER_FAILURE
    );
    assert!(c.radio.starts.is_empty());
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 1, Operation::StartFirst, 0);
    let r = Request::new(2, Operation::Stop, [8; 8]);
    assert_eq!(c.handle(r, 1).status, ORDER_FAILURE);
    assert_eq!(c.radio.starts, [Phase::First]);
    assert!(c.radio.stops > 0);
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 1, Operation::StartFirst, 0);
    handle(&mut c, 2, Operation::Stop, 1);
    handle(&mut c, 3, Operation::StartSecond, 2);
    handle(&mut c, 4, Operation::Stop, 3);
    assert_ne!(c.handle(request(5, Operation::StartSecond), 4).status, 0);
    assert_eq!(c.radio.starts.len(), 2);
}
#[test]
fn unsolicited_fault_and_late_start_trigger_cleanup_even_after_prior_failure() {
    let mut c = Controller::new(Fake::idle());
    handle(&mut c, 1, Operation::StartFirst, 0);
    c.radio.fault = Some(99);
    c.poll(10);
    assert_eq!(c.first_failure, Some(99));
    assert!(!c.radio.s.advertising);
    let stops = c.radio.stops;
    c.radio.s.advertising = true;
    c.poll(11);
    assert_eq!(c.radio.stops, stops + 1);
    assert!(!c.radio.s.advertising);
    assert_eq!(
        handle(&mut c, 2, Operation::Status, 12).first_failure,
        Some(99)
    );
}
#[test]
fn protocol_domain_shape_suffix_and_redacted_debug_controls() {
    let r = request(1, Operation::Status);
    let mut bytes = [0; MAX_REQUEST_BYTES];
    let n = encode_request(&r, &mut bytes).unwrap();
    assert_eq!(decode_request(&bytes[..n]).unwrap(), r);
    let text = core::str::from_utf8(&bytes[..n]).unwrap();
    for bad in [
        text.replace("ble_advertising", "wifi"),
        text.replace("\"version\":1", "\"version\":2"),
        text.replace("\"id\":1", "\"id\":0"),
        text.replace("\"id\":1", "\"id\":1,\"id\":2"),
        format!("{text}{{}}"),
        text.replace("\"id\":1", "\"id\":1,\"extra\":1"),
    ] {
        assert!(decode_request(bad.as_bytes()).is_err());
    }
    assert!(decode_request(&[b' '; MAX_REQUEST_BYTES + 1]).is_err());
    assert!(encode_request(&r, &mut [0; 1]).is_err());
    assert!(format!("{r:?}").contains("<redacted>"));
    assert!(!format!("{r:?}").contains("[7"));
    let mut c = Controller::new(Fake::idle());
    let s = handle(&mut c, 1, Operation::StartFirst, 0);
    let response = Response::new(1, Operation::StartFirst, 0, Some(s));
    let mut out = [0; MAX_RESPONSE_BYTES];
    let n = encode_response(&response, &mut out).unwrap();
    assert_eq!(decode_response(&out[..n]).unwrap(), response);
    assert!(
        response
            .validate_for(&request(2, Operation::StartFirst))
            .is_err()
    );
    assert!(!format!("{response:?}").contains("194"));
    let unavailable = Snapshot {
        prestart_readback: AddressReadback::Unavailable { status: 5 },
        ..s
    };
    assert!(
        Response::new(1, Operation::StartFirst, 0, Some(unavailable))
            .validate_for(&request(1, Operation::StartFirst))
            .is_ok()
    );
    let bad = Snapshot {
        used_readback: AddressReadback::Unavailable { status: 5 },
        ..s
    };
    assert!(
        encode_response(
            &Response::new(1, Operation::StartFirst, 0, Some(bad)),
            &mut out
        )
        .is_err()
    );
}
#[test]
fn mailbox_exact_completion_duplicates_errors_and_cleanup_replacement() {
    let m = Mailbox::new();
    m.arm(Kind::Data, false).unwrap();
    m.complete(Kind::Data, 0);
    assert_eq!(m.take(Kind::Address), None);
    assert_eq!(m.take(Kind::Data), Some(0));
    assert_eq!(m.take(Kind::Data), None);
    m.arm(Kind::Address, false).unwrap();
    m.arm(Kind::Stop, true).unwrap();
    m.complete(Kind::Address, 0);
    assert_eq!(m.take(Kind::Stop), None);
    assert_eq!(m.fault(), Some(UNEXPECTED));
    m.complete(Kind::Stop, 0);
    assert_eq!(m.take(Kind::Stop), Some(0));
    assert_eq!(m.advertising(), Some(false));
    assert!(m.arm(Kind::Start, false).is_err());
    let m = Mailbox::new();
    m.arm(Kind::Start, false).unwrap();
    assert_eq!(m.advertising(), None);
    m.complete(Kind::Start, 7);
    assert_eq!(m.take(Kind::Start), Some(7));
    m.unexpected(8);
    assert_eq!(m.fault(), Some(7));
    m.arm(Kind::Stop, true).unwrap();
    m.complete(Kind::Stop, 0);
    m.complete(Kind::Stop, 0);
    assert_eq!(m.fault(), Some(7));
    assert_eq!(m.take(Kind::Stop), Some(0));
}

#[test]
fn hardware_boundary_requires_a_selected_advertising_start() {
    for operation in [Operation::Status, Operation::Stop] {
        assert_eq!(check_advertise_operation(operation), Err(ADAPTER_FAILURE));
    }
    for operation in [Operation::StartFirst, Operation::StartSecond] {
        assert_eq!(check_advertise_operation(operation), Ok(()));
    }
}

#[test]
fn same_kind_events_have_no_generation_and_failed_retry_cannot_admit() {
    let mailbox = Mailbox::new();
    mailbox.arm(Kind::Stop, true).unwrap();
    mailbox.complete(Kind::Stop, 0);
    assert_eq!(mailbox.take(Kind::Stop), Some(0));
    mailbox.arm(Kind::Stop, true).unwrap();
    // The callback ABI cannot identify this as an old duplicate. This is an
    // explicit trust-boundary control, not a claim of duplicate suppression.
    mailbox.complete(Kind::Stop, 0);
    assert_eq!(mailbox.take(Kind::Stop), Some(0));
    assert_eq!(mailbox.fault(), None);
    let mut controller = Controller::new(Fake::idle());
    handle(&mut controller, 1, Operation::StartFirst, 0);
    controller.transport_failed(123, 1);
    let snapshot = handle(&mut controller, 2, Operation::Stop, 2);
    assert_eq!(snapshot.first_failure, Some(123));
    assert_ne!(
        controller
            .handle(request(3, Operation::StartSecond), 3)
            .status,
        0
    );
    assert_eq!(controller.radio.starts, [Phase::First]);
}
