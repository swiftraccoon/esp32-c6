//! Admit an empty startup interval or one complete, narrowly recognized C6 ROM boot.
//!
//! Input is private even though the recognized record contains only ROM/image
//! metadata. Reports and diagnostics never retain input-provided strings.

use serde::{Deserialize, Serialize};

/// Maximum retained startup interval size, including every line terminator.
pub(crate) const MAX_STARTUP_BYTES: usize = 8192;
/// Maximum physical line size, including its LF and optional CR.
pub(crate) const MAX_STARTUP_LINE_BYTES: usize = 256;
const MAX_LOAD_LINES: usize = 8;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: u32,
    byte_count: usize,
    line_count: usize,
    rom_boot_observed: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Stage {
    Header,
    Build,
    Reset,
    SavedPcOrSpi,
    Spi,
    Mode,
    Loads,
    Complete,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Date {
    year: u32,
    month: u32,
    day: u32,
}

fn decimal(value: &str, min_digits: usize, max_digits: usize) -> Option<u32> {
    if !(min_digits..=max_digits).contains(&value.len())
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    value.parse().ok()
}

fn hexadecimal(value: &str, min_digits: usize, max_digits: usize) -> Option<u32> {
    let digits = value.strip_prefix("0x")?;
    if !(min_digits..=max_digits).contains(&digits.len())
        || !digits.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    u32::from_str_radix(digits, 16).ok()
}

fn date(year: u32, month: u32, day: u32) -> Option<Date> {
    if year == 0 || year > 9999 || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days).contains(&day) {
        return None;
    }
    Some(Date { year, month, day })
}

fn header(line: &str) -> Option<Date> {
    let suffix = line.strip_prefix("ESP-ROM:esp32c6-")?;
    if suffix.len() != 8 || !suffix.is_ascii() {
        return None;
    }
    date(
        decimal(&suffix[..4], 4, 4)?,
        decimal(&suffix[4..6], 2, 2)?,
        decimal(&suffix[6..], 2, 2)?,
    )
}

fn build(line: &str) -> Option<Date> {
    let suffix = line.strip_prefix("Build:")?;
    // C __DATE__ has a three-letter month and a space-padded two-column day.
    if suffix.len() != 11 || !suffix.is_ascii() {
        return None;
    }
    if suffix.as_bytes()[3] != b' ' || suffix.as_bytes()[6] != b' ' {
        return None;
    }
    let month = match &suffix[..3] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let day = &suffix[4..6];
    let day = if let Some(single) = day.strip_prefix(' ') {
        decimal(single, 1, 1)?
    } else {
        decimal(day, 2, 2)?
    };
    date(decimal(&suffix[7..], 4, 4)?, month, day)
}

fn reset(line: &str) -> bool {
    let Some((reason, boot)) = line
        .strip_prefix("rst:")
        .and_then(|s| s.split_once(",boot:"))
    else {
        return false;
    };
    // These two reset classes are the prospective method's power/reset paths.
    // Their values match pinned C6 rom/rtc.h, not an arbitrary reset label.
    let reason_valid = reason
        .strip_suffix(" (POWERON)")
        .is_some_and(|value| hexadecimal(value, 1, 2) == Some(1))
        || reason
            .strip_suffix(" (USB_UART_CHIP_RESET)")
            .is_some_and(|value| hexadecimal(value, 1, 2) == Some(21));
    let boot_valid = boot
        .strip_suffix(" (SPI_FAST_FLASH_BOOT)")
        .is_some_and(|value| hexadecimal(value, 1, 2) == Some(12));
    reason_valid && boot_valid
}

fn saved_pc(line: &str) -> bool {
    line.strip_prefix("Saved PC:")
        .is_some_and(|value| hexadecimal(value, 8, 8).is_some())
}

fn spi(line: &str) -> bool {
    line.strip_prefix("SPIWP:")
        .is_some_and(|value| hexadecimal(value, 1, 2).is_some())
}

fn mode(line: &str) -> bool {
    line.strip_prefix("mode:DIO, clock div:")
        .and_then(|value| decimal(value, 1, 3))
        .is_some_and(|value| (1..=255).contains(&value))
}

