//! Two-phase AP adapter retaining one driver and one saved original MAC.

use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, Configuration, WifiDeviceId, WifiDriver,
};
use esp_idf_sys::ESP_ERR_TIMEOUT;
use wifi_protocol::restart::{AP_CHANNEL, Phase, Snapshot, State, ssid, valid_original_mac};

use crate::restart_controller::{ADAPTER_FAILURE, Radio};

const STATE_WAIT: Duration = Duration::from_secs(1);

pub(crate) struct RestartRadio {
    modem: Option<Modem<'static>>,
    sysloop: Option<EspSystemEventLoop>,
    driver: Option<WifiDriver<'static>>,
    nonce: Option<[u8; 8]>,
    phase: Option<Phase>,
    original_mac: Option<[u8; 6]>,
    state: State,
    auto_stopped: bool,
    checked_stop: bool,
    constructor_error: Option<i32>,
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

impl RestartRadio {
    pub(crate) fn new(modem: Modem<'static>) -> Self {
        Self {
            modem: Some(modem),
            sysloop: None,
            driver: None,
            nonce: None,
            phase: None,
            original_mac: None,
            state: State::Idle,
            auto_stopped: false,
            checked_stop: false,
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
            phase: self.phase,
            nonce: self.nonce,
            initialized: self.driver.is_some(),
            running,
            mac,
            original_mac: self.original_mac,
            auto_stopped: self.auto_stopped,
            first_failure: None,
        })
    }

    fn initialize_first(&mut self, nonce: [u8; 8]) -> Result<(), i32> {
        let sysloop = EspSystemEventLoop::take().map_err(|error| error.code())?;
        self.sysloop = Some(sysloop.clone());
        let modem = self.modem.take().ok_or(ADAPTER_FAILURE)?;
        // Initialization precedes the wrapper's event subscription. Constructor
        // failure can therefore leave unknown partial SDK state without a
        // handle. Never report such a boot as clean, and never call start.
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
        if !valid_original_mac(original) {
            return Err(ADAPTER_FAILURE);
        }
        self.original_mac = Some(original);
        let ssid_bytes = ssid(nonce);
        let text = core::str::from_utf8(&ssid_bytes).map_err(|_| ADAPTER_FAILURE)?;
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
            .push_str(text)
            .map_err(|_| ADAPTER_FAILURE)?;
        // Configuration is installed once. Stop retains these RAM settings;
        // the second phase changes only the disabled interface's MAC.
        driver
            .set_configuration(&Configuration::AccessPoint(configuration))
            .map_err(|error| error.code())
    }
}

impl Radio for RestartRadio {
    fn status(&mut self) -> Result<Snapshot, i32> {
        self.snapshot()
    }

    fn start(&mut self, phase: Phase, nonce: [u8; 8]) -> Result<Snapshot, i32> {
        if let Some(code) = self.constructor_error {
            return Err(code);
        }
        match phase {
            Phase::First => {
                if self.phase.is_some() || self.nonce.is_some() || self.driver.is_some() {
                    return Err(ADAPTER_FAILURE);
                }
                self.phase = Some(phase);
                self.nonce = Some(nonce);
                self.state = State::Stopped;
                self.initialize_first(nonce)?;
            }
            Phase::Second => {
                if self.phase != Some(Phase::First) || self.nonce != Some(nonce) {
                    return Err(ADAPTER_FAILURE);
                }
                // Tag the consumed attempt before fallible restart guards so
                // checked cleanup remains correlated even when a read fails.
                self.phase = Some(phase);
                self.auto_stopped = false;
                if self.state != State::Stopped || !self.checked_stop {
                    return Err(ADAPTER_FAILURE);
                }
                let original = self.original_mac.ok_or(ADAPTER_FAILURE)?;
                let driver = self.driver.as_ref().ok_or(ADAPTER_FAILURE)?;
                if !valid_original_mac(original)
                    || driver.is_ap_started().map_err(|error| error.code())?
                    || driver
                        .get_mac(WifiDeviceId::Ap)
                        .map_err(|error| error.code())?
                        != original
                {
                    return Err(ADAPTER_FAILURE);
                }
            }
        }
        self.auto_stopped = false;
        let driver = self.driver.as_mut().ok_or(ADAPTER_FAILURE)?;
        if driver.is_sta_enabled().map_err(|error| error.code())?
            || !driver.is_ap_enabled().map_err(|error| error.code())?
            || driver.is_ap_started().map_err(|error| error.code())?
        {
            return Err(ADAPTER_FAILURE);
        }
        driver
            .set_mac(WifiDeviceId::Ap, phase.mac())
            .map_err(|error| error.code())?;
        if driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| error.code())?
            != phase.mac()
        {
            return Err(ADAPTER_FAILURE);
        }
        // Reconfiguration while disabled preserves prior Stop evidence. Clear
        // it immediately BEFORE the actual SDK start attempt, including an
        // attempt that returns an error after partial driver work.
        self.checked_stop = false;
        driver.start().map_err(|error| error.code())?;
        wait_running(driver, true)?;
        if driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| error.code())?
            != phase.mac()
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
        if self.phase.is_none() {
            return self.snapshot();
        }
        self.auto_stopped |= automatic;
        if let Some(driver) = self.driver.as_mut() {
            if !self.checked_stop || driver.is_ap_started().map_err(|error| error.code())? {
                self.checked_stop = false;
                driver.stop().map_err(|error| error.code())?;
                wait_running(driver, false)?;
                self.checked_stop = true;
            }
            let original = match self.original_mac {
                Some(original) => original,
                None => {
                    // No setter runs before a valid original has been saved.
                    // Retry a failed initial getter without replacing a saved
                    // value; this branch cannot execute during phase two.
                    let original = driver
                        .get_mac(WifiDeviceId::Ap)
                        .map_err(|error| error.code())?;
                    if !valid_original_mac(original) {
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
