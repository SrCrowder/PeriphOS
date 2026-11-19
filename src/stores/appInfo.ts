import { create } from "zustand";

import { api } from "@/lib/ipc";
import type { AppInfo } from "@/types/app";

interface AppInfoState {
  info: AppInfo | null;
  load: () => Promise<void>;
}

export const useAppInfo = create<AppInfoState>((set) => ({
  info: null,
  load: async () => {
    try {
      set({ info: await api.appInfo() });
    } catch {
      // Non-critical: views that use it render a fallback.
    }
  },
}));
