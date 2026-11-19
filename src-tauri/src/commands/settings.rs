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
