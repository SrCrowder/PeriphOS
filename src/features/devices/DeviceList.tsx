import { ChevronRight } from "lucide-react";

import { Badge } from "@/components/ui";
import { formatUsbId } from "@/lib/format";
import { useNavigation } from "@/stores/navigation";
import type { DeviceInfo } from "@/types/device";

import { BatteryIndicator } from "./BatteryIndicator";
import { DeviceIllustration } from "./DeviceIllustration";
import { CONNECTION_LABEL, KIND_LABEL } from "./labels";

export function DeviceList({ devices }: { devices: DeviceInfo[] }) {
  const openDevice = useNavigation((s) => s.openDevice);
  return (
    <ul className="divide-y divide-line py-2">
      {devices.map((device) => (
        <li key={device.id}>
          <button
            type="button"
            onClick={() => openDevice(device.id)}
            className="flex w-full items-center gap-4 px-6 py-3.5 text-left transition-colors hover:bg-raised/60"
          >
            <div className="grid size-12 shrink-0 place-items-center rounded-xl bg-raised shadow-module">
              <DeviceIllustration kind={device.kind} className="size-9" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="truncate font-semibold">
                {device.manufacturer} {device.name}
              </p>
              <p className="truncate text-[13px] text-muted">
                {device.connection === "virtual"
                  ? `Virtual ${KIND_LABEL[device.kind].toLowerCase()}`
                  : device.connectedVia
                    ? `${KIND_LABEL[device.kind]}, through ${device.connectedVia}`
                    : `${KIND_LABEL[device.kind]}, ${CONNECTION_LABEL[device.connection]}`}
                {device.productId !== 0 && (
                  <span className="text-subtle tabular-nums">
                    {" "}
                    · {formatUsbId(device.vendorId)}:{formatUsbId(device.productId)}
                  </span>
                )}
              </p>
            </div>
            {device.battery && <BatteryIndicator battery={device.battery} />}
            {device.driverId !== "hid-generic" ? (
              <Badge tone="success">Identified</Badge>
            ) : device.knownModel ? (
              <Badge tone="success">Known model</Badge>
            ) : (
              <Badge>Generic</Badge>
            )}
            <ChevronRight className="size-4 shrink-0 text-subtle" />
          </button>
        </li>
      ))}
    </ul>
  );
}
