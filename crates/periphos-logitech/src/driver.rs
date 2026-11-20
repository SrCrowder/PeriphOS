use std::collections::{BTreeMap, HashMap};
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::{BusType, HidApi};
use periphos_core::device::{
    ButtonAction, ButtonLayout, Capability, CapabilityKind, ConnectionType, DeviceDriver,
    DeviceInfo, DeviceSettings, DriverInfo, Platform, SettingChange, SupportLevel,
};
use periphos_core::{CoreError, CoreResult};
use periphos_hid::database::{self, KnownDevice};

use crate::probe::{probe_receiver, PairedDevice, MAX_SLOTS};
use crate::receivers::{receiver, ReceiverFamily};
use crate::session::{self, Session};
use crate::settings;
use crate::transport::{HidTransport, HIDPP_USAGE_PAGE, USAGE_LONG, USAGE_SHORT};

pub const DRIVER_ID: &str = "logitech-hidpp";
const LOGITECH_VID: u16 = 0x046D;
/// How old the cached result may get before a new probe starts.
const REFRESH_AFTER: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_millis(400);
/// How long a device that stops answering stays listed with its last known
/// data. Wireless mice sleep after a few seconds without movement and then
/// do not answer until they are moved again.
const KEEP_UNREACHABLE: Duration = Duration::from_secs(60);
const LIVE_SETTING_NOTE: &str =
    "Applied right away. The mouse keeps it until it is switched off or other software, such as G HUB, changes it.";

#[derive(Default)]
struct Cache {
    devices: Vec<DeviceInfo>,
    /// When each listed device last answered.
    last_seen: HashMap<String, Instant>,
    updated: Option<Instant>,
}

/// Lists devices connected through Logitech receivers, reads their battery and
/// changes DPI and report rate.
pub struct LogitechDriver {
    cache: Arc<Mutex<Cache>>,
    refreshing: Arc<AtomicBool>,
    /// Serialises all HID++ traffic: background probes and user changes must not
    /// interleave requests that share the same software ID.
    io: Arc<Mutex<()>>,
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
            io: Arc::default(),
        }
    }

    /// Probing takes up to a few seconds, so it runs on its own thread and
    /// `enumerate` returns the latest result immediately.
    fn refresh_in_background(&self) {
        if self.refreshing.swap(true, Ordering::SeqCst) {
            return;
        }
        let (cache, refreshing, io) = (
            Arc::clone(&self.cache),
            Arc::clone(&self.refreshing),
            Arc::clone(&self.io),
        );
        let spawned = thread::Builder::new()
            .name("logitech-hidpp".into())
            .spawn(move || {
                let (previous, last_seen) = match cache.lock() {
                    Ok(c) => (c.devices.clone(), c.last_seen.clone()),
                    Err(_) => (Vec::new(), HashMap::new()),
                };
                let (devices, last_seen) = {
                    let _io = io.lock();
                    scan(&previous, &last_seen, Instant::now())
                };
                if let Ok(mut cache) = cache.lock() {
                    cache.devices = devices;
                    cache.last_seen = last_seen;
                    cache.updated = Some(Instant::now());
                }
                refreshing.store(false, Ordering::SeqCst);
            });
        if spawned.is_err() {
            self.refreshing.store(false, Ordering::SeqCst);
        }
    }

    /// Opens the receiver that `device_id` belongs to and runs `f` on its slot.
    fn with_device<R>(
        &self,
        device_id: &str,
        f: impl FnOnce(&mut Session<HidTransport>, u8) -> Result<R, session::Error>,
    ) -> CoreResult<R> {
        let id = DeviceId::parse(device_id).ok_or_else(|| CoreError::NotFound {
            entity: "Device",
            id: device_id.to_owned(),
        })?;
        let _io = self
            .io
            .lock()
            .map_err(|_| CoreError::Device("device access is unavailable".into()))?;
        let api = HidApi::new().map_err(|e| CoreError::Device(e.to_string()))?;
        let handles = find_receivers(&api)
            .into_iter()
            .find(|r| r.product_id == id.receiver_pid && r.serial_key() == id.receiver_serial)
            .ok_or_else(|| {
                CoreError::Device("The receiver for this device is not connected.".into())
            })?;
        let transport = HidTransport::open(&api, &handles.paths).map_err(CoreError::Device)?;
        let mut session = Session::new(transport, REQUEST_TIMEOUT);
        f(&mut session, id.slot).map_err(to_core_error)
    }
}

