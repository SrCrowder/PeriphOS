use std::ffi::CString;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use hidapi::{BusType, HidApi, HidDevice};
use periphos_core::device::{
    Capability, CapabilityKind, DeviceDriver, DeviceInfo, DeviceSettings, DriverInfo,
    OnboardProfileSetting, Platform, SettingChange, SupportLevel,
};
use periphos_core::{CoreError, CoreResult};
use periphos_hid::classify::{classify, Bus, HidInterface};
use periphos_hid::database;

use crate::protocol::{self, command, Interface, PROFILE_COUNT};

pub const WOOTING_VID: u16 = 0x31E3;
pub const DRIVER_ID: &str = "wooting";
const READ_TIMEOUT_MS: i32 = 1000;
/// Pause between profile commands; the keyboard needs time to load a profile.
const COMMAND_GAP: Duration = Duration::from_millis(250);

const PROFILES_NOTE: &str =
    "Switches between the four profiles stored on the keyboard. Actuation point, rapid \
     trigger, key bindings and lighting are set up per profile in Wootility.";

/// Lists Wooting keyboards and switches their onboard profile.
#[derive(Default)]
pub struct WootingDriver {
    /// Serialises access to the configuration interface.
    io: Mutex<()>,
}

impl WootingDriver {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A Wooting keyboard as enumerated by hidapi.
struct Keyboard {
    info: DeviceInfo,
    config: Option<(Interface, CString)>,
}

fn base_pid(pid: u16) -> u16 {
    // The last hex digit changes with the keyboard's controller emulation mode.
    pid & 0xFFF0
}

fn device_id(pid: u16, serial: &str) -> String {
    format!(
        "{DRIVER_ID}:{:04x}:{}",
        base_pid(pid),
        if serial.is_empty() { "0" } else { serial }
    )
}

fn keyboards(api: &HidApi) -> Vec<Keyboard> {
    let wooting: Vec<_> = api
        .device_list()
        .filter(|d| d.vendor_id() == WOOTING_VID)
        .collect();
    let interfaces: Vec<HidInterface> = wooting
        .iter()
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
                BusType::I2c | BusType::Spi => Bus::Internal,
                BusType::Unknown => Bus::Unknown,
            },
        })
        .collect();

    classify(&interfaces, database::bundled())
        .into_iter()
        .map(|mut info| {
            let serial = info.id.rsplit(':').next().unwrap_or("0").to_owned();
            let serial = if serial == "0" { String::new() } else { serial };
            let config = wooting
                .iter()
                .filter(|d| {
                    d.product_id() == info.product_id && d.serial_number().unwrap_or_default().trim() == serial
                })
                .find_map(|d| Interface::from_usage_page(d.usage_page()).map(|i| (i, d.path().to_owned())));

            info.id = device_id(info.product_id, &serial);
            info.driver_id = DRIVER_ID.into();
            if config.is_some() {
                upgrade_capabilities(&mut info.capabilities);
                info.note = Some(
                    "PeriphOS switches between the profiles stored on this keyboard. Set them up in Wootility, \
                     then close Wootility while PeriphOS manages the keyboard."
                        .into(),
                );
            }
            Keyboard { info, config }
        })
        .collect()
}

fn upgrade_capabilities(caps: &mut Vec<Capability>) {
    match caps
        .iter_mut()
        .find(|c| matches!(c.kind, CapabilityKind::OnboardProfiles { .. }))
    {
        Some(cap) => {
            cap.support = SupportLevel::Native;
            cap.note = Some(PROFILES_NOTE.into());
        }
        None => caps.push(
            Capability::new(
                CapabilityKind::OnboardProfiles {
                    slots: PROFILE_COUNT,
                },
                SupportLevel::Native,
            )
            .with_note(PROFILES_NOTE),
        ),
    }
}

/// An open configuration interface.
struct Config {
    device: HidDevice,
    interface: Interface,
}

impl Config {
    /// Sends a command and returns the complete response.
    fn command(&self, command: u8, parameter0: u8) -> CoreResult<Vec<u8>> {
        // Drop responses left over from earlier commands (for example Wootility's).
        let mut scratch = vec![0u8; self.interface.response_size()];
        for _ in 0..32 {
            if self.device.read_timeout(&mut scratch, 0).unwrap_or(0) == 0 {
                break;
            }
        }

        let report = protocol::encode(self.interface, command, parameter0);
        self.device
            .send_feature_report(&report)
            .map_err(|e| CoreError::Device(e.to_string()))?;

        let size = self.interface.response_size();
        let mut response = vec![0u8; size];
        let mut filled = 0;
        while filled < size {
            let n = self
                .device
                .read_timeout(&mut response[filled..], READ_TIMEOUT_MS)
                .map_err(|e| CoreError::Device(e.to_string()))?;
            if n == 0 {
                return Err(CoreError::Device(
                    "The keyboard did not answer. Close Wootility and try again.".into(),
                ));
            }
            filled += n;
            if self.interface.single_read() {
                break;
            }
        }
        response.truncate(filled);
        Ok(response)
    }

