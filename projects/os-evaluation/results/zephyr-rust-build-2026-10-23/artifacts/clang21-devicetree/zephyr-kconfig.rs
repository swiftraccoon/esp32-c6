#[allow(dead_code)]
pub const CONFIG_MAIN_STACK_SIZE: isize = 4096;
#[allow(dead_code)]
pub const CONFIG_SYSTEM_WORKQUEUE_STACK_SIZE: isize = 1024;
#[allow(dead_code)]
pub const CONFIG_NUM_IRQS: isize = 32;
#[allow(dead_code)]
pub const CONFIG_SYS_CLOCK_HW_CYCLES_PER_SEC: isize = 16000000;
#[allow(dead_code)]
pub const CONFIG_FLASH_SIZE: isize = 8388608;
#[allow(dead_code)]
pub const CONFIG_FLASH_BASE_ADDRESS: usize = 0x0;
#[allow(dead_code)]
pub const CONFIG_MP_MAX_NUM_CPUS: isize = 1;
#[allow(dead_code)]
pub const CONFIG_IDLE_STACK_SIZE: isize = 512;
#[allow(dead_code)]
pub const CONFIG_ISR_STACK_SIZE: isize = 2048;
#[allow(dead_code)]
pub const CONFIG_SYS_CLOCK_TICKS_PER_SEC: isize = 1000;
#[allow(dead_code)]
pub const CONFIG_KERNEL_ENTRY: &str = "__start";
#[allow(dead_code)]
pub const CONFIG_TEST_EXTRA_STACK_SIZE: isize = 1536;
#[allow(dead_code)]
pub const CONFIG_CLOCK_CONTROL_INIT_PRIORITY: isize = 30;
#[allow(dead_code)]
pub const CONFIG_PMP_SLOTS: isize = 16;
#[allow(dead_code)]
pub const CONFIG_PMP_GRANULARITY: isize = 4;
#[allow(dead_code)]
pub const CONFIG_NUM_PREEMPT_PRIORITIES: isize = 15;
#[allow(dead_code)]
pub const CONFIG_RISCV_MCAUSE_EXCEPTION_MASK: usize = 0x7FFFFFFF;
#[allow(dead_code)]
pub const CONFIG_TIMESLICE_SIZE: isize = 20;
#[allow(dead_code)]
pub const CONFIG_FLASH_LOAD_OFFSET: usize = 0x0;
#[allow(dead_code)]
pub const CONFIG_ARCH_SW_ISR_TABLE_ALIGN: isize = 4;
#[allow(dead_code)]
pub const CONFIG_LOG_DOMAIN_NAME: &str = "";
#[allow(dead_code)]
pub const CONFIG_LOG_PROCESS_THREAD_STACK_SIZE: isize = 768;
#[allow(dead_code)]
pub const CONFIG_COMMON_LIBC_MALLOC_ARENA_SIZE: isize = -1;
#[allow(dead_code)]
pub const CONFIG_ARCH_IRQ_VECTOR_TABLE_ALIGN: isize = 256;
#[allow(dead_code)]
pub const CONFIG_RISCV_TRAP_HANDLER_ALIGNMENT: isize = 4;
#[allow(dead_code)]
pub const CONFIG_XTAL_FREQ_HZ: isize = 40000000;
#[allow(dead_code)]
pub const CONFIG_ESP_HAL_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_ESP_HAL_EARLY_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_PERIPH_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_MAX_CHANNEL_NUM: isize = 7;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_ATTEN_NUM: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_CONTROLLER_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_PATT_LEN_MAX: isize = 8;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_MIN_BITWIDTH: isize = 12;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_MAX_BITWIDTH: isize = 12;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_RESULT_BYTES: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_DATA_BYTES_PER_CONV: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_MONITOR_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_SAMPLE_FREQ_THRES_HIGH: isize = 83333;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_SAMPLE_FREQ_THRES_LOW: isize = 611;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_RTC_MIN_BITWIDTH: isize = 12;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_RTC_MAX_BITWIDTH: isize = 12;
#[allow(dead_code)]
pub const CONFIG_SOC_CPU_CORES_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_CPU_INTR_NUM: isize = 32;
#[allow(dead_code)]
pub const CONFIG_SOC_CPU_BREAKPOINTS_NUM: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_CPU_WATCHPOINTS_NUM: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_CPU_WATCHPOINT_MAX_REGION_SIZE: usize = 0x80000000;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_PORT: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_PIN_COUNT: isize = 31;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_IN_RANGE_MAX: isize = 30;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_OUT_RANGE_MAX: isize = 30;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_VALID_DIGITAL_IO_PAD_MASK: usize = 0x000000007FFFFF00;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_CLOCKOUT_CHANNEL_NUM: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SOC_I2C_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_HP_I2C_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_I2S_PDM_MAX_TX_LINES: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_I2S_PDM_MAX_RX_LINES: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_LEDC_TIMER_NUM: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_LEDC_CHANNEL_NUM: isize = 6;
#[allow(dead_code)]
pub const CONFIG_SOC_LEDC_TIMER_BIT_WIDTH: isize = 20;
#[allow(dead_code)]
pub const CONFIG_SOC_MMU_PERIPH_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_MMU_LINEAR_ADDRESS_REGION_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_MPU_MIN_REGION_SIZE: usize = 0x20000000;
#[allow(dead_code)]
pub const CONFIG_SOC_MPU_REGIONS_MAX_NUM: isize = 8;
#[allow(dead_code)]
pub const CONFIG_SOC_RMT_MEM_WORDS_PER_CHANNEL: isize = 48;
#[allow(dead_code)]
pub const CONFIG_SOC_RTCIO_PIN_COUNT: isize = 8;
#[allow(dead_code)]
pub const CONFIG_SOC_SPI_PERIPH_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_SPI_MAXIMUM_BUFFER_SIZE: isize = 64;
#[allow(dead_code)]
pub const CONFIG_SOC_LP_TIMER_BIT_WIDTH_LO: isize = 32;
#[allow(dead_code)]
pub const CONFIG_SOC_LP_TIMER_BIT_WIDTH_HI: isize = 16;
#[allow(dead_code)]
pub const CONFIG_SOC_TWAI_CONTROLLER_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_TWAI_MASK_FILTER_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_UART_NUM: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SOC_UART_HP_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_UART_FIFO_LEN: isize = 128;
#[allow(dead_code)]
pub const CONFIG_SOC_UART_BITRATE_MAX: isize = 5000000;
#[allow(dead_code)]
pub const CONFIG_SOC_MPI_MEM_BLOCKS_NUM: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_MPI_OPERATIONS_NUM: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SOC_RSA_MAX_BIT_LEN: isize = 3072;
#[allow(dead_code)]
pub const CONFIG_SOC_EFUSE_SECURE_BOOT_KEY_DIGESTS: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SOC_FLASH_ENCRYPTED_XTS_AES_BLOCK_MAX: isize = 64;
#[allow(dead_code)]
pub const CONFIG_SOC_PHY_DIG_REGS_MEM_SIZE: isize = 21;
#[allow(dead_code)]
pub const CONFIG_SOC_ADC_DIGI_IIR_FILTER_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_SOC_DS_SIGNATURE_MAX_BIT_LEN: isize = 3072;
#[allow(dead_code)]
pub const CONFIG_SOC_DS_KEY_PARAM_MD_IV_LENGTH: isize = 16;
#[allow(dead_code)]
pub const CONFIG_SOC_DS_KEY_CHECK_MAX_WAIT_US: isize = 1100;
#[allow(dead_code)]
pub const CONFIG_SOC_SHA_DMA_MAX_BUFFER_SIZE: isize = 3968;
#[allow(dead_code)]
pub const CONFIG_SOC_WIFI_LIGHT_SLEEP_CLK_WIDTH: isize = 12;
#[allow(dead_code)]
pub const CONFIG_SOC_AHB_GDMA_VERSION: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_UHCI_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_HP_PERIPH_PD_SLEEP_WAKEABLE_PIN_CNT: isize = 8;
#[allow(dead_code)]
pub const CONFIG_SOC_CPU_PMP_REGION_GRANULARITY: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_GPIO_FLEX_GLITCH_FILTER_NUM: isize = 8;
#[allow(dead_code)]
pub const CONFIG_SOC_LP_I2C_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_LEDC_GAMMA_CURVE_FADE_RANGE_MAX: isize = 16;
#[allow(dead_code)]
pub const CONFIG_SOC_LEDC_FADE_PARAMS_BIT_WIDTH: isize = 10;
#[allow(dead_code)]
pub const CONFIG_SOC_PARLIO_TX_UNIT_MAX_DATA_WIDTH: isize = 16;
#[allow(dead_code)]
pub const CONFIG_SOC_PARLIO_RX_UNIT_MAX_DATA_WIDTH: isize = 16;
#[allow(dead_code)]
pub const CONFIG_SOC_UART_LP_NUM: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SOC_LP_UART_FIFO_LEN: isize = 16;
#[allow(dead_code)]
pub const CONFIG_SOC_UART_WAKEUP_CHARS_SEQ_MAX_LEN: isize = 5;
#[allow(dead_code)]
pub const CONFIG_SOC_PM_PAU_LINK_NUM: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_PM_RETENTION_MODULE_NUM: isize = 32;
#[allow(dead_code)]
pub const CONFIG_SOC_WIFI_MAC_VERSION_NUM: isize = 2;
#[allow(dead_code)]
pub const CONFIG_XTAL_FREQ: isize = 40;
#[allow(dead_code)]
pub const CONFIG_BOOTLOADER_OFFSET_IN_FLASH: usize = 0x0;
#[allow(dead_code)]
pub const CONFIG_BOOTLOADER_CPU_CLK_FREQ_MHZ: isize = 160;
#[allow(dead_code)]
pub const CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ: isize = 160;
#[allow(dead_code)]
pub const CONFIG_ESP_CLK_FREQ_HZ: isize = 160000000;
#[allow(dead_code)]
pub const CONFIG_ESP_TIMER_INTERRUPT_LEVEL: isize = 1;
#[allow(dead_code)]
pub const CONFIG_BOOTLOADER_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_portNUM_PROCESSORS: isize = 1;
#[allow(dead_code)]
pub const CONFIG_IDF_FIRMWARE_CHIP_ID: usize = 0x000D;
#[allow(dead_code)]
pub const CONFIG_BOARD: &str = "esp32c6_devkitc";
#[allow(dead_code)]
pub const CONFIG_BOARD_REVISION: &str = "";
#[allow(dead_code)]
pub const CONFIG_BOARD_TARGET: &str = "esp32c6_devkitc/esp32c6/hpcore";
#[allow(dead_code)]
pub const CONFIG_BOARD_QUALIFIERS: &str = "esp32c6/hpcore";
#[allow(dead_code)]
pub const CONFIG_HEAP_MEM_POOL_ADD_SIZE_BOARD: isize = 4096;
#[allow(dead_code)]
pub const CONFIG_SOC: &str = "esp32c6";
#[allow(dead_code)]
pub const CONFIG_SOC_SERIES: &str = "esp32c6";
#[allow(dead_code)]
pub const CONFIG_SOC_FAMILY: &str = "espressif_esp32";
#[allow(dead_code)]
pub const CONFIG_SOC_PART_NUMBER: &str = "ESP32_C6_WROOM_1U_N8";
#[allow(dead_code)]
pub const CONFIG_GPIO_INIT_PRIORITY: isize = 40;
#[allow(dead_code)]
pub const CONFIG_ESP32_TIMER_TASK_STACK_SIZE: isize = 4096;
#[allow(dead_code)]
pub const CONFIG_ESP32_TIMER_TASK_PRIO: isize = 3;
#[allow(dead_code)]
pub const CONFIG_ESP_CONSOLE_UART_BAUDRATE: isize = 115200;
#[allow(dead_code)]
pub const CONFIG_ESP32_EFUSE_MAX_BLK_LEN: isize = 256;
#[allow(dead_code)]
pub const CONFIG_ESPTOOLPY_FLASHMODE: &str = "dio";
#[allow(dead_code)]
pub const CONFIG_ESPTOOLPY_FLASHFREQ: &str = "80m";
#[allow(dead_code)]
pub const CONFIG_ESPTOOLPY_FLASHSIZE: &str = "2MB";
#[allow(dead_code)]
pub const CONFIG_ESPTOOLPY_BEFORE: &str = "default_reset";
#[allow(dead_code)]
pub const CONFIG_ESPTOOLPY_AFTER: &str = "hard_reset";
#[allow(dead_code)]
pub const CONFIG_MMU_PAGE_SIZE: usize = 0x10000;
#[allow(dead_code)]
pub const CONFIG_ESP32C6_UNIVERSAL_MAC_ADDRESSES: isize = 4;
#[allow(dead_code)]
pub const CONFIG_SOC_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_ARCH: &str = "riscv";
#[allow(dead_code)]
pub const CONFIG_RISCV_HART_MASK: isize = -1;
#[allow(dead_code)]
pub const CONFIG_ARCH_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SRAM_SIZE: isize = 512;
#[allow(dead_code)]
pub const CONFIG_SRAM_BASE_ADDRESS: usize = 0x40800000;
#[allow(dead_code)]
pub const CONFIG_ARCH_STACKWALK_MAX_FRAMES: isize = 8;
#[allow(dead_code)]
pub const CONFIG_ARCH_DEVICE_STATE_ALIGN: isize = 4;
#[allow(dead_code)]
pub const CONFIG_MPU_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_KERNEL_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_NUM_COOP_PRIORITIES: isize = 16;
#[allow(dead_code)]
pub const CONFIG_PRIORITY_CEILING: isize = -128;
#[allow(dead_code)]
pub const CONFIG_BOOT_BANNER_STRING: &str = "Booting Zephyr OS build";
#[allow(dead_code)]
pub const CONFIG_SYSTEM_WORKQUEUE_PRIORITY: isize = -1;
#[allow(dead_code)]
pub const CONFIG_NUM_MBOX_ASYNC_MSGS: isize = 10;
#[allow(dead_code)]
pub const CONFIG_SYS_CLOCK_MAX_TIMEOUT_DAYS: isize = 365;
#[allow(dead_code)]
pub const CONFIG_KERNEL_INIT_PRIORITY_OBJECTS: isize = 30;
#[allow(dead_code)]
pub const CONFIG_KERNEL_INIT_PRIORITY_LIBC: isize = 35;
#[allow(dead_code)]
pub const CONFIG_KERNEL_INIT_PRIORITY_DEFAULT: isize = 40;
#[allow(dead_code)]
pub const CONFIG_KERNEL_INIT_PRIORITY_DEVICE: isize = 50;
#[allow(dead_code)]
pub const CONFIG_APPLICATION_INIT_PRIORITY: isize = 90;
#[allow(dead_code)]
pub const CONFIG_CLOCK_CONTROL_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_CLOCK_CONTROL_ESP32_RTC_CLK_CAL_CYCLES: isize = 1024;
#[allow(dead_code)]
pub const CONFIG_CLOCK_CONTROL_TISCI_PRIORITY: isize = 40;
#[allow(dead_code)]
pub const CONFIG_CONSOLE_INPUT_MAX_LINE_LEN: isize = 128;
#[allow(dead_code)]
pub const CONFIG_CONSOLE_INIT_PRIORITY: isize = 60;
#[allow(dead_code)]
pub const CONFIG_UART_CONSOLE_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_GPIO_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_INTC_INIT_PRIORITY: isize = 40;
#[allow(dead_code)]
pub const CONFIG_INTC_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_PINCTRL_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SERIAL_INIT_PRIORITY: isize = 50;
#[allow(dead_code)]
pub const CONFIG_UART_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_UART_ESP32_TX_FIFO_THRESH: usize = 0x1;
#[allow(dead_code)]
pub const CONFIG_UART_ESP32_RX_FIFO_THRESH: usize = 0x16;
#[allow(dead_code)]
pub const CONFIG_SYS_HEAP_ALLOC_LOOPS: isize = 3;
#[allow(dead_code)]
pub const CONFIG_SYS_HEAP_HARDENING_LEVEL: isize = 1;
#[allow(dead_code)]
pub const CONFIG_SYS_HEAP_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_CBPRINTF_PACKAGE_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_CPU_LOAD_LOG_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_LOG_FLUSH_SLEEP_US: isize = 10000;
#[allow(dead_code)]
pub const CONFIG_LOG_DEFAULT_LEVEL: isize = 3;
#[allow(dead_code)]
pub const CONFIG_LOG_MAX_LEVEL: isize = 4;
#[allow(dead_code)]
pub const CONFIG_LOG_PROCESS_TRIGGER_THRESHOLD: isize = 10;
#[allow(dead_code)]
pub const CONFIG_LOG_PROCESS_THREAD_SLEEP_MS: isize = 1000;
#[allow(dead_code)]
pub const CONFIG_LOG_BUFFER_SIZE: isize = 1024;
#[allow(dead_code)]
pub const CONFIG_LOG_BACKEND_UART_BUFFER_SIZE: isize = 1;
#[allow(dead_code)]
pub const CONFIG_LOG_FAILURE_REPORT_PERIOD: isize = 1000;
#[allow(dead_code)]
pub const CONFIG_LOG_RATELIMIT_INTERVAL_MS: isize = 5000;
#[allow(dead_code)]
pub const CONFIG_TIMER_RANDOM_INITIAL_STATE: isize = 123456789;
#[allow(dead_code)]
pub const CONFIG_COVERAGE_DUMP_PATH_EXCLUDE: &str = "";
#[allow(dead_code)]
pub const CONFIG_LINKER_LAST_SECTION_ID_PATTERN: usize = 0xE015E015;
#[allow(dead_code)]
pub const CONFIG_LINKER_ITERABLE_SUBALIGN: isize = 4;
#[allow(dead_code)]
pub const CONFIG_COMPILER_OPT: &str = "";
#[allow(dead_code)]
pub const CONFIG_KERNEL_BIN_NAME: &str = "zephyr";
#[allow(dead_code)]
pub const CONFIG_BUILD_GAP_FILL_PATTERN: usize = 0xFF;
