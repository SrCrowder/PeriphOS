import { Save } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { Button } from "@/components/ui";
import { describeSaved } from "@/features/profiles/ProfileDeviceSettings";
import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { selectActiveProfile, useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";
import type { DeviceProfileSettings } from "@/types/profile";

/**
 * Stores the device's current settings in a profile, and lists the profiles
 * that already hold settings for it.
 */
export function SaveToProfile({ deviceId, deviceName }: { deviceId: string; deviceName: string }) {
  const profiles = useProfiles((s) => s.profiles);
  const active = useProfiles(selectActiveProfile);
  const [target, setTarget] = useState<string>("");
  const [saved, setSaved] = useState<{ profileName: string; entry: DeviceProfileSettings }[]>([]);
  const [busy, setBusy] = useState(false);

  const selected = target || active?.id || "";

  const loadSaved = useCallback(async () => {
    try {
      const lists = await Promise.all(profiles.map((p) => api.profileDeviceSettings(p.id)));
      setSaved(
        lists.flatMap((list, i) =>
          list.filter((e) => e.deviceId === deviceId).map((entry) => ({ profileName: profiles[i]!.name, entry })),
        ),
      );
    } catch {
      setSaved([]);
    }
  }, [profiles, deviceId]);

  useEffect(() => {
    void loadSaved();
  }, [loadSaved]);

  const save = async () => {
    const profile = profiles.find((p) => p.id === selected);
    if (!profile) return;
    setBusy(true);
    try {
      const entry = await api.saveDeviceSettingsToProfile(profile.id, deviceId);
      toast.success(`Saved ${deviceName} at ${describeSaved(entry)} in "${profile.name}".`);
      await loadSaved();
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-3 border-t border-line pt-5">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p className="text-sm font-semibold">Save to profile</p>
          <p className="text-[13px] text-muted">
            Stores the current values. They are applied whenever that profile is active and the
            device connects.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <label htmlFor="save-profile" className="sr-only">
            Profile
          </label>
          <select
            id="save-profile"
            value={selected}
            onChange={(e) => setTarget(e.target.value)}
            className="h-10 rounded-xl border border-transparent bg-raised px-3 text-sm focus:border-accent focus:outline-none"
          >
            {profiles.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
                {p.id === active?.id ? " (active)" : ""}
              </option>
            ))}
          </select>
          <Button icon={<Save className="size-4" />} loading={busy} disabled={!selected} onClick={() => void save()}>
            Save current settings
          </Button>
        </div>
      </div>
      {saved.length > 0 && (
        <p className="text-[13px] text-muted">
          Saved in{" "}
          {saved.map(({ profileName, entry }, i) => (
            <span key={profileName}>
              {i > 0 && ", "}
              <span className="font-semibold text-fg">{profileName}</span> ({describeSaved(entry)})
            </span>
          ))}
          .
        </p>
      )}
    </div>
  );
}
