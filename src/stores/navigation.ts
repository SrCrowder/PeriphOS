import { create } from "zustand";

export type PageId = "dashboard" | "devices" | "device" | "profiles" | "macros" | "rgb" | "settings";

interface NavigationState {
  page: PageId;
  /** Device shown on the "device" page. */
  deviceId: string | null;
  navigate: (page: PageId) => void;
  openDevice: (id: string) => void;
}

export const useNavigation = create<NavigationState>((set) => ({
  page: "dashboard",
  deviceId: null,
  navigate: (page) => set({ page }),
  openDevice: (id) => set({ page: "device", deviceId: id }),
}));
