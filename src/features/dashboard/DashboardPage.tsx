import { ArrowUpRight } from "lucide-react";

import { Led } from "@/components/layout/Led";
import { Badge, Button, Panel } from "@/components/ui";
import { formatRelative, PLATFORM_LABEL } from "@/lib/format";
import { useAppInfo } from "@/stores/appInfo";
import { useDevices } from "@/stores/devices";
import { useNavigation } from "@/stores/navigation";
import { selectActiveProfile, useProfiles } from "@/stores/profiles";

import { BatteryIndicator } from "@/features/devices/BatteryIndicator";
import { isPhysical } from "@/features/devices/labels";

import { ProfileDial } from "./ProfileDial";

const RECENT_LIMIT = 5;
const DEVICE_SLOTS = 4;

export function DashboardPage() {
  const navigate = useNavigation((s) => s.navigate);
  const openDevice = useNavigation((s) => s.openDevice);
  const profiles = useProfiles((s) => s.profiles);
  const activeProfile = useProfiles(selectActiveProfile);
  const mode = useProfiles((s) => s.mode);
  const matchedApp = useProfiles((s) => s.matchedApp);
  const overview = useDevices((s) => s.overview);
  const info = useAppInfo((s) => s.info);

  const recent = [...profiles].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, RECENT_LIMIT);
  const devices = (overview?.devices ?? []).filter(isPhysical);

  return (
    <div className="grid gap-4 lg:grid-cols-[1.6fr_1fr]">
      <h1 className="sr-only">Dashboard</h1>

      <section className="flex min-h-64 flex-wrap items-end justify-between gap-6 rounded-module bg-accent p-7 text-accent-fg lg:col-span-2">
        <div className="min-w-0 flex-1">
          <p className="text-[13px] font-semibold opacity-70">Active profile</p>
          <p className="mt-2 line-clamp-2 font-display text-[clamp(48px,7vw,92px)] leading-[0.92] tracking-[-0.04em] break-words">
            {activeProfile?.name ?? " "}
          </p>
          <p className="mt-4 max-w-md text-sm font-medium opacity-80">
            {mode === "manual"
              ? "Chosen manually. Automatic switching is paused."
              : matchedApp
                ? `Active while ${matchedApp} is in front.`
                : (activeProfile?.description ?? "Used when no linked game or app is in front.")}
          </p>
        </div>
        <button
          type="button"
          onClick={() => navigate("profiles")}
          aria-label="Manage profiles"
          className="rounded-full transition-transform hover:scale-[1.03] focus-visible:outline-accent-fg"
        >
          <ProfileDial count={profiles.length} />
        </button>
      </section>

      <Panel
        title="Devices"
        actions={
          <Button size="sm" variant="ghost" onClick={() => navigate("devices")}>
            Open <ArrowUpRight className="size-3.5" />
          </Button>
        }
      >
        <ul className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
          {Array.from({ length: Math.max(DEVICE_SLOTS, devices.length) }, (_, i) => {
            const device = devices[i];
            return device ? (
              <li key={device.id}>
                <button
                  type="button"
                  onClick={() => openDevice(device.id)}
                  className="flex h-16 w-full flex-col justify-center rounded-2xl bg-raised px-3 text-left shadow-module transition-colors hover:bg-hover"
                >
                  <span className="truncate text-[13px] font-semibold">{device.name}</span>
                  <span className="flex items-center justify-between gap-2 text-xs text-muted">
                    <span className="truncate">{device.manufacturer}</span>
                    {device.battery && <BatteryIndicator battery={device.battery} />}
                  </span>
                </button>
              </li>
            ) : (
              <li
                key={`empty-${i}`}
                className="grid h-16 place-items-center rounded-2xl border-2 border-dashed border-line-strong text-xs text-subtle"
              >
                Empty slot
              </li>
            );
          })}
        </ul>
        <p className="mt-4 text-[13px] text-muted">
          {devices.length > 0
            ? `${devices.length} connected. Select one to see what PeriphOS can do with it.`
            : "No mice, keyboards or controllers detected."}
        </p>
        <dl className="mt-5 flex flex-wrap gap-x-8 gap-y-2 border-t border-line pt-4 text-[13px]">
          <div className="flex items-center gap-2">
            <dt className="text-muted">Platform</dt>
            <dd className="font-semibold">{info ? PLATFORM_LABEL[info.platform] : "…"}</dd>
          </div>
          <div className="flex items-center gap-2">
            <dt className="text-muted">Device backends</dt>
            <dd className="flex items-center gap-1.5 font-semibold tabular-nums">
              <Led on={(overview?.drivers.length ?? 0) > 0} className="text-success" />
              {overview ? overview.drivers.length : "…"}
            </dd>
          </div>
          <div className="flex items-center gap-2">
            <dt className="text-muted">Version</dt>
            <dd className="font-semibold tabular-nums">{info?.version ?? "…"}</dd>
          </div>
        </dl>
      </Panel>

      <Panel
        title="Recently edited"
        flush
        actions={
          <Button size="sm" variant="ghost" onClick={() => navigate("profiles")}>
            All <ArrowUpRight className="size-3.5" />
          </Button>
        }
      >
        <ul className="px-6 pb-3">
          {recent.map((p) => (
            <li key={p.id} className="border-b border-line last:border-b-0">
              <button
                type="button"
                onClick={() => navigate("profiles")}
                className="flex w-full items-center gap-2 py-3 text-left hover:text-accent"
              >
                <span className="truncate text-sm font-semibold">{p.name}</span>
                {p.isDefault && <Badge tone="accent">Default</Badge>}
                <span className="ml-auto shrink-0 text-xs text-muted tabular-nums">
                  {formatRelative(p.updatedAt)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </Panel>
    </div>
  );
}
