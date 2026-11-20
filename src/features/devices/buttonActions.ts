import { mediaLabel, shortcutLabel } from "@/lib/hidKeys";
import type { ButtonAction, MouseButton, SpecialFunction } from "@/types/buttons";

export const MOUSE_BUTTON_LABEL: Record<MouseButton, string> = {
  left: "Left click",
  right: "Right click",
  middle: "Middle click",
  back: "Back",
  forward: "Forward",
};

export const SPECIAL_LABEL: Record<SpecialFunction, string> = {
  wheelLeft: "Scroll left",
  wheelRight: "Scroll right",
  dpiUp: "DPI up",
  dpiDown: "DPI down",
  dpiCycle: "Cycle DPI",
  dpiDefault: "Default DPI",
  dpiShift: "DPI shift (lower while held)",
  profileNext: "Next onboard profile",
  profilePrevious: "Previous onboard profile",
  profileCycle: "Cycle onboard profiles",
  gShift: "G-shift (second layer while held)",
  batteryLevel: "Show battery level",
};

export function actionLabel(action: ButtonAction): string {
  switch (action.type) {
    case "mouse":
      return MOUSE_BUTTON_LABEL[action.button];
    case "key":
      return shortcutLabel(action.key, action.modifiers);
    case "media":
      return mediaLabel(action.usage);
    case "special":
      return SPECIAL_LABEL[action.function];
    case "disabled":
      return "Does nothing";
    case "other":
      return action.description;
  }
}

export function sameAction(a: ButtonAction, b: ButtonAction): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
