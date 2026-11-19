import { Zap } from "lucide-react";

import { cn } from "@/lib/cn";
import type { BatteryLevel, BatteryState } from "@/types/device";

const LEVEL_PERCENT: Record<BatteryLevel, number> = {
  full: 100,
  good: 60,
  low: 20,
  critical: 8,
  empty: 0,
};

const LEVEL_LABEL: Record<BatteryLevel, string> = {
  full: "Full",
  good: "Good",
  low: "Low",
  critical: "Critical",
  empty: "Empty",
};

/** Fill fraction used for drawing; null when nothing usable was reported. */
function fillPercent(battery: BatteryState): number | null {
  if (battery.percent !== null) return battery.percent;
  if (battery.level) return LEVEL_PERCENT[battery.level];
  return battery.charging === "full" ? 100 : null;
}

export function batteryText(battery: BatteryState): string {
  if (battery.charging === "error") return "Battery error";
  const value =
    battery.percent !== null
      ? `${battery.estimated ? "about " : ""}${battery.percent}%`
      : battery.level
        ? LEVEL_LABEL[battery.level]
        : battery.charging === "full"
          ? "Full"
          : "Unknown";
  if (battery.charging === "charging") return `${value}, charging`;
  return value;
}

/** Small battery glyph with a fill that turns red when low. */
export function BatteryIndicator({ battery, showText = true }: { battery: BatteryState; showText?: boolean }) {
  const fill = fillPercent(battery);
  const low = fill !== null && fill <= 20 && battery.charging !== "charging";
  return (
    <span className="inline-flex items-center gap-1.5 text-[13px] font-semibold tabular-nums" title={batteryText(battery)}>
      <span className="relative inline-flex h-3 w-6 items-center rounded-[3px] border-2 border-current p-px text-muted">
        <span
          className={cn("block h-full rounded-[1px]", low ? "bg-danger" : "bg-success")}
          style={{ width: `${fill ?? 0}%` }}
        />
        <span className="absolute -right-[4px] h-1.5 w-[2px] rounded-r-sm bg-current" />
      </span>
      {battery.charging === "charging" && <Zap className="size-3.5 text-warning" aria-label="Charging" />}
      {showText && (
        <span className={low ? "text-danger" : undefined}>
          {battery.percent !== null
            ? `${battery.estimated ? "~" : ""}${battery.percent}%`
            : battery.level
              ? LEVEL_LABEL[battery.level]
              : ""}
        </span>
      )}
    </span>
  );
}
