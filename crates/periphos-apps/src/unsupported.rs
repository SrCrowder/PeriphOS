use periphos_core::apps::AppIdentity;

pub fn foreground_app() -> Option<AppIdentity> {
    None
}

pub fn running_apps() -> Vec<AppIdentity> {
    Vec::new()
}
