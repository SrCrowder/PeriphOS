import { Gamepad2, Plus, X } from "lucide-react";
import { useEffect, useState } from "react";

import { cn } from "@/lib/cn";
import { errorMessage } from "@/lib/errors";
import { useAppRules } from "@/stores/appRules";
import { toast } from "@/stores/toasts";
import type { Profile } from "@/types/profile";

import { LinkAppDialog } from "./LinkAppDialog";

/**
 * The games and apps that activate a profile, each removable, with a "Link
 * game" button that is prominent while the profile has none.
 */
export function LinkedGames({ profile }: { profile: Profile }) {
  const rules = useAppRules((s) => s.byProfile[profile.id]);
  const { load, add, remove } = useAppRules();
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!profile.isDefault) void load(profile.id).catch((e) => toast.error(errorMessage(e)));
  }, [profile.id, profile.isDefault, load]);

  if (profile.isDefault) {
    return (
      <p className="text-[13px] text-muted">
        Used whenever no linked game or app is in front, so nothing is linked to it.
      </p>
    );
  }

  const empty = rules !== undefined && rules.length === 0;

  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {rules?.map((rule) => (
        <span
          key={rule.id}
          title={rule.kind === "package" ? `Store / Game Pass package ${rule.value}` : rule.value}
          className="inline-flex h-7 max-w-56 items-center gap-1.5 rounded-lg bg-raised pr-1 pl-2 text-xs font-medium shadow-module"
        >
          <Gamepad2 className="size-3.5 shrink-0 text-accent" />
          <span className="truncate">{rule.displayName}</span>
          <button
            type="button"
            aria-label={`Unlink ${rule.displayName}`}
            onClick={async () => {
              try {
                await remove(rule);
                toast.success(`Unlinked ${rule.displayName}.`);
              } catch (e) {
                toast.error(errorMessage(e));
              }
            }}
            className="grid size-5 place-items-center rounded-md text-subtle hover:bg-hover hover:text-danger"
          >
            <X className="size-3" />
          </button>
        </span>
      ))}
      <button
        type="button"
        onClick={() => setOpen(true)}
        className={cn(
          "inline-flex h-7 items-center gap-1 rounded-lg px-2 text-xs font-semibold transition-colors",
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
