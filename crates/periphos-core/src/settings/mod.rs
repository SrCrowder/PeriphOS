//! Application preferences.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Accent {
    #[default]
    Orange,
    Violet,
    Blue,
    Cyan,
    Green,
    Rose,
}

/// Where application updates come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannel {
    /// Tagged releases.
    Stable,
    /// A build of every change merged into the main branch. For testers.
    Development,
}

/// User preferences. Every field has a default so that settings saved by an
/// older version always deserialize after an upgrade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: Theme,
    pub accent: Accent,
    pub reduce_motion: bool,
    /// `None` means "use the channel this build was published on".
    pub update_channel: Option<UpdateChannel>,
    /// Look for a new version when the app starts.
    pub auto_check_updates: bool,
    /// Keep running in the system tray when the window is closed, so profiles
    /// keep switching and settings keep being applied.
    pub minimize_to_tray: bool,
    /// Notify when a wireless device's battery runs low.
    pub battery_alerts: bool,
    /// First alert, in percent.
    pub battery_warning_percent: u8,
    /// Second, more urgent alert, in percent. Kept below the warning level.
    pub battery_critical_percent: u8,
}

impl AppSettings {
    /// Clamps values that came from older versions or the UI into a valid range.
    pub fn normalized(mut self) -> Self {
        self.battery_warning_percent = self.battery_warning_percent.clamp(5, 50);
        self.battery_critical_percent = self
            .battery_critical_percent
            .clamp(1, self.battery_warning_percent.saturating_sub(1).max(1));
        self
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            accent: Accent::default(),
            reduce_motion: false,
            update_channel: None,
            auto_check_updates: true,
            minimize_to_tray: true,
            battery_alerts: true,
            battery_warning_percent: 20,
            battery_critical_percent: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: AppSettings = serde_json::from_str(r#"{"theme":"light"}"#).unwrap();
        assert_eq!(s.theme, Theme::Light);
        assert_eq!(s.accent, Accent::Orange);
        assert!(!s.reduce_motion);
        assert_eq!(s.update_channel, None);
        assert!(s.auto_check_updates);
        assert!(s.minimize_to_tray);
        assert!(s.battery_alerts);
        assert_eq!(
            (s.battery_warning_percent, s.battery_critical_percent),
            (20, 10)
        );
    }

    #[test]
    fn critical_level_stays_below_warning() {
        let s = AppSettings {
            battery_warning_percent: 10,
            battery_critical_percent: 30,
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            (s.battery_warning_percent, s.battery_critical_percent),
            (10, 9)
        );
        let s = AppSettings {
            battery_warning_percent: 90,
            ..Default::default()
        }
        .normalized();
        assert_eq!(s.battery_warning_percent, 50);
    }
}
