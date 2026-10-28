//! Exclusive main-task adapter; no driver deinitialization occurs before reset.
use crate::initialization::Driver;
use crate::sdk::{self, EVENTS};
use device_identity_ble_firmware::completion::Kind;
use device_identity_ble_firmware::controller::{ADAPTER_FAILURE, Radio};
use device_identity_ble_firmware::protocol::{
    AddressReadback, Operation, Phase, Snapshot, State, nonce_ad,
};
use esp_idf_hal::modem::Modem;
use esp_idf_sys::ESP_ERR_TIMEOUT;
use std::thread;
use std::time::{Duration, Instant};

pub(crate) struct BleRadio {
    modem: Option<Modem<'static>>,
    driver: Option<Driver>,
    constructor_error: Option<i32>,
    epoch: Instant,
    phase: Option<Phase>,
    nonce: Option<[u8; 8]>,
    state: State,
    data: [u8; 26],
    data_configured: bool,
    address_set_completed: bool,
    prestart: AddressReadback,
    checked_stop: bool,
    auto_stopped: bool,
}
impl BleRadio {
    pub(crate) fn new(modem: Modem<'static>, epoch: Instant) -> Self {
        Self {
            modem: Some(modem),
            driver: None,
            constructor_error: None,
            epoch,
            phase: None,
            nonce: None,
            state: State::Idle,
            data: [0; 26],
            data_configured: false,
            address_set_completed: false,
            prestart: AddressReadback::NotRequested,
            checked_stop: false,
            auto_stopped: false,
        }
    }
    fn now(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
    fn run(
        &self,
        kind: Kind,
        cleanup: bool,
        phase_deadline: Option<u64>,
        call: impl FnOnce() -> Result<(), i32>,
    ) -> Result<(), i32> {
        if phase_deadline.is_some_and(|at| self.now() >= at) {
            return Err(ESP_ERR_TIMEOUT);
        }
        EVENTS.arm(kind, cleanup)?;
        if let Err(code) = call() {
            EVENTS.unexpected(code);
            return Err(code);
        }
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            let result = EVENTS.take(kind);
            // A completion at/after the deadline remains failed even if the
            // callback completed during a scheduling pause before this check.
            if Instant::now() >= until || phase_deadline.is_some_and(|at| self.now() >= at) {
                EVENTS.unexpected(ESP_ERR_TIMEOUT);
                return Err(ESP_ERR_TIMEOUT);
            }
            if let Some(code) = result {
                if code != 0 {
                    return Err(code);
                }
                if !cleanup && let Some(code) = EVENTS.fault() {
                    return Err(code);
                }
                return Ok(());
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    fn snapshot(&self) -> Result<Snapshot, i32> {
        if let Some(code) = self.constructor_error {
            return Err(code);
        }
        let advertising = if self.driver.is_some() {
            EVENTS.advertising().ok_or(ADAPTER_FAILURE)?
        } else {
            false
        };
        Ok(Snapshot {
            state: self.state,
            phase: self.phase,
            nonce: self.nonce,
            initialized: self.driver.is_some(),
            advertising,
            data_configured: self.data_configured,
            address_set_completed: self.address_set_completed,
            prestart_readback: self.prestart,
            used_readback: if self.driver.is_some() {
                sdk::read_address()
            } else {
                AddressReadback::NotRequested
            },
            auto_stopped: self.auto_stopped,
            first_failure: None,
        })
    }
}
impl Radio for BleRadio {
    fn fault(&self) -> Option<i32> {
        EVENTS.fault()
    }
    fn cleanup_needed(&self) -> bool {
        self.driver.is_some() && EVENTS.advertising() != Some(false)
    }
    fn status(&mut self) -> Result<Snapshot, i32> {
        self.snapshot()
    }
    fn start(&mut self, phase: Phase, nonce: [u8; 8], deadline_ms: u64) -> Result<Snapshot, i32> {
        let allowed = match phase {
            Phase::First => self.phase.is_none() && self.driver.is_none(),
            Phase::Second => {
                self.phase == Some(Phase::First)
                    && self.nonce == Some(nonce)
                    && self.checked_stop
                    && self.state == State::Stopped
                    && EVENTS.advertising() == Some(false)
                    && !EVENTS.pending()
            }
        };
        if !allowed || self.constructor_error.is_some() {
            return Err(ADAPTER_FAILURE);
        }
        self.phase = Some(phase);
        self.nonce = Some(nonce);
        self.state = State::Stopped;
        self.auto_stopped = false;
        self.address_set_completed = false;
        self.prestart = AddressReadback::NotRequested;
        if phase == Phase::First {
            let modem = self.modem.take().ok_or(ADAPTER_FAILURE)?;
            let driver = match Driver::new(modem) {
                Ok(d) => d,
                Err(code) => {
                    self.constructor_error = Some(code);
                    return Err(code);
                }
            };
            self.driver = Some(driver);
            sdk::register()?;
            self.data = nonce_ad(nonce);
            // The data stays in this owned adapter until reset. The SDK's
            // deep-copy allocation is separate and not a borrowed callback buffer.
            EVENTS.arm(Kind::Data, false)?;
            if let Err(code) = sdk::configure_data(&mut self.data) {
                EVENTS.unexpected(code);
                return Err(code);
            }
            self.wait_data(deadline_ms)?;
            self.data_configured = true;
        }
        if !self.data_configured || EVENTS.advertising() != Some(false) {
            return Err(ADAPTER_FAILURE);
        }
        self.run(Kind::Address, false, Some(deadline_ms), || {
            sdk::set_address(phase.address())
        })?;
        self.address_set_completed = true;
        self.prestart = sdk::read_address();
        self.checked_stop = false;
        let operation = match phase {
            Phase::First => Operation::StartFirst,
            Phase::Second => Operation::StartSecond,
        };
        self.run(Kind::Start, false, Some(deadline_ms), || {
            sdk::start(operation)
        })?;
        self.state = State::Active;
        let s = self.snapshot()?;
        if !s.used_readback.matches(phase) {
            return Err(ADAPTER_FAILURE);
        }
        Ok(s)
    }
    fn stop(&mut self, nonce: [u8; 8], automatic: bool) -> Result<Snapshot, i32> {
        if self.nonce.is_some_and(|n| n != nonce) {
            return Err(ADAPTER_FAILURE);
        }
        if let Some(code) = self.constructor_error {
            return Err(code);
        }
        if self.phase.is_none() {
            return self.snapshot();
        }
        self.auto_stopped |= automatic;
        if self.driver.is_some()
            && (!self.checked_stop || EVENTS.advertising() != Some(false) || EVENTS.pending())
        {
            self.run(Kind::Stop, true, None, sdk::stop)?;
            if EVENTS.advertising() != Some(false) {
                return Err(ADAPTER_FAILURE);
            }
            self.checked_stop = true;
        }
        self.state = State::Stopped;
        self.snapshot()
    }
}
impl BleRadio {
    fn wait_data(&self, phase_deadline: u64) -> Result<(), i32> {
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            let result = EVENTS.take(Kind::Data);
            if Instant::now() >= until || self.now() >= phase_deadline {
                EVENTS.unexpected(ESP_ERR_TIMEOUT);
                return Err(ESP_ERR_TIMEOUT);
            }
            if let Some(code) = result {
                if code != 0 {
                    return Err(code);
                }
                return EVENTS.fault().map_or(Ok(()), Err);
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
}
