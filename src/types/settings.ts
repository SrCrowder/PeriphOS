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
  /** Keep running in the system tray when the window is closed. */
  minimizeToTray: boolean;
  /** Notify when a wireless device's battery runs low. */
  batteryAlerts: boolean;
  batteryWarningPercent: number;
  /** Always below the warning level. */
  batteryCriticalPercent: number;
  /** How battery levels are shown outside the window. */
  batteryIndicator: BatteryIndicator;
}

/** `taskbar`: text next to the tray (Windows). `trayIcons`: one tray icon per device. */
export type BatteryIndicator = "taskbar" | "trayIcons" | "off";

export const DEFAULT_SETTINGS: AppSettings = {
  theme: "dark",
  accent: "orange",
  reduceMotion: false,
  updateChannel: null,
  autoCheckUpdates: true,
  minimizeToTray: true,
  batteryAlerts: true,
  batteryWarningPercent: 20,
  batteryCriticalPercent: 10,
  batteryIndicator: "taskbar",
};
