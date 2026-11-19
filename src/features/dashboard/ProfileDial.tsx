/**
 * Knob-like readout showing how many profiles exist. The tick marks one step
 * per profile (up to 12) around the dial, like detents on a control knob.
 */
export function ProfileDial({ count }: { count: number }) {
  const ticks = Math.min(Math.max(count, 1), 12);
  return (
    <div className="relative grid size-36 shrink-0 place-items-center rounded-full bg-bg text-accent">
      {Array.from({ length: ticks }, (_, i) => (
        <span
          key={i}
          aria-hidden
          className="absolute top-1/2 left-1/2 h-4 w-1 rounded-full bg-accent"
          style={{ transform: `rotate(${i * 30}deg) translateY(-56px)`, translate: "-50% -50%" }}
        />
      ))}
      <div className="text-center">
        <p className="font-display text-4xl leading-none tabular-nums">{count}</p>
        <p className="mt-1 text-[11px] font-semibold text-muted">
          {count === 1 ? "profile" : "profiles"}
        </p>
      </div>
    </div>
  );
}
