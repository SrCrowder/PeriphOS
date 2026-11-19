use periphos_core::device::Platform;
use serde::Serialize;
use tauri::State;

use crate::error::IpcResult;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    platform: Platform,
    database_path: String,
    schema_version: u32,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> IpcResult<AppInfo> {
    let schema_version = state.storage()?.schema_version()?;
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        platform: Platform::current(),
        database_path: state.database_path.display().to_string(),
        schema_version,
    })
}