fn load(line: &str) -> bool {
    let Some((address, length)) = line
        .strip_prefix("load:")
        .and_then(|s| s.split_once(",len:"))
    else {
        return false;
    };
    match (hexadecimal(address, 8, 8), hexadecimal(length, 1, 8)) {
        (Some(address), Some(length)) => length != 0 && address.checked_add(length).is_some(),
        _ => false,
    }
}

fn entry(line: &str) -> bool {
    line.strip_prefix("entry ")
        .is_some_and(|value| hexadecimal(value, 8, 8).is_some())
}

fn diagnostic(bytes: &[u8]) -> bool {
    [
        b"panic".as_slice(),
        b"abort",
        b"assert",
        b"error",
        b"failed",
    ]
    .iter()
    .any(|word| {
        bytes
            .windows(word.len())
            .any(|window| window.eq_ignore_ascii_case(word))
    })
}

/// Validate an empty/blank interval or one complete C6 ROM flash-boot record.
///
/// Nonblank records must appear in order: ROM header, matching build date,
/// recognized reset/flash boot, optional saved PC, SPIWP, DIO clock divider,
/// one through eight load records, entry. CRLF is accepted; a partial final
/// line, incomplete block, repeated boot, diagnostic or unknown text fails.
/// Counts include all raw bytes and physical LF-terminated lines. No raw field
/// or address reaches the report or error.
pub(crate) fn summarize(bytes: &[u8]) -> Result<Report, &'static str> {
    if bytes.len() > MAX_STARTUP_BYTES {
        return Err("startup input exceeds byte limit");
    }
    if !bytes.is_ascii() || diagnostic(bytes) {
        return Err("startup contains unsupported text or diagnostic output");
    }
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return Err("startup ends with an incomplete line");
    }
    let mut stage = Stage::Header;
    let mut rom_date = None;
    let mut loads = 0;
    let mut line_count = 0;
    for raw in bytes.split_inclusive(|&byte| byte == b'\n') {
        if raw.len() > MAX_STARTUP_LINE_BYTES {
            return Err("startup line exceeds byte limit");
        }
        line_count += 1;
        let line = raw
            .strip_suffix(b"\n")
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
            .ok_or("startup ends with an incomplete line")?;
        if line.contains(&b'\r') {
            return Err("startup contains an invalid line ending");
        }
        if line.iter().all(|&byte| matches!(byte, b' ' | b'\t')) {
            continue;
        }
        let line = std::str::from_utf8(line).map_err(|_| "startup text must be ASCII")?;
        stage = match stage {
            Stage::Header => {
                rom_date = Some(header(line).ok_or("unsupported C6 ROM header")?);
                Stage::Build
            }
            Stage::Build if build(line) == rom_date => Stage::Reset,
            Stage::Reset if reset(line) => Stage::SavedPcOrSpi,
            Stage::SavedPcOrSpi if saved_pc(line) => Stage::Spi,
            Stage::SavedPcOrSpi | Stage::Spi if spi(line) => Stage::Mode,
            Stage::Mode if mode(line) => Stage::Loads,
            Stage::Loads if load(line) && loads < MAX_LOAD_LINES => {
                loads += 1;
                Stage::Loads
            }
            Stage::Loads if entry(line) && loads != 0 => Stage::Complete,
            _ => return Err("unsupported or out-of-order C6 ROM startup record"),
        };
    }
    if !matches!(stage, Stage::Header | Stage::Complete) {
        return Err("startup ends with an incomplete ROM boot record");
    }
    Ok(Report {
        schema_version: 1,
        byte_count: bytes.len(),
        line_count,
        rom_boot_observed: stage == Stage::Complete,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normal() -> String {
        [
            "ESP-ROM:esp32c6-20220919",
            "Build:Sep 19 2022",
            "rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)",
            "SPIWP:0xee",
            "mode:DIO, clock div:2",
            "load:0x40000000,len:0x100",
            "load:0x40001000,len:0x200",
            "load:0x40002000,len:0x300",
            "entry 0x40000000",
            "",
        ]
        .join("\n")
    }

    #[test]
    fn accepts_complete_rom_record_and_crlf_without_copying_fields() {
        let record = normal();
        let report = summarize(record.as_bytes()).unwrap();
        assert_eq!(report.line_count, 9);
        assert_eq!(report.byte_count, record.len());
        assert!(report.rom_boot_observed);
        let json = serde_json::to_string(&report).unwrap();
        for field in ["esp32c6", "POWERON", "0x", "2022", "SPIWP"] {
            assert!(!json.contains(field));
        }
        let decoded: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);
        let crlf = record.replace('\n', "\r\n");
        assert_eq!(summarize(crlf.as_bytes()).unwrap().line_count, 9);
    }

    #[test]
    fn empty_and_complete_blank_lines_are_not_boot_observations() {
        for bytes in [b"".as_slice(), b"\n", b" \t\r\n\n"] {
            assert!(!summarize(bytes).unwrap().rom_boot_observed);
        }
        assert!(summarize(b" \t").is_err());
    }

    #[test]
    fn accepts_recognized_usb_reset_and_optional_saved_pc() {
        let record = normal().replace(
            "rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)\n",
            "rst:0x15 (USB_UART_CHIP_RESET),boot:0xc (SPI_FAST_FLASH_BOOT)\nSaved PC:0x40000000\n",
        );
        assert!(summarize(record.as_bytes()).is_ok());
    }

    #[test]
    fn rejects_known_prefix_injections_bad_fields_and_unicode() {
        for (from, to) in [
            ("esp32c6-", "esp32c3-"),
            ("20220919", "20220230"),
            ("Build:Sep 19 2022", "Build:Sep 32 2022"),
            ("Build:Sep 19 2022", "Build:Sep 18 2022"),
            ("rst:0x1 (POWERON)", "rst:0x15 (POWERON)"),
            (
                "0xc (SPI_FAST_FLASH_BOOT)",
                "0xc (SPI_FAST_FLASH_BOOT) panic",
            ),
            ("SPIWP:0xee", "SPIWP:0xeee"),
            ("clock div:2", "clock div:0"),
            ("len:0x100", "len:0x0"),
            ("len:0x100", "len:0x100 abort"),
            ("entry 0x40000000", "entry 0x40000000 trailing"),
            ("mode:DIO", "mode:DIO Error"),
            ("Build:Sep", "Build:Sep🐢"),
        ] {
            assert!(summarize(normal().replace(from, to).as_bytes()).is_err());
        }
        let mut nul = normal().into_bytes();
        nul.insert(1, 0);
        assert!(summarize(&nul).is_err());
    }

    #[test]
    fn rejects_partial_records_reordered_fields_and_multiple_boots() {
        let normal = normal();
        assert!(summarize(normal.trim_end().as_bytes()).is_err());
        let incomplete = normal.lines().take(8).collect::<Vec<_>>().join("\n") + "\n";
        assert!(summarize(incomplete.as_bytes()).is_err());
        assert!(summarize((normal.clone() + &normal).as_bytes()).is_err());
        assert!(summarize(normal.replace("SPIWP:0xee\n", "").as_bytes()).is_err());
        assert!(summarize((normal.clone() + "entry 0x40000000\n").as_bytes()).is_err());
    }

    #[test]
    fn enforces_exact_total_and_physical_line_caps() {
        assert!(summarize(&[b'\n'; MAX_STARTUP_BYTES]).is_ok());
        assert!(summarize(&[b'\n'; MAX_STARTUP_BYTES + 1]).is_err());
        let mut line = vec![b' '; MAX_STARTUP_LINE_BYTES - 1];
        line.push(b'\n');
        assert!(summarize(&line).is_ok());
        line.insert(0, b' ');
        assert!(summarize(&line).is_err());
        let mut excessive = normal();
        for _ in 0..MAX_LOAD_LINES {
            excessive = excessive.replace("entry ", "load:0x40000000,len:0x1\nentry ");
        }
        assert!(summarize(excessive.as_bytes()).is_err());
    }
}
