import { Pencil, Play, Star, Trash2 } from "lucide-react";

import { Led } from "@/components/layout/Led";
import { Badge, Button } from "@/components/ui";
import { cn } from "@/lib/cn";
import { formatDateTime, formatRelative } from "@/lib/format";
import type { Profile } from "@/types/profile";

import { LinkedGames } from "./LinkedGames";
import { ProfileDeviceSettings } from "./ProfileDeviceSettings";

interface ProfileCardProps {
  profile: Profile;
  active: boolean;
  activating: boolean;
  onActivate: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onSetDefault: () => void;
}

/** Everything about one profile, always visible: games that activate it and device settings. */
export function ProfileCard({
  profile,
  active,
  activating,
  onActivate,
  onEdit,
  onDelete,
  onSetDefault,
}: ProfileCardProps) {
  return (
    <section
      aria-label={profile.name}
      className={cn(
        "rounded-module bg-surface p-5 shadow-module",
        active && "ring-1 ring-accent-line",
      )}
    >
      <header className="flex flex-wrap items-start gap-4">
        <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-raised font-display text-lg text-accent shadow-module">
          {profile.name.slice(0, 1).toUpperCase()}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="truncate font-display text-lg leading-tight">{profile.name}</h2>
            {active && (
              <Badge tone="success" icon={<Led on className="size-1.5" />}>
                Active
              </Badge>
            )}
            {profile.isDefault && <Badge tone="accent">Default</Badge>}
          </div>
          <p className="mt-0.5 truncate text-[13px] text-muted">
            {profile.description ?? (
              <time dateTime={new Date(profile.updatedAt).toISOString()} title={formatDateTime(profile.updatedAt)}>
                Edited {formatRelative(profile.updatedAt)}
              </time>
            )}
          </p>
        </div>
        <div className="flex items-center gap-1">
          {!active && (
            <Button size="sm" icon={<Play className="size-3.5" />} loading={activating} onClick={onActivate}>
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
        </div>
      </header>

      <div className="mt-4 grid gap-5 border-t border-line pt-4 lg:grid-cols-2">
        <div>
          <h3 className="mb-2 text-xs font-semibold text-muted">Linked games and apps</h3>
          <LinkedGames profile={profile} />
        </div>
        <div>
          <h3 className="mb-2 text-xs font-semibold text-muted">Device settings</h3>
          <ProfileDeviceSettings profileId={profile.id} />
        </div>
      </div>
    </section>
  );
}
