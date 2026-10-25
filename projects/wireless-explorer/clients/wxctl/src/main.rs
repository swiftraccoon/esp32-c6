//! Desktop serial client. Captures are functional probes, never benchmark scores.
#![forbid(unsafe_code)]

mod evidence;
mod observation;
mod transcript;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand, ValueEnum};
use wireless_protocol::{Command, MAX_LINE_BYTES, Request, Response, encode};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// List serial ports without opening or resetting any device.
    Ports,
    /// Send one request; retain the request, received bytes, and outcome.
    Exchange {
        #[arg(long)]
        port: String,
        /// Revision 2 firmware uses 921600; revision 1 used 115200.
        #[arg(long, default_value_t = 921600)]
        baud: u32,
        /// Must not already exist; failed attempts are preserved too.
        #[arg(long)]
        output: PathBuf,
        #[arg(long, value_enum)]
        command: Operation,
        #[arg(long, default_value_t = 123456789)]
        nonce: u32,
        #[arg(long, default_value_t = 15, value_parser = clap::value_parser!(u64).range(1..=120))]
        timeout_seconds: u64,
    },
    /// Revalidate an existing capture and compare the saved response exactly.
    Replay {
        #[arg(long)]
        capture: PathBuf,
    },
    /// Replay every functional capture declared by the project ledger.
    VerifyProbes {
        #[arg(long)]
        ledger: PathBuf,
    },
    /// Record passive radio windows in one serial session; raw bytes stay local.
    Observe {
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 921600)]
        baud: u32,
        /// New directory; its parent must already exist. Prefer private/ storage.
        #[arg(long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value_t = Selection::All)]
        radio: Selection,
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "1,2,3,4,5,6,7,8,9,10,11,12,13"
        )]
        wifi_channels: Vec<u8>,
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26"
        )]
        ieee802154_channels: Vec<u8>,
        #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u32).range(100..=60000))]
        dwell_ms: u32,
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=100))]
        cycles: u16,
    },
    /// Reconstruct packets, activity, losses and window timing from saved bytes.
    ReplayObservation {
        #[arg(long)]
        capture: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Selection {
    All,
    Wifi,
    Ble,
    Ieee802154,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Operation {
    Info,
    Ping,
    WifiScan,
}

fn main() -> Result<()> {
    match Args::parse().action {
        Action::Ports => {
            for port in serialport::available_ports()? {
                match port.port_type {
                    serialport::SerialPortType::UsbPort(info) => println!(
                        "{} USB {:04x}:{:04x} {}",
                        port.port_name,
                        info.vid,
                        info.pid,
                        info.product.as_deref().unwrap_or("unknown")
                    ),
                    _ => println!("{}", port.port_name),
                }
            }
        }
        Action::Exchange {
            port,
            baud,
            output,
            command,
            nonce,
            timeout_seconds,
        } => {
            let command = match command {
                Operation::Info => Command::Info,
                Operation::Ping => Command::Ping { nonce },
                Operation::WifiScan => Command::WifiScan,
            };
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
            // A correlation ID only: collisions are possible; this is not authentication.
            let id = (now.as_secs() as u32)
                .wrapping_mul(1_000_000_000)
                .wrapping_add(now.subsec_nanos());
            let request = Request::new(id, command);
            fs::create_dir(&output)
                .context("create a new capture directory (parent must exist)")?;
            let result = exchange(
                &port,
                baud,
                &output,
                request,
                Duration::from_secs(timeout_seconds),
            );
            if let Err(error) = &result {
                fs::write(output.join("failure.txt"), format!("{error:#}\n"))?;
            }
            let response = result?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Action::Replay { capture } => {
            let response = replay(&capture)?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Action::VerifyProbes { ledger } => {
            let count = evidence::verify(&ledger)?;
            println!(
                "{count} functional captures replayed; no hardware, provenance, or comparative qualification"
            );
        }
        Action::Observe {
            port,
            baud,
            output,
            radio,
            wifi_channels,
            ieee802154_channels,
            dwell_ms,
            cycles,
        } => {
            use wireless_protocol::capture::Radio;
            let radios: &[Radio] = match radio {
                Selection::All => &[Radio::Wifi, Radio::Ble, Radio::Ieee802154],
                Selection::Wifi => &[Radio::Wifi],
                Selection::Ble => &[Radio::Ble],
                Selection::Ieee802154 => &[Radio::Ieee802154],
            };
            let plan = observation::plan(
                radios,
                &wifi_channels,
                &ieee802154_channels,
                dwell_ms,
                cycles,
            )?;
            let summaries = observation::capture(&port, baud, &output, plan)?;
            println!(
                "{} windows captured and replayed in {}",
                summaries.len(),
                output.display()
            );
        }
        Action::ReplayObservation { capture } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&observation::replay(&capture)?)?
            );
        }
    }
    Ok(())
}

fn exchange(
    port: &str,
    baud: u32,
    output: &Path,
    request: Request,
    timeout: Duration,
) -> Result<Response> {
    let mut wire = [0; MAX_LINE_BYTES + 1];
    let len = encode(&request, &mut wire)?;
    fs::write(output.join("request.frame"), &wire[..len])?;
    fs::write(
        output.join("connection.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "kind": "wireless-explorer-functional-probe-v1", "port": port, "baud": baud,
            "timeout_seconds": timeout.as_secs(), "client_version": env!("CARGO_PKG_VERSION"),
        "reset_requested": false, "control_lines": "DTR and RTS deasserted",
        "startup_capture_ms": 1000
        }))?,
    )?;
    let mut raw = File::create(output.join("serial.raw"))?;
    let mut serial = serialport::new(port, baud)
        .timeout(Duration::from_millis(100))
        .dtr_on_open(false)
        .open()
        .with_context(|| format!("open {port}"))?;
    // Opening a CP2102N tty can leave RTS asserted, holding this board in reset.
    // Release both auto-programming lines without requesting a reset pulse.
    serial.write_data_terminal_ready(false)?;
    serial.write_request_to_send(false)?;
    // Preserve startup bytes separately, before transmitting the request. Serial
    // open/control-line behavior may still reset a board despite no reset pulse.
    let mut startup = File::create(output.join("startup.raw"))?;
    let settle = Instant::now() + Duration::from_secs(1);
    let mut buffer = [0; 256];
    let mut startup_received = 0;
    while Instant::now() < settle {
        match serial.read(&mut buffer) {
            Ok(len) => {
                startup.write_all(&buffer[..len])?;
                startup_received += len;
                ensure!(
                    startup_received <= transcript::MAX_CAPTURE_BYTES,
                    "startup byte limit exceeded"
                );
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error.into()),
        }
    }
    startup.sync_all()?;
    serial.write_all(&wire[..len])?;
    let deadline = Instant::now() + timeout;
    let mut transcript = transcript::Transcript::new(request);
    let mut received = 0;
    let mut completed = None;
    loop {
        if Instant::now() >= deadline {
            bail!("response deadline expired");
        }
        match serial.read(&mut buffer) {
            Ok(0) => {}
            Ok(len) => {
                raw.write_all(&buffer[..len])?;
                received += len;
                ensure!(
                    received <= transcript::MAX_CAPTURE_BYTES,
                    "capture byte limit exceeded"
                );
                transcript.feed(&buffer[..len])?;
                if transcript.has_response() && completed.is_none() {
                    completed = Some(Instant::now());
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error.into()),
        }
        // Retain a short post-response window for duplicate/fatal output. It is
        // not proof that no reset or extra output occurs after the capture ends.
        if completed.is_some_and(|time| time.elapsed() >= Duration::from_millis(300)) {
            break;
        }
    }
    raw.sync_all()?;
    let response = transcript.finish()?;
    fs::write(
        output.join("response.json"),
        serde_json::to_vec_pretty(&response)?,
    )?;
    let replayed = replay(output)?;
    ensure!(
        replayed == response,
        "saved capture disagrees with live validation"
    );
    Ok(response)
}

fn replay(capture: &Path) -> Result<Response> {
    match fs::symlink_metadata(capture.join("failure.txt")) {
        Ok(_) => bail!("capture retains a failure marker"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let request_bytes = read_bounded(&capture.join("request.frame"), MAX_LINE_BYTES + 1)?;
    let request = transcript::request_frame(&request_bytes)?;
    let raw = read_bounded(&capture.join("serial.raw"), transcript::MAX_CAPTURE_BYTES)?;
    let mut transcript = transcript::Transcript::new(request);
    transcript.feed(&raw)?;
    let response = transcript.finish()?;
    let saved: Response =
        serde_json::from_slice(&read_bounded(&capture.join("response.json"), 4096)?)?;
    ensure!(response == saved, "saved response differs from raw capture");
    Ok(response)
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)
        .with_context(|| format!("read {}", path.display()))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= limit,
        "{} exceeds size limit",
        path.display()
    );
    Ok(bytes)
}

#[cfg(test)]
mod tests;
