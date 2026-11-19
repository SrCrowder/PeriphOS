// Mirrors src-tauri/src/commands/updates.rs.
import type { UpdateChannel } from "./settings";

export interface UpdateStatus {
  currentVersion: string;
  /** Channel in effect: the user's choice or the build's own channel. */
  channel: UpdateChannel;
  buildChannel: UpdateChannel;
  /** False for local builds that were not signed for updates. */
  configured: boolean;
}

export interface UpdateCheck {
  available: boolean;
  currentVersion: string;
  version: string | null;
  notes: string | null;
  date: string | null;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
}

export const UPDATE_PROGRESS_EVENT = "update-progress";
