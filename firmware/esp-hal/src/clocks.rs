//! Read-only clock telemetry for the ESP32-C6 high-performance CPU and SPI0.

use core::fmt::{self, Write};

use esp_hal::peripherals::{PCR, SPI0};

/// One task-context snapshot; no application code changes clocks after HAL init.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ClockSnapshot {
    cpu_mhz: u32,
    flash_source_mhz: u32,
    flash_mhz: u32,
    flash_divider: u32,
    pcr_sysclk_conf: u32,
    pcr_cpu_freq_conf: u32,
    pcr_mspi_clk_conf: u32,
    spi0_clock: u32,
}

impl ClockSnapshot {
    /// Reads configuration registers through the upstream safe PAC interface.
    ///
    /// These registers have no read-to-clear side effects. No peripheral handle
    /// is duplicated, no clock is written, and neither runtime enables dynamic
    /// clock scaling. The four reads are sequential rather than atomic; equality
    /// of the before/after snapshots detects persistent configuration changes.
    /// This is a configuration readback, not a physical frequency measurement.
    ///
    /// Returns `None` for an unsupported source/divider. Supported configurations
    /// are reported as observed; the host enforces the controlled 160 MHz CPU /
    /// 80 MHz flash settings independently from the raw registers.
    pub(super) fn read() -> Option<Self> {
        let sysclk = PCR::regs().sysclk_conf().read();
        let cpu_freq = PCR::regs().cpu_freq_conf().read();
        let mspi_clk = PCR::regs().mspi_clk_conf().read();
        let spi_clock = SPI0::regs().clock().read();

        // ESP32-C6 SPLL is 480 MHz. SYSCLK HS division is fixed at three;
        // CPU HS uses N+1, except the explicit 120 MHz force mode at N=0.
        // The LS/RC sources require separate calibration and are not inferred.
        if sysclk.soc_clk_sel().bits() != 1 || sysclk.hs_div_num().bits() != 2 {
            return None;
        }
        let cpu_hs = u32::from(cpu_freq.cpu_hs_div_num().bits());
        let mspi_hs = u32::from(mspi_clk.mspi_fast_hs_div_num().bits());
        if !matches!(cpu_hs, 0 | 1 | 3) || !matches!(mspi_hs, 3..=5) {
            return None;
        }
        let cpu_mhz = if cpu_hs == 0 && cpu_freq.cpu_hs_120m_force().bit_is_set() {
            120
        } else {
            480 / (u32::from(sysclk.hs_div_num().bits()) + 1) / (cpu_hs + 1)
        };
        let flash_source_mhz = 480 / (mspi_hs + 1);
        let flash_divider = if spi_clock.clk_equ_sysclk().bit_is_set() {
            1
        } else {
            u32::from(spi_clock.clkcnt_n().bits()) + 1
        };
        // Reject fractional MHz instead of silently truncating the readback.
        if flash_source_mhz % flash_divider != 0 {
            return None;
        }
        let flash_mhz = flash_source_mhz / flash_divider;
        Some(Self {
            cpu_mhz,
            flash_source_mhz,
            flash_mhz,
            flash_divider,
            pcr_sysclk_conf: sysclk.bits(),
            pcr_cpu_freq_conf: cpu_freq.bits(),
            pcr_mspi_clk_conf: mspi_clk.bits(),
            spi0_clock: spi_clock.bits(),
        })
    }

    /// Writes one fixed protocol object, propagating formatting/transport errors.
    pub(super) fn write(self, mut writer: impl Write) -> fmt::Result {
        write!(
            writer,
            "{{\"source\":\"register-readback\",\"cpu_mhz\":{},\"flash_source_mhz\":{},\"flash_mhz\":{},\"flash_divider\":{},\"registers\":{{\"pcr_sysclk_conf\":{},\"pcr_cpu_freq_conf\":{},\"pcr_mspi_clk_conf\":{},\"spi0_clock\":{}}}}}",
            self.cpu_mhz,
            self.flash_source_mhz,
            self.flash_mhz,
            self.flash_divider,
            self.pcr_sysclk_conf,
            self.pcr_cpu_freq_conf,
            self.pcr_mspi_clk_conf,
            self.spi0_clock,
        )
    }
}
