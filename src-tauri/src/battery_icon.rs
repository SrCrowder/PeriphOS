//! Draws the battery tray icon: the percentage in a small pixel font over a
//! gauge bar, coloured by state. No font or image dependencies.

use periphos_core::device::{BatteryLevel, BatteryState, ChargingState};

pub const SIZE: u32 = 32;

/// 3×5 pixel digits, one row per byte (bits 2..0 = left..right).
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Low,
    Critical,
    Charging,
}

impl Tone {
    fn rgb(self) -> [u8; 3] {
        match self {
            Tone::Normal => [0x3f, 0xd5, 0x8c],
            Tone::Low => [0xf5, 0xb9, 0x47],
            Tone::Critical => [0xff, 0x5f, 0x6d],
            Tone::Charging => [0x5f, 0xb0, 0xff],
        }
    }
}

/// Fill fraction (0–100) and tone for a battery, given the alert thresholds.
pub fn appearance(battery: &BatteryState, warning: u8, critical: u8) -> (u8, Tone) {
    let fill = battery.percent.unwrap_or(match battery.level {
        Some(BatteryLevel::Full) => 100,
        Some(BatteryLevel::Good) => 60,
        Some(BatteryLevel::Low) => 20,
        Some(BatteryLevel::Critical) => 8,
        Some(BatteryLevel::Empty) | None => 0,
    });
    let tone = match battery.charging {
        ChargingState::Charging | ChargingState::Full => Tone::Charging,
        _ if fill <= critical => Tone::Critical,
        _ if fill <= warning => Tone::Low,
        _ => Tone::Normal,
    };
    (fill.min(100), tone)
}

/// RGBA pixels of a `SIZE`×`SIZE` icon.
pub fn render(percent: Option<u8>, fill: u8, tone: Tone) -> Vec<u8> {
    let size = SIZE as usize;
    let mut rgba = vec![0u8; size * size * 4];
    let [r, g, b] = tone.rgb();
    let mut put = |x: usize, y: usize, alpha: u8| {
        if x < size && y < size {
            let i = (y * size + x) * 4;
            rgba[i..i + 4].copy_from_slice(&[r, g, b, alpha]);
        }
    };

    if let Some(p) = percent {
        let text: Vec<usize> = p
            .min(100)
            .to_string()
            .bytes()
            .map(|c| (c - b'0') as usize)
            .collect();
        // Larger digits when they fit: 2 digits at scale 4, 3 digits at scale 3.
        let scale = if text.len() >= 3 { 3 } else { 4 };
        let gap = 2;
        let width = text.len() * 3 * scale + (text.len() - 1) * gap;
        let mut x0 = (size - width) / 2;
        let y0 = 1;
        for digit in text {
            for (row, bits) in DIGITS[digit].iter().enumerate() {
                for col in 0..3 {
                    if bits >> (2 - col) & 1 == 1 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                put(x0 + col * scale + dx, y0 + row * scale + dy, 255);
                            }
                        }
                    }
                }
            }
            x0 += 3 * scale + gap;
        }
    }

    // Gauge: a faint track with the filled part on top.
    let (bar_top, bar_height) = if percent.is_some() { (25, 6) } else { (12, 10) };
    let filled = (size * fill as usize).div_ceil(100);
    for y in bar_top..bar_top + bar_height {
        for x in 0..size {
            put(x, y, if x < filled { 255 } else { 70 });
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque_pixels(rgba: &[u8]) -> usize {
        rgba.chunks(4).filter(|p| p[3] == 255).count()
    }

    #[test]
    fn renders_a_square_icon() {
        let icon = render(Some(61), 61, Tone::Normal);
        assert_eq!(icon.len(), (SIZE * SIZE * 4) as usize);
        assert!(opaque_pixels(&icon) > 100);
    }

    #[test]
    fn three_digits_fit() {
        // "100" spans the icon without being clipped on either side.
        let icon = render(Some(100), 100, Tone::Charging);
        let size = SIZE as usize;
        let columns: Vec<usize> = (0..24)
            .flat_map(|y| (0..size).map(move |x| (x, y)))
            .filter(|&(x, y)| icon[(y * size + x) * 4 + 3] == 255)
            .map(|(x, _)| x)
            .collect();
        let (left, right) = (
            *columns.iter().min().unwrap(),
            *columns.iter().max().unwrap(),
        );
        assert_eq!(right - left + 1, 31); // 3 digits × 9 px + 2 gaps × 2 px
    }

    /// Writes sample icons as raw RGBA for a visual check:
    /// `PERIPHOS_ICON_DUMP=/tmp/icons cargo test -p periphos dump_samples -- --ignored`
    #[test]
    #[ignore]
    fn dump_samples() {
        let Ok(dir) = std::env::var("PERIPHOS_ICON_DUMP") else {
            return;
        };
        for (name, percent, fill, tone) in [
            ("61", Some(61), 61, Tone::Normal),
            ("18", Some(18), 18, Tone::Low),
            ("7", Some(7), 7, Tone::Critical),
            ("100", Some(100), 100, Tone::Charging),
            ("level", None, 20, Tone::Low),
        ] {
            std::fs::write(format!("{dir}/{name}.rgba"), render(percent, fill, tone)).unwrap();
        }
    }

    #[test]
    fn tone_follows_thresholds_and_charging() {
        let state = |p: u8, charging| BatteryState {
            percent: Some(p),
            level: None,
            charging,
            estimated: false,
        };
        assert_eq!(
            appearance(&state(61, ChargingState::Discharging), 20, 10).1,
            Tone::Normal
        );
        assert_eq!(
            appearance(&state(18, ChargingState::Discharging), 20, 10).1,
            Tone::Low
        );
        assert_eq!(
            appearance(&state(9, ChargingState::Discharging), 20, 10).1,
            Tone::Critical
        );
        assert_eq!(
            appearance(&state(9, ChargingState::Charging), 20, 10).1,
            Tone::Charging
        );
        let level_only = BatteryState {
            percent: None,
            level: Some(BatteryLevel::Low),
            charging: ChargingState::Discharging,
            estimated: false,
        };
        assert_eq!(appearance(&level_only, 20, 10), (20, Tone::Low));
    }

    #[test]
    fn gauge_fill_is_proportional() {
        let size = SIZE as usize;
        let icon = render(Some(50), 50, Tone::Normal);
        let row = 27;
        let filled = (0..size)
            .filter(|&x| icon[(row * size + x) * 4 + 3] == 255)
            .count();
        assert_eq!(filled, 16);
    }
}
