use core::cell::Cell;
use core::fmt;
use core::mem::ManuallyDrop;
use core::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use c6_anonymization_identity::Identity;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, Configuration, WifiDeviceId, WifiDriver,
};
use esp_idf_sys as sys;

use crate::plan::Plan;
use crate::sequence::Sequence;
use crate::{Epoch, Error};

static ATTEMPTED: AtomicBool = AtomicBool::new(false);
const STATE_WAIT: Duration = Duration::from_secs(1);

fn sdk(operation: &'static str, error: sys::EspError) -> Error {
    Error::Sdk {
        operation,
        code: error.code(),
    }
}

fn raw(operation: &'static str, code: i32) -> Result<(), Error> {
    if code == sys::ESP_OK {
        Ok(())
    } else {
        Err(Error::Sdk { operation, code })
    }
}

/// One boot's explicitly controlled SoftAP adapter.
///
/// Ownership and the startup precondition exclude concurrent MAC/ADC/radio
/// users. The handle is not Sync; SDK failures, including diagnostic readback
/// failures, permanently prohibit Starts. Stop remains available for cleanup.
/// SDK calls can block; the example owns an independent cleanup deadline.
///
/// Dropping this handle deliberately retains initialized driver/event-loop
/// resources until chip reset. It does not stop RF. Call [`Self::stop`] explicitly
/// and retain the handle on failures; no destructor silently restores identity.
pub struct SoftAp {
    plan: Plan,
    sequence: Sequence,
    driver: ManuallyDrop<WifiDriver<'static>>,
    _sysloop: ManuallyDrop<EspSystemEventLoop>,
    failure: Cell<Option<Error>>,
    verified_active: Cell<bool>,
    start_requested: bool,
    checked_stopped: bool,
}

impl fmt::Debug for SoftAp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftAp")
            .field("epoch", &self.sequence.current)
            .field("verified_active", &self.verified_active.get())
            .field("failure", &self.failure.get())
            .finish_non_exhaustive()
    }
}

