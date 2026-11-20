/**
 * In-memory stand-in for the Rust backend, used ONLY when the frontend runs
 * in a regular browser instead of the Tauri webview. It mirrors the rules of
 * `periphos-storage` (validation, unique names, single default profile) so
 * the UI behaves the same, but nothing is persisted and no hardware is ever
 * reported. The app shows a banner whenever this backend is active.
 *
 * For UI work on device pages, `npm run dev` with `?fixtures` in the URL adds
 * the example devices from `previewFixtures.ts` (development server only).
 */
import type { IpcError } from "@/types/app";
import type { ButtonDefaults, ButtonLayout } from "@/types/buttons";
import type { DeviceInfo, DeviceSettings } from "@/types/device";
import type { AppRule } from "@/types/apps";
import type { DeviceProfileSettings, Profile, ProfileDraft, SwitchMode } from "@/types/profile";
import { PROFILE_DESCRIPTION_MAX, PROFILE_NAME_MAX } from "@/types/profile";
import { DEFAULT_SETTINGS, type AppSettings } from "@/types/settings";

import type { CommandName, Commands } from "./ipc";

const now = Date.now();
let profiles: Profile[] = [
  {
    id: crypto.randomUUID(),
    name: "Default",
    description: "Used when no application-specific profile applies.",
    isDefault: true,
    createdAt: now,
    updatedAt: now,
  },
];
let settings: AppSettings = { ...DEFAULT_SETTINGS };
let activeProfileId: string | null = null;
let launchAtStartup = false;
let switchMode: SwitchMode = "automatic";
let appRules: AppRule[] = [];
let deviceSettings: (DeviceProfileSettings & { profileId: string })[] = [];

function activeProfile(): Profile {
  return profiles.find((p) => p.id === activeProfileId) ?? profiles.find((p) => p.isDefault)!;
}

let fixtureDevices: DeviceInfo[] = [];
let fixtureSettings = new Map<string, DeviceSettings>();
let fixtureButtons = new Map<string, ButtonLayout>();
const buttonDefaults = new Map<string, ButtonDefaults>();
if (import.meta.env.DEV && new URLSearchParams(location.search).has("fixtures")) {
  const fixtures = await import("./previewFixtures");
  fixtureDevices = fixtures.PREVIEW_FIXTURE_DEVICES;
  fixtureSettings = fixtures.previewFixtureSettings();
  fixtureButtons = fixtures.previewFixtureButtons();
}

function fail(code: IpcError["code"], message: string): never {
  throw { code, message } satisfies IpcError;
}

function normalize(draft: ProfileDraft): ProfileDraft {
  const name = draft.name.trim();
  const description = draft.description?.trim() || null;
  if (!name) fail("validation", "Profile name cannot be empty.");
  if ([...name].length > PROFILE_NAME_MAX)
    fail("validation", `Profile name cannot be longer than ${PROFILE_NAME_MAX} characters.`);
  if (description && [...description].length > PROFILE_DESCRIPTION_MAX)
    fail("validation", `Description cannot be longer than ${PROFILE_DESCRIPTION_MAX} characters.`);
  return { name, description };
}

function assertUniqueName(name: string, exceptId?: string) {
  const taken = profiles.some(
    (p) => p.id !== exceptId && p.name.toLowerCase() === name.toLowerCase(),
  );
  if (taken) fail("conflict", `A profile named "${name}" already exists.`);
}

function find(id: string): Profile {
  return profiles.find((p) => p.id === id) ?? fail("not_found", `Profile '${id}' was not found`);
}

function sorted(): Profile[] {
  return [...profiles].sort(
    (a, b) => Number(b.isDefault) - Number(a.isDefault) || a.name.localeCompare(b.name),
  );
}

