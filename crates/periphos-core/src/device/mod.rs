//! Unified device abstraction.
//!
//! PeriphOS never talks to hardware directly from the UI or the profile
//! system. Instead, every supported backend (generic HID, Raw Input,
//! manufacturer protocols, ...) is implemented as a [`DeviceDriver`] and
//! registered in the [`DeviceManager`]. Drivers report what they found as
//! [`DeviceInfo`] values, including an honest list of [`Capability`]s with a
//! [`SupportLevel`] each.
//!
//! **Detecting a device does not imply being able to configure it.** A
//! driver must only report `SupportLevel::Native` for features it can
//! actually read and write.

mod capability;
mod driver;
mod info;
mod manager;

pub use capability::{Capability, CapabilityKind, SupportLevel};
pub use driver::{DeviceDriver, DriverInfo, Platform};
pub use info::{BatteryLevel, BatteryState, ChargingState, ConnectionType, DeviceInfo, DeviceKind};
pub use manager::{DeviceManager, DeviceOverview};
