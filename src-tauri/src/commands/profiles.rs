use periphos_core::profile::{DeviceProfileSettings, Profile, ProfileDraft};
use periphos_core::CoreError;
use tauri::{AppHandle, Emitter, State};

use crate::commands::devices::blocking;
use crate::error::IpcResult;
use crate::profile_runtime::{self, ActivationReport, ApplyReason, PROFILE_APPLIED_EVENT};
use crate::state::AppState;

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> IpcResult<Vec<Profile>> {
    Ok(state.storage()?.list_profiles()?)
}

#[tauri::command]
pub fn create_profile(state: State<'_, AppState>, draft: ProfileDraft) -> IpcResult<Profile> {
    Ok(state.storage()?.create_profile(draft)?)
}

#[tauri::command]
pub fn update_profile(
    state: State<'_, AppState>,
    id: String,
    draft: ProfileDraft,
) -> IpcResult<Profile> {
    Ok(state.storage()?.update_profile(&id, draft)?)
}

#[tauri::command]
pub fn delete_profile(state: State<'_, AppState>, id: String) -> IpcResult<()> {
    state.storage()?.delete_profile(&id)?;
    // Deleting the active profile falls back to the default one.
    if state.active_profile_id()?.as_deref() == Some(id.as_str()) {
        state.set_active_profile_id(None)?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_default_profile(state: State<'_, AppState>, id: String) -> IpcResult<Profile> {
    Ok(state.storage()?.set_default_profile(&id)?)
}

#[tauri::command]
pub fn get_active_profile(state: State<'_, AppState>) -> IpcResult<Profile> {
    Ok(profile_runtime::active_profile(&state)?)
}

/// Makes `id` the active profile and applies its device settings.
#[tauri::command]
pub async fn activate_profile(app: AppHandle, id: String) -> IpcResult<ActivationReport> {
    blocking(app.clone(), move |state| {
        let profile = state.storage()?.get_profile(&id)?;
        state.set_active_profile_id(Some(profile.id.clone()))?;
        let devices = state.devices.scan().devices;
        let results = profile_runtime::apply_to_devices(state, &profile, &devices)?;
        let report = ActivationReport {
            profile,
            reason: ApplyReason::Activated,
            results,
        };
        let _ = app.emit(PROFILE_APPLIED_EVENT, &report);
        Ok(report)
    })
    .await
}

#[tauri::command]
pub fn list_profile_device_settings(
    state: State<'_, AppState>,
    profile_id: String,
) -> IpcResult<Vec<DeviceProfileSettings>> {
    Ok(state.storage()?.profile_device_settings(&profile_id)?)
}

/// Reads the device's current settings and stores them in the profile.
#[tauri::command]
pub async fn save_device_settings_to_profile(
    app: AppHandle,
    profile_id: String,
    device_id: String,
) -> IpcResult<DeviceProfileSettings> {
    blocking(app, move |state| {
        let device = state
            .devices
            .scan()
            .devices
            .into_iter()
            .find(|d| d.id == device_id)
            .ok_or_else(|| CoreError::Device("The device is not connected.".into()))?;
        let live = state
            .devices
            .driver_for(&device.id)?
            .read_settings(&device.id)?;
        let entry = DeviceProfileSettings {
            device_id: device.id,
            device_name: device.name,
            dpi: live.dpi.map(|d| d.current),
            report_rate_hz: live.report_rate.map(|r| r.current_hz),
            updated_at: 0,
        };
        state
            .storage()?
            .save_profile_device_settings(&profile_id, &entry)
    })
    .await
}

#[tauri::command]
pub fn delete_profile_device_settings(
    state: State<'_, AppState>,
    profile_id: String,
    device_id: String,
) -> IpcResult<()> {
    Ok(state
        .storage()?
        .delete_profile_device_settings(&profile_id, &device_id)?)
}
