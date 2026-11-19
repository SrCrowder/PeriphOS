use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::{BusType, HidApi};
use periphos_core::device::{
    Capability, CapabilityKind, ConnectionType, DeviceDriver, DeviceInfo, DriverInfo, Platform,
    SupportLevel,
};
use periphos_core::CoreResult;

use crate::probe::{probe_receiver, PairedDevice};
use crate::receivers::{receiver, ReceiverFamily};
use crate::session::Session;
use crate::transport::{HidTransport, HIDPP_USAGE_PAGE, USAGE_LONG, USAGE_SHORT};

pub const DRIVER_ID: &str = "logitech-hidpp";
const LOGITECH_VID: u16 = 0x046D;
/// How old the cached result may get before a new probe starts.
const REFRESH_AFTER: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_millis(400);

#[derive(Default)]
struct Cache {
    devices: Vec<DeviceInfo>,
    updated: Option<Instant>,
}

/// Lists the devices connected through Logitech receivers and reads their
/// battery. Read-only: it never changes a device setting.
pub struct LogitechDriver {
    cache: Arc<Mutex<Cache>>,
    refreshing: Arc<AtomicBool>,
}

impl Default for LogitechDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl LogitechDriver {
    pub fn new() -> Self {
        Self {
            cache: Arc::default(),
            refreshing: Arc::default(),
        }
    }

    /// Probing takes up to a few seconds, so it runs on its own thread and
    /// `enumerate` returns the latest result immediately.
    fn refresh_in_background(&self) {
        if self.refreshing.swap(true, Ordering::SeqCst) {
            return;
        }
        let cache = Arc::clone(&self.cache);
        let refreshing = Arc::clone(&self.refreshing);
        let spawned = thread::Builder::new()
            .name("logitech-hidpp".into())
            .spawn(move || {
                let devices = scan();
                if let Ok(mut cache) = cache.lock() {
                    cache.devices = devices;
                    cache.updated = Some(Instant::now());
                }
                refreshing.store(false, Ordering::SeqCst);
            });
        if spawned.is_err() {
            self.refreshing.store(false, Ordering::SeqCst);
        }
    }
}

impl DeviceDriver for LogitechDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            id: DRIVER_ID.into(),
            name: "Logitech HID++".into(),
            description:
                "Finds devices connected through Logitech receivers and reads their battery. \
                          It does not change device settings yet."
                    .into(),
            platforms: vec![Platform::Windows, Platform::Linux],
        }
    }

    fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>> {
        let (devices, stale) = match self.cache.lock() {
            Ok(cache) => (
                cache.devices.clone(),
                cache.updated.map_or(true, |t| t.elapsed() >= REFRESH_AFTER),
            ),
            Err(_) => (Vec::new(), true),
        };
        if stale {
            self.refresh_in_background();
        }
        Ok(devices)
    }
}

/// HID++ collections of one physical receiver.
struct ReceiverHandles {
    product_id: u16,
    serial: String,
    paths: Vec<(u16, CString)>,
}

fn find_receivers(api: &HidApi) -> Vec<ReceiverHandles> {
    let mut grouped: BTreeMap<(u16, String), Vec<(u16, CString)>> = BTreeMap::new();
    for d in api.device_list() {
        let is_hidpp = d.vendor_id() == LOGITECH_VID
            && d.usage_page() == HIDPP_USAGE_PAGE
            && matches!(d.usage(), USAGE_SHORT | USAGE_LONG)
            && matches!(d.bus_type(), BusType::Usb)
            && receiver(d.product_id()).is_some();
        if !is_hidpp {
            continue;
        }
        let serial = d.serial_number().unwrap_or_default().trim().to_owned();
        let paths = grouped.entry((d.product_id(), serial)).or_default();
        if !paths.iter().any(|(_, p)| p.as_c_str() == d.path()) {
            paths.push((d.usage(), d.path().to_owned()));
        }
    }
    grouped
        .into_iter()
        .map(|((product_id, serial), paths)| ReceiverHandles {
            product_id,
            serial,
            paths,
        })
        .collect()
}

fn scan() -> Vec<DeviceInfo> {
    let Ok(api) = HidApi::new() else {
        return Vec::new();
    };
    let mut devices = Vec::new();
    for handles in find_receivers(&api) {
        let Some(info) = receiver(handles.product_id) else {
            continue;
        };
        let Ok(transport) = HidTransport::open(&api, &handles.paths) else {
            continue;
        };
        let mut session = Session::new(transport, REQUEST_TIMEOUT);
        let read_pairing = info.family != ReceiverFamily::Bolt;
        for paired in probe_receiver(&mut session, read_pairing) {
            devices.push(to_device_info(&handles, info.name, paired));
        }
    }
    devices
}

fn to_device_info(
    handles: &ReceiverHandles,
    receiver_name: &str,
    paired: PairedDevice,
) -> DeviceInfo {
    let serial = if handles.serial.is_empty() {
        "0"
    } else {
        &handles.serial
    };
    let name = paired
        .name
        .map(|n| n.strip_prefix("Logitech ").map(str::to_owned).unwrap_or(n))
        .unwrap_or_else(|| {
            if paired.legacy {
                "Older wireless device"
            } else {
                "Wireless device"
            }
            .into()
        });
    let mut capabilities = Vec::new();
    if paired.battery.is_some() {
        capabilities.push(Capability::new(
            CapabilityKind::Battery,
            SupportLevel::Native,
        ));
    }
    DeviceInfo {
        id: format!(
            "{DRIVER_ID}:{:04x}:{serial}:{}",
            handles.product_id, paired.slot
        ),
        driver_id: DRIVER_ID.into(),
        name,
        manufacturer: "Logitech".into(),
        vendor_id: LOGITECH_VID,
        product_id: paired.wireless_pid.unwrap_or(0),
        kind: paired.kind,
        connection: ConnectionType::Receiver,
        capabilities,
        known_model: false,
        interface_count: 0,
        note: Some(if paired.legacy {
            "Uses an older version of Logitech's protocol, so PeriphOS can only show that it is connected."
                .into()
        } else {
            "Identified through Logitech's HID++ protocol. Settings cannot be changed from PeriphOS yet.".into()
        }),
        battery: paired.battery,
        connected_via: Some(receiver_name.into()),
    }
}
