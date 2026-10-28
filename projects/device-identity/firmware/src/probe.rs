//! Exact boot-local controller, exercised on the host with injected operations.

use runtime_protocol::{Phase, Request, Response, Snapshot};

pub(crate) const PHASE_FAILURE: i32 = -2;

pub(crate) struct Probe {
    saved: Option<Snapshot>,
    next: usize,
    failed: bool,
}

impl Probe {
    pub(crate) fn new() -> Self {
        Self {
            saved: None,
            next: 0,
            failed: false,
        }
    }

    pub(crate) fn handle<F>(&mut self, request: Request, mut operation: F) -> Response
    where
        F: FnMut(Phase, Option<&Snapshot>) -> Result<Snapshot, i32>,
    {
        let phase = request.phase;
        if request.validate().is_err() {
            return Response::new(request.id, phase, -1, None);
        }
        // Restoration is idempotent whenever a baseline exists. A partial
        // mutation or lost response must not prevent the host's cleanup attempt.
        let restoration = phase == Phase::Restored && self.saved.is_some();
        if !restoration && (self.failed || Phase::ALL.get(self.next) != Some(&phase)) {
            return Response::new(request.id, phase, PHASE_FAILURE, None);
        }
        match operation(phase, self.saved.as_ref()) {
            Ok(snapshot) => {
                if phase == Phase::Defaults {
                    self.saved = Some(snapshot);
                }
                self.next = if phase == Phase::Restored {
                    Phase::ALL.len()
                } else {
                    self.next + 1
                };
                Response::new(request.id, phase, 0, Some(snapshot))
            }
            Err(code) => {
                self.failed = true;
                // Err(0) cannot become an inconsistent apparent success even
                // if an operation adapter violates its error-code contract.
                Response::new(request.id, phase, if code == 0 { -3 } else { code }, None)
            }
        }
    }
}
