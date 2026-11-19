use periphos_core::profile::{Profile, ProfileDraft};
use periphos_core::{CoreError, CoreResult};
use rusqlite::{params, OptionalExtension, Row};

use crate::{now_ms, to_core, Storage};

const DEFAULT_PROFILE_NAME: &str = "Default";
const COLUMNS: &str = "id, name, description, is_default, created_at, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Profile> {
    Ok(Profile {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        is_default: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

/// Translate SQLite constraint failures into user-facing errors.
fn map_write_err(err: rusqlite::Error, name: &str) -> CoreError {
    match &err {
        rusqlite::Error::SqliteFailure(e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            CoreError::Conflict(format!("A profile named \"{name}\" already exists."))
        }
        _ => to_core(err),
    }
}

impl Storage {
    /// Guarantee the invariant "exactly one default profile exists".
    pub(crate) fn ensure_default_profile(&self) -> CoreResult<()> {
        let has_default: bool = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM profiles WHERE is_default = 1)",
                [],
                |r| r.get(0),
            )
            .map_err(to_core)?;
        if has_default {
            return Ok(());
        }
        // Promote the oldest profile if any exist, otherwise create one.
        let promoted = self
            .conn
            .execute(
                "UPDATE profiles SET is_default = 1 WHERE id = \
                 (SELECT id FROM profiles ORDER BY created_at LIMIT 1)",
                [],
            )
            .map_err(to_core)?;
        if promoted == 0 {
            let now = now_ms();
            self.conn
                .execute(
                    "INSERT INTO profiles (id, name, description, is_default, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, 1, ?4, ?4)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        DEFAULT_PROFILE_NAME,
                        "Used when no application-specific profile applies.",
                        now
                    ],
                )
                .map_err(to_core)?;
        }
        Ok(())
    }

    /// All profiles, default first, then by name.
    pub fn list_profiles(&self) -> CoreResult<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM profiles ORDER BY is_default DESC, name COLLATE NOCASE"
            ))
            .map_err(to_core)?;
        let rows = stmt.query_map([], from_row).map_err(to_core)?;
        rows.collect::<Result<_, _>>().map_err(to_core)
    }

    pub fn get_profile(&self, id: &str) -> CoreResult<Profile> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM profiles WHERE id = ?1"),
                [id],
                from_row,
            )
            .optional()
            .map_err(to_core)?
            .ok_or_else(|| CoreError::NotFound {
                entity: "Profile",
                id: id.to_owned(),
            })
    }

    pub fn create_profile(&self, draft: ProfileDraft) -> CoreResult<Profile> {
        let draft = draft.normalized()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_ms();
        self.conn
            .execute(
                "INSERT INTO profiles (id, name, description, is_default, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, 0, ?4, ?4)",
                params![id, draft.name, draft.description, now],
            )
            .map_err(|e| map_write_err(e, &draft.name))?;
        self.get_profile(&id)
    }

    pub fn update_profile(&self, id: &str, draft: ProfileDraft) -> CoreResult<Profile> {
        let draft = draft.normalized()?;
        let changed = self
            .conn
            .execute(
                "UPDATE profiles SET name = ?2, description = ?3, updated_at = ?4 WHERE id = ?1",
                params![id, draft.name, draft.description, now_ms()],
            )
            .map_err(|e| map_write_err(e, &draft.name))?;
        if changed == 0 {
            return Err(CoreError::NotFound {
                entity: "Profile",
                id: id.to_owned(),
            });
        }
        self.get_profile(id)
    }

    /// Delete a profile. The default profile cannot be deleted; make another
    /// profile the default first.
    pub fn delete_profile(&self, id: &str) -> CoreResult<()> {
        let profile = self.get_profile(id)?;
        if profile.is_default {
            return Err(CoreError::Conflict(
                "The default profile cannot be deleted. Set another profile as default first."
                    .into(),
            ));
        }
        self.conn
            .execute("DELETE FROM profiles WHERE id = ?1", [id])
            .map_err(to_core)?;
        Ok(())
    }

    /// Make `id` the default profile, atomically clearing the previous one.
    pub fn set_default_profile(&mut self, id: &str) -> CoreResult<Profile> {
        self.get_profile(id)?;
        let tx = self.conn.transaction().map_err(to_core)?;
        tx.execute(
            "UPDATE profiles SET is_default = 0 WHERE is_default = 1",
            [],
        )
        .map_err(to_core)?;
        tx.execute(
            "UPDATE profiles SET is_default = 1, updated_at = ?2 WHERE id = ?1",
            params![id, now_ms()],
        )
        .map_err(to_core)?;
        tx.commit().map_err(to_core)?;
        self.get_profile(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(name: &str) -> ProfileDraft {
        ProfileDraft {
            name: name.into(),
            description: None,
        }
    }

    fn defaults(s: &Storage) -> Vec<Profile> {
        s.list_profiles()
            .unwrap()
            .into_iter()
            .filter(|p| p.is_default)
            .collect()
    }

    #[test]
    fn new_database_has_one_default_profile() {
        let s = Storage::open_in_memory().unwrap();
        let profiles = s.list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(profiles[0].is_default);
        assert_eq!(profiles[0].name, DEFAULT_PROFILE_NAME);
    }

    #[test]
    fn create_update_delete_roundtrip() {
        let s = Storage::open_in_memory().unwrap();
        let p = s
            .create_profile(ProfileDraft {
                name: " Valorant ".into(),
                description: Some("FPS".into()),
            })
            .unwrap();
        assert_eq!(p.name, "Valorant");
        assert!(!p.is_default);

        let updated = s.update_profile(&p.id, draft("Valorant Ranked")).unwrap();
        assert_eq!(updated.name, "Valorant Ranked");
        assert_eq!(updated.description, None);
        assert!(updated.updated_at >= p.updated_at);

        s.delete_profile(&p.id).unwrap();
        assert!(matches!(
            s.get_profile(&p.id),
            Err(CoreError::NotFound { .. })
        ));
    }

    #[test]
    fn names_are_unique_case_insensitively() {
        let s = Storage::open_in_memory().unwrap();
        s.create_profile(draft("Work")).unwrap();
        let err = s.create_profile(draft("work")).unwrap_err();
        assert_eq!(err.code(), "conflict");
    }

    #[test]
    fn default_profile_cannot_be_deleted() {
        let s = Storage::open_in_memory().unwrap();
        let default = defaults(&s).remove(0);
        assert_eq!(
            s.delete_profile(&default.id).unwrap_err().code(),
            "conflict"
        );
    }

    #[test]
    fn switching_default_keeps_exactly_one() {
        let mut s = Storage::open_in_memory().unwrap();
        let p = s.create_profile(draft("Gaming")).unwrap();
        s.set_default_profile(&p.id).unwrap();
        let d = defaults(&s);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].id, p.id);
    }

    #[test]
    fn updating_missing_profile_is_not_found() {
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(
            s.update_profile("nope", draft("x")).unwrap_err().code(),
            "not_found"
        );
    }
}
