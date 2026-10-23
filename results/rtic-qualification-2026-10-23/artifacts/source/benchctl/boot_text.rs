//! Single-boot admission and bounded framing shared by capture and replay.
//!
//! Device JSON can follow all measurements, so diagnostics before the first
//! record are evidence, not disposable boot noise. Known fatal/reboot markers,
//! ambiguous startup and any later boot permanently reject this stream.

use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};

pub(crate) const MAX_LINE_BYTES: usize = 4096;

#[derive(Debug, Default)]
pub(crate) struct BootTextGuard {
    rom_seen: bool,
    reset_reason_seen: bool,
    protocol_started: bool,
}

impl BootTextGuard {
    pub(crate) fn observe(&mut self, line: &[u8]) -> Result<()> {
        let line = line.trim_ascii();
        if line.starts_with(b"{") {
            ensure!(self.rom_seen, "protocol record preceded the C6 ROM startup");
            self.protocol_started = true;
            return Ok(());
        }
        let text = String::from_utf8_lossy(line).to_ascii_lowercase();
        for marker in [
            "guru meditation",
            "panic",
            "rebooting",
            "assertion failed",
            "assertion fail",
            "assert failed",
            "abort()",
            "abort was called",
            "fatal error",
            "stack canary",
            "stack smashing",
            "stack protection fault",
            "brownout detector",
            "task watchdog",
            "interrupt wdt timeout",
            "watchdog timeout",
            "sha-256 comparison failed",
            "exception '",
            "stack overflow detected",
        ] {
            ensure!(
                !text.contains(marker),
                "fatal or reboot diagnostic in capture: {marker}"
            );
        }
        if text.contains("esp-rom:") {
            ensure!(
                text == "esp-rom:esp32c6"
                    || text.starts_with("esp-rom:esp32c6-")
                    || text.starts_with("esp-rom:esp32c6 "),
                "unrecognized or ambiguous ROM startup"
            );
            ensure!(
                !self.rom_seen && !self.protocol_started,
                "additional ROM startup in the same attempt"
            );
            self.rom_seen = true;
        }
        if text.starts_with("rst:") {
            ensure!(self.rom_seen, "reset reason preceded the ROM startup");
            ensure!(
                !self.reset_reason_seen && !self.protocol_started,
                "additional reset reason in the same attempt"
            );
            self.reset_reason_seen = true;
        }
        Ok(())
    }

    /// Marks a recognized plain-text application record without inventing JSON.
    pub(crate) fn start_console(&mut self) -> Result<()> {
        ensure!(self.rom_seen, "console marker preceded the C6 ROM startup");
        self.protocol_started = true;
        Ok(())
    }

    pub(crate) fn finish(&self) -> Result<()> {
        ensure!(self.rom_seen, "missing C6 ROM startup");
        ensure!(self.protocol_started, "missing device protocol records");
        Ok(())
    }
}

/// Framing state for one attempt; a failed observation cannot be resumed.
#[derive(Debug, Default)]
struct Stream {
    text: BootTextGuard,
    line: Vec<u8>,
    complete: bool,
    rejected: bool,
}

impl Stream {
    fn accept_line(&mut self, accept: &mut impl FnMut(&[u8]) -> Result<bool>) -> Result<()> {
        self.text.observe(&self.line)?;
        self.complete |= accept(&self.line)?;
        self.line.clear();
        Ok(())
    }

    /// Consumes every byte in a received chunk, including trailing diagnostics.
    fn push(
        &mut self,
        chunk: &[u8],
        accept: &mut impl FnMut(&[u8]) -> Result<bool>,
    ) -> Result<bool> {
        ensure!(!self.rejected, "capture stream was already rejected");
        let result = (|| {
            for &byte in chunk {
                if byte == b'\n' {
                    self.accept_line(accept)?;
                } else {
                    ensure!(
                        self.line.len() < MAX_LINE_BYTES,
                        "serial line exceeds protocol limit"
                    );
                    self.line.push(byte);
                }
            }
            if self.complete {
                self.text.finish()?;
            }
            // A partial line already received after completion still belongs
            // to this observation. Continue until its newline/EOF/timeout;
            // otherwise read segmentation could hide a split fatal diagnostic.
            Ok(self.complete && self.line.trim_ascii().is_empty())
        })();
        if result.is_err() {
            self.rejected = true;
        }
        result
    }

