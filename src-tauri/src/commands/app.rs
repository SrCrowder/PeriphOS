use periphos_core::device::Platform;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::error::IpcResult;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// The app's release version (e.g. `0.2.0` or `0.2.0-dev.14`), the same one
    /// the updater compares.
    version: String,
    platform: Platform,
    database_path: String,
    schema_version: u32,
}

#[tauri::command]
pub fn get_app_info(app: AppHandle, state: State<'_, AppState>) -> IpcResult<AppInfo> {
    let schema_version = state.storage()?.schema_version()?;
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        platform: Platform::current(),
        database_path: state.database_path.display().to_string(),
        schema_version,
    })
}
