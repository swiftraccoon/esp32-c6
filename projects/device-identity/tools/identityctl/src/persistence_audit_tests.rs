use super::*;

// Independent bit-at-a-time zlib-compatible oracle. The SDK generator uses
// zlib.crc32(bytes, 0xffffffff), including its initial/final complement API.
fn oracle(parts: &[&[u8]]) -> u32 {
    let mut value = 0_u32;
    for byte in parts.iter().flat_map(|part| part.iter()) {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            value = (value >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(value & 1));
        }
    }
    !value
}

fn set_page_crc(page: &mut [u8]) {
    let value = oracle(&[&page[4..28]]);
    page[28..32].copy_from_slice(&value.to_le_bytes());
}

fn set_entry_crc(entry: &mut [u8]) {
    let value = oracle(&[&entry[..4], &entry[8..32]]);
    entry[4..8].copy_from_slice(&value.to_le_bytes());
}

fn head(namespace: u8, kind: u8, name: &[u8]) -> [u8; 32] {
    let mut bytes = [255; 32];
    bytes[0] = namespace;
    bytes[1] = kind;
    bytes[2] = 1;
    bytes[8..24].fill(0);
    bytes[8..8 + name.len()].copy_from_slice(name);
    bytes
}

fn write_entry(page: &mut [u8], index: usize, mut entry: [u8; 32], payload: &[u8]) {
    let span = usize::from(entry[2]);
    set_entry_crc(&mut entry);
    page[64 + index * 32..96 + index * 32].copy_from_slice(&entry);
    page[96 + index * 32..96 + index * 32 + payload.len()].copy_from_slice(payload);
    for slot in index..index + span {
        page[32 + slot / 4] &= !(1 << ((slot % 4) * 2));
    }
}

fn fixture_page(version: u8) -> Vec<u8> {
    let mut page = vec![255; PAGE_BYTES];
    page[..4].copy_from_slice(&0xffff_fffe_u32.to_le_bytes());
    page[4..8].copy_from_slice(&0_u32.to_le_bytes());
    page[8] = version;
    set_page_crc(&mut page);
    let mut namespace = head(0, 1, b"fixture_private");
    namespace[24] = 1;
    write_entry(&mut page, 0, namespace, &[]);
    let mut integer = head(1, 4, b"serial_num");
    integer[24..28].copy_from_slice(&1234_u32.to_le_bytes());
    write_entry(&mut page, 1, integer, &[]);
    let payload = b"PRIVATE_VALUE_SENTINEL\0";
    let mut string = head(1, 0x21, b"password");
    string[2] = 2;
    string[24..26].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    string[28..32].copy_from_slice(&oracle(&[payload]).to_le_bytes());
    write_entry(&mut page, 2, string, payload);
    let blob = [91_u8; 40];
    let mut data = head(1, if version == 255 { 0x41 } else { 0x42 }, b"mac_state");
    data[2] = 3;
    if version != 255 {
        data[3] = 0;
    }
    data[24..26].copy_from_slice(&40_u16.to_le_bytes());
    data[28..32].copy_from_slice(&oracle(&[&blob]).to_le_bytes());
    write_entry(&mut page, 4, data, &blob);
    if version != 255 {
        let mut index = head(1, 0x48, b"mac_state");
        index[24..28].copy_from_slice(&40_u32.to_le_bytes());
        index[28] = 1;
        index[29] = 0;
        write_entry(&mut page, 7, index, &[]);
    }
    page
}

fn partition(kind: u8, subtype: u8, offset: u32, size: u32, label: &[u8]) -> [u8; 32] {
    let mut entry = [0; 32];
    entry[..4].copy_from_slice(&[0xaa, 0x50, kind, subtype]);
    entry[4..8].copy_from_slice(&offset.to_le_bytes());
    entry[8..12].copy_from_slice(&size.to_le_bytes());
    entry[12..12 + label.len()].copy_from_slice(label);
    entry
}

fn write_table(image: &mut [u8], entries: &[[u8; 32]]) {
    image[0x8000..0x8c00].fill(255);
    for (index, entry) in entries.iter().enumerate() {
        image[0x8000 + index * 32..0x8020 + index * 32].copy_from_slice(entry);
    }
    let position = 0x8000 + entries.len() * 32;
    let digest = Md5::digest(&image[0x8000..position]);
    image[position..position + 2].fill(0xeb);
    image[position + 16..position + 32].copy_from_slice(&digest);
}