fn model_name(s: &mut Session<HidTransport>, slot: u8) -> String {
    s.name_and_type(slot)
        .ok()
        .flatten()
        .map(|(name, _)| name)
        .unwrap_or_default()
}

fn to_core_error(error: session::Error) -> CoreError {
    match error {
        session::Error::Timeout => {
            CoreError::Device("The device did not answer. It may be switched off or asleep.".into())
        }
        session::Error::Feature(code) => CoreError::Device(format!(
            "The device rejected the request (HID++ error {code})."
        )),
        session::Error::Hidpp10(_) => {
            CoreError::Device("The device is not reachable through its receiver.".into())
        }
        session::Error::Io(message) => CoreError::Device(message),
    }
}

impl DeviceDriver for LogitechDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            id: DRIVER_ID.into(),
            name: "Logitech HID++".into(),
            description:
                "Finds devices connected through Logitech receivers, reads their battery and \
                          changes sensitivity (DPI) and report rate."
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

    fn read_settings(&self, device_id: &str) -> CoreResult<DeviceSettings> {
        self.with_device(device_id, settings::read_all)
    }

    fn read_buttons(&self, device_id: &str) -> CoreResult<ButtonLayout> {
        self.with_device(device_id, |s, slot| {
            let model = model_name(s, slot);
            crate::onboard::read_buttons(s, slot, &model)
        })?
        .ok_or_else(|| CoreError::Unsupported("this device has no onboard profiles".into()))
    }

    fn set_button(
        &self,
        device_id: &str,
        index: u8,
        action: ButtonAction,
    ) -> CoreResult<ButtonLayout> {
        self.with_device(device_id, |s, slot| {
            let model = model_name(s, slot);
            crate::onboard::set_button(s, slot, &model, index, &action)
        })
    }

    fn diagnostics(&self, device_id: &str) -> CoreResult<String> {
        self.with_device(device_id, crate::diagnostics::report)
    }

    fn apply_setting(&self, device_id: &str, change: SettingChange) -> CoreResult<DeviceSettings> {
        self.with_device(device_id, |s, slot| {
            // Validate against what the device itself reports before writing anything.
            let current = settings::read_all(s, slot)?;
            match change {
                SettingChange::Dpi(dpi) => {
                    let allowed = current
                        .dpi
                        .as_ref()
                        .is_some_and(|d| d.values.contains(&dpi));
                    if !allowed {
                        return Err(session::Error::Io(format!(
                            "{dpi} DPI is not a value this mouse accepts."
                        )));
                    }
                    settings::set_dpi(s, slot, dpi)?;
                }
                SettingChange::ReportRateHz(hz) => {
                    let allowed = current
                        .report_rate
                        .as_ref()
                        .is_some_and(|r| r.values_hz.contains(&hz));
                    if !allowed {
                        return Err(session::Error::Io(format!(
                            "{hz} Hz is not a rate this device accepts."
                        )));
                    }
                    settings::set_report_rate(s, slot, hz)?;
                }
                SettingChange::OnboardProfile(_) => {
                    return Err(session::Error::Io(
                        "Switching onboard profiles is not supported for this device yet.".into(),
                    ));
                }
            }
            settings::read_all(s, slot)
        })
    }
}

/// `logitech-hidpp:{receiver pid}:{receiver serial or 0}:{slot}`
struct DeviceId {
    receiver_pid: u16,
    receiver_serial: String,
    slot: u8,
}

