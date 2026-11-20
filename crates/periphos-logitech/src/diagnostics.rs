//! Read-only technical report of a HID++ 2.0 device: its features, its
//! programmable controls and its onboard memory layout. Used to support new
//! devices (button remapping, macros) from facts reported by real hardware.
//!
//! Nothing here changes the device: only "get" functions and memory reads.

use std::fmt::Write as _;

use crate::protocol::feature;
use crate::session::{Error, Presence, Session, Transport};

const FEATURE_SET: u16 = 0x0001;
const FIRMWARE_INFO: u16 = 0x0003;
const REPROG_CONTROLS_V4: u16 = 0x1B04;

/// Well-known HID++ 2.0 feature names, from Logitech's public feature list.
fn feature_name(id: u16) -> &'static str {
    match id {
        0x0000 => "Root",
        0x0001 => "Feature set",
        0x0002 => "Feature info",
        0x0003 => "Firmware info",
        0x0005 => "Device name and type",
        0x0007 => "Device friendly name",
        0x0008 => "Keep alive",
        0x0020 => "Config change",
        0x0021 => "Unique identifier (32 bytes)",
        0x00C2 => "DFU control (signed)",
        0x00C3 => "DFU control (secure)",
        0x00D0 => "DFU",
        0x1000 => "Battery status",
        0x1001 => "Battery voltage",
        0x1004 => "Unified battery",
        0x1814 => "Change host",
        0x1815 => "Hosts info",
        0x1B04 => "Reprogrammable controls v4",
        0x1BC0 => "Persistent remappable action",
        0x1D4B => "Wireless device status",
        0x1E00 => "Enable hidden features",
        0x1EB0 => "TDE memory access",
        0x1F20 => "ADC measurement",
        0x2201 => "Adjustable DPI",
        0x2202 => "Extended adjustable DPI",
        0x2250 => "Analysis mode",
        0x8010 => "Gaming G-keys",
        0x8020 => "Gaming M-keys",
        0x8030 => "Macro record key",
        0x8040 => "Brightness control",
        0x8060 => "Report rate",
        0x8061 => "Extended report rate",
        0x8070 => "Color LED effects",
        0x8071 => "RGB effects",
        0x8080 => "Per-key lighting",
        0x8081 => "Per-key lighting v2",
        0x8090 => "Mode status",
        0x8100 => "Onboard profiles",
        0x8110 => "Mouse button spy",
        0x8111 => "Latency monitoring",
        0x8120 => "Gaming attachments",
        0x8123 => "Force feedback",
        _ => "",
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Builds the report. Sections that fail are reported inline, so one
/// unsupported request does not hide the rest.
pub fn report<T: Transport>(s: &mut Session<T>, dev: u8) -> Result<String, Error> {
    let mut out = String::new();
    match s.ping(dev)? {
        Presence::Online { major, minor } => {
            let _ = writeln!(out, "HID++ protocol {major}.{minor}");
        }
        Presence::Legacy => return Ok("HID++ 1.0 device: no 2.0 features to report.".into()),
        Presence::Empty | Presence::Unreachable => return Err(Error::Timeout),
    }
    if let Ok(Some((name, kind))) = s.name_and_type(dev) {
        let _ = writeln!(
            out,
            "Name: {name} (type {})",
            kind.map_or("?".into(), |k| k.to_string())
        );
    }

    let features = features(s, dev, &mut out)?;
    let index_of = |id: u16| features.iter().find(|f| f.1 == id).map(|f| f.0);

    if let Some(index) = index_of(FIRMWARE_INFO) {
        section(&mut out, "Firmware", |out| firmware(s, dev, index, out));
    }
    if let Some(index) = index_of(REPROG_CONTROLS_V4) {
        section(&mut out, "Reprogrammable controls (0x1B04)", |out| {
            controls(s, dev, index, out)
        });
    }
    if let Some(index) = index_of(feature::ONBOARD_PROFILES) {
        section(&mut out, "Onboard profiles (0x8100)", |out| {
            onboard(s, dev, index, out)
        });
    }
    Ok(out)
}

fn section(out: &mut String, title: &str, f: impl FnOnce(&mut String) -> Result<(), Error>) {
    let _ = writeln!(out, "\n## {title}");
    if let Err(e) = f(out) {
        let _ = writeln!(out, "(could not read: {e:?})");
    }
}

/// `(index, id)` of every feature, also written to `out`.
fn features<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    out: &mut String,
) -> Result<Vec<(u8, u16)>, Error> {
    let _ = writeln!(out, "\n## Features");
    let Some(set) = s.feature_index(dev, FEATURE_SET)? else {
        let _ = writeln!(out, "(feature set not available)");
        return Ok(Vec::new());
    };
    let count = *s.call_feature(dev, set, 0, &[])?.first().unwrap_or(&0);
    let mut list = vec![(0, 0x0000)];
    for index in 1..=count {
        let data = s.call_feature(dev, set, 1, &[index])?;
        if data.len() < 2 {
            continue;
        }
        let id = u16::from_be_bytes([data[0], data[1]]);
        let flags = data.get(2).copied().unwrap_or(0);
        let version = data.get(3).copied().unwrap_or(0);
        let _ = writeln!(
            out,
            "{index:>3}  0x{id:04X} v{version}  {}{}",
            feature_name(id),
            if flags & 0xE0 != 0 {
                "  (hidden/internal)"
            } else {
                ""
            }
        );
        list.push((index, id));
    }
    Ok(list)
}

