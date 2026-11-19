// Mirrors crates/periphos-core/src/settings.

export type Theme = "dark" | "light" | "system";
export type Accent = "orange" | "violet" | "blue" | "cyan" | "green" | "rose";

export const ACCENTS: readonly Accent[] = ["orange", "violet", "blue", "cyan", "green", "rose"];

export type UpdateChannel = "stable" | "development";

export interface AppSettings {
  theme: Theme;
  accent: Accent;
  reduceMotion: boolean;
  /** `null` = the channel this build was published on. */
  updateChannel: UpdateChannel | null;
  autoCheckUpdates: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme: "dark",
  accent: "orange",
  reduceMotion: false,
  updateChannel: null,
  autoCheckUpdates: true,
};
