//! Button assignments: what each programmable button of a device does.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// Functions handled by the device itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpecialFunction {
    WheelLeft,
    WheelRight,
    DpiUp,
    DpiDown,
    DpiCycle,
    DpiDefault,
    /// Lower sensitivity while held (a "sniper" button).
    DpiShift,
    ProfileNext,
    ProfilePrevious,
    ProfileCycle,
    /// Second layer of assignments while held.
    GShift,
    BatteryLevel,
}

/// What pressing a button does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ButtonAction {
    Mouse {
        button: MouseButton,
    },
    /// A keyboard key (HID usage, keyboard page) with modifiers held:
    /// bit 0 Ctrl, 1 Shift, 2 Alt, 3 Windows key.
    Key {
        key: u8,
        modifiers: u8,
    },
    /// A media or system key (HID usage, consumer page), e.g. volume up.
    Media {
        usage: u16,
    },
    Special {
        function: SpecialFunction,
    },
    Disabled,
    /// An assignment PeriphOS cannot edit yet, such as a macro. It is kept
    /// as it is unless the user replaces it.
    Other {
        description: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonBinding {
    /// Position in the device's list of buttons.
    pub index: u8,
    pub name: String,
    pub action: ButtonAction,
}

/// The device's buttons and where their assignments live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonLayout {
    pub buttons: Vec<ButtonBinding>,
    /// The onboard profile (1-based) the assignments are stored in, for
    /// devices that keep them in their own memory.
    pub onboard_profile: Option<u8>,
    /// False when the device does not use its onboard profiles right now, so
    /// stored assignments have no effect until that mode is turned on.
    pub in_effect: bool,
    /// Device functions that can be assigned.
    pub special_functions: Vec<SpecialFunction>,
}

impl ButtonLayout {
    /// Rejects layouts that would leave no button able to left-click.
    pub fn keeps_left_click(&self) -> bool {
        self.buttons.iter().any(|b| {
            b.action
                == ButtonAction::Mouse {
                    button: MouseButton::Left,
                }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_serialize_with_a_type_tag() {
        let json = serde_json::to_value(ButtonAction::Key {
            key: 0x04,
            modifiers: 1,
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "key", "key": 4, "modifiers": 1 })
        );
        let json = serde_json::to_value(ButtonAction::Special {
            function: SpecialFunction::DpiShift,
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "special", "function": "dpiShift" })
        );
    }
}
