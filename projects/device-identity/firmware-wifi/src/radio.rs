//! Single-owner AP adapter. Its initialized driver is retained until reset.

use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, Configuration, WifiDeviceId, WifiDriver,
};
use esp_idf_sys::ESP_ERR_TIMEOUT;
use wifi_protocol::{AP_CHANNEL, AP_MAC, Snapshot, State, ssid};

use crate::controller::{ADAPTER_FAILURE, Radio};

const STATE_WAIT: Duration = Duration::from_secs(1);

pub(crate) struct ApRadio {
    modem: Option<Modem<'static>>,
    // Keep the loop/subscription and driver owned for the rest of this boot.
    // Main never returns after construction; ordinary SDK failures cannot run
    // WifiDriver's implicit stop/deinit destructor.
    sysloop: Option<EspSystemEventLoop>,
    driver: Option<WifiDriver<'static>>,
    nonce: Option<[u8; 8]>,
    original_mac: Option<[u8; 6]>,
    state: State,
    auto_stopped: bool,
    previously_checked_stop: bool,
    constructor_error: Option<i32>,
}

fn valid_original(mac: [u8; 6]) -> bool {
    mac != [0; 6] && mac != AP_MAC && mac[0] & 1 == 0
}

fn wait_running(driver: &WifiDriver<'_>, expected: bool) -> Result<(), i32> {
    let deadline = Instant::now() + STATE_WAIT;
    loop {
        if driver.is_ap_started().map_err(|error| error.code())? == expected {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(ESP_ERR_TIMEOUT);
        }
        thread::sleep(Duration::from_millis(1));
    }
}

impl ApRadio {
    pub(crate) fn new(modem: Modem<'static>) -> Self {
        Self {
            modem: Some(modem),
            sysloop: None,
            driver: None,
            nonce: None,
            original_mac: None,
            state: State::Idle,
            auto_stopped: false,
            previously_checked_stop: false,
            constructor_error: None,
        }
    }

    fn snapshot(&self) -> Result<Snapshot, i32> {
        if let Some(code) = self.constructor_error {
            return Err(code);
        }
        let (running, mac) = match &self.driver {
            Some(driver) => (
                driver.is_ap_started().map_err(|error| error.code())?,
                Some(
                    driver
                        .get_mac(WifiDeviceId::Ap)
                        .map_err(|error| error.code())?,
                ),
            ),
            None => (false, None),
        };
        Ok(Snapshot {
            state: self.state,
            nonce: self.nonce,
            initialized: self.driver.is_some(),
            running,
            mac,
            original_mac: self.original_mac,
            auto_stopped: self.auto_stopped,
        })
    }
}

impl Radio for ApRadio {
    fn status(&mut self) -> Result<Snapshot, i32> {
        self.snapshot()
    }

    fn start(&mut self, nonce: [u8; 8]) -> Result<Snapshot, i32> {
        if self.nonce.is_some() {
            return Err(ADAPTER_FAILURE);
        }
        self.nonce = Some(nonce);
        self.state = State::Stopped;
        self.previously_checked_stop = false;
        let sysloop = EspSystemEventLoop::take().map_err(|error| error.code())?;
        self.sysloop = Some(sysloop.clone());
        let modem = self.modem.take().ok_or(ADAPTER_FAILURE)?;
        // The pinned constructor initializes before subscribing to events. On
        // failure, initialization may be partial without an owned handle. Keep
        // that uncertainty terminal; never claim a clean uninitialized state.
        let driver = match WifiDriver::new(modem, sysloop, None) {
            Ok(driver) => driver,
            Err(error) => {
                self.constructor_error = Some(error.code());
                return Err(error.code());
            }
        };
        self.driver = Some(driver);
        crate::storage::use_ram()?;
        let driver = self.driver.as_mut().ok_or(ADAPTER_FAILURE)?;
        let original = driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| error.code())?;
        if !valid_original(original) {
            return Err(ADAPTER_FAILURE);
        }
        self.original_mac = Some(original);
        let ssid_bytes = ssid(nonce);
        let ssid_text = core::str::from_utf8(&ssid_bytes).map_err(|_| ADAPTER_FAILURE)?;
        let mut configuration = AccessPointConfiguration {
            channel: AP_CHANNEL,
            auth_method: AuthMethod::None,
            max_connections: 1,
            ssid_hidden: false,
            secondary_channel: None,
            ..Default::default()
        };
        configuration.ssid.clear();
        configuration
            .ssid
            .push_str(ssid_text)
            .map_err(|_| ADAPTER_FAILURE)?;
        driver
            .set_configuration(&Configuration::AccessPoint(configuration))
            .map_err(|error| error.code())?;
        if driver.is_sta_enabled().map_err(|error| error.code())?
            || !driver.is_ap_enabled().map_err(|error| error.code())?
            || driver.is_ap_started().map_err(|error| error.code())?
        {
            return Err(ADAPTER_FAILURE);
        }
        // Only this disabled AP interface is changed; no base/STA setter runs.
        driver
            .set_mac(WifiDeviceId::Ap, AP_MAC)
            .map_err(|error| error.code())?;
        if driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| error.code())?
            != AP_MAC
        {
            return Err(ADAPTER_FAILURE);
        }
        driver.start().map_err(|error| error.code())?;
        wait_running(driver, true)?;
        if driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| error.code())?
            != AP_MAC
        {
            return Err(ADAPTER_FAILURE);
        }
        self.state = State::Active;
        self.snapshot()
    }

    fn stop(&mut self, nonce: [u8; 8], automatic: bool) -> Result<Snapshot, i32> {
        if self.nonce.is_some_and(|saved| saved != nonce) {
            return Err(ADAPTER_FAILURE);
        }
        if let Some(code) = self.constructor_error {
            return Err(code);
        }
        if self.nonce.is_none() {
            return self.snapshot();
        }
        self.auto_stopped |= automatic;
        if let Some(driver) = self.driver.as_mut() {
            // Only a previous successful SDK stop followed by an inactive
            // event state can authorize skipping a redundant SDK stop. The
            // application State::Stopped is also used during incomplete setup
            // and must never establish this condition by itself.
            let must_stop = !self.previously_checked_stop
                || driver.is_ap_started().map_err(|error| error.code())?;
            if must_stop {
                self.previously_checked_stop = false;
                driver.stop().map_err(|error| error.code())?;
                wait_running(driver, false)?;
                self.previously_checked_stop = true;
            }
            let original = match self.original_mac {
                Some(original) => original,
                None => {
                    // No override occurs until this value is saved. A getter
                    // failure during setup can therefore be retried safely.
                    let original = driver
                        .get_mac(WifiDeviceId::Ap)
                        .map_err(|error| error.code())?;
                    if !valid_original(original) {
                        return Err(ADAPTER_FAILURE);
                    }
                    self.original_mac = Some(original);
                    original
                }
            };
            driver
                .set_mac(WifiDeviceId::Ap, original)
                .map_err(|error| error.code())?;
            if driver
                .get_mac(WifiDeviceId::Ap)
                .map_err(|error| error.code())?
                != original
            {
                return Err(ADAPTER_FAILURE);
            }
        }
        self.state = State::Stopped;
        self.snapshot()
    }
}
