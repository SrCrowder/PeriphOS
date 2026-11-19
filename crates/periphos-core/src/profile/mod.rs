//! Configuration profiles.
//!
//! A profile groups every per-device setting (mappings, macros, lighting,
//! ...) that should be active together. Phase 1 only models the profile
//! itself; device settings and application associations are added in later
//! phases.

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

pub const NAME_MAX_LEN: usize = 64;
pub const DESCRIPTION_MAX_LEN: usize = 280;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// The fallback profile used when no application-specific profile
    /// matches. Exactly one profile is the default at any time.
    pub is_default: bool,
    /// Unix timestamp in milliseconds.
    pub created_at: i64,
    /// Unix timestamp in milliseconds.
    pub updated_at: i64,
}

/// Input for creating or editing a profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDraft {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl ProfileDraft {
    /// Trim and validate user input, returning the normalized draft.
    pub fn normalized(self) -> CoreResult<Self> {
        let name = self.name.trim().to_owned();
        if name.is_empty() {
            return Err(CoreError::Validation(
                "Profile name cannot be empty.".into(),
            ));
        }
        if name.chars().count() > NAME_MAX_LEN {
            return Err(CoreError::Validation(format!(
                "Profile name cannot be longer than {NAME_MAX_LEN} characters."
            )));
        }
        let description = self
            .description
            .map(|d| d.trim().to_owned())
            .filter(|d| !d.is_empty());
        if let Some(d) = &description {
            if d.chars().count() > DESCRIPTION_MAX_LEN {
                return Err(CoreError::Validation(format!(
                    "Description cannot be longer than {DESCRIPTION_MAX_LEN} characters."
                )));
            }
        }
        Ok(Self { name, description })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(name: &str, description: Option<&str>) -> ProfileDraft {
        ProfileDraft {
            name: name.into(),
            description: description.map(Into::into),
        }
    }

    #[test]
    fn trims_and_drops_blank_description() {
        let d = draft("  FPS  ", Some("   ")).normalized().unwrap();
        assert_eq!(d.name, "FPS");
        assert_eq!(d.description, None);
    }

    #[test]
    fn rejects_empty_and_long_names() {
        assert!(draft("   ", None).normalized().is_err());
        assert!(draft(&"a".repeat(NAME_MAX_LEN + 1), None)
            .normalized()
            .is_err());
        assert!(draft(&"a".repeat(NAME_MAX_LEN), None).normalized().is_ok());
    }

    #[test]
    fn rejects_long_description() {
        let long = "x".repeat(DESCRIPTION_MAX_LEN + 1);
        assert!(draft("ok", Some(&long)).normalized().is_err());
    }
}
