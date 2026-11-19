use serde::{Deserialize, Serialize};

use super::driver::{DeviceDriver, DriverInfo, Platform};
use super::info::DeviceInfo;

/// Result of a device scan across every registered driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceOverview {
    pub platform: Platform,
    pub drivers: Vec<DriverInfo>,
    pub devices: Vec<DeviceInfo>,
    /// Drivers that failed during the scan, with their error message.
    pub driver_errors: Vec<DriverError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverError {
    pub driver_id: String,
    pub message: String,
}

/// Registry of drivers and entry point for device discovery.
#[derive(Default)]
pub struct DeviceManager {
    drivers: Vec<Box<dyn DeviceDriver>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a driver. Drivers whose platform list does not include the
    /// current platform are ignored.
    pub fn register(&mut self, driver: Box<dyn DeviceDriver>) {
        if driver.info().platforms.contains(&Platform::current()) {
            self.drivers.push(driver);
        }
    }

    pub fn drivers(&self) -> Vec<DriverInfo> {
        self.drivers.iter().map(|d| d.info()).collect()
    }

    /// Ask every driver for its devices. A failing driver never prevents the
    /// others from reporting; its error is collected instead.
    pub fn scan(&self) -> DeviceOverview {
        let mut devices = Vec::new();
        let mut driver_errors = Vec::new();
        for driver in &self.drivers {
            match driver.enumerate() {
                Ok(found) => devices.extend(found),
                Err(err) => driver_errors.push(DriverError {
                    driver_id: driver.info().id,
                    message: err.to_string(),
                }),
            }
        }
        DeviceOverview {
            platform: Platform::current(),
            drivers: self.drivers(),
            devices,
            driver_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{ConnectionType, DeviceKind};
    use crate::{CoreError, CoreResult};

    struct StubDriver {
        id: &'static str,
        platforms: Vec<Platform>,
        fail: bool,
    }

    impl DeviceDriver for StubDriver {
        fn info(&self) -> DriverInfo {
            DriverInfo {
                id: self.id.into(),
                name: self.id.into(),
                description: String::new(),
                platforms: self.platforms.clone(),
            }
        }

        fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>> {
            if self.fail {
                return Err(CoreError::Device("boom".into()));
            }
            Ok(vec![DeviceInfo {
                id: format!("{}:1", self.id),
                driver_id: self.id.into(),
                name: "Stub".into(),
                manufacturer: "Test".into(),
                vendor_id: 1,
                product_id: 2,
                kind: DeviceKind::Mouse,
                connection: ConnectionType::Usb,
                capabilities: vec![],
            }])
        }
    }

    fn stub(id: &'static str, fail: bool) -> Box<StubDriver> {
        Box::new(StubDriver {
            id,
            platforms: vec![Platform::current()],
            fail,
        })
    }

    #[test]
    fn empty_manager_reports_nothing() {
        let overview = DeviceManager::new().scan();
        assert!(overview.drivers.is_empty());
        assert!(overview.devices.is_empty());
        assert!(overview.driver_errors.is_empty());
    }

    #[test]
    fn failing_driver_does_not_hide_others() {
        let mut manager = DeviceManager::new();
        manager.register(stub("ok", false));
        manager.register(stub("broken", true));
        let overview = manager.scan();
        assert_eq!(overview.devices.len(), 1);
        assert_eq!(overview.driver_errors.len(), 1);
        assert_eq!(overview.driver_errors[0].driver_id, "broken");
    }

    #[test]
    fn drivers_for_other_platforms_are_skipped() {
        let other = if Platform::current() == Platform::Linux {
            Platform::Windows
        } else {
            Platform::Linux
        };
        let mut manager = DeviceManager::new();
        manager.register(Box::new(StubDriver {
            id: "x",
            platforms: vec![other],
            fail: false,
        }));
        assert!(manager.drivers().is_empty());
    }
}