impl SoftAp {
    /// Prepare two identities and an inactive AP without starting a radio.
    ///
    /// This is attempted at most once per boot, including failed attempts. It
    /// opens a single SAR entropy window, prepares all identities, disables that
    /// source, sets global BASE/AP tables before their first interface reads,
    /// then initializes Wi-Fi with no NVS handle and explicitly selects RAM.
    ///
    /// # Safety
    /// Call once at normal application startup before *any* ADC, Wi-Fi,
    /// Bluetooth, IEEE 802.15.4, network interface or competing MAC API user is
    /// initialized. No task/callback/ISR may concurrently use those resources.
    /// The unsplit Modem must be the uniquely owned HAL peripheral. No later raw
    /// API user may change MAC tables, start radios or mutate this Wi-Fi driver.
    /// The application must keep other radios uninitialized and PHY/NVS storage
    /// disabled in its actual configuration. SAR disable is not restoration of
    /// prior ADC state; this API requires that no prior state exists.
    ///
    /// # Errors
    /// Rejects repeat attempts, known sentinel/colliding samples, SDK failures
    /// or mismatched table/driver state. Resources after partial initialization
    /// remain retained for chip-reset recovery; retrying initialization is denied.
    pub unsafe fn initialize_at_startup(modem: Modem<'static>) -> Result<Self, Error> {
        if ATTEMPTED.swap(true, Ordering::AcqRel) {
            return Err(Error::AlreadyAttempted);
        }
        let mut sample = [0; 34];
        // SAFETY: the caller guarantees exclusive, previously unused ADC/RF
        // resources. All34 stack bytes are writable for this synchronous call.
        // No fallible Rust operation intervenes before disabling SAR entropy.
        unsafe {
            sys::bootloader_random_enable();
            sys::esp_fill_random(sample.as_mut_ptr().cast(), sample.len());
            sys::bootloader_random_disable();
        }
        let plan = Plan::from_sample(sample)?;
        // SAFETY: generated C6 bindings, exact6-byte BASE/AP inputs, synchronous
        // no-retained-pointer APIs, and the caller's sole MAC-table ownership.
        unsafe {
            raw("set base", sys::esp_base_mac_addr_set(plan.base.as_ptr()))?;
            raw(
                "set AP table",
                sys::esp_iface_mac_addr_set(
                    plan.identities[0].mac().as_ptr(),
                    sys::esp_mac_type_t_ESP_MAC_WIFI_SOFTAP,
                ),
            )?;
        }
        for (kind, expected) in [
            (sys::esp_mac_type_t_ESP_MAC_BASE, plan.base),
            (
                sys::esp_mac_type_t_ESP_MAC_WIFI_SOFTAP,
                plan.identities[0].mac(),
            ),
        ] {
            // SAFETY: this query accepts a generated enum and takes no pointer.
            if unsafe { sys::esp_mac_addr_len_get(kind) } != 6 {
                return Err(Error::Readback);
            }
            let mut observed = [0; 6];
            // SAFETY: the checked SDK MAC type has exactly6 output bytes, this
            // stack buffer is live, and the table has already been overridden.
            raw("read prepared table", unsafe {
                sys::esp_read_mac(observed.as_mut_ptr(), kind)
            })?;
            if observed != expected {
                return Err(Error::Readback);
            }
        }
        let sysloop = ManuallyDrop::new(
            EspSystemEventLoop::take().map_err(|error| sdk("take event loop", error))?,
        );
        // The wrapper initializes before subscribing. Retain the event loop even
        // if construction fails after partial SDK initialization; never retry.
        let driver = WifiDriver::new(modem, (*sysloop).clone(), None)
            .map_err(|error| sdk("initialize Wi-Fi", error))?;
        let mut ap = Self {
            plan,
            sequence: Sequence::default(),
            driver: ManuallyDrop::new(driver),
            _sysloop: sysloop,
            failure: Cell::new(None),
            verified_active: Cell::new(false),
            start_requested: false,
            checked_stopped: false,
        };
        // SAFETY: Wi-Fi is initialized but has not started, no NVS handle was
        // provided, and the fixed generated enum selects volatile storage.
        raw("select RAM storage", unsafe {
            sys::esp_wifi_set_storage(sys::wifi_storage_t_WIFI_STORAGE_RAM)
        })?;
        ap.configure(Epoch::First)?;
        Ok(ap)
    }

    /// Borrow an explicitly observable identity; never publish it as a credential.
    pub fn identity(&self, epoch: Epoch) -> &Identity {
        &self.plan.identities[epoch.index()]
    }

    /// Return the first retained failure, even after later successful cleanup.
    pub fn failure(&self) -> Option<Error> {
        self.failure.get()
    }

    fn fail(&self, error: Error) -> Error {
        if self.failure.get().is_none() {
            self.failure.set(Some(error));
        }
        self.verified_active.set(false);
        error
    }

    fn expected(&self) -> [u8; 6] {
        self.identity(self.sequence.current.unwrap_or(Epoch::First))
            .mac()
    }

    fn running(&self) -> Result<bool, Error> {
        self.driver
            .is_ap_started()
            .map_err(|error| sdk("observe AP state", error))
    }

