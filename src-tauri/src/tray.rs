//! System tray icon: PeriphOS keeps running there when its window is closed.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

use crate::state::AppState;

/// Whether the tray icon exists. Without it, hiding the window would leave the
/// user no way to bring PeriphOS back, so closing quits instead.
static TRAY_AVAILABLE: AtomicBool = AtomicBool::new(false);

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

/// Brings the main window back (from the tray, minimised, or behind others).
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Creates the tray icon. Failure is not fatal: some desktops have no tray.
pub fn create<R: Runtime>(app: &AppHandle<R>) {
    match build(app) {
        Ok(()) => TRAY_AVAILABLE.store(true, Ordering::SeqCst),
        Err(err) => {
            eprintln!("PeriphOS: system tray unavailable, closing the window will quit: {err}")
        }
    }
}

fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, OPEN_ID, "Open PeriphOS", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit PeriphOS", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("PeriphOS")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => show_main_window(app),
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Closing the window hides it to the tray when the user wants PeriphOS to keep running.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    if !TRAY_AVAILABLE.load(Ordering::SeqCst) {
        return;
    }
    let keep_running = window
        .app_handle()
        .try_state::<AppState>()
        .and_then(|state| state.storage().ok().and_then(|s| s.load_settings().ok()))
        .map_or(true, |settings| settings.minimize_to_tray);
    if keep_running {
        api.prevent_close();
        let _ = window.hide();
    }
}
