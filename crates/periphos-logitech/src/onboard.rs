//! Button assignments stored in a device's onboard profiles (HID++ feature
//! 0x8100), for devices that keep their buttons in their own memory (Logitech
//! G mice).
//!
//! Memory is organised in sectors. Sector 0 lists the profiles (4 bytes per
//! entry: sector number, enabled flag, reserved; `FF FF` ends the list); each
//! profile is one sector. The last two bytes of every sector hold a
//! CRC-16/CCITT (initial value 0xFFFF) of the rest, big-endian.
//!
//! Profile layout (formats 4 and 5): report rate, DPI slots, power timeouts,
//! then from 0x20 one 4-byte assignment per button, and from 0x60 the same
//! for the G-shift layer. Only button assignments are changed here; every
//! other byte is written back exactly as read.
//!
//! Writing is read, modify, write, read back: a sector that fails its CRC is
//! never modified, and a write that does not read back identically is undone.

use periphos_core::device::{
    ButtonAction, ButtonBinding, ButtonLayout, MouseButton, SpecialFunction,
};

use crate::protocol::feature;
use crate::session::{Error, Session, Transport};

const FN_DESCRIPTION: u8 = 0;
const FN_GET_MODE: u8 = 2;
const FN_SET_CURRENT_PROFILE: u8 = 3;
const FN_GET_CURRENT_PROFILE: u8 = 4;
const FN_MEMORY_READ: u8 = 5;
const FN_WRITE_START: u8 = 6;
const FN_WRITE_DATA: u8 = 7;
const FN_WRITE_END: u8 = 8;

const MODE_ONBOARD: u8 = 1;
/// Bytes moved per memory read or write.
const CHUNK: usize = 16;
/// Offset of the first button assignment in a profile sector.
const BUTTONS_OFFSET: usize = 0x20;
/// Profile formats whose layout is known.
const KNOWN_FORMATS: [u8; 2] = [4, 5];

const SPECIALS: [(u8, SpecialFunction); 12] = [
    (0x01, SpecialFunction::WheelLeft),
    (0x02, SpecialFunction::WheelRight),
    (0x03, SpecialFunction::DpiUp),
    (0x04, SpecialFunction::DpiDown),
    (0x05, SpecialFunction::DpiCycle),
    (0x06, SpecialFunction::DpiDefault),
    (0x07, SpecialFunction::DpiShift),
    (0x08, SpecialFunction::ProfileNext),
    (0x09, SpecialFunction::ProfilePrevious),
    (0x0A, SpecialFunction::ProfileCycle),
    (0x0B, SpecialFunction::GShift),
    (0x0C, SpecialFunction::BatteryLevel),
];

const MOUSE_BUTTONS: [(u16, MouseButton); 5] = [
    (0x0001, MouseButton::Left),
    (0x0002, MouseButton::Right),
    (0x0004, MouseButton::Middle),
    (0x0008, MouseButton::Back),
    (0x0010, MouseButton::Forward),
];

/// CRC-16/CCITT-FALSE, as stored at the end of every sector.
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// Whether the sector's stored CRC matches its contents.
pub fn crc_ok(sector: &[u8]) -> bool {
    let Some(split) = sector.len().checked_sub(2) else {
        return false;
    };
    crc16(&sector[..split]) == u16::from_be_bytes([sector[split], sector[split + 1]])
}

fn seal(sector: &mut [u8]) {
    let split = sector.len() - 2;
    let [hi, lo] = crc16(&sector[..split]).to_be_bytes();
    sector[split] = hi;
    sector[split + 1] = lo;
}

