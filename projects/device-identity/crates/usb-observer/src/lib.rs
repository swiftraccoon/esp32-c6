//! Read-only USB control requests with private raw data and fixed-field summaries.
//!
//! The planner accepts no caller-provided request sequence. It reads the fresh
//! device descriptor and, when a serial index is present, its LANGID table and
//! serial string, and then the CP2102N part
//! and configuration only for the bridge domain. [`summarize`] replays that exact
//! order and validates every returned shape. This validates recorded transfers;
//! it does not authenticate their origin or infer bridge configuration lock.
//!
//! Native capture is available only on macOS. Offline decoding and synthetic
//! controls are portable. Raw serialization contains identifiers and bridge
//! configuration and must remain in private storage. Debug formatting redacts
//! transfer bytes and never prints descriptor strings.
//!
//! Version 2 requests a four-byte LANGID prefix and a two-byte serial prefix,
//! then fetches the exact declared descriptor length when it is larger. Every
//! repeated response must retain its complete earlier prefix. This is an
//! explicit method revision; version-1 records are not silently upgraded.
//!
//! A failed request remains in the raw record instead of becoming a successful
//! empty descriptor. This example performs no device operation:
//!
//! ```
//! use usb_observer::{Domain, FORMAT_VERSION, Record, Transfer, observe_domain, parse, summarize};
//! let failed = observe_domain(Domain::NativeUsbSerialJtag, 1, 0, |request| Transfer {
//!     request, status: -1, returned_length: 0, bytes: Vec::new(),
//! });
//! let missing = observe_domain(Domain::Cp2102nBridge, 0, 0, |_| {
//!     unreachable!("an absent device must not receive requests")
//! });
//! let record = Record { version: FORMAT_VERSION, domains: [failed, missing] };
//! let saved = serde_json::to_vec(&record)?;
//! assert!(summarize(&parse(&saved)?).is_err());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod record;

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod native;

#[cfg(target_os = "macos")]
pub use native::{native_provenance, observe};
pub use record::*;
