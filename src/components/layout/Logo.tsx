/**
 * PeriphOS mark ("Dial"): a control knob on a signal tile. Drawn with theme
 * tokens so it follows the user's accent color. The static app icon lives in
 * public/periphos.svg and docs/design/logos/.
 */
export function LogoMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" className={className} aria-hidden>
      <rect width="64" height="64" rx="16" className="fill-accent" />
      <circle cx="32" cy="32" r="19" className="fill-bg" />
      <rect x="29.5" y="16" width="5" height="12" rx="2.5" className="fill-accent" transform="rotate(35 32 32)" />
      <g className="fill-bg">
        <circle cx="32" cy="7.5" r="2" />
        <circle cx="49.5" cy="14.5" r="2" />
        <circle cx="56.5" cy="32" r="2" />
      </g>
    </svg>
  );
}

export function Logo() {
  return (
    <span className="flex items-center gap-2.5">
      <LogoMark className="size-8 shrink-0" />
      <span className="font-display text-[19px] leading-none">
        Periph<span className="text-accent">OS</span>
      </span>
    </span>
  );
}
