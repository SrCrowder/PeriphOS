import { create } from "zustand";

export type PageId = "dashboard" | "devices" | "profiles" | "macros" | "rgb" | "settings";

interface NavigationState {
  page: PageId;
  navigate: (page: PageId) => void;
}

export const useNavigation = create<NavigationState>((set) => ({
  page: "dashboard",
  navigate: (page) => set({ page }),
}));
