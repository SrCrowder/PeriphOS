import { CircleAlert, CircleCheck, Info, X } from "lucide-react";

import { cn } from "@/lib/cn";
import { useToasts, type ToastTone } from "@/stores/toasts";

const icons: Record<ToastTone, typeof Info> = {
  success: CircleCheck,
  error: CircleAlert,
  info: Info,
};

const colors: Record<ToastTone, string> = {
  success: "text-success",
  error: "text-danger",
  info: "text-info",
};

export function Toaster() {
  const toasts = useToasts((s) => s.toasts);
  const dismiss = useToasts((s) => s.dismiss);

  return (
    <div aria-live="polite" className="pointer-events-none fixed right-5 bottom-5 z-50 flex w-80 flex-col gap-2">
      {toasts.map((t) => {
        const Icon = icons[t.tone];
        return (
          <div
            key={t.id}
            role={t.tone === "error" ? "alert" : "status"}
            className="pointer-events-auto flex animate-pop-in items-start gap-3 rounded-2xl border border-line bg-raised px-4 py-3 text-sm shadow-pop"
          >
            <Icon className={cn("mt-0.5 size-4 shrink-0", colors[t.tone])} />
            <p className="flex-1">{t.message}</p>
            <button
              type="button"
              aria-label="Dismiss"
              onClick={() => dismiss(t.id)}
              className="text-subtle hover:text-fg"
            >
              <X className="size-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
