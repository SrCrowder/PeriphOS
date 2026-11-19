import { create } from "zustand";

import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import type { Profile, ProfileDraft } from "@/types/profile";

type Status = "idle" | "loading" | "ready" | "error";

interface ProfilesState {
  profiles: Profile[];
  status: Status;
  error: string | null;
  load: () => Promise<void>;
  /** The mutating actions throw an IpcError on failure so forms can show it. */
  create: (draft: ProfileDraft) => Promise<Profile>;
  update: (id: string, draft: ProfileDraft) => Promise<Profile>;
  remove: (id: string) => Promise<void>;
  setDefault: (id: string) => Promise<void>;
}

export const useProfiles = create<ProfilesState>((set, get) => ({
  profiles: [],
  status: "idle",
  error: null,

  load: async () => {
    set({ status: get().status === "ready" ? "ready" : "loading", error: null });
    try {
      set({ profiles: await api.listProfiles(), status: "ready" });
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
}));

export const selectDefaultProfile = (s: ProfilesState) =>
  s.profiles.find((p) => p.isDefault) ?? null;