impl DeviceId {
    fn format(receiver_pid: u16, receiver_serial: &str, slot: u8) -> String {
        format!("{DRIVER_ID}:{receiver_pid:04x}:{receiver_serial}:{slot}")
    }

    fn parse(id: &str) -> Option<Self> {
        let rest = id.strip_prefix(DRIVER_ID)?.strip_prefix(':')?;
        let (rest, slot) = rest.rsplit_once(':')?;
        let (pid, serial) = rest.split_once(':')?;
        Some(Self {
            receiver_pid: u16::from_str_radix(pid, 16).ok()?,
            receiver_serial: serial.to_owned(),
            slot: slot
                .parse()
                .ok()
                .filter(|s| (1..=crate::probe::MAX_SLOTS).contains(s))?,
        })
    }
}

/// HID++ collections of one physical receiver.
struct ReceiverHandles {
    product_id: u16,
    serial: String,
    paths: Vec<(u16, CString)>,
}

impl ReceiverHandles {
    fn serial_key(&self) -> String {
        if self.serial.is_empty() {
            "0".into()
        } else {
            self.serial.clone()
        }
    }
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

/// Probes every receiver. Devices that did not answer this time keep their
/// previous entry for [`KEEP_UNREACHABLE`], so a sleeping mouse does not
/// flicker out of the list.
fn scan(
    previous: &[DeviceInfo],
    last_seen: &HashMap<String, Instant>,
    now: Instant,
) -> (Vec<DeviceInfo>, HashMap<String, Instant>) {
    let Ok(api) = HidApi::new() else {
        return (Vec::new(), HashMap::new());
    };
    let mut merger = Merger::new(previous, last_seen, now);
    for handles in find_receivers(&api) {
        let Some(info) = receiver(handles.product_id) else {
            continue;
        };
        let result = HidTransport::open(&api, &handles.paths).map(|transport| {
            let mut session = Session::new(transport, REQUEST_TIMEOUT);
            probe_receiver(&mut session, info.family != ReceiverFamily::Bolt)
        });
        let (answered, unreachable) = match result {
            Ok(r) => (r.devices, r.unreachable),
            // Could not open the receiver this time: treat every slot as unreachable.
            Err(_) => (Vec::new(), (1..=MAX_SLOTS).collect()),
        };
        for paired in answered {
            merger.answered(to_device_info(&handles, info.name, paired));
        }
        for slot in unreachable {
            merger.unreachable(&DeviceId::format(
                handles.product_id,
                &handles.serial_key(),
                slot,
            ));
        }
    }
    merger.finish()
}

/// Combines a new probe with the previous result.
struct Merger<'a> {
    previous: HashMap<&'a str, &'a DeviceInfo>,
    last_seen: &'a HashMap<String, Instant>,
    now: Instant,
    devices: Vec<DeviceInfo>,
    seen: HashMap<String, Instant>,
}

impl<'a> Merger<'a> {
    fn new(
        previous: &'a [DeviceInfo],
        last_seen: &'a HashMap<String, Instant>,
        now: Instant,
    ) -> Self {
        Self {
            previous: previous.iter().map(|d| (d.id.as_str(), d)).collect(),
            last_seen,
            now,
            devices: Vec::new(),
            seen: HashMap::new(),
        }
    }

    fn answered(&mut self, mut device: DeviceInfo) {
        // The device answered the ping but a later request failed (for example its name):
        // keep the earlier, complete entry and only refresh the battery.
        if !device.known_model && device.name.ends_with("Wireless device") {
            if let Some(prev) = self.previous.get(device.id.as_str()) {
                let battery = device.battery.or(prev.battery);
                device = (*prev).clone();
                device.battery = battery;
            }
        }
        self.seen.insert(device.id.clone(), self.now);
        self.devices.push(device);
    }

