//! Live device settings that a driver can read and change.

use serde::{Deserialize, Serialize};

/// Current sensitivity and the values the device accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpiSetting {
    pub current: u32,
    /// Accepted values in ascending order.
    pub values: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRateSetting {
    pub current_hz: u32,
    /// Accepted rates in ascending order.
    pub values_hz: Vec<u32>,
}

/// Which of the profiles stored in the device's memory is in use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardProfileSetting {
    /// Zero-based index of the active onboard profile.
    pub current: u8,
    /// Number of onboard profiles.
    pub count: u8,
}

/// Settings read from the device right now. `None` means the device or the
/// driver does not support that setting.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSettings {
    pub dpi: Option<DpiSetting>,
    pub report_rate: Option<ReportRateSetting>,
    /// True when the device runs from a profile stored in its memory, which can
    /// ignore or override changes. `None` when the device has no such mode.
    pub onboard_profile_active: Option<bool>,
    /// Onboard profile selection, for devices whose profiles PeriphOS can switch.
    #[serde(default)]
    pub onboard_profile: Option<OnboardProfileSetting>,
    /// Other software running now that also controls this device and may undo
    /// PeriphOS's changes (for example the manufacturer's app).
    #[serde(default)]
    pub conflicting_software: Vec<String>,
}

/// A change requested by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SettingChange {
    Dpi(u32),
    ReportRateHz(u32),
    /// Zero-based index of the onboard profile to activate.
    OnboardProfile(u8),
}

impl SettingChange {
    /// Whether `settings`, read back after applying, show the change in effect.
    pub fn took_effect(&self, settings: &DeviceSettings) -> bool {
        match self {
            SettingChange::Dpi(v) => settings.dpi.as_ref().is_some_and(|d| d.current == *v),
            SettingChange::ReportRateHz(v) => settings
                .report_rate
                .as_ref()
                .is_some_and(|r| r.current_hz == *v),
            SettingChange::OnboardProfile(i) => settings
                .onboard_profile
                .as_ref()
                .is_some_and(|p| p.current == *i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_serializes_as_tagged_value() {
        let json = serde_json::to_value(SettingChange::Dpi(1600)).unwrap();
        assert_eq!(json, serde_json::json!({ "type": "dpi", "value": 1600 }));
    }

    #[test]
    fn took_effect_compares_read_back_value() {
        let settings = DeviceSettings {
            dpi: Some(DpiSetting {
                current: 800,
                values: vec![800, 1600],
            }),
            ..Default::default()
        };
        assert!(SettingChange::Dpi(800).took_effect(&settings));
        assert!(!SettingChange::Dpi(1600).took_effect(&settings));
        assert!(!SettingChange::ReportRateHz(1000).took_effect(&settings));
    }
}
