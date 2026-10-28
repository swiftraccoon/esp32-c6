use super::*;

const TARGET: [u8; 6] = [2, 0, 0, 0, 0, 1];
const LABEL: &[u8] = b"synthetic-private-profile";

fn compare(before: &[Vec<u8>], after: &[Vec<u8>], big: bool) -> Result<ProfileComparison, Error> {
    let before = capture(before, big);
    let after = capture(after, !big);
    compare_beacon_profiles(
        BeaconInput::new(&before, LABEL, TARGET)?,
        BeaconInput::new(&after, LABEL, TARGET)?,
    )
}

fn frame_with(tlvs: &[u8]) -> Vec<u8> {
    let mut frame = beacon(LABEL);
    frame.extend_from_slice(tlvs);
    frame
}

#[test]
fn changed_ssid_mac_tsf_and_sequence_are_outside_collateral_encodings() {
    for big in [false, true] {
        let before_label = b"synthetic-before-private-ssid";
        let after_label = b"synthetic-after";
        let mut before = beacon(before_label);
        let mut after = beacon(after_label);
        before.extend_from_slice(&[221, 4, 1, 2, 3, 4, 200, 0]);
        after.extend_from_slice(&[221, 4, 1, 2, 3, 4, 200, 0]);
        after[22..24].copy_from_slice(&[0x30, 0x12]);
        after[24..32].copy_from_slice(&[0xaa; 8]);
        let other_target = [4; 6];
        after[10..16].copy_from_slice(&other_target);
        after[16..22].copy_from_slice(&other_target);
        let left = capture(&[checked_packet(before)], big);
        let right = capture(&[checked_packet(after)], !big);
        let result = compare_beacon_profiles(
            BeaconInput::new(&left, before_label, TARGET).unwrap(),
            BeaconInput::new(&right, after_label, other_target).unwrap(),
        )
        .unwrap();
        assert!(!result.expected_ssids_equal);
        assert!(!result.expected_macs_equal);
        assert_eq!(result.before_target_packet_count, 1);
        assert_eq!(result.after_target_packet_count, 1);
        for field in [
            result.beacon_interval,
            result.capability_information,
            result.ordered_non_ssid_ies,
            result.ordered_vendor_ies,
        ] {
            assert_eq!(field.before_distinct_encodings, 1);
            assert_eq!(field.after_distinct_encodings, 1);
            assert_eq!(field.shared_distinct_encodings, 1);
            assert!(field.identical_across_all_beacons);
        }
        assert!(result.elements.iter().all(
            |element| element.element_id != 0 && element.encodings.identical_across_all_beacons
        ));
    }
}

#[test]
fn every_interval_and_capability_byte_is_compared() {
    for offset in 32..36 {
        let before = beacon(LABEL);
        let mut after = before.clone();
        after[offset] ^= if offset == 34 { 4 } else { 0x80 };
        let result = compare(&[checked_packet(before)], &[checked_packet(after)], false).unwrap();
        let (changed, untouched) = if offset < 34 {
            (result.beacon_interval, result.capability_information)
        } else {
            (result.capability_information, result.beacon_interval)
        };
        assert_eq!(changed.shared_distinct_encodings, 0);
        assert!(!changed.identical_across_all_beacons);
        assert!(untouched.identical_across_all_beacons);
        assert!(result.ordered_non_ssid_ies.identical_across_all_beacons);
    }
}

#[test]
fn vendor_unknown_duplicate_order_and_tlv_boundaries_are_exact() {
    let cases: &[(&[u8], &[u8], bool)] = &[
        (&[221, 4, 1, 2, 3, 4], &[221, 4, 1, 2, 3, 5], true),
        (&[200, 1, 1], &[200, 1, 2], false),
        (&[200, 1, 1, 200, 1, 2], &[200, 1, 2, 200, 1, 1], false),
        (&[221, 1, 1, 221, 1, 2], &[221, 1, 2, 221, 1, 1], true),
        // Same flattened values, different complete TLV boundaries.
        (&[200, 1, 1, 200, 1, 2], &[200, 2, 1, 2], false),
        (&[221, 1, 1, 221, 1, 2], &[221, 2, 1, 2], true),
        // Distinct IDs reordered; per-ID content stays the same.
        (&[200, 1, 1, 201, 1, 2], &[201, 1, 2, 200, 1, 1], false),
        // Duplication cannot disappear through per-packet deduplication.
        (&[200, 0], &[200, 0, 200, 0], false),
    ];
    for &(before, after, vendor_changed) in cases {
        let result = compare(
            &[checked_packet(frame_with(before))],
            &[checked_packet(frame_with(after))],
            false,
        )
        .unwrap();
        assert!(!result.ordered_non_ssid_ies.identical_across_all_beacons);
        assert_eq!(result.ordered_non_ssid_ies.shared_distinct_encodings, 0);
        assert_eq!(
            result.ordered_vendor_ies.identical_across_all_beacons,
            !vendor_changed
        );
        assert!(result.beacon_interval.identical_across_all_beacons);
        assert!(result.capability_information.identical_across_all_beacons);
    }
}

