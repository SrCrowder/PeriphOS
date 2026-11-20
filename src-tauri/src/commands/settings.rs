use periphos_core::settings::AppSettings;
use tauri::State;

use crate::error::IpcResult;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> IpcResult<AppSettings> {
    Ok(state.storage()?.load_settings()?)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> IpcResult<AppSettings> {
    let storage = state.storage()?;
    storage.save_settings(&settings)?;
    Ok(storage.load_settings()?)
}

/// Whether PeriphOS starts when the user signs in. The operating system is the
/// source of truth (the Run registry key on Windows), not PeriphOS's settings.
#[tauri::command]
pub fn get_launch_at_startup(app: tauri::AppHandle) -> IpcResult<bool> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| {
        periphos_core::CoreError::Unsupported(format!("start-up launch is unavailable: {e}")).into()
    })
}

#[tauri::command]
pub fn set_launch_at_startup(app: tauri::AppHandle, enabled: bool) -> IpcResult<bool> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.and_then(|()| manager.is_enabled()).map_err(|e| {
        periphos_core::CoreError::Unsupported(format!("could not change start-up launch: {e}"))
            .into()
    })
}
