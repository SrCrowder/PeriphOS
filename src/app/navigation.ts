import type { PageId } from "@/stores/navigation";

export interface NavItem {
  id: PageId;
  label: string;
  /** Roadmap milestone when the section is not implemented yet. */
  planned?: string;
}

export const NAV_ITEMS: readonly NavItem[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "devices", label: "Devices" },
  { id: "profiles", label: "Profiles" },
  { id: "macros", label: "Macros", planned: "Phase 4" },
  { id: "rgb", label: "RGB Studio", planned: "Phase 5" },
  { id: "settings", label: "Settings" },
];
