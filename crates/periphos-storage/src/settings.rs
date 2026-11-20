use periphos_core::settings::AppSettings;
use periphos_core::{CoreError, CoreResult};
use rusqlite::{params, OptionalExtension};

use crate::{to_core, Storage};

const APP_SETTINGS_KEY: &str = "app";

impl Storage {
    /// Load application settings, falling back to defaults when none are
    /// stored or the stored value cannot be read.
    pub fn load_settings(&self) -> CoreResult<AppSettings> {
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [APP_SETTINGS_KEY],
                |r| r.get(0),
            )
            .optional()
            .map_err(to_core)?;
        Ok(raw
            .and_then(|json| serde_json::from_str::<AppSettings>(&json).ok())
            .unwrap_or_default()
            .normalized())
    }

    pub fn save_settings(&self, settings: &AppSettings) -> CoreResult<()> {
        let settings = &settings.clone().normalized();
        let json =
            serde_json::to_string(settings).map_err(|e| CoreError::Storage(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![APP_SETTINGS_KEY, json],
            )
            .map_err(to_core)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use periphos_core::settings::{Accent, Theme, UpdateChannel};

    #[test]
    fn defaults_when_empty_and_roundtrip() {
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(s.load_settings().unwrap(), AppSettings::default());

        let custom = AppSettings {
            theme: Theme::Light,
            accent: Accent::Cyan,
            reduce_motion: true,
            update_channel: Some(UpdateChannel::Development),
            auto_check_updates: false,
            minimize_to_tray: false,
            battery_alerts: false,
            battery_warning_percent: 25,
            battery_critical_percent: 5,
            battery_in_tray: false,
        };
        s.save_settings(&custom).unwrap();
        assert_eq!(s.load_settings().unwrap(), custom);
    }

    #[test]
    fn corrupted_value_falls_back_to_defaults() {
        let s = Storage::open_in_memory().unwrap();
        s.conn
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', 'not json')",
                [],
            )
            .unwrap();
        assert_eq!(s.load_settings().unwrap(), AppSettings::default());
    }
}
