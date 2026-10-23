//! Portable integer work for comparing otherwise equivalent firmware builds.
//!
//! This crate defines a small, deterministic CPU workload, identified by
//! [`WORKLOAD_ID`]. It has no external dependencies, heap allocation, I/O, shared
//! mutable state, or unsafe code. It uses `core` without `alloc` or `std`.
//!
//! The workload measures one serial chain of 32-bit arithmetic. It is neither
//! CoreMark nor a comprehensive OS score; it does not exercise scheduling,
//! interrupts, networking, peripherals, memory allocation, or power management.
//! Compiler settings and instruction selection affect its execution time.
//!
//! # Timing and validation
//!
//! Pass runtime inputs through [`core::hint::black_box`] at the call site and
//! consume the result through it as well. This discourages optimization across
//! measurement boundaries; it is not a hardware memory barrier. Time only the
//! call with the adapter's timer, then validate the checksum outside that region.
//! Timer units, warmup, overhead, repetitions, and raw sample storage belong to
//! the adapter. [`DEFAULT_SAMPLES`] supplies a shared initial repetition count.
//!
//! ```
//! use bench_core::{DEFAULT_CHECKSUM, DEFAULT_ROUNDS, DEFAULT_SEED, workload};
//! use core::hint::black_box;
//!
//! // Read the adapter's timer immediately before and after this call.
//! let checksum = black_box(workload(black_box(DEFAULT_SEED), black_box(DEFAULT_ROUNDS)));
//! assert_eq!(checksum, DEFAULT_CHECKSUM);
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Protocol identifier for the exact recurrence documented by [`workload`].
///
/// A change to the recurrence requires a new identifier and known-answer vectors.
pub const WORKLOAD_ID: &str = "integer-mix-v1";

/// Default initial 32-bit state for [`workload`].
pub const DEFAULT_SEED: u32 = 0x1234_5678;

/// Default number of arithmetic rounds per [`workload`] invocation.
pub const DEFAULT_ROUNDS: u32 = 4096;

/// Default number of measured samples for an adapter to collect.
///
/// This excludes adapter-defined warmup; [`workload`] does not sample itself.
pub const DEFAULT_SAMPLES: u32 = 64;

/// Expected result of [`workload`] for [`DEFAULT_SEED`] and [`DEFAULT_ROUNDS`].
pub const DEFAULT_CHECKSUM: u32 = 0xa394_4d55;

/// Returns the final 32-bit state after `rounds` of integer mixing from `seed`.
///
/// All `u32` inputs are valid. Zero rounds returns `seed` unchanged. At round
/// index `r`, starting at zero, the state changes in this exact order:
///
/// 1. Add `0x9e37_79b9 ^ r`, discarding carries beyond bit 31.
/// 2. Rotate the state left by seven bits within its 32-bit word.
/// 3. Multiply by `0x85eb_ca6b`, discarding bits beyond bit 31.
/// 4. XOR the state with itself shifted right by 16 bits.
///
/// This function performs no allocation, I/O, locking, sleeping, or scheduler
/// interaction. It synchronously consumes CPU time proportional to `rounds` and
/// does not yield. Choose a bounded round count appropriate for the execution
/// context; a large count may prevent other cooperative tasks from running.
/// The function has no error, panic, or unsafe preconditions and is reentrant.
///
/// The call is deliberately not inlined so adapters share a function boundary.
/// Callers should still use [`core::hint::black_box`] as shown in the
/// [crate example](crate#timing-and-validation), then validate the result outside
/// their measured region. No timer is read here and no time unit is returned.
///
/// # Examples
///
/// ```
/// use bench_core::{DEFAULT_CHECKSUM, DEFAULT_ROUNDS, DEFAULT_SEED, workload};
///
/// assert_eq!(workload(DEFAULT_SEED, DEFAULT_ROUNDS), DEFAULT_CHECKSUM);
/// assert_eq!(workload(0xdead_beef, 0), 0xdead_beef);
/// ```
#[inline(never)]
pub fn workload(seed: u32, rounds: u32) -> u32 {
    let mut state = seed;
    for round in 0..rounds {
        state = state.wrapping_add(0x9e37_79b9 ^ round);
        state = state.rotate_left(7);
        state = state.wrapping_mul(0x85eb_ca6b);
        state ^= state >> 16;
    }
    state
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_CHECKSUM, DEFAULT_ROUNDS, DEFAULT_SEED, workload};

    #[test]
    fn matches_independently_calculated_vectors() {
        // Fixed answers were calculated with arbitrary-width integer arithmetic,
        // modulo 2^32 and explicit shift/concatenation for rotation. They do not
        // invoke this Rust implementation as an oracle.
        let vectors = [
            (0, 0, 0),
            (u32::MAX, 0, u32::MAX),
            (0, 1, 0xdb30_7bb5),
            (1, 1, 0xd115_0710),
            (0x1234_5678, 1, 0xb1e4_63ac),
            (u32::MAX, 1, 0xe54b_8e4e),
            (0x1234_5678, 64, 0xe72d_e5ac),
            (0xdead_beef, 257, 0x93d2_0012),
            (0, 4096, 0xeb6d_0751),
            (0x1234_5678, 4096, 0xa394_4d55),
            (u32::MAX, 4096, 0x2a8a_fcf1),
        ];
        for (seed, rounds, expected) in vectors {
            assert_eq!(workload(seed, rounds), expected, "{seed:#010x}, {rounds}");
        }
        assert_eq!(workload(DEFAULT_SEED, DEFAULT_ROUNDS), DEFAULT_CHECKSUM);
    }

    #[test]
    fn one_round_is_invertible_across_boundary_states() {
        // Every step of round zero is invertible modulo 2^32: undo the xor-shift
        // (which is its own inverse), multiply by the modular inverse, rotate
        // right, and subtract. This verifies direction and width independently
        // of the forward operation sequence. 0xa5cb_9243 is the multiplicative
        // inverse of 0x85eb_ca6b modulo 2^32.
        const MULTIPLIER_INVERSE: u32 = 0xa5cb_9243;
        assert_eq!(0x85eb_ca6b_u32.wrapping_mul(MULTIPLIER_INVERSE), 1);
        for seed in [0, 1, 0x7fff_ffff, 0x8000_0000, 0x1234_5678, u32::MAX] {
            let result = workload(seed, 1);
            let before_xor = result ^ (result >> 16);
            let before_multiply = before_xor.wrapping_mul(MULTIPLIER_INVERSE);
            let recovered_seed = before_multiply.rotate_right(7).wrapping_sub(0x9e37_79b9);
            assert_eq!(recovered_seed, seed);
        }
    }
}
