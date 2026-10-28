//! Bounded protocol identities constructed from caller-provided randomness.
//!
//! This dependency-free library performs no I/O, allocation, entropy collection
//! or interface initialization. The caller owns random-source quality and the
//! identity lifetime. An all-zero sample is only a rejected sentinel, not an
//! entropy-health test. Reusing bytes reuses the same identity; a fixed random
//! identity can still correlate sessions.
//!
//! MACs and SSIDs are observable protocol identifiers, not credentials. Their
//! [`Debug`](core::fmt::Debug) output is redacted; explicit accessors expose the
//! values needed by an integration. Construction does not prove application,
//! transmitted behavior or anonymity.
//!
//! ```
//! use c6_anonymization_identity::Identity;
//!
//! // Synthetic input for this example; production callers must obtain randomness
//! // from a source qualified for their actual startup and operating conditions.
//! let sample = [
//!     0x85, 0x27, 0x39, 0x4b, 0x5d, 0x6f,
//!     0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe,
//! ];
//! let identity = Identity::from_random_bytes(sample)?;
//! assert_eq!(identity.mac()[0] & 0b11, 0b10);
//! assert_eq!(identity.ssid(), "anon-1032547698badcfe");
//! # Ok::<(), c6_anonymization_identity::IdentityError>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// A locally administered unicast MAC, without an entropy or freshness claim.
///
/// The representation is private, and [`Self::from_random_bytes`] normalizes the
/// two address-type bits. See the [crate example](crate) for its use in an identity.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LocalMac([u8; 6]);

impl LocalMac {
    /// Set the locally administered bit and clear the multicast bit.
    ///
    /// All other bits are preserved. This adds no entropy and accepts every
    /// input; random-source quality and collision handling remain caller-owned.
    /// See the [crate example](crate) for a synthetic input.
    pub const fn from_random_bytes(mut bytes: [u8; 6]) -> Self {
        bytes[0] = (bytes[0] | 0b0000_0010) & 0b1111_1110;
        Self(bytes)
    }

    /// Return the address bytes for explicit integration with an interface.
    ///
    /// See [`Identity::mac`] and the [crate example](crate).
    pub const fn bytes(&self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Debug for LocalMac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LocalMac { <redacted> }")
    }
}

/// A local unicast MAC and a 21-byte ASCII SSID for one caller-chosen lifetime.
///
/// The [crate example](crate) demonstrates construction. Copying this data does
/// not authorize hardware operations or create a fresh identity.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Identity {
    mac: LocalMac,
    ssid: [u8; 21],
}

impl Identity {
    /// Construct a MAC from the first six bytes and an SSID from the last eight.
    ///
    /// The SSID is `anon-` followed by sixteen lowercase hexadecimal digits.
    /// The caller owns random-source quality, identity lifetime and collision
    /// handling. See the [crate example](crate).
    ///
    /// # Errors
    /// Returns [`IdentityError::AllZeroSample`] for an entirely zero input.
    /// Rejecting this sentinel does not establish entropy quality or freshness.
    pub fn from_random_bytes(sample: [u8; 14]) -> Result<Self, IdentityError> {
        if sample == [0; 14] {
            return Err(IdentityError::AllZeroSample);
        }

        let mut mac = [0; 6];
        mac.copy_from_slice(&sample[..6]);
        let mut ssid = *b"anon-0000000000000000";
        for (index, byte) in sample[6..].iter().copied().enumerate() {
            ssid[5 + index * 2] = HEX_DIGITS[usize::from(byte >> 4)];
            ssid[6 + index * 2] = HEX_DIGITS[usize::from(byte & 0x0f)];
        }

        Ok(Self {
            mac: LocalMac::from_random_bytes(mac),
            ssid,
        })
    }

    /// Return the MAC bytes for explicit interface configuration.
    ///
    /// See the [crate example](crate). Reading this value does not apply it.
    pub const fn mac(&self) -> [u8; 6] {
        self.mac.bytes()
    }

