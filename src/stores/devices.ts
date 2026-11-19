import { create } from "zustand";

import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import type { DeviceOverview } from "@/types/device";

interface DevicesState {
  overview: DeviceOverview | null;
  scanning: boolean;
  error: string | null;
  scan: () => Promise<void>;
}

export const useDevices = create<DevicesState>((set) => ({
  overview: null,
  scanning: false,
  error: null,
  scan: async () => {
    set({ scanning: true, error: null });
    try {
      set({ overview: await api.deviceOverview() });
    } catch (error) {
      set({ error: errorMessage(error) });
    } finally {
      set({ scanning: false });
    }
  },
}));
