//! Sensitivity (DPI), report rate and onboard-profile mode.

use periphos_core::device::{DeviceSettings, DpiSetting, ReportRateSetting};

use crate::protocol::feature;
use crate::session::{Error, Session, Transport};

/// Sensor index used for all requests: the mice PeriphOS supports have one sensor.
const SENSOR: u8 = 0;
/// Safety bound on DPI list pages; real devices answer in one or two.
const MAX_DPI_PAGES: u8 = 16;
/// Upper bound on the number of DPI values kept (a 100..=25600 step 50 list has 511).
const MAX_DPI_VALUES: usize = 4096;

/// Decodes a DPI list: big-endian u16 values; a value whose top three bits are
/// set encodes a step (low 13 bits) followed by the last value of a range;
/// 0x0000 ends the list.
pub fn parse_dpi_list(bytes: &[u8]) -> Vec<u32> {
    let mut values: Vec<u32> = Vec::new();
    let read = |i: usize| -> Option<u32> {
        bytes
            .get(i..i + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]) as u32)
    };
    let mut i = 0;
    while let Some(value) = read(i) {
        if value == 0 {
            break;
        }
        if value >> 13 == 0b111 {
            let step = value & 0x1FFF;
            let (Some(last), Some(&start)) = (read(i + 2), values.last()) else {
                break;
            };
            if step == 0 || last <= start {
                break;
            }
            let mut v = start + step;
            while v <= last && values.len() < MAX_DPI_VALUES {
                values.push(v);
                v += step;
            }
            i += 4;
        } else {
            values.push(value);
            i += 2;
        }
    }
    values.sort_unstable();
    values.dedup();
    values
}

/// REPORT_RATE (0x8060) list: bit n set means a period of n + 1 milliseconds.
pub fn parse_report_rate_flags(flags: u8) -> Vec<u32> {
    let mut rates: Vec<u32> = (0..8)
        .filter(|bit| flags >> bit & 1 == 1)
        .map(|bit| 1000 / (bit + 1))
        .collect();
    rates.sort_unstable();
    rates
}

/// EXTENDED_ADJUSTABLE_REPORT_RATE (0x8061) rate for each index.
pub const EXTENDED_RATES_HZ: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

pub fn parse_extended_rate_flags(flags: u16) -> Vec<u32> {
    EXTENDED_RATES_HZ
        .iter()
        .enumerate()
        .filter(|(bit, _)| flags >> bit & 1 == 1)
        .map(|(_, hz)| *hz)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RateFeature {
    Basic(u8),
    Extended(u8),
}

fn rate_feature<T: Transport>(s: &mut Session<T>, dev: u8) -> Result<Option<RateFeature>, Error> {
    if let Some(i) = s.feature_index(dev, feature::EXTENDED_REPORT_RATE)? {
        return Ok(Some(RateFeature::Extended(i)));
    }
    Ok(s.feature_index(dev, feature::REPORT_RATE)?
        .map(RateFeature::Basic))
}

pub fn read_dpi<T: Transport>(s: &mut Session<T>, dev: u8) -> Result<Option<DpiSetting>, Error> {
    let Some(index) = s.feature_index(dev, feature::ADJUSTABLE_DPI)? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    for page in 0..MAX_DPI_PAGES {
        let reply = s.call_feature(dev, index, 1, &[SENSOR, 0, page])?;
        // First byte echoes the sensor index.
        bytes.extend_from_slice(reply.get(1..).unwrap_or_default());
        if bytes.len() >= 2 && bytes[bytes.len() - 2..] == [0, 0] {
            break;
        }
    }
    let values = parse_dpi_list(&bytes);
    let reply = s.call_feature(dev, index, 2, &[SENSOR])?;
    let read = |i: usize| {
        reply
            .get(i..i + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]) as u32)
    };
    // A current value of 0 means "use the default", which follows it.
    let current = match read(1) {
        Some(0) | None => read(3).unwrap_or(0),
        Some(v) => v,
    };
    Ok((!values.is_empty() && current != 0).then_some(DpiSetting { current, values }))
}

pub fn set_dpi<T: Transport>(s: &mut Session<T>, dev: u8, dpi: u32) -> Result<(), Error> {
    let index = s
        .feature_index(dev, feature::ADJUSTABLE_DPI)?
        .ok_or(Error::Io("the device does not support changing DPI".into()))?;
    let [hi, lo] = u16::try_from(dpi)
        .map_err(|_| Error::Io("DPI out of range".into()))?
        .to_be_bytes();
    s.call_feature(dev, index, 3, &[SENSOR, hi, lo]).map(|_| ())
}

pub fn read_report_rate<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
) -> Result<Option<ReportRateSetting>, Error> {
    let setting = match rate_feature(s, dev)? {
        None => return Ok(None),
        Some(RateFeature::Basic(i)) => {
            let flags = *s.call_feature(dev, i, 0, &[])?.first().unwrap_or(&0);
            let ms = *s.call_feature(dev, i, 1, &[])?.first().unwrap_or(&0) as u32;
            (ms > 0).then(|| ReportRateSetting {
                current_hz: 1000 / ms,
                values_hz: parse_report_rate_flags(flags),
            })
        }
        Some(RateFeature::Extended(i)) => {
            let list = s.call_feature(dev, i, 1, &[])?;
            let flags = list
                .get(0..2)
                .map(|b| u16::from_be_bytes([b[0], b[1]]))
                .unwrap_or(0);
            let current = *s.call_feature(dev, i, 2, &[])?.first().unwrap_or(&0) as usize;
            EXTENDED_RATES_HZ.get(current).map(|&hz| ReportRateSetting {
                current_hz: hz,
                values_hz: parse_extended_rate_flags(flags),
            })
        }
    };
    Ok(setting.filter(|r| !r.values_hz.is_empty()))
}

