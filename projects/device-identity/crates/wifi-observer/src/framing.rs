//! Allocation-free PCAPNG block boundaries, shared with the private receiver worker.
//!
//! This module uses only Rust 1.90 standard-library APIs. It validates framing;
//! the owning analyzer must validate block semantics and packet contents.

use std::fmt;

/// A framing failure, with no captured bytes or interface identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// The configured block bound cannot accommodate a section header.
    InvalidLimit,
    /// The stream does not start with a valid byte-order section header.
    SectionHeader,
    /// A second section is outside this observation format.
    MultipleSections,
    /// A block length is unaligned, too short, or above the configured bound.
    BlockLength,
    /// The trailing block length differs from the leading length.
    LengthMismatch,
    /// The stream offset could not be represented.
    OffsetOverflow,
    /// EOF occurred before any complete section header.
    Empty,
    /// EOF occurred inside a block or its header.
    Partial,
    /// A previous framing failure prevents further admission.
    Failed,
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimit => "invalid PCAPNG block limit",
            Self::SectionHeader => "invalid PCAPNG section header",
            Self::MultipleSections => "multiple PCAPNG sections are unsupported",
            Self::BlockLength => "invalid or oversized PCAPNG block length",
            Self::LengthMismatch => "PCAPNG block lengths disagree",
            Self::OffsetOverflow => "PCAPNG stream offset overflow",
            Self::Empty => "missing complete PCAPNG section header",
            Self::Partial => "partial PCAPNG block at EOF",
            Self::Failed => "PCAPNG framing previously failed",
        })
    }
}

impl std::error::Error for FrameError {}

/// One complete block; offsets are exclusive byte counts from stream start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockBoundary {
    /// Number of bytes through the end of this block.
    pub end_offset: u64,
    /// Block type normalized to the section's byte order; EPBs have type six.
    pub block_type: u32,
}

#[derive(Clone, Copy, Debug)]
enum Endian {
    Little,
    Big,
}

impl Endian {
    fn read(self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }
}

/// Incremental, bounded, single-section framing with no retained packet data.
pub struct BlockFramer {
    maximum: usize,
    endian: Option<Endian>,
    header: [u8; 12],
    header_length: usize,
    block_length: usize,
    block_offset: usize,
    block_type: u32,
    trailer: [u8; 4],
    offset: u64,
    blocks: u64,
    failed: bool,
}

impl fmt::Debug for BlockFramer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockFramer")
            .field("offset", &self.offset)
            .field("blocks", &self.blocks)
            .field("at_boundary", &self.at_boundary())
            .field("failed", &self.failed)
            .finish_non_exhaustive()
    }
}

impl BlockFramer {
    /// Set an inclusive per-block bound, at least 28 and at most `u32::MAX`.
    pub fn new(max_block_bytes: usize) -> Result<Self, FrameError> {
        if max_block_bytes < 28 || u32::try_from(max_block_bytes).is_err() {
            return Err(FrameError::InvalidLimit);
        }
        Ok(Self {
            maximum: max_block_bytes,
            endian: None,
            header: [0; 12],
            header_length: 0,
            block_length: 0,
            block_offset: 0,
            block_type: 0,
            trailer: [0; 4],
            offset: 0,
            blocks: 0,
            failed: false,
        })
    }

