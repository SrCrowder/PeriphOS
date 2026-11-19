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
