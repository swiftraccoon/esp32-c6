//! Generated-binding FFI. Only the main task calls SDK functions; the static
//! GAP callback publishes atomics without allocating, blocking or logging.
use device_identity_ble_firmware::completion::{Kind, Mailbox};
use device_identity_ble_firmware::controller::ADAPTER_FAILURE;
use device_identity_ble_firmware::protocol::{AddressReadback, AddressType, Operation};
use esp_idf_sys as sys;

pub(crate) static EVENTS: Mailbox = Mailbox::new();
fn result(code: i32) -> Result<(), i32> {
    if code == 0 { Ok(()) } else { Err(code) }
}

// SAFETY: registered once after Bluedroid initialization, retained until reset.
// SDK owns the union and guarantees the member selected by event is live for
// this callback. A null pointer is rejected before union access. No reference,
// union value or callback pointer escapes; only integer status enters atomics.
unsafe extern "C" fn gap(
    event: sys::esp_gap_ble_cb_event_t,
    param: *mut sys::esp_ble_gap_cb_param_t,
) {
    use sys::*;
    const DATA_EVENT: esp_gap_ble_cb_event_t =
        esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_DATA_RAW_SET_COMPLETE_EVT;
    const ADDRESS_EVENT: esp_gap_ble_cb_event_t =
        esp_gap_ble_cb_event_t_ESP_GAP_BLE_SET_STATIC_RAND_ADDR_EVT;
    const START_EVENT: esp_gap_ble_cb_event_t =
        esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_START_COMPLETE_EVT;
    const STOP_EVENT: esp_gap_ble_cb_event_t =
        esp_gap_ble_cb_event_t_ESP_GAP_BLE_ADV_STOP_COMPLETE_EVT;
    let kind = match event {
        DATA_EVENT => Kind::Data,
        ADDRESS_EVENT => Kind::Address,
        START_EVENT => Kind::Start,
        STOP_EVENT => Kind::Stop,
        _ => {
            EVENTS.unexpected(ADAPTER_FAILURE);
            return;
        }
    };
    if param.is_null() {
        EVENTS.unexpected(ADAPTER_FAILURE);
        return;
    }
    // SAFETY: event/member correspondence and null guard above; SDK's callback
    // ABI supplies aligned readable storage that remains live until return.
    let status = unsafe {
        match kind {
            Kind::Data => (*param).adv_data_raw_cmpl.status,
            Kind::Address => (*param).set_rand_addr_cmpl.status,
            Kind::Start => (*param).adv_start_cmpl.status,
            Kind::Stop => (*param).adv_stop_cmpl.status,
        }
    };
    EVENTS.complete(kind, i32::try_from(status).unwrap_or(ADAPTER_FAILURE));
}

pub(crate) fn register() -> Result<(), i32> {
    // SAFETY: one static callback, one retained initialized driver, no other GAP
    // owner or callback registration exists. No borrowed callback state.
    result(unsafe { sys::esp_ble_gap_register_callback(Some(gap)) })
}
pub(crate) fn configure_data(data: &mut [u8; 26]) -> Result<(), i32> {
    // SAFETY: the full fixed array is readable/writable for this call and remains
    // owned until reset. Pinned API synchronously copies it through the BTC deep
    // copy hook before return, then frees its own copy after queued processing.
    result(unsafe { sys::esp_ble_gap_config_adv_data_raw(data.as_mut_ptr(), 26) })
}
pub(crate) fn set_address(mut address: [u8; 6]) -> Result<(), i32> {
    // SAFETY: six initialized bytes, synchronous copy into the BTC request; no
    // pointer is retained. Called only after checked inactive advertising.
    result(unsafe { sys::esp_ble_gap_set_rand_addr(address.as_mut_ptr()) })
}
pub(crate) fn start(operation: Operation) -> Result<(), i32> {
    device_identity_ble_firmware::controller::check_advertise_operation(operation)?;
    let mut params = sys::esp_ble_adv_params_t {
        adv_int_min: 160,
        adv_int_max: 160,
        adv_type: sys::esp_ble_adv_type_t_ADV_TYPE_NONCONN_IND,
        own_addr_type: sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_RANDOM,
        peer_addr: [0; 6],
        peer_addr_type: sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC,
        channel_map: sys::esp_ble_adv_channel_t_ADV_CHNL_ALL,
        adv_filter_policy: sys::esp_ble_adv_filter_t_ADV_FILTER_ALLOW_SCAN_ANY_CON_ANY,
    };
    // SAFETY: generated layout, valid fixed legacy settings, stable live stack
    // struct; the API copies all fields into its queued BTC argument on return.
    result(unsafe { sys::esp_ble_gap_start_advertising(&mut params) })
}
pub(crate) fn stop() -> Result<(), i32> {
    // SAFETY: main-task-only request on retained initialized Bluedroid; no raw
    // pointers. Async completion is required before reporting checked Stop.
    result(unsafe { sys::esp_ble_gap_stop_advertising() })
}
pub(crate) fn read_address() -> AddressReadback {
    let mut address = [0; 6];
    let mut address_type = 255;
    // SAFETY: independent initialized outputs of exact generated widths; the
    // synchronous SDK getter copies six bytes and one address-type byte.
    let code =
        unsafe { sys::esp_ble_gap_get_local_used_addr(address.as_mut_ptr(), &mut address_type) };
    if code != 0 {
        return AddressReadback::Unavailable { status: code };
    }
    let address_type = match u32::from(address_type) {
        sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC => AddressType::Public,
        sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_RANDOM => AddressType::Random,
        _ => {
            return AddressReadback::Unavailable {
                status: ADAPTER_FAILURE,
            };
        }
    };
    AddressReadback::Available {
        address,
        address_type,
    }
}
