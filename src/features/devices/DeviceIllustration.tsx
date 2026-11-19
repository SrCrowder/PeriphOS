import type { DeviceKind } from "@/types/device";

/**
 * Generic outline of the device type, drawn with theme tokens. It represents
 * the kind of device only; it is not a picture of the specific model.
 */
export function DeviceIllustration({ kind, className }: { kind: DeviceKind; className?: string }) {
  const stroke = "stroke-fg/80";
  return (
    <svg viewBox="0 0 200 140" className={className} aria-hidden fill="none" strokeWidth="5" strokeLinecap="round">
      {kind === "mouse" && (
        <g>
          <path className={stroke} d="M100 12c-30 0-46 20-46 48v24c0 26 18 46 46 46s46-20 46-46V60c0-28-16-48-46-48Z" />
          <path className={stroke} d="M54 62h92M100 12v50" />
          <rect x="94" y="26" width="12" height="22" rx="6" className="fill-accent stroke-none" />
        </g>
      )}
      {kind === "keyboard" && (
        <g>
          <rect x="12" y="30" width="176" height="80" rx="14" className={stroke} />
          {[0, 1, 2].map((row) =>
            Array.from({ length: 9 - row }, (_, i) => (
              <rect
                key={`${row}-${i}`}
                x={28 + row * 8 + i * 17}
                y={44 + row * 17}
                width="11"
                height="11"
                rx="3"
                className={row === 1 && i === 2 ? "fill-accent" : "fill-fg/25"}
              />
            )),
          )}
          <rect x="56" y="95" width="88" height="7" rx="3.5" className="fill-fg/25" />
        </g>
      )}
      {kind === "receiver" && (
        <g>
          <rect x="70" y="40" width="60" height="70" rx="12" className={stroke} />
          <path className={stroke} d="M84 40V22h32v18" />
          <circle cx="100" cy="75" r="7" className="fill-accent stroke-none" />
        </g>
      )}
      {(kind === "controller" || kind === "headset" || kind === "other") && (
        <g>
          <path className={stroke} d="M52 44h96c20 0 32 24 34 52 1 18-16 22-26 10l-14-16H58l-14 16c-10 12-27 8-26-10 2-28 14-52 34-52Z" />
          <circle cx="136" cy="66" r="7" className="fill-accent stroke-none" />
          <path className={stroke} d="M64 56v20M54 66h20" />
        </g>
      )}
    </svg>
  );
}
