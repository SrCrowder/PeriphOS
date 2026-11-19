//! Turns raw HID interfaces into [`DeviceInfo`]s.
//!
//! One physical device usually exposes several HID interfaces (a mouse often
//! also has a keyboard interface for macros, plus vendor interfaces). They are
//! grouped by vendor ID, product ID and serial number. Two identical devices
//! that report no serial number are therefore shown as one.

use std::collections::BTreeMap;

use periphos_core::device::{ConnectionType, DeviceInfo, DeviceKind};

use crate::database::Database;

pub const DRIVER_ID: &str = "hid-generic";

/// HID usage page "Generic Desktop" and the usages PeriphOS cares about.
const GENERIC_DESKTOP: u16 = 0x01;
const USAGE_MOUSE: u16 = 0x02;
const USAGE_JOYSTICK: u16 = 0x04;
const USAGE_GAMEPAD: u16 = 0x05;
const USAGE_KEYBOARD: u16 = 0x06;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    Usb,
    Bluetooth,
    /// I2C or SPI: devices built into the computer.
    Internal,
    /// Not a hardware bus `hidapi` recognises. In practice these are virtual
    /// devices created by software (e.g. Logitech G HUB or Corsair iCUE drivers).
    Unknown,
}

/// One HID interface (top-level collection) as enumerated by the OS.
#[derive(Debug, Clone)]
pub struct HidInterface {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub usage_page: u16,
    pub usage: u16,
    pub interface_number: i32,
    pub bus: Bus,
}

impl HidInterface {
    fn input_kind(&self) -> Option<DeviceKind> {
        if self.usage_page != GENERIC_DESKTOP {
            return None;
        }
        match self.usage {
            USAGE_MOUSE => Some(DeviceKind::Mouse),
            USAGE_KEYBOARD => Some(DeviceKind::Keyboard),
            USAGE_JOYSTICK | USAGE_GAMEPAD => Some(DeviceKind::Controller),
            _ => None,
        }
    }
}

