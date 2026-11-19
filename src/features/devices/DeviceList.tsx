import { Keyboard, Mouse, Headphones, Gamepad2, Cpu } from "lucide-react";

import { Badge } from "@/components/ui";
import { formatUsbId } from "@/lib/format";
import type { Capability, DeviceInfo, DeviceKind } from "@/types/device";

import { SUPPORT_LEVELS } from "./supportLevels";

const KIND_ICON: Record<DeviceKind, typeof Mouse> = {
  mouse: Mouse,
  keyboard: Keyboard,
  headset: Headphones,
  controller: Gamepad2,
  other: Cpu,
};

const CONNECTION_LABEL = {
  usb: "USB",
  bluetooth: "Bluetooth",
  receiver: "Wireless receiver",
  unknown: "Unknown connection",
} as const;

const CAPABILITY_LABEL: Record<Capability["kind"]["type"], string> = {
  buttonRemap: "Button remapping",
  keyRemap: "Key remapping",
  macros: "Macros",
  dpi: "DPI",
  pollingRate: "Polling rate",
  lighting: "Lighting",
  onboardProfiles: "Onboard profiles",
  battery: "Battery",
};

/** Renders devices reported by drivers. Empty until Phase 2 drivers ship. */
export function DeviceList({ devices }: { devices: DeviceInfo[] }) {
  return (
    <ul className="divide-y divide-line">
      {devices.map((device) => {
        const Icon = KIND_ICON[device.kind];
        return (
          <li key={device.id} className="flex items-start gap-4 px-6 py-4">
            <div className="grid size-10 shrink-0 place-items-center rounded-xl bg-raised text-muted shadow-module">
              <Icon className="size-5" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="font-semibold">{device.name}</p>
              <p className="text-[13px] text-muted">
                {device.manufacturer}, {CONNECTION_LABEL[device.connection]}, USB ID{" "}
                <span className="tabular-nums">
                  {formatUsbId(device.vendorId)}:{formatUsbId(device.productId)}
                </span>
              </p>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {device.capabilities.map((cap) => (
                  <Badge key={cap.kind.type} tone={SUPPORT_LEVELS[cap.support].tone}>
                    {CAPABILITY_LABEL[cap.kind.type]}: {SUPPORT_LEVELS[cap.support].label}
                  </Badge>
                ))}
              </div>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
