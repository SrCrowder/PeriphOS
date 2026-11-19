import { PlannedFeaturePage } from "./PlannedFeaturePage";

export function RgbStudioPage() {
  return (
    <PlannedFeaturePage
      title="RGB Studio"
      milestone="Phase 5"
      summary="One lighting editor for every device that exposes lighting control."
      scope={[
        "Static colors, presets and animated effects.",
        "Per-zone and per-key editing where the hardware allows it.",
        "Lighting saved as part of each profile.",
      ]}
      limitations={
        <p>
          Lighting control almost always needs a manufacturer-specific protocol, so support will
          grow one device family at a time. Devices whose lighting PeriphOS cannot drive will be
          marked as such instead of showing controls that do nothing.
        </p>
      }
    />
  );
}
