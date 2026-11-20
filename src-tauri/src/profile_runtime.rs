//! The active profile, automatic switching, and applying device settings.

use periphos_core::apps::{find_rule, AppIdentity};
use periphos_core::device::{ButtonAction, ButtonAssignment, DeviceInfo};
use periphos_core::profile::Profile;
use periphos_core::{CoreError, CoreResult};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::state::{ActiveState, AppState, SwitchMode};

/// Emitted with an [`ActivationReport`] whenever the active profile changes or
/// its settings are applied.
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

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplyReason {
    /// The user activated the profile.
    Activated,
    /// The foreground application changed.
    AppSwitched,
    /// A device connected while the profile was active.
    DeviceConnected,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationReport {
    pub profile: Profile,
    pub mode: SwitchMode,
    /// Linked application that selected the profile, in automatic mode.
    pub matched_app: Option<String>,
    pub reason: ApplyReason,
    pub results: Vec<DeviceApplyResult>,
}

/// What the UI shows about the active profile.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveProfileInfo {
    pub profile: Profile,
    pub mode: SwitchMode,
    pub matched_app: Option<String>,
    /// Whether this platform can follow the foreground application.
    pub automatic_supported: bool,
}

fn default_profile(state: &AppState) -> CoreResult<Profile> {
    state
        .storage()?
        .list_profiles()?
        .into_iter()
        .find(|p| p.is_default)
        .ok_or_else(|| CoreError::Storage("no default profile".into()))
}

/// The active profile: the chosen or matched one if it still exists, else the default.
pub fn active_profile(state: &AppState) -> CoreResult<Profile> {
    if let Some(id) = state.active()?.profile_id {
        if let Ok(profile) = state.storage()?.get_profile(&id) {
            return Ok(profile);
        }
    }
    default_profile(state)
}

pub fn active_info(state: &AppState) -> CoreResult<ActiveProfileInfo> {
    let active = state.active()?;
    Ok(ActiveProfileInfo {
        profile: active_profile(state)?,
        mode: active.mode,
        matched_app: active.matched_app,
        automatic_supported: periphos_apps::SUPPORTED,
    })
}

/// Applies `profile`'s saved settings to the connected devices in `devices`.
/// Buttons come from the profile, or else from the device's saved defaults.
/// Devices with nothing to apply are left untouched.
pub fn apply_to_devices(
    state: &AppState,
    profile: &Profile,
    devices: &[DeviceInfo],
) -> CoreResult<Vec<DeviceApplyResult>> {
    let saved = state.storage()?.profile_device_settings(&profile.id)?;
    let mut results = Vec::new();
    for device in devices {
        let entry = saved.iter().find(|e| e.device_id == device.id);
        let buttons = match entry.and_then(|e| e.buttons.clone()) {
            Some(buttons) => Some(buttons),
            None => state
                .storage()?
                .button_defaults(&device.id)?
                .map(|d| d.actions),
        };
        let changes = entry.map(|e| e.changes()).unwrap_or_default();
        if changes.is_empty() && buttons.is_none() {
            continue;
        }
        let outcome = state.devices.driver_for(&device.id).and_then(|driver| {
            let mut all_took_effect = true;
            for change in changes {
                let after = driver.apply_setting(&device.id, change)?;
                all_took_effect &= change.took_effect(&after);
            }
            if let Some(buttons) = buttons {
                let assignments: Vec<ButtonAssignment> = buttons
                    .into_iter()
                    .enumerate()
                    .map(|(i, action)| ButtonAssignment {
                        index: i as u8,
                        action,
                    })
                    .collect();
                // Only written when different from what the device has.
                match driver.set_buttons(&device.id, &assignments) {
                    Ok(layout) => {
                        all_took_effect &= assignments.iter().all(|a| {
                            matches!(a.action, ButtonAction::Other { .. })
                                || layout
                                    .buttons
                                    .get(a.index as usize)
                                    .is_some_and(|b| b.action == a.action)
                        });
                    }
                    // A device without assignable buttons: nothing to do.
                    Err(CoreError::Unsupported(_)) => {}
                    Err(err) => return Err(err),
                }
            }
            Ok(all_took_effect)
        });
        let (applied, message) = match outcome {
            Ok(true) => (true, None),
            Ok(false) => (
                false,
                Some("The device accepted the settings but reports different values.".into()),
            ),
            Err(err) => (false, Some(err.to_string())),
        };
        results.push(DeviceApplyResult {
            device_id: device.id.clone(),
            device_name: device.name.clone(),
            applied,
            message,
        });
    }
    Ok(results)
}

