//! UART-controlled, single-start SoftAP presence witness for ESP32-C6.
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod controller;
mod radio;
#[allow(unsafe_code)]
mod storage;

use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_sys::{ESP_ERR_INVALID_SIZE, ESP_ERR_TIMEOUT, EspError};
use wifi_protocol::{
    MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, Operation, Response, decode_request, encode_response,
};

use controller::{Controller, PROTOCOL_FAILURE};
use radio::ApRadio;

fn milliseconds(epoch: Instant) -> u64 {
    u64::try_from(epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn send(
    uart: &UartDriver<'_>,
    controller: &mut Controller<ApRadio>,
    epoch: Instant,
    response: &Response,
) -> Result<(), i32> {
    let mut bytes = [0; MAX_RESPONSE_BYTES];
    let length = encode_response(response, &mut bytes).map_err(|_| ESP_ERR_INVALID_SIZE)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut offset = 0;
    while offset < length {
        controller.poll(milliseconds(epoch));
        let written = uart
            .write_nb(&bytes[offset..length])
            .map_err(|error| error.code())?;
        if written > length - offset {
            return Err(ESP_ERR_INVALID_SIZE);
        }
        offset += written;
        if offset < length {
            if Instant::now() >= deadline {
                return Err(ESP_ERR_TIMEOUT);
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
    let mut controller = Controller::new(ApRadio::new(peripherals.modem));
    let epoch = Instant::now();
    let mut frame = [0; MAX_REQUEST_BYTES];
    let mut length = 0;
    let mut overflow = false;
    let mut input = [0; 128];
    // Never return after the lazy Wi-Fi constructor could have run. Failed
    // UART/SDK operations retain ownership, request checked stop, and keep
    // polling the independent software deadline rather than invoking Drop.
    loop {
        controller.poll(milliseconds(epoch));
        let count = match uart.read(&mut input, 0) {
            Ok(count) if count <= input.len() => count,
            Ok(_) => {
                controller.transport_failed(ESP_ERR_INVALID_SIZE);
                0
            }
            Err(error) if error.code() == ESP_ERR_TIMEOUT => 0,
            Err(error) => {
                controller.transport_failed(error.code());
                0
            }
        };
        for &byte in &input[..count] {
            controller.poll(milliseconds(epoch));
            if byte == b'\n' {
                let request = if overflow {
                    None
                } else {
                    decode_request(&frame[..length]).ok()
                };
                let response = match request {
                    Some(request) => controller.handle(request, milliseconds(epoch)),
                    None => {
                        controller.transport_failed(PROTOCOL_FAILURE);
                        // No valid correlation fields can be recovered from a
                        // malformed frame. This static failure cannot satisfy
                        // a valid Start or Stop admission exchange.
                        Response::new(1, Operation::Status, PROTOCOL_FAILURE, None)
                    }
                };
                if let Err(code) = send(&uart, &mut controller, epoch, &response) {
                    controller.transport_failed(code);
                }
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
        controller.poll(milliseconds(epoch));
        thread::sleep(Duration::from_millis(5));
    }
}
