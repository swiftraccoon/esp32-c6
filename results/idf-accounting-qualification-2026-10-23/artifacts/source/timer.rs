//! Version-pinned raw SYSTIMER access and read-only configuration evidence.

use std::io::{self, Write};

pub(super) const TICK_HZ: u64 = 16_000_000;
const COUNTER_LIMIT: u64 = 1 << 52;

// This private ABI is declared by the selected SDK's implementation, not sys's
// generated public headers. Re-review on any SDK change: IDF 5.5.6 commit
// 880a203ae603d08a94d06aa01e0d6761958e5e9e, esp_timer_impl_systimer.c.
unsafe extern "C" {
    fn esp_timer_impl_get_counter_reg() -> u64;
}

/// An absolute 52-bit counter snapshot; never interpreted as CPU cycles.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Tick(pub(super) u64);

impl Tick {
    pub(super) fn now() -> Self {
        // SAFETY: Called only from initialized IDF task context. The selected
        // no-argument C function returns uint64_t raw UNIT0 SYSTIMER snapshots;
        // its SDK HAL owns the update/valid/LO-HI-LO handshake. No pointer or
        // callback crosses this private ABI. Linkage is checked for each ELF.
        Self(unsafe { esp_timer_impl_get_counter_reg() })
    }

    pub(super) fn elapsed(self, end: Self) -> io::Result<u64> {
        if self.0 >= COUNTER_LIMIT || end.0 >= COUNTER_LIMIT {
            return Err(io::Error::other("raw SYSTIMER exceeds 52 bits"));
        }
        end.0
            .checked_sub(self.0)
            .ok_or_else(|| io::Error::other("raw SYSTIMER moved backwards"))
    }
}

pub(super) fn legacy_micros() -> io::Result<u64> {
    // SAFETY: The generated no-argument public SDK timer binding is callable
    // after startup in task context and retains no resources or pointers.
    let micros = unsafe { esp_idf_sys::esp_timer_get_time() };
    u64::try_from(micros).map_err(|_| io::Error::other("legacy timer returned negative time"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClockRegisters {
    pcr_sysclk_conf: u32,
    pcr_cpu_freq_conf: u32,
    pcr_mspi_clk_conf: u32,
    spi0_clock: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TimerRegisters {
    systimer_conf: u32,
    pcr_systimer_conf: u32,
    pcr_systimer_func_clk_conf: u32,
}

/// Exact raw configuration, required stable before/after the full experiment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Configuration {
    clocks: ClockRegisters,
    timer: TimerRegisters,
}

impl Configuration {
    pub(super) fn read() -> io::Result<Self> {
        // SAFETY: All seven addresses are fixed, mapped, naturally aligned C6
        // configuration registers from the pinned pcr_reg.h/systimer_reg.h/
        // spi_mem_reg.h. None is FIFO/read-to-clear state. Reads create no Rust
        // references or clock ownership; no authored task writes these fields.
        // PM is disabled. Volatile serves MMIO, not inter-task synchronization.
        let value = unsafe {
            Self {
                clocks: ClockRegisters {
                    pcr_sysclk_conf: std::ptr::read_volatile(
                        (esp_idf_sys::DR_REG_PCR_BASE + 0x110) as *const u32,
                    ),
                    pcr_cpu_freq_conf: std::ptr::read_volatile(
                        (esp_idf_sys::DR_REG_PCR_BASE + 0x118) as *const u32,
                    ),
                    pcr_mspi_clk_conf: std::ptr::read_volatile(
                        (esp_idf_sys::DR_REG_PCR_BASE + 0x1c) as *const u32,
                    ),
                    spi0_clock: std::ptr::read_volatile(
                        (esp_idf_sys::DR_REG_SPI0_BASE + 0x14) as *const u32,
                    ),
                },
                timer: TimerRegisters {
                    systimer_conf: std::ptr::read_volatile(
                        esp_idf_sys::DR_REG_SYSTIMER_BASE as *const u32,
                    ),
                    pcr_systimer_conf: std::ptr::read_volatile(
                        (esp_idf_sys::DR_REG_PCR_BASE + 0x54) as *const u32,
                    ),
                    pcr_systimer_func_clk_conf: std::ptr::read_volatile(
                        (esp_idf_sys::DR_REG_PCR_BASE + 0x58) as *const u32,
                    ),
                },
            }
        };
        let clocks = value.clocks;
        if (clocks.pcr_sysclk_conf >> 16) & 3 != 1
            || (clocks.pcr_sysclk_conf >> 8) & 0xff != 2
            || (clocks.pcr_cpu_freq_conf >> 8) & 0xff != 0
            || clocks.pcr_cpu_freq_conf & (1 << 16) != 0
            || (clocks.pcr_mspi_clk_conf >> 8) & 0xff != 5
            || clocks.spi0_clock & (1 << 31) == 0
        {
            return Err(io::Error::other(
                "CPU/flash clock configuration is off baseline",
            ));
        }
        let timer = value.timer;
        if timer.pcr_systimer_conf & 3 != 1
            || timer.pcr_systimer_func_clk_conf & (1 << 20) != 0
            || timer.pcr_systimer_func_clk_conf & (1 << 22) == 0
            || timer.systimer_conf & (1 << 30) == 0
            || timer.systimer_conf & (1 << 28) == 0
        {
            return Err(io::Error::other(
                "SYSTIMER source/enable/stall configuration is off baseline",
            ));
        }
        Ok(value)
    }

    pub(super) fn write_json(self, mut output: impl Write) -> io::Result<()> {
        write!(
            output,
            "{{\"clocks\":{{\"source\":\"register-readback\",\"cpu_mhz\":160,\"flash_source_mhz\":80,\"flash_mhz\":80,\"flash_divider\":1,\"registers\":{{\"pcr_sysclk_conf\":{},\"pcr_cpu_freq_conf\":{},\"pcr_mspi_clk_conf\":{},\"spi0_clock\":{}}}}},\"timer\":{{\"source\":\"xtal\",\"enabled\":true,\"unit0_enabled\":true,\"unit0_stalls_cpu\":true,\"systimer_conf\":{},\"pcr_systimer_conf\":{},\"pcr_systimer_func_clk_conf\":{}}}}}",
            self.clocks.pcr_sysclk_conf,
            self.clocks.pcr_cpu_freq_conf,
            self.clocks.pcr_mspi_clk_conf,
            self.clocks.spi0_clock,
            self.timer.systimer_conf,
            self.timer.pcr_systimer_conf,
            self.timer.pcr_systimer_func_clk_conf,
        )
    }
}
