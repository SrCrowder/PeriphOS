//! The active profile and applying its device settings.

use periphos_core::device::DeviceInfo;
use periphos_core::profile::Profile;
use periphos_core::CoreResult;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::state::AppState;

/// Emitted with an [`ActivationReport`] whenever profile settings are applied.
pub const PROFILE_APPLIED_EVENT: &str = "profile-applied";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceApplyResult {
    pub device_id: String,
    pub device_name: String,
    /// True when every saved setting was applied and read back.
    pub applied: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationReport {
    pub profile: Profile,
    /// Why settings were applied: the user activated the profile, or a device connected.
    pub reason: ApplyReason,
    pub results: Vec<DeviceApplyResult>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplyReason {
    Activated,
    DeviceConnected,
}

/// The active profile: the one chosen by the user, or the default profile.
pub fn active_profile(state: &AppState) -> CoreResult<Profile> {
    let storage = state.storage()?;
    let chosen = state.active_profile_id()?;
    if let Some(id) = chosen {
        if let Ok(profile) = storage.get_profile(&id) {
            return Ok(profile);
        }
    }
    storage
        .list_profiles()?
        .into_iter()
        .find(|p| p.is_default)
        .ok_or_else(|| periphos_core::CoreError::Storage("no default profile".into()))
}

/// Applies `profile`'s saved settings to the connected devices in `devices`.
/// Devices without saved settings are left untouched.
pub fn apply_to_devices(
    state: &AppState,
    profile: &Profile,
    devices: &[DeviceInfo],
) -> CoreResult<Vec<DeviceApplyResult>> {
    let saved = state.storage()?.profile_device_settings(&profile.id)?;
    let mut results = Vec::new();
    for entry in saved {
        let Some(device) = devices.iter().find(|d| d.id == entry.device_id) else {
            continue;
        };
        let outcome = state.devices.driver_for(&device.id).and_then(|driver| {
            let mut all_took_effect = true;
            for change in entry.changes() {
                let after = driver.apply_setting(&device.id, change)?;
                all_took_effect &= change.took_effect(&after);
            }
            Ok(all_took_effect)
        });
        results.push(match outcome {
            Ok(true) => DeviceApplyResult {
                device_id: device.id.clone(),
                device_name: device.name.clone(),
                applied: true,
                message: None,
            },
            Ok(false) => DeviceApplyResult {
                device_id: device.id.clone(),
                device_name: device.name.clone(),
                applied: false,
                message: Some(
                    "The device accepted the settings but reports different values.".into(),
                ),
            },
            Err(err) => DeviceApplyResult {
                device_id: device.id.clone(),
                device_name: device.name.clone(),
                applied: false,
                message: Some(err.to_string()),
            },
        });
    }
    Ok(results)
}

/// Applies the active profile to devices that just connected and tells the UI.
pub fn on_devices_connected(app: &AppHandle, state: &AppState, connected: &[DeviceInfo]) {
    let Ok(profile) = active_profile(state) else {
        return;
    };
    let Ok(results) = apply_to_devices(state, &profile, connected) else {
        return;
    };
    if !results.is_empty() {
        let report = ActivationReport {
            profile,
            reason: ApplyReason::DeviceConnected,
            results,
        };
        let _ = app.emit(PROFILE_APPLIED_EVENT, &report);
    }
}
