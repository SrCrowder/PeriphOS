import { Copy, Monitor, Moon, Sun } from "lucide-react";

import { PageHeader } from "@/components/layout/PageHeader";
import { Button, Panel, PlannedTag, SegmentedControl, SettingRow, Switch } from "@/components/ui";
import { PLATFORM_LABEL } from "@/lib/format";
import { useAppInfo } from "@/stores/appInfo";
import { useSettings } from "@/stores/settings";
import { toast } from "@/stores/toasts";
import type { Theme } from "@/types/settings";

import { AccentPicker } from "./AccentPicker";
import { LaunchAtStartupRow } from "./LaunchAtStartupRow";
import { UpdatesPanel } from "./UpdatesPanel";

const THEME_OPTIONS = [
  { value: "dark", label: "Dark", icon: <Moon className="size-3.5" /> },
  { value: "light", label: "Light", icon: <Sun className="size-3.5" /> },
  { value: "system", label: "System", icon: <Monitor className="size-3.5" /> },
] as const satisfies readonly { value: Theme; label: string; icon: React.ReactNode }[];

export function SettingsPage() {
  const { settings, update } = useSettings();
  const info = useAppInfo((s) => s.info);

  const copyPath = async () => {
    if (!info) return;
    try {
      await navigator.clipboard.writeText(info.databasePath);
      toast.success("Copied the data location.");
    } catch {
      toast.error("Could not access the clipboard.");
    }
  };

  return (
    <>
      <PageHeader title="Settings" description="Preferences are saved on this computer." />

      <div className="flex max-w-3xl flex-col gap-4">
        <Panel title="Appearance">
          <SettingRow
            title="Theme"
            description="System follows your operating system setting."
            control={
              <SegmentedControl
                label="Theme"
                value={settings.theme}
                options={THEME_OPTIONS}
                onChange={(theme) => void update({ theme })}
              />
            }
          />
          <SettingRow
            title="Accent color"
            control={<AccentPicker value={settings.accent} onChange={(accent) => void update({ accent })} />}
          />
          <SettingRow
            title="Reduce motion"
            description="Turns off interface animations. Also follows your system preference."
            control={
              <Switch
                label="Reduce motion"
                checked={settings.reduceMotion}
                onChange={(reduceMotion) => void update({ reduceMotion })}
              />
            }
          />
        </Panel>

        <Panel title="Behavior">
          <LaunchAtStartupRow />
          <SettingRow
            title="Keep running in the system tray"
            description="Closing the window keeps PeriphOS in the tray so profiles keep switching. Quit from the tray icon's menu."
            control={
              <Switch
                label="Keep running in the system tray"
                checked={settings.minimizeToTray}
                onChange={(minimizeToTray) => void update({ minimizeToTray })}
              />
            }
          />
        </Panel>

        <UpdatesPanel />

        <Panel title="Data">
          <SettingRow
            title="Data location"
            description={
              <span className="break-all select-text">{info?.databasePath ?? "…"}</span>
            }
            control={
              <Button size="sm" icon={<Copy className="size-3.5" />} onClick={() => void copyPath()} disabled={!info}>
                Copy
              </Button>
            }
          />
          <SettingRow
            disabled
            title={<>Backup and restore <PlannedTag milestone="Phase 5" /></>}
            description="Export all profiles and settings to a file, and restore them later."
            control={<Button size="sm" disabled>Export</Button>}
          />
        </Panel>

        <Panel title="About">
          <dl className="grid grid-cols-[auto_1fr] gap-x-8 gap-y-2 text-[13px]">
            <dt className="text-muted">Version</dt>
            <dd className="tabular-nums">{info?.version ?? "…"}</dd>
            <dt className="text-muted">Platform</dt>
            <dd>{info ? PLATFORM_LABEL[info.platform] : "…"}</dd>
            <dt className="text-muted">Database schema</dt>
            <dd className="tabular-nums">{info ? `v${info.schemaVersion}` : "…"}</dd>
            <dt className="text-muted">Source code</dt>
            <dd className="select-text">github.com/SrCrowder/PeriphOS</dd>
          </dl>
        </Panel>
      </div>
    </>
  );
}
