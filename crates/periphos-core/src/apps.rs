//! Applications and games, and the rules that link them to profiles.

use serde::{Deserialize, Serialize};

/// A running application as seen by the operating system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppIdentity {
    /// File name of the executable, e.g. `ForzaHorizon5.exe`.
    pub exe_name: String,
    /// Full path, when the OS allows reading it.
    pub exe_path: Option<String>,
    /// Package Family Name for Microsoft Store / Game Pass apps
    /// (e.g. `Microsoft.624F8B84B80_8wekyb3d8bbwe`). Identifies the game even
    /// when its folder under WindowsApps cannot be browsed.
    pub package_family: Option<String>,
    /// Window title, for display.
    pub title: String,
}

impl AppIdentity {
    /// Name to show for this app: the window title, or the executable name.
    pub fn display_name(&self) -> String {
        if self.title.trim().is_empty() {
            self.exe_name.clone()
        } else {
            self.title.trim().to_owned()
        }
    }
}

/// A Microsoft Store / Game Pass package installed for the current user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPackage {
    pub package_family: String,
    pub display_name: String,
    pub install_path: Option<String>,
    /// True for PC games (their package carries a `MicrosoftGame.config`).
    pub is_game: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppRuleKind {
    /// Matches the executable's file name, case-insensitively, wherever it is installed.
    Executable,
    /// Matches a Microsoft Store / Game Pass Package Family Name.
    Package,
}

/// Links an application to a profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRule {
    pub id: String,
    pub profile_id: String,
    pub kind: AppRuleKind,
    pub value: String,
    pub display_name: String,
    /// Unix timestamp in milliseconds.
    pub created_at: i64,
}

impl AppRule {
    pub fn matches(&self, app: &AppIdentity) -> bool {
        match self.kind {
            AppRuleKind::Package => app
                .package_family
                .as_deref()
                .is_some_and(|family| family.eq_ignore_ascii_case(&self.value)),
            AppRuleKind::Executable => file_name(&self.value).eq_ignore_ascii_case(&app.exe_name),
        }
    }
}

/// Last path component, accepting both separators.
pub fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

/// The rule for `app`, preferring package rules (more specific) over executable ones.
pub fn find_rule<'a>(rules: &'a [AppRule], app: &AppIdentity) -> Option<&'a AppRule> {
    rules
        .iter()
        .filter(|r| r.matches(app))
        .min_by_key(|r| match r.kind {
            AppRuleKind::Package => 0,
            AppRuleKind::Executable => 1,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(kind: AppRuleKind, value: &str, profile: &str) -> AppRule {
        AppRule {
            id: value.into(),
            profile_id: profile.into(),
            kind,
            value: value.into(),
            display_name: value.into(),
            created_at: 0,
        }
    }

    fn app(exe: &str, family: Option<&str>) -> AppIdentity {
        AppIdentity {
            exe_name: exe.into(),
            exe_path: Some(format!("C:\\XboxGames\\Game\\Content\\{exe}")),
            package_family: family.map(Into::into),
            title: "Game".into(),
        }
    }

    #[test]
    fn executable_rules_match_file_name_case_insensitively() {
        let r = rule(
            AppRuleKind::Executable,
            "D:\\Steam\\steamapps\\common\\cs2\\cs2.exe",
            "fps",
        );
        assert!(r.matches(&app("CS2.EXE", None)));
        assert!(!r.matches(&app("cs.exe", None)));
    }

    #[test]
    fn package_rules_match_family_name() {
        let r = rule(
            AppRuleKind::Package,
            "Microsoft.624F8B84B80_8wekyb3d8bbwe",
            "racing",
        );
        assert!(r.matches(&app(
            "ForzaHorizon5.exe",
            Some("microsoft.624f8b84b80_8wekyb3d8bbwe")
        )));
        assert!(!r.matches(&app("ForzaHorizon5.exe", None)));
    }

    #[test]
    fn package_rule_wins_over_executable_rule() {
        let rules = vec![
            rule(AppRuleKind::Executable, "game.exe", "generic"),
            rule(AppRuleKind::Package, "Studio.Game_abc", "specific"),
        ];
        let found = find_rule(&rules, &app("game.exe", Some("Studio.Game_abc"))).unwrap();
        assert_eq!(found.profile_id, "specific");
        assert!(find_rule(&rules, &app("other.exe", None)).is_none());
    }

    #[test]
    fn display_name_falls_back_to_exe() {
        let mut a = app("game.exe", None);
        a.title = "  ".into();
        assert_eq!(a.display_name(), "game.exe");
    }
}
