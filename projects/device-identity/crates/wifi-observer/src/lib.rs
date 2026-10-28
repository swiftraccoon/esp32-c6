//! Strict, bounded observations of private radiotap PCAPNG inputs.
//!
//! This decoder performs no I/O and returns counters, equality states and
//! standard element IDs, without private values or profile keys. One section,
//! interface link type 127, complete EPBs, timestamp-unit arithmetic and ordinary
//! infrastructure beacon framing are supported. Other frame classes are counted
//! without interpreting their address roles. A parsed capture does not establish
//! source authenticity, lossless delivery, target reception or RF silence.
//!
//! Pass privately retained bytes by borrowing them; only fixed counters leave
//! the decoder. Even an empty or incomplete capture must return an error:
//!
//! ```
//! use wifi_observer::{Error, analyze};
//!
//! fn beacon_count(saved_bytes: &[u8]) -> Result<u64, Error> {
//!     Ok(analyze(saved_bytes)?.beacon_count)
//! }
//!
//! assert!(beacon_count(&[]).is_err());
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;

use pcap_parser::pcapng::{Block, parse_block_be, parse_block_le};
use serde::{Deserialize, Serialize};

use framing::BlockFramer;
pub use profiles::{
    BeaconInput, ElementComparison, EncodingComparison, ProfileComparison, compare_beacon_profiles,
};

pub mod framing;
mod profiles;
mod station;
pub use station::{StationProbeSummary, station_probe_witness};

/// Maximum admitted capture size; the original failed input must be retained.
pub const MAX_CAPTURE_BYTES: usize = 8 * 1024 * 1024;
/// Inclusive bound for an individual PCAPNG block.
pub const MAX_BLOCK_BYTES: usize = 1024 * 1024;
const MAX_INTERFACES: usize = 16;
const MAX_PACKET_BYTES: usize = 65535;
const MAX_OPTIONS: usize = 256;

/// Static decoding failure; no variant contains packet data or identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Input exceeds an admitted storage or format bound.
    Limit,
    /// PCAPNG block framing or decoding failed.
    Pcapng,
    /// A section version or declared section length is unsupported or wrong.
    Section,
    /// A block type is outside the declared observation format.
    BlockType,
    /// A PCAPNG option list is malformed or contradictory.
    Options,
    /// An interface is absent, unsupported, malformed or ambiguous.
    Interface,
    /// Packet lengths disagree or represent captured truncation.
    PacketLength,
    /// Packet timestamp units or offset arithmetic are invalid.
    Timestamp,
    /// Radiotap framing or metadata is malformed or outside the supported profile.
    Radiotap,
    /// A supported management header or beacon is malformed.
    Beacon,
    /// Beacon information elements are malformed or contradictory.
    InformationElements,
    /// No packet was present in an otherwise bounded capture.
    NoPackets,
    /// A private expected beacon identity has invalid width or address class.
    WitnessTarget,
    /// A profile input contains a packet outside the exact checked target.
    ProfileCapture,
    /// A supported station probe header or its information elements are invalid.
    StationProbe,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Limit => "Wi-Fi observation exceeds a format bound",
            Self::Pcapng => "invalid or incomplete PCAPNG framing",
            Self::Section => "unsupported or inconsistent PCAPNG section",
            Self::BlockType => "unsupported PCAPNG block type",
            Self::Options => "invalid PCAPNG option list",
            Self::Interface => "unsupported or inconsistent PCAPNG interface",
            Self::PacketLength => "invalid or truncated PCAPNG packet",
            Self::Timestamp => "invalid PCAPNG packet timestamp",
            Self::Radiotap => "unsupported or invalid radiotap packet metadata",
            Self::Beacon => "invalid supported Wi-Fi management frame",
            Self::InformationElements => "invalid beacon information elements",
            Self::NoPackets => "PCAPNG observation contains no packets",
            Self::WitnessTarget => "invalid private beacon witness target",
            Self::ProfileCapture => {
                "profile comparison requires only CRC-verified exact target beacons"
            }
            Self::StationProbe => "invalid supported station probe request",
        })
    }
}

impl std::error::Error for Error {}

