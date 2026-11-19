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
