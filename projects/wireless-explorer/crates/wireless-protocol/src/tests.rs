use super::*;

#[test]
fn literal_wire_vectors_and_fragmentation() -> Result<(), FrameError> {
    let wire = b"WX {\"version\":1,\"id\":42,\"command\":{\"ping\":{\"nonce\":123}}}\n";
    let expected = Request::new(42, Command::Ping { nonce: 123 });
    // Every possible transport split must produce the same one complete request.
    for split in 0..=wire.len() {
        let mut decoder = LineDecoder::new();
        let mut found = None;
        for chunk in [&wire[..split], &wire[split..]] {
            for &byte in chunk {
                if let Some(line) = decoder.push(byte) {
                    found = Some(decode_request(line?)?);
                }
            }
        }
        assert_eq!(found, Some(expected));
        assert!(!decoder.is_pending());
    }
    let mut output = [0; MAX_LINE_BYTES + 1];
    let len = encode(&expected, &mut output)?;
    assert_eq!(&output[..len], wire);
    assert_eq!(encode(&expected, &mut [0; 3]), Err(FrameError::Capacity));
    Ok(())
}

#[test]
fn rejects_bad_versions_fields_trailing_data_and_integer_overflow() {
    for bad in [
        &b"WX {\"version\":1,\"id\":1,\"command\":\"info\",\"extra\":1}"[..],
        &b"WX {\"version\":1,\"id\":4294967296,\"command\":\"info\"}"[..],
        &b"WX {\"version\":1,\"id\":1,\"command\":\"info\"} {}"[..],
        &b"WX {\"version\":1,\"id\":1,\"command\":{\"ping\":{\"nonce\":1,\"extra\":2}}}"[..],
        &b"WX {\"version\":1,\"id\":1,\"command\":\"erase\"}"[..],
    ] {
        assert!(decode_request(bad).is_err(), "accepted {bad:?}");
    }
    assert_eq!(
        decode_request(b"WX {\"version\":2,\"id\":1,\"command\":\"info\"}"),
        Err(FrameError::Version)
    );
}

#[test]
fn overflow_discards_valid_looking_suffix_and_recovers() -> Result<(), FrameError> {
    let mut decoder = LineDecoder::new();
    for _ in 0..=MAX_LINE_BYTES {
        assert!(decoder.push(b'x').is_none());
    }
    for &byte in b"WX {\"version\":1,\"id\":1,\"command\":\"info\"}" {
        assert!(decoder.push(byte).is_none());
    }
    assert_eq!(decoder.push(b'\n'), Some(Err(FrameError::Capacity)));
    for &byte in b"WX {\"version\":1,\"id\":1,\"command\":\"info\"}" {
        decoder.push(byte);
    }
    assert_eq!(
        decode_request(decoder.push(b'\n').unwrap()?)?,
        Request::new(1, Command::Info)
    );
    Ok(())
}

#[test]
fn exact_line_boundary_and_crlf() {
    let mut decoder = LineDecoder::new();
    for _ in 0..MAX_LINE_BYTES {
        decoder.push(b'x');
    }
    assert_eq!(decoder.push(b'\n').unwrap().unwrap().len(), MAX_LINE_BYTES);
    assert!(decode_request(b"WX {\"version\":1,\"id\":1,\"command\":\"info\"}\r").is_ok());
}

#[test]
fn challenge_and_count_conservation_reject_plausible_wrong_results() {
    let request = Request::new(3, Command::Ping { nonce: 7 });
    assert!(
        Response::new(3, Reply::Pong { nonce: 7 })
            .validate_for(&request)
            .is_ok()
    );
    for response in [
        Response::new(4, Reply::Pong { nonce: 7 }),
        Response::new(3, Reply::Pong { nonce: 8 }),
    ] {
        assert_eq!(response.validate_for(&request), Err(FrameError::Mismatch));
    }
    let request = Request::new(4, Command::WifiScan);
    let reply = |total_found, summarized, unreported, out_of_range| Reply::WifiScan {
        total_found,
        summarized,
        unreported,
        channels: [0; 14],
        out_of_range,
    };
    assert!(
        Response::new(4, reply(40, 32, 8, 32))
            .validate_for(&request)
            .is_ok()
    );
    for invalid in [
        reply(40, 32, 7, 32),
        reply(40, 32, 8, 31),
        reply(40, 33, 7, 33),
    ] {
        assert_eq!(
            Response::new(4, invalid).validate_for(&request),
            Err(FrameError::Mismatch)
        );
    }
}
