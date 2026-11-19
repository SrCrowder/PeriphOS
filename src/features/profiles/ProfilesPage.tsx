import { AppWindow, CircleAlert, Plus } from "lucide-react";
import { useState } from "react";

import { PageHeader } from "@/components/layout/PageHeader";
import { Button, ConfirmDialog, EmptyState, Panel, PlannedTag, Spinner } from "@/components/ui";
import { errorMessage } from "@/lib/errors";
import { useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";
import type { Profile } from "@/types/profile";

import { ProfileFormDialog } from "./ProfileFormDialog";
import { ProfileRow } from "./ProfileRow";

export function ProfilesPage() {
  const { profiles, status, error, load, remove, setDefault } = useProfiles();
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [deleting, setDeleting] = useState<Profile | null>(null);

  const openCreate = () => {
    setEditing(null);
    setFormOpen(true);
  };

  const makeDefault = async (profile: Profile) => {
    try {
      await setDefault(profile.id);
      toast.success(`"${profile.name}" is now the default profile.`);
    } catch (e) {
      toast.error(errorMessage(e));
    }
  };

  return (
    <>
      <PageHeader
        title="Profiles"
        description="Profiles hold the settings that should be active together. The default profile applies whenever no other profile matches."
        actions={
          <Button variant="primary" icon={<Plus className="size-4" />} onClick={openCreate}>
            New profile
          </Button>
        }
      />

      <div className="flex flex-col gap-4">
        <Panel flush>
          {status === "loading" || status === "idle" ? (
            <div className="grid place-items-center py-16 text-muted">
              <Spinner />
            </div>
          ) : status === "error" ? (
            <EmptyState
              icon={<CircleAlert />}
              title="Profiles could not be loaded"
              actions={<Button onClick={() => void load()}>Try again</Button>}
            >
              {error}
            </EmptyState>
          ) : (
            <ul className="divide-y divide-line py-2">
              {profiles.map((profile) => (
                <ProfileRow
                  key={profile.id}
                  profile={profile}
                  onEdit={() => {
                    setEditing(profile);
                    setFormOpen(true);
                  }}
                  onDelete={() => setDeleting(profile)}
                  onSetDefault={() => void makeDefault(profile)}
                />
              ))}
            </ul>
          )}
        </Panel>

        <Panel
          title={
            <span className="flex items-center gap-2">
              Linked games and apps <PlannedTag milestone="Phase 3" />
            </span>
          }
        >
          <div className="flex gap-4">
            <AppWindow className="mt-0.5 size-5 shrink-0 text-subtle" />
            <p className="max-w-2xl text-[13px] text-muted">
              You will be able to link a profile to games and applications so it activates when they
              come to the foreground. Besides picking an executable, PeriphOS will let you choose
              from running processes and installed Microsoft Store / Game Pass packages, whose
              executables Windows does not let you browse to.
            </p>
          </div>
        </Panel>
      </div>

      <ProfileFormDialog open={formOpen} onClose={() => setFormOpen(false)} profile={editing} />

      <ConfirmDialog
        open={deleting !== null}
        onClose={() => setDeleting(null)}
        title="Delete profile?"
        description={
          <>
            <span className="font-medium text-fg">{deleting?.name}</span> will be deleted. This
            cannot be undone.
          </>
        }
        confirmLabel="Delete profile"
        destructive
        onConfirm={async () => {
          if (!deleting) return;
          await remove(deleting.id);
          toast.success(`Deleted "${deleting.name}".`);
        }}
      />
    </>
  );
}
