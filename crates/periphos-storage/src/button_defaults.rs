use periphos_core::device::{ButtonAction, ButtonDefaults};
use periphos_core::{CoreError, CoreResult};
use rusqlite::{params, OptionalExtension};

use crate::{now_ms, to_core, Storage};

impl Storage {
    /// The button assignments saved as this device's defaults, if any.
    pub fn button_defaults(&self, device_id: &str) -> CoreResult<Option<ButtonDefaults>> {
        let row = self
            .conn
            .query_row(
                "SELECT device_id, device_name, actions_json, updated_at \
                 FROM device_button_defaults WHERE device_id = ?1",
                [device_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(to_core)?;
        let Some((device_id, device_name, json, updated_at)) = row else {
            return Ok(None);
        };
        // Saved by a newer version in a shape this one cannot read: ignore it.
        let Ok(actions) = serde_json::from_str::<Vec<ButtonAction>>(&json) else {
            return Ok(None);
        };
        Ok(Some(ButtonDefaults {
            device_id,
            device_name,
            actions,
            updated_at,
        }))
    }

    pub fn save_button_defaults(
        &self,
        device_id: &str,
        device_name: &str,
        actions: &[ButtonAction],
    ) -> CoreResult<ButtonDefaults> {
        if actions.is_empty() {
            return Err(CoreError::Validation(
                "There are no buttons to save.".into(),
            ));
        }
        let json = serde_json::to_string(actions).map_err(|e| CoreError::Storage(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO device_button_defaults (device_id, device_name, actions_json, updated_at) \
                 VALUES (?1, ?2, ?3, ?4) \
                 ON CONFLICT(device_id) DO UPDATE SET device_name = excluded.device_name, \
                 actions_json = excluded.actions_json, updated_at = excluded.updated_at",
                params![device_id, device_name, json, now_ms()],
            )
            .map_err(to_core)?;
        self.button_defaults(device_id)?
            .ok_or_else(|| CoreError::Storage("the saved defaults could not be read back".into()))
    }

    pub fn delete_button_defaults(&self, device_id: &str) -> CoreResult<()> {
        self.conn
            .execute(
                "DELETE FROM device_button_defaults WHERE device_id = ?1",
                [device_id],
            )
            .map_err(to_core)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use periphos_core::device::MouseButton;

    use crate::Storage;

    use super::*;

    #[test]
    fn saves_replaces_and_deletes_defaults() {
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(s.button_defaults("m").unwrap(), None);
        let left = ButtonAction::Mouse {
            button: MouseButton::Left,
        };
        s.save_button_defaults("m", "Mouse", std::slice::from_ref(&left))
            .unwrap();
        let saved = s
            .save_button_defaults("m", "Mouse", &[left.clone(), ButtonAction::Disabled])
            .unwrap();
        assert_eq!(saved.actions, vec![left, ButtonAction::Disabled]);
        s.delete_button_defaults("m").unwrap();
        assert_eq!(s.button_defaults("m").unwrap(), None);
        assert!(s.save_button_defaults("m", "Mouse", &[]).is_err());
    }
}
