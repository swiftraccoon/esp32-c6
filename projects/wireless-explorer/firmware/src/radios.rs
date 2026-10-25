//! SDK adapters. Only main starts/stops radios. Wi-Fi/BLE callbacks run in SDK
//! tasks; 802.15.4 callbacks run in an ISR or synchronously during SDK stop under
//! its critical section. They only copy to fixed slots and
//! update atomics. No formatting, heap allocation, serial I/O, or waits there.

use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::modem::{Modem, ThreadModem};
use esp_idf_svc::bt::ble::gap::{BleGapEvent, EspBleGap};
use esp_idf_svc::bt::{Ble, BtDriver, BtStatus};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{Configuration, WifiDriver};
use esp_idf_sys::*;
use wireless_protocol::capture::{
    CHUNK_BYTES, Details, Event, Packet, Radio, Record, SNAPLEN, hex,
};

use crate::Serial;
use crate::buffer::Buffer;

#[derive(Clone, Copy)]
struct Captured {
    meta: Packet,
    bytes: [u8; SNAPLEN],
}

const EMPTY: Captured = Captured {
    meta: Packet {
        seq: 0,
        at_us: 0,
        channel: 0,
        rssi: 0,
        original_len: 0,
        captured_len: 0,
        details: Details::Ieee802154 {
            lqi: 0,
            sdk_time: 0,
        },
    },
    bytes: [0; SNAPLEN],
};
static PACKETS: Buffer<Captured, 8> = Buffer::new(EMPTY);
static ACTIVE: AtomicU32 = AtomicU32::new(0);
static IN_CALLBACK: AtomicU32 = AtomicU32::new(0);
static SEEN: AtomicU32 = AtomicU32::new(0);
static DROPPED: AtomicU32 = AtomicU32::new(0);
static INVALID: AtomicU32 = AtomicU32::new(0);
static CALLBACK_ERROR: AtomicI32 = AtomicI32::new(0);
static BLE_CONFIGURED: AtomicI32 = AtomicI32::new(-1);
static BLE_STARTED: AtomicI32 = AtomicI32::new(-1);
static BLE_STOPPED: AtomicI32 = AtomicI32::new(-1);

fn key(radio: Radio) -> u32 {
    match radio {
        Radio::Wifi => 1,
        Radio::Ble => 2,
        Radio::Ieee802154 => 3,
    }
}

#[inline(always)]
pub(crate) fn now_us() -> u64 {
    // SAFETY: esp_timer is initialized before app_main and this read is valid
    // from tasks and ISR contexts. Its signed uptime is nonnegative here.
    unsafe { esp_timer_get_time() as u64 }
}

