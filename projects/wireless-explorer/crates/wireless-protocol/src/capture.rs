//! Passive observation stream, separate from legacy control replies.
//!
//! One [`crate::Command::Observe`] produces Start, then Packet/Chunk pairs,
//! then End. A missing End is an incomplete capture, including an unplug/reset.
//! Packet bytes are hexadecimal; lengths always refer to decoded bytes.

use serde::{Deserialize, Serialize};

use crate::FrameError;

/// Maximum stream line length before LF. Legacy control stays at 512 bytes.
pub const MAX_LINE: usize = 1024;
/// Maximum retained bytes per SDK report; larger frames are explicitly truncated.
pub const SNAPLEN: usize = 4096;
/// Decoded bytes in each payload chunk.
pub const CHUNK_BYTES: usize = 128;
/// Stream discriminator.
pub const PREFIX: &[u8] = b"WC ";

/// Mutually exclusive receive mode for an observation window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Radio {
    /// Wi-Fi promiscuous frames on one 2.4 GHz channel.
    Wifi,
    /// Passive BLE extended advertising reports, including legacy advertisements.
    Ble,
    /// Raw IEEE 802.15.4 MAC frames on one channel, with no FCS bytes.
    Ieee802154,
}

impl Radio {
    /// Validate a bounded window; BLE channels are selected by the controller.
    pub const fn valid_window(self, channel: u8, millis: u32) -> bool {
        millis >= 100
            && millis <= 60_000
            && match self {
                Self::Wifi => channel >= 1 && channel <= 13,
                Self::Ble => channel == 0,
                Self::Ieee802154 => channel >= 11 && channel <= 26,
            }
    }
}

/// SDK-specific metadata. Numeric codes retain the pinned SDK's representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Details {
    /// Promiscuous receive metadata; no claim that every PHY frame is exposed.
    Wifi {
        /// SDK packet type (management/data/control/misc).
        kind: u8,
        /// SDK receive status; nonzero codes are not publicly specified.
        rx_state: u8,
        /// SDK's wrapping microsecond receive timestamp.
        sdk_time: u32,
        /// SDK PHY format code, not a rate in Mbps.
        phy: u8,
        /// SDK rate code; interpretation depends on PHY.
        rate: u8,
    },
    /// Extended scan report metadata. Data may be an SDK fragment.
    Ble {
        /// Advertiser address in SDK byte order; may be randomized/anonymous.
        address: [u8; 6],
        /// SDK advertiser address type.
        address_type: u8,
        /// Advertising event flags.
        event_type: u16,
        /// Primary advertising PHY code.
        primary_phy: u8,
        /// Secondary advertising PHY code.
        secondary_phy: u8,
        /// Advertising set identifier (255 means unavailable).
        sid: u8,
        /// SDK completeness/fragmentation status; never silently reassembled.
        data_status: u8,
        /// Signed transmit power in dBm, or SDK sentinel 127.
        tx_power: i8,
        /// Periodic advertisement interval in SDK units of 1.25 ms.
        periodic_interval: u16,
        /// Directed advertisement target address.
        direct_address: [u8; 6],
        /// SDK directed target address type.
        direct_address_type: u8,
    },
    /// IEEE 802.15.4 receive metadata; FCS was checked and removed by hardware.
    Ieee802154 {
        /// Link quality indicator.
        lqi: u8,
        /// SDK timestamp at reception of SFD, retained without clock conversion.
        sdk_time: u64,
    },
}

impl Details {
    /// Radio that produced these metadata.
    pub const fn radio(self) -> Radio {
        match self {
            Self::Wifi { .. } => Radio::Wifi,
            Self::Ble { .. } => Radio::Ble,
            Self::Ieee802154 { .. } => Radio::Ieee802154,
        }
    }
}

/// Metadata for exactly one callback report, preceding its payload chunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packet {
    /// Zero-based SDK report sequence, including reports dropped by our pool.
    pub seq: u32,
    /// Firmware monotonic microseconds since boot, sampled in the callback.
    pub at_us: u64,
    /// Actual receive channel, or zero when BLE does not report it.
    pub channel: u8,
    /// Receive signal strength in dBm (BLE 127 means unavailable).
    pub rssi: i8,
    /// SDK-reported frame/data length; 802.15.4 excludes removed FCS.
    pub original_len: u16,
    /// Number of retained bytes that follow; may be zero for Wi-Fi misc reports.
    pub captured_len: u16,
    /// Radio-specific fields.
    pub details: Details,
}

