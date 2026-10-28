//! Small synchronous boundary around the pinned SDK's RAM MAC table and eFuses.
//!
//! All calls run on the main task before any networking/radio service is
//! initialized. No other task calls these APIs. The SDK copies supplied arrays
//! synchronously and does not retain their pointers. No programming API is used.

use esp_idf_sys::{ESP_ERR_INVALID_SIZE, EspError, esp_mac_type_t};
use runtime_protocol::{
    BASE_OVERRIDE, BLUETOOTH_OVERRIDE, IEEE802154_OVERRIDE, Snapshot, WIFI_AP_OVERRIDE,
    WIFI_STATION_OVERRIDE,
};

const STATION: esp_mac_type_t = esp_idf_sys::esp_mac_type_t_ESP_MAC_WIFI_STA;
const ACCESS_POINT: esp_mac_type_t = esp_idf_sys::esp_mac_type_t_ESP_MAC_WIFI_SOFTAP;
const BLUETOOTH: esp_mac_type_t = esp_idf_sys::esp_mac_type_t_ESP_MAC_BT;
const IEEE802154: esp_mac_type_t = esp_idf_sys::esp_mac_type_t_ESP_MAC_IEEE802154;
const BASE: esp_mac_type_t = esp_idf_sys::esp_mac_type_t_ESP_MAC_BASE;

fn status(code: i32) -> Result<(), EspError> {
    match EspError::from(code) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn size_error() -> EspError {
    EspError::from_infallible::<ESP_ERR_INVALID_SIZE>()
}

fn read<const N: usize>(kind: esp_mac_type_t) -> Result<[u8; N], EspError> {
    // SAFETY: This generated C ABI accepts a numeric type and returns its table
    // width, with no pointer arguments or retained resources.
    if unsafe { esp_idf_sys::esp_mac_addr_len_get(kind) } != N {
        return Err(size_error());
    }
    let mut bytes = [0; N];
    // SAFETY: Width was checked against this exact N-byte writable array; the
    // main task has exclusive API use and the C function writes synchronously.
    status(unsafe { esp_idf_sys::esp_read_mac(bytes.as_mut_ptr(), kind) })?;
    Ok(bytes)
}

fn set<const N: usize>(kind: esp_mac_type_t, bytes: &[u8; N]) -> Result<(), EspError> {
    // SAFETY: As above, this C ABI has no pointer argument or mutable operation.
    if unsafe { esp_idf_sys::esp_mac_addr_len_get(kind) } != N {
        return Err(size_error());
    }
    // SAFETY: The selected SDK width equals N, bytes is valid/readable for the
    // synchronous memcpy, no pointer is retained, and main exclusively owns
    // this software table. This API updates RAM and cannot program eFuses.
    status(unsafe { esp_idf_sys::esp_iface_mac_addr_set(bytes.as_ptr(), kind) })
}

/// Read the direct factory EUI-64 and the five selected software table entries.
pub(crate) fn snapshot() -> Result<Snapshot, EspError> {
    let mut factory = [0; 8];
    // SAFETY: On this pinned C6 target SOC_IEEE802154_SUPPORTED is true, so
    // esp_efuse_mac_get_default writes all eight bytes including MAC_EXT.
    // The local writable array is exactly eight bytes, is live throughout the
    // synchronous call, and is not accessed concurrently or retained by C.
    status(unsafe { esp_idf_sys::esp_efuse_mac_get_default(factory.as_mut_ptr()) })?;
    Ok(Snapshot {
        factory,
        base: read(BASE)?,
        wifi_station: read(STATION)?,
        wifi_ap: read(ACCESS_POINT)?,
        bluetooth: read(BLUETOOTH)?,
        ieee802154: read(IEEE802154)?,
    })
}

/// Change only BASE, preserving the interface entries populated by the baseline.
pub(crate) fn set_base() -> Result<(), EspError> {
    set_base_value(&BASE_OVERRIDE)
}

fn set_base_value(bytes: &[u8; 6]) -> Result<(), EspError> {
    // SAFETY: The pinned base setter reads exactly six bytes synchronously and
    // delegates to its RAM-table setter. This six-byte array is live/readable,
    // no pointer is retained, and main has exclusive table access.
    status(unsafe { esp_idf_sys::esp_base_mac_addr_set(bytes.as_ptr()) })
}

/// Set all four software interface entries; no hardware radio setter is called.
pub(crate) fn set_interfaces() -> Result<(), EspError> {
    set(STATION, &WIFI_STATION_OVERRIDE)?;
    set(ACCESS_POINT, &WIFI_AP_OVERRIDE)?;
    set(BLUETOOTH, &BLUETOOTH_OVERRIDE)?;
    set(IEEE802154, &IEEE802154_OVERRIDE)
}

/// Attempt every saved RAM-table restoration even after an earlier set fails.
pub(crate) fn restore(saved: &Snapshot) -> Result<(), EspError> {
    let outcomes = [
        set_base_value(&saved.base),
        set(STATION, &saved.wifi_station),
        set(ACCESS_POINT, &saved.wifi_ap),
        set(BLUETOOTH, &saved.bluetooth),
        set(IEEE802154, &saved.ieee802154),
    ];
    for outcome in outcomes {
        outcome?;
    }
    Ok(())
}