    fn current_profile(&self) -> CoreResult<u8> {
        let response = self.command(command::GET_CURRENT_PROFILE_INDEX, 0)?;
        protocol::profile_index(self.interface, &response)
            .ok_or_else(|| CoreError::Device("The keyboard sent an unexpected answer.".into()))
    }

    fn activate_profile(&self, index: u8) -> CoreResult<()> {
        self.command(command::ACTIVATE_PROFILE, index)?;
        thread::sleep(COMMAND_GAP);
        self.command(command::RELOAD_PROFILE, index)?;
        // Make the lighting follow the newly loaded profile.
        thread::sleep(COMMAND_GAP);
        self.command(command::RESET_ALL_COLORS, 0)?;
        thread::sleep(COMMAND_GAP);
        self.command(command::REFRESH_RGB_COLORS, index)?;
        Ok(())
    }

    fn settings(&self) -> CoreResult<DeviceSettings> {
        Ok(DeviceSettings {
            onboard_profile: Some(OnboardProfileSetting {
                current: self.current_profile()?,
                count: PROFILE_COUNT,
            }),
            ..Default::default()
        })
    }
}

impl WootingDriver {
    fn open(&self, device_id: &str) -> CoreResult<Config> {
        let api = HidApi::new().map_err(|e| CoreError::Device(e.to_string()))?;
        let keyboard = keyboards(&api)
            .into_iter()
            .find(|k| k.info.id == device_id)
            .ok_or_else(|| CoreError::Device("The keyboard is not connected.".into()))?;
        let (interface, path) = keyboard.config.ok_or_else(|| {
            CoreError::Unsupported("this keyboard's configuration interface was not found".into())
        })?;
        let device = api.open_path(&path).map_err(|e| {
            CoreError::Device(format!(
                "Could not open the keyboard ({e}). Close Wootility and try again."
            ))
        })?;
        Ok(Config { device, interface })
    }
}

impl DeviceDriver for WootingDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            id: DRIVER_ID.into(),
            name: "Wooting".into(),
            description:
                "Lists Wooting keyboards and switches between the profiles stored on them.".into(),
            platforms: vec![Platform::Windows, Platform::Linux],
        }
    }

    fn enumerate(&self) -> CoreResult<Vec<DeviceInfo>> {
        let api = HidApi::new().map_err(|e| CoreError::Device(e.to_string()))?;
        Ok(keyboards(&api).into_iter().map(|k| k.info).collect())
    }

    fn read_settings(&self, device_id: &str) -> CoreResult<DeviceSettings> {
        let _io = self
            .io
            .lock()
            .map_err(|_| CoreError::Device("keyboard access is unavailable".into()))?;
        self.open(device_id)?.settings()
    }

    fn apply_setting(&self, device_id: &str, change: SettingChange) -> CoreResult<DeviceSettings> {
        let SettingChange::OnboardProfile(index) = change else {
            return Err(CoreError::Unsupported(
                "Wooting keyboards only support switching profiles here".into(),
            ));
        };
        if index >= PROFILE_COUNT {
            return Err(CoreError::Validation(format!(
                "Choose a keyboard profile from 1 to {PROFILE_COUNT}."
            )));
        }
        let _io = self
            .io
            .lock()
            .map_err(|_| CoreError::Device("keyboard access is unavailable".into()))?;
        let config = self.open(device_id)?;
        config.activate_profile(index)?;
        config.settings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_ignore_the_controller_mode_digit() {
        assert_eq!(device_id(0x1402, "A1B2"), "wooting:1400:A1B2");
        assert_eq!(device_id(0x1400, ""), "wooting:1400:0");
        assert_eq!(device_id(0x1401, "A1B2"), device_id(0x1402, "A1B2"));
    }

    #[test]
    fn onboard_profiles_become_native() {
        let mut caps = vec![Capability::new(
            CapabilityKind::KeyRemap,
            SupportLevel::VendorProtocol,
        )];
        upgrade_capabilities(&mut caps);
        let profiles = caps
            .iter()
            .find(|c| matches!(c.kind, CapabilityKind::OnboardProfiles { .. }))
            .unwrap();
        assert_eq!(profiles.support, SupportLevel::Native);
        assert_eq!(caps[0].support, SupportLevel::VendorProtocol);
    }
}
