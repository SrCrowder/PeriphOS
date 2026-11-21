import { RefreshCw, TriangleAlert } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { Button, Panel, SegmentedControl, Spinner } from "@/components/ui";
import { api } from "@/lib/ipc";
import { toIpcError } from "@/lib/errors";
import { toast } from "@/stores/toasts";
import { selectActiveProfile, useProfiles } from "@/stores/profiles";
import type { DeviceInfo, DeviceSettings, SettingChange } from "@/types/device";

import { WootingGuide } from "./WootingGuide";

const number = new Intl.NumberFormat("en");

type State =
  | { status: "loading" }
  | { status: "unsupported" }
  | { status: "error"; message: string }
  | { status: "ready"; settings: DeviceSettings };

/** Index of the value closest to `target` in an ascending list. */
function nearestIndex(values: number[], target: number): number {
  let best = 0;
  values.forEach((v, i) => {
    if (Math.abs(v - target) < Math.abs(values[best]! - target)) best = i;
  });
  return best;
}

function describe(change: SettingChange): string {
  switch (change.type) {
    case "dpi":
      return `${number.format(change.value)} DPI`;
    case "reportRateHz":
      return `${number.format(change.value)} Hz`;
    case "onboardProfile":
      return `profile ${change.value + 1}`;
  }
}

const SETTING_NAME: Record<SettingChange["type"], string> = {
  dpi: "Sensitivity",
  reportRateHz: "Report rate",
  onboardProfile: "Keyboard",
};

/**
 * Live settings read from the device. Hidden for devices whose driver cannot
 * read settings. Every change is read back to confirm it took effect.
 */
export function DeviceSettingsPanel({ device }: { device: DeviceInfo }) {
  const activeProfile = useProfiles(selectActiveProfile);
  const [state, setState] = useState<State>({ status: "loading" });
  const [dpiIndex, setDpiIndex] = useState(0);
  const [busy, setBusy] = useState(false);

  /** `quiet` keeps the current values on screen while reading (no spinner). */
  const load = useCallback(async (quiet = false) => {
    if (!quiet) setState({ status: "loading" });
    try {
      const settings = await api.deviceSettings(device.id);
      setState({ status: "ready", settings });
      if (settings.dpi) setDpiIndex(nearestIndex(settings.dpi.values, settings.dpi.current));
    } catch (e) {
      const error = toIpcError(e);
      setState(error.code === "unsupported" ? { status: "unsupported" } : { status: "error", message: error.message });
    }
  }, [device.id]);

  useEffect(() => {
    void load();
  }, [load]);

  // A profile was applied (switched by hand, automatically, or on connect):
  // the device may now use other values, so read them again.
  const appliedRevision = useProfiles((s) => s.appliedRevision);
  const firstRevision = useRef(appliedRevision);
  useEffect(() => {
    if (appliedRevision !== firstRevision.current) void load(true);
  }, [appliedRevision, load]);

  const apply = async (change: SettingChange) => {
    setBusy(true);
    try {
      const result = await api.applyDeviceSetting(device.id, change);
      setState({ status: "ready", settings: result.settings });
      const dpi = result.settings.dpi;
      if (dpi) setDpiIndex(nearestIndex(dpi.values, dpi.current));
      if (result.tookEffect) {
        // Changes belong to the active profile: store the device's settings there.
        const profile = selectActiveProfile(useProfiles.getState());
        const what = `${SETTING_NAME[change.type]} set to ${describe(change)}`;
        if (profile) {
          try {
            await api.saveDeviceSettingsToProfile(profile.id, device.id);
            toast.success(`${what} and saved to "${profile.name}".`);
          } catch (e) {
            toast.error(`${what}, but it could not be saved to "${profile.name}": ${toIpcError(e).message}`);
          }
        } else {
          toast.success(`${what}.`);
        }
      } else {
        toast.error(
          change.type === "onboardProfile"
            ? `${device.name} did not switch to ${describe(change)}. Make sure Wootility is closed and its App Linking is off, then try again.`
            : `${device.name} accepted the change but still reports a different value. A profile stored in the device may be in control.`,
        );
      }
    } catch (e) {
      toast.error(toIpcError(e).message);
    } finally {
      setBusy(false);
    }
  };

  if (state.status === "unsupported") return null;

  return (
    <Panel
      title="Settings"
      description={
        <>
          Read from the device just now. Changes apply right away and are saved to the active
          profile, <span className="font-semibold text-fg">{activeProfile?.name ?? "…"}</span>.
          Pick another profile from the sidebar to edit that one.
        </>
      }
      actions={
        <Button
          size="sm"
          variant="ghost"
          icon={<RefreshCw className="size-3.5" />}
          onClick={() => void load()}
          disabled={state.status === "loading" || busy}
        >
          Read again
        </Button>
      }
    >
      {state.status === "loading" && (
        <p className="flex items-center gap-2 text-[13px] text-muted">
          <Spinner className="size-3.5" /> Reading settings from the device…
        </p>
      )}

      {state.status === "error" && (
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-[13px] text-danger">{state.message}</p>
          <Button size="sm" onClick={() => void load()}>
            Try again
          </Button>
        </div>
      )}

      {state.status === "ready" && (
        <div className="flex flex-col gap-6">
          {device.driverId === "wooting" && <WootingGuide />}

          {state.settings.conflictingSoftware.map((name) => (
            <p key={name} className="flex gap-2.5 rounded-2xl bg-warning/10 px-4 py-3 text-[13px] text-warning">
              <TriangleAlert className="mt-0.5 size-4 shrink-0" />
              <span>
                <span className="font-semibold">{name} is running.</span> It also controls this device and
                can undo the changes made here. Close it, including from the system tray, while PeriphOS
                manages the device.
              </span>
            </p>
          ))}
          {state.settings.onboardProfileActive && (
            <p className="rounded-2xl bg-raised px-4 py-3 text-[13px] text-muted">
              The device runs from a profile stored in its own memory. PeriphOS writes the
              sensitivity and report rate into that profile, so they also keep working without
              PeriphOS.
            </p>
          )}

          {state.settings.dpi && (
            <DpiControl
              dpi={state.settings.dpi.current}
              values={state.settings.dpi.values}
              index={dpiIndex}
              onIndexChange={setDpiIndex}
              busy={busy}
              onApply={(value) => void apply({ type: "dpi", value })}
            />
          )}

          {state.settings.reportRate && (
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p className="text-sm font-semibold">Report rate</p>
                <p className="text-[13px] text-muted">How often the device reports to the computer.</p>
              </div>
              <SegmentedControl
                label="Report rate"
                value={String(state.settings.reportRate.currentHz)}
                options={state.settings.reportRate.valuesHz.map((hz) => ({
                  value: String(hz),
                  label: `${number.format(hz)} Hz`,
                }))}
                onChange={(hz) => {
                  if (!busy) void apply({ type: "reportRateHz", value: Number(hz) });
                }}
              />
            </div>
          )}

          {state.settings.onboardProfile && (
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p className="text-sm font-semibold">Keyboard profile</p>
                <p className="text-[13px] text-muted">
                  Profile {state.settings.onboardProfile.current + 1} of {state.settings.onboardProfile.count} is
                  active on the keyboard. A profile you have not set up in Wootility keeps its default
                  settings.
                </p>
              </div>
              <SegmentedControl
                label="Keyboard profile"
                value={String(state.settings.onboardProfile.current)}
                options={Array.from({ length: state.settings.onboardProfile.count }, (_, i) => ({
                  value: String(i),
                  label: `Profile ${i + 1}`,
                }))}
                onChange={(i) => {
                  if (!busy) void apply({ type: "onboardProfile", value: Number(i) });
                }}
              />
            </div>
          )}

          {!state.settings.dpi && !state.settings.reportRate && !state.settings.onboardProfile && (
            <p className="text-[13px] text-muted">This device has no settings PeriphOS can change yet.</p>
          )}
        </div>
      )}
    </Panel>
  );
}