/// Fixed counters, with no addresses, payloads, interface labels or timestamps.
///
/// `packet_count` equals beacon, other, bad-FCS and unsupported frame counts.
/// Management count overlaps the frame-class counters. FCS-unqualified count overlaps
/// the frame classes whose bytes contained no checkable FCS. Successful decoding
/// qualifies only the declared structural checks, not delivery fidelity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    /// Sanitized summary format version.
    pub schema_version: u32,
    /// Number of declared radiotap interfaces.
    pub interface_count: u32,
    /// Number of structurally complete enhanced packet blocks.
    pub packet_count: u64,
    /// Version-zero management frames with at least a complete base header.
    pub management_frame_count: u64,
    /// Ordinary beacons passing supported address and IE checks.
    pub beacon_count: u64,
    /// Other version-zero management, control or data frame classes.
    pub other_frame_count: u64,
    /// Frames marked bad-FCS or failing the included CRC32 check.
    pub bad_fcs_count: u64,
    /// Extension/version/unsupported beacon-header variants, without decoding.
    pub unsupported_frame_count: u64,
    /// Frames whose bytes did not include a checkable FCS.
    pub fcs_unqualified_count: u64,
    /// Validated beacons with a nonempty, nonzero SSID byte string.
    pub visible_ssid_beacon_count: u64,
    /// Validated beacons with an empty or all-zero SSID byte string.
    pub hidden_ssid_beacon_count: u64,
    /// Validated beacons with a well-shaped DS Parameter Set element.
    pub channel_ie_beacon_count: u64,
    /// Validated beacons with a well-shaped TIM element.
    pub tim_ie_beacon_count: u64,
    /// Validated beacons containing an opaque RSN element.
    pub rsn_ie_beacon_count: u64,
    /// Validated beacons containing an opaque vendor-specific element.
    pub vendor_ie_beacon_count: u64,
}

/// Counts relative to a private expected SSID/MAC, without exposing that target.
///
/// A nonzero `matching_beacon_count` is a presence witness only. It does not
/// authenticate a transmitter, establish full-window coverage or prove silence.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessSummary {
    /// Sanitized witness format version.
    pub schema_version: u32,
    /// All structurally complete packet records, not just target frames.
    pub packet_count: u64,
    /// Supported beacons whose included FCS was checked successfully.
    pub qualified_beacon_count: u64,
    /// CRC-verified beacons with exact SSID and matching transmitter/BSSID.
    pub matching_beacon_count: u64,
    /// Exact identity matches that lacked a checkable FCS; never witnesses.
    pub matching_unqualified_fcs_beacon_count: u64,
    /// CRC-verified beacons with the expected SSID but another transmitter/BSSID.
    pub matching_ssid_wrong_mac_count: u64,
    /// CRC-verified beacons with the expected transmitter/BSSID but another SSID.
    pub matching_mac_wrong_ssid_count: u64,
}

struct WitnessTarget<'a> {
    ssid: &'a [u8],
    bssid: [u8; 6],
    counts: WitnessSummary,
}

trait BeaconConsumer {
    fn observe(&mut self, frame: &[u8], ssid: &[u8], checked_fcs: bool) -> Result<(), Error>;
}

trait FrameConsumer {
    fn observe(&mut self, frame: &[u8], has_fcs: bool) -> Result<(), Error>;
}

impl BeaconConsumer for WitnessTarget<'_> {
    fn observe(&mut self, frame: &[u8], ssid: &[u8], checked_fcs: bool) -> Result<(), Error> {
        // The shared beacon validator has already established 36 header bytes,
        // a unicast transmitter equal to BSSID, and bounded unique SSID framing.
        let same_mac = frame[10..16] == self.bssid;
        let same_ssid = ssid == self.ssid;
        if !checked_fcs {
            self.counts.matching_unqualified_fcs_beacon_count += u64::from(same_mac && same_ssid);
            return Ok(());
        }
        self.counts.qualified_beacon_count += 1;
        self.counts.matching_beacon_count += u64::from(same_mac && same_ssid);
        self.counts.matching_ssid_wrong_mac_count += u64::from(!same_mac && same_ssid);
        self.counts.matching_mac_wrong_ssid_count += u64::from(same_mac && !same_ssid);
        Ok(())
    }
}

fn validate_target(expected_ssid: &[u8], expected_bssid: [u8; 6]) -> Result<(), Error> {
    if expected_ssid.is_empty()
        || expected_ssid.len() > 32
        || expected_bssid == [0; 6]
        || expected_bssid[0] & 1 != 0
    {
        return Err(Error::WitnessTarget);
    }
    Ok(())
}

