use super::*;

#[path = "profile_tests.rs"]
mod profiles;

#[path = "station_tests.rs"]
mod station;

#[test]
fn readiness_prefix_is_exactly_one_complete_stream_header() {
    for big in [false, true] {
        let bytes = section(big);
        assert!(validate_section_prefix(&bytes).is_ok());
        for end in 0..bytes.len() {
            assert!(validate_section_prefix(&bytes[..end]).is_err());
        }
        let mut extra = bytes.clone();
        extra.extend(idb(127, Vec::new(), big));
        assert!(validate_section_prefix(&extra).is_err());
        let mut wrong_version = bytes.clone();
        wrong_version[12..14].copy_from_slice(&u16_bytes(2, big));
        assert!(validate_section_prefix(&wrong_version).is_err());
        let mut finite = bytes;
        finite[16..24].fill(0);
        assert!(validate_section_prefix(&finite).is_err());
    }
}

fn checked_packet(mut frame: Vec<u8>) -> Vec<u8> {
    let checksum = crc32(&frame);
    frame.extend_from_slice(&checksum.to_le_bytes());
    packet(&frame, Some(0x10))
}

#[test]
fn private_beacon_witness_requires_exact_identity_and_checkable_crc() {
    let label = b"synthetic-private-witness";
    let address = [2, 0, 0, 0, 0, 1];
    for big in [false, true] {
        let bytes = capture(&vec![checked_packet(beacon(label)); 32], big);
        let witness = beacon_witness(&bytes, label, address).unwrap();
        assert_eq!(witness.packet_count, 32);
        assert_eq!(witness.qualified_beacon_count, 32);
        assert_eq!(witness.matching_beacon_count, 32);
        assert_eq!(witness.matching_unqualified_fcs_beacon_count, 0);
        assert!(!format!("{witness:?}").contains(std::str::from_utf8(label).unwrap()));
    }
    let bytes = capture(&[packet(&beacon(label), None)], false);
    let witness = beacon_witness(&bytes, label, address).unwrap();
    assert_eq!(witness.matching_beacon_count, 0);
    assert_eq!(witness.matching_unqualified_fcs_beacon_count, 1);
    let mut corrupt = checked_packet(beacon(label));
    *corrupt.last_mut().unwrap() ^= 1;
    assert_eq!(
        beacon_witness(&capture(&[corrupt], false), label, address)
            .unwrap()
            .matching_beacon_count,
        0
    );
}

#[test]
fn wrong_identities_other_frames_and_late_corruption_cannot_witness() {
    let label = b"synthetic-private-witness";
    let address = [2, 0, 0, 0, 0, 1];
    let wrong_label = checked_packet(beacon(b"other-private-label"));
    let summary = beacon_witness(&capture(&[wrong_label], false), label, address).unwrap();
    assert_eq!(summary.matching_beacon_count, 0);
    assert_eq!(summary.matching_mac_wrong_ssid_count, 1);
    let mut wrong_mac = beacon(label);
    wrong_mac[10..16].copy_from_slice(&[4; 6]);
    wrong_mac[16..22].copy_from_slice(&[4; 6]);
    let summary = beacon_witness(
        &capture(&[checked_packet(wrong_mac)], false),
        label,
        address,
    )
    .unwrap();
    assert_eq!(summary.matching_beacon_count, 0);
    assert_eq!(summary.matching_ssid_wrong_mac_count, 1);
    let mut wrong_bssid = beacon(label);
    wrong_bssid[16] = 4;
    assert!(
        beacon_witness(
            &capture(&[checked_packet(wrong_bssid)], false),
            label,
            address
        )
        .is_err()
    );
    let mut probe = beacon(label);
    probe[0] = 0x40;
    assert_eq!(
        beacon_witness(&capture(&[checked_packet(probe)], false), label, address)
            .unwrap()
            .matching_beacon_count,
        0
    );
    let mut bytes = capture(&[checked_packet(beacon(label))], false);
    *bytes.last_mut().unwrap() ^= 1;
    assert!(beacon_witness(&bytes, label, address).is_err());
    assert_eq!(beacon_witness(&[], &[], address), Err(Error::WitnessTarget));
    assert_eq!(
        beacon_witness(&[], label, [0; 6]),
        Err(Error::WitnessTarget)
    );
    assert_eq!(
        beacon_witness(&[], label, [1; 6]),
        Err(Error::WitnessTarget)
    );
}

