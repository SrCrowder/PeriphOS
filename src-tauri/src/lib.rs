//! PeriphOS desktop application: wires the core crates to Tauri.
//!
//! This crate should stay thin. Business rules belong in `periphos-core`,
//! persistence in `periphos-storage`; here we only open resources, register
//! IPC commands and translate errors.

mod app_watcher;
mod commands;
mod device_watcher;
mod error;
mod profile_runtime;
mod state;
mod tray;
mod updates;

use tauri::Manager;

use crate::state::AppState;
use crate::updates::PendingUpdate;

const DATABASE_FILE: &str = "periphos.db";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be first: a second launch focuses the running instance instead of
        // starting another one that would compete for the same devices.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app)
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(PendingUpdate::default())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let state = AppState::new(data_dir.join(DATABASE_FILE))?;
            app.manage(state);
            device_watcher::spawn(app.handle().clone());
            app_watcher::spawn(app.handle().clone());
            tray::create(app.handle());
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::devices::get_device_overview,
            commands::devices::get_device_settings,
            commands::devices::apply_device_setting,
            commands::profiles::list_profiles,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::set_default_profile,
            commands::profiles::get_active_profile,
            commands::profiles::activate_profile,
            commands::profiles::resume_automatic_switching,
            commands::profiles::list_app_rules,
            commands::profiles::add_app_rule,
            commands::profiles::delete_app_rule,
            commands::profiles::list_running_apps,
            commands::profiles::list_profile_device_settings,
            commands::profiles::save_device_settings_to_profile,
            commands::profiles::delete_profile_device_settings,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::updates::get_update_status,
            commands::updates::check_for_update,
            commands::updates::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PeriphOS");
}
