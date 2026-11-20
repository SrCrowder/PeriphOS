//! System tray icon: PeriphOS keeps running there when its window is closed.
//! The tooltip and menu show the active profile and battery levels, and the
//! menu switches profiles without opening the window.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

use periphos_core::device::{BatteryLevel, BatteryState, ChargingState, ConnectionType};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

use crate::battery_icon;
use crate::profile_runtime::{self, ApplyReason};
use crate::state::{AppState, SwitchMode};

/// Whether the tray icon exists. Without it, hiding the window would leave the
/// user no way to bring PeriphOS back, so closing quits instead.
static TRAY_AVAILABLE: AtomicBool = AtomicBool::new(false);

const TRAY_ID: &str = "main";
const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";
const AUTO_ID: &str = "auto";
const PROFILE_PREFIX: &str = "profile:";
const BATTERY_PREFIX: &str = "battery:";
/// Battery tray icons currently shown, by tray ID.
static BATTERY_ICONS: Mutex<Option<HashSet<String>>> = Mutex::new(None);
/// Windows truncates tray tooltips at 127 characters.
const TOOLTIP_MAX: usize = 120;

/// Brings the main window back (from the tray, minimised, or behind others).
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Argument passed when PeriphOS starts with the system: stay in the tray.
pub const HIDDEN_ARG: &str = "--hidden";

/// Shows the window unless PeriphOS started with the system and can wait in
/// the tray. Without a tray the window is always shown, so the app is never
/// left running with no way to reach it.
pub fn show_on_launch<R: Runtime>(app: &AppHandle<R>) {
    let started_hidden = std::env::args().any(|a| a == HIDDEN_ARG);
    if !(started_hidden && TRAY_AVAILABLE.load(Ordering::SeqCst)) {
        show_main_window(app);
    }
}

/// Creates the tray icon. Failure is not fatal: some desktops have no tray.
pub fn create(app: &AppHandle) {
    match build(app) {
        Ok(()) => {
            TRAY_AVAILABLE.store(true, Ordering::SeqCst);
            refresh(app);
        }
        Err(err) => {
            eprintln!("PeriphOS: system tray unavailable, closing the window will quit: {err}")
        }
    }
}

fn build(app: &AppHandle) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("PeriphOS")
        .menu(&basic_menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
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

fn basic_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let open = MenuItem::with_id(app, OPEN_ID, "Open PeriphOS", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit PeriphOS", true, None::<&str>)?;
    Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &quit])
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        OPEN_ID => show_main_window(app),
        QUIT_ID => app.exit(0),
        AUTO_ID => {
            let app = app.clone();
            // Applying settings talks to devices: keep it off the UI thread.
            thread::spawn(move || {
                let _ = profile_runtime::resume_automatic(&app, &app.state::<AppState>());
            });
        }
        _ => {
            let Some(profile_id) = id.strip_prefix(PROFILE_PREFIX).map(str::to_owned) else {
                return;
            };
            let app = app.clone();
            thread::spawn(move || {
                let state = app.state::<AppState>();
                if let Ok(profile) = state.storage().and_then(|s| s.get_profile(&profile_id)) {
                    let _ = profile_runtime::switch_to(
                        &app,
                        &state,
                        profile,
                        SwitchMode::Manual,
                        None,
                        ApplyReason::Activated,
                    );
                }
            });
        }
    }
}

/// "61%", "~61%" when estimated, or the coarse level; with a charging mark.
pub fn battery_label(battery: &BatteryState) -> String {
    let value = match (battery.percent, battery.level) {
        (Some(p), _) if battery.estimated => format!("~{p}%"),
        (Some(p), _) => format!("{p}%"),
        (None, Some(level)) => match level {
            BatteryLevel::Full => "full",
            BatteryLevel::Good => "good",
            BatteryLevel::Low => "low",
            BatteryLevel::Critical => "critical",
            BatteryLevel::Empty => "empty",
        }
        .to_owned(),
        (None, None) => "unknown".to_owned(),
    };
    match battery.charging {
        ChargingState::Charging => format!("{value}, charging"),
        ChargingState::Full => "full".to_owned(),
        _ => value,
    }
}

fn truncate(text: String, max: usize) -> String {
    if text.chars().count() <= max {
        return text;
    }
    let mut cut: String = text.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// Rebuilds the tooltip, menu and battery icons from the current profile and
/// devices. Safe to call from any thread: tray menus may only be changed on
/// the main thread (on Windows, changes from other threads are lost).
pub fn refresh(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        refresh_now(&handle);
        sync_battery_icons(&handle);
    });
}

