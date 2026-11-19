use serde::{Deserialize, Serialize};

use super::info::DeviceInfo;
use crate::CoreResult;

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

/// A backend able to discover (and, later, configure) a family of devices.
///
/// Implementations live outside this crate (one module or crate per
/// backend). The trait is intentionally small for now; configuration
/// methods will be added in Phase 4/5 as real drivers need them, instead of
/// guessing an API up-front.
pub trait DeviceDriver: Send + Sync {
    fn info(&self) -> DriverInfo;

    /// List the devices this driver currently sees. Must be side-effect free
    /// and must not change any device state.
    fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>>;
}
