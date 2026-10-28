//! Capture controlled runtime probes and derive value-free inventory summaries.
//!
//! Input files and argument values are never included in diagnostics. Offline
//! summaries never open devices. Runtime capture opens the selected UART and
//! requests reversible application-table changes, with private raw storage.
//! It explicitly releases the bridge's DTR/RTS lines; opening/releasing them
//! may reset or start the board before the baseline request.

#![forbid(unsafe_code)]

mod capture;
mod efuse;
mod identity_audit;
mod persistence_audit;
mod runtime;
mod startup;
mod usb;
mod usb_control;
mod wifi_capture;
mod wifi_restart;
mod wifi_trial;

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, error::ErrorKind};

const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Parser)]
#[command(
    bin_name = "identityctl",
    version,
    about = "Capture controlled identity probes and summarize private values safely"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Audit saved flash partition and NVS metadata without exposing values.
    FlashMetadataAudit {
        #[arg(long)]
        input: PathBuf,
    },
    /// Compare saved native USB serial contents with a validated factory MAC.
    UsbFactoryRelation {
        #[arg(long)]
        capture: PathBuf,
        #[arg(long)]
        efuse_input: PathBuf,
    },
    /// Check saved CP2102N configuration layout and Fletcher16 integrity.
    BridgeConfigAudit {
        #[arg(long)]
        capture: PathBuf,
    },
    /// Observe AP controls A and B across checked stops on one UART connection.
    WifiApRestartTrial {
        #[arg(long)]
        port_file: PathBuf,
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Replay both phases of one complete private AP restart session.
    WifiApRestartReplay {
        #[arg(long)]
        capture: PathBuf,
    },
    /// Observe a nonce-tagged SoftAP beacon from the controlled probe firmware.
    WifiApTrial {
        /// Private file containing the verified UART bridge path.
        #[arg(long)]
        port_file: PathBuf,
        /// Private receiver selection and worker provenance.
        #[arg(long)]
        profile: PathBuf,
        /// New directory under workspace-root private/device-identity/.
        #[arg(long)]
        output: PathBuf,
    },
    /// Replay one complete private AP witness trial and its cleanup receipts.
    WifiApReplay {
        #[arg(long)]
        capture: PathBuf,
    },
    /// Compare beacon profiles from two completely replayed AP trial bundles.
    WifiApCompareProfiles {
        #[arg(long)]
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
    },
    /// Capture a bounded read-only Kismet Wi-Fi stream into private storage.
    WifiCapture {
        /// Private receiver selection and remote-worker provenance.
        #[arg(long)]
        profile: PathBuf,
        /// New directory under workspace-root private/device-identity/.
        #[arg(long)]
        output: PathBuf,
    },
    /// Replay a complete saved Wi-Fi capture and its transport receipts.
    WifiReplay {
        #[arg(long)]
        capture: PathBuf,
    },
    /// Validate private PCAPNG framing and summarize supported Wi-Fi frames.
    WifiSummary {
        #[arg(long)]
        input: PathBuf,
    },
    /// Summarize a whitelisted ESP32-C6 espefuse JSON inventory.
    EfuseSummary {
        /// Private JSON capture. Only regular files without symlink components are accepted.
        #[arg(long)]
        input: PathBuf,
    },
    /// Summarize macOS SPUSBHostDataType JSON from system_profiler.
    UsbSummary {
        /// Private JSON capture. Only regular files without symlink components are accepted.
        #[arg(long)]
        input: PathBuf,
    },
    /// Compare current host serial properties without exposing their contents.
    UsbCompare {
        #[arg(long)]
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
    },
    /// Capture the fixed read-only USB control-transfer plan into private storage.
    UsbControlCapture {
        /// New directory under workspace-root private/device-identity/.
        #[arg(long)]
        output: PathBuf,
    },
    /// Replay a complete private USB control capture and its receipts.
    UsbControlReplay {
        #[arg(long)]
        capture: PathBuf,
    },
    /// Validate a private raw USB record without admitting capture receipts.
    UsbControlSummary {
        #[arg(long)]
        input: PathBuf,
    },
    /// Compare validated fresh serial records without exposing their values.
    UsbControlCompare {
        #[arg(long)]
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
    },
    /// Validate a complete saved runtime-table exchange sequence.
    RuntimeSummary {
        #[arg(long)]
        input: PathBuf,
    },
    /// Check bounded pre-command C6 startup output without publishing raw text.
    StartupSummary {
        #[arg(long)]
        input: PathBuf,
    },
    /// Run four controlled UART phases and retain raw observations privately.
    RuntimeCapture {
        /// Verified UART bridge; this opens a device at 115200 baud.
        #[arg(long)]
        port: String,
        /// New directory under workspace-root private/device-identity/.
        #[arg(long)]
        output: PathBuf,
    },
    /// Recheck a completed private capture, including its failure receipts.
    RuntimeReplay {
        #[arg(long)]
        capture: PathBuf,
    },
}