pub fn set_report_rate<T: Transport>(s: &mut Session<T>, dev: u8, hz: u32) -> Result<(), Error> {
    match rate_feature(s, dev)? {
        None => Err(Error::Io(
            "the device does not support changing the report rate".into(),
        )),
        Some(RateFeature::Basic(i)) => {
            // Period in whole milliseconds (1 to 8) that maps back to exactly `hz`.
            let ms = (1..=8u32)
                .find(|ms| 1000 / ms == hz)
                .ok_or(Error::Io("unsupported report rate".into()))?;
            s.call_feature(dev, i, 2, &[ms as u8]).map(|_| ())
        }
        Some(RateFeature::Extended(i)) => {
            let index = EXTENDED_RATES_HZ
                .iter()
                .position(|&r| r == hz)
                .ok_or(Error::Io("unsupported report rate".into()))?;
            s.call_feature(dev, i, 3, &[index as u8]).map(|_| ())
        }
    }
}

/// `Some(true)` when the device runs from its onboard memory.
pub fn onboard_profile_active<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
) -> Result<Option<bool>, Error> {
    let Some(index) = s.feature_index(dev, feature::ONBOARD_PROFILES)? else {
        return Ok(None);
    };
    let mode = *s.call_feature(dev, index, 2, &[])?.first().unwrap_or(&0);
    Ok(match mode {
        0x01 => Some(true),
        0x02 => Some(false),
        _ => None,
    })
}

pub fn read_all<T: Transport>(s: &mut Session<T>, dev: u8) -> Result<DeviceSettings, Error> {
    Ok(DeviceSettings {
        dpi: read_dpi(s, dev)?,
        report_rate: read_report_rate(s, dev)?,
        onboard_profile_active: onboard_profile_active(s, dev).unwrap_or(None),
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::session::tests::{long, FakeTransport};

    #[test]
    fn dpi_list_with_range() {
        // 100, then steps of 50 up to 25600, then the terminator.
        let bytes = [0x00, 0x64, 0xE0, 0x32, 0x64, 0x00, 0x00, 0x00];
        let list = parse_dpi_list(&bytes);
        assert_eq!(list.first(), Some(&100));
        assert_eq!(list.last(), Some(&25600));
        assert_eq!(list.len(), 511);
        assert!(list.windows(2).all(|w| w[1] - w[0] == 50));
    }

    #[test]
    fn dpi_list_with_discrete_values() {
        let bytes = [0x01, 0x90, 0x03, 0x20, 0x06, 0x40, 0x00, 0x00];
        assert_eq!(parse_dpi_list(&bytes), vec![400, 800, 1600]);
    }

    #[test]
    fn malformed_dpi_range_stops_safely() {
        // A range marker before any start value.
        assert!(parse_dpi_list(&[0xE0, 0x32, 0x64, 0x00]).is_empty());
        // A zero step.
        assert_eq!(
            parse_dpi_list(&[0x00, 0x64, 0xE0, 0x00, 0x64, 0x00]),
            vec![100]
        );
    }

    #[test]
    fn report_rate_flags() {
        // 1, 2, 4 and 8 ms.
        assert_eq!(
            parse_report_rate_flags(0b1000_1011),
            vec![125, 250, 500, 1000]
        );
        assert_eq!(
            parse_extended_rate_flags(0b111_1111),
            EXTENDED_RATES_HZ.to_vec()
        );
    }

    /// A mouse with ADJUSTABLE_DPI at index 7 and REPORT_RATE at index 8,
    /// holding its state like real hardware would.
    fn mouse() -> FakeTransport {
        let mut dpi: u16 = 1600;
        let mut period_ms: u8 = 1;
        FakeTransport::new(move |req| {
            let (dev, sub, addr) = (req[1], req[2], req[3]);
            let payload: Vec<u8> = match (sub, addr >> 4) {
                (0, 0) => match u16::from_be_bytes([req[4], req[5]]) {
                    0x2201 => vec![7],
                    0x8060 => vec![8],
                    _ => vec![0],
                },
                (7, 1) => vec![0, 0x00, 0x64, 0xE0, 0x32, 0x64, 0x00, 0, 0],
                (7, 2) => {
                    let [h, l] = dpi.to_be_bytes();
                    vec![0, h, l, 0x03, 0x20]
                }
                (7, 3) => {
                    dpi = u16::from_be_bytes([req[5], req[6]]);
                    vec![0]
                }
                (8, 0) => vec![0b1000_1011],
                (8, 1) => vec![period_ms],
                (8, 2) => {
                    period_ms = req[4];
                    vec![]
                }
                _ => return vec![vec![0x11, dev, 0xFF, sub, addr, 0x07, 0]],
            };
            vec![long(dev, sub, addr, &payload)]
        })
    }

    #[test]
    fn reads_and_changes_dpi_and_rate() {
        let mut s = Session::new(mouse(), Duration::from_millis(50));
        let before = read_all(&mut s, 1).unwrap();
        assert_eq!(before.dpi.as_ref().unwrap().current, 1600);
        assert_eq!(before.report_rate.as_ref().unwrap().current_hz, 1000);
        assert_eq!(before.onboard_profile_active, None);

        set_dpi(&mut s, 1, 3200).unwrap();
        set_report_rate(&mut s, 1, 500).unwrap();
        let after = read_all(&mut s, 1).unwrap();
        assert_eq!(after.dpi.unwrap().current, 3200);
        assert_eq!(after.report_rate.unwrap().current_hz, 500);
    }
}
