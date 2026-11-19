import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/** A label/description on the left and a control on the right. */
export function SettingRow({
  title,
  description,
  control,
  disabled = false,
}: {
  title: ReactNode;
  description?: ReactNode;
  control: ReactNode;
  disabled?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-6 border-t border-line py-4 first:border-t-0 first:pt-0 last:pb-0">
      <div className={cn("min-w-0", disabled && "opacity-60")}>
        <div className="flex items-center gap-2 text-sm font-medium">{title}</div>
        {description && <p className="mt-0.5 text-[13px] text-muted">{description}</p>}
      </div>
      <div className="shrink-0">{control}</div>
    </div>
  );
}
