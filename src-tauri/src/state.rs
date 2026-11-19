use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use periphos_core::device::DeviceManager;
use periphos_core::CoreError;
use periphos_storage::Storage;

/// How the active profile is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchMode {
    /// Follows the foreground application; the default profile otherwise.
    #[default]
    Automatic,
    /// Chosen by the user; stays until the user returns to automatic switching.
    Manual,
}

#[derive(Debug, Clone, Default)]
pub struct ActiveState {
    /// `None` means the default profile.
    pub profile_id: Option<String>,
    pub mode: SwitchMode,
    /// Linked application that selected the profile, in automatic mode.
    pub matched_app: Option<String>,
}

/// Shared application state managed by Tauri.
pub struct AppState {
    storage: Mutex<Storage>,
    active: Mutex<ActiveState>,
    pub devices: DeviceManager,
    pub database_path: PathBuf,
}

impl AppState {
    pub fn new(database_path: PathBuf) -> Result<Self, CoreError> {
        let storage = Storage::open(&database_path)?;
        // Every device backend is registered here (see docs/ARCHITECTURE.md).
        let mut devices = DeviceManager::new();
        // Vendors with a dedicated driver are left out of the generic one.
        devices.register(Box::new(
            periphos_hid::HidDriver::new().excluding_vendors(&[periphos_wooting::WOOTING_VID]),
        ));
        devices.register(Box::new(periphos_wooting::WootingDriver::new()));
        devices.register(Box::new(periphos_logitech::LogitechDriver::new()));
        Ok(Self {
            storage: Mutex::new(storage),
            active: Mutex::new(ActiveState::default()),
            devices,
            database_path,
        })
    }

    pub fn active(&self) -> Result<ActiveState, CoreError> {
        self.active
            .lock()
            .map(|g| g.clone())
            .map_err(|_| CoreError::Storage("profile state lock was poisoned".into()))
    }

    pub fn set_active(&self, state: ActiveState) -> Result<(), CoreError> {
        *self
            .active
            .lock()
            .map_err(|_| CoreError::Storage("profile state lock was poisoned".into()))? = state;
        Ok(())
    }

    pub fn storage(&self) -> Result<MutexGuard<'_, Storage>, CoreError> {
        self.storage
            .lock()
            .map_err(|_| CoreError::Storage("database lock was poisoned".into()))
    }
}
