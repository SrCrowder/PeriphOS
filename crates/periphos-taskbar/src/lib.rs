//! # periphos-taskbar
//!
//! A compact, always-visible readout drawn on the Windows taskbar, just left
//! of the notification area: a device glyph and its battery level per item
//! (`🖱 93%  🎧 30%`). It is a small transparent window placed over the
//! taskbar, in the spirit of TraySense; no code from that project is used.
//!
//! - Hidden while a full-screen app (a game) is in front, when the taskbar is
//!   auto-hidden or vertical, and when there is nothing to show.
//! - Text colour follows the taskbar theme; low, critical and charging levels
//!   get their own colours.
//! - Other platforms: not supported ([`SUPPORTED`] is false, calls do nothing).

#[cfg(windows)]
mod windows;

/// Whether the readout can be shown on this platform.
pub const SUPPORTED: bool = cfg!(windows);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Mouse,
    Keyboard,
    Headset,
    Controller,
    Other,
}

impl Glyph {
    /// Code point in Segoe Fluent Icons / Segoe MDL2 Assets.
    pub fn code_point(self) -> char {
        match self {
            Glyph::Mouse => '\u{E962}',
            Glyph::Keyboard => '\u{E765}',
            Glyph::Headset => '\u{E7F6}',
            Glyph::Controller => '\u{E7FC}',
            Glyph::Other => '\u{E772}',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Low,
    Critical,
    Charging,
}

impl Tone {
    /// RGB for a dark (`light_taskbar == false`) or light taskbar.
    pub fn rgb(self, light_taskbar: bool) -> [u8; 3] {
        match (self, light_taskbar) {
            (Tone::Normal, false) => [0xFF, 0xFF, 0xFF],
            (Tone::Normal, true) => [0x00, 0x00, 0x00],
            (Tone::Low, false) => [0xF5, 0xB9, 0x47],
            (Tone::Low, true) => [0xA8, 0x6C, 0x0B],
            (Tone::Critical, false) => [0xFF, 0x5F, 0x6D],
            (Tone::Critical, true) => [0xD9, 0x34, 0x45],
            (Tone::Charging, false) => [0x5F, 0xB0, 0xFF],
            (Tone::Charging, true) => [0x2A, 0x72, 0xCC],
        }
    }
}

/// One entry of the readout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub glyph: Glyph,
    /// Short text, e.g. `93%`.
    pub text: String,
    pub tone: Tone,
}

/// Replaces what the readout shows. An empty list hides it. Starts the
/// readout on first use.
pub fn set_items(items: Vec<Item>) {
    #[cfg(windows)]
    windows::set_items(items);
    #[cfg(not(windows))]
    let _ = items;
}

/// Called (on the readout's own thread) when the user clicks the readout.
pub fn set_on_click(callback: impl Fn() + Send + Sync + 'static) {
    #[cfg(windows)]
    windows::set_on_click(Box::new(callback));
    #[cfg(not(windows))]
    let _ = callback;
}

/// Turns white-on-black text coverage (as GDI draws it, alpha unset) into
/// premultiplied BGRA in the colour of each horizontal range. Pixels without
/// text get alpha 1, so the whole readout stays clickable while invisible.
pub fn colorize(bgra: &mut [u8], width: usize, ranges: &[(usize, usize, [u8; 3])]) {
    for (i, px) in bgra.chunks_exact_mut(4).enumerate() {
        let x = i % width;
        let coverage = px[0].max(px[1]).max(px[2]) as u16;
        let [r, g, b] = ranges
            .iter()
            .find(|(start, end, _)| x >= *start && x < *end)
            .map(|(_, _, rgb)| *rgb)
            .unwrap_or([0, 0, 0]);
        let premultiply = |c: u8| ((c as u16 * coverage) / 255) as u8;
        px[0] = premultiply(b);
        px[1] = premultiply(g);
        px[2] = premultiply(r);
        px[3] = coverage.max(1) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorize_tints_by_range_and_keeps_background_clickable() {
        // Two pixels: full coverage at x=0 (red range), none at x=1.
        let mut px = vec![255, 255, 255, 0, 0, 0, 0, 0];
        colorize(&mut px, 2, &[(0, 1, [255, 0, 0])]);
        assert_eq!(&px[0..4], &[0, 0, 255, 255]); // BGRA red, opaque
        assert_eq!(&px[4..8], &[0, 0, 0, 1]); // transparent but hit-testable
    }

    #[test]
    fn partial_coverage_is_premultiplied() {
        let mut px = vec![128, 128, 128, 0];
        colorize(&mut px, 1, &[(0, 1, [255, 255, 255])]);
        assert_eq!(px, vec![128, 128, 128, 128]);
    }

    #[test]
    fn tones_differ_between_taskbar_themes() {
        assert_ne!(Tone::Normal.rgb(false), Tone::Normal.rgb(true));
        assert_eq!(Glyph::Mouse.code_point(), '\u{E962}');
    }
}
