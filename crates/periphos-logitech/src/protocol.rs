//! HID++ message format and decoders. Pure functions, no I/O.
//!
//! Implemented from publicly documented protocol facts (Logitech's HID++
//! feature documentation as described by the Solaar and libratbag projects).
//! No code from those projects is used.

use periphos_core::device::{BatteryLevel, BatteryState, ChargingState, DeviceKind};

pub const SHORT_REPORT: u8 = 0x10;
pub const LONG_REPORT: u8 = 0x11;
pub const SHORT_LEN: usize = 7;
pub const LONG_LEN: usize = 20;

/// Device index used for the receiver itself, or for a device connected by cable.
pub const DIRECT_INDEX: u8 = 0xFF;

/// Identifies PeriphOS's requests so replies are not confused with those of other
/// HID++ software running at the same time. Values known to be used elsewhere:
/// 0x07 OpenRGB, 0x0A LGSTrayEx, 0x0B Solaar, 0x0D Logitech G HUB, 0x0F firmware.
pub const SOFTWARE_ID: u8 = 0x09;

const HIDPP10_ERROR: u8 = 0x8F;
const HIDPP20_ERROR: u8 = 0xFF;

pub mod feature {
    pub const ROOT: u16 = 0x0000;
    pub const DEVICE_NAME: u16 = 0x0005;
    pub const BATTERY_STATUS: u16 = 0x1000;
    pub const BATTERY_VOLTAGE: u16 = 0x1001;
    pub const UNIFIED_BATTERY: u16 = 0x1004;
    pub const ADC_MEASUREMENT: u16 = 0x1F20;
    pub const ADJUSTABLE_DPI: u16 = 0x2201;
    pub const REPORT_RATE: u16 = 0x8060;
    pub const EXTENDED_REPORT_RATE: u16 = 0x8061;
    pub const ONBOARD_PROFILES: u16 = 0x8100;
}

/// HID++ 1.0 error codes that mean "no device in this slot right now".
pub mod hidpp10_error {
    pub const INVALID_SUB_ID: u8 = 0x01;
    pub const CONNECTION_REQUEST_FAILED: u8 = 0x04;
    pub const UNKNOWN_DEVICE: u8 = 0x08;
    pub const RESOURCE_ERROR: u8 = 0x09;
}

/// HID++ 1.0 sub ID for reading a long receiver register.
pub const GET_LONG_REGISTER: u8 = 0x83;
/// Receiver information register; sub-register 0x20 + n holds slot n's pairing data.
pub const RECEIVER_INFO_REGISTER: u8 = 0xB5;
pub const PAIRING_INFO_SUB_REGISTER: u8 = 0x20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub device: u8,
    /// Feature index (HID++ 2.0) or sub ID (HID++ 1.0 register access).
    pub sub_id: u8,
    /// Function number and software ID (HID++ 2.0) or register address (HID++ 1.0).
    pub address: u8,
    pub params: Vec<u8>,
    /// Register replies must also echo the first parameter (the sub-register).
    match_first_param: bool,
}

impl Request {
    /// HID++ 2.0 feature call. `function` is 0 to 15.
    pub fn new(device: u8, feature_index: u8, function: u8, params: &[u8]) -> Self {
        Self {
            device,
            sub_id: feature_index,
            address: (function << 4) | SOFTWARE_ID,
            params: params.to_vec(),
            match_first_param: false,
        }
    }

    /// HID++ 1.0 long register read on a receiver.
    pub fn read_receiver_register(register: u8, sub_register: u8) -> Self {
        Self {
            device: DIRECT_INDEX,
            sub_id: GET_LONG_REGISTER,
            address: register,
            params: vec![sub_register],
            match_first_param: true,
        }
    }

