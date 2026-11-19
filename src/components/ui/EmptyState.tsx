import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

interface EmptyStateProps {
  icon: ReactNode;
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
  className?: string;
}

export function EmptyState({ icon, title, children, actions, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center px-6 py-12 text-center", className)}>
      <div className="mb-4 grid size-14 place-items-center rounded-2xl border-2 border-dashed border-line-strong text-subtle [&_svg]:size-5">
        {icon}
      </div>
      <h3 className="text-[15px] font-bold [font-stretch:112%]">{title}</h3>
      {children && <div className="mt-1.5 max-w-md text-[13px] text-muted">{children}</div>}
      {actions && <div className="mt-5 flex gap-2">{actions}</div>}
    </div>
  );
}
