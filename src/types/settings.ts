// Mirrors crates/periphos-core/src/settings.

export type Theme = "dark" | "light" | "system";
export type Accent = "orange" | "violet" | "blue" | "cyan" | "green" | "rose";

export const ACCENTS: readonly Accent[] = ["orange", "violet", "blue", "cyan", "green", "rose"];

export interface AppSettings {
  theme: Theme;
  accent: Accent;
  reduceMotion: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme: "dark",
  accent: "orange",
  reduceMotion: false,
};
