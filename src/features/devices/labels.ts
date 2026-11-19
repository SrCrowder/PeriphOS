import type { Capability, ConnectionType, DeviceKind } from "@/types/device";

export const KIND_LABEL: Record<DeviceKind, string> = {
  mouse: "Mouse",
  keyboard: "Keyboard",
  headset: "Headset",
  controller: "Controller",
  receiver: "Wireless receiver",
  other: "Device",
};

export const CONNECTION_LABEL: Record<ConnectionType, string> = {
  usb: "USB",
  bluetooth: "Bluetooth",
  receiver: "USB receiver",
  unknown: "Unknown connection",
};

const number = new Intl.NumberFormat("en");

/** Name and a short, factual detail for a capability. */
export function describeCapability(capability: Capability): { label: string; detail: string | null } {
  const k = capability.kind;
  switch (k.type) {
    case "buttonRemap":
      return {
        label: "Button remapping",
        detail: k.programmable ? `${k.programmable} programmable controls` : null,
      };
    case "keyRemap":
      return { label: "Key remapping", detail: null };
    case "analogKeys":
      return { label: "Analog keys", detail: null };
    case "macros":
      return { label: "Macros", detail: null };
    case "dpi":
      return {
        label: "Sensitivity (DPI)",
        detail: `${number.format(k.min)} to ${number.format(k.max)} DPI${k.step ? `, steps of ${k.step}` : ""}`,
      };
    case "pollingRate":
      return { label: "Polling rate", detail: `Up to ${number.format(k.maxHz)} Hz` };
    case "lighting":
      return {
        label: "Lighting",
        detail: k.perKey ? "Per-key RGB" : k.zones ? `${k.zones} RGB zones` : null,
      };
    case "onboardProfiles":
      return { label: "Onboard profiles", detail: `${k.slots} slots in device memory` };
    case "battery":
      return { label: "Battery level", detail: null };
  }
}
