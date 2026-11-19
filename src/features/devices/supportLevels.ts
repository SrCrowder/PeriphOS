import type { BadgeTone } from "@/components/ui";
import type { SupportLevel } from "@/types/device";

/** Single source of truth for how support levels are named and explained. */
export const SUPPORT_LEVELS: Record<
  SupportLevel,
  { label: string; tone: BadgeTone; description: string }
> = {
  native: {
    label: "On device",
    tone: "success",
    description: "Saved to the device's own memory. Keeps working without PeriphOS running.",
  },
  software: {
    label: "Software",
    tone: "info",
    description: "Applied by PeriphOS at the operating-system level. Works while PeriphOS runs.",
  },
  vendorProtocol: {
    label: "Vendor protocol",
    tone: "warning",
    description: "The hardware has it, but it needs a manufacturer protocol PeriphOS does not speak yet.",
  },
  unsupported: {
    label: "Unavailable",
    tone: "neutral",
    description: "Not available for this device.",
  },
};

export const SUPPORT_LEVEL_ORDER: readonly SupportLevel[] = [
  "native",
  "software",
  "vendorProtocol",
  "unsupported",
];
