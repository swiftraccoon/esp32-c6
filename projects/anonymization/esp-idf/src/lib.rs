//! Explicit startup preparation and two stopped/restarted SoftAP identities.
//!
//! [`Epoch`] selects two identities prepared from one startup entropy window.
//! The ESP32-C6-only `SoftAp` adapter is available on `target_os = "espidf"`.
//! It never starts a radio during construction, programs persistent identity,
//! or restores a factory-derived address on Stop. The example integration is
//! under `projects/anonymization/examples/softap` in this repository.
//!
//! Setters and SDK observations are not external packet witnesses, RF silence,
//! or anonymity. Initialization is unsafe because the SDK's global MAC table
//! and SAR entropy source require exclusive early application startup.

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use core::fmt;

#[cfg(target_os = "espidf")]
pub use backend::SoftAp;
pub use sequence::Epoch;

#[cfg(target_os = "espidf")]
mod backend;
#[cfg(any(target_os = "espidf", test))]
mod plan;
mod sequence;

#[cfg(all(target_os = "espidf", not(esp32c6)))]
compile_error!("the anonymization adapter supports ESP32-C6 only");
#[cfg(all(target_os = "espidf", esp_idf_esp_bringup_bypass_random_setting))]
compile_error!("the SDK bring-up randomness bypass must be disabled");

/// A fixed diagnostic that never contains an identity or random sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Initialization was already attempted in this boot.
    AlreadyAttempted,
    /// A rejected sentinel sample or unusable identity was returned.
    RandomSample,
    /// Prepared identities collide after normalization or share an SSID.
    IdentityCollision,
    /// The requested epoch is consumed, active, or lacks its required Stop.
    Order,
    /// A preceding error permanently prohibits another Start in this boot.
    Failed,
    /// The event-backed state did not reach the selected value in one second.
    Timeout,
    /// SDK mode, active state or address disagrees with the selected identity.
    Readback,
    /// An SDK operation returned its original numeric failure status.
    Sdk {
        /// Fixed operation context; never a caller-provided diagnostic.
        operation: &'static str,
        /// Original SDK error code.
        code: i32,
    },
}

impl Error {
    /// Return a fixed local status or the original SDK failure code.
    ///
    /// Local statuses are internal diagnostics, not a stable wire protocol.
    pub const fn code(self) -> i32 {
        match self {
            Self::AlreadyAttempted => -6001,
            Self::RandomSample => -6002,
            Self::IdentityCollision => -6003,
            Self::Order => -6004,
            Self::Failed => -6005,
            Self::Timeout => -6006,
            Self::Readback => -6007,
            Self::Sdk { code, .. } => code,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyAttempted => f.write_str("startup preparation already attempted"),
            Self::RandomSample => f.write_str("startup randomness sample rejected"),
            Self::IdentityCollision => f.write_str("prepared identity collision"),
            Self::Order => f.write_str("advertising epoch order rejected"),
            Self::Failed => f.write_str("a previous failure prohibits another start"),
            Self::Timeout => f.write_str("SDK state observation timed out"),
            Self::Readback => f.write_str("SDK identity or mode readback disagrees"),
            Self::Sdk { operation, code } => write!(f, "SDK {operation} failed ({code})"),
        }
    }
}

impl core::error::Error for Error {}
