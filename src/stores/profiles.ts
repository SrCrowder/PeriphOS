import { create } from "zustand";

import { api, isTauri } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import {
  PROFILE_APPLIED_EVENT,
  type ActivationReport,
  type Profile,
  type ProfileDraft,
} from "@/types/profile";

import { toast } from "./toasts";

type Status = "idle" | "loading" | "ready" | "error";

interface ProfilesState {
  profiles: Profile[];
  /** The profile whose settings are in effect. */
  activeId: string | null;
  status: Status;
  error: string | null;
  load: () => Promise<void>;
  /** The mutating actions throw an IpcError on failure so forms can show it. */
  create: (draft: ProfileDraft) => Promise<Profile>;
  update: (id: string, draft: ProfileDraft) => Promise<Profile>;
  remove: (id: string) => Promise<void>;
  setDefault: (id: string) => Promise<void>;
  activate: (id: string) => Promise<ActivationReport>;
  /** Follow profile changes made by the backend. Returns an unsubscribe function. */
  watch: () => Promise<() => void>;
}

/** Toast describing what happened when a profile's settings were applied. */
export function reportActivation(report: ActivationReport) {
  const failed = report.results.filter((r) => !r.applied);
  const applied = report.results.filter((r) => r.applied);
  const name = report.profile.name;
  for (const result of failed) {
    toast.error(`Could not apply "${name}" to ${result.deviceName}: ${result.message ?? "unknown error"}`);
  }
  if (report.reason === "activated") {
    if (failed.length === 0) {
      toast.success(
        applied.length > 0
          ? `"${name}" is active. Settings applied to ${applied.map((r) => r.deviceName).join(", ")}.`
          : `"${name}" is active.`,
      );
    }
  } else if (applied.length > 0) {
    toast.info(`Applied "${name}" settings to ${applied.map((r) => r.deviceName).join(", ")}.`);
  }
}

export const useProfiles = create<ProfilesState>((set, get) => ({
  profiles: [],
  activeId: null,
  status: "idle",
  error: null,

  load: async () => {
    set({ status: get().status === "ready" ? "ready" : "loading", error: null });
    try {
      const [profiles, active] = await Promise.all([api.listProfiles(), api.activeProfile()]);
      set({ profiles, activeId: active.id, status: "ready" });
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
    set({ activeId: report.profile.id });
    return report;
  },

  watch: async () => {
    if (!isTauri) return () => {};
    const { listen } = await import("@tauri-apps/api/event");
    return listen<ActivationReport>(PROFILE_APPLIED_EVENT, (event) => {
      set({ activeId: event.payload.profile.id });
      // Activations started from this window already report their own result.
      if (event.payload.reason === "deviceConnected") reportActivation(event.payload);
    });
  },
}));

export const selectActiveProfile = (s: ProfilesState) =>
  s.profiles.find((p) => p.id === s.activeId) ?? s.profiles.find((p) => p.isDefault) ?? null;

export const selectDefaultProfile = (s: ProfilesState) => s.profiles.find((p) => p.isDefault) ?? null;
