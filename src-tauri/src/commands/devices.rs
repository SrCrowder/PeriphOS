use periphos_core::device::DeviceOverview;
use tauri::State;

use crate::state::AppState;

/// Scan every registered driver. Runs on a worker thread because real
/// drivers may perform blocking I/O.
#[tauri::command]
pub async fn get_device_overview(state: State<'_, AppState>) -> Result<DeviceOverview, ()> {
    Ok(state.devices.scan())
}
