use periphos_core::settings::UpdateChannel;
use periphos_core::CoreError;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::error::IpcResult;
use crate::state::AppState;
use crate::updates::{self, PendingUpdate};

pub const PROGRESS_EVENT: &str = "update-progress";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    current_version: String,
    /// Channel in effect: the user's choice or the build's own channel.
    channel: UpdateChannel,
    build_channel: UpdateChannel,
    /// False for local builds that were not signed for updates.
    configured: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    available: bool,
    current_version: String,
    version: Option<String>,
    notes: Option<String>,
    /// RFC 3339 date of the release, if the manifest has one.
    date: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    downloaded: u64,
    total: Option<u64>,
}

fn effective_channel(state: &AppState) -> Result<UpdateChannel, CoreError> {
    Ok(state
        .storage()?
        .load_settings()?
        .update_channel
        .unwrap_or_else(updates::build_channel))
}

#[tauri::command]
pub fn get_update_status(app: AppHandle, state: State<'_, AppState>) -> IpcResult<UpdateStatus> {
    Ok(UpdateStatus {
        current_version: app.package_info().version.to_string(),
        channel: effective_channel(&state)?,
        build_channel: updates::build_channel(),
        configured: updates::is_configured(&app),
    })
}

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppState>,
    pending: State<'_, PendingUpdate>,
) -> IpcResult<UpdateCheck> {
    let channel = effective_channel(&state)?;
    let found = updates::check(&app, channel).await?;
    let current_version = app.package_info().version.to_string();
    let result = match &found {
        Some(update) => UpdateCheck {
            available: true,
            current_version,
            version: Some(update.version.clone()),
            notes: update.body.clone(),
            date: update.date.map(|d| d.to_string()),
        },
        None => UpdateCheck {
            available: false,
            current_version,
            version: None,
            notes: None,
            date: None,
        },
    };
    *pending
        .0
        .lock()
        .map_err(|_| CoreError::Update("update state is unavailable".into()))? = found;
    Ok(result)
}

/// Download, verify and install the update found by the last check, then
/// restart. On Windows the installer closes and reopens the app by itself.
#[tauri::command]
pub async fn install_update(app: AppHandle, pending: State<'_, PendingUpdate>) -> IpcResult<()> {
    let update = pending
        .0
        .lock()
        .map_err(|_| CoreError::Update("update state is unavailable".into()))?
        .take()
        .ok_or_else(|| CoreError::Update("Check for updates first.".into()))?;

    let mut downloaded: u64 = 0;
    let progress_app = app.clone();
    update
        .download_and_install(
            move |chunk, total| {
                downloaded += chunk as u64;
                let _ = progress_app.emit(PROGRESS_EVENT, Progress { downloaded, total });
            },
            || {},
        )
        .await
        .map_err(|e| CoreError::Update(format!("The update could not be installed: {e}")))?;

    app.restart();
}