    /// Short report when the parameters fit, long report otherwise.
    pub fn encode(&self) -> Vec<u8> {
        let (id, len) = if self.params.len() <= SHORT_LEN - 4 {
            (SHORT_REPORT, SHORT_LEN)
        } else {
            (LONG_REPORT, LONG_LEN)
        };
        let mut report = vec![id, self.device, self.sub_id, self.address];
        report.extend_from_slice(&self.params[..self.params.len().min(len - 4)]);
        report.resize(len, 0);
        report
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// The request succeeded; the payload follows the 4-byte header.
    Ok(Vec<u8>),
    /// HID++ 2.0 feature error.
    FeatureError(u8),
    /// HID++ 1.0 error (also returned by receivers for empty or sleeping slots).
    Hidpp10Error(u8),
}

/// Whether `report` answers `request`, and how. Returns `None` for unrelated
/// reports (notifications, replies to other software).
pub fn match_reply(request: &Request, report: &[u8]) -> Option<Reply> {
    if report.len() < 5 || !matches!(report[0], SHORT_REPORT | LONG_REPORT) {
        return None;
    }
    // Bluetooth devices may answer with index 0x00 to a 0xFF request.
    let device_matches = report[1] == request.device || report[1] == request.device ^ 0xFF;
    if !device_matches {
        return None;
    }
    let (sub_id, address) = (request.sub_id, request.address);
    match report[2] {
        HIDPP10_ERROR if report.len() >= 6 && report[3] == sub_id && report[4] == address => {
            Some(Reply::Hidpp10Error(report[5]))
        }
        HIDPP20_ERROR if report.len() >= 6 && report[3] == sub_id && report[4] == address => {
            Some(Reply::FeatureError(report[5]))
        }
        id if id == sub_id && report[3] == address => {
            if request.match_first_param && report.get(4) != request.params.first() {
                return None;
            }
            Some(Reply::Ok(report[4..].to_vec()))
        }
        _ => None,
    }
}

/// Type codes of the DEVICE_NAME feature's getDeviceType function.
pub fn device_kind(code: u8) -> DeviceKind {
    match code {
        0x00 | 0x02 => DeviceKind::Keyboard,
        0x03..=0x05 => DeviceKind::Mouse,
        0x07 => DeviceKind::Receiver,
        _ => DeviceKind::Other,
    }
}

fn status_byte(status: u8) -> ChargingState {
    match status {
        0x00 => ChargingState::Discharging,
        0x01 | 0x02 | 0x04 => ChargingState::Charging,
        0x03 => ChargingState::Full,
        _ => ChargingState::Error,
    }
}

/// BATTERY_STATUS (0x1000), function 0: level %, next level %, status.
pub fn battery_status(data: &[u8]) -> Option<BatteryState> {
    let [level, _next, status, ..] = *data else {
        return None;
    };
    Some(BatteryState {
        percent: (level > 0).then_some(level.min(100)),
        level: None,
        charging: status_byte(status),
        estimated: false,
    })
}

/// UNIFIED_BATTERY (0x1004), function 1 (getStatus): %, level flags, status.
pub fn unified_battery(data: &[u8]) -> Option<BatteryState> {
    let [percent, level, status, ..] = *data else {
        return None;
    };
    let level = match level {
        l if l & 0x08 != 0 => BatteryLevel::Full,
        l if l & 0x04 != 0 => BatteryLevel::Good,
        l if l & 0x02 != 0 => BatteryLevel::Low,
        l if l & 0x01 != 0 => BatteryLevel::Critical,
        _ => BatteryLevel::Empty,
    };
    Some(BatteryState {
        percent: (percent > 0).then_some(percent.min(100)),
        level: Some(level),
        charging: status_byte(status),
        estimated: false,
    })
}

/// BATTERY_VOLTAGE (0x1001), function 0: millivolts (big-endian) and flags.
pub fn battery_voltage(data: &[u8]) -> Option<BatteryState> {
    let [hi, lo, flags, ..] = *data else {
        return None;
    };
    let millivolts = u16::from_be_bytes([hi, lo]);
    // Bit 7: external power connected. Bits 0-1: 0 charging, 1 full, 2 not charging.
    let charging = if flags & 0x80 != 0 {
        match flags & 0x03 {
            0x00 => ChargingState::Charging,
            0x01 => ChargingState::Full,
            0x02 => ChargingState::Discharging,
            _ => ChargingState::Error,
        }
    } else {
        ChargingState::Discharging
    };
    Some(BatteryState {
        percent: Some(estimate_percent(millivolts)),
        level: None,
        charging,
        estimated: true,
    })
}

/// ADC_MEASUREMENT (0x1F20), function 0: millivolts (big-endian) and flags.
/// Bit 0 of the flags means the measurement is valid; bit 1 means charging.
pub fn adc_measurement(data: &[u8]) -> Option<BatteryState> {
    let [hi, lo, flags, ..] = *data else {
        return None;
    };
    if flags & 0x01 == 0 {
        return None;
    }
    Some(BatteryState {
        percent: Some(estimate_percent(u16::from_be_bytes([hi, lo]))),
        level: None,
        charging: if flags & 0x02 != 0 {
            ChargingState::Charging
        } else {
            ChargingState::Discharging
        },
        estimated: true,
    })
}

/// Rough charge estimate for a single-cell lithium battery from its voltage,
/// interpolated over a typical discharge curve. Approximate by nature.
pub fn estimate_percent(millivolts: u16) -> u8 {
    const CURVE: [(u16, u8); 9] = [
        (4150, 100),
        (4050, 90),
        (3970, 80),
        (3870, 60),
        (3800, 45),
        (3750, 30),
        (3700, 15),
        (3600, 5),
        (3450, 0),
    ];
    if millivolts >= CURVE[0].0 {
        return 100;
    }
    for pair in CURVE.windows(2) {
        let ((v_hi, p_hi), (v_lo, p_lo)) = (pair[0], pair[1]);
        if millivolts >= v_lo {
            let span = (v_hi - v_lo) as u32;
            let above = (millivolts - v_lo) as u32;
            return p_lo + (((p_hi - p_lo) as u32 * above + span / 2) / span) as u8;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_request_layout() {
        let r = Request::new(0x01, 0x00, 0x1, &[0, 0, 0xAA]);
        assert_eq!(r.encode(), vec![0x10, 0x01, 0x00, 0x19, 0x00, 0x00, 0xAA]);
    }

    #[test]
    fn long_request_is_padded() {
        let r = Request::new(0xFF, 0x03, 0x2, &[1, 2, 3, 4]);
        let e = r.encode();
        assert_eq!(e.len(), LONG_LEN);
        assert_eq!(&e[..8], &[0x11, 0xFF, 0x03, 0x29, 1, 2, 3, 4]);
    }

    #[test]
    fn replies_are_matched_by_index_function_and_software_id() {
        let req = Request::new(0x01, 0x05, 0x0, &[]);
        let mut ok = vec![0x11, 0x01, 0x05, 0x09, 0x10];
        ok.resize(LONG_LEN, 0);
        assert_eq!(match_reply(&req, &ok), Some(Reply::Ok(ok[4..].to_vec())));
        // Same request from G HUB (software ID 0x0D): not ours.
        let ghub = [0x11, 0x01, 0x05, 0x0D, 0x10, 0, 0];
        assert_eq!(match_reply(&req, &ghub), None);
        // Other device index: not ours.
        assert_eq!(match_reply(&req, &[0x11, 0x02, 0x05, 0x09, 0x10]), None);
    }

    #[test]
    fn errors_are_recognised() {
        let req = Request::new(0x02, 0x00, 0x1, &[0, 0, 1]);
        assert_eq!(
            match_reply(&req, &[0x10, 0x02, 0x8F, 0x00, 0x19, 0x08, 0x00]),
            Some(Reply::Hidpp10Error(hidpp10_error::UNKNOWN_DEVICE))
        );
        assert_eq!(
            match_reply(&req, &[0x11, 0x02, 0xFF, 0x00, 0x19, 0x06, 0x00]),
            Some(Reply::FeatureError(0x06))
        );
    }

    #[test]
    fn receiver_register_read_layout_and_matching() {
        let req = Request::read_receiver_register(RECEIVER_INFO_REGISTER, 0x21);
        assert_eq!(req.encode(), vec![0x10, 0xFF, 0x83, 0xB5, 0x21, 0, 0]);
        let mut reply = vec![0x11, 0xFF, 0x83, 0xB5, 0x21, 0x09, 0x40, 0xA9];
        reply.resize(LONG_LEN, 0);
        assert!(matches!(match_reply(&req, &reply), Some(Reply::Ok(_))));
        // Reply for another slot's sub-register is not ours.
        reply[4] = 0x22;
        assert_eq!(match_reply(&req, &reply), None);
    }

    #[test]
    fn bluetooth_index_quirk_is_accepted() {
        let req = Request::new(0xFF, 0x00, 0x1, &[]);
        assert!(match_reply(&req, &[0x11, 0x00, 0x00, 0x19, 4, 2, 0]).is_some());
    }

    #[test]
    fn battery_decoders() {
        let s = battery_status(&[55, 50, 0x01]).unwrap();
        assert_eq!((s.percent, s.charging), (Some(55), ChargingState::Charging));
        assert_eq!(battery_status(&[0, 0, 0]).unwrap().percent, None);

        let u = unified_battery(&[80, 0x04, 0x00, 0]).unwrap();
        assert_eq!((u.percent, u.level), (Some(80), Some(BatteryLevel::Good)));

        let v = battery_voltage(&[0x0F, 0xA0, 0x00]).unwrap(); // 4000 mV
        assert!(v.estimated);
        assert!((80..=90).contains(&v.percent.unwrap()));
        assert_eq!(
            battery_voltage(&[0x10, 0x00, 0x81]).unwrap().charging,
            ChargingState::Full
        );

        assert!(adc_measurement(&[0x0F, 0xA0, 0x00]).is_none());
        assert_eq!(
            adc_measurement(&[0x0F, 0xA0, 0x03]).unwrap().charging,
            ChargingState::Charging
        );
    }

    #[test]
    fn voltage_curve_is_monotonic_and_bounded() {
        assert_eq!(estimate_percent(4300), 100);
        assert_eq!(estimate_percent(3000), 0);
        let mut last = 0;
        for mv in (3400..=4200).step_by(10) {
            let p = estimate_percent(mv);
            assert!(p >= last && p <= 100);
            last = p;
        }
    }

    #[test]
    fn device_types() {
        assert_eq!(device_kind(3), DeviceKind::Mouse);
        assert_eq!(device_kind(0), DeviceKind::Keyboard);
        assert_eq!(device_kind(42), DeviceKind::Other);
    }
}
