import { Gamepad2, Plus } from "lucide-react";
import { useEffect, useState } from "react";

import { cn } from "@/lib/cn";
import { useAppRules } from "@/stores/appRules";
import { toast } from "@/stores/toasts";
import type { Profile } from "@/types/profile";

import { LinkAppDialog } from "./LinkAppDialog";

const VISIBLE_CHIPS = 4;

/**
 * Linked games shown right in the profile row, with a "Link game" button that
 * is prominent while the profile has none.
 */
export function LinkedGames({ profile }: { profile: Profile }) {
  const rules = useAppRules((s) => s.byProfile[profile.id]);
  const { load, add } = useAppRules();
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!profile.isDefault) void load(profile.id).catch(() => {});
  }, [profile.id, profile.isDefault, load]);

  if (profile.isDefault) {
    // Its own description usually says this already.
    return profile.description ? null : (
      <p className="text-xs text-subtle">Active when no linked game or app is in front.</p>
    );
  }

  const empty = rules !== undefined && rules.length === 0;
  const shown = rules?.slice(0, VISIBLE_CHIPS) ?? [];
  const more = (rules?.length ?? 0) - shown.length;

  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {shown.map((rule) => (
        <span
          key={rule.id}
          title={rule.value}
          className="inline-flex h-6 max-w-48 items-center gap-1.5 rounded-lg bg-raised px-2 text-xs font-medium shadow-module"
        >
          <Gamepad2 className="size-3.5 shrink-0 text-accent" />
          <span className="truncate">{rule.displayName}</span>
        </span>
      ))}
      {more > 0 && <span className="text-xs text-subtle">+{more} more</span>}
      <button
        type="button"
        onClick={() => setOpen(true)}
        className={cn(
          "inline-flex h-6 items-center gap-1 rounded-lg px-2 text-xs font-semibold transition-colors",
          empty
            ? "border border-dashed border-accent-line text-accent hover:bg-accent-soft"
            : "text-muted hover:bg-hover hover:text-fg",
        )}
      >
        <Plus className="size-3.5" />
        {empty ? "Link a game so this profile switches on automatically" : "Link game"}
      </button>
      <LinkAppDialog
        open={open}
        onClose={() => setOpen(false)}
        profileName={profile.name}
        onLink={async (kind, value, displayName) => {
          await add(profile.id, kind, value, displayName);
          toast.success(`"${profile.name}" will activate while ${displayName} is in front.`);
        }}
      />
    </div>
  );
}