fn firmware<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    index: u8,
    out: &mut String,
) -> Result<(), Error> {
    let entities = *s.call_feature(dev, index, 0, &[])?.first().unwrap_or(&0);
    for entity in 0..entities.min(8) {
        let d = s.call_feature(dev, index, 1, &[entity])?;
        if d.len() < 8 {
            continue;
        }
        let kind = match d[0] & 0x0F {
            0 => "main",
            1 => "bootloader",
            2 => "hardware",
            3 => "touchpad",
            4 => "optical sensor",
            5 => "softdevice",
            6 => "RF companion",
            _ => "other",
        };
        let prefix: String = d[1..4]
            .iter()
            .filter(|b| b.is_ascii_graphic())
            .map(|&b| b as char)
            .collect();
        let _ = writeln!(
            out,
            "{kind}: {prefix} {:02X}.{:02X} build {:02X}{:02X}",
            d[4], d[5], d[6], d[7]
        );
    }
    Ok(())
}

fn controls<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    index: u8,
    out: &mut String,
) -> Result<(), Error> {
    let count = *s.call_feature(dev, index, 0, &[])?.first().unwrap_or(&0);
    let _ = writeln!(out, "cid    tid    flags (raw)  group gmask  reporting");
    for i in 0..count {
        let d = s.call_feature(dev, index, 1, &[i])?;
        if d.len() < 9 {
            continue;
        }
        let cid = u16::from_be_bytes([d[0], d[1]]);
        let tid = u16::from_be_bytes([d[2], d[3]]);
        let flags = u16::from(d[4]) | (u16::from(d[8]) << 8);
        let mut names = Vec::new();
        for (bit, name) in [
            (0x0001, "mouse"),
            (0x0002, "fkey"),
            (0x0004, "hotkey"),
            (0x0008, "fn-toggle"),
            (0x0010, "reprogrammable"),
            (0x0020, "divertable"),
            (0x0040, "persist-divertable"),
            (0x0080, "virtual"),
            (0x0100, "raw-xy"),
            (0x0200, "force-raw-xy"),
            (0x0400, "analytics"),
            (0x0800, "raw-wheel"),
        ] {
            if flags & bit != 0 {
                names.push(name);
            }
        }
        let reporting = s
            .call_feature(dev, index, 2, &cid.to_be_bytes())
            .map(|r| hex(&r[..r.len().min(6)]))
            .unwrap_or_else(|e| format!("{e:?}"));
        let _ = writeln!(
            out,
            "0x{cid:04X} 0x{tid:04X} {} ({:04X})  {:>3} {:02X}  {reporting}",
            names.join(","),
            flags,
            d[6],
            d[7]
        );
    }
    Ok(())
}

/// Bytes read per memory read.
const CHUNK: u16 = 16;

fn onboard<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    index: u8,
    out: &mut String,
) -> Result<(), Error> {
    let d = s.call_feature(dev, index, 0, &[])?;
    let _ = writeln!(out, "description: {}", hex(&d));
    if d.len() < 9 {
        return Ok(());
    }
    let sector_size = u16::from_be_bytes([d[7], d[8]]);
    let _ = writeln!(
        out,
        "memory model {}, profile format {}, macro format {}, profiles {} (+{} built in), \
         buttons {}, sectors {}, sector size {sector_size}",
        d[0], d[1], d[2], d[3], d[4], d[5], d[6]
    );
    if let Ok(mode) = s.call_feature(dev, index, 2, &[]) {
        let _ = writeln!(out, "mode: {}", hex(&mode[..mode.len().min(4)]));
    }
    if let Ok(current) = s.call_feature(dev, index, 4, &[]) {
        let _ = writeln!(
            out,
            "current profile: {}",
            hex(&current[..current.len().min(4)])
        );
    }
    if !(CHUNK..=4096).contains(&sector_size) {
        return Ok(());
    }

    // Profile directory (sector 0), then each listed profile sector.
    let directory = read_sector(s, dev, index, 0x0000, sector_size)?;
    let _ = writeln!(out, "\nsector 0x0000 (directory):");
    dump(out, &directory);
    let mut sectors = Vec::new();
    for entry in directory.chunks(4).take(16) {
        let sector = u16::from_be_bytes([entry[0], entry[1]]);
        if sector == 0xFFFF || sector == 0 {
            break;
        }
        sectors.push(sector);
    }
    if sectors.is_empty() {
        // Nothing saved in writable memory: the built-in (ROM) profile is used.
        sectors.push(0x0101);
    }
    for sector in sectors {
        match read_sector(s, dev, index, sector, sector_size) {
            Ok(bytes) => {
                let _ = writeln!(out, "\nsector 0x{sector:04X}:");
                dump(out, &bytes);
            }
            Err(e) => {
                let _ = writeln!(out, "\nsector 0x{sector:04X}: (could not read: {e:?})");
            }
        }
    }
    Ok(())
}

