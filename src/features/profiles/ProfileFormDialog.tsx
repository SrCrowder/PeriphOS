import { useEffect, useState, type FormEvent } from "react";

import { Button, Dialog, TextAreaField, TextField } from "@/components/ui";
import { toIpcError } from "@/lib/errors";
import { useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";
import {
  PROFILE_DESCRIPTION_MAX,
  PROFILE_NAME_MAX,
  type Profile,
} from "@/types/profile";

interface ProfileFormDialogProps {
  open: boolean;
  onClose: () => void;
  /** Profile to edit; omit to create a new one. */
  profile?: Profile | null;
}

export function ProfileFormDialog({ open, onClose, profile }: ProfileFormDialogProps) {
  const create = useProfiles((s) => s.create);
  const update = useProfiles((s) => s.update);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const editing = Boolean(profile);

  useEffect(() => {
    if (open) {
      setName(profile?.name ?? "");
      setDescription(profile?.description ?? "");
      setError(null);
    }
  }, [open, profile]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setSaving(true);
    setError(null);
    const draft = { name, description: description || null };
    try {
      const saved = profile ? await update(profile.id, draft) : await create(draft);
      toast.success(editing ? `Saved "${saved.name}".` : `Created "${saved.name}".`);
      onClose();
    } catch (e) {
      setError(toIpcError(e).message);
    } finally {
      setSaving(false);
    }
  };

  const nameLength = [...name.trim()].length;
  const descriptionLength = [...description.trim()].length;
  const tooLong = nameLength > PROFILE_NAME_MAX || descriptionLength > PROFILE_DESCRIPTION_MAX;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={editing ? "Edit profile" : "New profile"}
      description={
        editing
          ? undefined
          : "A profile groups the settings that should be active together, for example for one game."
      }
    >
      <form onSubmit={submit} className="flex flex-col gap-4">
        <TextField
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Competitive FPS"
          data-autofocus
          required
          error={error}
          counter={{ value: nameLength, max: PROFILE_NAME_MAX }}
        />
        <TextAreaField
          label="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Optional"
          counter={{ value: descriptionLength, max: PROFILE_DESCRIPTION_MAX }}
        />
        <div className="mt-2 flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose} disabled={saving}>
            Cancel
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={saving}
            disabled={nameLength === 0 || tooLong}
          >
            {editing ? "Save changes" : "Create profile"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