fn entries() -> [[u8; 32]; 3] {
    [
        partition(1, 2, 0x9000, 0x6000, b"private_nvs"),
        partition(1, 1, 0xf000, 0x1000, b"private_phy"),
        partition(0, 0, 0x10000, 0x100000, b"private_app"),
    ]
}

fn fixture() -> Vec<u8> {
    let mut image = vec![255; FLASH_BYTES];
    write_table(&mut image, &entries());
    image[0x9000..0xa000].copy_from_slice(&fixture_page(254));
    image
}

#[test]
fn complete_metadata_and_name_heuristics_reveal_no_values() {
    let image = fixture();
    let report = audit(&image).unwrap();
    assert_eq!(
        (
            report.partition_count,
            report.application_partition_count,
            report.data_partition_count
        ),
        (3, 1, 2)
    );
    assert_eq!(report.nvs_partition_count, 1);
    let counts = &report.nvs;
    assert_eq!(
        (
            counts.page_count,
            counts.active_page_count,
            counts.empty_page_count
        ),
        (6, 1, 5)
    );
    assert_eq!(
        (counts.written_slot_count, counts.written_head_count),
        (8, 5)
    );
    assert_eq!(
        (
            counts.namespace_head_count,
            counts.namespace_identifier_count,
            counts.integer_head_count
        ),
        (1, 1, 2)
    );
    assert_eq!(
        (
            counts.string_head_count,
            counts.blob_data_head_count,
            counts.blob_index_head_count
        ),
        (1, 1, 1)
    );
    assert_eq!(
        (
            counts.network_name_or_address_key_head_count,
            counts.credential_or_key_name_head_count,
            counts.device_identity_key_head_count
        ),
        (2, 1, 1)
    );
    assert!(!report.credential_absence_qualified && !report.logical_values_reconstructed);
    let encoded = serde_json::to_string(&report).unwrap();
    let debug = format!("{report:?}");
    for text in [
        "PRIVATE_VALUE_SENTINEL",
        "fixture_private",
        "private_nvs",
        "private_app",
        "password\"",
        "mac_state",
        "serial_num",
    ] {
        assert!(!encoded.contains(text) && !debug.contains(text));
    }
    assert_eq!(serde_json::from_str::<Report>(&encoded).unwrap(), report);
}

#[test]
fn legacy_records_empty_and_erased_slots_remain_physical_metadata() {
    let mut image = fixture();
    let mut page = fixture_page(255);
    // Erased slot retains arbitrary raw bytes. The audit counts, never parses it.
    page[34] &= !3;
    page[64 + 8 * 32..96 + 8 * 32].fill(73);
    image[0x9000..0xa000].copy_from_slice(&page);
    let report = audit(&image).unwrap();
    assert_eq!(report.nvs.version_one_page_count, 1);
    assert_eq!(report.nvs.legacy_blob_head_count, 1);
    assert_eq!(report.nvs.blob_index_head_count, 0);
    assert_eq!(report.nvs.erased_slot_count, 1);
    assert!(!report.erased_entry_contents_audited);
    for length in [0, FLASH_BYTES - 1, FLASH_BYTES + 1] {
        assert!(audit(&vec![255; length]).is_err());
    }
}

#[test]
fn table_digest_ranges_types_labels_and_encryption_fail_closed() {
    let base = fixture();
    for offset in [0x8000, 0x8004, 0x8060, 0x8070, 0x8080] {
        let mut image = base.clone();
        image[offset] ^= 1;
        assert!(audit(&image).is_err());
    }
    for (offset, value) in [(2, 2_u8), (28, 1), (28, 4)] {
        let mut list = entries();
        list[0][offset] = value;
        let mut image = base.clone();
        write_table(&mut image, &list);
        assert!(audit(&image).is_err());
    }
    for (start, size) in [
        (0x8000_u32, 0x6000_u32),
        (0x9001, 0x6000),
        (0x9000, 0),
        (0x9000, 0x7000),
        (0x7ff000, 0x2000),
    ] {
        let mut list = entries();
        list[0][4..8].copy_from_slice(&start.to_le_bytes());
        list[0][8..12].copy_from_slice(&size.to_le_bytes());
        let mut image = base.clone();
        write_table(&mut image, &list);
        assert!(audit(&image).is_err());
    }
    let mut list = entries();
    let same_label = list[0][12..28].to_vec();
    list[1][12..28].copy_from_slice(&same_label);
    let mut image = base;
    write_table(&mut image, &list);
    assert!(audit(&image).is_err());
}

