//! Controlled UART entry point. No SDK radio operation occurs on boot/Status.
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
use device_identity_ble_firmware::controller::{Controller, PROTOCOL_FAILURE};
use device_identity_ble_firmware::protocol::{
    MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, Operation, Response, decode_request, encode_response,
};
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_sys::{ESP_ERR_INVALID_SIZE, ESP_ERR_TIMEOUT, EspError};
use radio::BleRadio;
use std::thread;
use std::time::{Duration, Instant};
#[allow(unsafe_code)]
mod initialization;
mod radio;
#[allow(unsafe_code)]
mod sdk;
fn now(epoch: Instant) -> u64 {
    u64::try_from(epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
}
fn send(
    uart: &UartDriver<'_>,
    controller: &mut Controller<BleRadio>,
    epoch: Instant,
    response: &Response,
) -> Result<(), i32> {
    let mut bytes = [0; MAX_RESPONSE_BYTES];
    let n = encode_response(response, &mut bytes).map_err(|_| ESP_ERR_INVALID_SIZE)?;
    let end = Instant::now() + Duration::from_secs(2);
    let mut offset = 0;
    while offset < n {
        controller.poll(now(epoch));
        let count = uart.write_nb(&bytes[offset..n]).map_err(|e| e.code())?;
        if count > n - offset {
            return Err(ESP_ERR_INVALID_SIZE);
        }
        offset += count;
        if Instant::now() >= end {
            return Err(ESP_ERR_TIMEOUT);
        }
        if offset < n {
            if Instant::now() >= end {
                return Err(ESP_ERR_TIMEOUT);
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    Ok(())
}
fn main() -> Result<(), EspError> {
    esp_idf_sys::link_patches();
    let p = Peripherals::take()?;
    let uart = UartDriver::new(
        p.uart0,
        p.pins.gpio16,
        p.pins.gpio17,
        None::<AnyIOPin>,
        None::<AnyIOPin>,
        &UartConfig::new()
            .baudrate(Hertz(115200))
            .rx_fifo_size(512)
            .tx_fifo_size(0),
    )?;
    let epoch = Instant::now();
    let mut controller = Controller::new(BleRadio::new(p.modem, epoch));
    let mut frame = [0; MAX_REQUEST_BYTES];
    let (mut length, mut overflow) = (0, false);
    let mut input = [0; 128];
    // Never normally return after a possible initialization. On failures retain
    // ownership, inhibit Starts and continue independent Stop/retry polling.
    loop {
        controller.poll(now(epoch));
        let count = match uart.read(&mut input, 0) {
            Ok(n) if n <= input.len() => n,
            Ok(_) => {
                controller.transport_failed(ESP_ERR_INVALID_SIZE, now(epoch));
                0
            }
            Err(e) if e.code() == ESP_ERR_TIMEOUT => 0,
            Err(e) => {
                controller.transport_failed(e.code(), now(epoch));
                0
            }
        };
        for &byte in &input[..count] {
            controller.poll(now(epoch));
            if byte == b'\n' {
                let request = if overflow {
                    None
                } else {
                    decode_request(&frame[..length]).ok()
                };
                let response = match request {
                    Some(r) => controller.handle(r, now(epoch)),
                    None => {
                        controller.transport_failed(PROTOCOL_FAILURE, now(epoch));
                        Response::new(1, Operation::Status, PROTOCOL_FAILURE, None)
                    }
                };
                if let Err(code) = send(&uart, &mut controller, epoch, &response) {
                    controller.transport_failed(code, now(epoch));
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
        controller.poll(now(epoch));
        thread::sleep(Duration::from_millis(5));
    }
}
