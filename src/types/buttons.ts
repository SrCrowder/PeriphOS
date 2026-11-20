// Mirrors crates/periphos-core/src/device/buttons.rs.

export type MouseButton = "left" | "right" | "middle" | "back" | "forward";

export type SpecialFunction =
  | "wheelLeft"
  | "wheelRight"
  | "dpiUp"
  | "dpiDown"
  | "dpiCycle"
  | "dpiDefault"
  | "dpiShift"
  | "profileNext"
  | "profilePrevious"
  | "profileCycle"
  | "gShift"
  | "batteryLevel";

export type ButtonAction =
  | { type: "mouse"; button: MouseButton }
  /** `key`: HID keyboard usage. `modifiers`: bit 0 Ctrl, 1 Shift, 2 Alt, 3 Windows. */
  | { type: "key"; key: number; modifiers: number }
  /** HID consumer usage (media and system keys). */
  | { type: "media"; usage: number }
  | { type: "special"; function: SpecialFunction }
  | { type: "disabled" }
  /** Something PeriphOS cannot edit yet, such as a macro. */
  | { type: "other"; description: string };

export interface ButtonBinding {
  index: number;
  name: string;
  action: ButtonAction;
}

export interface ButtonLayout {
  buttons: ButtonBinding[];
  /** Onboard profile (1-based) the assignments are stored in. */
  onboardProfile: number | null;
  /** False when the device is not using its onboard profiles right now. */
  inEffect: boolean;
  specialFunctions: SpecialFunction[];
  /** Factory assignments in button order, when the device stores them. */
  factoryDefaults: ButtonAction[] | null;
}

export interface ButtonAssignment {
  index: number;
  action: ButtonAction;
}

/** Assignments the user saved as a device's own defaults. */
export interface ButtonDefaults {
  deviceId: string;
  deviceName: string;
  actions: ButtonAction[];
  updatedAt: number;
}