    fn unreachable(&mut self, id: &str) {
        let (Some(prev), Some(&seen)) = (self.previous.get(id), self.last_seen.get(id)) else {
            return;
        };
        if self.now.duration_since(seen) < KEEP_UNREACHABLE {
            self.seen.insert(id.to_owned(), seen);
            self.devices.push((*prev).clone());
        }
    }

    fn finish(self) -> (Vec<DeviceInfo>, HashMap<String, Instant>) {
        (self.devices, self.seen)
    }
}

/// Model data from the database, upgraded with what this driver can really do.
fn capabilities(paired: &PairedDevice, known: Option<&KnownDevice>) -> Vec<Capability> {
    let mut caps: Vec<Capability> = known.map(|k| k.capabilities.clone()).unwrap_or_default();
    let mut upgrade = |matches: fn(&CapabilityKind) -> bool, note: Option<&str>| {
        if let Some(cap) = caps.iter_mut().find(|c| matches(&c.kind)) {
            cap.support = SupportLevel::Native;
            cap.note = note.map(str::to_owned);
        }
    };
    if paired.adjustable_dpi {
        upgrade(
            |k| matches!(k, CapabilityKind::Dpi { .. }),
            Some(LIVE_SETTING_NOTE),
        );
    }
    if paired.adjustable_report_rate {
        upgrade(
            |k| matches!(k, CapabilityKind::PollingRate { .. }),
            Some(LIVE_SETTING_NOTE),
        );
    }
    if paired.battery.is_some() {
        upgrade(|k| matches!(k, CapabilityKind::Battery), None);
        if !caps
            .iter()
            .any(|c| matches!(c.kind, CapabilityKind::Battery))
        {
            caps.push(Capability::new(
                CapabilityKind::Battery,
                SupportLevel::Native,
            ));
        }
    }
    caps
}

fn to_device_info(
    handles: &ReceiverHandles,
    receiver_name: &str,
    paired: PairedDevice,
) -> DeviceInfo {
    let reported_name = paired
        .name
        .as_deref()
        .map(|n| n.strip_prefix("Logitech ").unwrap_or(n).to_owned());
    let known = reported_name
        .as_deref()
        .and_then(|n| database::bundled().lookup_by_name(LOGITECH_VID, n));
    let name = reported_name.unwrap_or_else(|| {
        if paired.legacy {
            "Older wireless device"
        } else {
            "Wireless device"
        }
        .into()
    });
    let note = if paired.legacy {
        "Uses an older version of Logitech's protocol, so PeriphOS can only show that it is connected."
    } else if paired.adjustable_dpi || paired.adjustable_report_rate {
        "Identified through Logitech's HID++ protocol. Sensitivity and report rate can be changed below; \
         other settings cannot be changed from PeriphOS yet."
    } else {
        "Identified through Logitech's HID++ protocol. Settings cannot be changed from PeriphOS yet."
    };
    DeviceInfo {
        id: DeviceId::format(handles.product_id, &handles.serial_key(), paired.slot),
        driver_id: DRIVER_ID.into(),
        name,
        manufacturer: "Logitech".into(),
        vendor_id: LOGITECH_VID,
        product_id: paired.wireless_pid.unwrap_or(0),
        kind: known.map(|k| k.kind).unwrap_or(paired.kind),
        connection: ConnectionType::Receiver,
        capabilities: capabilities(&paired, known),
        known_model: known.is_some(),
        interface_count: 0,
        note: Some(note.into()),
        battery: paired.battery,
        connected_via: Some(receiver_name.into()),
    }
}

#[cfg(test)]
mod tests {
    use periphos_core::device::DeviceKind;

    use super::*;

    fn paired(name: &str) -> PairedDevice {
        PairedDevice {
            slot: 1,
            name: Some(name.into()),
            kind: DeviceKind::Mouse,
            battery: None,
            wireless_pid: Some(0x4099),
            legacy: false,
            adjustable_dpi: true,
            adjustable_report_rate: false,
        }
    }

