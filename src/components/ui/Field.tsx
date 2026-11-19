import { useId, type InputHTMLAttributes, type ReactNode, type TextareaHTMLAttributes } from "react";

import { cn } from "@/lib/cn";

const control =
  "w-full rounded-xl border border-transparent bg-raised px-3.5 text-sm text-fg placeholder:text-subtle transition-colors hover:border-line-strong focus:border-accent focus:outline-none focus-visible:outline-none aria-invalid:border-danger";

interface FieldShellProps {
  label: string;
  hint?: ReactNode;
  error?: string | null;
  counter?: { value: number; max: number };
  children: (props: { id: string; describedBy: string | undefined }) => ReactNode;
}

function FieldShell({ label, hint, error, counter, children }: FieldShellProps) {
  const id = useId();
  const hintId = `${id}-hint`;
  const showHint = Boolean(error || hint);
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-baseline justify-between">
        <label htmlFor={id} className="text-[13px] font-medium">
          {label}
        </label>
        {counter && (
          <span
            className={cn(
              "text-xs tabular-nums",
              counter.value > counter.max ? "text-danger" : "text-subtle",
            )}
          >
            {counter.value}/{counter.max}
          </span>
        )}
      </div>
      {children({ id, describedBy: showHint ? hintId : undefined })}
      {showHint && (
        <p id={hintId} className={cn("text-xs", error ? "text-danger" : "text-subtle")}>
          {error ?? hint}
        </p>
      )}
    </div>
  );
}

type Shared = Omit<FieldShellProps, "children">;

export function TextField({
  label,
  hint,
  error,
  counter,
  className,
  ...rest
}: Shared & InputHTMLAttributes<HTMLInputElement>) {
  return (
    <FieldShell label={label} hint={hint} error={error} counter={counter}>
      {({ id, describedBy }) => (
        <input
          id={id}
          aria-describedby={describedBy}
          aria-invalid={error ? true : undefined}
          className={cn(control, "h-10", className)}
          {...rest}
        />
      )}
    </FieldShell>
  );
}

export function TextAreaField({
  label,
  hint,
  error,
  counter,
  className,
  ...rest
}: Shared & TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <FieldShell label={label} hint={hint} error={error} counter={counter}>
      {({ id, describedBy }) => (
        <textarea
          id={id}
          aria-describedby={describedBy}
          aria-invalid={error ? true : undefined}
          className={cn(control, "min-h-20 resize-none py-2", className)}
          {...rest}
        />
      )}
    </FieldShell>
  );
}
