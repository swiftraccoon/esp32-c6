//! Pure bounded protocol and controller for stopped-state BLE controls.
//!
//! Host tests perform no I/O. The separate target binary lazily initializes
//! Bluetooth after the first valid Start. Neither compilation nor SDK events
//! qualify transmitted addresses; an independent observer is required.
#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[doc(hidden)]
pub mod completion;
pub mod controller;
pub mod protocol;

#[cfg(test)]
extern crate std;
