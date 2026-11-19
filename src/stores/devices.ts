import { create } from "zustand";

import { api, isTauri } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { DEVICES_CHANGED_EVENT, type DeviceOverview } from "@/types/device";

interface DevicesState {
  overview: DeviceOverview | null;
  scanning: boolean;
  error: string | null;
  scan: () => Promise<void>;
  /** Follow connection changes pushed by the backend. Returns an unsubscribe function. */
  watch: () => Promise<() => void>;
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

  watch: async () => {
    if (!isTauri) return () => {};
    const { listen } = await import("@tauri-apps/api/event");
    return listen<DeviceOverview>(DEVICES_CHANGED_EVENT, (event) =>
      set({ overview: event.payload, error: null }),
    );
  },
}));

export function selectDevice(id: string | null) {
  return (s: DevicesState) => s.overview?.devices.find((d) => d.id === id) ?? null;
}
