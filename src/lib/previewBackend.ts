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
import type { DeviceInfo } from "@/types/device";
import type { Profile, ProfileDraft } from "@/types/profile";
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

let fixtureDevices: DeviceInfo[] = [];
if (import.meta.env.DEV && new URLSearchParams(location.search).has("fixtures")) {
  fixtureDevices = (await import("./previewFixtures")).PREVIEW_FIXTURE_DEVICES;
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
    return null;
  },
  set_default_profile: ({ id }) => {
    find(id);
    profiles = profiles.map((p) =>
      p.id === id ? { ...p, isDefault: true, updatedAt: Date.now() } : { ...p, isDefault: false },
    );
    return find(id);
  },
  get_update_status: () => ({
    currentVersion: "0.1.0",
    channel: settings.updateChannel ?? "stable",
    buildChannel: "stable",
    configured: false,
  }),
  check_for_update: () => fail("unsupported", "Updates are only available in the desktop app."),
  install_update: () => fail("unsupported", "Updates are only available in the desktop app."),
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