#[inline(always)]
fn report<'a>(radio: Radio, build: impl FnOnce(u32) -> Option<(Packet, &'a [u8])>) {
    // Only used by FFI callbacks below, which limit the slice lifetime to this
    // immediate call. No references are stored; bytes are copied before return.
    IN_CALLBACK.fetch_add(1, Ordering::SeqCst);
    if ACTIVE.load(Ordering::SeqCst) == key(radio) {
        let seq = SEEN.fetch_add(1, Ordering::Relaxed);
        if let Some((mut meta, bytes)) = build(seq) {
            meta.captured_len = bytes.len().min(SNAPLEN) as u16;
            if !PACKETS.push_with(|slot| {
                slot.meta = meta;
                slot.bytes[..usize::from(meta.captured_len)]
                    .copy_from_slice(&bytes[..usize::from(meta.captured_len)]);
            }) {
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            INVALID.fetch_add(1, Ordering::Relaxed);
        }
    }
    IN_CALLBACK.fetch_sub(1, Ordering::SeqCst);
}

unsafe extern "C" fn wifi_rx(buf: *mut core::ffi::c_void, kind: wifi_promiscuous_pkt_type_t) {
    report(Radio::Wifi, |seq| {
        if buf.is_null() {
            return None;
        }
        // SAFETY: SDK promises an aligned wifi_pkt_rx_ctrl_t at the start of
        // every callback buffer, valid only during this callback.
        let ctrl = unsafe { &*buf.cast::<wifi_pkt_rx_ctrl_t>() };
        let len = ctrl.sig_len() as usize;
        let bytes = if kind == wifi_promiscuous_pkt_type_t_WIFI_PKT_MISC {
            &[][..]
        } else {
            // SAFETY: non-MISC buffers contain sig_len payload bytes after the
            // generated metadata layout. Never read payload for metadata-only MISC.
            unsafe {
                std::slice::from_raw_parts(
                    buf.cast::<u8>()
                        .add(std::mem::size_of::<wifi_pkt_rx_ctrl_t>()),
                    len,
                )
            }
        };
        Some((
            Packet {
                seq,
                at_us: now_us(),
                channel: ctrl.channel() as u8,
                rssi: ctrl.rssi() as i8,
                original_len: len as u16,
                captured_len: 0,
                details: Details::Wifi {
                    kind: kind as u8,
                    rx_state: ctrl.rx_state() as u8,
                    sdk_time: ctrl.timestamp(),
                    phy: ctrl.cur_bb_format() as u8,
                    rate: ctrl.rate() as u8,
                },
            },
            bytes,
        ))
    });
}

// SAFETY: the SDK calls this ABI with valid generated-layout pointers. The
// callback entry is in IRAM, and its copy/atomic path is bounded. This firmware
// performs no flash writes during capture; cache-off execution is not qualified.
#[unsafe(link_section = ".iram1.text")]
unsafe extern "C" fn ieee_rx(frame: *mut u8, info: *mut esp_ieee802154_frame_info_t) {
    report(Radio::Ieee802154, |seq| {
        if frame.is_null() || info.is_null() {
            return None;
        }
        // SAFETY: the SDK owns both objects until handle_done below. Length
        // includes two FCS positions replaced by RSSI/LQI; omit those positions.
        let len = unsafe { *frame } as usize;
        if !(2..=127).contains(&len) {
            return None;
        }
        let info = unsafe { &*info };
        let bytes = unsafe { std::slice::from_raw_parts(frame.add(1), len - 2) };
        Some((
            Packet {
                seq,
                at_us: now_us(),
                channel: info.channel,
                rssi: info.rssi,
                original_len: (len - 2) as u16,
                captured_len: 0,
                details: Details::Ieee802154 {
                    lqi: info.lqi,
                    sdk_time: info.timestamp,
                },
            },
            bytes,
        ))
    });
    if !frame.is_null() {
        // SAFETY: return exactly the SDK-provided buffer once, even on drop or
        // inactive capture. All copying finished; no references survive.
        let status = unsafe { esp_ieee802154_receive_handle_done(frame) };
        if status != 0 {
            CALLBACK_ERROR.store(status, Ordering::Relaxed);
        }
    }
}

fn ble_event(event: BleGapEvent<'_>) {
    match event {
        BleGapEvent::ExtendedAdvertisingScanParametersConfigured(status) => {
            BLE_CONFIGURED.store(status as i32, Ordering::Release)
        }
        BleGapEvent::ExtendedAdvertisingScanStarted(status) => {
            BLE_STARTED.store(status as i32, Ordering::Release)
        }
        BleGapEvent::ExtendedAdvertisingScanStopped(status) => {
            BLE_STOPPED.store(status as i32, Ordering::Release)
        }
        BleGapEvent::Other {
            raw_event,
            raw_data,
        } if raw_event == esp_gap_ble_cb_event_t_ESP_GAP_BLE_EXT_ADV_REPORT_EVT => {
            // SAFETY: event discriminator selects this union member; the safe
            // service owns the callback borrow until this function returns.
            let adv = unsafe { &raw_data.0.ext_adv_report.params };
            // Unlike raw FFI payloads, this borrowed array cannot escape as
            // 'static. Handle it directly with the same bounded copy protocol.
            IN_CALLBACK.fetch_add(1, Ordering::SeqCst);
            if ACTIVE.load(Ordering::SeqCst) == key(Radio::Ble) {
                let seq = SEEN.fetch_add(1, Ordering::Relaxed);
                if let Some(bytes) = adv.adv_data.get(..usize::from(adv.adv_data_len)) {
                    let meta = Packet {
                        seq,
                        at_us: now_us(),
                        channel: 0,
                        rssi: adv.rssi,
                        original_len: bytes.len() as u16,
                        captured_len: bytes.len() as u16,
                        details: Details::Ble {
                            address: adv.addr,
                            address_type: adv.addr_type,
                            event_type: adv.event_type as u16,
                            primary_phy: adv.primary_phy,
                            secondary_phy: adv.secondly_phy,
                            sid: adv.sid,
                            data_status: adv.data_status,
                            tx_power: adv.tx_power as i8,
                            periodic_interval: adv.per_adv_interval,
                            direct_address: adv.dir_addr,
                            direct_address_type: adv.dir_addr_type,
                        },
                    };
                    if !PACKETS.push_with(|slot| {
                        slot.meta = meta;
                        slot.bytes[..bytes.len()].copy_from_slice(bytes);
                    }) {
                        DROPPED.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    INVALID.fetch_add(1, Ordering::Relaxed);
                }
            }
            IN_CALLBACK.fetch_sub(1, Ordering::SeqCst);
        }
        _ => {}
    }
}

// Static storage satisfies the SDK's documented lifetime through async config
// completion, including a timeout. Passive on both PHYs; duplicate filtering off.
static BLE_PARAMS: esp_ble_ext_scan_params_t = esp_ble_ext_scan_params_t {
    own_addr_type: esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC,
    filter_policy: esp_ble_scan_filter_t_BLE_SCAN_FILTER_ALLOW_ALL,
    scan_duplicate: esp_ble_scan_duplicate_t_BLE_SCAN_DUPLICATE_DISABLE,
    cfg_mask: 3,
    uncoded_cfg: esp_ble_ext_scan_cfg_t {
        scan_type: esp_ble_scan_type_t_BLE_SCAN_TYPE_PASSIVE,
        scan_interval: 160,
        scan_window: 160,
    },
    coded_cfg: esp_ble_ext_scan_cfg_t {
        scan_type: esp_ble_scan_type_t_BLE_SCAN_TYPE_PASSIVE,
        scan_interval: 160,
        scan_window: 160,
    },
};

fn status(code: i32) -> Result<(), EspError> {
    EspError::convert(code)
}

fn await_ble(value: &AtomicI32) -> Result<(), EspError> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let code = value.load(Ordering::Acquire);
        if code == BtStatus::Success as i32 {
            return Ok(());
        }
        if code != -1 {
            return status(code);
        }
        if Instant::now() >= deadline {
            return Err(EspError::from_infallible::<ESP_ERR_TIMEOUT>());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub(crate) struct Radios {
    pub(crate) wifi: Result<WifiDriver<'static>, EspError>,
    ble: Result<EspBleGap<'static, Ble, BtDriver<'static, Ble>>, EspError>,
    _thread: ThreadModem<'static>,
    ieee_enabled: bool,
    ieee_uncertain: bool,
}

impl Radios {
    pub(crate) fn new(modem: Modem<'static>, event_loop: EspSystemEventLoop) -> Self {
        let (wifi_modem, thread_modem, ble_modem) = modem.split();
        let wifi = WifiDriver::new(wifi_modem, event_loop, None);
        let ble = BtDriver::<Ble>::new(ble_modem, None)
            .and_then(EspBleGap::new)
            .and_then(|gap| {
                gap.subscribe(ble_event)?;
                Ok(gap)
            });
        Self {
            wifi,
            ble,
            _thread: thread_modem,
            ieee_enabled: false,
            ieee_uncertain: false,
        }
    }

    fn start(&mut self, radio: Radio, channel: u8) -> Result<(), EspError> {
        match radio {
            Radio::Wifi => {
                let wifi = self.wifi.as_mut().map_err(|error| *error)?;
                wifi.set_configuration(&Configuration::None)?;
                wifi.start()?;
                // SAFETY: main exclusively owns the initialized Wi-Fi driver.
                // Registration uses a static callback, and filters are copied.
                unsafe {
                    status(esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE))?;
                    status(esp_wifi_set_channel(
                        channel,
                        wifi_second_chan_t_WIFI_SECOND_CHAN_NONE,
                    ))?;
                    status(esp_wifi_set_promiscuous_filter(
                        &wifi_promiscuous_filter_t {
                            filter_mask: WIFI_PROMIS_FILTER_MASK_ALL,
                        },
                    ))?;
                    status(esp_wifi_set_promiscuous_ctrl_filter(
                        &wifi_promiscuous_filter_t {
                            filter_mask: WIFI_PROMIS_CTRL_FILTER_MASK_ALL,
                        },
                    ))?;
                    status(esp_wifi_set_promiscuous_rx_cb(Some(wifi_rx)))?;
                    status(esp_wifi_set_promiscuous(true))
                }
            }
            Radio::Ble => {
                self.ble.as_ref().map_err(|error| *error)?;
                BLE_CONFIGURED.store(-1, Ordering::Release);
                BLE_STARTED.store(-1, Ordering::Release);
                // SAFETY: static params outlive asynchronous completion. This
                // task owns scan setup; no GATT/advertising/connection is started.
                unsafe {
                    status(esp_ble_gap_set_ext_scan_params(&BLE_PARAMS))?;
                }
                await_ble(&BLE_CONFIGURED)?;
                unsafe {
                    status(esp_ble_gap_start_ext_scan(0, 0))?;
                }
                await_ble(&BLE_STARTED)
            }
            Radio::Ieee802154 => {
                // SAFETY: owned ThreadModem excludes another upper-layer
                // driver. Register static callbacks only while disabled.
                unsafe {
                    status(esp_ieee802154_event_callback_list_register(
                        esp_ieee802154_event_cb_list_t {
                            rx_done_cb: Some(ieee_rx),
                            ..Default::default()
                        },
                    ))?;
                    // IDF can enable RF resources before MAC initialization
                    // fails. Such a failure has no qualified partial teardown.
                    // Stop must then terminate command handling, never continue.
                    self.ieee_uncertain = true;
                    status(esp_ieee802154_enable())?;
                    self.ieee_uncertain = false;
                    self.ieee_enabled = true;
                    // Promiscuous mode also turns off automatic ACK transmit in
                    // pinned IDF esp_ieee802154_set_promiscuous implementation.
                    status(esp_ieee802154_set_promiscuous(true))?;
                    status(esp_ieee802154_set_channel(channel))?;
                    status(esp_ieee802154_set_rx_when_idle(true))?;
                    status(esp_ieee802154_receive())
                }
            }
        }
    }

    fn stop(&mut self, radio: Radio) -> Result<(), EspError> {
        if self.ieee_uncertain {
            return Err(EspError::from_infallible::<ESP_ERR_INVALID_STATE>());
        }
        match radio {
            Radio::Wifi => {
                // SAFETY: matching main-task shutdown of the owned driver.
                let disabled = unsafe { status(esp_wifi_set_promiscuous(false)) };
                let stopped = self.wifi.as_mut().map_err(|e| *e)?.stop();
                disabled.and(stopped)
            }
            Radio::Ble => {
                BLE_STOPPED.store(-1, Ordering::Release);
                // SAFETY: matching main-task shutdown of the owned GAP scanner.
                unsafe {
                    status(esp_ble_gap_stop_ext_scan())?;
                }
                await_ble(&BLE_STOPPED)
            }
            Radio::Ieee802154 if self.ieee_enabled => {
                // SAFETY: main owns this initialized radio and callbacks retain
                // no SDK buffers. Explicitly stop continuous RX before removing
                // its ISR/clocks: IDF disable alone does not issue CMD_STOP.
                let quiesced = unsafe {
                    status(esp_ieee802154_set_rx_when_idle(false))
                        .and_then(|()| status(esp_ieee802154_sleep()))
                };
                // Attempt teardown even if quiescing failed, preserving either
                // error so command handling cannot reuse uncertain radio state.
                let stopped = unsafe { status(esp_ieee802154_disable()) };
                if stopped.is_ok() {
                    self.ieee_enabled = false;
                }
                quiesced.and(stopped)
            }
            Radio::Ieee802154 => Ok(()),
        }
    }

    pub(crate) fn observe(
        &mut self,
        port: &mut Serial<'_>,
        id: u32,
        radio: Radio,
        channel: u8,
        millis: u32,
    ) -> Result<(), EspError> {
        if !radio.valid_window(channel, millis) {
            return Err(EspError::from_infallible::<ESP_ERR_INVALID_ARG>());
        }
        SEEN.store(0, Ordering::Relaxed);
        DROPPED.store(0, Ordering::Relaxed);
        INVALID.store(0, Ordering::Relaxed);
        CALLBACK_ERROR.store(0, Ordering::Relaxed);
        while PACKETS.pop_with(|_| ()).is_some() {}
        port.stream(&Record::new(
            id,
            Event::Start {
                radio,
                channel,
                millis,
                at_us: now_us(),
                snaplen: SNAPLEN as u16,
            },
        ))?;
        ACTIVE.store(key(radio), Ordering::SeqCst);
        let mut emitted = 0;
        let mut truncated = 0;
        let run = (|| {
            self.start(radio, channel)?;
            port.stream(&Record::new(id, Event::Listening { at_us: now_us() }))?;
            let deadline = Instant::now() + Duration::from_millis(u64::from(millis));
            while Instant::now() < deadline {
                if !drain_one(port, id, &mut emitted, &mut truncated)? {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            Ok::<_, EspError>(())
        })();
        let stopped = self.stop(radio);
        ACTIVE.store(0, Ordering::SeqCst);
        while IN_CALLBACK.load(Ordering::SeqCst) != 0 {
            thread::yield_now();
        }
        let stopped_us = now_us();
        let mut outcome = run.and(stopped);
        while let Some(result) =
            PACKETS.pop_with(|packet| emit(port, id, packet, &mut emitted, &mut truncated))
        {
            if let Err(error) = result {
                outcome = Err(error);
                break;
            }
        }
        let callback_error = CALLBACK_ERROR.load(Ordering::Relaxed);
        let error = outcome.err().map_or(callback_error, |e| e.code());
        port.stream(&Record::new(
            id,
            Event::End {
                at_us: now_us(),
                stopped_us,
                seen: SEEN.load(Ordering::Relaxed),
                emitted,
                dropped: DROPPED.load(Ordering::Relaxed),
                invalid: INVALID.load(Ordering::Relaxed),
                truncated,
                error,
            },
        ))?;
        // A stop error leaves radio ownership uncertain: fail closed by ending
        // main, rather than starting another mode on a potentially active radio.
        stopped
    }
}

fn drain_one(
    port: &mut Serial<'_>,
    id: u32,
    emitted: &mut u32,
    truncated: &mut u32,
) -> Result<bool, EspError> {
    PACKETS
        .pop_with(|packet| emit(port, id, packet, emitted, truncated))
        .transpose()
        .map(|item| item.is_some())
}

fn emit(
    port: &mut Serial<'_>,
    id: u32,
    packet: &Captured,
    emitted: &mut u32,
    truncated: &mut u32,
) -> Result<(), EspError> {
    port.stream(&Record::new(id, Event::Packet(packet.meta)))?;
    for (index, chunk) in packet.bytes[..usize::from(packet.meta.captured_len)]
        .chunks(CHUNK_BYTES)
        .enumerate()
    {
        let data = hex(chunk).map_err(|_| EspError::from_infallible::<ESP_ERR_INVALID_SIZE>())?;
        port.stream(&Record::new(
            id,
            Event::Chunk {
                seq: packet.meta.seq,
                offset: (index * CHUNK_BYTES) as u16,
                data,
            },
        ))?;
    }
    *emitted += 1;
    if usize::from(packet.meta.original_len) > SNAPLEN {
        *truncated += 1;
    }
    Ok(())
}
