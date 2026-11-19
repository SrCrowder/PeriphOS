use serde::{Deserialize, Serialize};

/// How PeriphOS can provide a given capability for a specific device.
///
/// This is the single most important distinction in the UI: users must be
/// able to see whether a setting lives on the device itself or is emulated
/// by PeriphOS while it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportLevel {
    /// Configured on the hardware (onboard memory / device registers) through
    /// a protocol that PeriphOS implements. Persists without PeriphOS running.
    Native,
    /// Emulated by PeriphOS at the operating-system input layer. Only works
    /// while PeriphOS (or its background service) is running.
    Software,
    /// The hardware has the feature, but it is only reachable through a
    /// manufacturer-specific protocol that PeriphOS does not implement yet.
    VendorProtocol,
    /// Not available for this device.
    Unsupported,
}

/// A feature a device may expose, with the parameters needed to render a
/// correct configuration UI for it. Details the manufacturer does not publish
/// are optional rather than guessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CapabilityKind {
    /// Remapping of physical mouse buttons.
    ButtonRemap { programmable: Option<u8> },
    /// Remapping of keyboard keys.
    KeyRemap,
    /// Analog (e.g. Hall effect) keys: adjustable actuation point, rapid trigger.
    AnalogKeys,
    /// Playback of recorded or authored macros.
    Macros,
    /// Sensor resolution control.
    Dpi {
        min: u32,
        max: u32,
        step: Option<u32>,
    },
    /// USB report rate control.
    PollingRate { max_hz: u32 },
    /// Lighting control, per key or in zones.
    Lighting { per_key: bool, zones: Option<u16> },
    /// Profiles stored in device memory.
    OnboardProfiles { slots: u8 },
    /// Battery level reporting.
    Battery,
}

/// A capability together with how it is supported on a given device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub kind: CapabilityKind,
    pub support: SupportLevel,
    /// Optional human-readable caveat (e.g. "requires the wireless receiver").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Capability {
    pub fn new(kind: CapabilityKind, support: SupportLevel) -> Self {
        Self {
            kind,
            support,
            note: None,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Whether PeriphOS can actually configure this capability today.
    pub fn is_configurable(&self) -> bool {
        matches!(self.support, SupportLevel::Native | SupportLevel::Software)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_native_and_software_are_configurable() {
        let make = |support| Capability::new(CapabilityKind::Macros, support);
        assert!(make(SupportLevel::Native).is_configurable());
        assert!(make(SupportLevel::Software).is_configurable());
        assert!(!make(SupportLevel::VendorProtocol).is_configurable());
        assert!(!make(SupportLevel::Unsupported).is_configurable());
    }

    #[test]
    fn capability_serializes_with_tagged_kind() {
        let cap = Capability::new(
            CapabilityKind::PollingRate { max_hz: 8000 },
            SupportLevel::VendorProtocol,
        );
        let json = serde_json::to_value(&cap).unwrap();
        assert_eq!(json["kind"]["type"], "pollingRate");
        assert_eq!(json["kind"]["maxHz"], 8000);
        assert_eq!(json["support"], "vendorProtocol");
        assert!(json.get("note").is_none());
    }
}