/// Events for a complete bounded window. Counters count SDK callbacks, not airtime.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
// Inline chunk storage keeps this no_std stream allocation-free and bounded.
#[allow(clippy::large_enum_variant)]
pub enum Event {
    /// Settings accepted, immediately before enabling the receive mode.
    Start {
        /// Radio mode.
        radio: Radio,
        /// Requested channel, zero for BLE.
        channel: u8,
        /// Requested listening budget after initialization, in milliseconds.
        millis: u32,
        /// Monotonic start of this window, before radio setup.
        at_us: u64,
        /// Maximum payload bytes retained per callback.
        snaplen: u16,
    },
    /// The SDK has confirmed receive mode; the requested dwell starts here.
    Listening {
        /// Firmware monotonic timestamp after SDK setup/confirmation.
        at_us: u64,
    },
    /// Metadata for a packet. The next chunks must fill `captured_len` exactly.
    Packet(Packet),
    /// Lowercase hexadecimal payload fragment. No interleaving of packets.
    Chunk {
        /// Packet sequence number.
        seq: u32,
        /// Decoded byte offset in that packet.
        offset: u16,
        /// At most 128 bytes encoded as 256 hexadecimal characters.
        data: heapless::String<256>,
    },
    /// Radio stopped and all accepted reports drained. A nonzero error fails.
    End {
        /// Firmware monotonic time after radio shutdown and queue drain.
        at_us: u64,
        /// Monotonic time after receive shutdown, before draining queued bytes.
        stopped_us: u64,
        /// Callback reports admitted while the capture gate was open.
        seen: u32,
        /// Reports emitted with every chunk.
        emitted: u32,
        /// Reports dropped because the slots were full or a producer was busy.
        dropped: u32,
        /// Invalid SDK reports. Buffer-release failures set `error` separately.
        invalid: u32,
        /// Emitted reports whose payload exceeded the snapshot cap.
        truncated: u32,
        /// SDK or internal operation status, zero on success.
        error: i32,
    },
}

/// Versioned event correlated to one Observe request.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    version: u8,
    /// Observe request ID.
    pub id: u32,
    /// Capture event.
    pub event: Event,
}

impl Record {
    /// Construct stream revision 1; separate from the control protocol version.
    pub const fn new(id: u32, event: Event) -> Self {
        Self {
            version: 1,
            id,
            event,
        }
    }
}

/// Encode a stream record with `WC ` and LF into bounded caller storage.
///
/// # Errors
/// Returns capacity failure without a usable output frame on overflow.
pub fn encode(record: &Record, output: &mut [u8]) -> Result<usize, FrameError> {
    let cap = output.len().min(MAX_LINE + 1);
    if cap < 5 {
        return Err(FrameError::Capacity);
    }
    output[..3].copy_from_slice(PREFIX);
    let n = serde_json_core::to_slice(record, &mut output[3..cap - 1])
        .map_err(|_| FrameError::Capacity)?
        + 3;
    output[n] = b'\n';
    Ok(n + 1)
}

/// Decode one stream line without LF, rejecting trailing junk and other versions.
///
/// # Errors
/// Returns framing, capacity, or version failure for invalid input.
pub fn decode(line: &[u8]) -> Result<Record, FrameError> {
    if line.len() > MAX_LINE {
        return Err(FrameError::Capacity);
    }
    let bytes = line.strip_prefix(PREFIX).ok_or(FrameError::Malformed)?;
    let (record, used): (Record, usize) =
        serde_json_core::from_slice(bytes).map_err(|_| FrameError::Malformed)?;
    if !bytes[used..].iter().all(u8::is_ascii_whitespace) {
        return Err(FrameError::Malformed);
    }
    if record.version != 1 {
        return Err(FrameError::Version);
    }
    Ok(record)
}

/// Encode up to one chunk of bytes as hexadecimal without heap allocation.
///
/// # Errors
/// Rejects more than [`CHUNK_BYTES`] bytes.
pub fn hex(bytes: &[u8]) -> Result<heapless::String<256>, FrameError> {
    let mut out = heapless::String::new();
    for &b in bytes {
        for n in [b >> 4, b & 15] {
            out.push(char::from(b"0123456789abcdef"[usize::from(n)]))
                .map_err(|_| FrameError::Capacity)?;
        }
    }
    Ok(out)
}
