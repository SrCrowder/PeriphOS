//! Known-device database, embedded from `devices.json` at compile time.

use std::collections::HashMap;
use std::sync::OnceLock;

use periphos_core::device::{Capability, DeviceKind};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownDevice {
    #[serde(deserialize_with = "hex_u16")]
    pub vendor_id: u16,
    #[serde(deserialize_with = "hex_u16")]
    pub product_id: u16,
    #[serde(default = "full_mask", deserialize_with = "hex_u16")]
    pub product_id_mask: u16,
    pub name: String,
    pub kind: DeviceKind,
    pub sources: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
    /// Created by software rather than hardware.
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    vendors: HashMap<String, String>,
    devices: Vec<KnownDevice>,
}

fn full_mask() -> u16 {
    0xffff
}

fn hex_u16<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
    let raw = String::deserialize(deserializer)?;
    u16::from_str_radix(raw.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
}

impl Database {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn lookup(&self, vendor_id: u16, product_id: u16) -> Option<&KnownDevice> {
        self.devices.iter().find(|d| {
            d.vendor_id == vendor_id
                && (product_id & d.product_id_mask) == (d.product_id & d.product_id_mask)
        })
    }

    /// Model whose name matches what a device reports about itself (for devices
    /// identified through a vendor protocol rather than by USB ID).
    pub fn lookup_by_name(&self, vendor_id: u16, name: &str) -> Option<&KnownDevice> {
        let name = name.trim();
        self.devices.iter().find(|d| {
            d.vendor_id == vendor_id && !d.is_virtual && d.name.eq_ignore_ascii_case(name)
        })
    }

    pub fn vendor_name(&self, vendor_id: u16) -> Option<&str> {
        self.vendors
            .get(&format!("{vendor_id:04x}"))
            .map(String::as_str)
    }
}

/// The database bundled with PeriphOS.
pub fn bundled() -> &'static Database {
    static DB: OnceLock<Database> = OnceLock::new();
    DB.get_or_init(|| {
        // Validated by the `bundled_database_is_valid` test.
        Database::parse(include_str!("../devices.json")).expect("devices.json is invalid")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_database_is_valid() {
        let db = Database::parse(include_str!("../devices.json")).unwrap();
        for device in &db.devices {
            assert!(!device.sources.is_empty(), "{} has no source", device.name);
            assert!(
                db.vendor_name(device.vendor_id).is_some(),
                "{} has no vendor name",
                device.name
            );
        }
    }

    #[test]
    fn wooting_gamepad_modes_share_one_entry() {
        let db = bundled();
        // The last PID nibble changes with the keyboard's controller emulation mode.
        for pid in [0x1400, 0x1401, 0x1402] {
            assert_eq!(db.lookup(0x31e3, pid).unwrap().name, "80HE");
        }
        assert!(db.lookup(0x31e3, 0x1500).is_none());
    }

    #[test]
    fn devices_reported_by_a_tester_are_identified() {
        let db = bundled();
        assert!(db.lookup(0x046d, 0xc232).unwrap().is_virtual);
        assert_eq!(
            db.lookup(0x046d, 0xc52b).unwrap().kind,
            DeviceKind::Receiver
        );
        assert_eq!(db.lookup(0x1b1c, 0x1bf0).unwrap().name, "M75 Gaming Mouse");
        assert_eq!(db.vendor_name(0x1b1c), Some("Corsair"));
    }

    #[test]
    fn models_can_be_found_by_reported_name() {
        let db = bundled();
        assert_eq!(
            db.lookup_by_name(0x046d, "G502 X PLUS").unwrap().product_id,
            0xc095
        );
        assert!(db
            .lookup_by_name(0x046d, "G HUB virtual keyboard")
            .is_none());
        assert!(db.lookup_by_name(0x31e3, "G502 X PLUS").is_none());
    }

    #[test]
    fn logitech_ids_match_exactly() {
        let db = bundled();
        assert_eq!(db.lookup(0x046d, 0xc099).unwrap().name, "G502 X");
        assert_eq!(
            db.lookup(0x046d, 0xc547).unwrap().kind,
            DeviceKind::Receiver
        );
        assert!(db.lookup(0x046d, 0xc09a).is_none());
    }
}
