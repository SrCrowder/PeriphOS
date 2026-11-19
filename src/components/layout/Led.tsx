import { cn } from "@/lib/cn";

/** Small status light, the recurring "hardware" detail of the interface. */
export function Led({ on, className }: { on: boolean; className?: string }) {
  return (
    <span
      aria-hidden
      className={cn("size-1.5 shrink-0 rounded-full", on ? "bg-current" : "bg-led-off", className)}
    />
  );
}
