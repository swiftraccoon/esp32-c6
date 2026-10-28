//! Bounded metadata-only audit of a saved eight-MiB flash image.
//!
//! This profile admits an MD5-protected ESP-IDF app/data partition table at its
//! standard offset, and plaintext NVS version-one/two physical records. It
//! checks page/entry/payload CRCs and spans, but does not reconstruct logical
//! blob versions, interpret values, or inspect erased entries or other data
//! partitions. Erased records and opaque blobs can contain credentials. Neither
//! a valid CRC nor the absence of an encryption flag establishes plaintext
//! provenance or credential absence. Unsupported encryption and unexplained
//! integrity failure produce static errors rather than a clean report.
//!
//! Input is exactly eight MiB. Parsing borrows its bytes; private partition
//! labels and namespace names use bounded sets (at most96 partitions and254
//! namespaces per NVS partition). No raw names, offsets, values or hashes enter
//! the serializable report or diagnostics. This module performs no file or
//! hardware I/O and contains no unsafe code.

use std::collections::{BTreeMap, BTreeSet};

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

const FLASH_BYTES: usize = 8 * 1024 * 1024;
const PAGE_BYTES: usize = 4096;
const SLOT_BYTES: usize = 32;
const SLOT_COUNT: usize = 126;
const INTEGRITY: &str = "NVS integrity failed; corruption or unsupported encryption is possible";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    FlashPersistenceMetadata,
}

/// Aggregate physical metadata, with fixed qualifications and no input strings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: u32,
    kind: Kind,
    flash_bytes: usize,
    partition_table_md5_valid: bool,
    partition_count: usize,
    application_partition_count: usize,
    data_partition_count: usize,
    nvs_partition_count: usize,
    nvs_keys_partition_count: usize,
    read_only_partition_count: usize,
    nvs: NvsCounts,
    logical_values_reconstructed: bool,
    opaque_blob_contents_audited: bool,
    erased_entry_contents_audited: bool,
    non_nvs_contents_audited: bool,
    nvs_encryption_absence_qualified: bool,
    credential_absence_qualified: bool,
}

/// Counts describe physical slots and heads, including repeated versions.
/// A matching name category is a heuristic, not a claim about its value.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NvsCounts {
    page_count: usize,
    empty_page_count: usize,
    active_page_count: usize,
    full_page_count: usize,
    freeing_page_count: usize,
    version_one_page_count: usize,
    version_two_page_count: usize,
    empty_slot_count: usize,
    erased_slot_count: usize,
    written_slot_count: usize,
    written_head_count: usize,
    namespace_head_count: usize,
    namespace_identifier_count: usize,
    integer_head_count: usize,
    string_head_count: usize,
    legacy_blob_head_count: usize,
    blob_data_head_count: usize,
    blob_index_head_count: usize,
    network_name_or_address_key_head_count: usize,
    credential_or_key_name_head_count: usize,
    device_identity_key_head_count: usize,
    sdk_phy_namespace_identifier_count: usize,
}

// These private records borrow opaque labels. Their Debug representation is
// deliberately unavailable; only aggregate report types can be formatted.
struct Partition<'a> {
    kind: u8,
    subtype: u8,
    start: usize,
    end: usize,
    flags: u32,
    label: &'a [u8],
}

fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn crc(parts: &[&[u8]]) -> u32 {
    // Pinned SDK generator and nvs_parser.py use zlib.crc32(data, 0xffffffff).
    // crc32fast's public initial value follows that complemented zlib API.
    let mut hash = crc32fast::Hasher::new_with_initial(0xffff_ffff);
    for part in parts {
        hash.update(part);
    }
    hash.finalize()
}

fn table(image: &[u8]) -> Result<Vec<Partition<'_>>, &'static str> {
    let bytes = &image[0x8000..0x8c00];
    let mut result = Vec::new();
    let mut labels = BTreeSet::new();
    let mut md5_seen = false;
    for (index, entry) in bytes.chunks_exact(32).enumerate() {
        if md5_seen {
            if entry.iter().any(|&value| value != 0xff) {
                return Err("partition table has data after its MD5 record");
            }
            continue;
        }
        if entry[..2] == [0xeb, 0xeb] {
            if result.is_empty()
                || entry[2..16].iter().any(|&value| value != 0xff)
                || Md5::digest(&bytes[..index * 32]).as_slice() != &entry[16..32]
            {
                return Err("partition table MD5 did not validate");
            }
            md5_seen = true;
            continue;
        }
        if entry[..2] != [0xaa, 0x50] {
            return Err("unsupported partition table or missing MD5 record");
        }
        let kind = entry[2];
        if kind > 1 {
            return Err("unsupported partition type in metadata profile");
        }
        let start = le32(&entry[4..8]) as usize;
        let size = le32(&entry[8..12]) as usize;
        let end = start.checked_add(size).ok_or("partition range overflow")?;
        let flags = le32(&entry[28..32]);
        if start < 0x9000
            || size == 0
            || end > image.len()
            || !start.is_multiple_of(PAGE_BYTES)
            || !size.is_multiple_of(PAGE_BYTES)
            || (kind == 0 && !start.is_multiple_of(0x10000))
            || flags & !3 != 0
        {
            return Err("unsupported partition range, alignment or flags");
        }
        if flags & 1 != 0 {
            return Err("encrypted partition flag is unsupported by metadata audit");
        }
        let raw_label = &entry[12..28];
        let length = raw_label.iter().position(|&b| b == 0).unwrap_or(16);
        let label = &raw_label[..length];
        if label.is_empty()
            || raw_label[length..].iter().any(|&b| b != 0)
            || !labels.insert(label)
            || result
                .iter()
                .any(|part: &Partition<'_>| start < part.end && part.start < end)
        {
            return Err("partition labels or ranges are inconsistent");
        }
        result.push(Partition {
            kind,
            subtype: entry[3],
            start,
            end,
            flags,
            label,
        });
    }
    if !md5_seen {
        return Err("partition table MD5 record is required");
    }
    Ok(result)
}

fn key(entry: &[u8]) -> Result<&[u8], &'static str> {
    let bytes = &entry[8..24];
    let length = bytes.iter().position(|&b| b == 0).ok_or(INTEGRITY)?;
    if length == 0 || bytes[length..].iter().any(|&b| b != 0) {
        return Err(INTEGRITY);
    }
    Ok(&bytes[..length])
}

