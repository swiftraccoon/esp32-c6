//! Bare-metal ESP32-C6 benchmark entry point; see the firmware README for wiring.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::uart::{Config, Uart};

#[path = "../clocks.rs"]
mod clocks;
#[path = "../measurements.rs"]
mod measurements;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);
    let peripherals = esp_hal::init(config);
    let Ok(mut uart) = Uart::new(peripherals.UART0, Config::default()) else {
        halt();
    };
    uart = uart.with_tx(peripherals.GPIO16).with_rx(peripherals.GPIO17);

    // Let a host collector attach after reset; no measurements include this wait.
    Delay::new().delay_millis(2000);
    if measurements::run(&mut uart).is_err() {
        halt();
    }
    if uart.flush().is_err() {
        halt();
    }
    halt()
}

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// Panics stop the run without a completion record or an automatic reboot.
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    halt()
}