fn read_sector<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    index: u8,
    sector: u16,
    size: u16,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::with_capacity(size as usize);
    let mut offset = 0u16;
    while offset < size {
        // A read cannot run past the end of the sector: the last one overlaps.
        let start = offset.min(size - CHUNK);
        let [sh, sl] = sector.to_be_bytes();
        let [oh, ol] = start.to_be_bytes();
        let data = s.call_feature(dev, index, 5, &[sh, sl, oh, ol])?;
        let skip = (offset - start) as usize;
        let take = ((size - offset) as usize).min(CHUNK as usize);
        bytes.extend(data.iter().skip(skip).take(take));
        offset += CHUNK;
    }
    bytes.truncate(size as usize);
    Ok(bytes)
}

fn dump(out: &mut String, bytes: &[u8]) {
    for (row, chunk) in bytes.chunks(16).enumerate() {
        let _ = writeln!(out, "{:04X}  {}", row * 16, hex(chunk));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::session::tests::{long, FakeTransport};

    /// A mouse with the feature set at index 1, reprogrammable controls at 2
    /// and onboard profiles at 3 (one profile, 32-byte sectors).
    fn mouse() -> FakeTransport {
        FakeTransport::new(move |req| {
            let (dev, sub, addr) = (req[1], req[2], req[3]);
            let payload: Vec<u8> = match (sub, addr >> 4) {
                (0, 1) => vec![4, 2, 0x5A],
                (0, 0) => match u16::from_be_bytes([req[4], req[5]]) {
                    0x0001 => vec![1],
                    _ => vec![0],
                },
                (1, 0) => vec![3],
                (1, 1) => match req[4] {
                    1 => vec![0x00, 0x01, 0, 0],
                    2 => vec![0x1B, 0x04, 0, 5],
                    _ => vec![0x81, 0x00, 0, 0],
                },
                (2, 0) => vec![1],
                (2, 1) => vec![0x00, 0x50, 0x00, 0x38, 0x31, 0, 1, 1, 0],
                (2, 2) => vec![0x00, 0x50, 0, 0, 0],
                (3, 0) => vec![1, 1, 1, 1, 0, 11, 2, 0x00, 0x20, 0, 0],
                (3, 2) => vec![1],
                (3, 4) => vec![0, 1],
                (3, 5) => {
                    let sector = u16::from_be_bytes([req[4], req[5]]);
                    let offset = u16::from_be_bytes([req[6], req[7]]) as u8;
                    if sector == 0 {
                        let mut v = vec![0x00, 0x01, 0x01, 0x00, 0xFF, 0xFF, 0, 0];
                        v.resize(16, 0);
                        v
                    } else {
                        (offset..offset + 16).collect()
                    }
                }
                _ => return vec![vec![0x11, dev, 0xFF, sub, addr, 0x02, 0]],
            };
            vec![long(dev, sub, addr, &payload)]
        })
    }

    #[test]
    fn reports_features_controls_and_profile_memory() {
        let mut s = Session::new(mouse(), Duration::from_millis(50));
        let text = report(&mut s, 1).unwrap();
        assert!(text.contains("HID++ protocol 4.2"), "{text}");
        assert!(
            text.contains("0x1B04 v5  Reprogrammable controls v4"),
            "{text}"
        );
        assert!(
            text.contains("0x0050 0x0038 mouse,reprogrammable,divertable"),
            "{text}"
        );
        assert!(
            text.contains("buttons 11, sectors 2, sector size 32"),
            "{text}"
        );
        assert!(text.contains("sector 0x0001:"), "{text}");
        assert!(text.contains("0010  10 11 12"), "{text}");
    }
}
