import { Download, RefreshCw } from "lucide-react";

import { Badge, Button, Panel, SegmentedControl, SettingRow, Switch } from "@/components/ui";
import { formatRelative } from "@/lib/format";
import { useSettings } from "@/stores/settings";
import { useUpdates } from "@/stores/updates";
import type { UpdateChannel } from "@/types/settings";

const CHANNEL_OPTIONS = [
  { value: "stable", label: "Stable" },
  { value: "development", label: "Development" },
] as const satisfies readonly { value: UpdateChannel; label: string }[];

const MB = 1024 * 1024;

function formatMb(bytes: number) {
  return `${(bytes / MB).toFixed(1)} MB`;
}

function UpdateState() {
  const { status, phase, result, progress, error, lastCheckedAt, check, install } = useUpdates();

  if (status && !status.configured) {
    return (
      <p className="text-[13px] text-muted">
        This build cannot update itself because it was not signed for updates (for example, a local
        development build). Builds from the project&apos;s releases update in place.
      </p>
    );
  }

  if (phase === "available" && result?.version) {
    return (
      <div className="flex flex-wrap items-center justify-between gap-4 rounded-2xl bg-accent p-4 text-accent-fg">
        <div className="min-w-0">
          <p className="font-bold">Version {result.version} is available</p>
          {result.notes && <p className="mt-0.5 line-clamp-3 text-[13px] opacity-80">{result.notes}</p>}
        </div>
        <button
          type="button"
          onClick={() => void install()}
          className="inline-flex h-10 items-center gap-2 rounded-xl bg-accent-fg px-4 text-sm font-semibold text-accent transition-[filter] hover:brightness-125"
        >
          <Download className="size-4" />
          Install and restart
        </button>
      </div>
    );
  }

  if (phase === "installing") {
    const total = progress?.total ?? null;
    const done = progress?.downloaded ?? 0;
    const percent = total ? Math.min(100, Math.round((done / total) * 100)) : null;
    return (
      <div className="flex flex-col gap-2" aria-live="polite">
        <div className="flex justify-between text-[13px]">
          <span className="font-semibold">
            {percent === 100 ? "Installing… PeriphOS will restart" : "Downloading update…"}
          </span>
          <span className="text-muted tabular-nums">
            {total ? `${formatMb(done)} of ${formatMb(total)}` : done ? formatMb(done) : ""}
          </span>
        </div>
        <div
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={percent ?? undefined}
          className="h-2 overflow-hidden rounded-full bg-raised"
        >
          <div
            className="h-full rounded-full bg-accent transition-[width] duration-200"
            style={{ width: `${percent ?? 15}%` }}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-wrap items-center justify-between gap-4">
      <p className={phase === "error" ? "text-[13px] text-danger" : "text-[13px] text-muted"}>
        {phase === "error"
          ? error
          : phase === "upToDate"
            ? `PeriphOS is up to date. Checked ${lastCheckedAt ? formatRelative(lastCheckedAt) : "just now"}.`
            : phase === "checking"
              ? "Checking for updates…"
              : "Not checked yet."}
      </p>
      <Button
        size="sm"
        icon={<RefreshCw className="size-3.5" />}
        loading={phase === "checking"}
        onClick={() => void check()}
      >
        {phase === "error" ? "Try again" : "Check now"}
      </Button>
    </div>
  );
}

export function UpdatesPanel() {
  const { settings, update } = useSettings();
  const { status, loadStatus } = useUpdates();
  const channel = settings.updateChannel ?? status?.buildChannel ?? "stable";

  const changeChannel = async (updateChannel: UpdateChannel) => {
    await update({ updateChannel });
    await loadStatus();
    useUpdates.setState({ phase: "idle", result: null, error: null });
  };

  return (
    <Panel
      title={
        <span className="flex items-center gap-2">
          Updates
          {status && <Badge>Version {status.currentVersion}</Badge>}
        </span>
      }
    >
      <SettingRow
        title="Update channel"
        description={
          channel === "development"
            ? "Gets a new build for every change. Meant for testing; it can be unstable."
            : "Gets tested releases only."
        }
        control={
          <SegmentedControl
            label="Update channel"
            value={channel}
            options={CHANNEL_OPTIONS}
            onChange={(c) => void changeChannel(c)}
          />
        }
      />
      <SettingRow
        title="Check when PeriphOS starts"
        description="Updates are never installed without your confirmation."
        control={
          <Switch
            label="Check when PeriphOS starts"
            checked={settings.autoCheckUpdates}
            onChange={(autoCheckUpdates) => void update({ autoCheckUpdates })}
          />
        }
      />
      <div className="border-t border-line pt-4">
        <UpdateState />
      </div>
    </Panel>
  );
}
