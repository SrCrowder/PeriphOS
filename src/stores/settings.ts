import { create } from "zustand";

import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { DEFAULT_SETTINGS, type AppSettings } from "@/types/settings";

import { toast } from "./toasts";

interface SettingsState {
  settings: AppSettings;
  loaded: boolean;
  load: () => Promise<void>;
  /** Optimistically apply a change, persist it, and roll back on failure. */
  update: (patch: Partial<AppSettings>) => Promise<void>;
}

export const useSettings = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,

  load: async () => {
    try {
      set({ settings: await api.getSettings(), loaded: true });
    } catch (error) {
      set({ loaded: true });
      toast.error(`Could not load settings: ${errorMessage(error)}`);
    }
  },

  update: async (patch) => {
    const previous = get().settings;
    const next = { ...previous, ...patch };
    set({ settings: next });
    try {
      set({ settings: await api.updateSettings(next) });
    } catch (error) {
      set({ settings: previous });
      toast.error(`Could not save settings: ${errorMessage(error)}`);
    }
  },
}));
