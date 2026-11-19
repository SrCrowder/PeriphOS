import type { Platform } from "./device";

export interface AppInfo {
  version: string;
  platform: Platform;
  databasePath: string;
  schemaVersion: number;
}

/** Error shape returned by every backend command. */
export interface IpcError {
  code: "not_found" | "validation" | "conflict" | "unsupported" | "storage" | "device" | "update" | "unknown";
  message: string;
}