fn u16_bytes(value: u16, big: bool) -> [u8; 2] {
    if big {
        value.to_be_bytes()
    } else {
        value.to_le_bytes()
    }
}

fn u32_bytes(value: u32, big: bool) -> [u8; 4] {
    if big {
        value.to_be_bytes()
    } else {
        value.to_le_bytes()
    }
}

fn block(kind: u32, mut body: Vec<u8>, big: bool) -> Vec<u8> {
    while !body.len().is_multiple_of(4) {
        body.push(0);
    }
    let length = u32::try_from(body.len() + 12).unwrap();
    let mut bytes = u32_bytes(kind, big).to_vec();
    bytes.extend_from_slice(&u32_bytes(length, big));
    bytes.extend(body);
    bytes.extend_from_slice(&u32_bytes(length, big));
    bytes
}

fn section(big: bool) -> Vec<u8> {
    let mut body = u32_bytes(0x1a2b3c4d, big).to_vec();
    body.extend_from_slice(&u16_bytes(1, big));
    body.extend_from_slice(&u16_bytes(0, big));
    body.extend_from_slice(&[0xff; 8]);
    block(0x0a0d0d0a, body, big)
}

fn option(code: u16, value: &[u8], big: bool) -> Vec<u8> {
    let mut bytes = u16_bytes(code, big).to_vec();
    bytes.extend_from_slice(&u16_bytes(value.len() as u16, big));
    bytes.extend_from_slice(value);
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    bytes
}

fn idb(link: u16, mut options: Vec<u8>, big: bool) -> Vec<u8> {
    let mut body = u16_bytes(link, big).to_vec();
    body.extend_from_slice(&[0; 2]);
    body.extend_from_slice(&u32_bytes(65535, big));
    if !options.is_empty() {
        options.extend_from_slice(&[0; 4]);
    }
    body.extend(options);
    block(1, body, big)
}

fn epb(frame: &[u8], interface: u32, timestamp: u64, original: u32, big: bool) -> Vec<u8> {
    let mut body = Vec::new();
    for value in [
        interface,
        (timestamp >> 32) as u32,
        timestamp as u32,
        frame.len() as u32,
        original,
    ] {
        body.extend_from_slice(&u32_bytes(value, big));
    }
    body.extend_from_slice(frame);
    block(6, body, big)
}

fn beacon(ssid: &[u8]) -> Vec<u8> {
    let mut frame = vec![0x80, 0, 0, 0];
    frame.extend_from_slice(&[0xff; 6]);
    frame.extend_from_slice(&[0x02, 0, 0, 0, 0, 1]);
    frame.extend_from_slice(&[0x02, 0, 0, 0, 0, 1]);
    frame.extend_from_slice(&[0; 10]); // Sequence control and TSF.
    frame.extend_from_slice(&[100, 0, 1, 0]); // Interval and ESS capability.
    frame.extend_from_slice(&[0, ssid.len() as u8]);
    frame.extend_from_slice(ssid);
    frame.extend_from_slice(&[1, 1, 0x82, 3, 1, 1, 5, 4, 0, 1, 0, 0]);
    frame
}

fn packet(frame: &[u8], flags: Option<u8>) -> Vec<u8> {
    let mut bytes = match flags {
        Some(flags) => vec![0, 0, 9, 0, 2, 0, 0, 0, flags],
        None => vec![0, 0, 8, 0, 0, 0, 0, 0],
    };
    bytes.extend_from_slice(frame);
    bytes
}

fn capture(frames: &[Vec<u8>], big: bool) -> Vec<u8> {
    let mut bytes = section(big);
    bytes.extend(idb(127, Vec::new(), big));
    for (index, frame) in frames.iter().enumerate() {
        bytes.extend(epb(frame, 0, index as u64 + 1, frame.len() as u32, big));
    }
    bytes
}

