import { Hand, Radar } from "lucide-react";
import { useState } from "react";

import { Button, Panel } from "@/components/ui";
import { errorMessage } from "@/lib/errors";
import { selectActiveProfile, useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";

/** Shows how the active profile is being chosen and lets the user switch modes. */
export function SwitchingPanel() {
  const { mode, matchedApp, automaticSupported, resumeAutomatic } = useProfiles();
  const active = useProfiles(selectActiveProfile);
  const [busy, setBusy] = useState(false);

  const resume = async () => {
    setBusy(true);
    try {
      await resumeAutomatic();
      toast.success("Automatic switching is on.");
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const manual = mode === "manual";
  return (
    <Panel>
      <div className="flex flex-wrap items-center gap-5">
        <div className="grid size-11 shrink-0 place-items-center rounded-2xl bg-raised text-accent shadow-module">
          {manual ? <Hand className="size-5" /> : <Radar className="size-5" />}
        </div>
        <div className="min-w-0 flex-1">
          <p className="font-semibold">
            {manual ? "Manual: you chose this profile" : "Automatic switching"}
          </p>
          <p className="text-[13px] text-muted">
            {manual ? (
              <>
                <span className="font-semibold text-fg">{active?.name}</span> stays active until you
                switch back to automatic.
              </>
            ) : !automaticSupported ? (
              <>Following the app in front is available on Windows. On this system the default profile is used.</>
            ) : matchedApp ? (
              <>
                <span className="font-semibold text-fg">{matchedApp}</span> is in front, so{" "}
                <span className="font-semibold text-fg">{active?.name}</span> is active.
              </>
            ) : (
              <>
                No linked game or app is in front, so{" "}
                <span className="font-semibold text-fg">{active?.name}</span> is active. Link games
                to a profile below and it activates when they are in front.
              </>
            )}
          </p>
        </div>
        {manual && (
          <Button icon={<Radar className="size-4" />} loading={busy} onClick={() => void resume()}>
            Switch automatically
          </Button>
        )}
      </div>
    </Panel>
  );
}
