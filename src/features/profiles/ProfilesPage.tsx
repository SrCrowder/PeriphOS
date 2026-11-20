import { CircleAlert, Plus } from "lucide-react";
import { useState } from "react";

import { PageHeader } from "@/components/layout/PageHeader";
import { Button, ConfirmDialog, EmptyState, Panel, Spinner } from "@/components/ui";
import { errorMessage } from "@/lib/errors";
import { reportActivation, useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";
import type { Profile } from "@/types/profile";

import { ProfileFormDialog } from "./ProfileFormDialog";
import { ProfileCard } from "./ProfileCard";
import { SwitchingPanel } from "./SwitchingPanel";

export function ProfilesPage() {
  const { profiles, activeId, status, error, load, remove, setDefault, activate } = useProfiles();
  const [activating, setActivating] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Profile | null>(null);
  const [deleting, setDeleting] = useState<Profile | null>(null);

  const openCreate = () => {
    setEditing(null);
    setFormOpen(true);
  };

  const activateProfile = async (profile: Profile) => {
    setActivating(profile.id);
    try {
      reportActivation(await activate(profile.id));
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setActivating(null);
    }
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
        description="A profile holds device settings that apply together. Activating a profile applies its settings to connected devices; the default profile is active when PeriphOS starts."
        actions={
          <Button variant="primary" icon={<Plus className="size-4" />} onClick={openCreate}>
            New profile
          </Button>
        }
      />

      <div className="flex flex-col gap-4">
        <SwitchingPanel />

        {status === "loading" || status === "idle" ? (
          <div className="grid place-items-center py-16 text-muted">
            <Spinner />
          </div>
        ) : status === "error" ? (
          <Panel>
            <EmptyState
              icon={<CircleAlert />}
              title="Profiles could not be loaded"
              actions={<Button onClick={() => void load()}>Try again</Button>}
            >
              {error}
            </EmptyState>
          </Panel>
        ) : (
          profiles.map((profile) => (
            <ProfileCard
              key={profile.id}
              profile={profile}
              active={profile.id === activeId}
              activating={activating === profile.id}
              onActivate={() => void activateProfile(profile)}
              onEdit={() => {
                setEditing(profile);
                setFormOpen(true);
              }}
              onDelete={() => setDeleting(profile)}
              onSetDefault={() => void makeDefault(profile)}
            />
          ))
        )}


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
