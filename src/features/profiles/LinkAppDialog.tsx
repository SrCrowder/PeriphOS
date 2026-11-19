import { FileSearch, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { Badge, Button, Dialog, Spinner } from "@/components/ui";
import { api, isTauri } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import type { AppIdentity, AppRuleKind } from "@/types/apps";

interface LinkAppDialogProps {
  open: boolean;
  onClose: () => void;
  profileName: string;
  /** Throws an IpcError when the link cannot be created. */
  onLink: (kind: AppRuleKind, value: string, displayName: string) => Promise<void>;
}

/**
 * Pick a game or app to link: from the applications that are open (which also
 * covers Microsoft Store / Game Pass games, identified by package), or by
 * choosing an .exe file.
 */
export function LinkAppDialog({ open, onClose, profileName, onLink }: LinkAppDialogProps) {
  const [apps, setApps] = useState<AppIdentity[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setApps(null);
    try {
      setApps(await api.runningApps());
    } catch (e) {
      setError(errorMessage(e));
      setApps([]);
    }
  }, []);

  useEffect(() => {
    if (open) {
      setError(null);
      void refresh();
    }
  }, [open, refresh]);

  const link = async (kind: AppRuleKind, value: string, displayName: string) => {
    setBusy(true);
    setError(null);
    try {
      await onLink(kind, value, displayName);
      onClose();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const browse = async () => {
    const { open: openFile } = await import("@tauri-apps/plugin-dialog");
    const path = await openFile({
      multiple: false,
      directory: false,
      filters: [{ name: "Programs", extensions: ["exe"] }],
    });
    if (typeof path === "string") {
      const name = path.split(/[\\/]/).pop() ?? path;
      await link("executable", path, name.replace(/\.exe$/i, ""));
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={`Link a game or app to "${profileName}"`}
      description="The profile activates while it is in front. Start the game first to pick it from the list; Microsoft Store and Game Pass games are recognised by their package."
      className="w-[min(620px,calc(100vw-32px))]"
    >
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <p className="text-[13px] font-semibold">Open now</p>
          <Button size="sm" variant="ghost" icon={<RefreshCw className="size-3.5" />} onClick={() => void refresh()}>
            Refresh
          </Button>
        </div>

        {apps === null ? (
          <p className="flex items-center gap-2 text-[13px] text-muted">
            <Spinner className="size-3.5" /> Looking for open applications…
          </p>
        ) : apps.length === 0 ? (
          <p className="text-[13px] text-muted">
            {isTauri
              ? "No other applications with a window were found."
              : "Open applications can only be listed in the desktop app on Windows."}
          </p>
        ) : (
          <ul className="flex max-h-72 flex-col gap-1.5 overflow-y-auto">
            {apps.map((app) => {
              const kind: AppRuleKind = app.packageFamily ? "package" : "executable";
              const value = app.packageFamily ?? app.exePath ?? app.exeName;
              const name = app.title.trim() || app.exeName;
              return (
                <li key={`${kind}:${value}`} className="flex items-center gap-3 rounded-xl bg-raised px-3 py-2.5">
                  <div className="min-w-0 flex-1">
                    <p className="flex items-center gap-2 truncate text-sm font-semibold">
                      <span className="truncate">{name}</span>
                      {app.packageFamily && <Badge tone="info">Store / Game Pass</Badge>}
                    </p>
                    <p className="truncate text-xs text-muted" title={app.exePath ?? undefined}>
                      {app.exeName}
                    </p>
                  </div>
                  <Button size="sm" disabled={busy} onClick={() => void link(kind, value, name)}>
                    Link
                  </Button>
                </li>
              );
            })}
          </ul>
        )}

        <div className="flex items-center justify-between gap-4 border-t border-line pt-4">
          <p className="text-[13px] text-muted">Not running? Choose its program file instead.</p>
          <Button
            icon={<FileSearch className="size-4" />}
            disabled={busy || !isTauri}
            onClick={() => void browse().catch((e) => setError(errorMessage(e)))}
          >
            Choose .exe…
          </Button>
        </div>
        {error && <p className="text-[13px] text-danger">{error}</p>}
      </div>
    </Dialog>
  );
}
