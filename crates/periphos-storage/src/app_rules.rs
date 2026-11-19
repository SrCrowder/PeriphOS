use periphos_core::apps::{file_name, AppRule, AppRuleKind};
use periphos_core::{CoreError, CoreResult};
use rusqlite::{params, OptionalExtension, Row};

use crate::{now_ms, to_core, Storage};

const COLUMNS: &str = "id, profile_id, kind, value, display_name, created_at";

fn kind_to_str(kind: AppRuleKind) -> &'static str {
    match kind {
        AppRuleKind::Executable => "executable",
        AppRuleKind::Package => "package",
    }
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<AppRule> {
    let kind: String = row.get(2)?;
    Ok(AppRule {
        id: row.get(0)?,
        profile_id: row.get(1)?,
        kind: if kind == "package" {
            AppRuleKind::Package
        } else {
            AppRuleKind::Executable
        },
        value: row.get(3)?,
        display_name: row.get(4)?,
        created_at: row.get(5)?,
    })
}

impl Storage {
    /// Every rule, for matching the foreground application.
    pub fn app_rules(&self) -> CoreResult<Vec<AppRule>> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM app_rules ORDER BY display_name COLLATE NOCASE"
            ))
            .map_err(to_core)?;
        let rows = stmt.query_map([], from_row).map_err(to_core)?;
        rows.collect::<Result<_, _>>().map_err(to_core)
    }

    pub fn profile_app_rules(&self, profile_id: &str) -> CoreResult<Vec<AppRule>> {
        Ok(self
            .app_rules()?
            .into_iter()
            .filter(|r| r.profile_id == profile_id)
            .collect())
    }

    /// Links an application to a profile. An application can belong to one profile only.
    pub fn add_app_rule(
        &self,
        profile_id: &str,
        kind: AppRuleKind,
        value: &str,
        display_name: &str,
    ) -> CoreResult<AppRule> {
        let profile = self.get_profile(profile_id)?;
        if profile.is_default {
            return Err(CoreError::Validation(
                "The default profile is used when no linked app is running, so apps cannot be linked to it."
                    .into(),
            ));
        }
        let value = value.trim();
        if value.is_empty()
            || (kind == AppRuleKind::Executable
                && !file_name(value).to_ascii_lowercase().ends_with(".exe"))
        {
            return Err(CoreError::Validation(
                "Choose an .exe file or an installed app.".into(),
            ));
        }
        let display_name = match display_name.trim() {
            "" => file_name(value).to_owned(),
            name => name.to_owned(),
        };

        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT p.name FROM app_rules r JOIN profiles p ON p.id = r.profile_id \
                 WHERE r.kind = ?1 AND r.value = ?2",
                params![kind_to_str(kind), value],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(to_core)?
        {
            return Err(CoreError::Conflict(format!(
                "{display_name} is already linked to \"{existing}\"."
            )));
        }

        let rule = AppRule {
            id: uuid::Uuid::new_v4().to_string(),
            profile_id: profile.id,
            kind,
            value: value.to_owned(),
            display_name,
            created_at: now_ms(),
        };
        self.conn
            .execute(
                &format!("INSERT INTO app_rules ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"),
                params![
                    rule.id,
                    rule.profile_id,
                    kind_to_str(rule.kind),
                    rule.value,
                    rule.display_name,
                    rule.created_at
                ],
            )
            .map_err(to_core)?;
        Ok(rule)
    }

    pub fn delete_app_rule(&self, id: &str) -> CoreResult<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM app_rules WHERE id = ?1", [id])
            .map_err(to_core)?;
        if deleted == 0 {
            return Err(CoreError::NotFound {
                entity: "App link",
                id: id.to_owned(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use periphos_core::profile::ProfileDraft;

    use super::*;

    fn storage_with_profile() -> (Storage, String) {
        let s = Storage::open_in_memory().unwrap();
        let p = s
            .create_profile(ProfileDraft {
                name: "Racing".into(),
                description: None,
            })
            .unwrap();
        (s, p.id)
    }

    #[test]
    fn add_list_and_delete() {
        let (s, profile) = storage_with_profile();
        let rule = s
            .add_app_rule(
                &profile,
                AppRuleKind::Package,
                "Microsoft.624F8B84B80_8wekyb3d8bbwe",
                "Forza Horizon 5",
            )
            .unwrap();
        assert_eq!(s.profile_app_rules(&profile).unwrap(), vec![rule.clone()]);
        s.delete_app_rule(&rule.id).unwrap();
        assert!(s.app_rules().unwrap().is_empty());
        assert_eq!(s.delete_app_rule(&rule.id).unwrap_err().code(), "not_found");
    }

    #[test]
    fn an_app_belongs_to_one_profile() {
        let (s, profile) = storage_with_profile();
        let other = s
            .create_profile(ProfileDraft {
                name: "FPS".into(),
                description: None,
            })
            .unwrap();
        s.add_app_rule(&profile, AppRuleKind::Executable, "C:\\Games\\game.exe", "")
            .unwrap();
        let err = s
            .add_app_rule(
                &other.id,
                AppRuleKind::Executable,
                "c:\\games\\GAME.exe",
                "Game",
            )
            .unwrap_err();
        assert_eq!(err.code(), "conflict");
        assert!(err.to_string().contains("Racing"));
    }

    #[test]
    fn display_name_defaults_to_file_name() {
        let (s, profile) = storage_with_profile();
        let r = s
            .add_app_rule(&profile, AppRuleKind::Executable, "D:\\Steam\\cs2.exe", " ")
            .unwrap();
        assert_eq!(r.display_name, "cs2.exe");
    }

    #[test]
    fn rejects_default_profile_and_non_executables() {
        let (s, profile) = storage_with_profile();
        let default = s
            .list_profiles()
            .unwrap()
            .into_iter()
            .find(|p| p.is_default)
            .unwrap();
        let err = s
            .add_app_rule(&default.id, AppRuleKind::Executable, "a.exe", "")
            .unwrap_err();
        assert_eq!(err.code(), "validation");
        let err = s
            .add_app_rule(&profile, AppRuleKind::Executable, "notes.txt", "")
            .unwrap_err();
        assert_eq!(err.code(), "validation");
    }

    #[test]
    fn rules_are_removed_with_their_profile() {
        let (s, profile) = storage_with_profile();
        s.add_app_rule(&profile, AppRuleKind::Executable, "game.exe", "")
            .unwrap();
        s.delete_profile(&profile).unwrap();
        assert!(s.app_rules().unwrap().is_empty());
    }
}
