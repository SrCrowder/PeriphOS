use periphos_core::apps::{AppIdentity, InstalledPackage};

pub fn installed_packages() -> Vec<InstalledPackage> {
    Vec::new()
}

pub fn foreground_app() -> Option<AppIdentity> {
    None
}

pub fn running_apps() -> Vec<AppIdentity> {
    Vec::new()
}

/// Process names from /proc on Linux; nothing elsewhere.
pub fn running_process_names() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .chars()
                .all(|c| c.is_ascii_digit())
        })
        .filter_map(|e| std::fs::read_to_string(e.path().join("comm")).ok())
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}
