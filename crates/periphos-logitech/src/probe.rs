//! Finds the devices paired with a receiver and reads what they report.

use periphos_core::device::{BatteryState, DeviceKind};

use crate::protocol::{self, feature};
use crate::session::{Presence, Session, Transport};

/// Slots a receiver can have (Unifying receivers pair up to six devices).
pub const MAX_SLOTS: u8 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairedDevice {
    pub slot: u8,
    pub name: Option<String>,
    pub kind: DeviceKind,
    pub battery: Option<BatteryState>,
    pub wireless_pid: Option<u16>,
    /// HID++ 1.0 device: present, but only basic information is available.
    pub legacy: bool,
    /// The device accepts DPI changes (ADJUSTABLE_DPI).
    pub adjustable_dpi: bool,
    /// The device accepts report rate changes (REPORT_RATE or its extended version).
    pub adjustable_report_rate: bool,
}

#[derive(Debug, Default)]
pub struct ProbeResult {
    /// Devices that answered.
    pub devices: Vec<PairedDevice>,
    /// Slots whose device did not answer this time but may still be paired.
    pub unreachable: Vec<u8>,
}

/// Every device on the receiver. A failure on one slot never hides the others.
pub fn probe_receiver<T: Transport>(session: &mut Session<T>, read_pairing: bool) -> ProbeResult {
    let mut found = Vec::new();
    let mut unreachable = Vec::new();
    for slot in 1..=MAX_SLOTS {
        let presence = session.ping(slot).unwrap_or(Presence::Unreachable);
        match presence {
            Presence::Empty => {}
            Presence::Unreachable => unreachable.push(slot),
            Presence::Legacy => found.push(PairedDevice {
                slot,
                name: None,
                kind: DeviceKind::Other,
                battery: None,
                wireless_pid: read_pairing
                    .then(|| session.wireless_product_id(slot).ok().flatten())
                    .flatten(),
                legacy: true,
                adjustable_dpi: false,
                adjustable_report_rate: false,
            }),
            Presence::Online { .. } => {
                let (name, kind) = match session.name_and_type(slot) {
                    Ok(Some((name, kind))) => (Some(name), kind.map(protocol::device_kind)),
                    _ => (None, None),
                };
                found.push(PairedDevice {
                    slot,
                    name,
                    kind: kind.unwrap_or(DeviceKind::Other),
                    battery: read_battery(session, slot),
                    wireless_pid: read_pairing
                        .then(|| session.wireless_product_id(slot).ok().flatten())
                        .flatten(),
                    legacy: false,
                    adjustable_dpi: supports(session, slot, &[feature::ADJUSTABLE_DPI]),
                    adjustable_report_rate: supports(
                        session,
                        slot,
                        &[feature::REPORT_RATE, feature::EXTENDED_REPORT_RATE],
                    ),
                });
            }
        }
    }
    ProbeResult {
        devices: found,
        unreachable,
    }
}

fn supports<T: Transport>(session: &mut Session<T>, device: u8, features: &[u16]) -> bool {
    features
        .iter()
        .any(|&f| matches!(session.feature_index(device, f), Ok(Some(_))))
}

/// Tries each battery feature a Logitech device may implement.
pub fn read_battery<T: Transport>(session: &mut Session<T>, device: u8) -> Option<BatteryState> {
    type Decoder = fn(&[u8]) -> Option<BatteryState>;
    const SOURCES: [(u16, u8, Decoder); 4] = [
        (feature::UNIFIED_BATTERY, 1, protocol::unified_battery),
        (feature::BATTERY_STATUS, 0, protocol::battery_status),
        (feature::BATTERY_VOLTAGE, 0, protocol::battery_voltage),
        (feature::ADC_MEASUREMENT, 0, protocol::adc_measurement),
    ];
    for (feature_id, function, decode) in SOURCES {
        let Ok(Some(index)) = session.feature_index(device, feature_id) else {
            continue;
        };
        if let Some(state) = session
            .call_feature(device, index, function, &[])
            .ok()
            .and_then(|d| decode(&d))
        {
            return Some(state);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use periphos_core::device::ChargingState;

    use super::*;
    use crate::session::tests::{long, FakeTransport};

    /// A LIGHTSPEED receiver with a G502 X LIGHTSPEED in slot 1 that reports battery
    /// through BATTERY_VOLTAGE, and nothing in the other slots.
    fn lightspeed_with_mouse() -> FakeTransport {
        const NAME: &[u8] = b"G502 X LIGHTSPEED";
        FakeTransport::new(|req| {
            let (dev, sub, addr) = (req[1], req[2], req[3]);
            if dev == 0xFF && sub == 0x83 && addr == 0xB5 {
                return vec![long(0xFF, 0x83, 0xB5, &[req[4], 0x09, 0x0A, 0x40, 0xAD])];
            }
            if dev != 1 {
                return vec![vec![0x10, dev, 0x8F, sub, addr, 0x08, 0]];
            }
            let payload: Vec<u8> = match (sub, addr >> 4) {
                (0, 1) => vec![4, 2, req[6]],
                (0, 0) => match u16::from_be_bytes([req[4], req[5]]) {
                    0x0005 => vec![0x03],
                    0x1001 => vec![0x06],
                    _ => vec![0x00],
                },
                (3, 0) => vec![NAME.len() as u8],
                (3, 1) => NAME[req[4] as usize..(req[4] as usize + 16).min(NAME.len())].to_vec(),
                (3, 2) => vec![0x03],
                (6, 0) => vec![0x0F, 0x50, 0x00], // 3920 mV, discharging
                _ => return vec![vec![0x11, dev, 0xFF, sub, addr, 0x07, 0]],
            };
            vec![long(dev, sub, addr, &payload)]
        })
    }

    #[test]
    fn finds_the_mouse_behind_the_receiver() {
        let mut session = Session::new(lightspeed_with_mouse(), Duration::from_millis(50));
        let result = probe_receiver(&mut session, true);
        assert!(result.unreachable.is_empty());
        let devices = result.devices;
        assert_eq!(devices.len(), 1);
        let mouse = &devices[0];
        assert_eq!(mouse.slot, 1);
        assert_eq!(mouse.name.as_deref(), Some("G502 X LIGHTSPEED"));
        assert_eq!(mouse.kind, DeviceKind::Mouse);
        assert_eq!(mouse.wireless_pid, Some(0x40AD));
        let battery = mouse.battery.unwrap();
        assert!(battery.estimated);
        assert_eq!(battery.charging, ChargingState::Discharging);
        assert!((50..=75).contains(&battery.percent.unwrap()));
    }
}
