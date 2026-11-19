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
  updatedAt: number;
}

export interface DeviceApplyResult {
  deviceId: string;
  deviceName: string;
  applied: boolean;
  message: string | null;
}

export interface ActivationReport {
  profile: Profile;
  reason: "activated" | "deviceConnected";
  results: DeviceApplyResult[];
}

export const PROFILE_APPLIED_EVENT = "profile-applied";
