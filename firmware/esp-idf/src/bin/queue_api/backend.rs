//! Two bounded task-context backends with the same scalar-message contract.

use std::io;

#[cfg(not(feature = "native-queue"))]
mod selected {
    use std::sync::mpsc;

    use super::io;

    pub(crate) const NAME: &str = "rust-std-sync-channel";

    pub(crate) struct Sender(mpsc::SyncSender<u32>);
    pub(crate) struct Receiver(mpsc::Receiver<u32>);

    pub(crate) fn channel() -> io::Result<(Sender, Receiver)> {
        let (tx, rx) = mpsc::sync_channel(1);
        Ok((Sender(tx), Receiver(rx)))
    }

    impl Sender {
        pub(crate) fn send(&self, value: u32) -> io::Result<()> {
            self.0
                .send(value)
                .map_err(|_| io::Error::other("std channel send disconnected"))
        }

        pub(crate) fn try_send(&self, value: u32) -> io::Result<bool> {
            match self.0.try_send(value) {
                Ok(()) => Ok(true),
                Err(mpsc::TrySendError::Full(_)) => Ok(false),
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    Err(io::Error::other("std channel try_send disconnected"))
                }
            }
        }
    }

    impl Receiver {
        pub(crate) fn recv(&self) -> io::Result<u32> {
            self.0
                .recv()
                .map_err(|_| io::Error::other("std channel receive disconnected"))
        }

        pub(crate) fn try_recv(&self) -> io::Result<Option<u32>> {
            match self.0.try_recv() {
                Ok(value) => Ok(Some(value)),
                Err(mpsc::TryRecvError::Empty) => Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => {
                    Err(io::Error::other("std channel try_recv disconnected"))
                }
            }
        }
    }

    pub(crate) fn queues_empty(_tx: &Sender, _rx: &Receiver) -> io::Result<bool> {
        // std has no non-destructive public occupancy query. The single-inflight
        // protocol and phase/state audit remain the evidence for this variant.
        Ok(true)
    }
}

#[cfg(feature = "native-queue")]
mod selected {
    use std::ptr::NonNull;
    use std::sync::Arc;

    use super::io;

    pub(crate) const NAME: &str = "freertos-queue";
    // IDF 5.5.6 FreeRTOS queue.h: queueQUEUE_TYPE_BASE / queueSEND_TO_BACK.
    const QUEUE_TYPE_BASE: u8 = 0;
    const SEND_TO_BACK: esp_idf_sys::BaseType_t = 0;
    // portmacro.h uses 32-bit ticks; INCLUDE_vTaskSuspend=1 makes this an
    // indefinite task-context wait. An unresponsive backend needs host reset.
    const WAIT_FOREVER: esp_idf_sys::TickType_t = u32::MAX;

    struct Queue(NonNull<esp_idf_sys::QueueDefinition>);

    // SAFETY: These are ordinary FreeRTOS queues, whose task-context send,
    // receive and occupancy APIs synchronize access internally. Only aligned
    // owned u32 values cross tasks, copied by the kernel. No Rust references
    // into queue storage exist. Arc keeps the queue alive through every
    // borrowed call; its final owner deletes it only after all calls finish.
    // No ISR APIs, queue reset, callback, cancellation or raw handle is exposed.
    unsafe impl Send for Queue {}
    // SAFETY: The same task-safe FreeRTOS synchronization and Arc lifetime
    // invariant permit concurrent shared calls without unsynchronized memory.
    unsafe impl Sync for Queue {}

    pub(crate) struct Sender(Arc<Queue>);
    pub(crate) struct Receiver(Arc<Queue>);

    pub(crate) fn channel() -> io::Result<(Sender, Receiver)> {
        // SAFETY: Nonzero capacity one, item size exactly size_of::<u32>(), and
        // the pinned base-queue tag request ordinary dynamically allocated
        // copied-item storage. Null is checked before any use.
        let handle = unsafe {
            esp_idf_sys::xQueueGenericCreate(1, std::mem::size_of::<u32>() as u32, QUEUE_TYPE_BASE)
        };
        let queue =
            Arc::new(Queue(NonNull::new(handle).ok_or_else(|| {
                io::Error::other("FreeRTOS queue allocation failed")
            })?));
        Ok((Sender(Arc::clone(&queue)), Receiver(queue)))
    }

