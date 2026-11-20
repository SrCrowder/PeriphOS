use periphos_core::device::{
    ButtonAction, ButtonLayout, DeviceOverview, DeviceSettings, SettingChange,
};
use periphos_core::CoreError;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::conflicts;
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
        let mut settings = state
            .devices
            .driver_for(&device_id)?
            .read_settings(&device_id)?;
        settings.conflicting_software =
            conflicts::running_for_driver(conflicts::driver_of(&device_id));
        Ok(settings)
    })
    .await
}

/// What each programmable button of the device does.
#[tauri::command]
pub async fn get_device_buttons(app: AppHandle, device_id: String) -> IpcResult<ButtonLayout> {
    blocking(app, move |state| {
        state
            .devices
            .driver_for(&device_id)?
            .read_buttons(&device_id)
    })
    .await
}

/// Assigns an action to one button; returns the layout read back.
#[tauri::command]
pub async fn set_device_button(
    app: AppHandle,
    device_id: String,
    index: u8,
    action: ButtonAction,
) -> IpcResult<ButtonLayout> {
    blocking(app, move |state| {
        state
            .devices
            .driver_for(&device_id)?
            .set_button(&device_id, index, action)
    })
    .await
}

/// Read-only technical report of a device, for adding support for it.
#[tauri::command]
pub async fn device_diagnostics(app: AppHandle, device_id: String) -> IpcResult<String> {
    blocking(app, move |state| {
        state
            .devices
            .driver_for(&device_id)?
            .diagnostics(&device_id)
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
        let mut settings = state
            .devices
            .driver_for(&device_id)?
            .apply_setting(&device_id, change)?;
        settings.conflicting_software =
            conflicts::running_for_driver(conflicts::driver_of(&device_id));
        Ok(SettingResult {
            took_effect: change.took_effect(&settings),
            settings,
        })
    })
    .await
}
