//! In-app updates.
//!
//! Releases are published on GitHub by CI (see `.github/workflows`) together
//! with a `latest.json` manifest per channel. Every installer is signed with
//! the project's updater key and the app refuses anything whose signature does
//! not match the public key embedded in `tauri.conf.json`.

use std::sync::Mutex;

use periphos_core::settings::UpdateChannel;
use periphos_core::CoreError;
use tauri::{AppHandle, Runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

const REPOSITORY: &str = "https://github.com/SrCrowder/PeriphOS";

/// Channel used when the user has not picked one: the channel this build was
/// published on. CI sets `PERIPHOS_UPDATE_CHANNEL=development` for dev builds.
pub fn build_channel() -> UpdateChannel {
    match option_env!("PERIPHOS_UPDATE_CHANNEL") {
        Some("development") => UpdateChannel::Development,
        _ => UpdateChannel::Stable,
    }
}

pub fn manifest_url(channel: UpdateChannel) -> String {
    match channel {
        // `releases/latest` only ever points at a published, non-prerelease release.
        UpdateChannel::Stable => format!("{REPOSITORY}/releases/latest/download/latest.json"),
        // Rolling prerelease replaced by every build of the main branch.
        UpdateChannel::Development => format!("{REPOSITORY}/releases/download/nightly/latest.json"),
    }
}

/// Builds signed with a key carry its public key; local developer builds do not.
pub fn is_configured<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|c| c.get("pubkey"))
        .and_then(|k| k.as_str())
        .is_some_and(|k| !k.trim().is_empty())
}

/// Update found by the last check, kept until the user installs it.
#[derive(Default)]
pub struct PendingUpdate(pub Mutex<Option<Update>>);

pub async fn check<R: Runtime>(
    app: &AppHandle<R>,
    channel: UpdateChannel,
) -> Result<Option<Update>, CoreError> {
    if !is_configured(app) {
        return Err(CoreError::Unsupported(
            "this build was not signed for updates. Install a build from the project's releases."
                .into(),
        ));
    }
    let url = manifest_url(channel)
        .parse()
        .map_err(|e| CoreError::Update(format!("invalid update URL: {e}")))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .and_then(|b| b.build())
        .map_err(|e| CoreError::Update(e.to_string()))?;
    updater
        .check()
        .await
        .map_err(|e| friendly_error(&e.to_string(), channel))
}

fn friendly_error(message: &str, channel: UpdateChannel) -> CoreError {
    let channel_name = match channel {
        UpdateChannel::Stable => "Stable",
        UpdateChannel::Development => "Development",
    };
    if message.contains("404") || message.contains("Not Found") {
        CoreError::Update(format!(
            "No release has been published on the {channel_name} channel yet."
        ))
    } else if message.contains("platform") && message.contains("not found") {
        CoreError::Update(format!(
            "The {channel_name} channel has no build for this operating system yet."
        ))
    } else {
        CoreError::Update(format!("Could not check for updates: {message}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_use_distinct_manifests() {
        assert!(
            manifest_url(UpdateChannel::Stable).ends_with("/releases/latest/download/latest.json")
        );
        assert!(manifest_url(UpdateChannel::Development)
            .ends_with("/releases/download/nightly/latest.json"));
    }

    #[test]
    fn missing_release_gets_a_clear_message() {
        let err = friendly_error("http status 404 Not Found", UpdateChannel::Development);
        assert_eq!(
            err.to_string(),
            "No release has been published on the Development channel yet."
        );
    }
}