pub fn decode_action(raw: [u8; 4]) -> ButtonAction {
    match raw {
        [0xFF, ..] => ButtonAction::Disabled,
        [0x80, 0x01, hi, lo] => {
            let bits = u16::from_be_bytes([hi, lo]);
            MOUSE_BUTTONS.iter().find(|(b, _)| *b == bits).map_or_else(
                || ButtonAction::Other {
                    description: format!("Mouse buttons 0x{bits:04X}"),
                },
                |&(_, button)| ButtonAction::Mouse { button },
            )
        }
        [0x80, 0x02, modifiers, key] => ButtonAction::Key { key, modifiers },
        [0x80, 0x03, hi, lo] => ButtonAction::Media {
            usage: u16::from_be_bytes([hi, lo]),
        },
        [0x90, code, ..] => SPECIALS.iter().find(|(c, _)| *c == code).map_or_else(
            || ButtonAction::Other {
                description: format!("Device function 0x{code:02X}"),
            },
            |&(_, function)| ButtonAction::Special { function },
        ),
        [0x00, ..] => ButtonAction::Other {
            description: "Macro".into(),
        },
        _ => ButtonAction::Other {
            description: format!(
                "Unknown ({:02X} {:02X} {:02X} {:02X})",
                raw[0], raw[1], raw[2], raw[3]
            ),
        },
    }
}

/// Bytes for `action`, or `None` for actions that cannot be stored.
pub fn encode_action(action: &ButtonAction) -> Option<[u8; 4]> {
    Some(match action {
        ButtonAction::Disabled => [0xFF; 4],
        ButtonAction::Mouse { button } => {
            let bits = MOUSE_BUTTONS.iter().find(|(_, b)| b == button)?.0;
            let [hi, lo] = bits.to_be_bytes();
            [0x80, 0x01, hi, lo]
        }
        ButtonAction::Key { key, modifiers } => [0x80, 0x02, *modifiers, *key],
        ButtonAction::Media { usage } => {
            let [hi, lo] = usage.to_be_bytes();
            [0x80, 0x03, hi, lo]
        }
        ButtonAction::Special { function } => {
            [0x90, SPECIALS.iter().find(|(_, f)| f == function)?.0, 0, 0]
        }
        ButtonAction::Other { .. } => return None,
    })
}

/// Functions offered for assignment. G-shift is left out until its second
/// layer can be edited, and battery level has no visible effect on most mice.
pub fn assignable_specials() -> Vec<SpecialFunction> {
    SPECIALS
        .iter()
        .map(|&(_, f)| f)
        .filter(|f| !matches!(f, SpecialFunction::GShift | SpecialFunction::BatteryLevel))
        .collect()
}

/// Physical names of the buttons, in memory order, for known models.
fn button_names(model: &str, count: usize) -> Vec<String> {
    let g502_x = [
        "Left button",
        "Right button",
        "Wheel click",
        "Back button",
        "Sniper button",
        "Forward button",
        "Wheel tilt left",
        "Wheel tilt right",
        "Button below the wheel",
        "DPI up button",
        "DPI down button",
    ];
    let known: &[&str] = if model.starts_with("G502 X") && count == g502_x.len() {
        &g502_x
    } else {
        &[]
    };
    (0..count)
        .map(|i| {
            known
                .get(i)
                .map_or_else(|| format!("Button {}", i + 1), |n| (*n).to_owned())
        })
        .collect()
}

/// What the device says about its onboard memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Description {
    pub profile_format: u8,
    pub profile_count: u8,
    /// Built-in, read-only profiles (the factory settings).
    pub rom_profile_count: u8,
    pub button_count: u8,
    pub sector_size: u16,
}

impl Description {
    pub fn parse(d: &[u8]) -> Option<Self> {
        (d.len() >= 9).then(|| Self {
            profile_format: d[1],
            profile_count: d[3],
            rom_profile_count: d[4],
            button_count: d[5],
            sector_size: u16::from_be_bytes([d[7], d[8]]),
        })
    }
}

/// The profile the device uses now, as read from its memory.
#[derive(Clone)]
struct ActiveProfile {
    index: u8,
    number: u8,
    description: Description,
    sector: u16,
    bytes: Vec<u8>,
    onboard_mode: bool,
}