#[test]
fn key_partitions_and_unknown_or_corrupt_nvs_are_not_clean_reports() {
    let mut image = fixture();
    let mut list = entries().to_vec();
    list.push(partition(1, 4, 0x110000, 0x1000, b"private_keys"));
    write_table(&mut image, &list);
    assert_eq!(audit(&image).unwrap().nvs_keys_partition_count, 1);
    image[0x110000] = 0;
    assert!(audit(&image).unwrap_err().contains("key partition"));
    let base = fixture();
    for offset in [
        0x9004, 0x901c, 0x9020, 0x903f, 0x9044, 0x90a0, 0x9140, 0xa100,
    ] {
        let mut changed = base.clone();
        changed[offset] ^= 1;
        assert!(audit(&changed).is_err(), "offset {offset}");
    }
    let mut image = base;
    image[0x9008] = 253;
    set_page_crc(&mut image[0x9000..0xa000]);
    assert!(audit(&image).unwrap_err().contains("version"));
}

#[test]
fn recomputed_crc_does_not_rescue_bad_spans_namespaces_or_types() {
    for (slot, offset, value) in [
        (1, 0, 2),
        (1, 1, 0x70),
        (1, 2, 2),
        (1, 3, 0),
        (2, 2, 1),
        (4, 3, 255),
        (7, 29, 1),
    ] {
        let mut image = fixture();
        let start = 0x9000 + 64 + slot * 32;
        image[start + offset] = value;
        set_entry_crc(&mut image[start..start + 32]);
        assert!(audit(&image).is_err());
    }
    let mut image = fixture();
    let start = 0x9000 + 64 + 2 * 32;
    image[start + 24..start + 26].copy_from_slice(&4096_u16.to_le_bytes());
    set_entry_crc(&mut image[start..start + 32]);
    assert!(audit(&image).is_err());
    let mut image = fixture();
    image[0x9020] = (image[0x9020] & !3) | 1;
    assert!(audit(&image).is_err());
}

#[test]
fn checksum_seed_matches_independent_oracle_and_supports_split_updates() {
    for length in [0, 1, 24, 28, 32, 126, 4096] {
        let bytes = (0..length).map(|i| (i * 71 + 19) as u8).collect::<Vec<_>>();
        assert_eq!(crc(&[&bytes]), oracle(&[&bytes]));
        let split = length / 2;
        assert_eq!(crc(&[&bytes[..split], &bytes[split..]]), oracle(&[&bytes]));
    }
}

#[test]
fn a_namespace_from_another_partition_cannot_validate_an_orphan() {
    let mut image = fixture();
    let mut list = entries().to_vec();
    list.push(partition(1, 2, 0x110000, 0x1000, b"second_nvs"));
    write_table(&mut image, &list);
    let mut page = fixture_page(254);
    // Keep a valid namespace elsewhere in the image, but mark this partition's
    // declaration erased. Every remaining CRC and payload stays unchanged.
    page[32] &= !3;
    image[0x110000..0x111000].copy_from_slice(&page);
    assert!(audit(&image).is_err());
    image[0x110000..0x111000].copy_from_slice(&fixture_page(254));
    let report = audit(&image).unwrap();
    assert_eq!(report.nvs_partition_count, 2);
    assert_eq!(report.nvs.namespace_identifier_count, 2);
}

#[test]
fn a_later_partition_failure_never_returns_a_valid_prefix_report() {
    let mut image = fixture();
    let mut list = entries().to_vec();
    list.push(partition(1, 2, 0x110000, 0x1000, b"second_nvs"));
    write_table(&mut image, &list);
    image[0x110000..0x111000].copy_from_slice(&fixture_page(254));
    assert_eq!(audit(&image).unwrap().nvs_partition_count, 2);
    // Corrupt payload in the final partition, after the original has passed.
    image[0x110000 + 64 + 3 * 32] ^= 1;
    assert!(audit(&image).is_err());
    // An encrypted flag with a correctly recomputed table digest also fails.
    image[0x110000 + 64 + 3 * 32] ^= 1;
    list[3][28] = 1;
    write_table(&mut image, &list);
    assert!(audit(&image).is_err());
}
