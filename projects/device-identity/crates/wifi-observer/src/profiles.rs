//! Private exact-byte comparisons after the shared packet validator.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    BeaconConsumer, Error, InformationElements, MAX_CAPTURE_BYTES, analyze_with, validate_target,
};

/// Borrowed private capture and expected identity for [`compare_beacon_profiles`].
///
/// Construction validates only the input size and target; comparison validates
/// the complete capture. No input bytes, target values or lengths appear in
/// Debug, and this type deliberately has no serialization implementation.
/// See [`compare_beacon_profiles`] for a borrowed-input example.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BeaconInput<'a> {
    bytes: &'a [u8],
    expected_ssid: &'a [u8],
    expected_bssid: [u8; 6],
}

impl<'a> BeaconInput<'a> {
    /// Borrow at most [`MAX_CAPTURE_BYTES`] with a nonempty SSID of at most
    /// 32 bytes and a nonzero unicast target address. No I/O is performed.
    ///
    /// # Errors
    /// Returns [`Error::Limit`] for an oversized capture or
    /// [`Error::WitnessTarget`] for an invalid target. Capture framing/content
    /// errors are returned later by [`compare_beacon_profiles`].
    pub fn new(
        bytes: &'a [u8],
        expected_ssid: &'a [u8],
        expected_bssid: [u8; 6],
    ) -> Result<Self, Error> {
        validate_target(expected_ssid, expected_bssid)?;
        if bytes.len() > MAX_CAPTURE_BYTES {
            return Err(Error::Limit);
        }
        Ok(Self {
            bytes,
            expected_ssid,
            expected_bssid,
        })
    }
}

impl fmt::Debug for BeaconInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BeaconInput")
            .field("capture", &"<redacted>")
            .field("expected_identity", &"<redacted>")
            .finish()
    }
}

/// Equality counts for exact private field encodings; no encoding is emitted.
///
/// Distinct encodings describe within-capture variation. Sharing is the set
/// intersection, not a positional comparison. Identical means that the union
/// across every admitted beacon contains exactly one encoding. See
/// [`compare_beacon_profiles`] for an example and the compared fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncodingComparison {
    /// Number of distinct exact encodings in the first capture.
    pub before_distinct_encodings: u32,
    /// Number of distinct exact encodings in the second capture.
    pub after_distinct_encodings: u32,
    /// Number of exact encodings present in both captures.
    pub shared_distinct_encodings: u32,
    /// All beacons in both captures had one identical field encoding.
    pub identical_across_all_beacons: bool,
}

/// One non-SSID element ID, with presence and exact repeated-sequence counts.
///
/// A presence count is the number of beacons containing this ID at least once.
/// Absence is a distinct empty encoding, unlike a present zero-length TLV.
/// Complete repeated TLVs retain occurrence order. IDs absent in both captures
/// have no row; rows are sorted by ID only. See [`compare_beacon_profiles`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElementComparison {
    /// Information element ID; SSID ID zero is excluded.
    pub element_id: u8,
    /// First-capture beacons containing at least one occurrence.
    pub before_presence_count: u64,
    /// Second-capture beacons containing at least one occurrence.
    pub after_presence_count: u64,
    /// Counts for the ordered complete TLVs of this ID, including absence.
    pub encodings: EncodingComparison,
}

/// Counts-only comparison of two complete exact-target beacon captures.
///
/// Equality is descriptive for these saved packets. It does not establish
/// population uniqueness, continuity, transmitter authenticity or anonymity.
/// No values, payload lengths, hashes, OUIs or profile keys are returned. See
/// [`compare_beacon_profiles`] for admission, storage and example details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileComparison {
    /// Sanitized comparison format version.
    pub schema_version: u32,
    /// All admitted target packets in the first capture.
    pub before_target_packet_count: u64,
    /// All admitted target packets in the second capture.
    pub after_target_packet_count: u64,
    /// Whether the two private expected SSID byte strings are equal.
    pub expected_ssids_equal: bool,
    /// Whether the two private expected BSSID addresses are equal.
    pub expected_macs_equal: bool,
    /// Exact two-byte beacon interval encoding.
    pub beacon_interval: EncodingComparison,
    /// Exact two-byte capability-information encoding.
    pub capability_information: EncodingComparison,
    /// Full ordered complete IE TLVs, excluding only the complete SSID TLV.
    pub ordered_non_ssid_ies: EncodingComparison,
    /// Full ordered complete vendor-specific (ID 221) TLVs.
    pub ordered_vendor_ies: EncodingComparison,
    /// Non-SSID IDs observed in either capture, ordered by numeric ID.
    pub elements: Vec<ElementComparison>,
}

