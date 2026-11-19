import { FileSearch, RefreshCw, Search } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Badge, Button, Dialog, SegmentedControl, Spinner } from "@/components/ui";
import { api, isTauri } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import type { AppIdentity, AppRuleKind, InstalledPackage } from "@/types/apps";

interface LinkAppDialogProps {
  open: boolean;
  onClose: () => void;
  profileName: string;
  /** Throws an IpcError when the link cannot be created. */
  onLink: (kind: AppRuleKind, value: string, displayName: string) => Promise<void>;
}

type Source = "running" | "installed";

const SOURCES = [
  { value: "running", label: "Open now" },
  { value: "installed", label: "Installed games" },
] as const satisfies readonly { value: Source; label: string }[];

interface Choice {
  key: string;
  kind: AppRuleKind;
  value: string;
  name: string;
  detail: string;
  badge: string | null;
}

function fromRunning(app: AppIdentity): Choice {
  const kind: AppRuleKind = app.packageFamily ? "package" : "executable";
  const value = app.packageFamily ?? app.exePath ?? app.exeName;
  return {
    key: `${kind}:${value}`,
    kind,
    value,
    name: app.title.trim() || app.exeName,
    detail: app.exeName,
    badge: app.packageFamily ? "Store / Game Pass" : null,
  };
}

function fromInstalled(pkg: InstalledPackage): Choice {
  return {
    key: `package:${pkg.packageFamily}`,
    kind: "package",
    value: pkg.packageFamily,
    name: pkg.displayName,
    detail: pkg.packageFamily,
    badge: pkg.isGame ? "Game" : null,
  };
}

/**
 * Pick a game or app to link: from the applications that are open, from the
 * Microsoft Store / Game Pass packages installed on this PC (no need to browse
 * the protected WindowsApps folder), or by choosing an .exe file.
 */
export function LinkAppDialog({ open, onClose, profileName, onLink }: LinkAppDialogProps) {
  const [source, setSource] = useState<Source>("running");
  const [running, setRunning] = useState<Choice[] | null>(null);
  const [installed, setInstalled] = useState<Choice[] | null>(null);
  const [showAllPackages, setShowAllPackages] = useState(false);
  const [allInstalled, setAllInstalled] = useState<InstalledPackage[]>([]);
  const [query, setQuery] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const loadRunning = useCallback(async () => {
    setRunning(null);
    try {
      setRunning((await api.runningApps()).map(fromRunning));
    } catch (e) {
      setError(errorMessage(e));
      setRunning([]);
    }
  }, []);

  const loadInstalled = useCallback(async () => {
    setInstalled(null);
    try {
      const packages = await api.installedPackages();
      setAllInstalled(packages);
      setInstalled(packages.map(fromInstalled));
    } catch (e) {
      setError(errorMessage(e));
      setInstalled([]);
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    setError(null);
    setQuery("");
    void loadRunning();
  }, [open, loadRunning]);

  useEffect(() => {
    if (open && source === "installed" && installed === null) void loadInstalled();
  }, [open, source, installed, loadInstalled]);

  const visible = useMemo(() => {
    const list =
      source === "running"
        ? running
        : installed?.filter((_, i) => showAllPackages || allInstalled[i]?.isGame) ?? null;
    if (!list) return null;
    const q = query.trim().toLowerCase();
    return q ? list.filter((c) => c.name.toLowerCase().includes(q) || c.detail.toLowerCase().includes(q)) : list;
  }, [source, running, installed, allInstalled, showAllPackages, query]);

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

  const emptyText =
    !isTauri
      ? "This list is only available in the desktop app on Windows."
      : source === "running"
        ? "No other applications with a window were found."
        : showAllPackages
          ? "No Microsoft Store apps were found."
          : "No Microsoft Store or Game Pass games were found. Show all apps to see every installed package.";

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={`Link a game or app to "${profileName}"`}
      description="The profile activates while it is in front. Microsoft Store and Game Pass games are recognised by their package, so their protected folder does not matter."
      className="w-[min(640px,calc(100vw-32px))]"
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <SegmentedControl label="Source" value={source} options={SOURCES} onChange={setSource} />
          <Button
            size="sm"
            variant="ghost"
            icon={<RefreshCw className="size-3.5" />}
            onClick={() => void (source === "running" ? loadRunning() : loadInstalled())}
          >
            Refresh
          </Button>
        </div>

        <div className="flex items-center gap-3">
          <label className="relative flex-1">
            <span className="sr-only">Search</span>
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-subtle" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search"
              className="h-10 w-full rounded-xl border border-transparent bg-raised pr-3 pl-9 text-sm focus:border-accent focus:outline-none"
            />
          </label>
          {source === "installed" && (
            <label className="flex shrink-0 items-center gap-2 text-[13px] text-muted">
              <input
                type="checkbox"
                checked={showAllPackages}
                onChange={(e) => setShowAllPackages(e.target.checked)}
                className="accent-[var(--accent)]"
              />
              Show all apps
            </label>
          )}
        </div>

        {visible === null ? (
          <p className="flex items-center gap-2 text-[13px] text-muted">
            <Spinner className="size-3.5" />
            {source === "running" ? "Looking for open applications…" : "Reading installed apps…"}
          </p>
        ) : visible.length === 0 ? (
          <p className="text-[13px] text-muted">{query ? "Nothing matches your search." : emptyText}</p>
        ) : (
          <ul className="flex max-h-72 flex-col gap-1.5 overflow-y-auto">
            {visible.map((choice) => (
              <li key={choice.key} className="flex items-center gap-3 rounded-xl bg-raised px-3 py-2.5">
                <div className="min-w-0 flex-1">
                  <p className="flex items-center gap-2 truncate text-sm font-semibold">
                    <span className="truncate">{choice.name}</span>
                    {choice.badge && <Badge tone="info">{choice.badge}</Badge>}
                  </p>
                  <p className="truncate text-xs text-muted" title={choice.detail}>
                    {choice.detail}
                  </p>
                </div>
                <Button size="sm" disabled={busy} onClick={() => void link(choice.kind, choice.value, choice.name)}>
                  Link
                </Button>
              </li>
            ))}
          </ul>
        )}

        <div className="flex items-center justify-between gap-4 border-t border-line pt-4">
          <p className="text-[13px] text-muted">Not in the lists? Choose its program file instead.</p>
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
