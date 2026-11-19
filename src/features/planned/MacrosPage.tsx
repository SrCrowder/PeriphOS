import { PlannedFeaturePage } from "./PlannedFeaturePage";

export function MacrosPage() {
  return (
    <PlannedFeaturePage
      title="Macros"
      milestone="Phase 4"
      summary="Record and edit sequences of key presses, mouse clicks and delays, then bind them to any button."
      scope={[
        "Record keyboard and mouse sequences, or build them step by step.",
        "Exact or randomized delays between steps.",
        "Play once, repeat while held, or toggle on and off.",
        "Enable or disable each macro per profile.",
      ]}
      limitations={
        <p>
          Macros run in software unless a device can store them in its own memory, and each macro
          will say which applies. Some games with anti-cheat block or ban simulated input, so
          PeriphOS will never hide that it is sending synthetic events.
        </p>
      }
    />
  );
}
