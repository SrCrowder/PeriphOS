import { Copy, FileSearch } from "lucide-react";
import { useState } from "react";

import { Button, Panel } from "@/components/ui";
import { errorMessage } from "@/lib/errors";
import { api } from "@/lib/ipc";
import { toast } from "@/stores/toasts";
import type { DeviceInfo } from "@/types/device";

/** Drivers that can produce a technical report. */
const DRIVERS_WITH_REPORT = new Set(["logitech-hidpp"]);

export function hasTechnicalReport(device: DeviceInfo): boolean {
  return DRIVERS_WITH_REPORT.has(device.driverId);
}

/**
 * Read-only dump of what the device exposes (protocol features, programmable
 * buttons, onboard memory). Users share it so support for their device can
 * be added from real data.
 */
export function TechnicalReportPanel({ device }: { device: DeviceInfo }) {
  const [report, setReport] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const read = async () => {
    setLoading(true);
    try {
      setReport(await api.deviceDiagnostics(device.id));
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const copy = async () => {
    if (!report) return;
    try {
      await navigator.clipboard.writeText(`${device.name}\n${report}`);
      toast.success("Technical report copied.");
    } catch {
      toast.error("Could not access the clipboard.");
    }
  };

  return (
    <Panel
      title="Technical report"
      description="What the device reports about itself: its features, programmable buttons and onboard memory. Reading it changes nothing on the device."
      actions={
        report ? (
          <Button size="sm" icon={<Copy className="size-3.5" />} onClick={() => void copy()}>
            Copy
          </Button>
        ) : null
      }
    >
      {report ? (
        <pre className="max-h-96 overflow-auto rounded-xl bg-raised p-3 font-mono text-[11px] leading-relaxed whitespace-pre select-text">
          {report}
        </pre>
      ) : (
        <Button
          size="sm"
          icon={<FileSearch className="size-3.5" />}
          loading={loading}
          onClick={() => void read()}
        >
          Read report
        </Button>
      )}
    </Panel>
  );
}
