//! Synchronous, selected-service IOKit reads without opening or seizing devices.
//!
//! Generated SDK declarations stay private. No callback or pointer survives a
//! synchronous call, and all native references are owned on the calling thread.

#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, c_void};
use std::ptr;

use crate::{Domain, DomainRecord, FORMAT_VERSION, Record, Request, Transfer, observe_domain};

#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/native-bindings.rs"));
}

// These local negative statuses are disjoint from successful IOReturn zero.
const INVALID_NATIVE_METADATA: i32 = -1;
const INVALID_NATIVE_INTERFACE: i32 = -2;
const REQUEST_NOT_ALLOWED: i32 = -3;
const REGISTRY_LIMIT: i32 = -4;
const MAX_SERVICES: usize = 1024;
const TIMEOUT_MS: u32 = 1000;

const USER_CLIENT_UUID: [u8; 16] = [
    0x9d, 0xc7, 0xb7, 0x80, 0x9e, 0xc0, 0x11, 0xd4, 0xa5, 0x4f, 0x00, 0x0a, 0x27, 0x05, 0x28, 0x61,
];
const PLUGIN_UUID: [u8; 16] = [
    0xc2, 0x44, 0xe8, 0x58, 0x10, 0x9c, 0x11, 0xd4, 0x91, 0xd4, 0x00, 0x50, 0xe4, 0xc6, 0x42, 0x6f,
];
const INTERFACE_UUID: [u8; 16] = [
    0x56, 0xad, 0x08, 0x9d, 0x87, 0x8d, 0x4b, 0xea, 0xa1, 0xf5, 0x2c, 0x8d, 0xc4, 0x3e, 0x8a, 0x98,
];

fn first_error(first: i32, second: i32) -> i32 {
    if first == 0 { second } else { first }
}

struct IoObject(u32);

impl IoObject {
    fn release(&mut self) -> i32 {
        if self.0 == 0 {
            return 0;
        }
        let object = std::mem::replace(&mut self.0, 0);
        // SAFETY: This guard owns one reference returned by matching/iteration;
        // taking the value before release prevents a second native release.
        unsafe { ffi::IOObjectRelease(object) }
    }
}

impl Drop for IoObject {
    fn drop(&mut self) {
        // Explicit release records statuses on normal paths. This fallback
        // releases ownership during unwinding without panicking.
        let _ = self.release();
    }
}

struct CfObject(ffi::CFTypeRef);

impl Drop for CfObject {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Each non-null Create result contributes one owned CF
            // reference, released once. CFRelease has no fallible return.
            unsafe { ffi::CFRelease(self.0) };
        }
    }
}

fn property(service: u32, name: &CStr) -> Result<u16, i32> {
    // SAFETY: Name is a live static NUL-terminated UTF-8 string. Null allocator
    // selects the CF default, and the returned Create reference is owned.
    let key = unsafe {
        ffi::CFStringCreateWithCString(ptr::null(), name.as_ptr(), ffi::kCFStringEncodingUTF8)
    };
    if key.is_null() {
        return Err(INVALID_NATIVE_METADATA);
    }
    let _key = CfObject(key.cast());
    // SAFETY: Service is an owned IORegistry reference. The key stays live for
    // this synchronous lookup; Create returns a separately owned CF object.
    let value = unsafe { ffi::IORegistryEntryCreateCFProperty(service, key, ptr::null(), 0) };
    if value.is_null() {
        return Err(INVALID_NATIVE_METADATA);
    }
    let _value = CfObject(value);
    // SAFETY: The value is a live CF object. Type checking precedes the cast
    // required by CFNumberGetValue and its writable i32 output contract.
    if unsafe { ffi::CFGetTypeID(value) != ffi::CFNumberGetTypeID() } {
        return Err(INVALID_NATIVE_METADATA);
    }
    let mut number = 0_i32;
    // SAFETY: Type checked above; output points to an aligned live i32, matching
    // kCFNumberSInt32Type. The CF call copies synchronously and retains no pointer.
    let valid = unsafe {
        ffi::CFNumberGetValue(
            value.cast(),
            ffi::kCFNumberSInt32Type.into(),
            (&mut number as *mut i32).cast(),
        )
    };
    if valid == 0 {
        return Err(INVALID_NATIVE_METADATA);
    }
    u16::try_from(number).map_err(|_| INVALID_NATIVE_METADATA)
}

struct Selection {
    domain: Domain,
    count: u32,
    service: Option<IoObject>,
}

