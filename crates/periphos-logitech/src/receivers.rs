//! Logitech receivers PeriphOS talks to. Product IDs as listed by the Solaar
//! project (https://github.com/pwr-Solaar/Solaar, lib/logitech_receiver/base_usb.py).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverFamily {
    Unifying,
    Lightspeed,
    Nano,
    Bolt,
}

#[derive(Debug, Clone, Copy)]
pub struct ReceiverInfo {
    pub family: ReceiverFamily,
    pub name: &'static str,
}

pub fn receiver(product_id: u16) -> Option<ReceiverInfo> {
    use ReceiverFamily::*;
    let family = match product_id {
        0xC52B | 0xC532 => Unifying,
        0xC539 | 0xC53A | 0xC53D | 0xC53F | 0xC541 | 0xC545 | 0xC547 | 0xC54D => Lightspeed,
        0xC548 => Bolt,
        0xC518 | 0xC51A | 0xC51B | 0xC521 | 0xC525 | 0xC526 | 0xC52E | 0xC52F | 0xC531 | 0xC534
        | 0xC535 | 0xC537 => Nano,
        _ => return None,
    };
    let name = match family {
        Unifying => "Unifying receiver",
        Lightspeed => "LIGHTSPEED receiver",
        Nano => "Nano receiver",
        Bolt => "Bolt receiver",
    };
    Some(ReceiverInfo { family, name })
}