/// Validate exactly one stream section header before any transmitter is started.
///
/// The prefix must be a complete version-1.0 SHB with an unknown section length
/// and a bounded valid option list. No IDB/EPB or trailing byte is accepted.
/// This does not authenticate its source or establish packet reception.
///
/// # Errors
/// Returns a static section/framing/options error; input bytes are never emitted.
pub fn validate_section_prefix(bytes: &[u8]) -> Result<(), Error> {
    let mut framer = BlockFramer::new(MAX_BLOCK_BYTES).map_err(|_| Error::Pcapng)?;
    let (consumed, boundary) = framer.push(bytes).map_err(|_| Error::Pcapng)?;
    let boundary = boundary.ok_or(Error::Section)?;
    if consumed != bytes.len() || boundary.block_type != 0x0a0d0d0a {
        return Err(Error::Section);
    }
    framer.finish().map_err(|_| Error::Pcapng)?;
    let order = ByteOrder(bytes[8..12] == [0x4d, 0x3c, 0x2b, 0x1a]);
    if order.u16(&bytes[12..]) != 1 || order.u16(&bytes[14..]) != 0 || order.i64(&bytes[16..]) != -1
    {
        return Err(Error::Section);
    }
    options(&bytes[24..bytes.len() - 4], order, |_, _| Ok(()))
}

/// Validate all packets and count exact private identity matches in good beacons.
///
/// Only ordinary supported beacons with an included verified FCS can witness
/// the target. Valid captures without such a beacon return zero matching count;
/// the trial admission layer must reject that outcome. See [`analyze`] for the
/// complete container and decoder profile.
///
/// # Errors
/// Rejects an empty/oversized expected SSID, invalid expected unicast MAC, or any
/// malformed/unsupported capture content rejected by the shared decoder.
pub fn beacon_witness(
    bytes: &[u8],
    expected_ssid: &[u8],
    expected_bssid: [u8; 6],
) -> Result<WitnessSummary, Error> {
    validate_target(expected_ssid, expected_bssid)?;
    let mut target = WitnessTarget {
        ssid: expected_ssid,
        bssid: expected_bssid,
        counts: WitnessSummary {
            schema_version: 1,
            ..WitnessSummary::default()
        },
    };
    let summary = analyze_with(bytes, Some(&mut target))?;
    target.counts.packet_count = summary.packet_count;
    Ok(target.counts)
}

#[derive(Clone, Copy)]
struct ByteOrder(bool);

impl ByteOrder {
    fn u16(self, bytes: &[u8]) -> u16 {
        let value = [bytes[0], bytes[1]];
        if self.0 {
            u16::from_le_bytes(value)
        } else {
            u16::from_be_bytes(value)
        }
    }

    fn u32(self, bytes: &[u8]) -> u32 {
        let value = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if self.0 {
            u32::from_le_bytes(value)
        } else {
            u32::from_be_bytes(value)
        }
    }

    fn i64(self, bytes: &[u8]) -> i64 {
        let value: [u8; 8] = bytes[..8].try_into().expect("validated fixed field");
        if self.0 {
            i64::from_le_bytes(value)
        } else {
            i64::from_be_bytes(value)
        }
    }
}

struct Interface {
    snaplen: u32,
    units_per_second: u64,
    offset_seconds: i64,
    fcs: Option<bool>,
}

fn options(
    mut bytes: &[u8],
    order: ByteOrder,
    mut accept: impl FnMut(u16, &[u8]) -> Result<(), Error>,
) -> Result<(), Error> {
    if bytes.is_empty() {
        return Ok(());
    }
    for _ in 0..MAX_OPTIONS {
        // Readers must also recognize an exactly completed option list whose
        // writer omitted opt_endofopt (pcapng draft section 3.5).
        if bytes.is_empty() {
            return Ok(());
        }
        if bytes.len() < 4 {
            return Err(Error::Options);
        }
        let code = order.u16(bytes);
        let length = usize::from(order.u16(&bytes[2..]));
        bytes = &bytes[4..];
        if code == 0 {
            return if length == 0 && bytes.is_empty() {
                Ok(())
            } else {
                Err(Error::Options)
            };
        }
        let padded = length.checked_add(3).ok_or(Error::Options)? & !3;
        if padded > bytes.len() {
            return Err(Error::Options);
        }
        accept(code, &bytes[..length])?;
        bytes = &bytes[padded..];
    }
    Err(Error::Options)
}