#[test]
fn shared_sets_late_variation_and_absence_have_distinct_semantics() {
    let absent = checked_packet(beacon(LABEL));
    let empty = checked_packet(frame_with(&[200, 0]));
    let value = checked_packet(frame_with(&[200, 1, 7]));
    let result = compare(
        &[absent, empty.clone(), empty.clone()],
        &[empty, value],
        false,
    )
    .unwrap();
    assert_eq!(result.before_target_packet_count, 3);
    assert_eq!(result.after_target_packet_count, 2);
    assert_eq!(result.ordered_non_ssid_ies.before_distinct_encodings, 2);
    assert_eq!(result.ordered_non_ssid_ies.after_distinct_encodings, 2);
    assert_eq!(result.ordered_non_ssid_ies.shared_distinct_encodings, 1);
    assert!(!result.ordered_non_ssid_ies.identical_across_all_beacons);
    let element = result
        .elements
        .iter()
        .find(|element| element.element_id == 200)
        .unwrap();
    assert_eq!(element.before_presence_count, 2);
    assert_eq!(element.after_presence_count, 2);
    assert_eq!(element.encodings.before_distinct_encodings, 2);
    assert_eq!(element.encodings.after_distinct_encodings, 2);
    assert_eq!(element.encodings.shared_distinct_encodings, 1);
    assert!(!element.encodings.identical_across_all_beacons);
    assert!(result.ordered_vendor_ies.identical_across_all_beacons); // absent in all packets

    let no_element = checked_packet(beacon(LABEL));
    let result = compare(
        &[no_element],
        &[checked_packet(frame_with(&[200, 0]))],
        false,
    )
    .unwrap();
    let element = result
        .elements
        .iter()
        .find(|element| element.element_id == 200)
        .unwrap();
    assert_eq!(element.before_presence_count, 0);
    assert_eq!(element.encodings.before_distinct_encodings, 1); // absence is one encoding
    assert_eq!(element.encodings.shared_distinct_encodings, 0); // unlike a present empty TLV
}

#[test]
fn no_packet_subset_can_admit_wrong_target_background_or_unqualified_crc() {
    let valid = checked_packet(beacon(LABEL));
    let mut wrong_mac = beacon(LABEL);
    wrong_mac[10..16].copy_from_slice(&[4; 6]);
    wrong_mac[16..22].copy_from_slice(&[4; 6]);
    let mut probe = beacon(LABEL);
    probe[0] = 0x40;
    let mut bad_crc = valid.clone();
    *bad_crc.last_mut().unwrap() ^= 1;
    let mut flagged_bad_crc = valid.clone();
    flagged_bad_crc[8] |= 0x40;
    let invalid_packets = [
        checked_packet(beacon(b"another-private-label")),
        checked_packet(wrong_mac),
        checked_packet(probe),
        packet(&beacon(LABEL), None),
        bad_crc,
        flagged_bad_crc,
    ];
    for invalid in invalid_packets {
        assert_eq!(
            compare(
                std::slice::from_ref(&valid),
                &[valid.clone(), invalid],
                false
            ),
            Err(Error::ProfileCapture)
        );
    }
    let mut unsupported = beacon(LABEL);
    unsupported[1] |= 0x80;
    assert_eq!(
        compare(
            std::slice::from_ref(&valid),
            &[checked_packet(unsupported)],
            false
        ),
        Err(Error::ProfileCapture)
    );
}

#[test]
fn late_malformed_records_and_nonempty_bounds_are_not_hidden_by_a_good_beacon() {
    let valid = checked_packet(beacon(LABEL));
    let mut malformed_ie = beacon(LABEL);
    malformed_ie.extend_from_slice(&[200, 2, 1]);
    assert_eq!(
        compare(
            std::slice::from_ref(&valid),
            &[valid.clone(), checked_packet(malformed_ie)],
            false
        ),
        Err(Error::InformationElements)
    );
    let before = capture(std::slice::from_ref(&valid), false);
    let mut after = capture(&[valid], false);
    *after.last_mut().unwrap() ^= 1;
    assert!(
        compare_beacon_profiles(
            BeaconInput::new(&before, LABEL, TARGET).unwrap(),
            BeaconInput::new(&after, LABEL, TARGET).unwrap()
        )
        .is_err()
    );
    assert_eq!(compare(&[], &[], false), Err(Error::NoPackets));
    assert_eq!(
        BeaconInput::new(&[], b"", TARGET),
        Err(Error::WitnessTarget)
    );
    assert_eq!(
        BeaconInput::new(&[], &[1; 33], TARGET),
        Err(Error::WitnessTarget)
    );
    for target in [[0; 6], [1; 6]] {
        assert_eq!(
            BeaconInput::new(&[], LABEL, target),
            Err(Error::WitnessTarget)
        );
    }
    assert_eq!(
        BeaconInput::new(&vec![0; MAX_CAPTURE_BYTES + 1], LABEL, TARGET),
        Err(Error::Limit)
    );
}

#[test]
fn private_sentinels_never_leave_debug_summary_or_error() {
    let sentinel = b"synthetic-private-profile-sentinel";
    let mut frame = beacon(LABEL);
    frame.extend_from_slice(&[221, sentinel.len() as u8]);
    frame.extend_from_slice(sentinel);
    let bytes = capture(&[checked_packet(frame)], false);
    let input = BeaconInput::new(&bytes, LABEL, TARGET).unwrap();
    let debug = format!("{input:?}");
    assert!(debug.contains("BeaconInput") && debug.contains("<redacted>"));
    assert!(!debug.contains(std::str::from_utf8(LABEL).unwrap()));
    let result = compare_beacon_profiles(input, input).unwrap();
    let output = format!(
        "{result:?} {error:?} {error}",
        error = Error::ProfileCapture
    );
    for private in [
        std::str::from_utf8(LABEL).unwrap(),
        std::str::from_utf8(sentinel).unwrap(),
    ] {
        assert!(!debug.contains(private));
        assert!(!output.contains(private));
    }
    let encoded_address = TARGET
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(":");
    assert!(!debug.contains(&encoded_address));
    assert!(!output.contains(&encoded_address));
    assert!(
        result
            .elements
            .iter()
            .all(|element| element.element_id != 0)
    );
}
