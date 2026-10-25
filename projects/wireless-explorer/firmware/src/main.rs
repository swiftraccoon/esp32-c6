//! Dual serial control and bounded passive capture windows on DevKitC-1-N8.
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unsafe_code)]
mod buffer;
#[allow(unsafe_code)]
mod radios;

use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_hal::usb_serial::{UsbSerialConfig, UsbSerialDriver};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{
    ClientConfiguration, Configuration, WifiDriver,
    config::{ScanConfig, ScanType},
};
use esp_idf_sys::{ESP_ERR_INVALID_SIZE, ESP_ERR_TIMEOUT, EspError};
use wireless_protocol::{
    Command, FailureStage, LineDecoder, MAX_LINE_BYTES, Reply, Response, Transport, decode_request,
    encode,
};

enum Serial<'a> {
    Uart(UartDriver<'a>),
    Usb(UsbSerialDriver<'a>),
}

impl Serial<'_> {
    fn transport(&self) -> Transport {
        match self {
            Self::Uart(_) => Transport::Uart,
            Self::Usb(_) => Transport::NativeUsb,
        }
    }

    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, EspError> {
        match self {
            Self::Uart(driver) => match driver.read(bytes, 0) {
                Err(error) if error.code() == ESP_ERR_TIMEOUT => Ok(0),
                result => result,
            },
            Self::Usb(driver) => driver.read(bytes, 0),
        }
    }

    fn send(&mut self, response: &Response) -> Result<(), EspError> {
        let mut bytes = [0; MAX_LINE_BYTES + 1];
        let len = encode(response, &mut bytes)
            .map_err(|_| EspError::from_infallible::<ESP_ERR_INVALID_SIZE>())?;
        self.send_bytes(&bytes[..len])
    }

    fn stream(&mut self, record: &wireless_protocol::capture::Record) -> Result<(), EspError> {
        let mut bytes = [0; wireless_protocol::capture::MAX_LINE + 1];
        let len = wireless_protocol::capture::encode(record, &mut bytes)
            .map_err(|_| EspError::from_infallible::<ESP_ERR_INVALID_SIZE>())?;
        self.send_bytes(&bytes[..len])
    }

    fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), EspError> {
        let len = bytes.len();
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut offset = 0;
        while offset < len {
            let written = match self {
                Self::Uart(driver) => driver.write_nb(&bytes[offset..len])?,
                Self::Usb(driver) => driver.write(&bytes[offset..len], 0)?,
            };
            if written > len - offset {
                return Err(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>());
            }
            offset += written;
            if offset == len {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(EspError::from_infallible::<ESP_ERR_TIMEOUT>());
            }
            thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}

fn main() -> Result<(), EspError> {
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take()?;
    let uart = UartDriver::new(
        peripherals.uart0,
        peripherals.pins.gpio16,
        peripherals.pins.gpio17,
        None::<AnyIOPin>,
        None::<AnyIOPin>,
        &UartConfig::new()
            .baudrate(Hertz(921600))
            .rx_fifo_size(1024)
            .tx_fifo_size(0),
    )?;
    let usb = UsbSerialDriver::new(
        peripherals.usb_serial,
        peripherals.pins.gpio12,
        peripherals.pins.gpio13,
        &UsbSerialConfig::new()
            .rx_buffer_size(1024)
            .tx_buffer_size(1024),
    )?;
    let mut ports = [Serial::Uart(uart), Serial::Usb(usb)];
    let mut decoders = [LineDecoder::new(), LineDecoder::new()];
    let mut counters = [(0_u32, 0_u32); 2];
    let event_loop = EspSystemEventLoop::take()?;
    // None selects RAM-only Wi-Fi configuration; no credentials or NVS writes.
    let mut radios = radios::Radios::new(peripherals.modem, event_loop);
    let mut buffer = [0; 128];
    loop {
        for ((port, decoder), (protocol_errors, tx_failures)) in
            ports.iter_mut().zip(&mut decoders).zip(&mut counters)
        {
            let count = port.read(&mut buffer)?;
            if count > buffer.len() {
                return Err(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>());
            }
            for &byte in &buffer[..count] {
                if let Some(line) = decoder.push(byte) {
                    let request = line.and_then(decode_request);
                    let response = match request {
                        Ok(request) => {
                            if let Command::Observe {
                                radio,
                                channel,
                                millis,
                            } = request.command()
                            {
                                if !radio.valid_window(channel, millis) {
                                    port.send(&Response::new(
                                        request.id(),
                                        Reply::Error {
                                            stage: FailureStage::Capture,
                                            code: esp_idf_sys::ESP_ERR_INVALID_ARG,
                                        },
                                    ))?;
                                    continue;
                                }
                                radios.observe(port, request.id(), radio, channel, millis)?;
                                continue;
                            }
                            let reply = match request.command() {
                                Command::Info => Reply::Info {
                                    transport: port.transport(),
                                    firmware_revision: 2,
                                    wifi_scan_implemented: true,
                                    ble_implemented: true,
                                    ieee802154_implemented: true,
                                    protocol_errors: *protocol_errors,
                                    tx_failures: *tx_failures,
                                },
                                Command::Ping { nonce } => Reply::Pong { nonce },
                                Command::WifiScan => match &mut radios.wifi {
                                    Ok(wifi) => scan(wifi).unwrap_or_else(wifi_error),
                                    Err(error) => wifi_error(*error),
                                },
                                Command::Observe { .. } => {
                                    unreachable!("handled before reply dispatch")
                                }
                            };
                            Response::new(request.id(), reply)
                        }
                        Err(_) => {
                            *protocol_errors = protocol_errors.saturating_add(1);
                            Response::new(
                                0,
                                Reply::Error {
                                    stage: FailureStage::Protocol,
                                    code: -1,
                                },
                            )
                        }
                    };
                    // Stop waiting after one second if a reader disappears. A
                    // partial reply is rejected by the host, never a success.
                    // Continue serving subsequent requests on either port.
                    if port.send(&response).is_err() {
                        *tx_failures = tx_failures.saturating_add(1);
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn wifi_error(error: EspError) -> Reply {
    Reply::Error {
        stage: FailureStage::Wifi,
        code: error.code(),
    }
}

fn scan(wifi: &mut WifiDriver<'_>) -> Result<Reply, EspError> {
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
    wifi.start()?;
    let result = (|| {
        // No probe requests, association, credentials or application callbacks.
        // Waiting occurs in the main task, never in an interrupt/radio callback.
        wifi.start_scan(
            &ScanConfig {
                scan_type: ScanType::Passive(Duration::from_millis(120)),
                show_hidden: true,
                ..ScanConfig::default()
            },
            false,
        )?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while !wifi.is_scan_done()? {
            if Instant::now() >= deadline {
                let _ = wifi.stop_scan();
                return Err(EspError::from_infallible::<ESP_ERR_TIMEOUT>());
            }
            thread::sleep(Duration::from_millis(20));
        }
        let (records, total) = wifi.get_scan_result_n::<32>()?;
        let mut channels = [0; 14];
        let mut out_of_range = 0;
        for record in &records {
            if let Some(count) = record
                .channel
                .checked_sub(1)
                .and_then(|index| channels.get_mut(usize::from(index)))
            {
                *count += 1;
            } else {
                out_of_range += 1;
            }
        }
        let total_found = u16::try_from(total)
            .map_err(|_| EspError::from_infallible::<ESP_ERR_INVALID_SIZE>())?;
        let summarized = records.len() as u16; // bounded by the 32-record array
        let unreported = total_found
            .checked_sub(summarized)
            .ok_or(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>())?;
        Ok(Reply::WifiScan {
            total_found,
            summarized,
            unreported,
            channels,
            out_of_range,
        })
    })();
    let stopped = wifi.stop();
    // Attempt shutdown even when scan/retrieval fails; preserve the first error.
    let reply = result?;
    stopped?;
    Ok(reply)
}