#[derive(Default)]
struct ElementProfile {
    presence_count: u64,
    encodings: BTreeSet<Vec<u8>>,
}

struct ProfileCollector<'a> {
    target: BeaconInput<'a>,
    packet_count: u64,
    intervals: BTreeSet<Vec<u8>>,
    capabilities: BTreeSet<Vec<u8>>,
    ies: BTreeSet<Vec<u8>>,
    vendors: BTreeSet<Vec<u8>>,
    elements: BTreeMap<u8, ElementProfile>,
}

impl<'a> ProfileCollector<'a> {
    fn collect(target: BeaconInput<'a>) -> Result<Self, Error> {
        let mut collector = Self {
            target,
            packet_count: 0,
            intervals: BTreeSet::new(),
            capabilities: BTreeSet::new(),
            ies: BTreeSet::new(),
            vendors: BTreeSet::new(),
            elements: BTreeMap::new(),
        };
        let summary = analyze_with(target.bytes, Some(&mut collector))?;
        // Every packet, including background, unsupported and bad-FCS frames,
        // must have reached this exact checked-beacon consumer. No subset can
        // turn an otherwise valid container into an admitted profile input.
        if collector.packet_count != summary.packet_count {
            return Err(Error::ProfileCapture);
        }
        Ok(collector)
    }
}

impl BeaconConsumer for ProfileCollector<'_> {
    fn observe(&mut self, frame: &[u8], ssid: &[u8], checked_fcs: bool) -> Result<(), Error> {
        if !checked_fcs
            || ssid != self.target.expected_ssid
            || frame[10..16] != self.target.expected_bssid
        {
            return Err(Error::ProfileCapture);
        }
        // The shared validator has established the complete ordinary beacon
        // shape, TA == BSSID, CRC and IE structure. The same TLV iterator used
        // by that validator supplies complete encodings here.
        let mut ies = Vec::new();
        let mut vendors = Vec::new();
        let mut elements: BTreeMap<u8, Vec<u8>> = BTreeMap::new();
        for element in InformationElements(&frame[36..]) {
            let element = element?;
            if element.kind == 0 {
                continue;
            }
            ies.extend_from_slice(element.tlv);
            if element.kind == 221 {
                vendors.extend_from_slice(element.tlv);
            }
            elements
                .entry(element.kind)
                .or_default()
                .extend_from_slice(element.tlv);
        }
        self.intervals.insert(frame[32..34].to_vec());
        self.capabilities.insert(frame[34..36].to_vec());
        self.ies.insert(ies);
        self.vendors.insert(vendors);
        for (kind, encoding) in elements {
            let profile = self.elements.entry(kind).or_default();
            profile.presence_count += 1;
            profile.encodings.insert(encoding);
        }
        self.packet_count += 1;
        Ok(())
    }
}

fn comparison(
    before: &BTreeSet<Vec<u8>>,
    after: &BTreeSet<Vec<u8>>,
) -> Result<EncodingComparison, Error> {
    Ok(EncodingComparison {
        before_distinct_encodings: before.len().try_into().map_err(|_| Error::Limit)?,
        after_distinct_encodings: after.len().try_into().map_err(|_| Error::Limit)?,
        shared_distinct_encodings: before
            .intersection(after)
            .count()
            .try_into()
            .map_err(|_| Error::Limit)?,
        identical_across_all_beacons: before.len() == 1
            && after.len() == 1
            && before.first() == after.first(),
    })
}

