//! # periphos-apps
//!
//! Which application is in the foreground, and which applications have
//! windows open, so profiles can follow the game being played.
//!
//! - **Windows**: foreground window, executable path and, for Microsoft Store
//!   and Game Pass titles, the Package Family Name. That identifies a game
//!   even though its folder under `WindowsApps` cannot be browsed.
//! - **Other platforms**: not implemented yet. On Linux, X11 can report the
//!   active window but Wayland deliberately does not, so this needs
//!   desktop-specific work (see docs/ARCHITECTURE.md).

pub use periphos_core::apps::{AppIdentity, InstalledPackage};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as platform;

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
use unsupported as platform;

/// Whether this platform can report the foreground application.
pub const SUPPORTED: bool = cfg!(windows);

/// The application owning the foreground window, unless it belongs to this process.
pub fn foreground_app() -> Option<AppIdentity> {
    platform::foreground_app()
}

/// Microsoft Store / Game Pass packages installed for the current user, games
/// first, then by name. Frameworks, resource packages and system apps are left out.
pub fn installed_packages() -> Vec<InstalledPackage> {
    let mut packages = platform::installed_packages();
    packages.sort_by(|a, b| {
        b.is_game.cmp(&a.is_game).then_with(|| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        })
    });
    packages.dedup_by(|a, b| a.package_family.eq_ignore_ascii_case(&b.package_family));
    packages
}

/// Applications with a visible top-level window, one entry per application,
/// sorted by name. PeriphOS itself is left out.
pub fn running_apps() -> Vec<AppIdentity> {
    let mut apps = platform::running_apps();
    apps.sort_by_key(|a| a.display_name().to_lowercase());
    apps.dedup_by(|a, b| match (&a.package_family, &b.package_family) {
        (Some(x), Some(y)) => x.eq_ignore_ascii_case(y),
        _ => a.exe_path == b.exe_path && a.exe_name.eq_ignore_ascii_case(&b.exe_name),
    });
    apps
}
