#[path = "/Users/builduser/Git/esp32-c6/projects/wireless-explorer/firmware/src/buffer.rs"]
mod buffer;

use buffer::Buffer;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, Mutex};

#[test]
fn invalid_capacities_reject() {
    assert!(catch_unwind(|| Buffer::<u32, 0>::new(0)).is_err());
    assert!(catch_unwind(|| Buffer::<u32, 3>::new(0)).is_err());
}

#[test]
fn one_slot_holds_reader_and_rejects_nested_operations() {
    let queue = Buffer::<[u32; 1024], 1>::new([0; 1024]);
    assert!(queue.push_with(|slot| slot.fill(7)));
    assert_eq!(queue.pop_with(|slot| {
        assert!(!queue.push_with(|_| panic!("overwrote borrowed slot")));
        assert!(queue.pop_with(|_| panic!("nested reader")).is_none());
        *slot
    }), Some([7; 1024]));
    assert!(queue.push_with(|slot| slot.fill(8)));
    assert_eq!(queue.pop_with(|slot| *slot), Some([8; 1024]));
}

#[test]
fn unpublished_writer_cannot_be_observed_and_writer_contention_returns() {
    let queue = Buffer::<[u32; 64], 2>::new([0; 64]);
    let entered = Barrier::new(2);
    let release = Barrier::new(2);
    std::thread::scope(|scope| {
        scope.spawn(|| assert!(queue.push_with(|slot| {
            slot.fill(42);
            entered.wait();
            release.wait();
        })));
        entered.wait();
        assert!(queue.pop_with(|_| panic!("read unpublished data")).is_none());
        assert!(!queue.push_with(|_| panic!("second concurrent writer")));
        release.wait();
    });
    assert_eq!(queue.pop_with(|slot| *slot), Some([42; 64]));
}

#[test]
fn reader_blocks_reuse_until_closure_returns() {
    let queue = Buffer::<[u64; 64], 1>::new([0; 64]);
    assert!(queue.push_with(|slot| slot.fill(17)));
    let entered = Barrier::new(2);
    let release = Barrier::new(2);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            assert_eq!(queue.pop_with(|slot| {
                entered.wait();
                release.wait();
                *slot
            }), Some([17; 64]));
        });
        entered.wait();
        for _ in 0..1000 {
            assert!(!queue.push_with(|_| panic!("reused live slot")));
            assert!(queue.pop_with(|_| panic!("second concurrent reader")).is_none());
        }
        release.wait();
    });
    assert!(queue.push_with(|slot| slot.fill(23)));
    assert_eq!(queue.pop_with(|slot| *slot), Some([23; 64]));
}

#[test]
fn panics_release_gates_without_publishing_or_removing() {
    let queue = Buffer::<[u32; 8], 2>::new([0; 8]);
    assert!(catch_unwind(AssertUnwindSafe(|| queue.push_with(|slot| {
        slot.fill(91);
        panic!("deliberate producer failure");
    }))).is_err());
    assert!(queue.pop_with(|_| panic!("published failed producer")).is_none());
    assert!(queue.push_with(|slot| slot.fill(92)));
    assert!(catch_unwind(AssertUnwindSafe(|| queue.pop_with(|slot| {
        assert_eq!(*slot, [92; 8]);
        panic!("deliberate consumer failure");
    }))).is_err());
    assert_eq!(queue.pop_with(|slot| *slot), Some([92; 8]));
    assert!(queue.push_with(|slot| slot.fill(93)));
    assert_eq!(queue.pop_with(|slot| *slot), Some([93; 8]));
}

#[test]
fn four_producers_three_consumers_no_torn_duplicate_or_missing_publications() {
    const EACH: usize = 10000;
    const TOTAL: usize = 4 * EACH;
    let queue = Buffer::<[usize; 32], 8>::new([usize::MAX; 32]);
    let done = AtomicUsize::new(0);
    let seen = Mutex::new(vec![0_u8; TOTAL]);
    std::thread::scope(|scope| {
        for producer in 0..4 {
            let queue = &queue;
            let done = &done;
            scope.spawn(move || {
                for value in producer * EACH..(producer + 1) * EACH {
                    while !queue.push_with(|slot| slot.fill(value)) {
                        std::thread::yield_now();
                    }
                }
                done.fetch_add(1, Ordering::Release);
            });
        }
        for _ in 0..3 {
            let queue = &queue;
            let done = &done;
            let seen = &seen;
            scope.spawn(move || loop {
                if let Some(packet) = queue.pop_with(|slot| *slot) {
                    let value = packet[0];
                    assert!(value < TOTAL);
                    assert!(packet.iter().all(|part| *part == value));
                    let mut seen = seen.lock().unwrap();
                    seen[value] += 1;
                    assert_eq!(seen[value], 1, "duplicate publication");
                } else if done.load(Ordering::Acquire) == 4 {
                    // A rival consumer may hold the reader gate, but no producer
                    // can add more work. The remaining reader drains any tail.
                    break;
                } else {
                    std::thread::yield_now();
                }
            });
        }
    });
    while let Some(packet) = queue.pop_with(|slot| *slot) {
        assert!(packet.iter().all(|part| *part == packet[0]));
        seen.lock().unwrap()[packet[0]] += 1;
    }
    assert!(seen.lock().unwrap().iter().all(|count| *count == 1));
}