fn selections() -> ([Selection; 2], i32, i32) {
    let mut selections = Domain::ALL.map(|domain| Selection {
        domain,
        count: 0,
        service: None,
    });
    // SAFETY: The C string is static. The matching dictionary's reference is
    // consumed by IOServiceGetMatchingServices, including its failure path.
    let matching = unsafe { ffi::IOServiceMatching(c"IOUSBHostDevice".as_ptr()) };
    if matching.is_null() {
        return (selections, INVALID_NATIVE_METADATA, 0);
    }
    let mut iterator = 0;
    // SAFETY: Matching is a newly owned dictionary; output is a live u32. The
    // immutable SDK global is the documented default Mach main port.
    let status = unsafe {
        ffi::IOServiceGetMatchingServices(ffi::kIOMainPortDefault, matching, &mut iterator)
    };
    if status != 0 {
        return (selections, status, 0);
    }
    let mut iterator = IoObject(iterator);
    let mut status = 0;
    let mut cleanup = 0;
    let mut seen = 0;
    loop {
        // SAFETY: Iterator is owned/live until explicitly released below. Each
        // nonzero result contributes a distinct owned registry reference.
        let next = unsafe { ffi::IOIteratorNext(iterator.0) };
        if next == 0 {
            break;
        }
        let mut service = IoObject(next);
        seen += 1;
        if seen > MAX_SERVICES {
            cleanup = first_error(cleanup, service.release());
            status = REGISTRY_LIMIT;
            break;
        }
        let ids = property(next, c"idVendor")
            .and_then(|vendor| property(next, c"idProduct").map(|product| (vendor, product)));
        let ids = match ids {
            Ok(ids) => ids,
            Err(error) => {
                cleanup = first_error(cleanup, service.release());
                status = error;
                break;
            }
        };
        if let Some(selection) = selections
            .iter_mut()
            .find(|entry| entry.domain.ids() == ids)
        {
            selection.count += 1;
            if selection.count == 1 {
                selection.service = Some(service);
            } else {
                cleanup = first_error(cleanup, service.release());
            }
        } else {
            cleanup = first_error(cleanup, service.release());
        }
    }
    // SAFETY: Checking a live iterator detects invalidated enumeration instead
    // of admitting a potentially incomplete match count as an absence.
    if unsafe { ffi::IOIteratorIsValid(iterator.0) } == 0 {
        status = first_error(status, INVALID_NATIVE_METADATA);
    }
    cleanup = first_error(cleanup, iterator.release());
    (selections, status, cleanup)
}

fn uuid(bytes: &[u8; 16]) -> ffi::CFUUIDRef {
    // SAFETY: Exactly sixteen copied scalar bytes and a null default allocator;
    // GetConstant returns a borrowed immutable reference, not an owned Create.
    unsafe {
        ffi::CFUUIDGetConstantUUIDWithBytes(
            ptr::null(),
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15],
        )
    }
}

struct Plugin(*mut *mut ffi::IOCFPlugInInterface);

impl Plugin {
    fn table(&self) -> Option<&ffi::IOCFPlugInInterface> {
        // SAFETY: A successful IOCreatePlugInInterfaceForService returned the
        // owned COM object; its immutable vtable stays valid while held.
        unsafe { self.0.as_ref()?.as_ref() }
    }

