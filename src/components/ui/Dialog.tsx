import { X } from "lucide-react";
import { useEffect, useRef, type ReactNode } from "react";

import { cn } from "@/lib/cn";

interface DialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  description?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

/**
 * Modal built on the native <dialog> element, which provides focus
 * trapping, Escape handling and an accessible modal role for free.
 * Mark the element that should receive initial focus with `data-autofocus`.
 */
export function Dialog({ open, onClose, title, description, children, footer, className }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open && !dialog.open) {
      dialog.showModal();
      // React's autoFocus runs before showModal(), which then moves focus to
      // the first focusable element. Honor an explicit [data-autofocus] target.
      dialog.querySelector<HTMLElement>("[data-autofocus]")?.focus();
    }
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClick={(event) => {
        // Clicking the backdrop (the dialog element itself) closes it.
        if (event.target === ref.current) onClose();
      }}
      className={cn(
        "m-auto w-[min(460px,calc(100vw-32px))] rounded-module border border-line bg-surface p-0 text-fg shadow-pop backdrop:bg-black/55 open:animate-pop-in",
        className,
      )}
    >
      {open && (
        <div className="flex flex-col">
          <header className="flex items-start justify-between gap-4 px-6 pt-5">
            <div>
              <h2 className="font-display text-lg">{title}</h2>
              {description && <p className="mt-1 text-[13px] text-muted">{description}</p>}
            </div>
            <button
              type="button"
              onClick={onClose}
              aria-label="Close"
              className="-mr-2 rounded-md p-1.5 text-subtle hover:bg-hover hover:text-fg"
            >
              <X className="size-4" />
            </button>
          </header>
          <div className="px-6 py-5">{children}</div>
          {footer && (
            <footer className="flex justify-end gap-2 border-t border-line px-6 py-4">{footer}</footer>
          )}
        </div>
      )}
    </dialog>
  );
}