    fn finish(&mut self, accept: &mut impl FnMut(&[u8]) -> Result<bool>) -> Result<()> {
        ensure!(!self.rejected, "capture stream was already rejected");
        let result = (|| {
            ensure!(
                !self.complete || self.line.trim_ascii().is_empty(),
                "unterminated trailing text after completion"
            );
            if !self.line.is_empty() {
                self.accept_line(accept)?;
            }
            self.text.finish()?;
            ensure!(self.complete, "incomplete device protocol");
            Ok(())
        })();
        if result.is_err() {
            self.rejected = true;
        }
        result
    }
}

/// Reads a complete archived stream through the same framing/boot guard as capture.
///
/// Rejects missing/extra startups, known fatal diagnostics, framing/record errors
/// and incomplete input. EOF does not discard an unterminated final line.
pub(crate) fn replay(
    mut input: impl Read,
    mut accept: impl FnMut(&[u8]) -> Result<bool>,
) -> Result<()> {
    let mut stream = Stream::default();
    let mut chunk = [0_u8; 1024];
    loop {
        let count = match input.read(&mut chunk) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error).context("reading archived capture"),
        };
        if count == 0 {
            break;
        }
        stream.push(&chunk[..count], &mut accept)?;
    }
    stream.finish(&mut accept)
}

