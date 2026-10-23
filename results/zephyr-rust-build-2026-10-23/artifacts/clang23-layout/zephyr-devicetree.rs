pub const NADDRESS_CELLS: u32 = 1u32;
pub const NSIZE_CELLS: u32 = 1u32;
pub const MODEL_DEBUG: &str = "[String(\"esp32c6_devkitc HP Core\")]";
pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32c6\")]";
pub mod aliases {
    pub const ORD: usize = 1usize;
    pub mod die_temp0 {
        pub use crate::devicetree::soc::coretemp_6000e058::*;
    }
    pub mod retainedmemdevice {
        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
    }
    pub mod sw0 {
        pub use crate::devicetree::gpio_keys::button_1::*;
    }
    pub mod watchdog0 {
        pub use crate::devicetree::soc::watchdog_60008048::*;
    }
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod chosen {
    pub const ORD: usize = 2usize;
    pub mod zephyr_canbus {
        pub use crate::devicetree::soc::can_6000b000::*;
    }
    pub mod zephyr_entropy {
        pub use crate::devicetree::soc::trng_600b2808::*;
    }
    pub mod zephyr_flash_controller {
        pub use crate::devicetree::soc::flash_controller_60002000::*;
    }
    pub mod zephyr_bt_hci {
        pub use crate::devicetree::esp32_bt_hci::*;
    }
    pub mod zephyr_sram {
        pub use crate::devicetree::soc::memory_40800000::*;
    }
    pub mod zephyr_console {
        pub use crate::devicetree::soc::uart_60000000::*;
    }
    pub mod zephyr_shell_uart {
        pub use crate::devicetree::soc::uart_60000000::*;
    }
    pub mod zephyr_flash {
        pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
    }
    pub mod zephyr_code_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
    }
    pub mod zephyr_ieee802154 {
        pub use crate::devicetree::ieee802154::*;
    }
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod cpus {
    pub const ORD: usize = 6usize;
    pub const NADDRESS_CELLS: u32 = 1u32;
    pub const NSIZE_CELLS: u32 = 0u32;
    pub mod cpu_0 {
        pub const ORD: usize = 10usize;
        pub const DEVICE_TYPE_DEBUG: &str = "[String(\"cpu\")]";
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,riscv\"), String(\"riscv\")]";
        pub const RISCV_ISA_BASE_DEBUG: &str = "[String(\"rv32i\")]";
        pub const RISCV_ISA_EXTENSIONS_DEBUG : & str = "[String(\"i\"), String(\"m\"), String(\"a\"), String(\"c\"), String(\"zicsr\"), String(\"zifencei\")]" ;
        pub const REG: u32 = 0u32;
        pub const CPU_POWER_STATES_DEBUG : & str = "[Words([Phandle(Phandle { name: \"light_sleep\", .. }), Phandle(Phandle { name: \"deep_sleep\", .. })])]" ;
        pub const CLOCK_SOURCE: u32 = 1u32;
        pub const CLOCK_FREQUENCY: u32 = 160000000u32;
        pub const XTAL_FREQ: u32 = 40000000u32;
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod power_states {
        pub const ORD: usize = 7usize;
        pub mod light_sleep {
            pub const ORD: usize = 9usize;
            pub const COMPATIBLE_DEBUG: &str = "[String(\"zephyr,power-state\")]";
            pub const POWER_STATE_NAME_DEBUG: &str = "[String(\"standby\")]";
            pub const MIN_RESIDENCY_US: u32 = 2500u32;
            pub const EXIT_LATENCY_US: u32 = 50u32;
            pub const PHANDLE: u32 = 1u32;
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod deep_sleep {
            pub const ORD: usize = 8usize;
            pub const COMPATIBLE_DEBUG: &str = "[String(\"zephyr,power-state\")]";
            pub const POWER_STATE_NAME_DEBUG: &str = "[String(\"soft-off\")]";
            pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
            pub const PHANDLE: u32 = 2u32;
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod pin_controller {
    pub const ORD: usize = 16usize;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-pinctrl\")]";
    pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
    pub mod uart0_default {
        pub const ORD: usize = 24usize;
        pub const PHANDLE: u32 = 10u32;
        pub mod group1 {
            pub const ORD: usize = 25usize;
            pub const PINMUX: u32 = 229328u32;
            pub const OUTPUT_HIGH_DEBUG: &str = "[]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod group2 {
            pub const ORD: usize = 26usize;
            pub const PINMUX: u32 = 16744849u32;
            pub const BIAS_PULL_UP_DEBUG: &str = "[]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod spim2_default {
        pub const ORD: usize = 21usize;
        pub const PHANDLE: u32 = 6u32;
        pub mod group1 {
            pub const ORD: usize = 22usize;
            pub const PINMUX_DEBUG: &str =
                "[Words([Number(16748546)]), Words([Number(2097094)]), Words([Number(2260938)])]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod group2 {
            pub const ORD: usize = 23usize;
            pub const PINMUX: u32 = 2162631u32;
            pub const OUTPUT_LOW_DEBUG: &str = "[]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod i2c0_default {
        pub const ORD: usize = 17usize;
        pub const PHANDLE: u32 = 7u32;
        pub mod group1 {
            pub const ORD: usize = 18usize;
            pub const PINMUX_DEBUG: &str = "[Words([Number(1510278)]), Words([Number(1477447)])]";
            pub const BIAS_PULL_UP_DEBUG: &str = "[]";
            pub const DRIVE_OPEN_DRAIN_DEBUG: &str = "[]";
            pub const OUTPUT_HIGH_DEBUG: &str = "[]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod i2s_default {
        pub const ORD: usize = 19usize;
        pub const PHANDLE: u32 = 9u32;
        pub mod group1 {
            pub const ORD: usize = 20usize;
            pub const PINMUX_DEBUG : & str = "[Words([Number(393989)]), Words([Number(459654)]), Words([Number(426823)]), Words([Number(524232)]), Words([Number(16745431)])]" ;
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod esp32_bt_hci {
    pub const ORD: usize = 3usize;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-bt-hci\")]";
    pub const BT_HCI_VS_EXT_DEBUG: &str = "[]";
    pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod clock {
    pub const ORD: usize = 27usize;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-clock\")]";
    pub const FAST_CLK_SRC: u32 = 0u32;
    pub const SLOW_CLK_SRC: u32 = 0u32;
    pub const NCLOCK_CELLS: u32 = 1u32;
    pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
    pub const PHANDLE: u32 = 5u32;
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod wifi {
    pub const ORD: usize = 5usize;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-wifi\")]";
    pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod ieee802154 {
    pub const ORD: usize = 4usize;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-ieee802154\")]";
    pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod soc {
    pub const ORD: usize = 11usize;
    pub const NADDRESS_CELLS: u32 = 1u32;
    pub const NSIZE_CELLS: u32 = 1u32;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"simple-bus\")]";
    pub const RANGES_DEBUG: &str = "[]";
    pub mod memory_40000000 {
        pub const ORD: usize = 38usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"zephyr,memory-region\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1073741824), Number(327680)])]";
        pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SOC_ROM\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod memory_40800000 {
        pub const ORD: usize = 39usize;
        pub const COMPATIBLE_DEBUG: &str =
            "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1082130432), Number(524288)])]";
        pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMHP\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod memory_50000000 {
        pub const ORD: usize = 68usize;
        pub const NADDRESS_CELLS: u32 = 1u32;
        pub const NSIZE_CELLS: u32 = 1u32;
        pub const COMPATIBLE_DEBUG: &str =
            "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1342177280), Number(16384)])]";
        pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMLP\")]";
        pub const RANGES_DEBUG: &str = "[Words([Number(0), Number(1342177280), Number(16384)])]";
        pub mod memory_0 {
            pub const ORD: usize = 71usize;
            pub const COMPATIBLE_DEBUG: &str =
                "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
            pub const REG_DEBUG: &str = "[Words([Number(0), Number(15360)])]";
            pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMLP_ULP\")]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod memory_3c00 {
            pub const ORD: usize = 72usize;
            pub const COMPATIBLE_DEBUG: &str =
                "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
            pub const REG_DEBUG: &str = "[Words([Number(15360), Number(16)])]";
            pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMLP_ULP_SHM\")]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod memory_3c10 {
            pub const ORD: usize = 73usize;
            pub const COMPATIBLE_DEBUG: &str =
                "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
            pub const REG_DEBUG: &str = "[Words([Number(15376), Number(232)])]";
            pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMLP_RTC\")]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod memory_3cf8 {
            pub const ORD: usize = 74usize;
            pub const COMPATIBLE_DEBUG: &str =
                "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
            pub const REG_DEBUG: &str = "[Words([Number(15608), Number(512)])]";
            pub const RANGES_DEBUG: &str = "[Words([Number(0), Number(15608), Number(512)])]";
            pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMLP_RETAINED_MEM\")]";
            pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
            pub const NADDRESS_CELLS: u32 = 1u32;
            pub const NSIZE_CELLS: u32 = 1u32;
            pub mod retainedmem {
                pub const ORD: usize = 75usize;
                pub const COMPATIBLE_DEBUG: &str = "[String(\"zephyr,retained-ram\")]";
                pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
                pub const RANGES_DEBUG: &str = "[]";
                pub const NADDRESS_CELLS: u32 = 1u32;
                pub const NSIZE_CELLS: u32 = 1u32;
                pub mod labels {
                    #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                    #[doc = r" enforces that these are unique, allowing references such as"]
                    #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                    pub mod adc0 {
                        pub use crate::devicetree::soc::adc_6000e000::*;
                    }
                    pub mod aes {
                        pub use crate::devicetree::soc::aes_60088000::*;
                    }
                    pub mod boot_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                    }
                    pub mod clock {
                        pub use crate::devicetree::clock::*;
                    }
                    pub mod coredump_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                    }
                    pub mod coretemp {
                        pub use crate::devicetree::soc::coretemp_6000e058::*;
                    }
                    pub mod cpu0 {
                        pub use crate::devicetree::cpus::cpu_0::*;
                    }
                    pub mod deep_sleep {
                        pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                    }
                    pub mod dma {
                        pub use crate::devicetree::soc::dma_60080000::*;
                    }
                    pub mod esp32_bt_hci {
                        pub use crate::devicetree::esp32_bt_hci::*;
                    }
                    pub mod flash {
                        pub use crate::devicetree::soc::flash_controller_60002000::*;
                    }
                    pub mod flash0 {
                        pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                    }
                    pub mod gpio0 {
                        pub use crate::devicetree::soc::gpio_60091000::*;
                    }
                    pub mod i2c0 {
                        pub use crate::devicetree::soc::i2c_60004000::*;
                    }
                    pub mod i2c0_default {
                        pub use crate::devicetree::pin_controller::i2c0_default::*;
                    }
                    pub mod i2s {
                        pub use crate::devicetree::soc::i2s_6000c000::*;
                    }
                    pub mod i2s_default {
                        pub use crate::devicetree::pin_controller::i2s_default::*;
                    }
                    pub mod ieee802154 {
                        pub use crate::devicetree::ieee802154::*;
                    }
                    pub mod intc {
                        pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                    }
                    pub mod ipc_shm {
                        pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                    }
                    pub mod ledc0 {
                        pub use crate::devicetree::soc::ledc_60007000::*;
                    }
                    pub mod light_sleep {
                        pub use crate::devicetree::cpus::power_states::light_sleep::*;
                    }
                    pub mod lp_rtc {
                        pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                    }
                    pub mod lp_uart {
                        pub use crate::devicetree::soc::uart_600b1400::*;
                    }
                    pub mod mbox0 {
                        pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                    }
                    pub mod mcpwm0 {
                        pub use crate::devicetree::soc::mcpwm_60014000::*;
                    }
                    pub mod pcnt {
                        pub use crate::devicetree::soc::pcnt_60012000::*;
                    }
                    pub mod pinctrl {
                        pub use crate::devicetree::pin_controller::*;
                    }
                    pub mod retainedmem {
                        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                    }
                    pub mod retainedmem0 {
                        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                    }
                    pub mod rtc_timer {
                        pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                    }
                    pub mod scratch_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                    }
                    pub mod sha {
                        pub use crate::devicetree::soc::sha_60089000::*;
                    }
                    pub mod slot0_lpcore_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                    }
                    pub mod slot0_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                    }
                    pub mod slot1_lpcore_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                    }
                    pub mod slot1_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                    }
                    pub mod soc_rom {
                        pub use crate::devicetree::soc::memory_40000000::*;
                    }
                    pub mod spi2 {
                        pub use crate::devicetree::soc::spi_60081000::*;
                    }
                    pub mod spim2_default {
                        pub use crate::devicetree::pin_controller::spim2_default::*;
                    }
                    pub mod sramhp {
                        pub use crate::devicetree::soc::memory_40800000::*;
                    }
                    pub mod sramlp {
                        pub use crate::devicetree::soc::memory_50000000::*;
                    }
                    pub mod storage_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                    }
                    pub mod sys_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                    }
                    pub mod systimer0 {
                        pub use crate::devicetree::soc::systimer_6000a000::*;
                    }
                    pub mod timer0 {
                        pub use crate::devicetree::soc::counter_60008000::*;
                    }
                    pub mod timer1 {
                        pub use crate::devicetree::soc::counter_60009000::*;
                    }
                    pub mod trng0 {
                        pub use crate::devicetree::soc::trng_600b2808::*;
                    }
                    pub mod twai0 {
                        pub use crate::devicetree::soc::can_6000b000::*;
                    }
                    pub mod twai1 {
                        pub use crate::devicetree::soc::can_6000d000::*;
                    }
                    pub mod uart0 {
                        pub use crate::devicetree::soc::uart_60000000::*;
                    }
                    pub mod uart0_default {
                        pub use crate::devicetree::pin_controller::uart0_default::*;
                    }
                    pub mod uart1 {
                        pub use crate::devicetree::soc::uart_60001000::*;
                    }
                    pub mod ulp_ram {
                        pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                    }
                    pub mod ulp_shm {
                        pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                    }
                    pub mod usb_serial {
                        pub use crate::devicetree::soc::uart_6000f000::*;
                    }
                    pub mod user_button1 {
                        pub use crate::devicetree::gpio_keys::button_1::*;
                    }
                    pub mod wdt0 {
                        pub use crate::devicetree::soc::watchdog_60008048::*;
                    }
                    pub mod wdt1 {
                        pub use crate::devicetree::soc::watchdog_60009048::*;
                    }
                    pub mod wifi {
                        pub use crate::devicetree::wifi::*;
                    }
                }
            }
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod memory_3ef8 {
            pub const ORD: usize = 69usize;
            pub const COMPATIBLE_DEBUG: &str =
                "[String(\"zephyr,memory-region\"), String(\"mmio-sram\")]";
            pub const REG_DEBUG: &str = "[Words([Number(16120), Number(256)])]";
            pub const ZEPHYR_MEMORY_REGION_DEBUG: &str = "[String(\"SRAMLP_IPC_SHM\")]";
            pub const PHANDLE: u32 = 3u32;
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod mbox_3ff8 {
            pub const ORD: usize = 70usize;
            pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,mbox-esp32\")]";
            pub const REG_DEBUG: &str = "[Words([Number(16376), Number(8)])]";
            pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
            pub const SHARED_MEMORY_DEBUG: &str =
                "[Words([Phandle(Phandle { name: \"ipc_shm\", .. })])]";
            pub const SHARED_MEMORY_SIZE: u32 = 256u32;
            pub const INTERRUPTS_DEBUG : & str = "[Words([Number(13), Number(0), Number(0)]), Words([Number(5), Number(0), Number(0)])]" ;
            pub const INTERRUPT_PARENT_DEBUG: &str =
                "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
            pub const NMBOX_CELLS: u32 = 1u32;
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod interrupt_controller_60010000 {
        pub const ORD: usize = 12usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-intc\")]";
        pub const NADDRESS_CELLS: u32 = 0u32;
        pub const NINTERRUPT_CELLS: u32 = 3u32;
        pub const INTERRUPT_CONTROLLER_DEBUG: &str = "[]";
        pub const REG_DEBUG: &str = "[Words([Number(1610678272), Number(4096)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub const PHANDLE: u32 = 4u32;
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod systimer_6000a000 {
        pub const ORD: usize = 44usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-systimer\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610653696), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(57), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod counter_60008000 {
        pub const ORD: usize = 52usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-timer\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610645504), Number(4096)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(0)])]";
        pub const GROUP: u32 = 0u32;
        pub const INDEX: u32 = 0u32;
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(51), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod counter {
            pub const ORD: usize = 53usize;
            pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-counter\")]";
            pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod counter_60009000 {
        pub const ORD: usize = 54usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-timer\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610649600), Number(4096)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(1)])]";
        pub const GROUP: u32 = 1u32;
        pub const INDEX: u32 = 0u32;
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(54), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod counter {
            pub const ORD: usize = 55usize;
            pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-counter\")]";
            pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod rtc_timer_600b0c00 {
        pub const ORD: usize = 41usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-rtc-timer\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611336704), Number(1024)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(13)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(15), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod trng_600b2808 {
        pub const ORD: usize = 45usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-trng\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611343880), Number(4)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(112)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod can_6000b000 {
        pub const ORD: usize = 30usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-twai\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610657792), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(46), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(110)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod can_6000d000 {
        pub const ORD: usize = 31usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-twai\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610665984), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(47), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(111)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod spi_60081000 {
        pub const ORD: usize = 43usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-spi\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611141120), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(72), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(109)])]";
        pub const DMA_HOST: u32 = 0u32;
        pub const NADDRESS_CELLS: u32 = 1u32;
        pub const NSIZE_CELLS: u32 = 0u32;
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub const PINCTRL_0_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"spim2_default\", .. })])]";
        pub const PINCTRL_NAMES_DEBUG: &str = "[String(\"default\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod watchdog_60008048 {
        pub const ORD: usize = 50usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-watchdog\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610645576), Number(32)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(53), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(0)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod watchdog_60009048 {
        pub const ORD: usize = 51usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-watchdog\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610649672), Number(32)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(56), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(1)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod flash_controller_60002000 {
        pub const ORD: usize = 56usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-flash-controller\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610620928), Number(4096)])]";
        pub const NADDRESS_CELLS: u32 = 1u32;
        pub const NSIZE_CELLS: u32 = 1u32;
        pub mod flash_0 {
            pub const ORD: usize = 57usize;
            pub const COMPATIBLE_DEBUG: &str = "[String(\"soc-nv-flash\")]";
            pub const ERASE_BLOCK_SIZE: u32 = 4096u32;
            pub const WRITE_BLOCK_SIZE: u32 = 4u32;
            pub const REG_DEBUG: &str = "[Words([Number(0), Number(8388608)])]";
            pub mod partitions {
                pub const ORD: usize = 58usize;
                pub const COMPATIBLE_DEBUG: &str = "[String(\"fixed-partitions\")]";
                pub const NADDRESS_CELLS: u32 = 1u32;
                pub const NSIZE_CELLS: u32 = 1u32;
                pub mod partition_0 {
                    pub const ORD: usize = 59usize;
                    pub const LABEL_DEBUG: &str = "[String(\"mcuboot\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(0), Number(65536)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 0u32, 65536u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_10000 {
                    pub const ORD: usize = 60usize;
                    pub const LABEL_DEBUG: &str = "[String(\"sys\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(65536), Number(65536)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 65536u32, 65536u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_20000 {
                    pub const ORD: usize = 61usize;
                    pub const LABEL_DEBUG: &str = "[String(\"image-0\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(131072), Number(1835008)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 131072u32, 1835008u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_1e0000 {
                    pub const ORD: usize = 62usize;
                    pub const LABEL_DEBUG: &str = "[String(\"image-1\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(1966080), Number(1835008)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 1966080u32, 1835008u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_3a0000 {
                    pub const ORD: usize = 63usize;
                    pub const LABEL_DEBUG: &str = "[String(\"image-0-lpcore\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(3801088), Number(32768)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 3801088u32, 32768u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_3a8000 {
                    pub const ORD: usize = 64usize;
                    pub const LABEL_DEBUG: &str = "[String(\"image-1-lpcore\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(3833856), Number(32768)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 3833856u32, 32768u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_3b0000 {
                    pub const ORD: usize = 65usize;
                    pub const LABEL_DEBUG: &str = "[String(\"storage\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(3866624), Number(196608)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 3866624u32, 196608u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_3e0000 {
                    pub const ORD: usize = 66usize;
                    pub const LABEL_DEBUG: &str = "[String(\"image-scratch\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(4063232), Number(126976)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 4063232u32, 126976u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod partition_3ff000 {
                    pub const ORD: usize = 67usize;
                    pub const LABEL_DEBUG: &str = "[String(\"coredump\")]";
                    pub const REG_DEBUG: &str = "[Words([Number(4190208), Number(4096)])]";
                    static UNIQUE: crate::device::Unique = crate::device::Unique::new();
                    static STATIC: crate::device::NoStatic = crate::device::NoStatic::new();
                    pub fn get_instance() -> Option<crate::device::flash::FlashPartition> {
                        unsafe {
                            let device = super::super::super::get_instance_raw();
                            crate::device::flash::FlashPartition::new(
                                &UNIQUE, &STATIC, device, 4190208u32, 4096u32,
                            )
                        }
                    }
                    pub mod labels {
                        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                        #[doc = r" enforces that these are unique, allowing references such as"]
                        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                        pub mod adc0 {
                            pub use crate::devicetree::soc::adc_6000e000::*;
                        }
                        pub mod aes {
                            pub use crate::devicetree::soc::aes_60088000::*;
                        }
                        pub mod boot_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                        }
                        pub mod clock {
                            pub use crate::devicetree::clock::*;
                        }
                        pub mod coredump_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                        }
                        pub mod coretemp {
                            pub use crate::devicetree::soc::coretemp_6000e058::*;
                        }
                        pub mod cpu0 {
                            pub use crate::devicetree::cpus::cpu_0::*;
                        }
                        pub mod deep_sleep {
                            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                        }
                        pub mod dma {
                            pub use crate::devicetree::soc::dma_60080000::*;
                        }
                        pub mod esp32_bt_hci {
                            pub use crate::devicetree::esp32_bt_hci::*;
                        }
                        pub mod flash {
                            pub use crate::devicetree::soc::flash_controller_60002000::*;
                        }
                        pub mod flash0 {
                            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                        }
                        pub mod gpio0 {
                            pub use crate::devicetree::soc::gpio_60091000::*;
                        }
                        pub mod i2c0 {
                            pub use crate::devicetree::soc::i2c_60004000::*;
                        }
                        pub mod i2c0_default {
                            pub use crate::devicetree::pin_controller::i2c0_default::*;
                        }
                        pub mod i2s {
                            pub use crate::devicetree::soc::i2s_6000c000::*;
                        }
                        pub mod i2s_default {
                            pub use crate::devicetree::pin_controller::i2s_default::*;
                        }
                        pub mod ieee802154 {
                            pub use crate::devicetree::ieee802154::*;
                        }
                        pub mod intc {
                            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                        }
                        pub mod ipc_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                        }
                        pub mod ledc0 {
                            pub use crate::devicetree::soc::ledc_60007000::*;
                        }
                        pub mod light_sleep {
                            pub use crate::devicetree::cpus::power_states::light_sleep::*;
                        }
                        pub mod lp_rtc {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                        }
                        pub mod lp_uart {
                            pub use crate::devicetree::soc::uart_600b1400::*;
                        }
                        pub mod mbox0 {
                            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                        }
                        pub mod mcpwm0 {
                            pub use crate::devicetree::soc::mcpwm_60014000::*;
                        }
                        pub mod pcnt {
                            pub use crate::devicetree::soc::pcnt_60012000::*;
                        }
                        pub mod pinctrl {
                            pub use crate::devicetree::pin_controller::*;
                        }
                        pub mod retainedmem {
                            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                        }
                        pub mod retainedmem0 {
                            pub use crate :: devicetree :: soc :: memory_50000000 :: memory_3cf8 :: retainedmem :: * ;
                        }
                        pub mod rtc_timer {
                            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                        }
                        pub mod scratch_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                        }
                        pub mod sha {
                            pub use crate::devicetree::soc::sha_60089000::*;
                        }
                        pub mod slot0_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                        }
                        pub mod slot0_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                        }
                        pub mod slot1_lpcore_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                        }
                        pub mod slot1_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                        }
                        pub mod soc_rom {
                            pub use crate::devicetree::soc::memory_40000000::*;
                        }
                        pub mod spi2 {
                            pub use crate::devicetree::soc::spi_60081000::*;
                        }
                        pub mod spim2_default {
                            pub use crate::devicetree::pin_controller::spim2_default::*;
                        }
                        pub mod sramhp {
                            pub use crate::devicetree::soc::memory_40800000::*;
                        }
                        pub mod sramlp {
                            pub use crate::devicetree::soc::memory_50000000::*;
                        }
                        pub mod storage_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                        }
                        pub mod sys_partition {
                            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                        }
                        pub mod systimer0 {
                            pub use crate::devicetree::soc::systimer_6000a000::*;
                        }
                        pub mod timer0 {
                            pub use crate::devicetree::soc::counter_60008000::*;
                        }
                        pub mod timer1 {
                            pub use crate::devicetree::soc::counter_60009000::*;
                        }
                        pub mod trng0 {
                            pub use crate::devicetree::soc::trng_600b2808::*;
                        }
                        pub mod twai0 {
                            pub use crate::devicetree::soc::can_6000b000::*;
                        }
                        pub mod twai1 {
                            pub use crate::devicetree::soc::can_6000d000::*;
                        }
                        pub mod uart0 {
                            pub use crate::devicetree::soc::uart_60000000::*;
                        }
                        pub mod uart0_default {
                            pub use crate::devicetree::pin_controller::uart0_default::*;
                        }
                        pub mod uart1 {
                            pub use crate::devicetree::soc::uart_60001000::*;
                        }
                        pub mod ulp_ram {
                            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                        }
                        pub mod ulp_shm {
                            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                        }
                        pub mod usb_serial {
                            pub use crate::devicetree::soc::uart_6000f000::*;
                        }
                        pub mod user_button1 {
                            pub use crate::devicetree::gpio_keys::button_1::*;
                        }
                        pub mod wdt0 {
                            pub use crate::devicetree::soc::watchdog_60008048::*;
                        }
                        pub mod wdt1 {
                            pub use crate::devicetree::soc::watchdog_60009048::*;
                        }
                        pub mod wifi {
                            pub use crate::devicetree::wifi::*;
                        }
                    }
                }
                pub mod labels {
                    #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                    #[doc = r" enforces that these are unique, allowing references such as"]
                    #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                    pub mod adc0 {
                        pub use crate::devicetree::soc::adc_6000e000::*;
                    }
                    pub mod aes {
                        pub use crate::devicetree::soc::aes_60088000::*;
                    }
                    pub mod boot_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                    }
                    pub mod clock {
                        pub use crate::devicetree::clock::*;
                    }
                    pub mod coredump_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                    }
                    pub mod coretemp {
                        pub use crate::devicetree::soc::coretemp_6000e058::*;
                    }
                    pub mod cpu0 {
                        pub use crate::devicetree::cpus::cpu_0::*;
                    }
                    pub mod deep_sleep {
                        pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                    }
                    pub mod dma {
                        pub use crate::devicetree::soc::dma_60080000::*;
                    }
                    pub mod esp32_bt_hci {
                        pub use crate::devicetree::esp32_bt_hci::*;
                    }
                    pub mod flash {
                        pub use crate::devicetree::soc::flash_controller_60002000::*;
                    }
                    pub mod flash0 {
                        pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                    }
                    pub mod gpio0 {
                        pub use crate::devicetree::soc::gpio_60091000::*;
                    }
                    pub mod i2c0 {
                        pub use crate::devicetree::soc::i2c_60004000::*;
                    }
                    pub mod i2c0_default {
                        pub use crate::devicetree::pin_controller::i2c0_default::*;
                    }
                    pub mod i2s {
                        pub use crate::devicetree::soc::i2s_6000c000::*;
                    }
                    pub mod i2s_default {
                        pub use crate::devicetree::pin_controller::i2s_default::*;
                    }
                    pub mod ieee802154 {
                        pub use crate::devicetree::ieee802154::*;
                    }
                    pub mod intc {
                        pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                    }
                    pub mod ipc_shm {
                        pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                    }
                    pub mod ledc0 {
                        pub use crate::devicetree::soc::ledc_60007000::*;
                    }
                    pub mod light_sleep {
                        pub use crate::devicetree::cpus::power_states::light_sleep::*;
                    }
                    pub mod lp_rtc {
                        pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                    }
                    pub mod lp_uart {
                        pub use crate::devicetree::soc::uart_600b1400::*;
                    }
                    pub mod mbox0 {
                        pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                    }
                    pub mod mcpwm0 {
                        pub use crate::devicetree::soc::mcpwm_60014000::*;
                    }
                    pub mod pcnt {
                        pub use crate::devicetree::soc::pcnt_60012000::*;
                    }
                    pub mod pinctrl {
                        pub use crate::devicetree::pin_controller::*;
                    }
                    pub mod retainedmem {
                        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                    }
                    pub mod retainedmem0 {
                        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                    }
                    pub mod rtc_timer {
                        pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                    }
                    pub mod scratch_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                    }
                    pub mod sha {
                        pub use crate::devicetree::soc::sha_60089000::*;
                    }
                    pub mod slot0_lpcore_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                    }
                    pub mod slot0_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                    }
                    pub mod slot1_lpcore_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                    }
                    pub mod slot1_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                    }
                    pub mod soc_rom {
                        pub use crate::devicetree::soc::memory_40000000::*;
                    }
                    pub mod spi2 {
                        pub use crate::devicetree::soc::spi_60081000::*;
                    }
                    pub mod spim2_default {
                        pub use crate::devicetree::pin_controller::spim2_default::*;
                    }
                    pub mod sramhp {
                        pub use crate::devicetree::soc::memory_40800000::*;
                    }
                    pub mod sramlp {
                        pub use crate::devicetree::soc::memory_50000000::*;
                    }
                    pub mod storage_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                    }
                    pub mod sys_partition {
                        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                    }
                    pub mod systimer0 {
                        pub use crate::devicetree::soc::systimer_6000a000::*;
                    }
                    pub mod timer0 {
                        pub use crate::devicetree::soc::counter_60008000::*;
                    }
                    pub mod timer1 {
                        pub use crate::devicetree::soc::counter_60009000::*;
                    }
                    pub mod trng0 {
                        pub use crate::devicetree::soc::trng_600b2808::*;
                    }
                    pub mod twai0 {
                        pub use crate::devicetree::soc::can_6000b000::*;
                    }
                    pub mod twai1 {
                        pub use crate::devicetree::soc::can_6000d000::*;
                    }
                    pub mod uart0 {
                        pub use crate::devicetree::soc::uart_60000000::*;
                    }
                    pub mod uart0_default {
                        pub use crate::devicetree::pin_controller::uart0_default::*;
                    }
                    pub mod uart1 {
                        pub use crate::devicetree::soc::uart_60001000::*;
                    }
                    pub mod ulp_ram {
                        pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                    }
                    pub mod ulp_shm {
                        pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                    }
                    pub mod usb_serial {
                        pub use crate::devicetree::soc::uart_6000f000::*;
                    }
                    pub mod user_button1 {
                        pub use crate::devicetree::gpio_keys::button_1::*;
                    }
                    pub mod wdt0 {
                        pub use crate::devicetree::soc::watchdog_60008048::*;
                    }
                    pub mod wdt1 {
                        pub use crate::devicetree::soc::watchdog_60009048::*;
                    }
                    pub mod wifi {
                        pub use crate::devicetree::wifi::*;
                    }
                }
            }
            pub mod labels {
                #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
                #[doc = r" enforces that these are unique, allowing references such as"]
                #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
                pub mod adc0 {
                    pub use crate::devicetree::soc::adc_6000e000::*;
                }
                pub mod aes {
                    pub use crate::devicetree::soc::aes_60088000::*;
                }
                pub mod boot_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
                }
                pub mod clock {
                    pub use crate::devicetree::clock::*;
                }
                pub mod coredump_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
                }
                pub mod coretemp {
                    pub use crate::devicetree::soc::coretemp_6000e058::*;
                }
                pub mod cpu0 {
                    pub use crate::devicetree::cpus::cpu_0::*;
                }
                pub mod deep_sleep {
                    pub use crate::devicetree::cpus::power_states::deep_sleep::*;
                }
                pub mod dma {
                    pub use crate::devicetree::soc::dma_60080000::*;
                }
                pub mod esp32_bt_hci {
                    pub use crate::devicetree::esp32_bt_hci::*;
                }
                pub mod flash {
                    pub use crate::devicetree::soc::flash_controller_60002000::*;
                }
                pub mod flash0 {
                    pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
                }
                pub mod gpio0 {
                    pub use crate::devicetree::soc::gpio_60091000::*;
                }
                pub mod i2c0 {
                    pub use crate::devicetree::soc::i2c_60004000::*;
                }
                pub mod i2c0_default {
                    pub use crate::devicetree::pin_controller::i2c0_default::*;
                }
                pub mod i2s {
                    pub use crate::devicetree::soc::i2s_6000c000::*;
                }
                pub mod i2s_default {
                    pub use crate::devicetree::pin_controller::i2s_default::*;
                }
                pub mod ieee802154 {
                    pub use crate::devicetree::ieee802154::*;
                }
                pub mod intc {
                    pub use crate::devicetree::soc::interrupt_controller_60010000::*;
                }
                pub mod ipc_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
                }
                pub mod ledc0 {
                    pub use crate::devicetree::soc::ledc_60007000::*;
                }
                pub mod light_sleep {
                    pub use crate::devicetree::cpus::power_states::light_sleep::*;
                }
                pub mod lp_rtc {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
                }
                pub mod lp_uart {
                    pub use crate::devicetree::soc::uart_600b1400::*;
                }
                pub mod mbox0 {
                    pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
                }
                pub mod mcpwm0 {
                    pub use crate::devicetree::soc::mcpwm_60014000::*;
                }
                pub mod pcnt {
                    pub use crate::devicetree::soc::pcnt_60012000::*;
                }
                pub mod pinctrl {
                    pub use crate::devicetree::pin_controller::*;
                }
                pub mod retainedmem {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
                }
                pub mod retainedmem0 {
                    pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
                }
                pub mod rtc_timer {
                    pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
                }
                pub mod scratch_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
                }
                pub mod sha {
                    pub use crate::devicetree::soc::sha_60089000::*;
                }
                pub mod slot0_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
                }
                pub mod slot0_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
                }
                pub mod slot1_lpcore_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
                }
                pub mod slot1_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
                }
                pub mod soc_rom {
                    pub use crate::devicetree::soc::memory_40000000::*;
                }
                pub mod spi2 {
                    pub use crate::devicetree::soc::spi_60081000::*;
                }
                pub mod spim2_default {
                    pub use crate::devicetree::pin_controller::spim2_default::*;
                }
                pub mod sramhp {
                    pub use crate::devicetree::soc::memory_40800000::*;
                }
                pub mod sramlp {
                    pub use crate::devicetree::soc::memory_50000000::*;
                }
                pub mod storage_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
                }
                pub mod sys_partition {
                    pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
                }
                pub mod systimer0 {
                    pub use crate::devicetree::soc::systimer_6000a000::*;
                }
                pub mod timer0 {
                    pub use crate::devicetree::soc::counter_60008000::*;
                }
                pub mod timer1 {
                    pub use crate::devicetree::soc::counter_60009000::*;
                }
                pub mod trng0 {
                    pub use crate::devicetree::soc::trng_600b2808::*;
                }
                pub mod twai0 {
                    pub use crate::devicetree::soc::can_6000b000::*;
                }
                pub mod twai1 {
                    pub use crate::devicetree::soc::can_6000d000::*;
                }
                pub mod uart0 {
                    pub use crate::devicetree::soc::uart_60000000::*;
                }
                pub mod uart0_default {
                    pub use crate::devicetree::pin_controller::uart0_default::*;
                }
                pub mod uart1 {
                    pub use crate::devicetree::soc::uart_60001000::*;
                }
                pub mod ulp_ram {
                    pub use crate::devicetree::soc::memory_50000000::memory_0::*;
                }
                pub mod ulp_shm {
                    pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
                }
                pub mod usb_serial {
                    pub use crate::devicetree::soc::uart_6000f000::*;
                }
                pub mod user_button1 {
                    pub use crate::devicetree::gpio_keys::button_1::*;
                }
                pub mod wdt0 {
                    pub use crate::devicetree::soc::watchdog_60008048::*;
                }
                pub mod wdt1 {
                    pub use crate::devicetree::soc::watchdog_60009048::*;
                }
                pub mod wifi {
                    pub use crate::devicetree::wifi::*;
                }
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod coretemp_6000e058 {
        pub const ORD: usize = 32usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-temp\")]";
        pub const FRIENDLY_NAME_DEBUG: &str = "[String(\"coretemp\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610670168), Number(4)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod adc_6000e000 {
        pub const ORD: usize = 28usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-adc\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610670080), Number(4)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(118)])]";
        pub const UNIT: u32 = 1u32;
        pub const CHANNEL_COUNT: u32 = 7u32;
        pub const NIO_CHANNEL_CELLS: u32 = 1u32;
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod dma_60080000 {
        pub const ORD: usize = 34usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-gdma\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611137024), Number(4096)])]";
        pub const NDMA_CELLS: u32 = 1u32;
        pub const INTERRUPTS_DEBUG : & str = "[Words([Number(66), Number(0), Number(0)]), Words([Number(69), Number(0), Number(0)]), Words([Number(67), Number(0), Number(0)]), Words([Number(70), Number(0), Number(0)]), Words([Number(68), Number(0), Number(0)]), Words([Number(71), Number(0), Number(0)])]" ;
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(113)])]";
        pub const DMA_CHANNELS: u32 = 6u32;
        pub const DMA_BUF_ADDR_ALIGNMENT: u32 = 4u32;
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub const PHANDLE: u32 = 8u32;
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod gpio_60091000 {
        pub const ORD: usize = 13usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-gpio\")]";
        pub const GPIO_CONTROLLER_DEBUG: &str = "[]";
        pub const NGPIO_CELLS: u32 = 2u32;
        pub const REG_DEBUG: &str = "[Words([Number(1611206656), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(30), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const NGPIOS: u32 = 30u32;
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub const PHANDLE: u32 = 11u32;
        #[doc = r" Get the raw `const struct device *` of the device tree generated node."]
        pub unsafe fn get_instance_raw() -> *const crate::raw::device {
            &crate::raw::__device_dts_ord_13
        }
        #[allow(dead_code)]
        pub(crate) unsafe fn get_static_raw() -> &'static crate::device::gpio::GpioStatic {
            &STATIC
        }
        static UNIQUE: crate::device::Unique = crate::device::Unique::new();
        static STATIC: crate::device::gpio::GpioStatic = crate::device::gpio::GpioStatic::new();
        pub fn get_instance() -> Option<crate::device::gpio::Gpio> {
            unsafe {
                let device = get_instance_raw();
                crate::device::gpio::Gpio::new(&UNIQUE, &STATIC, device)
            }
        }
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod i2c_60004000 {
        pub const ORD: usize = 33usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-i2c\")]";
        pub const NADDRESS_CELLS: u32 = 1u32;
        pub const NSIZE_CELLS: u32 = 0u32;
        pub const REG_DEBUG: &str = "[Words([Number(1610629120), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(50), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(104)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub const CLOCK_FREQUENCY: u32 = 400000u32;
        pub const PINCTRL_0_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"i2c0_default\", .. })])]";
        pub const PINCTRL_NAMES_DEBUG: &str = "[String(\"default\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod i2s_6000c000 {
        pub const ORD: usize = 35usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-i2s\")]";
        pub const NADDRESS_CELLS: u32 = 1u32;
        pub const NSIZE_CELLS: u32 = 0u32;
        pub const REG_DEBUG: &str = "[Words([Number(1610661888), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(41), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(105)])]";
        pub const DMAS_DEBUG : & str = "[Words([Phandle(Phandle { name: \"dma\", .. }), Number(2)]), Words([Phandle(Phandle { name: \"dma\", .. }), Number(3)])]" ;
        pub const DMA_NAMES_DEBUG: &str = "[String(\"rx\"), String(\"tx\")]";
        pub const UNIT: u32 = 0u32;
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub const PINCTRL_0_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"i2s_default\", .. })])]";
        pub const PINCTRL_NAMES_DEBUG: &str = "[String(\"default\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod uart_60000000 {
        pub const ORD: usize = 46usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-uart\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610612736), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(43), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(101)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub const CURRENT_SPEED: u32 = 115200u32;
        pub const PINCTRL_0_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"uart0_default\", .. })])]";
        pub const PINCTRL_NAMES_DEBUG: &str = "[String(\"default\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod uart_60001000 {
        pub const ORD: usize = 47usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-uart\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610616832), Number(4096)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(44), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(102)])]";
        pub const CURRENT_SPEED: u32 = 115200u32;
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod uart_600b1400 {
        pub const ORD: usize = 49usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-lpuart\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611338752), Number(4096)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub const CURRENT_SPEED: u32 = 115200u32;
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod uart_6000f000 {
        pub const ORD: usize = 48usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-usb-serial\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610674176), Number(4096)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(48), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(103)])]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod ledc_60007000 {
        pub const ORD: usize = 36usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-ledc\")]";
        pub const PWM_CONTROLLER_DEBUG: &str = "[]";
        pub const NPWM_CELLS: u32 = 3u32;
        pub const REG_DEBUG: &str = "[Words([Number(1610641408), Number(4096)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(100)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod mcpwm_60014000 {
        pub const ORD: usize = 37usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-mcpwm\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610694656), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(61), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(114)])]";
        pub const NPWM_CELLS: u32 = 3u32;
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod pcnt_60012000 {
        pub const ORD: usize = 40usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-pcnt\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1610686464), Number(4096)])]";
        pub const INTERRUPTS_DEBUG: &str = "[Words([Number(62), Number(0), Number(0)])]";
        pub const INTERRUPT_PARENT_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"intc\", .. })])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(107)])]";
        pub const STATUS_DEBUG: &str = "[String(\"disabled\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod sha_60089000 {
        pub const ORD: usize = 42usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-sha\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611173888), Number(256)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(121)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod aes_60088000 {
        pub const ORD: usize = 29usize;
        pub const COMPATIBLE_DEBUG: &str = "[String(\"espressif,esp32-aes\")]";
        pub const REG_DEBUG: &str = "[Words([Number(1611169792), Number(4096)])]";
        pub const CLOCKS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"clock\", .. }), Number(120)])]";
        pub const STATUS_DEBUG: &str = "[String(\"okay\")]";
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod gpio_keys {
    pub const ORD: usize = 14usize;
    pub const COMPATIBLE_DEBUG: &str = "[String(\"gpio-keys\")]";
    pub mod button_1 {
        pub const ORD: usize = 15usize;
        pub const LABEL_DEBUG: &str = "[String(\"User SW1\")]";
        pub const GPIOS_DEBUG: &str =
            "[Words([Phandle(Phandle { name: \"gpio0\", .. }), Number(9), Number(17)])]";
        pub const ZEPHYR_CODE: u32 = 11u32;
        pub mod labels {
            #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
            #[doc = r" enforces that these are unique, allowing references such as"]
            #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
            pub mod adc0 {
                pub use crate::devicetree::soc::adc_6000e000::*;
            }
            pub mod aes {
                pub use crate::devicetree::soc::aes_60088000::*;
            }
            pub mod boot_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
            }
            pub mod clock {
                pub use crate::devicetree::clock::*;
            }
            pub mod coredump_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
            }
            pub mod coretemp {
                pub use crate::devicetree::soc::coretemp_6000e058::*;
            }
            pub mod cpu0 {
                pub use crate::devicetree::cpus::cpu_0::*;
            }
            pub mod deep_sleep {
                pub use crate::devicetree::cpus::power_states::deep_sleep::*;
            }
            pub mod dma {
                pub use crate::devicetree::soc::dma_60080000::*;
            }
            pub mod esp32_bt_hci {
                pub use crate::devicetree::esp32_bt_hci::*;
            }
            pub mod flash {
                pub use crate::devicetree::soc::flash_controller_60002000::*;
            }
            pub mod flash0 {
                pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
            }
            pub mod gpio0 {
                pub use crate::devicetree::soc::gpio_60091000::*;
            }
            pub mod i2c0 {
                pub use crate::devicetree::soc::i2c_60004000::*;
            }
            pub mod i2c0_default {
                pub use crate::devicetree::pin_controller::i2c0_default::*;
            }
            pub mod i2s {
                pub use crate::devicetree::soc::i2s_6000c000::*;
            }
            pub mod i2s_default {
                pub use crate::devicetree::pin_controller::i2s_default::*;
            }
            pub mod ieee802154 {
                pub use crate::devicetree::ieee802154::*;
            }
            pub mod intc {
                pub use crate::devicetree::soc::interrupt_controller_60010000::*;
            }
            pub mod ipc_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
            }
            pub mod ledc0 {
                pub use crate::devicetree::soc::ledc_60007000::*;
            }
            pub mod light_sleep {
                pub use crate::devicetree::cpus::power_states::light_sleep::*;
            }
            pub mod lp_rtc {
                pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
            }
            pub mod lp_uart {
                pub use crate::devicetree::soc::uart_600b1400::*;
            }
            pub mod mbox0 {
                pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
            }
            pub mod mcpwm0 {
                pub use crate::devicetree::soc::mcpwm_60014000::*;
            }
            pub mod pcnt {
                pub use crate::devicetree::soc::pcnt_60012000::*;
            }
            pub mod pinctrl {
                pub use crate::devicetree::pin_controller::*;
            }
            pub mod retainedmem {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
            }
            pub mod retainedmem0 {
                pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
            }
            pub mod rtc_timer {
                pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
            }
            pub mod scratch_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
            }
            pub mod sha {
                pub use crate::devicetree::soc::sha_60089000::*;
            }
            pub mod slot0_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
            }
            pub mod slot0_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
            }
            pub mod slot1_lpcore_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
            }
            pub mod slot1_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
            }
            pub mod soc_rom {
                pub use crate::devicetree::soc::memory_40000000::*;
            }
            pub mod spi2 {
                pub use crate::devicetree::soc::spi_60081000::*;
            }
            pub mod spim2_default {
                pub use crate::devicetree::pin_controller::spim2_default::*;
            }
            pub mod sramhp {
                pub use crate::devicetree::soc::memory_40800000::*;
            }
            pub mod sramlp {
                pub use crate::devicetree::soc::memory_50000000::*;
            }
            pub mod storage_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
            }
            pub mod sys_partition {
                pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
            }
            pub mod systimer0 {
                pub use crate::devicetree::soc::systimer_6000a000::*;
            }
            pub mod timer0 {
                pub use crate::devicetree::soc::counter_60008000::*;
            }
            pub mod timer1 {
                pub use crate::devicetree::soc::counter_60009000::*;
            }
            pub mod trng0 {
                pub use crate::devicetree::soc::trng_600b2808::*;
            }
            pub mod twai0 {
                pub use crate::devicetree::soc::can_6000b000::*;
            }
            pub mod twai1 {
                pub use crate::devicetree::soc::can_6000d000::*;
            }
            pub mod uart0 {
                pub use crate::devicetree::soc::uart_60000000::*;
            }
            pub mod uart0_default {
                pub use crate::devicetree::pin_controller::uart0_default::*;
            }
            pub mod uart1 {
                pub use crate::devicetree::soc::uart_60001000::*;
            }
            pub mod ulp_ram {
                pub use crate::devicetree::soc::memory_50000000::memory_0::*;
            }
            pub mod ulp_shm {
                pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
            }
            pub mod usb_serial {
                pub use crate::devicetree::soc::uart_6000f000::*;
            }
            pub mod user_button1 {
                pub use crate::devicetree::gpio_keys::button_1::*;
            }
            pub mod wdt0 {
                pub use crate::devicetree::soc::watchdog_60008048::*;
            }
            pub mod wdt1 {
                pub use crate::devicetree::soc::watchdog_60009048::*;
            }
            pub mod wifi {
                pub use crate::devicetree::wifi::*;
            }
        }
    }
    pub mod labels {
        #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
        #[doc = r" enforces that these are unique, allowing references such as"]
        #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
        pub mod adc0 {
            pub use crate::devicetree::soc::adc_6000e000::*;
        }
        pub mod aes {
            pub use crate::devicetree::soc::aes_60088000::*;
        }
        pub mod boot_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
        }
        pub mod clock {
            pub use crate::devicetree::clock::*;
        }
        pub mod coredump_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
        }
        pub mod coretemp {
            pub use crate::devicetree::soc::coretemp_6000e058::*;
        }
        pub mod cpu0 {
            pub use crate::devicetree::cpus::cpu_0::*;
        }
        pub mod deep_sleep {
            pub use crate::devicetree::cpus::power_states::deep_sleep::*;
        }
        pub mod dma {
            pub use crate::devicetree::soc::dma_60080000::*;
        }
        pub mod esp32_bt_hci {
            pub use crate::devicetree::esp32_bt_hci::*;
        }
        pub mod flash {
            pub use crate::devicetree::soc::flash_controller_60002000::*;
        }
        pub mod flash0 {
            pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
        }
        pub mod gpio0 {
            pub use crate::devicetree::soc::gpio_60091000::*;
        }
        pub mod i2c0 {
            pub use crate::devicetree::soc::i2c_60004000::*;
        }
        pub mod i2c0_default {
            pub use crate::devicetree::pin_controller::i2c0_default::*;
        }
        pub mod i2s {
            pub use crate::devicetree::soc::i2s_6000c000::*;
        }
        pub mod i2s_default {
            pub use crate::devicetree::pin_controller::i2s_default::*;
        }
        pub mod ieee802154 {
            pub use crate::devicetree::ieee802154::*;
        }
        pub mod intc {
            pub use crate::devicetree::soc::interrupt_controller_60010000::*;
        }
        pub mod ipc_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
        }
        pub mod ledc0 {
            pub use crate::devicetree::soc::ledc_60007000::*;
        }
        pub mod light_sleep {
            pub use crate::devicetree::cpus::power_states::light_sleep::*;
        }
        pub mod lp_rtc {
            pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
        }
        pub mod lp_uart {
            pub use crate::devicetree::soc::uart_600b1400::*;
        }
        pub mod mbox0 {
            pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
        }
        pub mod mcpwm0 {
            pub use crate::devicetree::soc::mcpwm_60014000::*;
        }
        pub mod pcnt {
            pub use crate::devicetree::soc::pcnt_60012000::*;
        }
        pub mod pinctrl {
            pub use crate::devicetree::pin_controller::*;
        }
        pub mod retainedmem {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
        }
        pub mod retainedmem0 {
            pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
        }
        pub mod rtc_timer {
            pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
        }
        pub mod scratch_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
        }
        pub mod sha {
            pub use crate::devicetree::soc::sha_60089000::*;
        }
        pub mod slot0_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
        }
        pub mod slot0_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
        }
        pub mod slot1_lpcore_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
        }
        pub mod slot1_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
        }
        pub mod soc_rom {
            pub use crate::devicetree::soc::memory_40000000::*;
        }
        pub mod spi2 {
            pub use crate::devicetree::soc::spi_60081000::*;
        }
        pub mod spim2_default {
            pub use crate::devicetree::pin_controller::spim2_default::*;
        }
        pub mod sramhp {
            pub use crate::devicetree::soc::memory_40800000::*;
        }
        pub mod sramlp {
            pub use crate::devicetree::soc::memory_50000000::*;
        }
        pub mod storage_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
        }
        pub mod sys_partition {
            pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
        }
        pub mod systimer0 {
            pub use crate::devicetree::soc::systimer_6000a000::*;
        }
        pub mod timer0 {
            pub use crate::devicetree::soc::counter_60008000::*;
        }
        pub mod timer1 {
            pub use crate::devicetree::soc::counter_60009000::*;
        }
        pub mod trng0 {
            pub use crate::devicetree::soc::trng_600b2808::*;
        }
        pub mod twai0 {
            pub use crate::devicetree::soc::can_6000b000::*;
        }
        pub mod twai1 {
            pub use crate::devicetree::soc::can_6000d000::*;
        }
        pub mod uart0 {
            pub use crate::devicetree::soc::uart_60000000::*;
        }
        pub mod uart0_default {
            pub use crate::devicetree::pin_controller::uart0_default::*;
        }
        pub mod uart1 {
            pub use crate::devicetree::soc::uart_60001000::*;
        }
        pub mod ulp_ram {
            pub use crate::devicetree::soc::memory_50000000::memory_0::*;
        }
        pub mod ulp_shm {
            pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
        }
        pub mod usb_serial {
            pub use crate::devicetree::soc::uart_6000f000::*;
        }
        pub mod user_button1 {
            pub use crate::devicetree::gpio_keys::button_1::*;
        }
        pub mod wdt0 {
            pub use crate::devicetree::soc::watchdog_60008048::*;
        }
        pub mod wdt1 {
            pub use crate::devicetree::soc::watchdog_60009048::*;
        }
        pub mod wifi {
            pub use crate::devicetree::wifi::*;
        }
    }
}
pub mod labels {
    #[doc = r" All of the labeles in the device tree.  The device tree compiler"]
    #[doc = r" enforces that these are unique, allowing references such as"]
    #[doc = r" `zephyr::devicetree::labels::labelname::get_instance()`."]
    pub mod adc0 {
        pub use crate::devicetree::soc::adc_6000e000::*;
    }
    pub mod aes {
        pub use crate::devicetree::soc::aes_60088000::*;
    }
    pub mod boot_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_0 :: * ;
    }
    pub mod clock {
        pub use crate::devicetree::clock::*;
    }
    pub mod coredump_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3ff000 :: * ;
    }
    pub mod coretemp {
        pub use crate::devicetree::soc::coretemp_6000e058::*;
    }
    pub mod cpu0 {
        pub use crate::devicetree::cpus::cpu_0::*;
    }
    pub mod deep_sleep {
        pub use crate::devicetree::cpus::power_states::deep_sleep::*;
    }
    pub mod dma {
        pub use crate::devicetree::soc::dma_60080000::*;
    }
    pub mod esp32_bt_hci {
        pub use crate::devicetree::esp32_bt_hci::*;
    }
    pub mod flash {
        pub use crate::devicetree::soc::flash_controller_60002000::*;
    }
    pub mod flash0 {
        pub use crate::devicetree::soc::flash_controller_60002000::flash_0::*;
    }
    pub mod gpio0 {
        pub use crate::devicetree::soc::gpio_60091000::*;
    }
    pub mod i2c0 {
        pub use crate::devicetree::soc::i2c_60004000::*;
    }
    pub mod i2c0_default {
        pub use crate::devicetree::pin_controller::i2c0_default::*;
    }
    pub mod i2s {
        pub use crate::devicetree::soc::i2s_6000c000::*;
    }
    pub mod i2s_default {
        pub use crate::devicetree::pin_controller::i2s_default::*;
    }
    pub mod ieee802154 {
        pub use crate::devicetree::ieee802154::*;
    }
    pub mod intc {
        pub use crate::devicetree::soc::interrupt_controller_60010000::*;
    }
    pub mod ipc_shm {
        pub use crate::devicetree::soc::memory_50000000::memory_3ef8::*;
    }
    pub mod ledc0 {
        pub use crate::devicetree::soc::ledc_60007000::*;
    }
    pub mod light_sleep {
        pub use crate::devicetree::cpus::power_states::light_sleep::*;
    }
    pub mod lp_rtc {
        pub use crate::devicetree::soc::memory_50000000::memory_3c10::*;
    }
    pub mod lp_uart {
        pub use crate::devicetree::soc::uart_600b1400::*;
    }
    pub mod mbox0 {
        pub use crate::devicetree::soc::memory_50000000::mbox_3ff8::*;
    }
    pub mod mcpwm0 {
        pub use crate::devicetree::soc::mcpwm_60014000::*;
    }
    pub mod pcnt {
        pub use crate::devicetree::soc::pcnt_60012000::*;
    }
    pub mod pinctrl {
        pub use crate::devicetree::pin_controller::*;
    }
    pub mod retainedmem {
        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::*;
    }
    pub mod retainedmem0 {
        pub use crate::devicetree::soc::memory_50000000::memory_3cf8::retainedmem::*;
    }
    pub mod rtc_timer {
        pub use crate::devicetree::soc::rtc_timer_600b0c00::*;
    }
    pub mod scratch_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3e0000 :: * ;
    }
    pub mod sha {
        pub use crate::devicetree::soc::sha_60089000::*;
    }
    pub mod slot0_lpcore_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a0000 :: * ;
    }
    pub mod slot0_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_20000 :: * ;
    }
    pub mod slot1_lpcore_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3a8000 :: * ;
    }
    pub mod slot1_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_1e0000 :: * ;
    }
    pub mod soc_rom {
        pub use crate::devicetree::soc::memory_40000000::*;
    }
    pub mod spi2 {
        pub use crate::devicetree::soc::spi_60081000::*;
    }
    pub mod spim2_default {
        pub use crate::devicetree::pin_controller::spim2_default::*;
    }
    pub mod sramhp {
        pub use crate::devicetree::soc::memory_40800000::*;
    }
    pub mod sramlp {
        pub use crate::devicetree::soc::memory_50000000::*;
    }
    pub mod storage_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_3b0000 :: * ;
    }
    pub mod sys_partition {
        pub use crate :: devicetree :: soc :: flash_controller_60002000 :: flash_0 :: partitions :: partition_10000 :: * ;
    }
    pub mod systimer0 {
        pub use crate::devicetree::soc::systimer_6000a000::*;
    }
    pub mod timer0 {
        pub use crate::devicetree::soc::counter_60008000::*;
    }
    pub mod timer1 {
        pub use crate::devicetree::soc::counter_60009000::*;
    }
    pub mod trng0 {
        pub use crate::devicetree::soc::trng_600b2808::*;
    }
    pub mod twai0 {
        pub use crate::devicetree::soc::can_6000b000::*;
    }
    pub mod twai1 {
        pub use crate::devicetree::soc::can_6000d000::*;
    }
    pub mod uart0 {
        pub use crate::devicetree::soc::uart_60000000::*;
    }
    pub mod uart0_default {
        pub use crate::devicetree::pin_controller::uart0_default::*;
    }
    pub mod uart1 {
        pub use crate::devicetree::soc::uart_60001000::*;
    }
    pub mod ulp_ram {
        pub use crate::devicetree::soc::memory_50000000::memory_0::*;
    }
    pub mod ulp_shm {
        pub use crate::devicetree::soc::memory_50000000::memory_3c00::*;
    }
    pub mod usb_serial {
        pub use crate::devicetree::soc::uart_6000f000::*;
    }
    pub mod user_button1 {
        pub use crate::devicetree::gpio_keys::button_1::*;
    }
    pub mod wdt0 {
        pub use crate::devicetree::soc::watchdog_60008048::*;
    }
    pub mod wdt1 {
        pub use crate::devicetree::soc::watchdog_60009048::*;
    }
    pub mod wifi {
        pub use crate::devicetree::wifi::*;
    }
}
