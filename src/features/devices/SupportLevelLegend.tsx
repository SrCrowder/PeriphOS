import { Badge } from "@/components/ui";

import { SUPPORT_LEVEL_ORDER, SUPPORT_LEVELS } from "./supportLevels";

/** Explains the labels every device feature will carry. */
export function SupportLevelLegend() {
  return (
    <dl className="grid gap-x-6 gap-y-4 sm:grid-cols-2">
      {SUPPORT_LEVEL_ORDER.map((level) => {
        const { label, tone, description } = SUPPORT_LEVELS[level];
        return (
          <div key={level} className="flex flex-col gap-1.5">
            <dt>
              <Badge tone={tone}>{label}</Badge>
            </dt>
            <dd className="text-[13px] text-muted">{description}</dd>
          </div>
        );
      })}
    </dl>
  );
}
