import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "@/lib/cn";

interface PanelProps extends Omit<HTMLAttributes<HTMLElement>, "title"> {
  title?: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  /** Remove the inner padding (for lists that draw their own rows). */
  flush?: boolean;
}

/** Bordered surface used to group related content. */
export function Panel({
  title,
  description,
  actions,
  flush = false,
  className,
  children,
  ...rest
}: PanelProps) {
  const hasHeader = title || description || actions;
  return (
    <section
      className={cn("rounded-module bg-surface shadow-module", className)}
      {...rest}
    >
      {hasHeader && (
        <header className="flex items-start justify-between gap-4 px-6 pt-5 pb-3">
          <div className="min-w-0">
            {title && <h2 className="text-[15px] font-bold [font-stretch:112%]">{title}</h2>}
            {description && <p className="mt-0.5 text-[13px] text-muted">{description}</p>}
          </div>
          {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
        </header>
      )}
      <div className={cn(!flush && "px-6 pb-6", !hasHeader && !flush && "pt-6")}>{children}</div>
    </section>
  );
}
