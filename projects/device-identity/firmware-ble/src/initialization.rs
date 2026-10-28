//! C6-only SDK initialization, retaining exclusive HAL modem ownership.
//!
//! Configuration transcribes ESP-IDF5.5.6 esp_bt.h C6
//! BT_CONTROLLER_INIT_CONFIG_DEFAULT, compared with pinned esp-idf-svc
//! a209052 BtDriver::init. Upstream controller fields are Espressif
//! SPDX-License-Identifier: Apache-2.0; no authored C shim is used.
use core::sync::atomic::{AtomicBool, Ordering};
use device_identity_ble_firmware::controller::ADAPTER_FAILURE;
use esp_idf_hal::modem::Modem;
use esp_idf_sys as sys;
static TAKEN: AtomicBool = AtomicBool::new(false);

/// Initialized state is retained without an automatic deinitializer. Main never
/// normally returns after initialization; partial failure requires host reset.
pub(crate) struct Driver {
    _modem: Modem<'static>,
    _configuration: *mut sys::esp_bt_controller_config_t,
}
impl Driver {
    pub(crate) fn new(modem: Modem<'static>) -> Result<Self, i32> {
        if TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(ADAPTER_FAILURE);
        }
        // One bounded allocation is deliberately retained until reset, including
        // partial failure. The SDK forwards this pointer into a binary library;
        // do not infer that opaque code stops using it after init returns.
        let config = Box::into_raw(Box::new(configuration()));
        // SAFETY: exact generated SDK layout and C6 default fields; exclusive
        // consumed Modem plus a permanent one-attempt latch prevents another
        // controller owner, including after partial failure. Config is aligned,
        // initialized, permanently live, and has no other Rust reference/owner.
        check(unsafe { sys::esp_bt_controller_init(config) })?;
        // SAFETY: successful prior initialization, fixed BLE-only SDK mode.
        check(unsafe { sys::esp_bt_controller_enable(sys::esp_bt_mode_t_ESP_BT_MODE_BLE) })?;
        // SAFETY: initialized enabled controller, one Bluedroid owner, compiled
        // GATT/SMP/persistent-key features disabled. No NVS initialization.
        check(unsafe { sys::esp_bluedroid_init() })?;
        // SAFETY: successful Bluedroid init above; not called more than once.
        check(unsafe { sys::esp_bluedroid_enable() })?;
        Ok(Self {
            _modem: modem,
            _configuration: config,
        })
    }
}
fn check(code: i32) -> Result<(), i32> {
    if code == 0 { Ok(()) } else { Err(code) }
}
fn configuration() -> sys::esp_bt_controller_config_t {
    // The chip-revision getter only reads silicon metadata. It has no pointer
    // arguments and runs under exclusive peripheral ownership before init.
    sys::esp_bt_controller_config_t {
        config_version: (sys::CONFIG_VERSION) as _,
        ble_ll_resolv_list_size: (sys::CONFIG_BT_LE_LL_RESOLV_LIST_SIZE) as _,
        ble_hci_evt_hi_buf_count: (sys::DEFAULT_BT_LE_HCI_EVT_HI_BUF_COUNT) as _,
        ble_hci_evt_lo_buf_count: (sys::DEFAULT_BT_LE_HCI_EVT_LO_BUF_COUNT) as _,
        ble_ll_sync_list_cnt: (sys::DEFAULT_BT_LE_MAX_PERIODIC_ADVERTISER_LIST) as _,
        ble_ll_sync_cnt: (sys::DEFAULT_BT_LE_MAX_PERIODIC_SYNCS) as _,
        ble_ll_rsp_dup_list_count: (sys::CONFIG_BT_LE_LL_DUP_SCAN_LIST_COUNT) as _,
        ble_ll_adv_dup_list_count: (sys::CONFIG_BT_LE_LL_DUP_SCAN_LIST_COUNT) as _,
        ble_ll_tx_pwr_dbm: (sys::BLE_LL_TX_PWR_DBM_N) as _,
        rtc_freq: (sys::RTC_FREQ_N) as _,
        ble_ll_sca: (sys::CONFIG_BT_LE_LL_SCA) as _,
        ble_ll_scan_phy_number: (sys::BLE_LL_SCAN_PHY_NUMBER_N) as _,
        ble_ll_conn_def_auth_pyld_tmo: (sys::BLE_LL_CONN_DEF_AUTH_PYLD_TMO_N) as _,
        ble_ll_jitter_usecs: (sys::BLE_LL_JITTER_USECS_N) as _,
        ble_ll_sched_max_adv_pdu_usecs: (sys::BLE_LL_SCHED_MAX_ADV_PDU_USECS_N) as _,
        ble_ll_sched_direct_adv_max_usecs: (sys::BLE_LL_SCHED_DIRECT_ADV_MAX_USECS_N) as _,
        ble_ll_sched_adv_max_usecs: (sys::BLE_LL_SCHED_ADV_MAX_USECS_N) as _,
        ble_scan_rsp_data_max_len: (sys::DEFAULT_BT_LE_SCAN_RSP_DATA_MAX_LEN_N) as _,
        ble_ll_cfg_num_hci_cmd_pkts: (sys::BLE_LL_CFG_NUM_HCI_CMD_PKTS_N) as _,
        ble_ll_ctrl_proc_timeout_ms: (sys::BLE_LL_CTRL_PROC_TIMEOUT_MS_N) as _,
        nimble_max_connections: (sys::DEFAULT_BT_LE_MAX_CONNECTIONS) as _,
        ble_whitelist_size: (sys::DEFAULT_BT_NIMBLE_WHITELIST_SIZE) as _,
        ble_acl_buf_size: (sys::DEFAULT_BT_LE_ACL_BUF_SIZE) as _,
        ble_acl_buf_count: (sys::DEFAULT_BT_LE_ACL_BUF_COUNT) as _,
        ble_hci_evt_buf_size: (sys::DEFAULT_BT_LE_HCI_EVT_BUF_SIZE) as _,
        ble_multi_adv_instances: (sys::DEFAULT_BT_LE_MAX_EXT_ADV_INSTANCES) as _,
        ble_ext_adv_max_size: (sys::DEFAULT_BT_LE_EXT_ADV_MAX_SIZE) as _,
        controller_task_stack_size: (sys::NIMBLE_LL_STACK_SIZE) as _,
        controller_task_prio: (sys::ESP_TASK_BT_CONTROLLER_PRIO) as _,
        controller_run_cpu: (0) as _,
        enable_qa_test: (sys::RUN_QA_TEST) as _,
        enable_bqb_test: (sys::RUN_BQB_TEST) as _,
        enable_tx_cca: (sys::DEFAULT_BT_LE_TX_CCA_ENABLED) as _,
        cca_rssi_thresh: (256 - sys::DEFAULT_BT_LE_CCA_RSSI_THRESH) as _,
        sleep_en: (sys::NIMBLE_SLEEP_ENABLE) as _,
        coex_phy_coded_tx_rx_time_limit: (sys::DEFAULT_BT_LE_COEX_PHY_CODED_TX_RX_TLIM_EFF) as _,
        dis_scan_backoff: (sys::NIMBLE_DISABLE_SCAN_BACKOFF) as _,
        ble_scan_classify_filter_enable: (1) as _,
        main_xtal_freq: (sys::CONFIG_XTAL_FREQ) as _,
        version_num: (unsafe { sys::efuse_hal_chip_revision() }) as _,
        cpu_freq_mhz: (sys::CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ) as _,
        ignore_wl_for_direct_adv: (0) as _,
        enable_pcl: (sys::DEFAULT_BT_LE_POWER_CONTROL_ENABLED) as _,
        csa2_select: (sys::DEFAULT_BT_LE_50_FEATURE_SUPPORT) as _,
        enable_csr: (sys::DEFAULT_BT_LE_SUBRATE_ENABLED) as _,
        ble_aa_check: (sys::DEFAULT_BT_LE_CTRL_CHECK_CONNECT_IND_ACCESS_ADDRESS) as _,
        ble_llcp_disc_flag: (sys::BT_LE_CTRL_LLCP_DISC_FLAG) as _,
        scan_backoff_upperlimitmax: (sys::BT_CTRL_SCAN_BACKOFF_UPPERLIMITMAX) as _,
        ble_chan_ass_en: (sys::DEFAULT_BT_LE_CTRL_CHAN_ASS_EN) as _,
        ble_data_lenth_zero_aux: (sys::DEFAULT_BT_LE_CTRL_ADV_DATA_LENGTH_ZERO_AUX) as _,
        vhci_enabled: (sys::DEFAULT_BT_LE_VHCI_ENABLED) as _,
        ptr_check_enabled: (sys::DEFAULT_BT_LE_PTR_CHECK_ENABLED) as _,
        ble_adv_tx_options: (sys::DEFAULT_BT_LE_ADV_TX_OPTIONS) as _,
        skip_unnecessary_checks_en: (0) as _,
        fast_conn_data_tx_en: (sys::DEFAULT_BT_LE_CTRL_FAST_CONN_DATA_TX_EN) as _,
        ch39_txpwr: (sys::BLE_CH39_TX_PWR_DBM_N) as _,
        adv_rsv_cnt: (core::cmp::min(
            sys::DEFAULT_BT_LE_MAX_EXT_ADV_INSTANCES,
            sys::CONFIG_BT_LE_EXT_ADV_RESERVED_MEMORY_COUNT,
        )) as _,
        conn_rsv_cnt: (core::cmp::min(
            sys::DEFAULT_BT_LE_MAX_CONNECTIONS,
            sys::CONFIG_BT_LE_CONN_RESERVED_MEMORY_COUNT,
        )) as _,
        priority_level_cfg: (sys::BT_LL_CTRL_PRIO_LVL_CFG) as _,
        slv_fst_rx_lat_en: (sys::DEFAULT_BT_LE_CTRL_SLV_FAST_RX_CONN_DATA_EN) as _,
        dl_itvl_phy_sync_en: (sys::DEFAULT_BT_LE_CTRL_DL_ITVL_PHY_SYNC_EN) as _,
        scan_allow_adi_filter: (sys::DEFAULT_BT_SCAN_ALLOW_ENH_ADI_FILTER) as _,
        enhanced_mem_resv: (sys::DEFAULT_BT_LE_CTRL_ENH_MEM_RESV_ADV) as _,
        rxbuf_reserved: (sys::DEFAULT_BT_LE_CTRL_RXBUF_MEM_RESV) as _,
        config_magic: (sys::CONFIG_MAGIC) as _,
        ..Default::default()
    }
}
