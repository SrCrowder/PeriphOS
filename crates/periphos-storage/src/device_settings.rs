use periphos_core::profile::DeviceProfileSettings;
use periphos_core::{CoreError, CoreResult};
use rusqlite::{params, Row};

use crate::{now_ms, to_core, Storage};

fn from_row(row: &Row<'_>) -> rusqlite::Result<DeviceProfileSettings> {
    Ok(DeviceProfileSettings {
        device_id: row.get(0)?,
        device_name: row.get(1)?,
        dpi: row.get(2)?,
        report_rate_hz: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

impl Storage {
    /// Device settings saved in a profile, by device name.
    pub fn profile_device_settings(
        &self,
        profile_id: &str,
    ) -> CoreResult<Vec<DeviceProfileSettings>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT device_id, device_name, dpi, report_rate_hz, updated_at \
                 FROM profile_device_settings WHERE profile_id = ?1 ORDER BY device_name COLLATE NOCASE",
            )
            .map_err(to_core)?;
        let rows = stmt.query_map([profile_id], from_row).map_err(to_core)?;
        rows.collect::<Result<_, _>>().map_err(to_core)
    }

    /// Create or replace a device's settings in a profile.
    pub fn save_profile_device_settings(
        &self,
        profile_id: &str,
        settings: &DeviceProfileSettings,
    ) -> CoreResult<DeviceProfileSettings> {
        self.get_profile(profile_id)?;
        if settings.is_empty() {
            return Err(CoreError::Validation(
                "There are no settings to save for this device.".into(),
            ));
        }
        let now = now_ms();
        self.conn
            .execute(
                "INSERT INTO profile_device_settings \
                 (profile_id, device_id, device_name, dpi, report_rate_hz, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                 ON CONFLICT(profile_id, device_id) DO UPDATE SET \
                 device_name = excluded.device_name, dpi = excluded.dpi, \
                 report_rate_hz = excluded.report_rate_hz, updated_at = excluded.updated_at",
                params![
                    profile_id,
                    settings.device_id,
                    settings.device_name,
                    settings.dpi,
                    settings.report_rate_hz,
                    now
                ],
            )
            .map_err(to_core)?;
        Ok(DeviceProfileSettings {
            updated_at: now,
            ..settings.clone()
        })
    }

    pub fn delete_profile_device_settings(
        &self,
        profile_id: &str,
        device_id: &str,
    ) -> CoreResult<()> {
        self.conn
            .execute(
                "DELETE FROM profile_device_settings WHERE profile_id = ?1 AND device_id = ?2",
                params![profile_id, device_id],
            )
            .map_err(to_core)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use periphos_core::profile::ProfileDraft;

    use super::*;

    fn mouse(dpi: u32) -> DeviceProfileSettings {
        DeviceProfileSettings {
            device_id: "logitech-hidpp:c547:0:1".into(),
            device_name: "G502 X PLUS".into(),
            dpi: Some(dpi),
            report_rate_hz: Some(1000),
            updated_at: 0,
        }
    }

    #[test]
    fn save_replace_and_delete() {
        let s = Storage::open_in_memory().unwrap();
        let p = s
            .create_profile(ProfileDraft {
                name: "FPS".into(),
                description: None,
            })
            .unwrap();
        s.save_profile_device_settings(&p.id, &mouse(800)).unwrap();
        s.save_profile_device_settings(&p.id, &mouse(1600)).unwrap();
        let saved = s.profile_device_settings(&p.id).unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].dpi, Some(1600));
        assert!(saved[0].updated_at > 0);

        s.delete_profile_device_settings(&p.id, &saved[0].device_id)
            .unwrap();
        assert!(s.profile_device_settings(&p.id).unwrap().is_empty());
    }

    #[test]
    fn deleting_a_profile_deletes_its_device_settings() {
        let s = Storage::open_in_memory().unwrap();
        let p = s
            .create_profile(ProfileDraft {
                name: "FPS".into(),
                description: None,
            })
            .unwrap();
        s.save_profile_device_settings(&p.id, &mouse(800)).unwrap();
        s.delete_profile(&p.id).unwrap();
        let count: i64 = s
            .conn
            .query_row("SELECT COUNT(*) FROM profile_device_settings", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn rejects_unknown_profile_and_empty_settings() {
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(
            s.save_profile_device_settings("nope", &mouse(800))
                .unwrap_err()
                .code(),
            "not_found"
        );
        let default = s.list_profiles().unwrap().remove(0);
        let mut empty = mouse(800);
        empty.dpi = None;
        empty.report_rate_hz = None;
        assert_eq!(
            s.save_profile_device_settings(&default.id, &empty)
                .unwrap_err()
                .code(),
            "validation"
        );
    }
}
