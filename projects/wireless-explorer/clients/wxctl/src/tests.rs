use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use wireless_protocol::{Reply, encode};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "wxctl-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn wire<T: serde::Serialize>(record: &T) -> Result<Vec<u8>> {
    let mut bytes = [0; MAX_LINE_BYTES + 1];
    let len = encode(record, &mut bytes)?;
    Ok(bytes[..len].to_vec())
}

#[test]
fn transcript_controls_reject_fatal_missing_duplicate_and_partial_output() -> Result<()> {
    let request = Request::new(42, Command::Ping { nonce: 7 });
    let good = wire(&Response::new(42, Reply::Pong { nonce: 7 }))?;
    let mut control = transcript::Transcript::new(request);
    control.feed(&good)?;
    assert!(control.finish().is_ok());
    for bad in [
        Vec::new(),
        good[..good.len() - 1].to_vec(),
        [good.as_slice(), good.as_slice()].concat(),
        [b"ESP-ROM:esp32c6\n".as_slice(), good.as_slice()].concat(),
        [good.as_slice(), b"Guru Meditation Error\n"].concat(),
        wire(&Response::new(43, Reply::Pong { nonce: 7 }))?,
        wire(&Response::new(42, Reply::Pong { nonce: 8 }))?,
        wire(&Response::new(
            42,
            Reply::Error {
                stage: wireless_protocol::FailureStage::Wifi,
                code: 123,
            },
        ))?,
    ] {
        let mut transcript = transcript::Transcript::new(request);
        assert!(
            transcript
                .feed(&bad)
                .and_then(|()| transcript.finish())
                .is_err(),
            "accepted {bad:?}"
        );
    }
    Ok(())
}

#[test]
fn saved_raw_replay_rejects_changed_summary_and_retained_failure() -> Result<()> {
    let fixture = Fixture::new()?;
    let request = Request::new(42, Command::Ping { nonce: 7 });
    let response = Response::new(42, Reply::Pong { nonce: 7 });
    fs::write(fixture.0.join("request.frame"), wire(&request)?)?;
    fs::write(fixture.0.join("serial.raw"), wire(&response)?)?;
    fs::write(
        fixture.0.join("response.json"),
        serde_json::to_vec(&response)?,
    )?;
    assert_eq!(replay(&fixture.0)?, response);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("missing-target", fixture.0.join("failure.txt"))?;
        assert!(replay(&fixture.0).is_err());
        fs::remove_file(fixture.0.join("failure.txt"))?;
    }
    fs::write(
        fixture.0.join("response.json"),
        serde_json::to_vec(&Response::new(42, Reply::Pong { nonce: 8 }))?,
    )?;
    assert!(replay(&fixture.0).is_err());
    fs::write(
        fixture.0.join("response.json"),
        serde_json::to_vec(&response)?,
    )?;
    fs::write(fixture.0.join("failure.txt"), "original failed attempt\n")?;
    assert!(replay(&fixture.0).is_err());
    Ok(())
}

#[test]
fn saved_request_requires_one_complete_frame_and_reads_are_bounded() -> Result<()> {
    let fixture = Fixture::new()?;
    let request = wire(&Request::new(1, Command::Info))?;
    assert!(transcript::request_frame(&request).is_ok());
    assert!(transcript::request_frame(&request[..request.len() - 1]).is_err());
    assert!(transcript::request_frame(&[request.as_slice(), request.as_slice()].concat()).is_err());
    let path = fixture.0.join("large");
    fs::write(&path, [0; 5])?;
    assert!(read_bounded(&path, 4).is_err());
    Ok(())
}

#[test]
fn ledger_checks_actual_commands_and_rejects_path_escape_or_duplicates() -> Result<()> {
    let fixture = Fixture::new()?;
    let capture = fixture.0.join("ping");
    fs::create_dir(&capture)?;
    let request = Request::new(42, Command::Ping { nonce: 7 });
    let response = Response::new(42, Reply::Pong { nonce: 7 });
    fs::write(capture.join("request.frame"), wire(&request)?)?;
    fs::write(capture.join("serial.raw"), wire(&response)?)?;
    fs::write(
        capture.join("response.json"),
        serde_json::to_vec(&response)?,
    )?;
    let path = fixture.0.join("evidence.json");
    let entry = serde_json::json!({"directory":"ping", "command":request.command()});
    let mut ledger = serde_json::json!({"schema_version":1,"kind":"wireless-functional-probes-v1","captures":[entry.clone()]});
    ledger["coverage"] = serde_json::json!({
        "uart_control":"not_run", "passive_wifi_survey":"not_run", "native_usb":"not_run",
        "iphone_power":"not_run", "iphone_usb_data":"not_run", "ble":"not_run",
        "ieee802154":"not_run", "controlled_peer_exchange":"not_run", "coexistence":"not_run",
        "disconnect_recovery":"not_run", "energy":"not_run"
    });
    fs::write(&path, serde_json::to_vec(&ledger)?)?;
    assert_eq!(evidence::verify(&path)?, 1);
    ledger["coverage"]["ble"] = serde_json::json!("observed_transcript");
    fs::write(&path, serde_json::to_vec(&ledger)?)?;
    assert!(evidence::verify(&path).is_err());
    ledger["coverage"]["ble"] = serde_json::json!("not_run");
    let duplicate = serde_json::to_string(&ledger)?.replace(
        "\"ble\":\"not_run\"",
        "\"ble\":\"observed_transcript\",\"ble\":\"not_run\"",
    );
    fs::write(&path, duplicate)?;
    assert!(evidence::verify(&path).is_err());
    for captures in [
        serde_json::json!([]),
        serde_json::json!([entry.clone(), entry.clone()]),
        serde_json::json!([{"directory":"../ping","command":request.command()}]),
        serde_json::json!([{"directory":"ping","command":"wifi_scan"}]),
    ] {
        ledger["captures"] = captures;
        fs::write(&path, serde_json::to_vec(&ledger)?)?;
        assert!(evidence::verify(&path).is_err());
    }
    Ok(())
}
