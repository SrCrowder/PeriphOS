const relative = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 365 * 24 * 3600],
  ["month", 30 * 24 * 3600],
  ["week", 7 * 24 * 3600],
  ["day", 24 * 3600],
  ["hour", 3600],
  ["minute", 60],
];

/** "3 minutes ago", "yesterday", "just now". */
export function formatRelative(timestampMs: number, now = Date.now()): string {
  const seconds = Math.round((timestampMs - now) / 1000);
  for (const [unit, size] of UNITS) {
    if (Math.abs(seconds) >= size) return relative.format(Math.round(seconds / size), unit);
  }
  return "just now";
}

export function formatDateTime(timestampMs: number): string {
  return new Date(timestampMs).toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

/** USB ids are conventionally shown as four hex digits. */
export function formatUsbId(id: number): string {
  return id.toString(16).padStart(4, "0").toUpperCase();
}

export const PLATFORM_LABEL = { windows: "Windows", linux: "Linux", macOs: "macOS" } as const;
