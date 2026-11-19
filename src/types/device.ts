// Mirrors crates/periphos-core/src/device. Keep in sync with the Rust types.

export type Platform = "windows" | "linux" | "macOs";

/** How PeriphOS can provide a capability for a specific device. */
export type SupportLevel = "native" | "software" | "vendorProtocol" | "unsupported";

export type CapabilityKind =
  | { type: "buttonRemap"; buttons: number }
  | { type: "keyRemap" }
  | { type: "macros" }
  | { type: "dpi"; min: number; max: number; step: number; stages: number }
  | { type: "pollingRate"; rates: number[] }
  | { type: "lighting"; zones: number }
  | { type: "onboardProfiles"; slots: number }
  | { type: "battery" };

export interface Capability {
  kind: CapabilityKind;
  support: SupportLevel;
  note?: string;
}

export type DeviceKind = "mouse" | "keyboard" | "headset" | "controller" | "other";
export type ConnectionType = "usb" | "bluetooth" | "receiver" | "unknown";

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

export interface DeviceOverview {
  platform: Platform;
  drivers: DriverInfo[];
  devices: DeviceInfo[];
  driverErrors: DriverError[];
}
