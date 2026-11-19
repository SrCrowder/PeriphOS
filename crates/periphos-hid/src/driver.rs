use std::sync::Mutex;

use hidapi::{BusType, HidApi};
use periphos_core::device::{DeviceDriver, DeviceInfo, DriverInfo, Platform};
use periphos_core::{CoreError, CoreResult};

use crate::classify::{classify, Bus, HidInterface, DRIVER_ID};
use crate::database;

/// Identifies HID mice, keyboards, controllers and known receivers.
#[derive(Default)]
pub struct HidDriver {
    // Created on first scan; refreshed on every scan after that.
    api: Mutex<Option<HidApi>>,
}

impl HidDriver {
    pub fn new() -> Self {
        Self::default()
    }

    fn interfaces(&self) -> CoreResult<Vec<HidInterface>> {
        let mut guard = self
            .api
            .lock()
            .map_err(|_| CoreError::Device("HID state is unavailable".into()))?;
        let api = match guard.as_mut() {
            Some(api) => {
                api.refresh_devices()
                    .map_err(|e| CoreError::Device(e.to_string()))?;
                api
            }
            None => guard.insert(HidApi::new().map_err(|e| CoreError::Device(e.to_string()))?),
        };
        Ok(api
            .device_list()
            .map(|d| HidInterface {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                serial_number: d.serial_number().map(str::to_owned),
                manufacturer: d.manufacturer_string().map(str::to_owned),
                product: d.product_string().map(str::to_owned),
                usage_page: d.usage_page(),
                usage: d.usage(),
                interface_number: d.interface_number(),
                bus: match d.bus_type() {
                    BusType::Usb => Bus::Usb,
                    BusType::Bluetooth => Bus::Bluetooth,
                    _ => Bus::Other,
                },
            })
            .collect())
    }
}

impl DeviceDriver for HidDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            id: DRIVER_ID.into(),
            name: "Generic HID".into(),
            description: "Identifies USB and Bluetooth mice, keyboards and controllers. \
                          It reads device information only and never changes device settings."
                .into(),
            platforms: vec![Platform::Windows, Platform::Linux],
        }
    }

    fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>> {
        Ok(classify(&self.interfaces()?, database::bundled()))
    }
}