fn interface(bytes: &[u8], order: ByteOrder) -> Result<Interface, Error> {
    // The reserved IDB field is ignored by readers.
    if order.u16(&bytes[8..]) != 127 {
        return Err(Error::Interface);
    }
    let snaplen = order.u32(&bytes[12..]);
    if snaplen > MAX_PACKET_BYTES as u32 {
        return Err(Error::Interface);
    }
    let mut resolution = None;
    let mut offset = None;
    let mut fcs = None;
    options(&bytes[16..bytes.len() - 4], order, |code, value| {
        match code {
            9 => {
                if value.len() != 1 || resolution.replace(value[0]).is_some() {
                    return Err(Error::Options);
                }
            }
            14 => {
                if value.len() != 8 || offset.replace(order.i64(value)).is_some() {
                    return Err(Error::Options);
                }
            }
            13 => {
                if value.len() != 1 || !matches!(value[0], 0 | 32) || fcs.is_some() {
                    return Err(Error::Options);
                }
                fcs = Some(value[0] == 32);
            }
            _ => {}
        }
        Ok(())
    })?;
    let resolution = resolution.unwrap_or(6);
    let exponent = resolution & 0x7f;
    let units = if resolution & 0x80 == 0 {
        (exponent <= 19).then(|| 10_u64.pow(u32::from(exponent)))
    } else {
        (exponent <= 63).then(|| 1_u64 << exponent)
    }
    .ok_or(Error::Timestamp)?;
    Ok(Interface {
        snaplen,
        units_per_second: units,
        offset_seconds: offset.unwrap_or(0),
        fcs,
    })
}

/// Decode a complete, bounded private capture into identifier-free counters.
///
/// Container errors, unsupported link types, capture truncation, invalid
/// timestamp arithmetic and malformed supported beacon/radiotap structures
/// fail the whole observation. Bad-FCS frames and unsupported frame variants are
/// counted separately and never become validated beacons. Input is borrowed only
/// during this call; diagnostics never include input bytes or decoder internals.
pub fn analyze(bytes: &[u8]) -> Result<Summary, Error> {
    analyze_with(bytes, None)
}

fn analyze_with(bytes: &[u8], consumer: Option<&mut dyn BeaconConsumer>) -> Result<Summary, Error> {
    analyze_consumers(bytes, consumer, None)
}

