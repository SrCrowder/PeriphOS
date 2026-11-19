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

export type ChargingState = "discharging" | "charging" | "full" | "error";
export type BatteryLevel = "full" | "good" | "low" | "critical" | "empty";

export interface BatteryState {
  percent: number | null;
  level: BatteryLevel | null;
  charging: ChargingState;
  /** `percent` is estimated from the battery voltage. */
  estimated: boolean;
}

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
  battery: BatteryState | null;
  /** Receiver the device is connected through. */
  connectedVia: string | null;
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

export interface DpiSetting {
  current: number;
  /** Accepted values, ascending. */
  values: number[];
}

export interface ReportRateSetting {
  currentHz: number;
  valuesHz: number[];
}

/** Live settings read from the device. `null`: not supported. */
export interface DeviceSettings {
  dpi: DpiSetting | null;
  reportRate: ReportRateSetting | null;
  /** True when a profile stored in the device's memory is in control. */
  onboardProfileActive: boolean | null;
}

export type SettingChange = { type: "dpi"; value: number } | { type: "reportRateHz"; value: number };

export interface SettingResult {
  settings: DeviceSettings;
  /** False when the device reports a different value after the change. */
  tookEffect: boolean;
}
