import { ArrowLeft, Unplug } from "lucide-react";

import { Led } from "@/components/layout/Sidebar";
import { Badge, Button, EmptyState, Panel } from "@/components/ui";
import { formatUsbId } from "@/lib/format";
import { selectDevice, useDevices } from "@/stores/devices";
import { useNavigation } from "@/stores/navigation";

import { BatteryIndicator, batteryText } from "./BatteryIndicator";
import { DeviceIllustration } from "./DeviceIllustration";
import { DeviceSettingsPanel } from "./DeviceSettingsPanel";
import { describeCapability, describeDevice } from "./labels";
import { SUPPORT_LEVELS } from "./supportLevels";

export function DeviceDetailPage() {
  const deviceId = useNavigation((s) => s.deviceId);
  const navigate = useNavigation((s) => s.navigate);
  const device = useDevices(selectDevice(deviceId));

  const back = (
    <Button variant="ghost" size="sm" icon={<ArrowLeft className="size-4" />} onClick={() => navigate("devices")}>
      All devices
    </Button>
  );

  if (!device) {
    return (
      <div className="mt-4 flex flex-col gap-4">
        <div>{back}</div>
        <Panel>
          <EmptyState icon={<Unplug />} title="This device is not connected">
            It was disconnected or is no longer detected. Plug it back in and it will appear in
            the device list.
          </EmptyState>
        </Panel>
      </div>
    );
  }

  return (
    <div className="mt-4 flex flex-col gap-4">
      <div>{back}</div>

      <section className="flex flex-wrap items-center gap-8 rounded-module bg-surface p-7 shadow-module">
        <div className="grid size-40 shrink-0 place-items-center rounded-3xl bg-raised shadow-module">
          <DeviceIllustration kind={device.kind} className="w-32" />
        </div>
        <div className="min-w-0 flex-1">
          <p className="flex items-center gap-2 text-[13px] font-semibold text-success">
            <Led on /> Connected
          </p>
          <h1 className="mt-2 font-display text-[40px] leading-none break-words">{device.name}</h1>
          <p className="mt-2 text-sm text-muted">{describeDevice(device)}</p>
          {device.note && <p className="mt-4 max-w-xl text-[13px] text-muted">{device.note}</p>}
        </div>
        {device.battery && (
          <div className="flex shrink-0 flex-col items-start gap-2 rounded-2xl bg-raised px-5 py-4 shadow-module">
            <p className="text-xs font-semibold text-muted">Battery</p>
            <p className="font-display text-[34px] leading-none tabular-nums">
              {device.battery.percent !== null
                ? `${device.battery.estimated ? "~" : ""}${device.battery.percent}%`
                : batteryText(device.battery)}
            </p>
            <BatteryIndicator battery={device.battery} showText={false} />
            <p className="max-w-44 text-xs text-muted">
              {device.battery.charging === "charging"
                ? "Charging."
                : device.battery.charging === "full"
                  ? "Fully charged."
                  : device.battery.charging === "error"
                    ? "The device reports a battery error."
                    : "On battery."}
              {device.battery.estimated && " Estimated from the battery voltage."}
            </p>
          </div>
        )}
      </section>

      <DeviceSettingsPanel key={device.id} device={device} />

      <div className="grid gap-4 lg:grid-cols-[1.6fr_1fr]">
        <Panel
          title="Features"
          description="What this model can do, and what PeriphOS can control today."
          flush
        >
          {device.capabilities.length === 0 ? (
            <p className="px-6 pb-6 text-[13px] text-muted">
              {device.knownModel
                ? "No configurable features are listed for this model yet."
                : "PeriphOS can identify this device, but has no information about its features yet. You can help by opening a device support request on GitHub."}
            </p>
          ) : (
            <ul className="divide-y divide-line border-t border-line">
              {device.capabilities.map((capability) => {
                const { label, detail } = describeCapability(capability);
                // Reading the battery is not a setting, so "on device" would mislead.
                const level =
                  capability.kind.type === "battery" && capability.support === "native"
                    ? { label: "Supported", tone: "success" as const }
                    : SUPPORT_LEVELS[capability.support];
                return (
                  <li key={capability.kind.type} className="flex items-start gap-4 px-6 py-3.5">
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-semibold">{label}</p>
                      {(detail || capability.note) && (
                        <p className="text-[13px] text-muted">
                          {[detail, capability.note].filter(Boolean).join(". ")}
                        </p>
                      )}
                    </div>
                    <Badge tone={level.tone} className="mt-0.5" >
                      {level.label}
                    </Badge>
                  </li>
                );
              })}
            </ul>
          )}
        </Panel>

        <Panel title="Identification">
          <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-2.5 text-[13px]">
            {device.productId !== 0 && (
              <>
                <dt className="text-muted">{device.connectedVia ? "Wireless ID" : "USB ID"}</dt>
                <dd className="font-semibold tabular-nums select-text">
                  {formatUsbId(device.vendorId)}:{formatUsbId(device.productId)}
                </dd>
              </>
            )}
            {device.connectedVia && (
              <>
                <dt className="text-muted">Connected through</dt>
                <dd className="font-semibold">{device.connectedVia}</dd>
              </>
            )}
            {device.interfaceCount > 0 && (
              <>
                <dt className="text-muted">HID interfaces</dt>
                <dd className="font-semibold tabular-nums">{device.interfaceCount}</dd>
              </>
            )}
            <dt className="text-muted">Model data</dt>
            <dd className="font-semibold">
              {device.knownModel ? "PeriphOS database" : "Read from the device"}
            </dd>
            <dt className="text-muted">Detected by</dt>
            <dd className="font-semibold">{device.driverId}</dd>
          </dl>
        </Panel>
      </div>
    </div>
  );
}