fn analyze_consumers(
    bytes: &[u8],
    mut consumer: Option<&mut dyn BeaconConsumer>,
    mut frames: Option<&mut dyn FrameConsumer>,
) -> Result<Summary, Error> {
    if bytes.len() > MAX_CAPTURE_BYTES {
        return Err(Error::Limit);
    }
    let mut framer = BlockFramer::new(MAX_BLOCK_BYTES).map_err(|_| Error::Pcapng)?;
    let mut interfaces = Vec::new();
    let mut order = None;
    let mut start = 0;
    let mut consumed = 0;
    let mut summary = Summary {
        schema_version: 1,
        ..Summary::default()
    };
    while consumed < bytes.len() {
        let (count, boundary) = framer.push(&bytes[consumed..]).map_err(|_| Error::Pcapng)?;
        consumed += count;
        let Some(boundary) = boundary else { continue };
        let end = usize::try_from(boundary.end_offset).map_err(|_| Error::Limit)?;
        let block_bytes = &bytes[start..end];
        if order.is_none() {
            order = Some(ByteOrder(block_bytes[8..12] == [0x4d, 0x3c, 0x2b, 0x1a]));
        }
        let order = order.ok_or(Error::Section)?;
        // Bound option counts and packet-data ranges before the third-party
        // parser allocates its option vector. Unsupported block parsers are
        // never invoked by this narrow observation format.
        let option_start = match boundary.block_type {
            0x0a0d0d0a => 24,
            1 => 16,
            6 => {
                let length = order.u32(&block_bytes[20..]) as usize;
                if length > MAX_PACKET_BYTES {
                    return Err(Error::PacketLength);
                }
                28 + ((length + 3) & !3)
            }
            _ => return Err(Error::BlockType),
        };
        if option_start > block_bytes.len() - 4 {
            return Err(Error::PacketLength);
        }
        options(
            &block_bytes[option_start..block_bytes.len() - 4],
            order,
            |_, _| Ok(()),
        )?;
        let (remaining, block) = if order.0 {
            parse_block_le(block_bytes)
        } else {
            parse_block_be(block_bytes)
        }
        .map_err(|_| Error::Pcapng)?;
        if !remaining.is_empty() {
            return Err(Error::Pcapng);
        }
        match block {
            Block::SectionHeader(section) => {
                if start != 0
                    || section.major_version != 1
                    || section.minor_version != 0
                    || (section.section_len != -1
                        && usize::try_from(section.section_len).ok() != Some(bytes.len() - end))
                {
                    return Err(Error::Section);
                }
                options(
                    &block_bytes[24..block_bytes.len() - 4],
                    order,
                    |_, _| Ok(()),
                )?;
            }
            Block::InterfaceDescription(_) => {
                if interfaces.len() == MAX_INTERFACES {
                    return Err(Error::Limit);
                }
                interfaces.push(interface(block_bytes, order)?);
                summary.interface_count += 1;
            }
            Block::EnhancedPacket(packet) => {
                let interface = interfaces
                    .get_mut(packet.if_id as usize)
                    .ok_or(Error::Interface)?;
                if packet.caplen != packet.origlen
                    || packet.caplen == 0
                    || packet.caplen as usize > MAX_PACKET_BYTES
                    || (interface.snaplen != 0 && packet.caplen > interface.snaplen)
                {
                    return Err(Error::PacketLength);
                }
                let padded = (packet.caplen as usize + 3) & !3;
                let mut packet_flags = None;
                options(
                    &block_bytes[28 + padded..block_bytes.len() - 4],
                    order,
                    |code, value| {
                        if code == 2
                            && (value.len() != 4
                                || packet_flags.replace(order.u32(value)).is_some())
                        {
                            return Err(Error::Options);
                        }
                        Ok(())
                    },
                )?;
                let packet_flags = packet_flags.unwrap_or(0);
                let fcs_length = (packet_flags >> 5) & 15;
                if !matches!(fcs_length, 0 | 4) {
                    return Err(Error::Options);
                }
                if packet_flags & 0xfeff_0000 != 0 {
                    return Err(Error::PacketLength);
                }
                let fcs = (fcs_length == 4).then_some(true).or(interface.fcs);
                let timestamp = (u64::from(packet.ts_high) << 32) | u64::from(packet.ts_low);
                let _absolute_units = i128::from(interface.offset_seconds)
                    .checked_mul(i128::from(interface.units_per_second))
                    .and_then(|offset| offset.checked_add(i128::from(timestamp)))
                    .ok_or(Error::Timestamp)?;
                summary.packet_count += 1;
                packet_summary(
                    &packet.data[..packet.caplen as usize],
                    fcs,
                    packet_flags & (1 << 24) != 0,
                    &mut summary,
                    &mut consumer,
                    &mut frames,
                )?;
            }
            _ => return Err(Error::BlockType),
        }
        start = end;
    }
    framer.finish().map_err(|_| Error::Pcapng)?;
    if summary.packet_count == 0 {
        return Err(Error::NoPackets);
    }
    Ok(summary)
}

