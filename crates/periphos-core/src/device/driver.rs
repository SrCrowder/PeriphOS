use serde::{Deserialize, Serialize};

use super::info::DeviceInfo;
use super::settings::{DeviceSettings, SettingChange};
use crate::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Windows,
    Linux,
    MacOs,
}

impl Platform {
    /// The platform PeriphOS was compiled for.
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }
}

/// Static description of a driver, shown in the UI so users know which
/// backends are active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverInfo {
    /// Short unique id, e.g. `"hid-generic"` or `"logitech-hidpp"`.
    pub id: String,
    pub name: String,
    pub description: String,
    pub platforms: Vec<Platform>,
}

/// A backend able to discover and, where implemented, configure a family of
/// devices.
///
/// Implementations live outside this crate (one module or crate per
/// backend). Methods are added as real drivers need them instead of guessing
/// an API up-front; configuration methods default to "unsupported".
pub trait DeviceDriver: Send + Sync {
    fn info(&self) -> DriverInfo;

    /// List the devices this driver currently sees. Must be side-effect free
    /// and must not change any device state.
    fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>>;

    /// Read the live settings of one of this driver's devices.
    fn read_settings(&self, _device_id: &str) -> CoreResult<DeviceSettings> {
        Err(CoreError::Unsupported(
            "this device's settings cannot be read yet".into(),
        ))
    }

    /// Apply a change and return the settings read back from the device, so
    /// callers can tell whether the change really took effect.
    fn apply_setting(
        &self,
        _device_id: &str,
        _change: SettingChange,
    ) -> CoreResult<DeviceSettings> {
        Err(CoreError::Unsupported(
            "this device's settings cannot be changed yet".into(),
        ))
    }

    /// A read-only technical report of the device (protocol features,
    /// programmable controls, memory layout), for adding support for it.
    fn diagnostics(&self, _device_id: &str) -> CoreResult<String> {
        Err(CoreError::Unsupported(
            "no technical report is available for this device".into(),
        ))
    }
}