#[test]
fn both_section_orders_and_32_complete_beacons() {
    for big in [false, true] {
        let frames = vec![packet(&beacon(b"fixture"), None); 32];
        let summary = analyze(&capture(&frames, big)).unwrap();
        assert_eq!(summary.packet_count, 32);
        assert_eq!(summary.beacon_count, 32);
        assert_eq!(summary.management_frame_count, 32);
        assert_eq!(summary.visible_ssid_beacon_count, 32);
        assert_eq!(summary.channel_ie_beacon_count, 32);
        assert_eq!(summary.tim_ie_beacon_count, 32);
        assert_eq!(summary.fcs_unqualified_count, 32);
        assert_eq!(summary.interface_count, 1);
    }
}

#[test]
fn all_partial_tails_and_corrupt_trailers_fail() {
    let bytes = capture(&[packet(&beacon(b"fixture"), None)], false);
    for end in 0..bytes.len() {
        assert!(analyze(&bytes[..end]).is_err(), "partial boundary admitted");
    }
    let mut changed = bytes.clone();
    let last = changed.len() - 1;
    changed[last] ^= 1;
    assert_eq!(analyze(&changed), Err(Error::Pcapng));
    let mut changed = bytes;
    changed.extend(section(false));
    assert_eq!(analyze(&changed), Err(Error::Pcapng));
}

#[test]
fn interface_references_link_types_and_truncation_are_fatal() {
    let frame = packet(&beacon(b"fixture"), None);
    let mut bytes = section(false);
    bytes.extend(epb(&frame, 0, 1, frame.len() as u32, false));
    assert_eq!(analyze(&bytes), Err(Error::Interface));
    let mut bytes = section(false);
    bytes.extend(idb(0, Vec::new(), false));
    bytes.extend(epb(&frame, 0, 1, frame.len() as u32, false));
    assert_eq!(analyze(&bytes), Err(Error::Interface));
    for (interface, original) in [(1, frame.len() as u32), (0, frame.len() as u32 + 1), (0, 1)] {
        let mut bytes = section(false);
        bytes.extend(idb(127, Vec::new(), false));
        bytes.extend(epb(&frame, interface, 1, original, false));
        assert!(analyze(&bytes).is_err());
    }
}

#[test]
fn options_timestamp_units_offsets_and_out_of_order_packets() {
    let frame = packet(&beacon(b"fixture"), None);
    for big in [false, true] {
        let mut opts = option(9, &[0x81], big); // Binary half-second units.
        let offset = if big {
            (-2_i64).to_be_bytes()
        } else {
            (-2_i64).to_le_bytes()
        };
        opts.extend(option(14, &offset, big));
        opts.extend(option(2989, b"PRIVATE_SYNTHETIC_OPTION", big));
        let mut bytes = section(big);
        bytes.extend(idb(127, opts, big));
        bytes.extend(epb(&frame, 0, 8, frame.len() as u32, big));
        bytes.extend(epb(&frame, 0, 6, frame.len() as u32, big));
        assert_eq!(analyze(&bytes).unwrap().packet_count, 2);
    }
    let mut opts = option(9, &[6], false);
    opts.extend(option(9, &[6], false));
    let mut bytes = section(false);
    bytes.extend(idb(127, opts, false));
    bytes.extend(epb(&frame, 0, 1, frame.len() as u32, false));
    assert_eq!(analyze(&bytes), Err(Error::Options));
    for resolution in [20, 0xc0] {
        let mut bytes = section(false);
        bytes.extend(idb(127, option(9, &[resolution], false), false));
        bytes.extend(epb(&frame, 0, 1, frame.len() as u32, false));
        assert_eq!(analyze(&bytes), Err(Error::Timestamp));
    }
    // Signed interface offsets may legitimately place timestamps before epoch;
    // their ordering or correspondence to physical RF time is not inferred.
    let mut bytes = section(false);
    bytes.extend(idb(
        127,
        option(14, &(-100_i64).to_le_bytes(), false),
        false,
    ));
    bytes.extend(epb(&frame, 0, 1, frame.len() as u32, false));
    assert_eq!(analyze(&bytes).unwrap().packet_count, 1);
}

