use periphos_core::device::{DeviceOverview, DeviceSettings, SettingChange};
use periphos_core::CoreError;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::IpcResult;
use crate::state::AppState;

/// Scan every registered driver. Runs on a worker thread because real
/// drivers may perform blocking I/O.
#[tauri::command]
pub async fn get_device_overview(state: State<'_, AppState>) -> Result<DeviceOverview, ()> {
    Ok(state.devices.scan())
}

/// Runs blocking device I/O off the async runtime.
pub async fn blocking<T: Send + 'static>(
    app: AppHandle,
    f: impl FnOnce(&AppState) -> Result<T, CoreError> + Send + 'static,
) -> IpcResult<T> {
    tauri::async_runtime::spawn_blocking(move || f(&app.state::<AppState>()))
        .await
        .map_err(|e| CoreError::Device(format!("device task failed: {e}")))?
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_device_settings(app: AppHandle, device_id: String) -> IpcResult<DeviceSettings> {
    blocking(app, move |state| {
        state
            .devices
            .driver_for(&device_id)?
            .read_settings(&device_id)
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingResult {
    settings: DeviceSettings,
    /// False when the device accepted the request but reports a different value
    /// afterwards (for example because an onboard profile is in control).
    took_effect: bool,
}

#[tauri::command]
pub async fn apply_device_setting(
    app: AppHandle,
    device_id: String,
    change: SettingChange,
) -> IpcResult<SettingResult> {
    blocking(app, move |state| {
        let settings = state
            .devices
            .driver_for(&device_id)?
            .apply_setting(&device_id, change)?;
        Ok(SettingResult {
            took_effect: change.took_effect(&settings),
            settings,
        })
    })
    .await
}
