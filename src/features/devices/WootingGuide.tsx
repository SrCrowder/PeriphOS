import { Info } from "lucide-react";

/** How PeriphOS works with a Wooting keyboard, and what the user has to do once. */
export function WootingGuide() {
  return (
    <div className="flex gap-3 rounded-2xl bg-info/10 px-4 py-3.5 text-[13px]">
      <Info className="mt-0.5 size-4 shrink-0 text-info" />
      <div className="flex flex-col gap-2">
        <p className="font-semibold text-fg">How it works with your Wooting</p>
        <ol className="flex list-decimal flex-col gap-1.5 pl-4 text-muted marker:text-subtle">
          <li>
            Set up the keyboard&apos;s four profiles in <span className="font-semibold text-fg">Wootility</span>:
            actuation point, rapid trigger, key bindings and lighting are stored in each one.
          </li>
          <li>
            In Wootility, turn off <span className="font-semibold text-fg">App Linking</span>, so it does not
            switch profiles at the same time as PeriphOS.
          </li>
          <li>
            Close Wootility completely, including its background service in the system tray. Both
            apps use the same connection to the keyboard.
          </li>
          <li>
            Choose a keyboard profile below. It is saved to the active PeriphOS profile, so each
            game can use a different keyboard profile.
          </li>
        </ol>
      </div>
    </div>
  );
}