    fn handles() -> ReceiverHandles {
        ReceiverHandles {
            product_id: 0xC547,
            serial: String::new(),
            paths: vec![],
        }
    }

    #[test]
    fn sleeping_device_stays_listed_for_a_while() {
        let start = Instant::now();
        let mouse = to_device_info(&handles(), "LIGHTSPEED receiver", paired("G502 X PLUS"));
        let id = mouse.id.clone();

        let empty = HashMap::new();
        let mut merger = Merger::new(&[], &empty, start);
        merger.answered(mouse);
        let (devices, seen) = merger.finish();

        // 20 s later it does not answer: still listed.
        let mut merger = Merger::new(&devices, &seen, start + Duration::from_secs(20));
        merger.unreachable(&id);
        let (devices, seen) = merger.finish();
        assert_eq!(devices.len(), 1);

        // After the grace period: removed.
        let mut merger = Merger::new(
            &devices,
            &seen,
            start + KEEP_UNREACHABLE + Duration::from_secs(1),
        );
        merger.unreachable(&id);
        assert!(merger.finish().0.is_empty());
    }

    #[test]
    fn partial_answer_keeps_the_previous_name() {
        let start = Instant::now();
        let complete = to_device_info(&handles(), "LIGHTSPEED receiver", paired("G502 X PLUS"));
        let empty = HashMap::new();
        let mut merger = Merger::new(&[], &empty, start);
        merger.answered(complete);
        let (devices, seen) = merger.finish();

        let mut nameless = paired("x");
        nameless.name = None;
        let mut merger = Merger::new(&devices, &seen, start);
        merger.answered(to_device_info(&handles(), "LIGHTSPEED receiver", nameless));
        assert_eq!(merger.finish().0[0].name, "G502 X PLUS");
    }

    #[test]
    fn device_ids_round_trip() {
        let id = DeviceId::format(0xC547, "ABC123", 2);
        assert_eq!(id, "logitech-hidpp:c547:ABC123:2");
        let parsed = DeviceId::parse(&id).unwrap();
        assert_eq!(
            (
                parsed.receiver_pid,
                parsed.receiver_serial.as_str(),
                parsed.slot
            ),
            (0xC547, "ABC123", 2)
        );
        assert!(DeviceId::parse("logitech-hidpp:c547:0:9").is_none());
        assert!(DeviceId::parse("hid-generic:046d:c547:0").is_none());
    }

    #[test]
    fn known_model_gets_database_features_with_real_support_levels() {
        let info = to_device_info(&handles(), "LIGHTSPEED receiver", paired("G502 X PLUS"));
        assert!(info.known_model);
        assert_eq!(info.id, "logitech-hidpp:c547:0:1");
        let support = |f: fn(&CapabilityKind) -> bool| {
            info.capabilities
                .iter()
                .find(|c| f(&c.kind))
                .unwrap()
                .support
        };
        assert_eq!(
            support(|k| matches!(k, CapabilityKind::Dpi { .. })),
            SupportLevel::Native
        );
        // Report rate not adjustable on this simulated device: stays a vendor feature.
        assert_eq!(
            support(|k| matches!(k, CapabilityKind::PollingRate { .. })),
            SupportLevel::VendorProtocol
        );
        assert_eq!(
            support(|k| matches!(k, CapabilityKind::Lighting { .. })),
            SupportLevel::VendorProtocol
        );
    }

    #[test]
    fn unknown_model_keeps_only_what_was_read() {
        let mut p = paired("Mystery Mouse");
        p.battery = Some(periphos_core::device::BatteryState {
            percent: Some(50),
            level: None,
            charging: periphos_core::device::ChargingState::Discharging,
            estimated: false,
        });
        let info = to_device_info(&handles(), "LIGHTSPEED receiver", p);
        assert!(!info.known_model);
        assert_eq!(info.capabilities.len(), 1);
        assert_eq!(info.capabilities[0].support, SupportLevel::Native);
    }
}