fn read_input(path: &Path) -> Result<Vec<u8>, &'static str> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        if component == Component::ParentDir {
            return Err("input path must not contain parent components");
        }
        prefix.push(component);
        let metadata = fs::symlink_metadata(&prefix).map_err(|_| "cannot inspect input path")?;
        if metadata.file_type().is_symlink() {
            return Err("input path must not contain symlinks");
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| "cannot inspect input file")?;
    if !metadata.is_file() {
        return Err("input must be a regular file");
    }
    if metadata.len() > MAX_INPUT_BYTES as u64 {
        return Err("input exceeds size limit");
    }
    let file = File::open(path).map_err(|_| "cannot open input file")?;
    if !file
        .metadata()
        .map_err(|_| "cannot inspect opened input file")?
        .is_file()
    {
        return Err("opened input must be a regular file");
    }
    let mut bytes = Vec::new();
    file.take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read input file")?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err("input exceeds size limit");
    }
    Ok(bytes)
}

fn run(cli: Cli) -> Result<(), &'static str> {
    let output = match cli.command {
        Command::FlashMetadataAudit { input } => {
            serde_json::to_vec_pretty(&persistence_audit::audit(&read_input(&input)?)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbFactoryRelation {
            capture,
            efuse_input,
        } => serde_json::to_vec_pretty(&identity_audit::usb_factory_mac_relation(
            &capture,
            &efuse_input,
        )?)
        .map_err(|_| "cannot serialize summary")?,
        Command::BridgeConfigAudit { capture } => {
            serde_json::to_vec_pretty(&identity_audit::bridge_configuration_audit(&capture)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::WifiApRestartTrial {
            port_file,
            profile,
            output,
        } => serde_json::to_vec_pretty(&wifi_restart::capture(&port_file, &profile, &output)?)
            .map_err(|_| "cannot serialize summary")?,
        Command::WifiApRestartReplay { capture } => {
            serde_json::to_vec_pretty(&wifi_restart::replay(&capture)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::WifiApTrial {
            port_file,
            profile,
            output,
        } => serde_json::to_vec_pretty(&wifi_trial::capture(&port_file, &profile, &output)?)
            .map_err(|_| "cannot serialize summary")?,
        Command::WifiApReplay { capture } => {
            serde_json::to_vec_pretty(&wifi_trial::replay(&capture)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::WifiApCompareProfiles { before, after } => {
            serde_json::to_vec_pretty(&wifi_trial::compare_profiles(&before, &after)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::WifiCapture { profile, output } => {
            serde_json::to_vec_pretty(&wifi_capture::capture(&profile, &output)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::WifiReplay { capture } => {
            serde_json::to_vec_pretty(&wifi_capture::replay(&capture)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::WifiSummary { input } => {
            let bytes = read_input(&input)?;
            serde_json::to_vec_pretty(
                &wifi_observer::analyze(&bytes)
                    .map_err(|_| "Wi-Fi capture record did not validate")?,
            )
            .map_err(|_| "cannot serialize summary")?
        }
        Command::EfuseSummary { input } => {
            let bytes = read_input(&input)?;
            serde_json::to_vec_pretty(&efuse::summarize(&bytes)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbSummary { input } => {
            let bytes = read_input(&input)?;
            serde_json::to_vec_pretty(&usb::summarize(&bytes)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbCompare { before, after } => {
            let before = read_input(&before)?;
            let after = read_input(&after)?;
            serde_json::to_vec_pretty(&usb::compare(&before, &after)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbControlCapture { output } => {
            serde_json::to_vec_pretty(&usb_control::capture(&output)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbControlReplay { capture } => {
            serde_json::to_vec_pretty(&usb_control::replay(&capture)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbControlSummary { input } => {
            let record = usb_control::record(&input)?;
            serde_json::to_vec_pretty(
                &usb_observer::summarize(&record)
                    .map_err(|_| "USB control record did not validate")?,
            )
            .map_err(|_| "cannot serialize summary")?
        }
        Command::UsbControlCompare { before, after } => {
            let before = usb_control::record(&before)?;
            let after = usb_control::record(&after)?;
            serde_json::to_vec_pretty(
                &usb_observer::compare(&before, &after)
                    .map_err(|_| "USB control comparison did not validate")?,
            )
            .map_err(|_| "cannot serialize summary")?
        }
        Command::RuntimeSummary { input } => {
            let bytes = read_input(&input)?;
            serde_json::to_vec_pretty(&runtime::summarize(&bytes)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::StartupSummary { input } => {
            let bytes = read_input(&input)?;
            serde_json::to_vec_pretty(&startup::summarize(&bytes)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::RuntimeCapture { port, output } => {
            serde_json::to_vec_pretty(&capture::capture(&port, &output)?)
                .map_err(|_| "cannot serialize summary")?
        }
        Command::RuntimeReplay { capture } => {
            serde_json::to_vec_pretty(&capture::replay(&capture)?)
                .map_err(|_| "cannot serialize summary")?
        }
    };
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&output)
        .and_then(|()| stdout.write_all(b"\n"))
        .map_err(|_| "cannot write summary")
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            // Help/version text is generated only from static program metadata.
            return if error.print().is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Err(_) => {
            eprintln!("identityctl: invalid arguments; use --help");
            return ExitCode::from(2);
        }
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("identityctl: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod profile_cli_tests {
    use super::*;

    #[test]
    fn restart_commands_require_bound_inputs_and_use_static_help() {
        let cli = Cli::try_parse_from([
            "identityctl",
            "wifi-ap-restart-trial",
            "--port-file",
            "synthetic-port",
            "--profile",
            "synthetic-profile",
            "--output",
            "synthetic-output",
        ])
        .unwrap_or_else(|_| panic!("valid restart arguments rejected"));
        match cli.command {
            Command::WifiApRestartTrial {
                port_file,
                profile,
                output,
            } => {
                assert_eq!(port_file, PathBuf::from("synthetic-port"));
                assert_eq!(profile, PathBuf::from("synthetic-profile"));
                assert_eq!(output, PathBuf::from("synthetic-output"));
            }
            _ => panic!("wrong restart command"),
        }
        let cli = Cli::try_parse_from([
            "identityctl",
            "wifi-ap-restart-replay",
            "--capture",
            "synthetic-capture",
        ])
        .unwrap_or_else(|_| panic!("valid restart replay rejected"));
        assert!(matches!(cli.command, Command::WifiApRestartReplay { .. }));
        assert!(
            Cli::try_parse_from([
                "identityctl",
                "wifi-ap-restart-trial",
                "--port-file",
                "synthetic-port",
                "--output",
                "synthetic-output"
            ])
            .is_err()
        );
        for command in ["wifi-ap-restart-trial", "wifi-ap-restart-replay"] {
            let help = match Cli::try_parse_from(["PRIVATE_ARGV_SENTINEL", command, "--help"]) {
                Err(help) => help,
                Ok(_) => panic!("help did not terminate parsing"),
            };
            assert_eq!(help.kind(), ErrorKind::DisplayHelp);
            let text = help.to_string();
            assert!(text.contains("identityctl"));
            assert!(!text.contains("PRIVATE_ARGV_SENTINEL"));
        }
    }

    #[test]
    fn offline_profile_command_parses_both_bundle_paths_and_keeps_help_static() {
        let cli = Cli::try_parse_from([
            "identityctl",
            "wifi-ap-compare-profiles",
            "--before",
            "synthetic-before",
            "--after",
            "synthetic-after",
        ])
        .unwrap_or_else(|_| panic!("valid profile arguments rejected"));
        match cli.command {
            Command::WifiApCompareProfiles { before, after } => {
                assert_eq!(before, PathBuf::from("synthetic-before"));
                assert_eq!(after, PathBuf::from("synthetic-after"));
            }
            _ => panic!("wrong profile command"),
        }
        let help = match Cli::try_parse_from([
            "PRIVATE_ARGV_SENTINEL",
            "wifi-ap-compare-profiles",
            "--help",
        ]) {
            Err(help) => help,
            Ok(_) => panic!("help did not terminate parsing"),
        };
        assert_eq!(help.kind(), ErrorKind::DisplayHelp);
        let text = help.to_string();
        assert!(text.contains("identityctl"));
        assert!(!text.contains("PRIVATE_ARGV_SENTINEL"));
        assert!(
            Cli::try_parse_from([
                "identityctl",
                "wifi-ap-compare-profiles",
                "--before",
                "synthetic-before"
            ])
            .is_err()
        );
    }
}
