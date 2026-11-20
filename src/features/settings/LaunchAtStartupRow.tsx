import { useEffect, useState } from "react";

import { SettingRow, Switch } from "@/components/ui";
import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { toast } from "@/stores/toasts";

/** Start PeriphOS at sign-in, straight into the tray. Stored by the OS, not in PeriphOS's settings. */
export function LaunchAtStartupRow() {
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api
      .launchAtStartup()
      .then(setEnabled)
      .catch(() => setEnabled(null));
  }, []);

  const change = async (next: boolean) => {
    setBusy(true);
    try {
      setEnabled(await api.setLaunchAtStartup(next));
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <SettingRow
      disabled={enabled === null}
      title="Start when you sign in"
      description={
        enabled === null
          ? "Not available on this system."
          : "PeriphOS starts in the system tray, so profiles switch and settings apply without opening it."
      }
      control={
        <Switch
          label="Start when you sign in"
          checked={enabled ?? false}
          disabled={enabled === null || busy}
          onChange={(next) => void change(next)}
        />
      }
    />
  );
}