/// Compare exact collateral encodings without selecting a favorable subset.
///
/// Every packet in each nonempty input must pass the shared PCAPNG, radiotap,
/// ordinary-beacon, IE and included-FCS CRC validator and match its private
/// expected SSID and TA/BSSID. Both captures are validated through their final
/// blocks. Container validity or one good matching beacon is insufficient.
///
/// Fields are interval, capability information and complete ordered IE TLVs
/// excluding only the complete SSID TLV. Vendor TLVs and repeated TLVs per ID
/// are also compared in occurrence order. Boundaries, duplicates and unknown
/// IDs remain significant; only summary ID rows are sorted. TSF, sequence
/// control, other MAC-header bytes, radiotap, PCAPNG metadata and FCS bytes are
/// outside these profile encodings. Expected SSID/MAC equality is reported
/// separately. Equal profiles intentionally exclude the nonce SSID and do not
/// establish a unique fingerprint or correlation capability.
///
/// Inputs are borrowed, at most 8 MiB each. Exact private keys allocate only
/// during this call and are dropped before return. Retained IE-key byte content
/// is at most three times admitted IE content per capture (all, vendor and
/// per-ID sets), plus four fixed-field bytes per distinct packet and normal
/// BTreeSet/Vec bookkeeping linear in input size. Per-packet temporary encoding
/// buffers are bounded by the existing 65,535-byte packet limit. Standard Rust
/// allocation failure behavior applies; no allocation-recovery guarantee is
/// made. No I/O, hash keys, unsafe code or additional packet parser is used.
///
/// # Errors
/// Returns the shared decoder's static error for malformed, oversized, empty
/// or unsupported captures. Returns [`Error::ProfileCapture`] when any packet
/// is outside the complete checked target-beacon set, including background,
/// wrong identity, absent/check-failing FCS or unsupported frame variants.
///
/// ```
/// use wifi_observer::{BeaconInput, Error, ProfileComparison, compare_beacon_profiles};
///
/// fn compare_saved(before: &[u8], after: &[u8], ssid: &[u8]) -> Result<ProfileComparison, Error> {
///     let synthetic_target = [2, 0, 0, 0, 0, 1];
///     compare_beacon_profiles(
///         BeaconInput::new(before, ssid, synthetic_target)?,
///         BeaconInput::new(after, ssid, synthetic_target)?,
///     )
/// }
/// assert!(compare_saved(&[], &[], b"synthetic-private-trial").is_err());
/// ```
pub fn compare_beacon_profiles(
    before: BeaconInput<'_>,
    after: BeaconInput<'_>,
) -> Result<ProfileComparison, Error> {
    let mut left = ProfileCollector::collect(before)?;
    let mut right = ProfileCollector::collect(after)?;
    let ids: BTreeSet<_> = left
        .elements
        .keys()
        .chain(right.elements.keys())
        .copied()
        .collect();
    let mut elements = Vec::with_capacity(ids.len());
    for element_id in ids {
        let before = left.elements.entry(element_id).or_default();
        let after = right.elements.entry(element_id).or_default();
        if before.presence_count < left.packet_count {
            before.encodings.insert(Vec::new());
        }
        if after.presence_count < right.packet_count {
            after.encodings.insert(Vec::new());
        }
        elements.push(ElementComparison {
            element_id,
            before_presence_count: before.presence_count,
            after_presence_count: after.presence_count,
            encodings: comparison(&before.encodings, &after.encodings)?,
        });
    }
    Ok(ProfileComparison {
        schema_version: 1,
        before_target_packet_count: left.packet_count,
        after_target_packet_count: right.packet_count,
        expected_ssids_equal: before.expected_ssid == after.expected_ssid,
        expected_macs_equal: before.expected_bssid == after.expected_bssid,
        beacon_interval: comparison(&left.intervals, &right.intervals)?,
        capability_information: comparison(&left.capabilities, &right.capabilities)?,
        ordered_non_ssid_ies: comparison(&left.ies, &right.ies)?,
        ordered_vendor_ies: comparison(&left.vendors, &right.vendors)?,
        elements,
    })
}