    fn wait_running(&self, expected: bool) -> Result<(), Error> {
        let deadline = Instant::now() + STATE_WAIT;
        loop {
            let observed = self.running()?;
            if Instant::now() >= deadline {
                return Err(Error::Timeout);
            }
            if observed == expected {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn configure(&mut self, epoch: Epoch) -> Result<(), Error> {
        if self.running()? {
            return Err(Error::Order);
        }
        let mut config = AccessPointConfiguration {
            channel: 1,
            secondary_channel: None,
            auth_method: AuthMethod::None,
            max_connections: 1,
            ssid_hidden: false,
            ..Default::default()
        };
        config.ssid.clear();
        config
            .ssid
            .push_str(self.identity(epoch).ssid())
            .map_err(|_| Error::Readback)?;
        self.driver
            .set_configuration(&Configuration::AccessPoint(config))
            .map_err(|error| sdk("configure AP", error))?;
        match self
            .driver
            .get_configuration()
            .map_err(|error| sdk("read AP configuration", error))?
        {
            Configuration::AccessPoint(config)
                if config.ssid.as_str() == self.identity(epoch).ssid()
                    && config.channel == 1
                    && config.auth_method == AuthMethod::None => {}
            _ => return Err(Error::Readback),
        }
        if self
            .driver
            .is_sta_enabled()
            .map_err(|error| sdk("observe STA mode", error))?
            || !self
                .driver
                .is_ap_enabled()
                .map_err(|error| sdk("observe AP mode", error))?
            || self.running()?
        {
            return Err(Error::Readback);
        }
        let mac = self.identity(epoch).mac();
        self.driver
            .set_mac(WifiDeviceId::Ap, mac)
            .map_err(|error| sdk("set AP MAC", error))?;
        if self
            .driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| sdk("read AP MAC", error))?
            != self.identity(epoch).mac()
        {
            return Err(Error::Readback);
        }
        Ok(())
    }

    /// Consume the selected Start attempt and check SDK state/address afterward.
    ///
    /// First can run once; Second requires a successful explicit Stop after
    /// First. Any error sticks and prohibits later Starts. The caller must
    /// prearm its cleanup deadline and call Stop after every attempt, including
    /// an error. One-second state waits do not bound blocking SDK operations.
    ///
    /// # Errors
    /// Returns [`Error::Failed`] after an earlier failure, or a new order, SDK,
    /// timeout or readback error. This never establishes transmitted behavior.
    pub fn start(&mut self, epoch: Epoch) -> Result<(), Error> {
        if self.failure.get().is_some() {
            return Err(Error::Failed);
        }
        let result = (|| {
            self.sequence.begin(epoch)?;
            self.configure(epoch)?;
            self.checked_stopped = false;
            self.start_requested = true;
            self.driver
                .start()
                .map_err(|error| sdk("start AP", error))?;
            self.wait_running(true)?;
            self.readback_mac()?;
            self.verified_active.set(true);
            Ok(())
        })();
        result.map_err(|error| self.fail(error))
    }

    /// Explicitly Stop, check inactivity and retain the selected masked identity.
    ///
    /// Retry is allowed after failures. Successful cleanup never clears the
    /// first failure or restores a factory address. The initialized driver and
    /// RAM configuration remain owned until reset. SDK state is not RF silence.
    ///
    /// # Errors
    /// Returns the current stop/state/readback failure while retaining the first
    /// failure separately in [`Self::failure`].
    pub fn stop(&mut self) -> Result<(), Error> {
        let result = (|| {
            self.verified_active.set(false);
            if (self.start_requested && !self.checked_stopped) || self.running()? {
                self.checked_stopped = false;
                self.driver.stop().map_err(|error| sdk("stop AP", error))?;
                self.wait_running(false)?;
                self.checked_stopped = true;
            }
            if self.running()? {
                return Err(Error::Readback);
            }
            self.readback_mac()?;
            self.sequence.stopped();
            Ok(())
        })();
        result.map_err(|error| self.fail(error))
    }

    /// Observe started SDK state and the exact selected address after a Start.
    ///
    /// False also covers unverified or failed state; it never attests RF silence.
    /// Observation failures latch [`Self::failure`] and block later Starts.
    pub fn active(&self) -> bool {
        if !self.verified_active.get() {
            return false;
        }
        let result = self.running().and_then(|running| {
            if !running {
                return Err(Error::Readback);
            }
            self.readback_mac().map(|_| ())
        });
        match result {
            Ok(()) => true,
            Err(error) => {
                self.fail(error);
                false
            }
        }
    }

    /// Read and validate the current SDK AP MAC without proving its transmission.
    ///
    /// # Errors
    /// A getter failure or mismatch is retained and prohibits later Starts.
    pub fn readback_mac(&self) -> Result<[u8; 6], Error> {
        let result = self
            .driver
            .get_mac(WifiDeviceId::Ap)
            .map_err(|error| sdk("read AP MAC", error))
            .and_then(|mac| {
                if mac == self.expected() {
                    Ok(mac)
                } else {
                    Err(Error::Readback)
                }
            });
        result.map_err(|error| self.fail(error))
    }
}
