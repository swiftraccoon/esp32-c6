//! Allocation-free completion mailbox shared with the target callback.
//!
//! Only the main task arms/takes results. A callback publishes one matching
//! completion; mismatched, unsolicited and error events latch permanently.
//! SDK callbacks contain no request ID: a same-kind delayed duplicate is
//! indistinguishable from a new completion. Normal sequencing therefore relies
//! on the pinned SDK's ordered, exactly-once event path. Failed cleanup retries
//! remain permanently failed regardless of which queued Stop event is observed.
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

/// Static unexpected-completion failure.
pub const UNEXPECTED: i32 = -3101;
/// Only asynchronous operations issued by this firmware.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Kind {
    /// Raw data configuration.
    Data = 1,
    /// Static random address setter.
    Address = 2,
    /// Advertising start.
    Start = 3,
    /// Advertising stop.
    Stop = 4,
}
/// A static atomics-only mailbox; no callback pointer or payload is retained.
pub struct Mailbox {
    pending: AtomicU32,
    result: AtomicI32,
    fault: AtomicI32,
    activity: AtomicU32,
}
impl core::fmt::Debug for Mailbox {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Mailbox { <redacted> }")
    }
}
impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}
impl Mailbox {
    /// Create a stopped, empty mailbox before Bluetooth initialization.
    pub const fn new() -> Self {
        Self {
            pending: AtomicU32::new(0),
            result: AtomicI32::new(0),
            fault: AtomicI32::new(0),
            activity: AtomicU32::new(0),
        }
    }
    /// Arm one operation before its SDK call. Cleanup can replace a timed-out
    /// pending operation; a late different completion cannot satisfy that Stop.
    ///
    /// # Errors
    /// Returns [`UNEXPECTED`] for a busy/publishing slot or forbidden rearm.
    pub fn arm(&self, kind: Kind, cleanup: bool) -> Result<(), i32> {
        let old = self.pending.load(Ordering::Acquire);
        // A publishing callback owns the slot until its result is released.
        // Cleanup may replace a timed-out pending slot, never a publishing one.
        if old & 0x200 != 0 || !cleanup && (self.fault().is_some() || old != 0) {
            return Err(UNEXPECTED);
        }
        self.pending
            .compare_exchange(old, kind as u32, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| UNEXPECTED)?;
        if matches!(kind, Kind::Start | Kind::Stop) {
            self.activity.store(2, Ordering::Release);
        }
        Ok(())
    }
    /// Callback-only publication; release/acquire orders result and activity
    /// before the completed state. No allocation, blocking, formatting or panics.
    pub fn complete(&self, kind: Kind, status: i32) {
        if matches!(kind, Kind::Start | Kind::Stop) {
            self.activity.store(
                if status != 0 {
                    2
                } else if kind == Kind::Start {
                    1
                } else {
                    0
                },
                Ordering::Release,
            );
        }
        if status != 0 {
            self.unexpected(status);
        }
        if self
            .pending
            .compare_exchange(
                kind as u32,
                0x200 | kind as u32,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            self.unexpected(UNEXPECTED);
            return;
        }
        self.result.store(status, Ordering::Relaxed);
        self.pending.store(0x100 | kind as u32, Ordering::Release);
    }
    /// Latch an unknown/null/error event without replacing the first failure.
    pub fn unexpected(&self, status: i32) {
        let code = if status == 0 { UNEXPECTED } else { status };
        let _ = self
            .fault
            .compare_exchange(0, code, Ordering::AcqRel, Ordering::Acquire);
    }
    /// Main-task-only one-time completion retrieval.
    pub fn take(&self, kind: Kind) -> Option<i32> {
        if self
            .pending
            .compare_exchange(0x100 | kind as u32, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            Some(self.result.load(Ordering::Acquire))
        } else {
            None
        }
    }

    /// First asynchronous failure, retained permanently.
    pub fn fault(&self) -> Option<i32> {
        match self.fault.load(Ordering::Acquire) {
            0 => None,
            n => Some(n),
        }
    }
    /// Event-backed state: None means an operation is unresolved/failed.
    pub fn advertising(&self) -> Option<bool> {
        match self.activity.load(Ordering::Acquire) {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
    /// Whether any asynchronous operation remains pending.
    pub fn pending(&self) -> bool {
        self.pending.load(Ordering::Acquire) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publishing_callback_cannot_be_rearmed_or_mistaken_for_cleanup() {
        let mailbox = Mailbox::new();
        mailbox
            .pending
            .store(0x200 | Kind::Address as u32, Ordering::Release);
        assert_eq!(mailbox.arm(Kind::Stop, true), Err(UNEXPECTED));
        assert_eq!(mailbox.take(Kind::Stop), None);
        mailbox.result.store(0, Ordering::Relaxed);
        mailbox
            .pending
            .store(0x100 | Kind::Address as u32, Ordering::Release);
        assert_eq!(mailbox.take(Kind::Stop), None);
        assert_eq!(mailbox.take(Kind::Address), Some(0));
    }
}