fn contains_ascii(bytes: &[u8], needles: &[&[u8]]) -> bool {
    needles.iter().any(|needle| {
        bytes.windows(needle.len()).any(|window| {
            window
                .iter()
                .zip(*needle)
                .all(|(&a, &b)| a.to_ascii_lowercase() == b)
        })
    })
}

fn classify_key(bytes: &[u8], counts: &mut NvsCounts) {
    if contains_ascii(bytes, &[b"ssid", b"bssid", b"mac", b"addr"]) {
        counts.network_name_or_address_key_head_count += 1;
    }
    if contains_ascii(
        bytes,
        &[b"password", b"passwd", b"psk", b"cert", b"private", b"key"],
    ) {
        counts.credential_or_key_name_head_count += 1;
    }
    if contains_ascii(bytes, &[b"serial", b"identity", b"device_id", b"uuid"]) {
        counts.device_identity_key_head_count += 1;
    }
}

fn nvs(bytes: &[u8], counts: &mut NvsCounts) -> Result<(), &'static str> {
    let mut namespaces = BTreeMap::<u8, &[u8]>::new();
    let mut namespace_names = BTreeMap::<&[u8], u8>::new();
    let mut referenced = [false; 256];
    for page in bytes.chunks_exact(PAGE_BYTES) {
        counts.page_count += 1;
        let state = le32(page);
        if state == 0xffff_ffff {
            if page.iter().any(|&b| b != 0xff) {
                return Err(INTEGRITY);
            }
            counts.empty_page_count += 1;
            counts.empty_slot_count += SLOT_COUNT;
            continue;
        }
        match state {
            0xffff_fffe => counts.active_page_count += 1,
            0xffff_fffc => counts.full_page_count += 1,
            0xffff_fff8 => counts.freeing_page_count += 1,
            _ => return Err(INTEGRITY),
        }
        match page[8] {
            0xff => counts.version_one_page_count += 1,
            0xfe => counts.version_two_page_count += 1,
            _ => return Err("unsupported NVS page version"),
        }
        if page[9..28].iter().any(|&b| b != 0xff)
            || crc(&[&page[4..28]]) != le32(&page[28..32])
            || page[63] & 0xf0 != 0xf0
        {
            return Err(INTEGRITY);
        }
        let mut states = [0; SLOT_COUNT];
        for (index, slot_state) in states.iter_mut().enumerate() {
            *slot_state = (page[32 + index / 4] >> ((index % 4) * 2)) & 3;
            match *slot_state {
                0 => counts.erased_slot_count += 1,
                2 => counts.written_slot_count += 1,
                3 => {
                    counts.empty_slot_count += 1;
                    if page[64 + index * 32..96 + index * 32]
                        .iter()
                        .any(|&b| b != 0xff)
                    {
                        return Err(INTEGRITY);
                    }
                }
                _ => return Err(INTEGRITY),
            }
        }
        let mut index = 0;
        while index < SLOT_COUNT {
            if states[index] != 2 {
                index += 1;
                continue;
            }
            let entry = &page[64 + index * SLOT_BYTES..96 + index * SLOT_BYTES];
            let span = usize::from(entry[2]);
            if span == 0
                || span > SLOT_COUNT - index
                || states[index..index + span].iter().any(|&state| state != 2)
                || crc(&[&entry[..4], &entry[8..32]]) != le32(&entry[4..8])
            {
                return Err(INTEGRITY);
            }
            let name = key(entry)?;
            let namespace = entry[0];
            if namespace == 255 {
                return Err(INTEGRITY);
            }
            referenced[usize::from(namespace)] = true;
            counts.written_head_count += 1;
            let kind = entry[1];
            match kind {
                1 | 0x11 | 2 | 0x12 | 4 | 0x14 | 8 | 0x18 => {
                    if span != 1 || entry[3] != 255 {
                        return Err(INTEGRITY);
                    }
                    counts.integer_head_count += 1;
                }
                0x21 | 0x41 | 0x42 => {
                    let size = usize::from(u16::from_le_bytes([entry[24], entry[25]]));
                    if span != 1 + size.div_ceil(SLOT_BYTES)
                        || entry[26..28] != [255, 255]
                        || (kind == 0x42 && entry[3] == 255)
                        || (kind != 0x42 && entry[3] != 255)
                    {
                        return Err(INTEGRITY);
                    }
                    let start = 64 + (index + 1) * SLOT_BYTES;
                    let payload = &page[start..start + size];
                    if crc(&[payload]) != le32(&entry[28..32])
                        || (kind == 0x21 && (payload.is_empty() || payload.last() != Some(&0)))
                    {
                        return Err(INTEGRITY);
                    }
                    match kind {
                        0x21 => counts.string_head_count += 1,
                        0x41 => counts.legacy_blob_head_count += 1,
                        _ => counts.blob_data_head_count += 1,
                    }
                }
                0x48 => {
                    if span != 1
                        || entry[3] != 255
                        || entry[29] != 0 && entry[29] != 128
                        || entry[30..32] != [255, 255]
                        || le32(&entry[24..28]) > 127 * 125 * 32
                    {
                        return Err(INTEGRITY);
                    }
                    counts.blob_index_head_count += 1;
                }
                _ => return Err("unsupported NVS entry type"),
            }
            if namespace == 0 {
                let identifier = entry[24];
                if kind != 1 || identifier == 0 || identifier == 255 {
                    return Err(INTEGRITY);
                }
                if namespaces
                    .insert(identifier, name)
                    .is_some_and(|old| old != name)
                    || namespace_names
                        .insert(name, identifier)
                        .is_some_and(|old| old != identifier)
                {
                    return Err(INTEGRITY);
                }
                counts.namespace_head_count += 1;
            } else {
                classify_key(name, counts);
            }
            index += span;
        }
    }
    if referenced
        .iter()
        .enumerate()
        .any(|(id, used)| *used && id != 0 && !namespaces.contains_key(&(id as u8)))
    {
        return Err(INTEGRITY);
    }
    counts.namespace_identifier_count += namespaces.len();
    counts.sdk_phy_namespace_identifier_count +=
        namespaces.values().filter(|&&name| name == b"phy").count();
    Ok(())
}

