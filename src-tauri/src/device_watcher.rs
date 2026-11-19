//! Background thread that rescans devices and notifies the UI of changes.
//!
//! Polling keeps this portable across Windows and Linux. Native notifications
//! (WM_DEVICECHANGE, udev monitors) can replace it later without changing the
//! event the frontend listens to.

use std::thread;
use std::time::Duration;

use periphos_core::device::DeviceOverview;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub const DEVICES_CHANGED_EVENT: &str = "devices-changed";
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn(app: AppHandle) {
    thread::Builder::new()
        .name("device-watcher".into())
        .spawn(move || {
            let mut last: Option<DeviceOverview> = None;
            loop {
                let overview = app.state::<AppState>().devices.scan();
                if last.as_ref() != Some(&overview) {
                    let _ = app.emit(DEVICES_CHANGED_EVENT, &overview);
                    last = Some(overview);
                }
                thread::sleep(POLL_INTERVAL);
            }
        })
        .expect("failed to start the device watcher thread");
}