fn read_sector<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    index: u8,
    sector: u16,
    size: usize,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::with_capacity(size);
    let mut offset = 0;
    while offset < size {
        // A read cannot run past the end of the sector: the last one overlaps.
        let start = offset.min(size - CHUNK);
        let [sh, sl] = sector.to_be_bytes();
        let [oh, ol] = (start as u16).to_be_bytes();
        let data = s.call_feature(dev, index, FN_MEMORY_READ, &[sh, sl, oh, ol])?;
        if data.len() < CHUNK {
            return Err(Error::Io("Short reply while reading device memory.".into()));
        }
        bytes.extend_from_slice(&data[offset - start..CHUNK.min(size - start)]);
        offset = start + CHUNK;
    }
    Ok(bytes)
}

fn write_sector<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    index: u8,
    sector: u16,
    bytes: &[u8],
) -> Result<(), Error> {
    let [sh, sl] = sector.to_be_bytes();
    let [nh, nl] = (bytes.len() as u16).to_be_bytes();
    s.call_feature(dev, index, FN_WRITE_START, &[sh, sl, 0, 0, nh, nl])?;
    for chunk in bytes.chunks(CHUNK) {
        let mut data = [0u8; CHUNK];
        data[..chunk.len()].copy_from_slice(chunk);
        s.call_feature(dev, index, FN_WRITE_DATA, &data)?;
    }
    s.call_feature(dev, index, FN_WRITE_END, &[])?;
    Ok(())
}

fn unsupported(message: &str) -> Error {
    Error::Io(message.to_owned())
}

fn read_active<T: Transport>(s: &mut Session<T>, dev: u8) -> Result<Option<ActiveProfile>, Error> {
    let Some(index) = s.feature_index(dev, feature::ONBOARD_PROFILES)? else {
        return Ok(None);
    };
    let description = Description::parse(&s.call_feature(dev, index, FN_DESCRIPTION, &[])?)
        .ok_or_else(|| unsupported("The mouse described its memory in an unknown way."))?;
    if !KNOWN_FORMATS.contains(&description.profile_format) {
        return Err(unsupported(&format!(
            "This mouse stores its profiles in a format PeriphOS does not know yet ({}).",
            description.profile_format
        )));
    }
    let size = description.sector_size as usize;
    if !(BUTTONS_OFFSET + 4 * description.button_count as usize + 2..=4096).contains(&size) {
        return Err(unsupported(
            "The mouse reported an unexpected memory layout.",
        ));
    }
    let onboard_mode = s
        .call_feature(dev, index, FN_GET_MODE, &[])?
        .first()
        .is_some_and(|&m| m == MODE_ONBOARD);
    let current = s.call_feature(dev, index, FN_GET_CURRENT_PROFILE, &[])?;
    let (Some(&bank), Some(&number)) = (current.first(), current.get(1)) else {
        return Err(unsupported("The mouse did not say which profile it uses."));
    };
    if bank != 0 || number == 0 {
        return Err(unsupported(
            "The mouse is using its built-in profile, which cannot be changed. Save a profile to \
             the mouse with its manufacturer's app first.",
        ));
    }
    let directory = read_sector(s, dev, index, 0x0000, size)?;
    if !crc_ok(&directory) {
        return Err(unsupported(
            "The mouse's profile list failed its integrity check.",
        ));
    }
    let entry = (number as usize - 1) * 4;
    let sector = directory
        .get(entry..entry + 2)
        .map(|b| u16::from_be_bytes([b[0], b[1]]))
        .filter(|&s| s != 0 && s != 0xFFFF)
        .ok_or_else(|| unsupported("The active profile is not in the mouse's profile list."))?;
    let bytes = read_sector(s, dev, index, sector, size)?;
    if !crc_ok(&bytes) {
        return Err(unsupported(
            "The active profile in the mouse failed its integrity check, so PeriphOS will not change it.",
        ));
    }
    Ok(Some(ActiveProfile {
        index,
        number,
        description,
        sector,
        bytes,
        onboard_mode,
    }))
}