fn radiotap(bytes: &[u8]) -> Result<(usize, Option<u8>), Error> {
    if bytes.len() < 8 || bytes[0] != 0 {
        return Err(Error::Radiotap);
    }
    let length = usize::from(u16::from_le_bytes([bytes[2], bytes[3]]));
    if length < 8 || length > bytes.len() {
        return Err(Error::Radiotap);
    }
    let mut masks = [0_u32; 64];
    masks[0] = u32::from_le_bytes(bytes[4..8].try_into().expect("fixed header"));
    let mut fields = 8;
    let mut words = 1;
    while masks[words - 1] & (1 << 31) != 0 {
        if fields + 4 > length || words == 64 {
            return Err(Error::Radiotap);
        }
        masks[words] =
            u32::from_le_bytes(bytes[fields..fields + 4].try_into().expect("bitmap word"));
        fields += 4;
        words += 1;
    }
    // Declared standard field widths/alignments are checked before the frame is
    // located with it_len. Only Flags values are interpreted. Vendor namespaces
    // remain opaque within their declared skip length.
    const LAYOUT: [(usize, usize); 28] = [
        (8, 8),
        (1, 1),
        (1, 1),
        (2, 4),
        (2, 2),
        (1, 1),
        (1, 1),
        (2, 2),
        (2, 2),
        (2, 2),
        (1, 1),
        (1, 1),
        (1, 1),
        (1, 1),
        (2, 2),
        (2, 2),
        (1, 1),
        (1, 1),
        (4, 8),
        (1, 3),
        (4, 8),
        (2, 12),
        (8, 12),
        (2, 12),
        (2, 12),
        (2, 6),
        (1, 1),
        (2, 4),
    ];
    let mut flags = None;
    let mut standard = true;
    let mut base = 0;
    for mask in &masks[..words] {
        let mut reset = false;
        if mask & (1 << 29) != 0 && mask & (1 << 30) != 0 {
            return Err(Error::Radiotap);
        }
        for bit in 0..31 {
            if mask & (1 << bit) == 0 {
                continue;
            }
            if bit == 29 {
                standard = true;
                reset = true;
                continue;
            }
            if bit == 30 {
                fields = (fields + 1) & !1;
                if fields + 6 > length {
                    return Err(Error::Radiotap);
                }
                let skip = usize::from(u16::from_le_bytes([bytes[fields + 4], bytes[fields + 5]]));
                fields += 6 + skip;
                if fields > length {
                    return Err(Error::Radiotap);
                }
                standard = false;
                reset = true;
                continue;
            }
            if !standard {
                continue;
            }
            let field = base + bit;
            // TLV lists (field 28) and newer standard fields need a distinct
            // metadata decoder. They fail this profile rather than hiding a
            // later malformed TLV behind it_len.
            let (alignment, size) = *LAYOUT.get(field).ok_or(Error::Radiotap)?;
            fields = (fields + alignment - 1) & !(alignment - 1);
            if fields + size > length {
                return Err(Error::Radiotap);
            }
            if field == 1 {
                let value = bytes[fields];
                if flags.is_some_and(|previous| previous != value) {
                    return Err(Error::Radiotap);
                }
                flags = Some(value);
            }
            fields += size;
        }
        base = if reset { 0 } else { base + 32 };
    }
    Ok((length, flags))
}

