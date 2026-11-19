use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use periphos_core::device::DeviceManager;
use periphos_core::CoreError;
use periphos_storage::Storage;

/// Shared application state managed by Tauri.
pub struct AppState {
    storage: Mutex<Storage>,
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
            devices,
            database_path,
        })
    }

    pub fn storage(&self) -> Result<MutexGuard<'_, Storage>, CoreError> {
        self.storage
            .lock()
            .map_err(|_| CoreError::Storage("database lock was poisoned".into()))
    }
}