/// Assignments of the first `count` buttons in a profile sector.
fn decode_buttons(bytes: &[u8], count: usize) -> Vec<ButtonAction> {
    (0..count)
        .map(|i| {
            let at = BUTTONS_OFFSET + i * 4;
            decode_action([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        })
        .collect()
}

fn layout(profile: &ActiveProfile, model: &str) -> ButtonLayout {
    let count = profile.description.button_count as usize;
    let names = button_names(model, count);
    let buttons = decode_buttons(&profile.bytes, count)
        .into_iter()
        .zip(names)
        .enumerate()
        .map(|(i, (action, name))| ButtonBinding {
            index: i as u8,
            name,
            action,
        })
        .collect();
    ButtonLayout {
        buttons,
        onboard_profile: Some(profile.number),
        in_effect: profile.onboard_mode,
        special_functions: assignable_specials(),
        factory_defaults: None,
    }
}

/// First sector of the built-in profiles in read-only memory.
const ROM_FIRST_SECTOR: u16 = 0x0101;

/// Button assignments of the first built-in profile: the factory settings.
/// `None` when they cannot be read or fail their integrity check.
fn factory_defaults<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    profile: &ActiveProfile,
) -> Option<Vec<ButtonAction>> {
    if profile.description.rom_profile_count == 0 {
        return None;
    }
    let size = profile.description.sector_size as usize;
    let bytes = read_sector(s, dev, profile.index, ROM_FIRST_SECTOR, size).ok()?;
    crc_ok(&bytes).then(|| decode_buttons(&bytes, profile.description.button_count as usize))
}

/// Buttons of the active onboard profile, or `None` without onboard profiles.
pub fn read_buttons<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    model: &str,
) -> Result<Option<ButtonLayout>, Error> {
    let Some(profile) = read_active(s, dev)? else {
        return Ok(None);
    };
    let mut layout = layout(&profile, model);
    layout.factory_defaults = factory_defaults(s, dev, &profile);
    Ok(Some(layout))
}