/// Makes `profile` active, applies it to every connected device and tells the UI.
pub fn switch_to(
    app: &AppHandle,
    state: &AppState,
    profile: Profile,
    mode: SwitchMode,
    matched_app: Option<String>,
    reason: ApplyReason,
) -> CoreResult<ActivationReport> {
    state.set_active(ActiveState {
        profile_id: Some(profile.id.clone()),
        mode,
        matched_app: matched_app.clone(),
    })?;
    let devices = state.devices.scan().devices;
    let results = apply_to_devices(state, &profile, &devices)?;
    let report = ActivationReport {
        profile,
        mode,
        matched_app,
        reason,
        results,
    };
    let _ = app.emit(PROFILE_APPLIED_EVENT, &report);
    Ok(report)
}

/// Automatic mode: the profile linked to `foreground`, or the default profile.
/// Does nothing in manual mode or when the right profile is already active.
pub fn follow_foreground(
    app: &AppHandle,
    state: &AppState,
    foreground: &AppIdentity,
) -> CoreResult<()> {
    let current = state.active()?;
    if current.mode != SwitchMode::Automatic {
        return Ok(());
    }
    let rules = state.storage()?.app_rules()?;
    let (target, matched_app) = match find_rule(&rules, foreground) {
        Some(rule) => match state.storage()?.get_profile(&rule.profile_id) {
            Ok(profile) => (profile, Some(rule.display_name.clone())),
            Err(_) => (default_profile(state)?, None),
        },
        None => (default_profile(state)?, None),
    };
    if current.profile_id.as_deref() == Some(target.id.as_str())
        && current.matched_app == matched_app
    {
        return Ok(());
    }
    switch_to(
        app,
        state,
        target,
        SwitchMode::Automatic,
        matched_app,
        ApplyReason::AppSwitched,
    )
    .map(|_| ())
}

/// Applies the active profile to devices that just connected and tells the UI.
pub fn on_devices_connected(app: &AppHandle, state: &AppState, connected: &[DeviceInfo]) {
    let (Ok(profile), Ok(active)) = (active_profile(state), state.active()) else {
        return;
    };
    let Ok(results) = apply_to_devices(state, &profile, connected) else {
        return;
    };
    if !results.is_empty() {
        let report = ActivationReport {
            profile,
            mode: active.mode,
            matched_app: active.matched_app,
            reason: ApplyReason::DeviceConnected,
            results,
        };
        let _ = app.emit(PROFILE_APPLIED_EVENT, &report);
    }
}

/// Returns to automatic switching, starting from the current foreground app,
/// or the default profile when nothing can be followed.
pub fn resume_automatic(app: &AppHandle, state: &AppState) -> CoreResult<ActiveProfileInfo> {
    let current = state.active()?;
    state.set_active(ActiveState {
        mode: SwitchMode::Automatic,
        matched_app: None,
        ..current
    })?;
    match periphos_apps::foreground_app() {
        Some(foreground) => follow_foreground(app, state, &foreground)?,
        None => {
            let default = default_profile(state)?;
            switch_to(
                app,
                state,
                default,
                SwitchMode::Automatic,
                None,
                ApplyReason::AppSwitched,
            )?;
        }
    }
    crate::tray::refresh(app);
    active_info(state)
}
