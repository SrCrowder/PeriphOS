import { useState } from "react";

import { cn } from "@/lib/cn";
import type { ButtonBinding } from "@/types/buttons";

import { actionLabel } from "./buttonActions";
import { CARD_WIDTH, type MouseDiagram, VIEW_HEIGHT, VIEW_WIDTH } from "./mouseDiagrams";

/** Where a leader line leaves its card, in view-box units. */
function cardEdge(side: "left" | "right", y: number): [number, number] {
  return side === "left" ? [CARD_WIDTH, y] : [VIEW_WIDTH - CARD_WIDTH, y];
}

/**
 * The mouse seen from above, with a label card per button joined to it by a
 * line. Hovering or focusing a card highlights its button; clicking it opens
 * the assignment dialog.
 */
export function MouseDiagramView({
  diagram,
  buttons,
  modified,
  disabled,
  onSelect,
}: {
  diagram: MouseDiagram;
  buttons: ButtonBinding[];
  /** Indexes of buttons that differ from the factory settings. */
  modified: Set<number>;
  disabled: boolean;
  onSelect: (binding: ButtonBinding) => void;
}) {
  const [hovered, setHovered] = useState<number | null>(null);
  const byIndex = new Map(buttons.map((b) => [b.index, b]));

  return (
    <div className="relative w-full" style={{ aspectRatio: `${VIEW_WIDTH} / ${VIEW_HEIGHT}` }}>
      <svg
        viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
        className="absolute inset-0 size-full"
        aria-hidden="true"
      >
        <path d={diagram.body} className="fill-raised stroke-line-strong" strokeWidth={2} />
        {diagram.wells.map((d) => (
          <path key={d} d={d} className="fill-bg stroke-line-strong" strokeWidth={1.5} />
        ))}
        {diagram.lines.map((d) => (
          <path key={d} d={d} className="fill-none stroke-line-strong" strokeWidth={1.5} />
        ))}
        {diagram.parts.map((part) => (
          <path
            key={part.index}
            d={part.shape}
            className={cn(
              "stroke-line-strong transition-colors duration-150",
              hovered === part.index ? "fill-accent-soft stroke-accent" : "fill-hover",
            )}
            strokeWidth={1.5}
          />
        ))}
        {diagram.buttons.map((b) => {
          const [x1, y1] = cardEdge(b.side, b.cardY);
          const [x2, y2] = b.anchor;
          const bend = b.side === "left" ? x1 + 24 : x1 - 24;
          const active = hovered === b.index;
          return (
            <g key={b.index} className={active ? "text-accent" : "text-line-strong"}>
              <polyline
                points={`${x1},${y1} ${bend},${y1} ${x2},${y2}`}
                fill="none"
                stroke="currentColor"
                strokeWidth={active ? 2 : 1.25}
              />
              <circle cx={x2} cy={y2} r={active ? 6 : 4.5} fill="currentColor" />
            </g>
          );
        })}
      </svg>

      {diagram.buttons.map((b) => {
        const binding = byIndex.get(b.index);
        if (!binding) return null;
        const left = b.side === "left";
        return (
          <button
            key={b.index}
            type="button"
            disabled={disabled}
            onClick={() => onSelect(binding)}
            onMouseEnter={() => setHovered(b.index)}
            onMouseLeave={() => setHovered(null)}
            onFocus={() => setHovered(b.index)}
            onBlur={() => setHovered(null)}
            style={{
              top: `${(b.cardY / VIEW_HEIGHT) * 100}%`,
              width: `${(CARD_WIDTH / VIEW_WIDTH) * 100}%`,
              [left ? "left" : "right"]: 0,
            }}
            className={cn(
              "absolute -translate-y-1/2 rounded-xl border px-3 py-1.5 transition-colors duration-150",
              "disabled:cursor-not-allowed disabled:opacity-60",
              left ? "text-right" : "text-left",
              hovered === b.index ? "border-accent bg-accent-soft" : "border-line bg-surface hover:bg-hover",
            )}
          >
            <span className="block truncate text-[12px] text-muted">{binding.name}</span>
            <span
              className={cn(
                "flex items-center gap-1.5 text-[13px] font-semibold",
                left ? "justify-end" : "justify-start",
                binding.action.type === "disabled" && "text-subtle",
              )}
            >
              {modified.has(b.index) && left && <ModifiedDot />}
              <span className="truncate">{actionLabel(binding.action)}</span>
              {modified.has(b.index) && !left && <ModifiedDot />}
            </span>
          </button>
        );
      })}
    </div>
  );
}

function ModifiedDot() {
  return (
    <span className="size-1.5 shrink-0 rounded-full bg-accent" title="Changed from the factory settings">
      <span className="sr-only">Changed from the factory settings</span>
    </span>
  );
}