fn clean(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// Group interfaces into devices. Devices that are neither known models nor
/// expose a mouse, keyboard or controller interface (e.g. UPSs, sensors) are
/// skipped.
pub fn classify(interfaces: &[HidInterface], db: &Database) -> Vec<DeviceInfo> {
    let mut groups: BTreeMap<(u16, u16, String), Vec<&HidInterface>> = BTreeMap::new();
    for interface in interfaces {
        let serial = clean(&interface.serial_number).unwrap_or_default();
        groups
            .entry((interface.vendor_id, interface.product_id, serial))
            .or_default()
            .push(interface);
    }

    groups
        .into_iter()
        .filter_map(|((vendor_id, product_id, serial), members)| {
            let first = members[0];
            let on_unknown_bus = first.bus == Bus::Unknown;
            let catalogued = db.lookup(vendor_id, product_id);
            // A device on an unknown bus is software-made. If it borrows the ID of a real
            // model, do not present it as that model.
            let known = catalogued.filter(|k| k.is_virtual || !on_unknown_bus);
            let borrowed_id = catalogued.is_some() && known.is_none();
            // The lowest-numbered input interface is the device's main function.
            let detected_kind = members
                .iter()
                .filter_map(|i| i.input_kind().map(|k| (i.interface_number, k)))
                .min_by_key(|(number, _)| *number)
                .map(|(_, kind)| kind);
            if known.is_none() && detected_kind.is_none() {
                return None;
            }

            let vendor = db.vendor_name(vendor_id).map(str::to_owned);
            let manufacturer = vendor
                .clone()
                .or_else(|| members.iter().find_map(|i| clean(&i.manufacturer)))
                .unwrap_or_else(|| "Unknown manufacturer".into());
            let name = known
                .map(|k| k.name.clone())
                .or_else(|| members.iter().find_map(|i| clean(&i.product)))
                .unwrap_or_else(|| {
                    if on_unknown_bus { "Virtual device" } else { "Unknown device" }.into()
                });
            let kind = known.map(|k| k.kind).or(detected_kind).unwrap_or(DeviceKind::Other);
            let connection = match first.bus {
                _ if known.is_some_and(|k| k.is_virtual) => ConnectionType::Virtual,
                Bus::Unknown => ConnectionType::Virtual,
                Bus::Bluetooth => ConnectionType::Bluetooth,
                Bus::Internal => ConnectionType::Internal,
                _ if kind == DeviceKind::Receiver => ConnectionType::Receiver,
                Bus::Usb => ConnectionType::Usb,
            };
            let note = match (known, catalogued) {
                (Some(k), _) => k.note.clone(),
                (None, Some(model)) if borrowed_id => Some(format!(
                    "Not connected through USB or Bluetooth, so it is most likely a virtual device \
                     created by software. It uses the same ID as the {} {}, but it is not that device.",
                    vendor.as_deref().unwrap_or("manufacturer's"),
                    model.name
                )),
                _ if on_unknown_bus => Some(
                    "Not connected through USB or Bluetooth, so it is most likely a virtual device \
                     created by software."
                        .into(),
                ),
                _ => None,
            };
            let serial_part = if serial.is_empty() {
                "0".to_owned()
            } else {
                serial
            };

            Some(DeviceInfo {
                id: format!("{DRIVER_ID}:{vendor_id:04x}:{product_id:04x}:{serial_part}"),
                driver_id: DRIVER_ID.into(),
                name,
                manufacturer,
                vendor_id,
                product_id,
                kind,
                connection,
                capabilities: known.map(|k| k.capabilities.clone()).unwrap_or_default(),
                known_model: known.is_some(),
                interface_count: members.len().min(u16::MAX as usize) as u16,
                note,
                battery: None,
                connected_via: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::bundled;

    fn iface(vid: u16, pid: u16, page: u16, usage: u16, number: i32) -> HidInterface {
        HidInterface {
            vendor_id: vid,
            product_id: pid,
            serial_number: None,
            manufacturer: Some("Maker ".into()),
            product: Some(" Gadget".into()),
            usage_page: page,
            usage,
            interface_number: number,
            bus: Bus::Usb,
        }
    }

    #[test]
    fn interfaces_of_one_device_are_grouped() {
        let g502 = [
            iface(0x046d, 0xc099, 0x01, USAGE_MOUSE, 0),
            iface(0x046d, 0xc099, 0x01, USAGE_KEYBOARD, 1),
            iface(0x046d, 0xc099, 0xff00, 0x01, 2),
        ];
        let devices = classify(&g502, bundled());
        assert_eq!(devices.len(), 1);
        let d = &devices[0];
        assert_eq!(d.name, "G502 X");
        assert_eq!(d.manufacturer, "Logitech");
        assert_eq!(d.kind, DeviceKind::Mouse);
        assert!(d.known_model);
        assert_eq!(d.interface_count, 3);
        assert_eq!(d.id, "hid-generic:046d:c099:0");
        assert!(!d.capabilities.is_empty());
    }

    #[test]
    fn unknown_input_device_uses_descriptor_strings() {
        let devices = classify(&[iface(0x1234, 0x5678, 0x01, USAGE_KEYBOARD, 0)], bundled());
        assert_eq!(devices[0].name, "Gadget");
        assert_eq!(devices[0].manufacturer, "Maker");
        assert_eq!(devices[0].kind, DeviceKind::Keyboard);
        assert!(!devices[0].known_model);
        assert!(devices[0].capabilities.is_empty());
    }

    #[test]
    fn non_input_unknown_devices_are_skipped() {
        let ups = iface(0x0463, 0xffff, 0x84, 0x04, 0);
        assert!(classify(&[ups], bundled()).is_empty());
    }

    #[test]
    fn known_receiver_is_reported_as_receiver_connection() {
        let devices = classify(&[iface(0x046d, 0xc547, 0x01, USAGE_MOUSE, 0)], bundled());
        assert_eq!(devices[0].kind, DeviceKind::Receiver);
        assert_eq!(devices[0].connection, ConnectionType::Receiver);
    }

    #[test]
    fn main_function_is_lowest_input_interface() {
        let keyboard_with_mouse = [
            iface(0x1234, 0x0001, 0x01, USAGE_MOUSE, 2),
            iface(0x1234, 0x0001, 0x01, USAGE_KEYBOARD, 0),
        ];
        assert_eq!(
            classify(&keyboard_with_mouse, bundled())[0].kind,
            DeviceKind::Keyboard
        );
    }

    #[test]
    fn serial_numbers_keep_identical_devices_apart() {
        let mut a = iface(0x1234, 0x0001, 0x01, USAGE_MOUSE, 0);
        let mut b = a.clone();
        a.serial_number = Some("A".into());
        b.serial_number = Some("B".into());
        assert_eq!(classify(&[a, b], bundled()).len(), 2);
    }

    #[test]
    fn software_devices_are_marked_virtual() {
        let mut ghub = iface(0x046d, 0xc232, 0x01, USAGE_KEYBOARD, -1);
        ghub.bus = Bus::Unknown;
        assert_eq!(
            classify(&[ghub], bundled())[0].connection,
            ConnectionType::Virtual
        );
    }

    #[test]
    fn unknown_bus_device_does_not_borrow_a_real_model_name() {
        // Seen on Windows with Corsair iCUE installed: a virtual device using the
        // Corsair M75's ID on Corsair's virtual bus.
        let mut i = iface(0x1b1c, 0x1bf0, 0x01, USAGE_MOUSE, 0);
        i.bus = Bus::Unknown;
        i.product = None;
        let d = &classify(&[i], bundled())[0];
        assert_eq!(d.connection, ConnectionType::Virtual);
        assert_eq!(d.manufacturer, "Corsair");
        assert_eq!(d.name, "Virtual device");
        assert!(!d.known_model);
        assert!(d.capabilities.is_empty());
        assert!(d.note.as_deref().unwrap().contains("M75 Gaming Mouse"));
    }

    #[test]
    fn real_model_on_usb_keeps_its_name() {
        let d = &classify(&[iface(0x1b1c, 0x1bf0, 0x01, USAGE_MOUSE, 0)], bundled())[0];
        assert_eq!(d.name, "M75 Gaming Mouse");
        assert_eq!(d.connection, ConnectionType::Usb);
    }

    #[test]
    fn built_in_devices_are_not_virtual() {
        let mut i = iface(0x1234, 0x0001, 0x01, USAGE_KEYBOARD, 0);
        i.bus = Bus::Internal;
        assert_eq!(
            classify(&[i], bundled())[0].connection,
            ConnectionType::Internal
        );
    }

    #[test]
    fn bluetooth_bus_is_reported() {
        let mut i = iface(0x1234, 0x0001, 0x01, USAGE_MOUSE, 0);
        i.bus = Bus::Bluetooth;
        assert_eq!(
            classify(&[i], bundled())[0].connection,
            ConnectionType::Bluetooth
        );
    }
}
