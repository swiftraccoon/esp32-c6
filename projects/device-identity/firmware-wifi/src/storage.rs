//! The pinned safe Wi-Fi driver does not expose its SDK storage selector.

use esp_idf_sys::{ESP_OK, esp_wifi_set_storage, wifi_storage_t_WIFI_STORAGE_RAM};

/// Select RAM before the first AP configuration. Called only on the main task,
/// after an owned WifiDriver initialized the SDK and before start/configuration.
pub(crate) fn use_ram() -> Result<(), i32> {
    // SAFETY: ESP-IDF 5.5.6's generated binding takes a scalar storage enum;
    // WIFI_STORAGE_RAM is its declared valid variant. Initialization precedes
    // this call, no pointers/callbacks are passed, and this application has one
    // main-task owner of Wi-Fi configuration. No concurrent SDK user exists.
    let status = unsafe { esp_wifi_set_storage(wifi_storage_t_WIFI_STORAGE_RAM) };
    if status == ESP_OK {
        Ok(())
    } else {
        Err(status)
    }
}
