//! Embassy on esp-rtos benchmark entry point; see the README for wiring.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config, Uart};

#[path = "../clocks.rs"]
mod clocks;
#[path = "../measurements.rs"]
mod measurements;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);
    let peripherals = esp_hal::init(config);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let Ok(mut uart) = Uart::new(peripherals.UART0, Config::default()) else {
        halt();
    };
    uart = uart.with_tx(peripherals.GPIO16).with_rx(peripherals.GPIO17);

    // Also exercises the actual Embassy timer driver before the timed workload.
    Timer::after_millis(2000).await;
    if measurements::run_embassy(&mut uart, spawner).await.is_err() {
        halt();
    }
    if uart.flush().is_err() {
        halt();
    }
    core::future::pending::<()>().await;
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
