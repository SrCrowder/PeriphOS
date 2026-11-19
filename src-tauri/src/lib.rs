//! PeriphOS desktop application: wires the core crates to Tauri.
//!
//! This crate should stay thin. Business rules belong in `periphos-core`,
//! persistence in `periphos-storage`; here we only open resources, register
//! IPC commands and translate errors.

mod commands;
mod error;
mod state;

use tauri::Manager;

use crate::state::AppState;

const DATABASE_FILE: &str = "periphos.db";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let state = AppState::new(data_dir.join(DATABASE_FILE))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::devices::get_device_overview,
            commands::profiles::list_profiles,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::set_default_profile,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PeriphOS");
}