    fn release(&mut self) -> i32 {
        if self.0.is_null() {
            return 0;
        }
        let release = self.table().and_then(|table| table.Release);
        let object = std::mem::replace(&mut self.0, ptr::null_mut());
        if let Some(release) = release {
            // SAFETY: Object owns one COM reference. C's self parameter is the
            // interface double pointer, not the dereferenced vtable address.
            // Return is a remaining reference count, never an IOReturn error.
            unsafe { release(object.cast()) };
            0
        } else {
            INVALID_NATIVE_INTERFACE
        }
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

struct Device(*mut *mut ffi::IOUSBDeviceInterface942, Domain);

impl Device {
    fn table(&self) -> Option<&ffi::IOUSBDeviceInterface942> {
        // SAFETY: A successful QueryInterface returned an owned COM reference.
        // Its vtable remains valid until the reference is explicitly released.
        unsafe { self.0.as_ref()?.as_ref() }
    }

    fn release(&mut self) -> i32 {
        if self.0.is_null() {
            return 0;
        }
        let release = self.table().and_then(|table| table.Release);
        let object = std::mem::replace(&mut self.0, ptr::null_mut());
        if let Some(release) = release {
            // SAFETY: As for Plugin, release exactly one owned COM reference;
            // no USBDeviceClose, service Stop or device-open operation occurs.
            unsafe { release(object.cast()) };
            0
        } else {
            INVALID_NATIVE_INTERFACE
        }
    }

    fn transfer(&mut self, request: Request) -> Transfer {
        let method = self.table().and_then(|table| table.DeviceRequestTO);
        if !allowed(self.1, request) || method.is_none() {
            return Transfer {
                request,
                status: REQUEST_NOT_ALLOWED,
                returned_length: 0,
                bytes: Vec::new(),
            };
        }
        let mut bytes = vec![0; usize::from(request.length)];
        let mut native = ffi::IOUSBDevRequestTO {
            bmRequestType: request.request_type,
            bRequest: request.request,
            wValue: request.value,
            wIndex: request.index,
            wLength: request.length,
            pData: bytes.as_mut_ptr().cast::<c_void>(),
            wLenDone: 0,
            noDataTimeout: TIMEOUT_MS,
            completionTimeout: TIMEOUT_MS,
        };
        let status = match method {
            Some(method) => {
                // SAFETY: The live owned COM object's generated ABI takes its
                // double pointer as self. Native fields are host-endian. The
                // writable initialized buffer has exactly wLength bytes, remains
                // fixed for the synchronous call, and no callback is installed.
                // The allowlist admits only bounded IN reads; no device-open,
                // reset, set-configuration, interface or modem operation exists.
                unsafe { method(self.0.cast(), &mut native) }
            }
            None => REQUEST_NOT_ALLOWED,
        };
        // Retain the reported prefix even on a partial failed transfer; the
        // zero-initialized unused suffix is not an observed response. An
        // overreported length remains explicit and fails semantic validation.
        bytes.truncate(usize::try_from(native.wLenDone).unwrap_or(usize::MAX));
        Transfer {
            request,
            status,
            returned_length: native.wLenDone,
            bytes,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

fn acquire(service: u32, domain: Domain) -> Result<Device, (i32, i32)> {
    let user = uuid(&USER_CLIENT_UUID);
    let plugin_type = uuid(&PLUGIN_UUID);
    let interface_type = uuid(&INTERFACE_UUID);
    if user.is_null() || plugin_type.is_null() || interface_type.is_null() {
        return Err((INVALID_NATIVE_METADATA, 0));
    }
    let mut pointer = ptr::null_mut();
    let mut score = 0;
    // SAFETY: Service is a live selected IORegistry reference. UUIDs are valid
    // borrowed CF constants; output/score are writable locals. No USB open is
    // requested. Only successful creation establishes plugin ownership.
    let status = unsafe {
        ffi::IOCreatePlugInInterfaceForService(service, user, plugin_type, &mut pointer, &mut score)
    };
    if status != 0 {
        return Err((status, 0));
    }
    let mut plugin = Plugin(pointer);
    let query = plugin.table().and_then(|table| table.QueryInterface);
    let Some(query) = query else {
        return Err((INVALID_NATIVE_INTERFACE, plugin.release()));
    };
    let mut device = ptr::null_mut();
    // SAFETY: The QueryInterface method belongs to the live owned plugin.
    // CFUUIDGetUUIDBytes copies a valid borrowed UUID into the exact generated
    // REFIID value. Output is an aligned pointer local, cast only after success.
    let status = unsafe {
        query(
            plugin.0.cast(),
            ffi::CFUUIDGetUUIDBytes(interface_type),
            &mut device,
        )
    };
    // Plain COM Release is intentional: IODestroyPlugInInterface may stop the
    // associated IOService. A queried device reference retains its own lifetime.
    let cleanup = plugin.release();
    if status != 0 {
        return Err((status, cleanup));
    }
    let mut device = Device(device.cast(), domain);
    if cleanup != 0 {
        let cleanup = first_error(cleanup, device.release());
        return Err((0, cleanup));
    }
    if device
        .table()
        .and_then(|table| table.DeviceRequestTO)
        .is_none()
        || device.table().and_then(|table| table.Release).is_none()
    {
        return Err((INVALID_NATIVE_INTERFACE, device.release()));
    }
    Ok(device)
}

fn allowed(domain: Domain, request: Request) -> bool {
    if request.request_type == 0x80 && request.request == 6 {
        return (request.value == 0x0100 && request.index == 0 && request.length == 18)
            || (request.value == 0x0300
                && request.index == 0
                && (4..=254).contains(&request.length)
                && request.length.is_multiple_of(2))
            || (request.value >> 8 == 3
                && request.value & 0xff != 0
                && request.index != 0
                && (2..=254).contains(&request.length)
                && request.length.is_multiple_of(2));
    }
    domain == Domain::Cp2102nBridge
        && request.request_type == 0xc0
        && request.request == 0xff
        && request.index == 0
        && ((request.value == 0x370b && request.length == 1)
            || (request.value == 0x000e && request.length == 678))
}

/// Return the selected SDK/header/bindgen identities, with no personal paths.
pub fn native_provenance() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/native-inputs.json"))
}

/// Collect bounded private control-transfer records from the two fixed domains.
///
/// This macOS-only function reads cached registry IDs for selection, then uses
/// generated IOKit user-client ABI to issue the planner's IN requests. It never
/// calls device open/seize/close, configuration, reset, interface-claim or modem
/// APIs. The SDK allows these reads without opening the device. Calls and native
/// cleanup are synchronous and can block; each control request has 1 s timeouts.
/// Missing/ambiguous devices and setup/transfer/cleanup errors remain in the raw
/// record. Call [`crate::summarize`] to decide whether that record is admissible.
/// Serialization contains unique values and belongs in ignored private storage.
///
/// This example requires the documented board, both USB ports, SDK and access.
/// It is compiled, not executed, by host documentation tests.
///
/// ```no_run
/// let raw = usb_observer::observe();
/// let summary = usb_observer::summarize(&raw)?;
/// # Ok::<(), usb_observer::Error>(())
/// ```
pub fn observe() -> Record {
    let (selections, setup, cleanup) = selections();
    let domains = selections.map(|mut selection| {
        let mut record = DomainRecord {
            domain: selection.domain,
            matched_count: selection.count,
            setup_status: setup,
            cleanup_status: cleanup,
            transfers: Vec::new(),
        };
        if setup == 0
            && cleanup == 0
            && selection.count == 1
            && let Some(service) = selection.service.as_ref()
        {
            match acquire(service.0, selection.domain) {
                Ok(mut device) => {
                    record = observe_domain(selection.domain, selection.count, 0, |request| {
                        device.transfer(request)
                    });
                    record.cleanup_status = device.release();
                }
                Err((setup, cleanup)) => {
                    record.setup_status = setup;
                    record.cleanup_status = cleanup;
                }
            }
        }
        if let Some(service) = selection.service.as_mut() {
            record.cleanup_status = first_error(record.cleanup_status, service.release());
        }
        record
    });
    Record {
        version: FORMAT_VERSION,
        domains,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_descriptor_reads_bound_prefixes_and_declared_lengths() {
        let language = Request {
            request_type: 0x80,
            request: 6,
            value: 0x0300,
            index: 0,
            length: 4,
        };
        let serial = Request {
            value: 0x0303,
            index: 0x0409,
            length: 2,
            ..language
        };
        for domain in Domain::ALL {
            for length in [4, 6, 254] {
                assert!(allowed(domain, Request { length, ..language }));
            }
            for length in [2, 4, 36, 254] {
                assert!(allowed(domain, Request { length, ..serial }));
            }
            for length in [0, 1, 2, 3, 5, 253, 255, 256, 65534, 65535] {
                assert!(!allowed(domain, Request { length, ..language }));
            }
            for length in [0, 1, 3, 253, 255, 256, 65534, 65535] {
                assert!(!allowed(domain, Request { length, ..serial }));
            }
            for request in [
                Request {
                    index: 1,
                    ..language
                },
                Request { index: 0, ..serial },
                Request {
                    value: 0x0300,
                    ..serial
                },
                Request {
                    value: 0x0403,
                    ..serial
                },
                Request {
                    request_type: 0,
                    ..serial
                },
            ] {
                assert!(!allowed(domain, request));
            }
        }
    }

    #[test]
    fn native_allowlist_rejects_writes_wrong_domains_and_unbounded_requests() {
        let device = Request {
            request_type: 0x80,
            request: 6,
            value: 0x0100,
            index: 0,
            length: 18,
        };
        assert!(allowed(Domain::NativeUsbSerialJtag, device));
        for changed in [
            Request {
                request_type: 0,
                ..device
            },
            Request {
                request: 9,
                ..device
            },
            Request {
                length: 65535,
                ..device
            },
            Request { index: 1, ..device },
        ] {
            assert!(!allowed(Domain::NativeUsbSerialJtag, changed));
        }
        let config = Request {
            request_type: 0xc0,
            request: 0xff,
            value: 0x000e,
            index: 0,
            length: 678,
        };
        assert!(allowed(Domain::Cp2102nBridge, config));
        assert!(!allowed(Domain::NativeUsbSerialJtag, config));
        assert!(!allowed(
            Domain::Cp2102nBridge,
            Request {
                length: 679,
                ..config
            }
        ));
    }
}
