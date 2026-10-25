//! Allocation-free, versioned control records for the wireless explorer.
//!
//! Frames are `WX `, a JSON object, and LF. [`LineDecoder`] accepts arbitrarily
//! fragmented input; oversized lines are discarded through the next LF.
//! This is a stop-and-wait control protocol, not a radio packet stream.
//!
//! ```
//! use wireless_protocol::{Command, Request, MAX_LINE_BYTES, encode, decode_request};
//! let request = Request::new(42, Command::Ping { nonce: 123 });
//! let mut bytes = [0; MAX_LINE_BYTES + 1];
//! let len = encode(&request, &mut bytes)?;
//! assert_eq!(decode_request(&bytes[..len - 1])?, request);
//! # Ok::<(), wireless_protocol::FrameError>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;
use serde::{Deserialize, Serialize};

pub mod capture;

/// Current wire version; unknown versions are rejected, not negotiated.
pub const WIRE_VERSION: u8 = 1;
/// Maximum line size including the prefix, excluding the terminating LF.
pub const MAX_LINE_BYTES: usize = 512;
/// Distinguishes protocol frames from startup diagnostics.
pub const PREFIX: &[u8] = b"WX ";

/// A single operation. Wi-Fi discovery is passive and never joins a network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    /// Read implemented functionality and the responding port.
    Info,
    /// Echo a caller-selected challenge without touching a radio.
    Ping {
        /// Challenge echoed in the reply; not an authentication token.
        nonce: u32,
    },
    /// Count access points by channel, with at most 32 summarized records.
    WifiScan,
    /// Record one passive radio window; replies use the separate `WC ` stream.
    Observe {
        /// Radio to observe exclusively for this window.
        radio: capture::Radio,
        /// Wi-Fi 1–13, IEEE 802.15.4 11–26, or zero for BLE controller scanning.
        channel: u8,
        /// Requested listening time in milliseconds, from 100 through 60,000.
        millis: u32,
    },
}

/// A request identity and operation. IDs correlate replies; they are not secrets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    version: u8,
    id: u32,
    command: Command,
}

impl Request {
    /// Construct a request using the supported wire version.
    pub const fn new(id: u32, command: Command) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            command,
        }
    }

    /// Correlation ID selected by the caller.
    pub const fn id(&self) -> u32 {
        self.id
    }

    /// Requested operation.
    pub const fn command(&self) -> Command {
        self.command
    }
}

/// Physical serial interface that received the request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    /// CP2102N bridge, UART0 GPIO16/17, 8N1; baud depends on firmware revision.
    Uart,
    /// Fixed USB Serial/JTAG controller, GPIO12/13.
    NativeUsb,
}

/// Stage at which firmware rejected an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureStage {
    /// Malformed, oversized, or unsupported request; correlation ID is zero.
    Protocol,
    /// SDK Wi-Fi initialization, scan, retrieval, or stop failure.
    Wifi,
    /// Rejected passive capture settings.
    Capture,
}

/// Response data. Capability flags mean implemented commands, not silicon support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Reply {
    /// Firmware configuration; board identity is configured, not autodetected.
    Info {
        /// Responding interface.
        transport: Transport,
        /// Firmware contract revision, separate from wire version.
        firmware_revision: u16,
        /// Passive Wi-Fi scan command is present.
        wifi_scan_implemented: bool,
        /// BLE commands are present.
        ble_implemented: bool,
        /// IEEE 802.15.4 commands are present.
        ieee802154_implemented: bool,
        /// Rejected request lines on this port since startup (saturating).
        protocol_errors: u32,
        /// Failed reply writes on this port since startup (saturating).
        tx_failures: u32,
    },
    /// Successful challenge echo.
    Pong {
        /// Exact request challenge.
        nonce: u32,
    },
    /// Aggregate survey; identifiers and SSIDs are never returned.
    WifiScan {
        /// SDK's total found count, before the 32-record cap and UTF-8 filtering.
        total_found: u16,
        /// Records actually included in channel counts.
        summarized: u16,
        /// Total minus summarized, including cap and upstream decode omissions.
        unreported: u16,
        /// Counts for channels 1 through 14, respectively, among summarized APs.
        channels: [u16; 14],
        /// Summarized records outside channels 1 through 14.
        out_of_range: u16,
    },
    /// Operation failed; the host must not turn this into a passing observation.
    Error {
        /// Failing subsystem.
        stage: FailureStage,
        /// SDK error integer, or -1 for a protocol rejection.
        code: i32,
    },
}

/// One reply correlated to a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    version: u8,
    id: u32,
    reply: Reply,
}

impl Response {
    /// Construct a versioned reply.
    pub const fn new(id: u32, reply: Reply) -> Self {
        Self {
            version: WIRE_VERSION,
            id,
            reply,
        }
    }

    /// Correlation ID (zero for an unparseable request).
    pub const fn id(&self) -> u32 {
        self.id
    }

    /// Operation outcome.
    pub const fn reply(&self) -> Reply {
        self.reply
    }

