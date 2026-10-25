//! Fixed slots shared by SDK callbacks and the serial task. No allocation,
//! waiting, or large by-value packet copies on callback stacks.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(crate) struct Buffer<T: Copy, const N: usize> {
    slots: UnsafeCell<[T; N]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    writer: AtomicBool,
    reader: AtomicBool,
}

// SAFETY: writer/reader gates enforce at most one producer/consumer at a time.
// The head/tail acquire-release protocol keeps their slots disjoint, and a
// slot is published only after initialization. Values cannot escape by reference
// through the higher-ranked closure calls. T is Copy (no destructor) and Send.
unsafe impl<T: Copy + Send, const N: usize> Sync for Buffer<T, N> {}

struct Gate<'a>(&'a AtomicBool);
impl Drop for Gate<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl<T: Copy, const N: usize> Buffer<T, N> {
    pub(crate) const fn new(initial: T) -> Self {
        assert!(N.is_power_of_two());
        Self {
            slots: UnsafeCell::new([initial; N]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            writer: AtomicBool::new(false),
            reader: AtomicBool::new(false),
        }
    }

    // Producers never wait: contention or a full buffer returns false. The
    // caller must count the rejected report. A panicking closure publishes none.
    #[inline(always)]
    pub(crate) fn push_with(&self, fill: impl FnOnce(&mut T)) -> bool {
        if self
            .writer
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return false;
        }
        let _gate = Gate(&self.writer);
        let head = self.head.load(Ordering::Relaxed);
        if head.wrapping_sub(self.tail.load(Ordering::Acquire)) == N {
            return false;
        }
        // SAFETY: gate gives exclusive producer access. This slot is outside
        // the published [tail, head) range; the consumer cannot access it.
        fill(unsafe { &mut *std::ptr::addr_of_mut!((*self.slots.get())[head & (N - 1)]) });
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    // Consumer closure can fail normally; the consumed slot is then released.
    // On panic it stays published and can be retried, without aliasing.
    pub(crate) fn pop_with<R>(&self, consume: impl FnOnce(&T) -> R) -> Option<R> {
        if self
            .reader
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }
        let _gate = Gate(&self.reader);
        let tail = self.tail.load(Ordering::Relaxed);
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        // SAFETY: gate gives exclusive consumer access. Acquire observed the
        // initialized slot, which the producer cannot reuse until tail release.
        let result = consume(unsafe { &*std::ptr::addr_of!((*self.slots.get())[tail & (N - 1)]) });
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_fifo_and_counter_wrap() {
        let buffer = Buffer::<u32, 4>::new(0);
        buffer.head.store(usize::MAX - 1, Ordering::Relaxed);
        buffer.tail.store(usize::MAX - 1, Ordering::Relaxed);
        for n in 0..4 {
            assert!(buffer.push_with(|slot| *slot = n));
        }
        assert!(!buffer.push_with(|_| panic!("must not overwrite full slot")));
        for n in 0..4 {
            assert_eq!(buffer.pop_with(|slot| *slot), Some(n));
        }
        assert_eq!(buffer.pop_with(|slot| *slot), None);
    }

    #[test]
    fn producer_consumer_publication_and_contention() {
        let buffer = Buffer::<[u32; 32], 8>::new([0; 32]);
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for n in 0..20_000 {
                    while !buffer.push_with(|slot| slot.fill(n)) {
                        std::thread::yield_now();
                    }
                }
            });
            for n in 0..20_000 {
                loop {
                    if let Some(value) = buffer.pop_with(|slot| *slot) {
                        assert_eq!(value, [n; 32]);
                        break;
                    }
                    std::thread::yield_now();
                }
            }
        });
        assert!(buffer.push_with(|slot| {
            assert!(!buffer.push_with(|_| panic!("recursive writer")));
            slot.fill(123);
        }));
        assert_eq!(buffer.pop_with(|slot| *slot), Some([123; 32]));
    }
}
