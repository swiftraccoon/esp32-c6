//! UART-controlled software MAC-table probe; no radio service is initialized.
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unsafe_code)]
mod mac;
mod probe;

use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_sys::{ESP_ERR_INVALID_SIZE, ESP_ERR_TIMEOUT, EspError};
use runtime_protocol::{
    MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, Phase, Response, Snapshot, decode_request,
    encode_response,
};

const PROTOCOL_FAILURE: i32 = -1;

fn perform(phase: Phase, saved: Option<&Snapshot>) -> Result<Snapshot, i32> {
    let outcome = match phase {
        Phase::Defaults => mac::snapshot(),
        Phase::BaseOverride => mac::set_base().and_then(|()| mac::snapshot()),
        Phase::InterfaceOverride => mac::set_interfaces().and_then(|()| mac::snapshot()),
        Phase::Restored => match saved {
            Some(saved) => mac::restore(saved).and_then(|()| mac::snapshot()),
            None => return Err(probe::PHASE_FAILURE),
        },
    };
    outcome.map_err(|error| error.code())
}

fn send(uart: &UartDriver<'_>, response: &Response) -> Result<(), EspError> {
    let mut bytes = [0; MAX_RESPONSE_BYTES];
    let length = encode_response(response, &mut bytes)
        .map_err(|_| EspError::from_infallible::<ESP_ERR_INVALID_SIZE>())?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut offset = 0;
    while offset < length {
        let written = uart.write_nb(&bytes[offset..length])?;
        if written > length - offset {
            return Err(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>());
        }
        offset += written;
        if offset < length {
            if Instant::now() >= deadline {
                return Err(EspError::from_infallible::<ESP_ERR_TIMEOUT>());
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    Ok(())
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
            .baudrate(Hertz(115200))
            .rx_fifo_size(512)
            .tx_fifo_size(0),
    )?;
    let mut probe = probe::Probe::new();
    let mut frame = [0; MAX_REQUEST_BYTES];
    let mut length = 0;
    let mut overflow = false;
    let mut input = [0; 128];
    loop {
        let count = match uart.read(&mut input, 0) {
            Ok(count) => count,
            Err(error) if error.code() == ESP_ERR_TIMEOUT => 0,
            Err(error) => return Err(error),
        };
        if count > input.len() {
            return Err(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>());
        }
        for &byte in &input[..count] {
            if byte == b'\n' {
                let response = if overflow {
                    Response::new(0, Phase::Defaults, PROTOCOL_FAILURE, None)
                } else {
                    match decode_request(&frame[..length]) {
                        Ok(request) => probe.handle(request, perform),
                        Err(_) => Response::new(0, Phase::Defaults, PROTOCOL_FAILURE, None),
                    }
                };
                // A failed/partial transport reply does not erase saved state.
                // The host rejects it and may still request Restored. SDK I/O
                // itself has no claimed hard real-time bound.
                let _ = send(&uart, &response);
                length = 0;
                overflow = false;
            } else if !overflow {
                if length == MAX_REQUEST_BYTES - 1 {
                    overflow = true;
                } else {
                    frame[length] = byte;
                    length += 1;
                }
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
}
