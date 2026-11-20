import { create } from "zustand";

import { api } from "@/lib/ipc";
import type { AppRule, AppRuleKind } from "@/types/apps";

interface AppRulesState {
  /** Linked games and apps, by profile ID. Missing until loaded. */
  byProfile: Record<string, AppRule[]>;
  load: (profileId: string) => Promise<void>;
  /** Throws an IpcError when the link cannot be created. */
  add: (profileId: string, kind: AppRuleKind, value: string, displayName: string) => Promise<AppRule>;
  remove: (rule: AppRule) => Promise<void>;
}

export const useAppRules = create<AppRulesState>((set, get) => ({
  byProfile: {},

  load: async (profileId) => {
    const rules = await api.appRules(profileId);
    set((s) => ({ byProfile: { ...s.byProfile, [profileId]: rules } }));
  },

  add: async (profileId, kind, value, displayName) => {
    const rule = await api.addAppRule(profileId, kind, value, displayName);
    await get().load(profileId);
    return rule;
  },

  remove: async (rule) => {
    await api.deleteAppRule(rule.id);
    await get().load(rule.profileId);
  },
}));