    /// Consume at most through the next complete block and return its boundary.
    ///
    /// The returned count refers only to this input slice. Any remaining suffix
    /// has not been inspected: callers can stop exactly at a packet quota without
    /// treating later network-read bytes as part of the admitted capture. Retain
    /// all received bytes separately when preserving a failed transport attempt.
    /// A failure poisons this instance; it cannot resume with different bytes.
    pub fn push(&mut self, input: &[u8]) -> Result<(usize, Option<BlockBoundary>), FrameError> {
        if self.failed {
            return Err(FrameError::Failed);
        }
        let result = self.consume(input);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn consume(&mut self, input: &[u8]) -> Result<(usize, Option<BlockBoundary>), FrameError> {
        let mut consumed = 0;
        while consumed < input.len() {
            if self.block_length == 0 {
                let required = if self.endian.is_none() { 12 } else { 8 };
                let count = (required - self.header_length).min(input.len() - consumed);
                self.header[self.header_length..self.header_length + count]
                    .copy_from_slice(&input[consumed..consumed + count]);
                self.header_length += count;
                consumed += count;
                self.advance(count)?;
                if self.header_length < required {
                    continue;
                }
                let first = self.endian.is_none();
                let endian = if first {
                    if self.header[..4] != [0x0a, 0x0d, 0x0d, 0x0a] {
                        return Err(FrameError::SectionHeader);
                    }
                    match self.header[8..12] {
                        [0x4d, 0x3c, 0x2b, 0x1a] => Endian::Little,
                        [0x1a, 0x2b, 0x3c, 0x4d] => Endian::Big,
                        _ => return Err(FrameError::SectionHeader),
                    }
                } else {
                    self.endian.ok_or(FrameError::SectionHeader)?
                };
                let block_type = endian.read(self.header[..4].try_into().expect("four bytes"));
                if !first && block_type == 0x0a0d0d0a {
                    return Err(FrameError::MultipleSections);
                }
                let length = endian.read(self.header[4..8].try_into().expect("four bytes"));
                let minimum = match block_type {
                    0x0a0d0d0a => 28,
                    1 => 20,
                    6 => 32,
                    _ => 12,
                };
                let length = usize::try_from(length).map_err(|_| FrameError::BlockLength)?;
                if length < minimum || length > self.maximum || !length.is_multiple_of(4) {
                    return Err(FrameError::BlockLength);
                }
                self.endian = Some(endian);
                self.block_type = block_type;
                self.block_length = length;
                self.block_offset = required;
            }
            let count = (self.block_length - self.block_offset).min(input.len() - consumed);
            let trailer_start = self.block_length - 4;
            let overlap = self.block_offset.max(trailer_start);
            let end = self.block_offset + count;
            if overlap < end {
                let source = consumed + overlap - self.block_offset;
                self.trailer[overlap - trailer_start..end - trailer_start]
                    .copy_from_slice(&input[source..consumed + count]);
            }
            self.block_offset = end;
            consumed += count;
            self.advance(count)?;
            if self.block_offset == self.block_length {
                let trailing = self
                    .endian
                    .ok_or(FrameError::SectionHeader)?
                    .read(self.trailer);
                if usize::try_from(trailing).ok() != Some(self.block_length) {
                    return Err(FrameError::LengthMismatch);
                }
                let boundary = BlockBoundary {
                    end_offset: self.offset,
                    block_type: self.block_type,
                };
                self.blocks += 1;
                self.header_length = 0;
                self.block_length = 0;
                self.block_offset = 0;
                self.trailer = [0; 4];
                return Ok((consumed, Some(boundary)));
            }
        }
        Ok((consumed, None))
    }

    fn advance(&mut self, count: usize) -> Result<(), FrameError> {
        self.offset = self
            .offset
            .checked_add(count as u64)
            .ok_or(FrameError::OffsetOverflow)?;
        Ok(())
    }

    /// Whether no partial header or block remains buffered in the framing state.
    pub fn at_boundary(&self) -> bool {
        !self.failed && self.header_length == 0 && self.block_length == 0
    }

    /// Reject an empty stream, a retained framing failure, or partial EOF.
    pub fn finish(&self) -> Result<(), FrameError> {
        if self.failed {
            Err(FrameError::Failed)
        } else if !self.at_boundary() {
            Err(FrameError::Partial)
        } else if self.blocks == 0 {
            Err(FrameError::Empty)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(big: bool) -> Vec<u8> {
        let mut bytes = vec![0x0a, 0x0d, 0x0d, 0x0a];
        for value in [28_u32, 0x1a2b3c4d, 1, u32::MAX, u32::MAX, 28] {
            bytes.extend_from_slice(&if big {
                value.to_be_bytes()
            } else {
                value.to_le_bytes()
            });
        }
        bytes
    }

    #[test]
    fn fragmented_both_orders_stop_before_suffix() {
        for big in [false, true] {
            let bytes = section(big);
            let mut framer = BlockFramer::new(1024).unwrap();
            for byte in &bytes[..27] {
                assert_eq!(framer.push(&[*byte]).unwrap(), (1, None));
            }
            assert_eq!(framer.finish(), Err(FrameError::Partial));
            assert_eq!(
                framer.push(&[bytes[27], 0xff]).unwrap(),
                (
                    1,
                    Some(BlockBoundary {
                        end_offset: 28,
                        block_type: 0x0a0d0d0a,
                    })
                )
            );
            assert!(framer.at_boundary());
            assert_eq!(framer.finish(), Ok(()));
        }
    }

    #[test]
    fn invalid_lengths_trailers_and_extra_sections_fail_closed() {
        assert!(BlockFramer::new(27).is_err());
        for length in [0_u32, 12, 27, 30, 1028] {
            let mut bytes = section(false);
            bytes[4..8].copy_from_slice(&length.to_le_bytes());
            assert!(BlockFramer::new(1024).unwrap().push(&bytes).is_err());
        }
        let mut bytes = section(false);
        bytes[24] = 24;
        let mut framer = BlockFramer::new(1024).unwrap();
        assert_eq!(framer.push(&bytes), Err(FrameError::LengthMismatch));
        assert_eq!(framer.push(&section(false)), Err(FrameError::Failed));
        let mut framer = BlockFramer::new(1024).unwrap();
        framer.push(&section(false)).unwrap();
        assert_eq!(
            framer.push(&section(false)),
            Err(FrameError::MultipleSections)
        );
    }

    #[test]
    fn malformed_header_bytes_stay_out_of_debug() {
        let mut framer = BlockFramer::new(1024).unwrap();
        assert!(framer.push(b"PRIVATE_DATA").is_err());
        let debug = format!("{framer:?}");
        assert!(!debug.contains("PRIVATE_DATA"));
        assert!(!debug.contains("header"));
        assert!(!debug.contains("trailer"));
    }
}
