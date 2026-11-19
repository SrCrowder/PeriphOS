import { Gamepad2, Plus, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { Badge, Button, Spinner } from "@/components/ui";
import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { toast } from "@/stores/toasts";
import type { AppRule } from "@/types/apps";
import type { Profile } from "@/types/profile";

import { LinkAppDialog } from "./LinkAppDialog";

/** Games and apps that activate a profile. */
export function ProfileApps({ profile }: { profile: Profile }) {
  const [rules, setRules] = useState<AppRule[] | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);

  const load = useCallback(async () => {
    try {
      setRules(await api.appRules(profile.id));
    } catch (e) {
      toast.error(errorMessage(e));
      setRules([]);
    }
  }, [profile.id]);

  useEffect(() => {
    void load();
  }, [load]);

  if (profile.isDefault) {
    return (
      <p className="text-[13px] text-muted">
        The default profile is used whenever no linked game or app is in front, so nothing is linked
        to it.
      </p>
    );
  }
  if (!rules) return <Spinner className="size-3.5 text-muted" />;

  return (
    <div className="flex flex-col gap-1.5">
      {rules.length === 0 && <p className="text-[13px] text-muted">No games or apps linked yet.</p>}
      {rules.map((rule) => (
        <div key={rule.id} className="flex items-center gap-3 rounded-xl bg-raised px-3 py-2 text-[13px]">
          <Gamepad2 className="size-4 shrink-0 text-subtle" />
          <span className="min-w-0 flex-1 truncate" title={rule.value}>
            <span className="font-semibold">{rule.displayName}</span>
            <span className="text-muted"> · {rule.kind === "package" ? "Store / Game Pass package" : rule.value.split(/[\\/]/).pop()}</span>
          </span>
          {rule.kind === "package" && <Badge tone="info">Package</Badge>}
          <Button
            size="sm"
            variant="ghost"
            aria-label={`Unlink ${rule.displayName}`}
            className="hover:text-danger"
            onClick={async () => {
              try {
                await api.deleteAppRule(rule.id);
                toast.success(`Unlinked ${rule.displayName}.`);
                await load();
              } catch (e) {
                toast.error(errorMessage(e));
              }
            }}
          >
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      ))}
      <div>
        <Button size="sm" variant="ghost" icon={<Plus className="size-3.5" />} onClick={() => setDialogOpen(true)}>
          Link game or app
        </Button>
      </div>
      <LinkAppDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
        profileName={profile.name}
        onLink={async (kind, value, displayName) => {
          await api.addAppRule(profile.id, kind, value, displayName);
          toast.success(`"${profile.name}" will activate while ${displayName} is in front.`);
          await load();
        }}
      />
    </div>
  );
}