fn refresh_now(app: &AppHandle) {
    if !TRAY_AVAILABLE.load(Ordering::SeqCst) {
        return;
    }
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let state = app.state::<AppState>();
    let (Ok(info), Ok(profiles)) = (
        profile_runtime::active_info(&state),
        state.storage().and_then(|s| s.list_profiles()),
    ) else {
        return;
    };
    let batteries: Vec<(String, String)> = state
        .last_devices()
        .into_iter()
        .filter(|d| d.connection != ConnectionType::Virtual)
        .filter_map(|d| d.battery.map(|b| (d.name, battery_label(&b))))
        .collect();

    let mode = match (info.mode, &info.matched_app) {
        (SwitchMode::Manual, _) => "chosen manually".to_owned(),
        (SwitchMode::Automatic, Some(app_name)) => format!("for {app_name}"),
        (SwitchMode::Automatic, None) => "automatic".to_owned(),
    };
    let mut tooltip = format!("PeriphOS · {}", info.profile.name);
    for (name, label) in &batteries {
        tooltip.push_str(&format!(" · {name} {label}"));
    }
    let _ = tray.set_tooltip(Some(truncate(tooltip, TOOLTIP_MAX)));

    let menu = (|| -> tauri::Result<Menu<tauri::Wry>> {
        let header = MenuItem::new(
            app,
            format!("Profile: {} ({mode})", info.profile.name),
            false,
            None::<&str>,
        )?;
        let mut battery_items = Vec::new();
        for (name, label) in &batteries {
            battery_items.push(MenuItem::new(
                app,
                format!("{name}: {label}"),
                false,
                None::<&str>,
            )?);
        }
        let mut profile_items = Vec::new();
        for profile in &profiles {
            profile_items.push(CheckMenuItem::with_id(
                app,
                format!("{PROFILE_PREFIX}{}", profile.id),
                &profile.name,
                true,
                profile.id == info.profile.id,
                None::<&str>,
            )?);
        }
        let auto = MenuItem::with_id(
            app,
            AUTO_ID,
            "Switch automatically",
            info.mode == SwitchMode::Manual && info.automatic_supported,
            None::<&str>,
        )?;
        let separator = PredefinedMenuItem::separator(app)?;
        let mut submenu_items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = profile_items
            .iter()
            .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
            .collect();
        submenu_items.push(&separator);
        submenu_items.push(&auto);
        let use_profile = Submenu::with_items(app, "Use profile", true, &submenu_items)?;
        let open = MenuItem::with_id(app, OPEN_ID, "Open PeriphOS", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, QUIT_ID, "Quit PeriphOS", true, None::<&str>)?;
        let separators = [
            PredefinedMenuItem::separator(app)?,
            PredefinedMenuItem::separator(app)?,
            PredefinedMenuItem::separator(app)?,
        ];

        let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&header];
        items.extend(
            battery_items
                .iter()
                .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>),
        );
        items.push(&separators[0]);
        items.push(&use_profile);
        items.push(&separators[1]);
        items.push(&open);
        items.push(&separators[2]);
        items.push(&quit);
        Menu::with_items(app, &items)
    })();
    if let Ok(menu) = menu {
        let _ = tray.set_menu(Some(menu));
    }
}

/// One tray icon per wireless device with a readable battery, drawn with its
/// level; icons of devices that are gone are removed.
fn sync_battery_icons(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state
        .storage()
        .and_then(|s| s.load_settings())
        .unwrap_or_default();
    let devices = if settings.battery_in_tray {
        state.last_devices()
    } else {
        Vec::new()
    };
    let Ok(mut shown) = BATTERY_ICONS.lock() else {
        return;
    };
    let shown = shown.get_or_insert_with(HashSet::new);

    let mut wanted = HashSet::new();
    for device in devices
        .iter()
        .filter(|d| d.connection != ConnectionType::Virtual)
    {
        let Some(battery) = device.battery else {
            continue;
        };
        let id = format!("{BATTERY_PREFIX}{}", device.id);
        let (fill, tone) = battery_icon::appearance(
            &battery,
            settings.battery_warning_percent,
            settings.battery_critical_percent,
        );
        let icon = Image::new_owned(
            battery_icon::render(battery.percent, fill, tone),
            battery_icon::SIZE,
            battery_icon::SIZE,
        );
        let tooltip = format!("{}: {}", device.name, battery_label(&battery));
        match app.tray_by_id(&id) {
            Some(tray) => {
                let _ = tray.set_icon(Some(icon));
                let _ = tray.set_tooltip(Some(tooltip));
            }
            None => {
                let built = TrayIconBuilder::with_id(&id)
                    .icon(icon)
                    .tooltip(tooltip)
                    .show_menu_on_left_click(false)
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            show_main_window(tray.app_handle());
                        }
                    })
                    .build(app);
                if built.is_err() {
                    continue;
                }
            }
        }
        wanted.insert(id);
    }
    for gone in shown.difference(&wanted) {
        let _ = app.remove_tray_by_id(gone);
    }
    *shown = wanted;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn battery(percent: Option<u8>, charging: ChargingState, estimated: bool) -> BatteryState {
        BatteryState {
            percent,
            level: None,
            charging,
            estimated,
        }
    }

    #[test]
    fn battery_labels() {
        assert_eq!(
            battery_label(&battery(Some(61), ChargingState::Discharging, false)),
            "61%"
        );
        assert_eq!(
            battery_label(&battery(Some(61), ChargingState::Discharging, true)),
            "~61%"
        );
        assert_eq!(
            battery_label(&battery(Some(40), ChargingState::Charging, false)),
            "40%, charging"
        );
        assert_eq!(
            battery_label(&battery(Some(100), ChargingState::Full, false)),
            "full"
        );
        let level_only = BatteryState {
            level: Some(BatteryLevel::Low),
            ..battery(None, ChargingState::Discharging, false)
        };
        assert_eq!(battery_label(&level_only), "low");
    }

    #[test]
    fn long_tooltips_are_cut() {
        let cut = truncate("x".repeat(200), TOOLTIP_MAX);
        assert_eq!(cut.chars().count(), TOOLTIP_MAX);
        assert!(cut.ends_with('…'));
        assert_eq!(truncate("short".into(), TOOLTIP_MAX), "short");
    }
}
