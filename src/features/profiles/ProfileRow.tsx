import { ChevronDown, Pencil, Play, Star, Trash2 } from "lucide-react";
import { useState } from "react";

import { Led } from "@/components/layout/Sidebar";
import { Badge, Button } from "@/components/ui";
import { cn } from "@/lib/cn";
import { formatDateTime, formatRelative } from "@/lib/format";
import type { Profile } from "@/types/profile";

import { ProfileDeviceSettings } from "./ProfileDeviceSettings";

interface ProfileRowProps {
  profile: Profile;
  active: boolean;
  activating: boolean;
  onActivate: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onSetDefault: () => void;
}

export function ProfileRow({
  profile,
  active,
  activating,
  onActivate,
  onEdit,
  onDelete,
  onSetDefault,
}: ProfileRowProps) {
  const [open, setOpen] = useState(false);
  return (
    <li className="group">
      <div className="flex items-center gap-4 px-6 py-4 transition-colors hover:bg-raised/60">
        <button
          type="button"
          onClick={() => setOpen((o) => !o)}
          aria-expanded={open}
          aria-label={`${open ? "Hide" : "Show"} ${profile.name} device settings`}
          className="grid size-10 shrink-0 place-items-center rounded-xl bg-raised font-display text-base text-accent shadow-module"
        >
          {profile.name.slice(0, 1).toUpperCase()}
        </button>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <p className="truncate font-semibold">{profile.name}</p>
            {active && (
              <Badge tone="success" icon={<Led on className="size-1.5" />}>
                Active
              </Badge>
            )}
            {profile.isDefault && <Badge tone="accent">Default</Badge>}
          </div>
          <p className="truncate text-[13px] text-muted">
            {profile.description ?? <span className="text-subtle">No description</span>}
          </p>
        </div>
        <time
          dateTime={new Date(profile.updatedAt).toISOString()}
          title={`Last edited ${formatDateTime(profile.updatedAt)}`}
          className="hidden text-xs text-subtle tabular-nums lg:block"
        >
          Edited {formatRelative(profile.updatedAt)}
        </time>
        <div className="flex items-center gap-1">
          {!active && (
            <Button
              size="sm"
              variant="secondary"
              icon={<Play className="size-3.5" />}
              loading={activating}
              onClick={onActivate}
            >
              Activate
            </Button>
          )}
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
          <Button
            size="sm"
            variant="ghost"
            aria-label="Device settings"
            aria-expanded={open}
            onClick={() => setOpen((o) => !o)}
          >
            <ChevronDown className={cn("size-4 transition-transform", open && "rotate-180")} />
          </Button>
        </div>
      </div>
      {open && (
        <div className="px-6 pb-4 pl-20">
          <p className="mb-2 text-xs font-semibold text-muted">Device settings in this profile</p>
          <ProfileDeviceSettings profileId={profile.id} />
        </div>
      )}
    </li>
  );
}