function DpiControl({
  dpi,
  values,
  index,
  onIndexChange,
  busy,
  onApply,
}: {
  dpi: number;
  values: number[];
  index: number;
  onIndexChange: (index: number) => void;
  busy: boolean;
  onApply: (value: number) => void;
}) {
  const selected = values[index] ?? dpi;
  const min = values[0]!;
  const max = values[values.length - 1]!;
  const [typed, setTyped] = useState<string | null>(null);
  const [message, setMessage] = useState<{ error: boolean; text: string } | null>(null);

  // A typo must never jump to an extreme value: out-of-range input is rejected,
  // and in-range input is snapped to the nearest accepted value with a notice.
  // Returns the accepted value, or null when the input was rejected.
  const commitTyped = (): number | null => {
    if (typed === null) return selected;
    const parsed = Number(typed.trim());
    setTyped(null);
    if (!Number.isInteger(parsed) || parsed < min || parsed > max) {
      setMessage({ error: true, text: `Enter a whole number from ${number.format(min)} to ${number.format(max)}.` });
      return null;
    }
    const nearest = nearestIndex(values, parsed);
    onIndexChange(nearest);
    setMessage(
      values[nearest] === parsed
        ? null
        : {
            error: false,
            text: `This mouse does not accept ${number.format(parsed)}; ${number.format(values[nearest]!)} is the closest value.`,
          },
    );
    return values[nearest]!;
  };

  const applyTyped = () => {
    const value = commitTyped();
    if (value !== null && value !== dpi) onApply(value);
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="text-sm font-semibold">Sensitivity</p>
          <p className="text-[13px] text-muted">
            Currently {number.format(dpi)} DPI. Accepts {number.format(values[0]!)} to{" "}
            {number.format(values[values.length - 1]!)}.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <label className="sr-only" htmlFor="dpi-input">
            DPI value
          </label>
          <input
            id="dpi-input"
            inputMode="numeric"
            aria-invalid={message?.error || undefined}
            aria-describedby={message ? "dpi-message" : undefined}
            value={typed ?? String(selected)}
            onChange={(e) => setTyped(e.target.value)}
            onBlur={() => void commitTyped()}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitTyped();
            }}
            className="h-10 w-24 rounded-xl border border-transparent bg-raised px-3 text-right font-display text-lg tabular-nums focus:border-accent focus:outline-none aria-invalid:border-danger"
          />
          <span className="text-[13px] font-semibold text-muted">DPI</span>
          <Button
            variant="primary"
            loading={busy}
            disabled={typed === null && (selected === dpi || message?.error === true)}
            // Keep focus in the field so a typed value is read here, not lost to blur.
            onMouseDown={(e) => e.preventDefault()}
            onClick={applyTyped}
          >
            Apply
          </Button>
        </div>
      </div>
      {message && (
        <p id="dpi-message" className={message.error ? "text-[13px] text-danger" : "text-[13px] text-muted"}>
          {message.text}
        </p>
      )}
      <input
        type="range"
        aria-label="Sensitivity"
        min={0}
        max={values.length - 1}
        value={index}
        onChange={(e) => {
          setMessage(null);
          onIndexChange(Number(e.target.value));
        }}
        className="w-full accent-[var(--accent)]"
      />
    </div>
  );
}