/// Consumes serial reads after reset, preserving each whole chunk before admission.
///
/// Returns an error for rejected text/records, I/O failures or expiry. It stops
/// after validated completion and the remainder of that received chunk; it cannot
/// certify diagnostics that arrive after this bounded observation has returned.
pub(crate) fn capture(
    input: &mut impl Read,
    raw: &mut impl Write,
    timeout: Duration,
    mut accept: impl FnMut(&[u8]) -> Result<bool>,
) -> Result<()> {
    ensure!(!timeout.is_zero(), "capture timeout must be positive");
    let start = Instant::now();
    let mut stream = Stream::default();
    let mut chunk = [0_u8; 1024];
    while start.elapsed() < timeout {
        match input.read(&mut chunk) {
            Ok(0) => return stream.finish(&mut accept),
            Ok(count) => {
                raw.write_all(&chunk[..count])?;
                if stream.push(&chunk[..count], &mut accept)? {
                    return Ok(());
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error).context("reading target output"),
        }
    }
    anyhow::bail!("timed out before complete valid boot; partial evidence retained")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_start_requires_rom_and_closes_reset_reason_admission() {
        let mut missing_rom = BootTextGuard::default();
        missing_rom.observe(b"init").unwrap();
        assert!(missing_rom.start_console().is_err());

        let mut console = BootTextGuard::default();
        console.observe(b"ESP-ROM:esp32c6-20220919").unwrap();
        console.observe(b"rst:0x1").unwrap();
        console.observe(b"init").unwrap();
        console.start_console().unwrap();
        console.finish().unwrap();
        assert!(console.observe(b"rst:0xc").is_err());
        assert!(console.observe(b"ESP-ROM:esp32c6-20220919").is_err());
    }

    fn records(line: &[u8]) -> Result<bool> {
        Ok(line.trim_ascii() == b"{complete}")
    }

    #[test]
    fn normal_startup_and_json_configuration_are_not_fatal_diagnostics() {
        let bytes = b"ordinary startup noise\r\nESP-ROM:esp32c6-20220919\r\nrst:0x1 (POWERON)\r\nI (1) boot: loading app\n{\"panic\":\"abort\"}\n{complete}\n";
        replay(io::Cursor::new(bytes), records).unwrap();
        let mut raw = Vec::new();
        capture(
            &mut io::Cursor::new(bytes),
            &mut raw,
            Duration::from_secs(1),
            records,
        )
        .unwrap();
        assert_eq!(raw, bytes);
    }

    #[test]
    fn failures_before_first_record_are_permanent_even_if_valid_records_follow() {
        for fault in [
            "Guru Meditation Error: Core 0 panic'ed",
            "thread 'main' panicked at x.rs:7",
            "Rebooting...",
            "assertion failed: x",
            "abort() was called at PC 0x4000",
            "Task watchdog got triggered",
            "SHA-256 comparison failed:",
            "Exception 'Illegal instruction' mepc=0x42000020",
            "Stack overflow detected at 0x40800000",
            "ESP-ROM:esp32c6-20220919",
            "rst:0xc (SW_CPU)",
        ] {
            let prefix = b"ESP-ROM:esp32c6-20220919\nrst:0x1\n";
            let suffix = format!("{fault}\n{{complete}}\n");
            let bytes = [prefix.as_slice(), suffix.as_bytes()].concat();
            assert!(replay(io::Cursor::new(&bytes), records).is_err(), "{fault}");
            let mut raw = Vec::new();
            assert!(
                capture(
                    &mut io::Cursor::new(&bytes),
                    &mut raw,
                    Duration::from_secs(1),
                    records
                )
                .is_err(),
                "{fault}"
            );
            assert_eq!(raw, bytes, "entire received chunk is retained for {fault}");
            let mut stream = Stream::default();
            assert!(stream.push(&bytes, &mut records).is_err());
            assert!(stream.push(b"{complete}\n", &mut records).is_err());
            assert!(stream.finish(&mut records).is_err());
        }
    }

    #[derive(Debug)]
    struct Fragmented<'a> {
        bytes: &'a [u8],
        width: usize,
    }

    impl Read for Fragmented<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = self.bytes.len().min(self.width).min(output.len());
            output[..count].copy_from_slice(&self.bytes[..count]);
            self.bytes = &self.bytes[count..];
            Ok(count)
        }
    }

    #[test]
    fn split_diagnostics_and_additional_boots_are_not_lost_at_read_boundaries() {
        for fault in [
            "panic: bad state",
            "Exception 'Illegal instruction' mepc=0x42000020",
            "Stack overflow detected at 0x40800000",
            "Rebooting...",
            "ESP-ROM:esp32c6",
            "rst:0xc",
        ] {
            let bytes = format!("ESP-ROM:esp32c6\nrst:0x1\n{{hello}}\n{fault}\n{{complete}}\n");
            for width in [1, 2, 7, 1024] {
                let mut reader = Fragmented {
                    bytes: bytes.as_bytes(),
                    width,
                };
                let mut raw = Vec::new();
                assert!(
                    capture(&mut reader, &mut raw, Duration::from_secs(1), records).is_err(),
                    "{fault}, width {width}"
                );
                assert!(
                    replay(
                        Fragmented {
                            bytes: bytes.as_bytes(),
                            width
                        },
                        records
                    )
                    .is_err()
                );
            }
        }
    }

    #[test]
    fn a_pending_post_completion_line_is_not_decided_by_read_segmentation() {
        let first = b"ESP-ROM:esp32c6\n{complete}\nI (2) boot: ";
        for suffix in [
            b"ordinary status\n".as_slice(),
            b"panic: late failure\n".as_slice(),
        ] {
            let should_pass = suffix.starts_with(b"ordinary");
            let mut stream = Stream::default();
            assert!(!stream.push(first, &mut records).unwrap());
            assert_eq!(stream.push(suffix, &mut records).is_ok(), should_pass);
            let all = [first.as_slice(), suffix].concat();
            assert_eq!(replay(io::Cursor::new(all), records).is_ok(), should_pass);
            let mut reader = io::Cursor::new(first).chain(io::Cursor::new(suffix));
            assert_eq!(
                capture(
                    &mut reader,
                    &mut Vec::new(),
                    Duration::from_secs(1),
                    records
                )
                .is_ok(),
                should_pass
            );
        }
        assert!(replay(io::Cursor::new(b"ESP-ROM:esp32c61\n{complete}\n"), records).is_err());
    }

    #[test]
    fn trailing_faults_and_partial_prefixes_in_received_chunks_reject_completion() {
        for suffix in [
            "panic: late\n",
            "Rebooting...",
            "ESP-ROM:esp32c6\n",
            "rst:0xc\n",
            "Gu",
            "{",
            "ordinary partial",
        ] {
            let bytes = format!("ESP-ROM:esp32c6\n{{complete}}\n{suffix}");
            assert!(
                replay(io::Cursor::new(&bytes), records).is_err(),
                "{suffix}"
            );
            assert!(
                capture(
                    &mut io::Cursor::new(&bytes),
                    &mut Vec::new(),
                    Duration::from_secs(1),
                    records
                )
                .is_err(),
                "{suffix}"
            );
        }
        assert!(replay(io::Cursor::new(b"{complete}\n"), records).is_err());
        assert!(replay(io::Cursor::new(b"ESP-ROM:esp32c3\n{complete}\n"), records).is_err());
    }
}
