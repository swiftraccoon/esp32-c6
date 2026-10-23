//! Offline correction of retained espflash 4.6 C6 DIO/8MB images.
//! Original inputs are validated and preserved; outputs are created exclusively.
//! See README.md for source/format evidence and the private API review.

use anyhow::{Context, Result, bail, ensure};
use object::{Object, ObjectSection};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ops::Range;
use std::path::Path;

#[derive(Debug)]
struct Segment {
    address: u32,
    data: Range<usize>,
}

#[derive(Debug)]
struct Image {
    segments: Vec<Segment>,
    checksum: usize,
    digest: usize,
}

fn word(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset.checked_add(4).context("word offset overflow")?)
            .context("truncated word")?
            .try_into()?,
    ))
}

fn parse(bytes: &[u8], expected_flash: u8) -> Result<Image> {
    ensure!(bytes.len() >= 24, "truncated image header");
    ensure!(bytes[0] == 0xe9, "unexpected image magic");
    ensure!((1..=16).contains(&bytes[1]), "unexpected segment count");
    ensure!(bytes[2] == 2, "expected DIO mode");
    ensure!(
        bytes[3] == expected_flash,
        "unexpected frequency/size header"
    );
    ensure!(bytes[12..14] == [13, 0], "expected ESP32-C6 chip ID");
    ensure!(bytes[23] == 1, "appended SHA256 is required");
    let mut position = 24_usize;
    let mut segments = Vec::new();
    let mut checksum = 0xef_u8;
    for _ in 0..bytes[1] {
        let address = word(bytes, position)?;
        let length = usize::try_from(word(bytes, position + 4)?)?;
        ensure!(length != 0 && length % 4 == 0, "invalid segment length");
        position = position.checked_add(8).context("segment header overflow")?;
        let end = position.checked_add(length).context("segment overflow")?;
        let payload = bytes.get(position..end).context("truncated segment")?;
        let end_address = u64::from(address) + length as u64;
        ensure!(
            (address >= 0x4080_0000 && end_address <= 0x4088_0000)
                || (address >= 0x4200_0000 && end_address <= 0x4300_0000),
            "unexpected C6 segment address"
        );
        ensure!(
            segments.iter().all(|other: &Segment| {
                end_address <= u64::from(other.address)
                    || u64::from(address) >= u64::from(other.address) + other.data.len() as u64
            }),
            "overlapping loaded image segments"
        );
        checksum = payload.iter().fold(checksum, |sum, byte| sum ^ byte);
        segments.push(Segment {
            address,
            data: position..end,
        });
        position = end;
    }
    let checksum_offset = position.checked_add(16).context("footer overflow")? / 16 * 16 - 1;
    let digest = checksum_offset + 1;
    ensure!(bytes.len() == digest + 32, "unexpected image/footer length");
    ensure!(
        bytes[position..checksum_offset].iter().all(|&b| b == 0),
        "nonzero padding"
    );
    ensure!(
        bytes[checksum_offset] == checksum,
        "segment XOR checksum mismatch"
    );
    ensure!(
        Sha256::digest(&bytes[..digest])[..] == bytes[digest..],
        "image SHA256 mismatch"
    );
    Ok(Image {
        segments,
        checksum: checksum_offset,
        digest,
    })
}

fn bind_app(bytes: &[u8], image: &Image, elf: &[u8]) -> Result<()> {
    let file = object::File::parse(elf).context("parsing original ELF")?;
    ensure!(
        file.architecture() == object::Architecture::Riscv32 && file.is_little_endian(),
        "expected RV32 little-endian ELF"
    );
    ensure!(
        file.entry() == u64::from(word(bytes, 4)?),
        "ELF/image entry differs"
    );
    let descriptor = file
        .section_by_name(".rodata_desc")
        .context("missing old-HAL descriptor section")?;
    let first = &image.segments[0];
    ensure!(
        first.data.start == 0x20 && first.address == 0x4200_0020,
        "unexpected descriptor mapping"
    );
    ensure!(
        descriptor.address() == u64::from(first.address) && descriptor.size() == 256,
        "unexpected ELF descriptor layout"
    );
    ensure!(
        word(bytes, 0x20)? == 0xabcd_5432,
        "unexpected app descriptor magic"
    );
    ensure!(
        bytes.get(0x20..0x120).context("truncated descriptor")? == descriptor.data()?,
        "ELF/image descriptor differs"
    );
    // IDF app descriptor has its ELF hash at offset 144. This descriptor is
    // the first image segment at 32: 32 + 144 = 176 = 0xb0.
    ensure!(
        bytes[0xb0..0xd0].iter().all(|&b| b == 0),
        "original ELF hash must be zero"
    );
    for section in file.sections() {
        let object::SectionFlags::Elf { sh_flags, .. } = section.flags() else {
            continue;
        };
        if !sh_flags.contains(object::elf::SHF_ALLOC)
            || section.kind() == object::SectionKind::UninitializedData
            || section.size() == 0
        {
            continue;
        }
        let data = section.data()?;
        if data.is_empty() {
            continue;
        }
        bind_bytes(bytes, image, section.address(), data).with_context(|| {
            format!(
                "binding ELF section {}",
                section.name().unwrap_or("<unnamed>")
            )
        })?;
    }
    Ok(())
}

