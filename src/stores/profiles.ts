import { create } from "zustand";

import { api, isTauri } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import {
  PROFILE_APPLIED_EVENT,
  type ActivationReport,
  type Profile,
  type ProfileDraft,
  type SwitchMode,
} from "@/types/profile";

import { toast } from "./toasts";

type Status = "idle" | "loading" | "ready" | "error";

interface ProfilesState {
  profiles: Profile[];
  /** The profile whose settings are in effect. */
  activeId: string | null;
  mode: SwitchMode;
  /** Linked app that selected the active profile, in automatic mode. */
  matchedApp: string | null;
  automaticSupported: boolean;
  /**
   * Increases every time a profile's settings are applied to devices, so views
   * showing live device values know to read them again.
   */
  appliedRevision: number;
  status: Status;
  error: string | null;
  load: () => Promise<void>;
  /** The mutating actions throw an IpcError on failure so forms can show it. */
  create: (draft: ProfileDraft) => Promise<Profile>;
  update: (id: string, draft: ProfileDraft) => Promise<Profile>;
  remove: (id: string) => Promise<void>;
  setDefault: (id: string) => Promise<void>;
  activate: (id: string) => Promise<ActivationReport>;
  resumeAutomatic: () => Promise<void>;
  /** Follow profile changes made by the backend. Returns an unsubscribe function. */
  watch: () => Promise<() => void>;
}

/** Toasts describing what happened when a profile's settings were applied. */
export function reportActivation(report: ActivationReport) {
  const failed = report.results.filter((r) => !r.applied);
  const applied = report.results.filter((r) => r.applied);
  const name = report.profile.name;
  for (const result of failed) {
    toast.error(`Could not apply "${name}" to ${result.deviceName}: ${result.message ?? "unknown error"}`);
  }
  if (failed.length > 0) return;
  if (report.reason === "activated") {
    toast.success(
      applied.length > 0
        ? `"${name}" is active. Settings applied to ${applied.map((r) => r.deviceName).join(", ")}.`
        : `"${name}" is active.`,
    );
  } else if (report.reason === "deviceConnected" && applied.length > 0) {
    toast.info(`Applied "${name}" settings to ${applied.map((r) => r.deviceName).join(", ")}.`);
  }
  // Automatic switches are shown in the sidebar; a toast on every app change would be noise.
}

export const useProfiles = create<ProfilesState>((set, get) => ({
  profiles: [],
  activeId: null,
  mode: "automatic",
  matchedApp: null,
  automaticSupported: false,
  appliedRevision: 0,
  status: "idle",
  error: null,

  load: async () => {
    set({ status: get().status === "ready" ? "ready" : "loading", error: null });
    try {
      const [profiles, active] = await Promise.all([api.listProfiles(), api.activeProfile()]);
      set({
        profiles,
        activeId: active.profile.id,
        mode: active.mode,
        matchedApp: active.matchedApp,
        automaticSupported: active.automaticSupported,
        status: "ready",
      });
    } catch (error) {
      set({ status: "error", error: errorMessage(error) });
    }
  },

  create: async (draft) => {
    const profile = await api.createProfile(draft);
    await get().load();
    return profile;
  },

  update: async (id, draft) => {
    const profile = await api.updateProfile(id, draft);
    await get().load();
    return profile;
  },

  remove: async (id) => {
    await api.deleteProfile(id);
    await get().load();
  },

  setDefault: async (id) => {
    await api.setDefaultProfile(id);
    await get().load();
  },

  activate: async (id) => {
    const report = await api.activateProfile(id);
    set((s) => ({
      activeId: report.profile.id,
      mode: report.mode,
      matchedApp: report.matchedApp,
      appliedRevision: s.appliedRevision + 1,
    }));
    return report;
  },

  resumeAutomatic: async () => {
    const active = await api.resumeAutomaticSwitching();
    set((s) => ({
      activeId: active.profile.id,
      mode: active.mode,
      matchedApp: active.matchedApp,
      appliedRevision: s.appliedRevision + 1,
    }));
  },

  watch: async () => {
    if (!isTauri) return () => {};
    const { listen } = await import("@tauri-apps/api/event");
    return listen<ActivationReport>(PROFILE_APPLIED_EVENT, (event) => {
      const report = event.payload;
      set((s) => ({
        activeId: report.profile.id,
        mode: report.mode,
        matchedApp: report.matchedApp,
        appliedRevision: s.appliedRevision + 1,
      }));
      // Activations started from this window already report their own result.
      if (report.reason !== "activated") reportActivation(report);
    });
  },
}));

export const selectActiveProfile = (s: ProfilesState) =>
  s.profiles.find((p) => p.id === s.activeId) ?? s.profiles.find((p) => p.isDefault) ?? null;

export const selectDefaultProfile = (s: ProfilesState) => s.profiles.find((p) => p.isDefault) ?? null;
