//! Project-local functional observations. The historical OS ledger stays frozen.
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use wireless_protocol::{Command, Reply, Transport};

use crate::{read_bounded, replay, transcript};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    schema_version: u8,
    kind: String,
    captures: Vec<Capture>,
    coverage: Coverage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    ObservedTranscript,
    NotRun,
}

impl Status {
    fn observed(value: bool) -> Self {
        if value {
            Self::ObservedTranscript
        } else {
            Self::NotRun
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Coverage {
    uart_control: Status,
    passive_wifi_survey: Status,
    native_usb: Status,
    iphone_power: Status,
    iphone_usb_data: Status,
    ble: Status,
    ieee802154: Status,
    controlled_peer_exchange: Status,
    coexistence: Status,
    disconnect_recovery: Status,
    energy: Status,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    /// Relative to the ledger's parent directory; only normal components allowed.
    directory: PathBuf,
    command: Command,
}

pub(crate) fn verify(path: &Path) -> Result<usize> {
    let ledger: Ledger = serde_json::from_slice(&read_bounded(path, 64 * 1024)?)?;
    ensure!(
        ledger.schema_version == 1 && ledger.kind == "wireless-functional-probes-v1",
        "unsupported ledger"
    );
    ensure!(!ledger.captures.is_empty(), "ledger contains no captures");
    let root = path.parent().context("ledger needs a parent directory")?;
    let mut seen = BTreeSet::new();
    let (mut uart_info, mut ping, mut survey) = (false, false, false);
    for capture in &ledger.captures {
        ensure!(
            !capture.directory.as_os_str().is_empty()
                && capture
                    .directory
                    .components()
                    .all(|part| matches!(part, Component::Normal(_))),
            "invalid capture path"
        );
        ensure!(
            seen.insert(capture.directory.clone()),
            "duplicate capture path"
        );
        let mut directory = root.to_path_buf();
        for component in capture.directory.components() {
            directory.push(component);
            let metadata = std::fs::symlink_metadata(&directory)?;
            ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "capture path is not a regular directory"
            );
        }
        for file in ["request.frame", "serial.raw", "response.json"] {
            ensure!(
                std::fs::symlink_metadata(directory.join(file))?.is_file(),
                "capture input is not a regular file"
            );
        }
        let request = transcript::request_frame(&read_bounded(
            &directory.join("request.frame"),
            wireless_protocol::MAX_LINE_BYTES + 1,
        )?)?;
        ensure!(
            request.command() == capture.command,
            "ledger command differs from capture"
        );
        let response = replay(&directory)
            .with_context(|| format!("replay {}", capture.directory.display()))?;
        match response.reply() {
            Reply::Info {
                transport: Transport::Uart,
                ..
            } => uart_info = true,
            Reply::Pong { .. } => ping = true,
            Reply::WifiScan { .. } => survey = true,
            _ => {}
        }
    }
    let expected = Coverage {
        uart_control: Status::observed(uart_info && ping),
        passive_wifi_survey: Status::observed(survey),
        native_usb: Status::NotRun,
        iphone_power: Status::NotRun,
        iphone_usb_data: Status::NotRun,
        ble: Status::NotRun,
        ieee802154: Status::NotRun,
        controlled_peer_exchange: Status::NotRun,
        coexistence: Status::NotRun,
        disconnect_recovery: Status::NotRun,
        energy: Status::NotRun,
    };
    ensure!(
        ledger.coverage == expected,
        "coverage differs from the supported functional transcript scope"
    );
    Ok(ledger.captures.len())
}