/// Validate a complete saved image and return only physical metadata counts.
///
/// The exact-size and partition checks run before any NVS interpretation. This
/// profile rejects encrypted partition flags and non-erased NVS-key partitions;
/// other encryption cannot be ruled out by these metadata checks. CRC errors
/// cannot distinguish encryption from corruption. Keys are opaque bounded byte
/// strings used only privately for namespace references and fixed ASCII name
/// heuristics. A successful report establishes no absence of credentials.
///
/// # Errors
///
/// Returns a static error for a wrong input size, missing/invalid table MD5,
/// unsupported partition types/flags, overlapping or out-of-bounds partitions,
/// encrypted flags/non-erased NVS-key partitions, unsupported NVS types/versions,
/// or invalid page/entry/payload CRCs, bitmap states, spans or namespace references.
/// Empty pages/slots must be erased under this profile; erased slots remain
/// uninterpreted. Integrity failure cannot distinguish encryption from corruption.
pub(crate) fn audit(image: &[u8]) -> Result<Report, &'static str> {
    if image.len() != FLASH_BYTES {
        return Err("flash metadata profile requires exactly eight MiB");
    }
    let partitions = table(image)?;
    let mut result = Report {
        schema_version: 1,
        kind: Kind::FlashPersistenceMetadata,
        flash_bytes: FLASH_BYTES,
        partition_table_md5_valid: true,
        partition_count: partitions.len(),
        application_partition_count: 0,
        data_partition_count: 0,
        nvs_partition_count: 0,
        nvs_keys_partition_count: 0,
        read_only_partition_count: 0,
        nvs: NvsCounts::default(),
        logical_values_reconstructed: false,
        opaque_blob_contents_audited: false,
        erased_entry_contents_audited: false,
        non_nvs_contents_audited: false,
        nvs_encryption_absence_qualified: false,
        credential_absence_qualified: false,
    };
    // Examine all key partitions before interpreting any NVS partition.
    for part in &partitions {
        if part.kind == 1
            && part.subtype == 4
            && image[part.start..part.end].iter().any(|&b| b != 255)
        {
            return Err("non-erased NVS key partition is unsupported by metadata audit");
        }
    }
    for part in partitions {
        // Label is deliberately private even when it identifies the NVS subtype.
        let _ = part.label;
        if part.flags & 2 != 0 {
            result.read_only_partition_count += 1;
        }
        if part.kind == 0 {
            result.application_partition_count += 1;
        } else {
            result.data_partition_count += 1;
            if part.subtype == 2 {
                result.nvs_partition_count += 1;
                nvs(&image[part.start..part.end], &mut result.nvs)?;
            } else if part.subtype == 4 {
                result.nvs_keys_partition_count += 1;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "persistence_audit_tests.rs"]
mod tests;
