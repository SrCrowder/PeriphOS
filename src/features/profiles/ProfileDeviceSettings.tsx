import { Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Button, Spinner } from "@/components/ui";
import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { formatRelative } from "@/lib/format";
import { useDevices } from "@/stores/devices";
import { toast } from "@/stores/toasts";
import type { DeviceProfileSettings } from "@/types/profile";

const number = new Intl.NumberFormat("en");

export function describeSaved(s: DeviceProfileSettings): string {
  return [
    s.onboardProfile != null ? `keyboard profile ${s.onboardProfile + 1}` : null,
    s.dpi !== null ? `${number.format(s.dpi)} DPI` : null,
    s.reportRateHz !== null ? `${number.format(s.reportRateHz)} Hz` : null,
    s.buttons ? "own button assignments" : null,
  ]
    .filter(Boolean)
    .join(", ");
}

/** Device settings stored in one profile, with a way to remove each entry. */
export function ProfileDeviceSettings({ profileId }: { profileId: string }) {
  const [entries, setEntries] = useState<DeviceProfileSettings[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const devices = useDevices((s) => s.overview?.devices);
  const connected = useMemo(() => new Set(devices?.map((d) => d.id) ?? []), [devices]);

  const load = useCallback(async () => {
    try {
      setEntries(await api.profileDeviceSettings(profileId));
    } catch (e) {
      setError(errorMessage(e));
    }
  }, [profileId]);

  useEffect(() => {
    void load();
  }, [load]);

  if (error) return <p className="text-[13px] text-danger">{error}</p>;
  if (!entries) return <Spinner className="size-3.5 text-muted" />;
  if (entries.length === 0) {
    return (
      <p className="text-[13px] text-muted">
        No device settings yet. Select this profile in the sidebar, open a device and apply a change.
      </p>
    );
  }

  return (
    <ul className="flex flex-col gap-1.5">
      {entries.map((entry) => (
        <li key={entry.deviceId} className="flex items-center gap-3 rounded-xl bg-raised px-3 py-2 text-[13px]">
          <span className="min-w-0 flex-1">
            <span className="font-semibold">{entry.deviceName}</span>
            <span className="text-muted"> · {describeSaved(entry)}</span>
            {!connected.has(entry.deviceId) && <span className="text-subtle"> · not connected</span>}
          </span>
          <span className="text-xs text-subtle">Saved {formatRelative(entry.updatedAt)}</span>
          <Button
            size="sm"
            variant="ghost"
            aria-label={`Remove ${entry.deviceName} settings`}
            className="hover:text-danger"
            onClick={async () => {
              try {
                await api.deleteProfileDeviceSettings(profileId, entry.deviceId);
                toast.success(`Removed ${entry.deviceName} settings from the profile.`);
                await load();
              } catch (e) {
                toast.error(errorMessage(e));
              }
            }}
          >
            <Trash2 className="size-3.5" />
          </Button>
        </li>
      ))}
    </ul>
  );
}
