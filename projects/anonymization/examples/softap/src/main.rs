//! Private attached-host SoftAP integration. Ordinary SDK logs are disabled.
#![deny(unsafe_op_in_unsafe_fn)]

use std::thread;
use std::time::{Duration, Instant};

use c6_anonymization_esp_idf::{Epoch, SoftAp};
use c6_anonymization_softap_example::{
    ADAPTER_ERROR, Domain, Expected, Operation, PROTOCOL_ERROR, REPLY_BYTES, REQUEST_BYTES, Reply,
    Session, decode, encode,
};
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::uart::{UartConfig, UartDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_sys::{ESP_ERR_TIMEOUT, EspError};

fn milliseconds(epoch: Instant) -> u64 {
    u64::try_from(epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn poll(session: &mut Session, radio: &mut SoftAp, epoch: Instant) {
    if session.cleanup_due(milliseconds(epoch)) {
        session.stopped(radio.stop().map_err(|error| error.code()), false);
    }
}

fn expected(radio: &SoftAp, epoch: Epoch) -> Expected {
    let identity = radio.identity(epoch);
    Expected {
        mac: identity.mac(),
        // The dependency's private constructor produces exactly 21 ASCII bytes.
        ssid: identity
            .ssid()
            .as_bytes()
            .try_into()
            .expect("identity SSID width"),
    }
}

fn send(
    uart: &UartDriver<'_>,
    session: &mut Session,
    radio: &mut SoftAp,
    clock: Instant,
    reply: &Reply,
) -> Result<(), i32> {
    let mut bytes = [0; REPLY_BYTES];
    let length = encode(reply, &mut bytes)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut offset = 0;
    while offset < length {
        poll(session, radio, clock);
        let written = uart
            .write_nb(&bytes[offset..length])
            .map_err(|error| error.code())?;
        if written > length - offset || Instant::now() >= deadline {
            return Err(PROTOCOL_ERROR);
        }
        offset += written;
        if offset < length {
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
    // SAFETY: Only UART was initialized. No ADC/RF subsystem or interface MAC
    // getter ran, no concurrent task uses them, and this is the sole startup
    // preparation with the concrete Modem. The adapter disables SAR entropy
    // before constructing Wi-Fi and retains all SDK ownership until reset.
    let mut radio = match unsafe { SoftAp::initialize_at_startup(peripherals.modem) } {
        Ok(radio) => radio,
        Err(error) => {
            let message = format!(
                "{{\"domain\":\"anon_softap\",\"version\":1,\"initialization_failed\":true,\"status\":{}}}\n",
                error.code()
            );
            let _ = uart.write(message.as_bytes());
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
    };
    let mut session = Session::default();
    let clock = Instant::now();
    let mut frame = [0; REQUEST_BYTES];
    let mut length = 0;
    let mut overflow = false;
    let mut input = [0; 128];
    loop {
        poll(&mut session, &mut radio, clock);
        let count = match uart.read(&mut input, 0) {
            Ok(count) if count <= input.len() => count,
            Err(error) if error.code() == ESP_ERR_TIMEOUT => 0,
            _ => {
                session.fail(PROTOCOL_ERROR);
                session.request_cleanup(milliseconds(clock));
                0
            }
        };
        for &byte in &input[..count] {
            poll(&mut session, &mut radio, clock);
            if byte != b'\n' {
                if !overflow && length < REQUEST_BYTES - 1 {
                    frame[length] = byte;
                    length += 1;
                } else {
                    overflow = true;
                    session.fail(PROTOCOL_ERROR);
                    session.request_cleanup(milliseconds(clock));
                }
                continue;
            }
            let request = if overflow {
                Err(PROTOCOL_ERROR)
            } else {
                decode(&frame[..length])
            };
            let mut status = 0;
            let (id, operation) = match request {
                Ok(request) => {
                    let admitted = session.admit(request, milliseconds(clock));
                    let result = admitted.and_then(|()| match request.operation {
                        Operation::Status => Ok(()),
                        Operation::Stop => {
                            let result = radio.stop().map_err(|error| error.code());
                            session.stopped(result, true);
                            result
                        }
                        Operation::StartFirst | Operation::StartSecond => {
                            let selected = if request.operation == Operation::StartFirst {
                                Epoch::First
                            } else {
                                Epoch::Second
                            };
                            radio.start(selected).map_err(|error| error.code())?;
                            session.start_completed(milliseconds(clock))
                        }
                    });
                    if let Err(code) = result {
                        status = code;
                    }
                    (request.id, request.operation)
                }
                Err(code) => {
                    status = code;
                    (0, Operation::Status)
                }
            };
            if status != 0 {
                session.fail(status);
                session.request_cleanup(milliseconds(clock));
            }
            poll(&mut session, &mut radio, clock);
            let readback = match radio.readback_mac() {
                Ok(mac) => Some(mac),
                Err(error) => {
                    let code = error.code();
                    status = if code == 0 { ADAPTER_ERROR } else { code };
                    session.fail(status);
                    session.request_cleanup(milliseconds(clock));
                    None
                }
            };
            let active = radio.active();
            if let Some(error) = radio.failure() {
                let code = error.code();
                status = if code == 0 { ADAPTER_ERROR } else { code };
                session.fail(status);
                session.request_cleanup(milliseconds(clock));
            }
            let reply = Reply {
                domain: Domain::SoftAp,
                version: 1,
                id,
                operation,
                status,
                failure: session.failure(),
                active,
                readback,
                first: expected(&radio, Epoch::First),
                second: expected(&radio, Epoch::Second),
            };
            if let Err(code) = send(&uart, &mut session, &mut radio, clock, &reply) {
                session.fail(code);
                session.request_cleanup(milliseconds(clock));
            }
            length = 0;
            overflow = false;
        }
        poll(&mut session, &mut radio, clock);
        thread::sleep(Duration::from_millis(5));
    }
}
