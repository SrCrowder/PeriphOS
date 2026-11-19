//! Wooting configuration protocol: 8-byte feature commands on the keyboard's
//! configuration HID interface, answered with an input report.
//!
//! Implemented from publicly available protocol facts: the report layout and
//! interface usage pages as used by Wooting's open RGB SDK, and the profile
//! command numbers as used by Wooting Profile Switcher (ShayBox, MIT licence).
//! No code from either project is used.

/// Usage page of the configuration interface (firmware before 2.12).
pub const USAGE_PAGE_V2: u16 = 0x1337;
/// Usage page of the configuration interface with numbered, single-report responses.
pub const USAGE_PAGE_V3: u16 = 0xFF55;

pub const COMMAND_SIZE: usize = 8;
const V2_RESPONSE_SIZE: usize = 256;
const V3_RESPONSE_SIZE: usize = 2046;

/// Command numbers.
pub mod command {
    pub const RELOAD_PROFILE: u8 = 7;
    pub const GET_CURRENT_PROFILE_INDEX: u8 = 11;
    pub const ACTIVATE_PROFILE: u8 = 23;
    pub const REFRESH_RGB_COLORS: u8 = 29;
    pub const RESET_ALL_COLORS: u8 = 32;
}

/// Profiles stored on current Wooting keyboards.
pub const PROFILE_COUNT: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    V2,
    V3,
}

impl Interface {
    pub fn from_usage_page(page: u16) -> Option<Self> {
        match page {
            USAGE_PAGE_V2 => Some(Self::V2),
            USAGE_PAGE_V3 => Some(Self::V3),
            _ => None,
        }
    }

    /// Size of a complete response.
    pub fn response_size(self) -> usize {
        match self {
            Self::V2 => V2_RESPONSE_SIZE,
            Self::V3 => V3_RESPONSE_SIZE,
        }
    }

    /// V3 answers in a single numbered report that may be shorter than the
    /// maximum size; V2 answers may span several reads.
    pub fn single_read(self) -> bool {
        self == Self::V3
    }

    /// Position of the first payload byte in a response.
    pub fn payload_offset(self) -> usize {
        match self {
            // Report ID, then the V2 header.
            Self::V3 => 1 + 5 + 1,
            Self::V2 => 5,
        }
    }
}

/// `[report id, magic (2 bytes), command, parameter 3, 2, 1, 0]`.
pub fn encode(interface: Interface, command: u8, parameter0: u8) -> [u8; COMMAND_SIZE] {
    let (report_id, magic) = match interface {
        Interface::V2 => (0x00, 0xD0),
        Interface::V3 => (0x01, 0xD1),
    };
    [report_id, magic, 0xDA, command, 0, 0, 0, parameter0]
}

/// Active profile index from a GET_CURRENT_PROFILE_INDEX response.
pub fn profile_index(interface: Interface, response: &[u8]) -> Option<u8> {
    response
        .get(interface.payload_offset())
        .copied()
        .filter(|&i| i < PROFILE_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_layout() {
        assert_eq!(
            encode(Interface::V2, command::ACTIVATE_PROFILE, 2),
            [0x00, 0xD0, 0xDA, 23, 0, 0, 0, 2]
        );
        assert_eq!(
            encode(Interface::V3, command::GET_CURRENT_PROFILE_INDEX, 0),
            [0x01, 0xD1, 0xDA, 11, 0, 0, 0, 0]
        );
    }

    #[test]
    fn interface_from_usage_page() {
        assert_eq!(Interface::from_usage_page(0x1337), Some(Interface::V2));
        assert_eq!(Interface::from_usage_page(0xFF55), Some(Interface::V3));
        assert_eq!(Interface::from_usage_page(0x0001), None);
    }

    #[test]
    fn profile_index_offsets() {
        let mut v2 = vec![0u8; 256];
        v2[5] = 2;
        assert_eq!(profile_index(Interface::V2, &v2), Some(2));
        let mut v3 = vec![0u8; 64];
        v3[7] = 3;
        assert_eq!(profile_index(Interface::V3, &v3), Some(3));
        // Out-of-range values and short responses are rejected.
        v3[7] = 9;
        assert_eq!(profile_index(Interface::V3, &v3), None);
        assert_eq!(profile_index(Interface::V3, &[0, 1]), None);
    }
}
