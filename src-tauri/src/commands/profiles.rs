use periphos_core::profile::{Profile, ProfileDraft};
use tauri::State;

use crate::error::IpcResult;
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
    Ok(state.storage()?.delete_profile(&id)?)
}

#[tauri::command]
pub fn set_default_profile(state: State<'_, AppState>, id: String) -> IpcResult<Profile> {
    Ok(state.storage()?.set_default_profile(&id)?)
}