    impl Queue {
        fn send(&self, value: u32, wait: esp_idf_sys::TickType_t) -> io::Result<bool> {
            // SAFETY: Arc ownership keeps this valid queue alive; value is a
            // naturally aligned initialized u32 whose borrowed storage remains
            // live throughout the call, including a wait. SDK copies four
            // bytes and retains no caller pointer after the call returns.
            let status = unsafe {
                esp_idf_sys::xQueueGenericSend(
                    self.0.as_ptr(),
                    std::ptr::from_ref(&value).cast(),
                    wait,
                    SEND_TO_BACK,
                )
            };
            match status {
                1 => Ok(true),
                0 => Ok(false),
                other => Err(io::Error::other(format!(
                    "FreeRTOS queue send returned unexpected status {other}"
                ))),
            }
        }

        fn recv(&self, wait: esp_idf_sys::TickType_t) -> io::Result<Option<u32>> {
            let mut value = 0_u32;
            // SAFETY: The live queue's configured item is exactly four bytes;
            // value is aligned initialized writable storage retained for the
            // entire blocking call. Kernel retains no output pointer afterward.
            let status = unsafe {
                esp_idf_sys::xQueueReceive(
                    self.0.as_ptr(),
                    std::ptr::from_mut(&mut value).cast(),
                    wait,
                )
            };
            match status {
                1 => Ok(Some(value)),
                0 => Ok(None),
                other => Err(io::Error::other(format!(
                    "FreeRTOS queue receive returned unexpected status {other}"
                ))),
            }
        }

        fn is_empty(&self) -> bool {
            // SAFETY: This is a live ordinary queue; the task-safe query retains
            // no pointers and changes neither queue contents nor ownership.
            unsafe { esp_idf_sys::uxQueueMessagesWaiting(self.0.as_ptr()) == 0 }
        }
    }

    impl Sender {
        pub(crate) fn send(&self, value: u32) -> io::Result<()> {
            if self.0.send(value, WAIT_FOREVER)? {
                Ok(())
            } else {
                Err(io::Error::other("FreeRTOS indefinite queue send failed"))
            }
        }

        pub(crate) fn try_send(&self, value: u32) -> io::Result<bool> {
            self.0.send(value, 0)
        }
    }

    impl Receiver {
        pub(crate) fn recv(&self) -> io::Result<u32> {
            self.0
                .recv(WAIT_FOREVER)?
                .ok_or_else(|| io::Error::other("FreeRTOS indefinite queue receive failed"))
        }

        pub(crate) fn try_recv(&self) -> io::Result<Option<u32>> {
            self.0.recv(0)
        }
    }

    pub(crate) fn queues_empty(tx: &Sender, rx: &Receiver) -> io::Result<bool> {
        Ok(tx.0.is_empty() && rx.0.is_empty())
    }

    impl Drop for Queue {
        fn drop(&mut self) {
            // SAFETY: This is the final Arc owner. Every operation borrows an
            // owner, so none can remain blocked or access the queue now. The
            // sole queue allocation is deleted once, with no active waiters.
            // vQueueDelete releases storage without waiting for another task.
            unsafe { esp_idf_sys::vQueueDelete(self.0.as_ptr()) }
        }
    }
}

pub(crate) use selected::{NAME, Receiver, Sender, channel, queues_empty};

/// Checks full/empty behavior on a separate object, then explicitly destroys it.
pub(crate) fn qualify() -> io::Result<()> {
    let (tx, rx) = channel()?;
    if rx.try_recv()?.is_some()
        || !tx.try_send(0x1234_5678)?
        || tx.try_send(0x8765_4321)?
        || rx.try_recv()? != Some(0x1234_5678)
        || rx.try_recv()?.is_some()
    {
        return Err(io::Error::other(
            "backend empty/full correctness probe failed",
        ));
    }
    drop(tx);
    drop(rx);
    Ok(())
}