const handlers: { [C in CommandName]: (args: Commands[C]["args"]) => Commands[C]["result"] } = {
  get_app_info: () => ({
    version: "0.1.0",
    platform: navigator.userAgent.includes("Windows") ? "windows" : "linux",
    databasePath: "In-memory (browser preview)",
    schemaVersion: 1,
  }),
  get_device_overview: () => ({
    platform: navigator.userAgent.includes("Windows") ? "windows" : "linux",
    drivers: [],
    devices: fixtureDevices,
    driverErrors: [],
  }),
  list_profiles: () => sorted(),
  create_profile: ({ draft }) => {
    const clean = normalize(draft);
    assertUniqueName(clean.name);
    const t = Date.now();
    const profile: Profile = {
      id: crypto.randomUUID(),
      ...clean,
      isDefault: false,
      createdAt: t,
      updatedAt: t,
    };
    profiles.push(profile);
    return profile;
  },
  update_profile: ({ id, draft }) => {
    const clean = normalize(draft);
    find(id);
    assertUniqueName(clean.name, id);
    profiles = profiles.map((p) => (p.id === id ? { ...p, ...clean, updatedAt: Date.now() } : p));
    return find(id);
  },
  delete_profile: ({ id }) => {
    if (find(id).isDefault)
      fail(
        "conflict",
        "The default profile cannot be deleted. Set another profile as default first.",
      );
    profiles = profiles.filter((p) => p.id !== id);
    deviceSettings = deviceSettings.filter((s) => s.profileId !== id);
    appRules = appRules.filter((r) => r.profileId !== id);
    if (activeProfileId === id) activeProfileId = null;
    return null;
  },
  get_active_profile: () => ({
    profile: activeProfile(),
    mode: switchMode,
    matchedApp: null,
    automaticSupported: false,
  }),
  resume_automatic_switching: () => {
    switchMode = "automatic";
    activeProfileId = null;
    return { profile: activeProfile(), mode: switchMode, matchedApp: null, automaticSupported: false };
  },
  list_app_rules: ({ profileId }) => appRules.filter((r) => r.profileId === profileId),
  add_app_rule: ({ profileId, kind, value, displayName }) => {
    const profile = find(profileId);
    if (profile.isDefault)
      fail("validation", "The default profile is used when no linked app is running, so apps cannot be linked to it.");
    const existing = appRules.find((r) => r.kind === kind && r.value.toLowerCase() === value.toLowerCase());
    if (existing) fail("conflict", `${displayName} is already linked to "${find(existing.profileId).name}".`);
    const rule: AppRule = { id: crypto.randomUUID(), profileId, kind, value, displayName, createdAt: Date.now() };
    appRules.push(rule);
    return rule;
  },
  delete_app_rule: ({ id }) => {
    appRules = appRules.filter((r) => r.id !== id);
    return null;
  },
  list_running_apps: () =>
    fixtureDevices.length > 0
      ? [
          { exeName: "ForzaHorizon5.exe", exePath: "C:\\XboxGames\\Forza Horizon 5\\Content\\ForzaHorizon5.exe", packageFamily: "Example.ForzaHorizon5_8wekyb3d8bbwe", title: "Forza Horizon 5" },
          { exeName: "cs2.exe", exePath: "D:\\Steam\\steamapps\\common\\Counter-Strike Global Offensive\\game\\bin\\win64\\cs2.exe", packageFamily: null, title: "Counter-Strike 2" },
        ]
      : [],
  list_installed_packages: () =>
    fixtureDevices.length > 0
      ? [
          { packageFamily: "Example.ForzaHorizon5_8wekyb3d8bbwe", displayName: "Forza Horizon 5", installPath: null, isGame: true },
          { packageFamily: "Example.SeaOfThieves_8wekyb3d8bbwe", displayName: "Sea of Thieves", installPath: null, isGame: true },
          { packageFamily: "Example.Notepad_8wekyb3d8bbwe", displayName: "Notepad", installPath: null, isGame: false },
        ]
      : [],
  activate_profile: ({ id }) => {
    const profile = find(id);
    activeProfileId = id;
    switchMode = "manual";
    const connected = new Set(fixtureDevices.map((d) => d.id));
    const results = deviceSettings
      .filter((s) => s.profileId === id && connected.has(s.deviceId))
      .map((s) => {
        const live = fixtureSettings.get(s.deviceId);
        if (live?.dpi && s.dpi) live.dpi.current = s.dpi;
        if (live?.reportRate && s.reportRateHz) live.reportRate.currentHz = s.reportRateHz;
        if (live?.onboardProfile && s.onboardProfile != null) live.onboardProfile.current = s.onboardProfile;
        return { deviceId: s.deviceId, deviceName: s.deviceName, applied: true, message: null };
      });
    return { profile, mode: "manual", matchedApp: null, reason: "activated", results };
  },
  list_profile_device_settings: ({ profileId }) =>
    deviceSettings.filter((s) => s.profileId === profileId).map(({ profileId: _, ...s }) => s),
  save_device_settings_to_profile: ({ profileId, deviceId }) => {
    find(profileId);
    const device = fixtureDevices.find((d) => d.id === deviceId) ?? fail("device", "The device is not connected.");
    const live = fixtureSettings.get(deviceId) ?? fail("unsupported", "this device's settings cannot be read yet");
    const entry = {
      profileId,
      deviceId,
      deviceName: device.name,
      dpi: live.dpi?.current ?? null,
      reportRateHz: live.reportRate?.currentHz ?? null,
      onboardProfile: live.onboardProfile?.current ?? null,
      updatedAt: Date.now(),
    };
    deviceSettings = [...deviceSettings.filter((s) => !(s.profileId === profileId && s.deviceId === deviceId)), entry];
    const { profileId: _, ...saved } = entry;
    return saved;
  },
  delete_profile_device_settings: ({ profileId, deviceId }) => {
    deviceSettings = deviceSettings.filter((s) => !(s.profileId === profileId && s.deviceId === deviceId));
    return null;
  },
  set_default_profile: ({ id }) => {
    find(id);
    profiles = profiles.map((p) =>
      p.id === id ? { ...p, isDefault: true, updatedAt: Date.now() } : { ...p, isDefault: false },
    );
    return find(id);
  },
  get_device_settings: ({ deviceId }) => {
    const settings = fixtureSettings.get(deviceId);
    return settings ? structuredClone(settings) : fail("unsupported", "this device's settings cannot be read yet");
  },
  get_device_buttons: ({ deviceId }) =>
    fixtureButtons.has(deviceId)
      ? structuredClone(fixtureButtons.get(deviceId)!)
      : fail("unsupported", "this device's buttons cannot be read yet"),
  set_device_buttons: ({ deviceId, assignments }) => {
    const layout = fixtureButtons.get(deviceId) ?? fail("unsupported", "this device's buttons cannot be changed yet");
    const next = structuredClone(layout.buttons);
    for (const { index, action } of assignments) {
      const button = next[index] ?? fail("validation", "There is no such button on this device.");
      if (action.type !== "other") button.action = action;
    }
    if (!next.some((b) => b.action.type === "mouse" && b.action.button === "left")) {
      return fail("device", "At least one button must keep the left click, or the mouse could not click.");
    }
    layout.buttons = next;
    return structuredClone(layout);
  },
  get_button_defaults: ({ deviceId }) => structuredClone(buttonDefaults.get(deviceId) ?? null),
  save_button_defaults: ({ deviceId, deviceName, actions }) => {
    if (actions.length === 0) return fail("validation", "There are no buttons to save.");
    const saved = { deviceId, deviceName, actions: structuredClone(actions), updatedAt: Date.now() };
    buttonDefaults.set(deviceId, saved);
    return structuredClone(saved);
  },
  delete_button_defaults: ({ deviceId }) => {
    buttonDefaults.delete(deviceId);
    return null;
  },
  device_diagnostics: () =>
    "HID++ protocol 4.2\n\n## Features\n  1  0x0001 v2  Feature set\n(Browser preview: sample text, no device was read.)",
  apply_device_setting: ({ deviceId, change }) => {
    const settings = fixtureSettings.get(deviceId);
    if (!settings) return fail("unsupported", "this device's settings cannot be changed yet");
    if (change.type === "onboardProfile") {
      if (!settings.onboardProfile || change.value >= settings.onboardProfile.count)
        fail("validation", "Choose a keyboard profile from 1 to 4.");
      settings.onboardProfile!.current = change.value;
    } else if (change.type === "dpi") {
      if (!settings.dpi?.values.includes(change.value)) fail("device", `${change.value} DPI is not a value this mouse accepts.`);
      settings.dpi!.current = change.value;
    } else {
      if (!settings.reportRate?.valuesHz.includes(change.value)) fail("device", `${change.value} Hz is not a rate this device accepts.`);
      settings.reportRate!.currentHz = change.value;
    }
    return { settings: structuredClone(settings), tookEffect: true };
  },
  get_update_status: () => ({
    currentVersion: "0.1.0",
    channel: settings.updateChannel ?? "stable",
    buildChannel: "stable",
    configured: false,
  }),
  check_for_update: () => fail("unsupported", "Updates are only available in the desktop app."),
  install_update: () => fail("unsupported", "Updates are only available in the desktop app."),
  send_test_notification: () => fail("unsupported", "Notifications are only available in the desktop app."),
  get_launch_at_startup: () => launchAtStartup,
  set_launch_at_startup: ({ enabled }) => (launchAtStartup = enabled),
  get_settings: () => ({ ...settings }),
  update_settings: ({ settings: next }) => {
    settings = { ...next };
    return { ...settings };
  },
};

export async function previewBackend<C extends CommandName>(
  command: C,
  args: Commands[C]["args"],
): Promise<Commands[C]["result"]> {
  // Simulate IPC latency so loading states are exercised during development.
  await new Promise((resolve) => setTimeout(resolve, 120));
  return handlers[command](args);
}
