// Mirrors crates/periphos-core/src/device. Keep in sync with the Rust types.

export type Platform = "windows" | "linux" | "macOs";

/** How PeriphOS can provide a capability for a specific device. */
export type SupportLevel = "native" | "software" | "vendorProtocol" | "unsupported";

/** Details the manufacturer does not publish are `null`. */
export type CapabilityKind =
  | { type: "buttonRemap"; programmable: number | null }
  | { type: "keyRemap" }
  | { type: "analogKeys" }
  | { type: "macros" }
  | { type: "dpi"; min: number; max: number; step: number | null }
  | { type: "pollingRate"; maxHz: number }
  | { type: "lighting"; perKey: boolean; zones: number | null }
  | { type: "onboardProfiles"; slots: number }
  | { type: "battery" };

export interface Capability {
  kind: CapabilityKind;
  support: SupportLevel;
  note?: string;
}

export type DeviceKind = "mouse" | "keyboard" | "headset" | "controller" | "receiver" | "other";
/** `virtual`: created by software (e.g. a manufacturer app), not hardware. */
export type ConnectionType = "usb" | "bluetooth" | "receiver" | "virtual" | "internal" | "unknown";

export interface DeviceInfo {
  id: string;
  driverId: string;
  name: string;
  manufacturer: string;
  vendorId: number;
  productId: number;
  kind: DeviceKind;
  connection: ConnectionType;
  capabilities: Capability[];
  /** In PeriphOS's device database: name and capabilities are known. */
  knownModel: boolean;
  interfaceCount: number;
  note: string | null;
}

export interface DriverInfo {
  id: string;
  name: string;
  description: string;
  platforms: Platform[];
}

export interface DriverError {
  driverId: string;
  message: string;
}

export const DEVICES_CHANGED_EVENT = "devices-changed";

export interface DeviceOverview {
  platform: Platform;
  drivers: DriverInfo[];
  devices: DeviceInfo[];
  driverErrors: DriverError[];
}
