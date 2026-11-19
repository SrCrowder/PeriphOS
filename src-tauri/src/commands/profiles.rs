use periphos_core::apps::{AppIdentity, AppRule, AppRuleKind, InstalledPackage};
use periphos_core::profile::{DeviceProfileSettings, Profile, ProfileDraft};
use periphos_core::CoreError;
use tauri::{AppHandle, State};

use crate::commands::devices::blocking;
use crate::error::IpcResult;
use crate::profile_runtime::{self, ActivationReport, ActiveProfileInfo, ApplyReason};
use crate::state::{ActiveState, AppState, SwitchMode};

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
    let active = state.active()?;
    if active.profile_id.as_deref() == Some(id.as_str()) {
        state.set_active(ActiveState {
            profile_id: None,
            matched_app: None,
            ..active
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_default_profile(state: State<'_, AppState>, id: String) -> IpcResult<Profile> {
    Ok(state.storage()?.set_default_profile(&id)?)
}

#[tauri::command]
pub fn get_active_profile(state: State<'_, AppState>) -> IpcResult<ActiveProfileInfo> {
    Ok(profile_runtime::active_info(&state)?)
}

/// Makes `id` the active profile and applies its device settings. Automatic
/// switching pauses until the user resumes it.
#[tauri::command]
pub async fn activate_profile(app: AppHandle, id: String) -> IpcResult<ActivationReport> {
    blocking(app.clone(), move |state| {
        let profile = state.storage()?.get_profile(&id)?;
        profile_runtime::switch_to(
            &app,
            state,
            profile,
            SwitchMode::Manual,
            None,
            ApplyReason::Activated,
        )
    })
    .await
}

/// Returns to automatic switching, starting from the current foreground app.
#[tauri::command]
pub async fn resume_automatic_switching(app: AppHandle) -> IpcResult<ActiveProfileInfo> {
    blocking(app.clone(), move |state| {
        let current = state.active()?;
        state.set_active(ActiveState {
            mode: SwitchMode::Automatic,
            matched_app: None,
            ..current
        })?;
        match periphos_apps::foreground_app() {
            Some(foreground) => profile_runtime::follow_foreground(&app, state, &foreground)?,
            None => {
                // Nothing to follow (e.g. PeriphOS itself is in front): use the default profile.
                let default = state
                    .storage()?
                    .list_profiles()?
                    .into_iter()
                    .find(|p| p.is_default)
                    .ok_or_else(|| CoreError::Storage("no default profile".into()))?;
                profile_runtime::switch_to(
                    &app,
                    state,
                    default,
                    SwitchMode::Automatic,
                    None,
                    ApplyReason::AppSwitched,
                )?;
            }
        }
        profile_runtime::active_info(state)
    })
    .await
}

#[tauri::command]
pub fn list_app_rules(state: State<'_, AppState>, profile_id: String) -> IpcResult<Vec<AppRule>> {
    Ok(state.storage()?.profile_app_rules(&profile_id)?)
}

#[tauri::command]
pub fn add_app_rule(
    state: State<'_, AppState>,
    profile_id: String,
    kind: AppRuleKind,
    value: String,
    display_name: String,
) -> IpcResult<AppRule> {
    Ok(state
        .storage()?
        .add_app_rule(&profile_id, kind, &value, &display_name)?)
}

#[tauri::command]
pub fn delete_app_rule(state: State<'_, AppState>, id: String) -> IpcResult<()> {
    Ok(state.storage()?.delete_app_rule(&id)?)
}

/// Microsoft Store / Game Pass packages installed for the current user.
#[tauri::command]
pub async fn list_installed_packages() -> IpcResult<Vec<InstalledPackage>> {
    tauri::async_runtime::spawn_blocking(periphos_apps::installed_packages)
        .await
        .map_err(|e| CoreError::Device(format!("could not list installed apps: {e}")).into())
}

/// Applications with an open window, for linking them to a profile.
#[tauri::command]
pub async fn list_running_apps() -> IpcResult<Vec<AppIdentity>> {
    tauri::async_runtime::spawn_blocking(periphos_apps::running_apps)
        .await
        .map_err(|e| CoreError::Device(format!("could not list applications: {e}")).into())
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
