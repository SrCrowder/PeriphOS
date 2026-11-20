import type { ButtonAction } from "./buttons";

// Mirrors crates/periphos-core/src/profile.

export const PROFILE_NAME_MAX = 64;
export const PROFILE_DESCRIPTION_MAX = 280;

export interface Profile {
  id: string;
  name: string;
  description: string | null;
  isDefault: boolean;
  /** Unix milliseconds. */
  createdAt: number;
  /** Unix milliseconds. */
  updatedAt: number;
}

export interface ProfileDraft {
  name: string;
  description: string | null;
}

/** Device settings saved in a profile. */
export interface DeviceProfileSettings {
  deviceId: string;
  deviceName: string;
  dpi: number | null;
  reportRateHz: number | null;
  /** Zero-based onboard profile. */
  onboardProfile: number | null;
  /** Button assignments for this profile; `null`: the device's saved defaults. */
  buttons: ButtonAction[] | null;
  updatedAt: number;
}

export interface DeviceApplyResult {
  deviceId: string;
  deviceName: string;
  applied: boolean;
  message: string | null;
}

/** `automatic`: follows the game or app in front. `manual`: chosen by the user. */
export type SwitchMode = "automatic" | "manual";

export interface ActivationReport {
  profile: Profile;
  mode: SwitchMode;
  /** Linked app that selected the profile, in automatic mode. */
  matchedApp: string | null;
  reason: "activated" | "appSwitched" | "deviceConnected";
  results: DeviceApplyResult[];
}

export interface ActiveProfileInfo {
  profile: Profile;
  mode: SwitchMode;
  matchedApp: string | null;
  /** Whether this platform can follow the foreground app. */
  automaticSupported: boolean;
}

export const PROFILE_APPLIED_EVENT = "profile-applied";