    /// Borrow the 21-byte ASCII SSID for explicit interface configuration.
    ///
    /// See the [crate example](crate). Private construction restricts every byte
    /// to the fixed ASCII prefix and hexadecimal alphabet, so UTF-8 conversion
    /// cannot fail for a constructible identity.
    pub fn ssid(&self) -> &str {
        core::str::from_utf8(&self.ssid).expect("identity SSID is ASCII by construction")
    }
}

impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Identity { <redacted> }")
    }
}

/// A static identity-input failure, without the input bytes.
///
/// See [`Identity::from_random_bytes`] and the [crate example](crate).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    /// The complete fourteen-byte input was zero.
    AllZeroSample,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllZeroSample => f.write_str("identity sample must not be entirely zero"),
        }
    }
}

impl core::error::Error for IdentityError {}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::format;

    const SAMPLE: [u8; 14] = [
        0x85, 0x27, 0x39, 0x4b, 0x5d, 0x6f, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe,
    ];

    #[test]
    fn every_address_type_normalizes_to_local_unicast_without_changing_other_bits() {
        for first in 0..=u8::MAX {
            let input = [first, 0x27, 0x39, 0x4b, 0x5d, 0x6f];
            let output = LocalMac::from_random_bytes(input).bytes();
            assert_eq!(output[0] & 0b11, 0b10);
            assert_eq!(output[0] & 0b1111_1100, input[0] & 0b1111_1100);
            assert_eq!(&output[1..], &input[1..]);
        }
    }

    #[test]
    fn nonuniform_input_has_exact_mac_and_bounded_lowercase_ascii_ssid() {
        let identity = Identity::from_random_bytes(SAMPLE).unwrap();
        assert_eq!(identity.mac(), [0x86, 0x27, 0x39, 0x4b, 0x5d, 0x6f]);
        assert_eq!(identity.ssid(), "anon-1032547698badcfe");
        assert_eq!(identity.ssid().len(), 21);
        assert!(identity.ssid().is_ascii());
        assert!(
            identity.ssid().as_bytes()[5..]
                .iter()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        );
    }

    #[test]
    fn only_the_entire_zero_sample_is_the_rejected_sentinel() {
        assert_eq!(
            Identity::from_random_bytes([0; 14]),
            Err(IdentityError::AllZeroSample)
        );
        for index in 0..14 {
            let mut input = [0; 14];
            input[index] = 1;
            assert!(Identity::from_random_bytes(input).is_ok());
        }
    }

    #[test]
    fn hexadecimal_encoding_preserves_leading_zeroes_and_extreme_bytes() {
        let sample = [1, 0, 0, 0, 0, 0, 0, 1, 0x0f, 0x10, 0x7f, 0x80, 0xfe, 0xff];
        assert_eq!(
            Identity::from_random_bytes(sample).unwrap().ssid(),
            "anon-00010f107f80feff"
        );
    }

    #[test]
    fn repeated_input_reproduces_data_without_a_freshness_claim() {
        let first = Identity::from_random_bytes(SAMPLE).unwrap();
        let repeated = Identity::from_random_bytes(SAMPLE).unwrap();
        assert_eq!(first, repeated);
        let copied = first;
        assert_eq!(copied, first);
    }

    #[test]
    fn debug_and_errors_do_not_expose_input_or_protocol_identifiers() {
        let identity = Identity::from_random_bytes(SAMPLE).unwrap();
        let mac = LocalMac::from_random_bytes([0x85, 0x27, 0x39, 0x4b, 0x5d, 0x6f]);
        assert_eq!(format!("{identity:?}"), "Identity { <redacted> }");
        assert_eq!(format!("{mac:?}"), "LocalMac { <redacted> }");
        assert_eq!(
            format!("{}", IdentityError::AllZeroSample),
            "identity sample must not be entirely zero"
        );
        assert_eq!(
            format!("{:?}", IdentityError::AllZeroSample),
            "AllZeroSample"
        );
    }
}
