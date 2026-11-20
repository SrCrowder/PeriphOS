// Keyboard keys as HID usages (USB HID Usage Tables, keyboard page 0x07),
// mapped from KeyboardEvent.code so a key can be captured by pressing it.

const CODES: Record<string, [usage: number, label: string]> = {
  Enter: [0x28, "Enter"],
  Escape: [0x29, "Esc"],
  Backspace: [0x2a, "Backspace"],
  Tab: [0x2b, "Tab"],
  Space: [0x2c, "Space"],
  Minus: [0x2d, "-"],
  Equal: [0x2e, "="],
  BracketLeft: [0x2f, "["],
  BracketRight: [0x30, "]"],
  Backslash: [0x31, "\\"],
  Semicolon: [0x33, ";"],
  Quote: [0x34, "'"],
  Backquote: [0x35, "`"],
  Comma: [0x36, ","],
  Period: [0x37, "."],
  Slash: [0x38, "/"],
  CapsLock: [0x39, "Caps Lock"],
  PrintScreen: [0x46, "Print Screen"],
  ScrollLock: [0x47, "Scroll Lock"],
  Pause: [0x48, "Pause"],
  Insert: [0x49, "Insert"],
  Home: [0x4a, "Home"],
  PageUp: [0x4b, "Page Up"],
  Delete: [0x4c, "Delete"],
  End: [0x4d, "End"],
  PageDown: [0x4e, "Page Down"],
  ArrowRight: [0x4f, "Right"],
  ArrowLeft: [0x50, "Left"],
  ArrowDown: [0x51, "Down"],
  ArrowUp: [0x52, "Up"],
  NumLock: [0x53, "Num Lock"],
  NumpadDivide: [0x54, "Num /"],
  NumpadMultiply: [0x55, "Num *"],
  NumpadSubtract: [0x56, "Num -"],
  NumpadAdd: [0x57, "Num +"],
  NumpadEnter: [0x58, "Num Enter"],
  NumpadDecimal: [0x63, "Num ."],
  IntlBackslash: [0x64, "<"],
  ContextMenu: [0x65, "Menu"],
  ControlLeft: [0xe0, "Ctrl"],
  ShiftLeft: [0xe1, "Shift"],
  AltLeft: [0xe2, "Alt"],
  MetaLeft: [0xe3, "Win"],
  ControlRight: [0xe4, "Right Ctrl"],
  ShiftRight: [0xe5, "Right Shift"],
  AltRight: [0xe6, "Right Alt"],
  MetaRight: [0xe7, "Right Win"],
};

for (let i = 0; i < 26; i++) {
  const letter = String.fromCharCode(65 + i);
  CODES[`Key${letter}`] = [0x04 + i, letter];
}
for (let i = 1; i <= 9; i++) {
  CODES[`Digit${i}`] = [0x1d + i, String(i)];
  CODES[`Numpad${i}`] = [0x58 + i, `Num ${i}`];
}
CODES.Digit0 = [0x27, "0"];
CODES.Numpad0 = [0x62, "Num 0"];
for (let i = 1; i <= 12; i++) CODES[`F${i}`] = [0x39 + i, `F${i}`];
for (let i = 13; i <= 24; i++) CODES[`F${i}`] = [0x68 + i - 13, `F${i}`];

const LABELS = new Map<number, string>(Object.values(CODES).map(([usage, label]) => [usage, label]));

/** Modifier bits, as stored with a key. */
export const MODIFIERS = [
  { bit: 0x01, label: "Ctrl" },
  { bit: 0x02, label: "Shift" },
  { bit: 0x04, label: "Alt" },
  { bit: 0x08, label: "Win" },
] as const;

/** HID usage for a KeyboardEvent.code, or null for keys that cannot be stored. */
export function usageForCode(code: string): number | null {
  return CODES[code]?.[0] ?? null;
}

export function keyLabel(usage: number): string {
  return LABELS.get(usage) ?? `Key 0x${usage.toString(16).toUpperCase().padStart(2, "0")}`;
}

/** "Ctrl + Shift + A". A modifier pressed alone is shown once. */
export function shortcutLabel(key: number, modifiers: number): string {
  const parts: string[] = MODIFIERS.filter((m) => modifiers & m.bit).map((m) => m.label);
  const label = keyLabel(key);
  if (!parts.includes(label)) parts.push(label);
  return parts.join(" + ");
}

/** Modifier bits held in a keyboard event, leaving out the key itself when it is a modifier. */
export function modifiersOf(
  event: { ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean },
  usage: number,
): number {
  let bits = 0;
  if (event.ctrlKey && usage !== 0xe0 && usage !== 0xe4) bits |= 0x01;
  if (event.shiftKey && usage !== 0xe1 && usage !== 0xe5) bits |= 0x02;
  if (event.altKey && usage !== 0xe2 && usage !== 0xe6) bits |= 0x04;
  if (event.metaKey && usage !== 0xe3 && usage !== 0xe7) bits |= 0x08;
  return bits;
}

/** Media and system keys (HID consumer page). */
export const MEDIA_KEYS = [
  { usage: 0xcd, label: "Play / pause" },
  { usage: 0xb5, label: "Next track" },
  { usage: 0xb6, label: "Previous track" },
  { usage: 0xb7, label: "Stop" },
  { usage: 0xe9, label: "Volume up" },
  { usage: 0xea, label: "Volume down" },
  { usage: 0xe2, label: "Mute" },
] as const;

export function mediaLabel(usage: number): string {
  return MEDIA_KEYS.find((m) => m.usage === usage)?.label ?? `Media key 0x${usage.toString(16).toUpperCase()}`;
}
