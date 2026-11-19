//! Follows the foreground application to switch profiles automatically.

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::profile_runtime;
use crate::state::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Starts the watcher on platforms that can report the foreground application.
pub fn spawn(app: AppHandle) {
    if !periphos_apps::SUPPORTED {
        return;
    }
    thread::Builder::new()
        .name("app-watcher".into())
        .spawn(move || loop {
            // PeriphOS's own window is ignored, so opening it never changes the profile.
            if let Some(foreground) = periphos_apps::foreground_app() {
                let state = app.state::<AppState>();
                let _ = profile_runtime::follow_foreground(&app, &state, &foreground);
            }
            thread::sleep(POLL_INTERVAL);
        })
        .expect("failed to start the app watcher thread");
}