/// Stores the given assignments (button index, action) in the active
/// onboard profile with a single write, and returns the layout read back.
/// Assignments that cannot be stored (such as an unknown macro) are left as
/// they are.
pub fn set_buttons<T: Transport>(
    s: &mut Session<T>,
    dev: u8,
    model: &str,
    assignments: &[(u8, ButtonAction)],
) -> Result<ButtonLayout, Error> {
    let profile =
        read_active(s, dev)?.ok_or_else(|| unsupported("This device has no onboard profiles."))?;
    let mut updated = profile.bytes.clone();
    for (button, action) in assignments {
        if *button >= profile.description.button_count {
            return Err(unsupported("There is no such button on this device."));
        }
        let Some(raw) = encode_action(action) else {
            if assignments.len() == 1 {
                return Err(unsupported("This action cannot be stored on the device."));
            }
            continue;
        };
        let at = BUTTONS_OFFSET + *button as usize * 4;
        updated[at..at + 4].copy_from_slice(&raw);
    }
    seal(&mut updated);
    if !layout(
        &ActiveProfile {
            bytes: updated.clone(),
            ..profile.clone()
        },
        model,
    )
    .keeps_left_click()
    {
        return Err(unsupported(
            "At least one button must keep the left click, or the mouse could not click.",
        ));
    }
    if updated != profile.bytes {
        let size = updated.len();
        write_sector(s, dev, profile.index, profile.sector, &updated)?;
        let check = read_sector(s, dev, profile.index, profile.sector, size)?;
        if check != updated {
            // Put the previous contents back rather than leave a damaged profile.
            let _ = write_sector(s, dev, profile.index, profile.sector, &profile.bytes);
            return Err(unsupported(
                "The mouse did not store the change correctly, so it was undone.",
            ));
        }
        if profile.onboard_mode {
            // Reload the profile so the new assignments are used right away.
            // They are already stored, so a failure here is not an error: the
            // mouse picks them up the next time it loads the profile.
            let _ = s.call_feature(
                dev,
                profile.index,
                FN_SET_CURRENT_PROFILE,
                &[0, profile.number],
            );
        }
    }
    read_buttons(s, dev, model)?.ok_or_else(|| unsupported("This device has no onboard profiles."))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::time::Duration;

    use super::*;
    use crate::session::tests::{long, FakeTransport};

    fn hex(text: &str) -> Vec<u8> {
        text.split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }

    /// Profile 1 of a G502 X PLUS, as read from a real mouse.
    fn real_profile() -> Vec<u8> {
        let mut bytes = hex("01 02 00 B0 04 B0 04 B0 04 B0 04 B0 04 FF FF FF
             FF 00 FF FF FF FF FF FF FF FF FF FF 3C 00 2C 01
             80 01 00 01 80 01 00 02 80 01 00 04 80 01 00 08
             90 07 00 00 80 01 00 10 90 01 00 00 90 02 00 00
             90 0A 00 00 90 03 00 00 90 04 00 00");
        bytes.resize(0xD0, 0xFF);
        bytes.resize(0xFD, 0x00);
        bytes.extend([0x8E, 0x1A]);
        bytes
    }

    fn real_directory() -> Vec<u8> {
        let mut bytes = hex("00 01 01 FF 00 02 00 FF 00 03 00 FF 00 04 00 FF 00 05 00 FF");
        bytes.resize(0xFD, 0xFF);
        bytes.extend([0x40, 0x37]);
        bytes
    }

    #[test]
    fn crc_matches_real_sectors() {
        assert!(crc_ok(&real_profile()));
        assert!(crc_ok(&real_directory()));
        let mut broken = real_profile();
        broken[0x20] ^= 1;
        assert!(!crc_ok(&broken));
    }

    #[test]
    fn actions_round_trip() {
        for raw in [
            [0x80, 0x01, 0x00, 0x08],
            [0x80, 0x02, 0x01, 0x17],
            [0x80, 0x03, 0x00, 0xEA],
            [0x90, 0x07, 0x00, 0x00],
            [0xFF, 0xFF, 0xFF, 0xFF],
        ] {
            assert_eq!(encode_action(&decode_action(raw)), Some(raw), "{raw:02X?}");
        }
        assert!(matches!(
            decode_action([0x00, 0x03, 0x00, 0x10]),
            ButtonAction::Other { .. }
        ));
    }

    /// A mouse with onboard profiles at feature index 5 whose memory is
    /// `memory` (sector → bytes), writable like the real thing.
    fn mouse(memory: Memory, mode: u8) -> FakeTransport {
        let mut pending: Option<(u16, Vec<u8>, usize)> = None;
        FakeTransport::new(move |req| {
            let (dev, sub, addr) = (req[1], req[2], req[3]);
            let p = &req[4..];
            let payload: Vec<u8> = match (sub, addr >> 4) {
                (0, 0) => match u16::from_be_bytes([p[0], p[1]]) {
                    0x8100 => vec![5],
                    _ => vec![0],
                },
                (5, 0) => hex("01 05 01 05 02 0B 10 00 FF 0A 04"),
                (5, 2) => vec![mode],
                (5, 3) => vec![],
                (5, 4) => vec![0, 1],
                (5, 5) => {
                    let sector = u16::from_be_bytes([p[0], p[1]]);
                    let offset = u16::from_be_bytes([p[2], p[3]]) as usize;
                    memory.borrow()[&sector][offset..offset + 16].to_vec()
                }
                (5, 6) => {
                    let sector = u16::from_be_bytes([p[0], p[1]]);
                    let len = u16::from_be_bytes([p[4], p[5]]) as usize;
                    pending = Some((sector, Vec::new(), len));
                    vec![]
                }
                (5, 7) => {
                    if let Some((_, data, _)) = pending.as_mut() {
                        data.extend_from_slice(&p[..16]);
                    }
                    vec![]
                }
                (5, 8) => {
                    if let Some((sector, mut data, len)) = pending.take() {
                        data.truncate(len);
                        memory.borrow_mut().insert(sector, data);
                    }
                    vec![]
                }
                _ => return vec![vec![0x11, dev, 0xFF, sub, addr, 0x02, 0]],
            };
            vec![long(dev, sub, addr, &payload)]
        })
    }

    type Memory = Rc<RefCell<HashMap<u16, Vec<u8>>>>;

    /// Directory, profile 1 and a factory profile whose back button is
    /// "forward", so factory values are distinguishable.
    fn memory() -> Memory {
        let mut rom = real_profile();
        rom[0x2E..0x30].copy_from_slice(&[0x00, 0x10]);
        seal(&mut rom);
        Rc::new(RefCell::new(HashMap::from([
            (0x0000, real_directory()),
            (0x0001, real_profile()),
            (0x0101, rom),
        ])))
    }

    #[test]
    fn reads_the_real_layout() {
        let mut s = Session::new(mouse(memory(), 1), Duration::from_millis(50));
        let layout = read_buttons(&mut s, 1, "G502 X PLUS").unwrap().unwrap();
        assert_eq!(layout.onboard_profile, Some(1));
        assert!(layout.in_effect);
        assert_eq!(layout.buttons.len(), 11);
        assert_eq!(layout.buttons[3].name, "Back button");
        let factory = layout.factory_defaults.unwrap();
        assert_eq!(
            factory[3],
            ButtonAction::Mouse {
                button: MouseButton::Forward
            }
        );
        assert_eq!(
            layout.buttons[4].action,
            ButtonAction::Special {
                function: SpecialFunction::DpiShift
            }
        );
        assert_eq!(
            layout.buttons[9].action,
            ButtonAction::Special {
                function: SpecialFunction::DpiUp
            }
        );
    }

    #[test]
    fn changes_one_button_and_keeps_everything_else() {
        let mem = memory();
        let before = mem.borrow()[&1].clone();
        let mut s = Session::new(mouse(mem.clone(), 1), Duration::from_millis(50));
        let action = ButtonAction::Key {
            key: 0x2C,
            modifiers: 0,
        };
        let layout = set_buttons(&mut s, 1, "G502 X PLUS", &[(4, action.clone())]).unwrap();
        assert_eq!(layout.buttons[4].action, action);
        let after = mem.borrow()[&1].clone();
        assert!(crc_ok(&after));
        let changed: Vec<usize> = (0..after.len())
            .filter(|&i| after[i] != before[i])
            .collect();
        // Only the 4 assignment bytes of button 5 and the CRC.
        assert!(
            changed
                .iter()
                .all(|&i| (0x30..0x34).contains(&i) || i >= 0xFD),
            "{changed:?}"
        );
    }

    #[test]
    fn refuses_to_remove_the_last_left_click() {
        let mut s = Session::new(mouse(memory(), 1), Duration::from_millis(50));
        let err =
            set_buttons(&mut s, 1, "G502 X PLUS", &[(0, ButtonAction::Disabled)]).unwrap_err();
        assert!(matches!(err, Error::Io(m) if m.contains("left click")));
    }

    #[test]
    fn refuses_a_damaged_profile() {
        let mem = memory();
        mem.borrow_mut().get_mut(&1).unwrap()[0x22] ^= 0xFF;
        let mut s = Session::new(mouse(mem, 1), Duration::from_millis(50));
        assert!(read_buttons(&mut s, 1, "G502 X PLUS").is_err());
    }
}
