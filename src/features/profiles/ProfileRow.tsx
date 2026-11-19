import { Pencil, Star, Trash2 } from "lucide-react";

import { Badge, Button } from "@/components/ui";
import { formatDateTime, formatRelative } from "@/lib/format";
import type { Profile } from "@/types/profile";

interface ProfileRowProps {
  profile: Profile;
  onEdit: () => void;
  onDelete: () => void;
  onSetDefault: () => void;
}

export function ProfileRow({ profile, onEdit, onDelete, onSetDefault }: ProfileRowProps) {
  return (
    <li className="group flex items-center gap-4 px-6 py-4 transition-colors hover:bg-raised/60">
      <div
        aria-hidden
        className="grid size-10 shrink-0 place-items-center rounded-xl bg-raised font-display text-base text-accent shadow-module"
      >
        {profile.name.slice(0, 1).toUpperCase()}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="truncate font-semibold">{profile.name}</p>
          {profile.isDefault && <Badge tone="accent">Default</Badge>}
        </div>
        <p className="truncate text-[13px] text-muted">
          {profile.description ?? <span className="text-subtle">No description</span>}
        </p>
      </div>
      <time
        dateTime={new Date(profile.updatedAt).toISOString()}
        title={`Last edited ${formatDateTime(profile.updatedAt)}`}
        className="hidden text-xs text-subtle tabular-nums md:block"
      >
        Edited {formatRelative(profile.updatedAt)}
      </time>
      <div className="flex items-center gap-1">
        {!profile.isDefault && (
          <Button size="sm" variant="ghost" icon={<Star className="size-3.5" />} onClick={onSetDefault}>
            Make default
          </Button>
        )}
        <Button size="sm" variant="ghost" aria-label={`Edit ${profile.name}`} onClick={onEdit}>
          <Pencil className="size-3.5" />
        </Button>
        <Button
          size="sm"
          variant="ghost"
          aria-label={`Delete ${profile.name}`}
          title={profile.isDefault ? "The default profile cannot be deleted" : undefined}
          disabled={profile.isDefault}
          onClick={onDelete}
          className="hover:text-danger"
        >
          <Trash2 className="size-3.5" />
        </Button>
      </div>
    </li>
  );
}
