import { BellRing } from "lucide-react";
import { useState } from "react";

import { Button, Panel, SettingRow, Switch } from "@/components/ui";
import { api } from "@/lib/ipc";
import { errorMessage } from "@/lib/errors";
import { useSettings } from "@/stores/settings";
import { toast } from "@/stores/toasts";

const WARNING_OPTIONS = [10, 15, 20, 25, 30, 40];
const CRITICAL_OPTIONS = [5, 10, 15, 20];

function PercentSelect({
  id,
  label,
  value,
  options,
  disabled,
  onChange,
}: {
  id: string;
  label: string;
  value: number;
  options: number[];
  disabled: boolean;
  onChange: (value: number) => void;
}) {
  return (
    <>
      <label htmlFor={id} className="sr-only">
        {label}
      </label>
      <select
        id={id}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(Number(e.target.value))}
        className="h-9 rounded-xl border border-transparent bg-raised px-3 text-sm tabular-nums focus:border-accent focus:outline-none disabled:opacity-50"
      >
        {options.map((o) => (
          <option key={o} value={o}>
            {o}%
          </option>
        ))}
      </select>
    </>
  );
}

/** Low-battery notifications for wireless devices. */
export function BatteryAlertsPanel() {
  const { settings, update } = useSettings();
  const [testing, setTesting] = useState(false);
  const off = !settings.batteryAlerts;

  const test = async () => {
    setTesting(true);
    try {
      await api.sendTestNotification();
      toast.info("Test notification sent. If it did not appear, check Windows notification settings.");
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setTesting(false);
    }
  };

  return (
    <Panel title="Battery alerts">
      <SettingRow
        title="Notify when a battery runs low"
        description="Wireless devices whose battery PeriphOS can read. Each alert is sent once until the device charges."
        control={
          <Switch
            label="Notify when a battery runs low"
            checked={settings.batteryAlerts}
            onChange={(batteryAlerts) => void update({ batteryAlerts })}
          />
        }
      />
      <SettingRow
        disabled={off}
        title="First alert"
        description="Charge it soon."
        control={
          <PercentSelect
            id="battery-warning"
            label="First alert"
            value={settings.batteryWarningPercent}
            options={WARNING_OPTIONS}
            disabled={off}
            onChange={(batteryWarningPercent) =>
              void update({
                batteryWarningPercent,
                batteryCriticalPercent: Math.min(settings.batteryCriticalPercent, batteryWarningPercent - 5),
              })
            }
          />
        }
      />
      <SettingRow
        disabled={off}
        title="Second alert"
        description="Charge it now."
        control={
          <PercentSelect
            id="battery-critical"
            label="Second alert"
            value={settings.batteryCriticalPercent}
            options={CRITICAL_OPTIONS.filter((o) => o < settings.batteryWarningPercent)}
            disabled={off}
            onChange={(batteryCriticalPercent) => void update({ batteryCriticalPercent })}
          />
        }
      />
      <SettingRow
        disabled={off}
        title="Check notifications"
        description="Shows a sample alert, so you can confirm Windows lets PeriphOS notify you."
        control={
          <Button
            size="sm"
            icon={<BellRing className="size-3.5" />}
            loading={testing}
            disabled={off}
            onClick={() => void test()}
          >
            Send a test notification
          </Button>
        }
      />
    </Panel>
  );
}
