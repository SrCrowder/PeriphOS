import { Check } from "lucide-react";

import { cn } from "@/lib/cn";
import { ACCENTS, type Accent } from "@/types/settings";

export function AccentPicker({ value, onChange }: { value: Accent; onChange: (a: Accent) => void }) {
  return (
    <div role="radiogroup" aria-label="Accent color" className="flex gap-2">
      {ACCENTS.map((accent) => {
        const active = accent === value;
        return (
          <button
            key={accent}
            type="button"
            role="radio"
            aria-checked={active}
            aria-label={accent}
            title={accent[0]!.toUpperCase() + accent.slice(1)}
            data-accent={accent}
            onClick={() => onChange(accent)}
            className={cn(
              "grid size-7 place-items-center rounded-full bg-accent text-accent-fg ring-offset-2 ring-offset-surface transition-shadow",
              active ? "ring-2 ring-accent" : "hover:ring-2 hover:ring-line-strong",
            )}
          >
            {active && <Check className="size-3.5" strokeWidth={3} />}
          </button>
        );
      })}
    </div>
  );
}
