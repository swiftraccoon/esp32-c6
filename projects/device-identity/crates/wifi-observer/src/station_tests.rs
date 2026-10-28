use super::*;

const LABEL: &[u8] = b"PRIVATE_STATION_NONCE";
const STATION: [u8; 6] = [2, 0, 0, 0, 0, 1];

fn probe(ssid: &[u8], station: [u8; 6]) -> Vec<u8> {
    let mut frame = vec![0; 24];
    frame[0] = 0x40;
    frame[4..10].fill(255);
    frame[10..16].copy_from_slice(&station);
    frame[16..22].fill(255);
    frame.extend([0, ssid.len() as u8]);
    frame.extend(ssid);
    // Opaque but bounded supported-rate and vendor IEs are retained in framing.
    frame.extend([1, 2, 0x82, 0x84, 221, 3, 91, 73, 49]);
    frame
}

#[test]
fn whole_mixed_capture_counts_every_packet_and_matches_exact_nonce_and_ta() {
    for big in [false, true] {
        let raw = capture(
            &[
                checked_packet(probe(LABEL, STATION)),
                checked_packet(beacon(b"other")),
                checked_packet(probe(b"different-nonce", STATION)),
                checked_packet(probe(LABEL, [4; 6])),
            ],
            big,
        );
        let result = station_probe_witness(&raw, LABEL, STATION).unwrap();
        assert_eq!(result.capture, analyze(&raw).unwrap());
        assert_eq!(result.capture.packet_count, 4);
        assert_eq!(result.capture.beacon_count, 1);
        assert_eq!(result.capture.other_frame_count, 3);
        assert_eq!(result.qualified_probe_count, 3);
        assert_eq!(result.matching_probe_count, 1);
        assert_eq!(result.matching_mac_wrong_ssid_count, 1);
        assert_eq!(result.matching_ssid_wrong_mac_count, 1);
        let text = serde_json::to_string(&result).unwrap();
        assert!(!text.contains(std::str::from_utf8(LABEL).unwrap()));
        assert!(!format!("{result:?}").contains(std::str::from_utf8(LABEL).unwrap()));
        assert_eq!(
            serde_json::from_str::<StationProbeSummary>(&text).unwrap(),
            result
        );
    }
}

#[test]
fn missing_or_bad_crc_and_wrong_frame_type_never_witness() {
    let mut corrupt = checked_packet(probe(LABEL, STATION));
    *corrupt.last_mut().unwrap() ^= 1;
    let mut data = probe(LABEL, STATION);
    data[0] = 0x48;
    let raw = capture(
        &[
            packet(&probe(LABEL, STATION), None),
            corrupt,
            checked_packet(data),
        ],
        false,
    );
    let result = station_probe_witness(&raw, LABEL, STATION).unwrap();
    assert_eq!(result.capture.packet_count, 3);
    assert_eq!(result.capture.bad_fcs_count, 1);
    assert_eq!(result.matching_probe_count, 0);
    assert_eq!(result.matching_unqualified_fcs_probe_count, 1);
    assert_eq!(result.qualified_probe_count, 0);
}

#[test]
fn recomputed_crc_does_not_rescue_bad_probe_header_or_late_tlv() {
    let baseline = probe(LABEL, STATION);
    let mut cases = Vec::new();
    for offset in [4, 10, 16, 22] {
        let mut bad = baseline.clone();
        bad[offset] ^= 1;
        cases.push(bad);
    }
    let mut duplicate = baseline.clone();
    duplicate.extend([0, 0]);
    cases.push(duplicate);
    let mut truncated = baseline;
    truncated.extend([91, 2, 73]);
    cases.push(truncated);
    for bad in cases {
        let raw = capture(
            &[checked_packet(probe(LABEL, STATION)), checked_packet(bad)],
            false,
        );
        assert_eq!(analyze(&raw).unwrap().packet_count, 2);
        assert_eq!(
            station_probe_witness(&raw, LABEL, STATION),
            Err(Error::StationProbe)
        );
    }
}

#[test]
fn unsupported_headers_and_invalid_targets_are_explicit() {
    for mask in [0x0100_u16, 0x0200, 0x0400, 0x4000, 0x8000] {
        let mut frame = probe(LABEL, STATION);
        frame[..2].copy_from_slice(&(0x40 | mask).to_le_bytes());
        let result =
            station_probe_witness(&capture(&[checked_packet(frame)], false), LABEL, STATION)
                .unwrap();
        assert_eq!(result.matching_probe_count, 0);
        assert_eq!(result.unsupported_probe_count, 1);
    }
    for (label, address) in [(&[][..], STATION), (LABEL, [0; 6]), (LABEL, [1; 6])] {
        assert_eq!(
            station_probe_witness(&[], label, address),
            Err(Error::WitnessTarget)
        );
    }
}
