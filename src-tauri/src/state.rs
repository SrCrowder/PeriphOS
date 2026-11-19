use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use periphos_core::device::DeviceManager;
use periphos_core::CoreError;
use periphos_storage::Storage;

/// Shared application state managed by Tauri.
pub struct AppState {
    storage: Mutex<Storage>,
    /// Profile chosen by the user; `None` means the default profile.
    active_profile: Mutex<Option<String>>,
    pub devices: DeviceManager,
    pub database_path: PathBuf,
}

impl AppState {
    pub fn new(database_path: PathBuf) -> Result<Self, CoreError> {
        let storage = Storage::open(&database_path)?;
        // Every device backend is registered here (see docs/ARCHITECTURE.md).
        let mut devices = DeviceManager::new();
        devices.register(Box::new(periphos_hid::HidDriver::new()));
        devices.register(Box::new(periphos_logitech::LogitechDriver::new()));
        Ok(Self {
            storage: Mutex::new(storage),
            active_profile: Mutex::new(None),
            devices,
            database_path,
        })
    }

    pub fn active_profile_id(&self) -> Result<Option<String>, CoreError> {
        self.active_profile
            .lock()
            .map(|g| g.clone())
            .map_err(|_| CoreError::Storage("profile state lock was poisoned".into()))
    }

    pub fn set_active_profile_id(&self, id: Option<String>) -> Result<(), CoreError> {
        *self
            .active_profile
            .lock()
            .map_err(|_| CoreError::Storage("profile state lock was poisoned".into()))? = id;
        Ok(())
    }

    pub fn storage(&self) -> Result<MutexGuard<'_, Storage>, CoreError> {
        self.storage
            .lock()
            .map_err(|_| CoreError::Storage("database lock was poisoned".into()))
    }
}
