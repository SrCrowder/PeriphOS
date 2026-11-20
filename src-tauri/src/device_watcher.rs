//! Background thread that rescans devices and notifies the UI of changes.
//!
//! Polling keeps this portable across Windows and Linux. Native notifications
//! (WM_DEVICECHANGE, udev monitors) can replace it later without changing the
//! event the frontend listens to.

use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use periphos_core::device::DeviceOverview;
use tauri::{AppHandle, Emitter, Manager};

use crate::battery_alerts::BatteryAlerts;
use crate::profile_runtime;
use crate::state::AppState;

pub const DEVICES_CHANGED_EVENT: &str = "devices-changed";
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn(app: AppHandle) {
    thread::Builder::new()
        .name("device-watcher".into())
        .spawn(move || {
            let mut last: Option<DeviceOverview> = None;
            let mut known: HashSet<String> = HashSet::new();
            let mut battery_alerts = BatteryAlerts::default();
            loop {
                let state = app.state::<AppState>();
                let overview = state.devices.scan();
                if last.as_ref() != Some(&overview) {
                    let _ = app.emit(DEVICES_CHANGED_EVENT, &overview);
                    // Devices that just appeared (plugged in, switched on, or seen for the
                    // first time) get the active profile's settings: a mouse that was
                    // switched off has lost them.
                    let connected: Vec<_> = overview
                        .devices
                        .iter()
                        .filter(|d| !known.contains(&d.id))
                        .cloned()
                        .collect();
                    known = overview.devices.iter().map(|d| d.id.clone()).collect();
                    if !connected.is_empty() {
                        profile_runtime::on_devices_connected(&app, &state, &connected);
                    }
                    notify_low_batteries(&app, &state, &mut battery_alerts, &overview.devices);
                    state.set_last_devices(overview.devices.clone());
                    crate::tray::refresh(&app);
                    last = Some(overview);
                }
                thread::sleep(POLL_INTERVAL);
            }
        })
        .expect("failed to start the device watcher thread");
}

fn notify_low_batteries(
    app: &AppHandle,
    state: &AppState,
    alerts: &mut BatteryAlerts,
    devices: &[periphos_core::device::DeviceInfo],
) {
    use tauri_plugin_notification::NotificationExt;
    let Ok(settings) = state.storage().and_then(|s| s.load_settings()) else {
        return;
    };
    for alert in alerts.check(devices, &settings) {
        let _ = app
            .notification()
            .builder()
            .title(alert.title())
            .body(alert.body())
            .show();
    }
}
