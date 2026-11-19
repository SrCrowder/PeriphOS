/**
 * Typed bridge to the Rust backend.
 *
 * Every backend command is declared once in {@link Commands}; callers use
 * the `api` object and get full argument and return typing. When the UI runs
 * in a plain browser (`npm run dev` without Tauri) calls are routed to an
 * in-memory preview backend so the interface can be developed and reviewed.
 */
import type { AppInfo } from "@/types/app";
import type { DeviceOverview, DeviceSettings, SettingChange, SettingResult } from "@/types/device";
import type { ActivationReport, DeviceProfileSettings, Profile, ProfileDraft } from "@/types/profile";
import type { AppSettings } from "@/types/settings";
import type { UpdateCheck, UpdateStatus } from "@/types/updates";

import { toIpcError } from "./errors";

export interface Commands {
  get_app_info: { args: undefined; result: AppInfo };
  get_device_overview: { args: undefined; result: DeviceOverview };
  get_device_settings: { args: { deviceId: string }; result: DeviceSettings };
  apply_device_setting: { args: { deviceId: string; change: SettingChange }; result: SettingResult };
  list_profiles: { args: undefined; result: Profile[] };
  create_profile: { args: { draft: ProfileDraft }; result: Profile };
  update_profile: { args: { id: string; draft: ProfileDraft }; result: Profile };
  delete_profile: { args: { id: string }; result: null };
  set_default_profile: { args: { id: string }; result: Profile };
  get_active_profile: { args: undefined; result: Profile };
  activate_profile: { args: { id: string }; result: ActivationReport };
  list_profile_device_settings: { args: { profileId: string }; result: DeviceProfileSettings[] };
  save_device_settings_to_profile: {
    args: { profileId: string; deviceId: string };
    result: DeviceProfileSettings;
  };
  delete_profile_device_settings: { args: { profileId: string; deviceId: string }; result: null };
  get_settings: { args: undefined; result: AppSettings };
  update_settings: { args: { settings: AppSettings }; result: AppSettings };
  get_update_status: { args: undefined; result: UpdateStatus };
  check_for_update: { args: undefined; result: UpdateCheck };
  install_update: { args: undefined; result: null };
}

export type CommandName = keyof Commands;
type Args<C extends CommandName> = Commands[C]["args"];
type Result<C extends CommandName> = Commands[C]["result"];

/** True when running inside the Tauri webview. */
export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<C extends CommandName>(command: C, args?: Args<C>): Promise<Result<C>> {
  try {
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<Result<C>>(command, args ?? {});
    }
    const { previewBackend } = await import("./previewBackend");
    return await previewBackend(command, args);
  } catch (error) {
    throw toIpcError(error);
  }
}

export const api = {
  appInfo: () => call("get_app_info"),
  deviceOverview: () => call("get_device_overview"),
  deviceSettings: (deviceId: string) => call("get_device_settings", { deviceId }),
  applyDeviceSetting: (deviceId: string, change: SettingChange) =>
    call("apply_device_setting", { deviceId, change }),
  listProfiles: () => call("list_profiles"),
  createProfile: (draft: ProfileDraft) => call("create_profile", { draft }),
  updateProfile: (id: string, draft: ProfileDraft) => call("update_profile", { id, draft }),
  deleteProfile: (id: string) => call("delete_profile", { id }),
  setDefaultProfile: (id: string) => call("set_default_profile", { id }),
  activeProfile: () => call("get_active_profile"),
  activateProfile: (id: string) => call("activate_profile", { id }),
  profileDeviceSettings: (profileId: string) => call("list_profile_device_settings", { profileId }),
  saveDeviceSettingsToProfile: (profileId: string, deviceId: string) =>
    call("save_device_settings_to_profile", { profileId, deviceId }),
  deleteProfileDeviceSettings: (profileId: string, deviceId: string) =>
    call("delete_profile_device_settings", { profileId, deviceId }),
  getSettings: () => call("get_settings"),
  updateSettings: (settings: AppSettings) => call("update_settings", { settings }),
  updateStatus: () => call("get_update_status"),
  checkForUpdate: () => call("check_for_update"),
  installUpdate: () => call("install_update"),
};
