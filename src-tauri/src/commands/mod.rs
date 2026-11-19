//! Tauri IPC commands, grouped by domain. Each command is a thin adapter
//! over `periphos-core` / `periphos-storage`.

pub mod app;
pub mod devices;
pub mod profiles;
pub mod settings;
pub mod updates;