fn packet_summary(
    bytes: &[u8],
    interface_fcs: Option<bool>,
    reported_bad_crc: bool,
    summary: &mut Summary,
    consumer: &mut Option<&mut dyn BeaconConsumer>,
    frames: &mut Option<&mut dyn FrameConsumer>,
) -> Result<(), Error> {
    let (length, flags) = radiotap(bytes)?;
    let flagged_fcs = flags.map(|flags| flags & 0x10 != 0);
    if interface_fcs.zip(flagged_fcs).is_some_and(|(a, b)| a != b) {
        return Err(Error::Radiotap);
    }
    let has_fcs = flagged_fcs.or(interface_fcs).unwrap_or(false);
    let mut frame = &bytes[length..];
    let mut invalid_fcs = reported_bad_crc || flags.is_some_and(|flags| flags & 0x40 != 0);
    let mut stored_fcs = None;
    if has_fcs {
        if frame.len() < 4 {
            return Err(Error::Radiotap);
        }
        let end = frame.len() - 4;
        stored_fcs = Some(u32::from_le_bytes(
            frame[end..].try_into().expect("FCS width"),
        ));
        frame = &frame[..end];
    } else {
        summary.fcs_unqualified_count += 1;
    }
    if invalid_fcs {
        summary.bad_fcs_count += 1;
        return Ok(());
    }
    if frame.len() < 2 {
        return Err(Error::Beacon);
    }
    let control = u16::from_le_bytes([frame[0], frame[1]]);
    let ordinary_beacon = control & 0xc0ff == 0x0080;
    if flags.is_some_and(|flags| flags & 0x20 != 0) && !ordinary_beacon {
        // Other headers can contain capture-only padding inside the FCS range.
        // Their on-wire reconstruction is outside this beacon decoder.
        if has_fcs {
            summary.fcs_unqualified_count += 1;
        }
        summary.unsupported_frame_count += 1;
        return Ok(());
    }
    invalid_fcs |= stored_fcs.is_some_and(|stored| crc32(frame) != stored);
    if invalid_fcs {
        summary.bad_fcs_count += 1;
        return Ok(());
    }
    if control & 3 != 0 || (control >> 2) & 3 == 3 {
        summary.unsupported_frame_count += 1;
        return Ok(());
    }
    if let Some(frames) = frames {
        frames.observe(frame, has_fcs)?;
    }
    if (control >> 2) & 3 != 0 {
        summary.other_frame_count += 1;
        return Ok(());
    }
    if frame.len() < 24 {
        return Err(Error::Beacon);
    }
    summary.management_frame_count += 1;
    if (control >> 4) & 15 != 8 {
        summary.other_frame_count += 1;
        return Ok(());
    }
    if control & 0xc000 != 0 {
        summary.unsupported_frame_count += 1;
        return Ok(());
    }
    if frame.len() < 36
        || control & 0x0700 != 0
        || frame[22] & 15 != 0
        || frame[4..10] != [0xff; 6]
        || frame[10] & 1 != 0
        || frame[10..16] == [0; 6]
        || frame[16] & 1 != 0
        || frame[16..22] == [0; 6]
    {
        return Err(Error::Beacon);
    }
    let capabilities = u16::from_le_bytes([frame[34], frame[35]]);
    if capabilities & 2 != 0 {
        // IBSS beacons have different transmitter/BSSID relationships.
        summary.unsupported_frame_count += 1;
        return Ok(());
    }
    if frame[10..16] != frame[16..22] {
        return Err(Error::Beacon);
    }
    // A 24-byte ordinary beacon header is already four-byte aligned, so the
    // radiotap data-padding flag adds no bytes to this supported header shape.
    let ssid = information_elements(&frame[36..], summary)?;
    summary.beacon_count += 1;
    if let Some(consumer) = consumer {
        consumer.observe(frame, ssid, has_fcs)?;
    }
    Ok(())
}

struct InformationElement<'a> {
    kind: u8,
    tlv: &'a [u8],
    value: &'a [u8],
}

struct InformationElements<'a>(&'a [u8]);

impl<'a> Iterator for InformationElements<'a> {
    type Item = Result<InformationElement<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        if self.0.len() < 2 || usize::from(self.0[1]) > self.0.len() - 2 {
            self.0 = &[];
            return Some(Err(Error::InformationElements));
        }
        let end = 2 + usize::from(self.0[1]);
        let (tlv, remaining) = self.0.split_at(end);
        self.0 = remaining;
        Some(Ok(InformationElement {
            kind: tlv[0],
            tlv,
            value: &tlv[2..],
        }))
    }
}

fn information_elements<'a>(bytes: &'a [u8], summary: &mut Summary) -> Result<&'a [u8], Error> {
    let mut ssid = None;
    let mut channel = false;
    let mut tim = false;
    let mut rsn = false;
    let mut vendor = false;
    for element in InformationElements(bytes) {
        let InformationElement { kind, value, .. } = element?;
        let length = value.len();
        match kind {
            0 => {
                if length > 32 || ssid.is_some() {
                    return Err(Error::InformationElements);
                }
                ssid = Some(value);
            }
            1 if !(1..=8).contains(&length) => return Err(Error::InformationElements),
            3 => {
                if length != 1 || channel {
                    return Err(Error::InformationElements);
                }
                channel = true;
            }
            5 => {
                if length < 4 || tim {
                    return Err(Error::InformationElements);
                }
                tim = true;
            }
            48 => {
                if rsn {
                    return Err(Error::InformationElements);
                }
                rsn = true;
            }
            221 => vendor = true,
            _ => {}
        }
    }
    let ssid = ssid.ok_or(Error::InformationElements)?;
    if ssid.iter().all(|byte| *byte == 0) {
        summary.hidden_ssid_beacon_count += 1;
    } else {
        summary.visible_ssid_beacon_count += 1;
    }
    summary.channel_ie_beacon_count += u64::from(channel);
    summary.tim_ie_beacon_count += u64::from(tim);
    summary.rsn_ie_beacon_count += u64::from(rsn);
    summary.vendor_ie_beacon_count += u64::from(vendor);
    Ok(ssid)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

#[cfg(test)]
mod tests;
