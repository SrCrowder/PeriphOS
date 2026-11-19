use serde::{Deserialize, Serialize};

use super::capability::Capability;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Headset,
    Controller,
    /// Wireless receiver (dongle) for one or more devices.
    Receiver,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionType {
    Usb,
    Bluetooth,
    /// Proprietary 2.4 GHz link through a USB receiver.
    Receiver,
    /// Created by software (e.g. a manufacturer app's driver), not hardware.
    Virtual,
    Unknown,
}

/// A peripheral as reported by a driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// Stable identifier, unique across drivers (prefixed with the driver id).
    pub id: String,
    /// Identifier of the [`super::DeviceDriver`] that reported the device.
    pub driver_id: String,
    pub name: String,
    pub manufacturer: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub kind: DeviceKind,
    pub connection: ConnectionType,
    pub capabilities: Vec<Capability>,
    /// True when the model is in PeriphOS's device database, so its name and
    /// capabilities are known rather than read from generic descriptors.
    pub known_model: bool,
    /// Number of HID interfaces the device exposes.
    pub interface_count: u16,
    /// Human-readable remark about how the device is supported.
    pub note: Option<String>,
}