#[test]
fn radiotap_bitmap_alignment_and_declared_fields_are_bounded() {
    let frame = beacon(b"fixture");
    // Two bitmap words then TSFT aligned to offset 16, followed by Flags.
    let mut packet = vec![0, 0, 25, 0, 3, 0, 0, 0x80, 0, 0, 0, 0];
    packet.extend_from_slice(&[0; 12]);
    packet.push(0x20); // Beacon header is already four-byte aligned.
    packet.extend_from_slice(&frame);
    assert_eq!(analyze(&capture(&[packet], false)).unwrap().beacon_count, 1);
    for header in [
        vec![0, 0, 8, 0, 0, 0, 0, 0x80], // Missing extension word.
        vec![0, 0, 8, 0, 4, 0, 0, 0],    // Declared Rate has no payload.
        vec![0, 0, 8, 0, 8, 0, 0, 0],    // Declared Channel has no payload.
        vec![0, 0, 8, 0, 1, 0, 0, 0],    // Declared TSFT has no payload.
    ] {
        let mut packet = header;
        packet.extend_from_slice(&frame);
        assert_eq!(analyze(&capture(&[packet], false)), Err(Error::Radiotap));
    }
}

#[test]
fn fcs_valid_corrupt_flagged_and_padded_unsupported_cases() {
    assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    let mut frame = beacon(b"fixture");
    let checksum = crc32(&frame);
    frame.extend_from_slice(&checksum.to_le_bytes());
    let good = analyze(&capture(&[packet(&frame, Some(0x10))], false)).unwrap();
    assert_eq!(good.beacon_count, 1);
    assert_eq!(good.fcs_unqualified_count, 0);
    let mut changed = frame.clone();
    changed[12] ^= 1;
    let bad = analyze(&capture(&[packet(&changed, Some(0x10))], false)).unwrap();
    assert_eq!((bad.beacon_count, bad.bad_fcs_count), (0, 1));
    let bad = analyze(&capture(&[packet(&frame, Some(0x50))], false)).unwrap();
    assert_eq!((bad.beacon_count, bad.bad_fcs_count), (0, 1));
    // A QoS data header has capture-only padding; its CRC is not interpreted.
    let mut data = vec![0_u8; 26];
    data[0] = 0x88;
    data.extend_from_slice(&[1, 2, 3]);
    let checksum = crc32(&data);
    data.splice(26..26, [0, 0]);
    data.extend_from_slice(&checksum.to_le_bytes());
    let other = analyze(&capture(&[packet(&data, Some(0x30))], false)).unwrap();
    assert_eq!(
        (
            other.bad_fcs_count,
            other.unsupported_frame_count,
            other.fcs_unqualified_count
        ),
        (0, 1, 1)
    );
}

#[test]
fn malformed_supported_beacons_do_not_become_absence() {
    let frame = beacon(b"fixture");
    for (index, value) in [(4, 0), (10, 3), (16, 4), (22, 1), (1, 1), (1, 4)] {
        let mut changed = frame.clone();
        changed[index] = value;
        assert_eq!(
            analyze(&capture(&[packet(&changed, None)], false)),
            Err(Error::Beacon)
        );
    }
    assert_eq!(
        analyze(&capture(&[packet(&frame[..35], None)], false)),
        Err(Error::Beacon)
    );
}

#[test]
fn unsupported_beacon_variants_and_other_classes_are_explicit() {
    for flag in [0x40, 0x80] {
        let mut frame = beacon(b"fixture");
        frame[1] = flag;
        let summary = analyze(&capture(&[packet(&frame, None)], false)).unwrap();
        assert_eq!(
            (summary.beacon_count, summary.unsupported_frame_count),
            (0, 1)
        );
    }
    let mut ibss = beacon(b"fixture");
    ibss[34] = 2;
    let summary = analyze(&capture(&[packet(&ibss, None)], false)).unwrap();
    assert_eq!(
        (summary.beacon_count, summary.unsupported_frame_count),
        (0, 1)
    );
    for control in [0x0008_u16, 0x0108, 0x0208, 0x0308, 0x00d4] {
        let summary = analyze(&capture(&[packet(&control.to_le_bytes(), None)], false)).unwrap();
        assert_eq!((summary.other_frame_count, summary.beacon_count), (1, 0));
    }
}

