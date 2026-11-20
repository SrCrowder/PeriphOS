import { Gamepad2, Trash2 } from "lucide-react";
import { useEffect } from "react";

import { Badge, Button, Spinner } from "@/components/ui";
import { errorMessage } from "@/lib/errors";
import { useAppRules } from "@/stores/appRules";
import { toast } from "@/stores/toasts";
import type { Profile } from "@/types/profile";

/** Full list of the games and apps that activate a profile, with unlinking. */
export function ProfileApps({ profile }: { profile: Profile }) {
  const rules = useAppRules((s) => s.byProfile[profile.id]);
  const { load, remove } = useAppRules();

  useEffect(() => {
    void load(profile.id).catch((e) => toast.error(errorMessage(e)));
  }, [profile.id, load]);

  if (profile.isDefault) {
    return (
      <p className="text-[13px] text-muted">
        The default profile is used whenever no linked game or app is in front, so nothing is linked
        to it.
      </p>
    );
  }
  if (!rules) return <Spinner className="size-3.5 text-muted" />;
  if (rules.length === 0) return <p className="text-[13px] text-muted">No games or apps linked yet.</p>;

  return (
    <div className="flex flex-col gap-1.5">
      {rules.map((rule) => (
        <div key={rule.id} className="flex items-center gap-3 rounded-xl bg-raised px-3 py-2 text-[13px]">
          <Gamepad2 className="size-4 shrink-0 text-subtle" />
          <span className="min-w-0 flex-1 truncate" title={rule.value}>
            <span className="font-semibold">{rule.displayName}</span>
            <span className="text-muted">
              {" "}
              · {rule.kind === "package" ? "Store / Game Pass package" : rule.value.split(/[\\/]/).pop()}
            </span>
          </span>
          {rule.kind === "package" && <Badge tone="info">Package</Badge>}
          <Button
            size="sm"
            variant="ghost"
            aria-label={`Unlink ${rule.displayName}`}
            className="hover:text-danger"
            onClick={async () => {
              try {
                await remove(rule);
                toast.success(`Unlinked ${rule.displayName}.`);
              } catch (e) {
                toast.error(errorMessage(e));
              }
            }}
          >
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      ))}
    </div>
  );
}
