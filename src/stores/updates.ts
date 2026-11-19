import { create } from "zustand";

import { api, isTauri } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import {
  UPDATE_PROGRESS_EVENT,
  type UpdateCheck,
  type UpdateProgress,
  type UpdateStatus,
} from "@/types/updates";

export type UpdatePhase = "idle" | "checking" | "upToDate" | "available" | "installing" | "error";

interface UpdatesState {
  status: UpdateStatus | null;
  phase: UpdatePhase;
  result: UpdateCheck | null;
  progress: UpdateProgress | null;
  error: string | null;
  lastCheckedAt: number | null;
  loadStatus: () => Promise<void>;
  /** `silent` keeps errors out of sight (used for the automatic startup check). */
  check: (options?: { silent?: boolean }) => Promise<void>;
  install: () => Promise<void>;
}

export const useUpdates = create<UpdatesState>((set, get) => ({
  status: null,
  phase: "idle",
  result: null,
  progress: null,
  error: null,
  lastCheckedAt: null,

  loadStatus: async () => {
    try {
      set({ status: await api.updateStatus() });
    } catch (error) {
      set({ error: errorMessage(error) });
    }
  },

  check: async ({ silent = false } = {}) => {
    if (get().phase === "checking" || get().phase === "installing") return;
    set({ phase: "checking", error: null });
    try {
      const result = await api.checkForUpdate();
      set({
        result,
        phase: result.available ? "available" : "upToDate",
        lastCheckedAt: Date.now(),
      });
    } catch (error) {
      set(silent ? { phase: "idle" } : { phase: "error", error: errorMessage(error) });
    }
  },

  install: async () => {
    set({ phase: "installing", progress: null, error: null });
    let unlisten: (() => void) | undefined;
    if (isTauri) {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<UpdateProgress>(UPDATE_PROGRESS_EVENT, (event) =>
        set({ progress: event.payload }),
      );
    }
    try {
      // On success the app restarts, so this normally never returns.
      await api.installUpdate();
    } catch (error) {
      set({ phase: "error", error: errorMessage(error) });
    } finally {
      unlisten?.();
    }
  },
}));