#[test]
fn information_elements_hidden_binary_and_malformed_controls() {
    for ssid in [&[][..], &[0, 0][..], &[0xff, 0x80][..]] {
        let summary = analyze(&capture(&[packet(&beacon(ssid), None)], false)).unwrap();
        assert_eq!(
            summary.hidden_ssid_beacon_count,
            u64::from(ssid.iter().all(|byte| *byte == 0))
        );
    }
    for tail in [
        vec![0],
        vec![0, 2, 1],
        vec![0, 0, 0, 0],
        vec![3, 0],
        vec![5, 3, 0, 0, 0],
    ] {
        let mut frame = beacon(b"fixture");
        frame.extend(tail);
        assert_eq!(
            analyze(&capture(&[packet(&frame, None)], false)),
            Err(Error::InformationElements)
        );
    }
    assert_eq!(
        analyze(&capture(&[packet(&beacon(&[1; 33]), None)], false)),
        Err(Error::InformationElements)
    );
}

#[test]
fn summaries_and_errors_never_carry_synthetic_payload_or_addresses() {
    let summary = analyze(&capture(
        &[packet(&beacon(b"PRIVATE_FIXTURE_SSID"), None)],
        false,
    ))
    .unwrap();
    let debug = format!("{summary:?}");
    assert!(!debug.contains("PRIVATE_FIXTURE_SSID"));
    let synthetic_address = [0x02_u8, 0, 0, 0, 0, 1]
        .map(|byte| format!("{byte:02x}"))
        .join(":");
    assert!(!debug.contains(&synthetic_address));
    for error in [Error::Pcapng, Error::Beacon, Error::InformationElements] {
        assert!(!format!("{error:?}: {error}").contains("PRIVATE_FIXTURE_SSID"));
    }
}

#[test]
fn complete_options_without_end_and_reserved_idb_bytes_are_accepted() {
    let frame = packet(&beacon(b"fixture"), None);
    let mut body = vec![127, 0, 0x12, 0x34];
    body.extend_from_slice(&65535_u32.to_le_bytes());
    body.extend(option(9, &[6], false));
    let mut bytes = section(false);
    bytes.extend(block(1, body, false));
    bytes.extend(epb(&frame, 0, 1, frame.len() as u32, false));
    assert_eq!(analyze(&bytes).unwrap().beacon_count, 1);
}

#[test]
fn epb_fcs_metadata_overrides_interface_default_and_flags_bad_crc() {
    let mut frame = beacon(b"fixture");
    frame.extend_from_slice(&crc32(&frame).to_le_bytes());
    let frame = packet(&frame, None);
    for (flags, expected_bad) in [(4_u32 << 5, 0), ((4 << 5) | (1 << 24), 1)] {
        let base = epb(&frame, 0, 1, frame.len() as u32, false);
        let mut body = base[8..base.len() - 4].to_vec();
        body.extend(option(2, &flags.to_le_bytes(), false));
        body.extend_from_slice(&[0; 4]);
        let mut bytes = section(false);
        bytes.extend(idb(127, option(13, &[0], false), false));
        bytes.extend(block(6, body, false));
        let summary = analyze(&bytes).unwrap();
        assert_eq!(summary.bad_fcs_count, expected_bad);
        assert_eq!(summary.beacon_count, 1 - expected_bad);
    }
}

#[test]
fn unsupported_radiotap_tlv_list_cannot_hide_a_malformed_later_item() {
    let mut header = vec![
        0, 0, 20, 0, 0, 0, 0, 0x10, 28, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0,
    ];
    for second_length in [1, 5] {
        header[14] = second_length;
        let mut packet = header.clone();
        packet.extend(beacon(b"fixture"));
        assert_eq!(analyze(&capture(&[packet], false)), Err(Error::Radiotap));
    }
}
