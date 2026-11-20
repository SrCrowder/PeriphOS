//! Low-battery notifications for wireless devices.

use std::collections::HashMap;

use periphos_core::device::{BatteryLevel, ChargingState, DeviceInfo};
use periphos_core::settings::AppSettings;

/// Points above the warning level the battery must reach before a new alert
/// can be sent, so readings that hover around the threshold do not repeat it.
const REARM_MARGIN: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    Low,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub device_name: String,
    pub level: AlertLevel,
    pub percent: Option<u8>,
}

impl Alert {
    pub fn title(&self) -> String {
        match self.level {
            AlertLevel::Low => format!("{} battery is low", self.device_name),
            AlertLevel::Critical => format!("{} battery is almost empty", self.device_name),
        }
    }

    pub fn body(&self) -> String {
        let left = self
            .percent
            .map(|p| format!("{p}% left. "))
            .unwrap_or_default();
        match self.level {
            AlertLevel::Low => format!("{left}Charge it soon."),
            AlertLevel::Critical => format!("{left}Charge it now to keep using it."),
        }
    }
}

/// Remembers which alert each device already got.
#[derive(Default)]
pub struct BatteryAlerts {
    notified: HashMap<String, AlertLevel>,
}

impl BatteryAlerts {
    /// Alerts to send for this scan. Each level is sent once per discharge;
    /// charging, or recovering past the warning level plus a margin, re-arms it.
    pub fn check(&mut self, devices: &[DeviceInfo], settings: &AppSettings) -> Vec<Alert> {
        let mut alerts = Vec::new();
        for device in devices {
            let Some(battery) = device.battery else {
                continue;
            };
            if battery.charging != ChargingState::Discharging {
                self.notified.remove(&device.id);
                continue;
            }
            let level = match battery.percent {
                Some(p) if p <= settings.battery_critical_percent => Some(AlertLevel::Critical),
                Some(p) if p <= settings.battery_warning_percent => Some(AlertLevel::Low),
                Some(p) => {
                    if p > settings
                        .battery_warning_percent
                        .saturating_add(REARM_MARGIN)
                    {
                        self.notified.remove(&device.id);
                    }
                    None
                }
                None => match battery.level {
                    Some(BatteryLevel::Critical | BatteryLevel::Empty) => {
                        Some(AlertLevel::Critical)
                    }
                    Some(BatteryLevel::Low) => Some(AlertLevel::Low),
                    Some(BatteryLevel::Good | BatteryLevel::Full) => {
                        self.notified.remove(&device.id);
                        None
                    }
                    None => None,
                },
            };
            let Some(level) = level else { continue };
            if self
                .notified
                .get(&device.id)
                .is_some_and(|&sent| sent >= level)
            {
                continue;
            }
            self.notified.insert(device.id.clone(), level);
            if settings.battery_alerts {
                alerts.push(Alert {
                    device_name: device.name.clone(),
                    level,
                    percent: battery.percent,
                });
            }
        }
        alerts
    }
}

#[cfg(test)]
mod tests {
    use periphos_core::device::{BatteryState, ConnectionType, DeviceKind};

    use super::*;

    fn mouse(
        percent: Option<u8>,
        level: Option<BatteryLevel>,
        charging: ChargingState,
    ) -> DeviceInfo {
        DeviceInfo {
            id: "m".into(),
            driver_id: "x".into(),
            name: "G502 X PLUS".into(),
            manufacturer: "Logitech".into(),
            vendor_id: 0,
            product_id: 0,
            kind: DeviceKind::Mouse,
            connection: ConnectionType::Receiver,
            capabilities: vec![],
            known_model: true,
            interface_count: 0,
            note: None,
            battery: Some(BatteryState {
                percent,
                level,
                charging,
                estimated: false,
            }),
            connected_via: None,
        }
    }

    fn at(p: u8) -> Vec<DeviceInfo> {
        vec![mouse(Some(p), None, ChargingState::Discharging)]
    }

    #[test]
    fn each_level_alerts_once_while_discharging() {
        let s = AppSettings::default(); // 20 % and 10 %
        let mut a = BatteryAlerts::default();
        assert!(a.check(&at(50), &s).is_empty());
        let low = a.check(&at(20), &s);
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].level, AlertLevel::Low);
        assert!(a.check(&at(18), &s).is_empty());
        assert!(a.check(&at(21), &s).is_empty()); // hovering: no repeat
        assert!(a.check(&at(19), &s).is_empty());
        let critical = a.check(&at(9), &s);
        assert_eq!(critical[0].level, AlertLevel::Critical);
        assert_eq!(
            critical[0].body(),
            "9% left. Charge it now to keep using it."
        );
        assert!(a.check(&at(5), &s).is_empty());
    }

    #[test]
    fn charging_or_recovering_re_arms_alerts() {
        let s = AppSettings::default();
        let mut a = BatteryAlerts::default();
        assert_eq!(a.check(&at(15), &s).len(), 1);
        a.check(&[mouse(Some(16), None, ChargingState::Charging)], &s);
        assert_eq!(a.check(&at(15), &s).len(), 1);
        a.check(&at(30), &s); // above 20 + 5
        assert_eq!(a.check(&at(20), &s).len(), 1);
    }

    #[test]
    fn level_only_devices_alert_too() {
        let s = AppSettings::default();
        let mut a = BatteryAlerts::default();
        let alerts = a.check(
            &[mouse(
                None,
                Some(BatteryLevel::Critical),
                ChargingState::Discharging,
            )],
            &s,
        );
        assert_eq!(alerts[0].level, AlertLevel::Critical);
        assert_eq!(alerts[0].body(), "Charge it now to keep using it.");
    }

    #[test]
    fn disabled_alerts_send_nothing_and_do_not_fire_later() {
        let off = AppSettings {
            battery_alerts: false,
            ..Default::default()
        };
        let mut a = BatteryAlerts::default();
        assert!(a.check(&at(15), &off).is_empty());
        // Turning alerts on later does not replay an alert for the same discharge.
        assert!(a.check(&at(14), &AppSettings::default()).is_empty());
    }
}