// espflash can split one ELF section across adjacent RAM image segments.
// `parse` already proved mapping uniqueness; every backed byte must match.
fn bind_bytes(bytes: &[u8], image: &Image, start: u64, data: &[u8]) -> Result<()> {
    let mut consumed = 0;
    while consumed < data.len() {
        let address = start
            .checked_add(consumed as u64)
            .context("ELF address overflow")?;
        let segment = image
            .segments
            .iter()
            .find(|segment| {
                let start = u64::from(segment.address);
                start <= address && address < start + segment.data.len() as u64
            })
            .context("ELF section is unmapped")?;
        let inside = usize::try_from(address - u64::from(segment.address))?;
        let count = (data.len() - consumed).min(segment.data.len() - inside);
        let offset = segment.data.start + inside;
        ensure!(
            bytes[offset..offset + count] == data[consumed..consumed + count],
            "ELF section bytes differ"
        );
        consumed += count;
    }
    Ok(())
}

fn correct(original: &[u8], elf: Option<&[u8]>) -> Result<Vec<u8>> {
    let image = parse(original, 0x3f)?;
    if let Some(elf) = elf {
        bind_app(original, &image, elf)?;
    } else {
        ensure!(
            (0x4080_0000..0x4088_0000).contains(&word(original, 4)?),
            "bootloader entry must be RAM"
        );
        ensure!(
            image
                .segments
                .iter()
                .all(|segment| segment.address < 0x4088_0000),
            "bootloader must contain only RAM segments"
        );
    }
    let mut output = original.to_vec();
    // esptool 5.1 targets/esp32c6.py:80MHz encodes 0, a documented ROM workaround.
    // Preserve the 8MB size nibble; only the frequency nibble changes.
    output[3] = 0x30;
    if let Some(elf) = elf {
        output[0xb0..0xd0].copy_from_slice(&Sha256::digest(elf));
    }
    output[image.checksum] = image.segments.iter().fold(0xef, |sum, segment| {
        output[segment.data.clone()]
            .iter()
            .fold(sum, |sum, byte| sum ^ byte)
    });
    let hash = Sha256::digest(&output[..image.digest]);
    output[image.digest..].copy_from_slice(&hash);
    parse(&output, 0x30).context("validating corrected image")?;
    ensure!(
        original
            .iter()
            .zip(&output)
            .enumerate()
            .all(|(i, (old, new))| old == new
                || i == 3
                || i == image.checksum
                || i >= image.digest
                || (elf.is_some() && (0xb0..0xd0).contains(&i))),
        "unexpected byte change"
    );
    Ok(output)
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (input, elf, output) = match args.as_slice() {
        [mode, input, output] if mode == "boot" => (input, None, output),
        [mode, input, elf, output] if mode == "app" => (input, Some(elf), output),
        _ => bail!("usage: rtic-image-correction boot INPUT OUTPUT | app INPUT ELF OUTPUT"),
    };
    let original = fs::read(input).context("reading original image")?;
    let elf = elf
        .map(fs::read)
        .transpose()
        .context("reading original ELF")?;
    let corrected = correct(&original, elf.as_deref())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(output))
        .context("creating exclusive corrected output")?;
    file.write_all(&corrected)
        .context("writing corrected output")?;
    file.sync_all().context("syncing corrected output")?;
    println!(
        "input_sha256={:x}\noutput_sha256={:x}\nbytes={}",
        Sha256::digest(&original),
        Sha256::digest(&corrected),
        corrected.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0; 48];
        bytes[0] = 0xe9;
        bytes[1] = 1;
        bytes[2] = 2;
        bytes[3] = 0x3f;
        bytes[12] = 13;
        bytes[23] = 1;
        bytes[4..8].copy_from_slice(&0x4080_0000_u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&0x4080_0000_u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&4_u32.to_le_bytes());
        bytes[32..36].copy_from_slice(&[1, 2, 3, 4]);
        bytes[47] = 0xeb;
        let hash = Sha256::digest(&bytes);
        bytes.extend_from_slice(&hash);
        bytes
    }

    #[test]
    fn intact_image_changes_only_frequency_and_digest() {
        let original = fixture();
        let output = correct(&original, None).unwrap();
        assert_eq!(output[3], 0x30);
        assert_eq!(&original[4..48], &output[4..48]);
        assert_eq!(parse(&output, 0x30).unwrap().checksum, 47);
    }

    #[test]
    fn malformed_inputs_are_rejected_before_output() {
        for kind in 0..8 {
            let mut bytes = fixture();
            match kind {
                0 => {
                    bytes.pop();
                }
                1 => bytes[0] = 0,
                2 => bytes[12] = 0,
                3 => bytes[32] ^= 1,
                4 => bytes[47] ^= 1,
                5 => bytes[79] ^= 1,
                6 => bytes[3] = 0x30,
                7 => bytes[28..32].copy_from_slice(&u32::MAX.to_le_bytes()),
                _ => unreachable!(),
            }
            assert!(correct(&bytes, None).is_err(), "case {kind}");
        }
    }

    #[test]
    fn split_section_requires_complete_matching_mapping() {
        let bytes = [1, 2, 3, 4];
        let mut image = Image {
            segments: vec![
                Segment {
                    address: 0x4080_0000,
                    data: 0..2,
                },
                Segment {
                    address: 0x4080_0002,
                    data: 2..4,
                },
            ],
            checksum: 0,
            digest: 0,
        };
        bind_bytes(&bytes, &image, 0x4080_0000, &bytes).unwrap();
        assert!(bind_bytes(&bytes, &image, 0x4080_0000, &[1, 2, 3, 5]).is_err());
        image.segments[1].address += 1;
        assert!(bind_bytes(&bytes, &image, 0x4080_0000, &bytes).is_err());
    }
}
