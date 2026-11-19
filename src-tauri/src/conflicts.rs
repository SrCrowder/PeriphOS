//! Other software that also controls a device and may undo PeriphOS's changes.

/// Manufacturer apps, by the driver that handles their devices: the name shown
/// to the user and how their processes start (lower case).
const KNOWN: &[(&str, &str, &[&str])] = &[
    ("logitech-hidpp", "Logitech G HUB", &["lghub"]),
    ("wooting", "Wootility", &["wootility"]),
];

/// Names of the running apps that also control devices of `driver_id`.
pub fn running_for_driver(driver_id: &str) -> Vec<String> {
    let candidates: Vec<_> = KNOWN
        .iter()
        .filter(|(driver, _, _)| *driver == driver_id)
        .collect();
    if candidates.is_empty() {
        return Vec::new();
    }
    let processes = periphos_apps::running_process_names();
    candidates
        .into_iter()
        .filter(|(_, _, prefixes)| {
            processes
                .iter()
                .any(|p| prefixes.iter().any(|prefix| p.starts_with(prefix)))
        })
        .map(|(_, name, _)| (*name).to_owned())
        .collect()
}

/// The driver part of a device ID (`driver:...`).
pub fn driver_of(device_id: &str) -> &str {
    device_id.split(':').next().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_is_the_id_prefix() {
        assert_eq!(driver_of("wooting:1400:ABC"), "wooting");
        assert_eq!(driver_of("logitech-hidpp:c547:0:1"), "logitech-hidpp");
    }

    #[test]
    fn unknown_drivers_have_no_conflicts() {
        assert!(running_for_driver("hid-generic").is_empty());
    }
}
