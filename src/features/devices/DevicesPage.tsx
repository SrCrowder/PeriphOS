import { CircleAlert, RefreshCw, Unplug } from "lucide-react";
import { useEffect } from "react";

import { PageHeader } from "@/components/layout/PageHeader";
import { Button, EmptyState, Panel, Spinner } from "@/components/ui";
import { PLATFORM_LABEL } from "@/lib/format";
import { useDevices } from "@/stores/devices";

import { DeviceList } from "./DeviceList";
import { SupportLevelLegend } from "./SupportLevelLegend";

export function DevicesPage() {
  const { overview, scanning, error, scan } = useDevices();

  useEffect(() => {
    void scan();
  }, [scan]);

  const noBackends = overview !== null && overview.drivers.length === 0;

  return (
    <>
      <PageHeader
        title="Devices"
        description="Peripherals detected by PeriphOS and what can be configured on each one."
        actions={
          <Button icon={<RefreshCw className="size-4" />} loading={scanning} onClick={() => void scan()}>
            Scan again
          </Button>
        }
      />

      <div className="flex flex-col gap-4">
        <Panel flush>
          {error ? (
            <EmptyState icon={<CircleAlert />} title="Device scan failed">
              {error}
            </EmptyState>
          ) : !overview ? (
            <div className="grid place-items-center py-16 text-muted">
              <Spinner />
            </div>
          ) : overview.devices.length > 0 ? (
            <DeviceList devices={overview.devices} />
          ) : (
            <EmptyState icon={<Unplug />} title="No devices detected">
              {noBackends ? (
                <>
                  This build does not include any device backends yet, so PeriphOS cannot see your
                  hardware. Detection of USB HID mice and keyboards on {PLATFORM_LABEL[overview.platform]}{" "}
                  is the next milestone (Phase 2).
                </>
              ) : (
                <>Connect a supported mouse or keyboard, then scan again.</>
              )}
            </EmptyState>
          )}
        </Panel>

        {overview && overview.driverErrors.length > 0 && (
          <Panel title="Backend errors" description="These backends failed during the last scan.">
            <ul className="flex flex-col gap-2 text-[13px]">
              {overview.driverErrors.map((e) => (
                <li key={e.driverId}>
                  <span className="font-medium">{e.driverId}</span>
                  <span className="text-danger">: {e.message}</span>
                </li>
              ))}
            </ul>
          </Panel>
        )}

        <div className="grid gap-4 lg:grid-cols-[1.4fr_1fr]">
          <Panel
            title="How device features are labeled"
            description="Detecting a device does not mean every setting can be changed. Each feature will show one of these labels."
          >
            <SupportLevelLegend />
          </Panel>
          <Panel title="Device backends" description="Drivers that look for hardware.">
            {overview && overview.drivers.length > 0 ? (
              <ul className="flex flex-col gap-3">
                {overview.drivers.map((d) => (
                  <li key={d.id}>
                    <p className="text-sm font-medium">{d.name}</p>
                    <p className="text-[13px] text-muted">{d.description}</p>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-[13px] text-muted">
                None enabled yet. Planned first: generic USB HID enumeration, then manufacturer
                protocols contributed as separate drivers.
              </p>
            )}
          </Panel>
        </div>
      </div>
    </>
  );
}