    /// Validate correlation, reply kind, challenge, and scan count conservation.
    ///
    /// # Errors
    /// Returns [`FrameError::Mismatch`] for inconsistent replies, and
    /// [`FrameError::Device`] for an explicit device error.
    pub fn validate_for(&self, request: &Request) -> Result<(), FrameError> {
        if self.version != WIRE_VERSION || request.version != WIRE_VERSION {
            return Err(FrameError::Version);
        }
        if self.id != request.id {
            return Err(FrameError::Mismatch);
        }
        let valid = match (request.command, self.reply) {
            (_, Reply::Error { stage, code }) => return Err(FrameError::Device { stage, code }),
            (Command::Info, Reply::Info { .. }) => true,
            (Command::Ping { nonce: expected }, Reply::Pong { nonce }) => nonce == expected,
            (
                Command::WifiScan,
                Reply::WifiScan {
                    total_found,
                    summarized,
                    unreported,
                    channels,
                    out_of_range,
                },
            ) => {
                let sum = channels.iter().map(|&n| u32::from(n)).sum::<u32>();
                summarized <= 32
                    && u32::from(summarized) + u32::from(unreported) == u32::from(total_found)
                    && sum + u32::from(out_of_range) == u32::from(summarized)
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(FrameError::Mismatch)
        }
    }
}

/// Framing or semantic failure; all normal input failures use this error type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameError {
    /// Input exceeds the line cap, or the output buffer is too small.
    Capacity,
    /// Prefix, JSON, fields, or trailing bytes are invalid.
    Malformed,
    /// Wire version is not supported.
    Version,
    /// Reply does not satisfy the request or count invariants.
    Mismatch,
    /// Device returned an explicit failure, preserving its original code.
    Device {
        /// Failing device stage.
        stage: FailureStage,
        /// Original integer status.
        code: i32,
    },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capacity => f.write_str("frame capacity exceeded"),
            Self::Malformed => f.write_str("malformed frame"),
            Self::Version => f.write_str("unsupported wire version"),
            Self::Mismatch => f.write_str("reply does not match request"),
            Self::Device { stage, code } => write!(f, "device {stage:?} failure: {code}"),
        }
    }
}
impl core::error::Error for FrameError {}

/// Encode one record into caller-owned storage, including its prefix and LF.
///
/// # Errors
/// Returns [`FrameError::Capacity`] if serialization exceeds the buffer or cap.
/// The output is unspecified on error; never transmit it in that case.
pub fn encode<T: Serialize>(value: &T, output: &mut [u8]) -> Result<usize, FrameError> {
    let capacity = output.len().min(MAX_LINE_BYTES + 1);
    if capacity < PREFIX.len() + 2 {
        return Err(FrameError::Capacity);
    }
    output[..PREFIX.len()].copy_from_slice(PREFIX);
    let len = serde_json_core::to_slice(value, &mut output[PREFIX.len()..capacity - 1])
        .map_err(|_| FrameError::Capacity)?
        + PREFIX.len();
    output[len] = b'\n';
    Ok(len + 1)
}

fn decode<T: for<'de> Deserialize<'de>>(line: &[u8]) -> Result<T, FrameError> {
    if line.len() > MAX_LINE_BYTES {
        return Err(FrameError::Capacity);
    }
    let json = line.strip_prefix(PREFIX).ok_or(FrameError::Malformed)?;
    let (value, used) = serde_json_core::from_slice(json).map_err(|_| FrameError::Malformed)?;
    if !json[used..].iter().all(u8::is_ascii_whitespace) {
        return Err(FrameError::Malformed);
    }
    Ok(value)
}

/// Decode a request line without its LF, rejecting unknown fields and versions.
///
/// # Errors
/// Returns a framing, JSON, capacity, or version error for invalid input.
pub fn decode_request(line: &[u8]) -> Result<Request, FrameError> {
    let request: Request = decode(line)?;
    if request.version != WIRE_VERSION {
        return Err(FrameError::Version);
    }
    Ok(request)
}

/// Decode a response line without its LF. Call [`Response::validate_for`] next.
///
/// # Errors
/// Returns a framing, JSON, capacity, or version error for invalid input.
pub fn decode_response(line: &[u8]) -> Result<Response, FrameError> {
    let response: Response = decode(line)?;
    if response.version != WIRE_VERSION {
        return Err(FrameError::Version);
    }
    Ok(response)
}

/// Fixed storage for a partial line; not a queue and never allocates.
#[derive(Debug)]
pub struct LineDecoder {
    bytes: [u8; MAX_LINE_BYTES],
    len: usize,
    overflow: bool,
}

impl Default for LineDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl LineDecoder {
    /// Construct an empty decoder.
    pub const fn new() -> Self {
        Self {
            bytes: [0; MAX_LINE_BYTES],
            len: 0,
            overflow: false,
        }
    }

    /// Whether an incomplete line or discarded overflow remains.
    pub const fn is_pending(&self) -> bool {
        self.len != 0 || self.overflow
    }

    /// Consume one byte. LF completes a nonempty line or reports its overflow.
    ///
    /// Empty lines are ignored. Returned storage is borrowed until the next call.
    /// Overflow discards the entire line, including any valid-looking suffix.
    pub fn push(&mut self, byte: u8) -> Option<Result<&[u8], FrameError>> {
        if byte == b'\n' {
            let len = self.len;
            self.len = 0;
            if core::mem::take(&mut self.overflow) {
                return Some(Err(FrameError::Capacity));
            }
            return (len != 0).then_some(Ok(&self.bytes[..len]));
        }
        if !self.overflow {
            if self.len == self.bytes.len() {
                self.overflow = true;
            } else {
                self.bytes[self.len] = byte;
                self.len += 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;
