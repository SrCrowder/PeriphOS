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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            accent: Accent::default(),
            reduce_motion: false,
            update_channel: None,
            auto_check_updates: true,
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
    }
}
