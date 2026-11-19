import { isTauri } from "@/lib/ipc";

/**
 * Shown when the UI runs in a regular browser. In that mode the in-memory
 * preview backend answers commands: nothing is saved and no hardware is read.
 */
export function PreviewBanner() {
  if (isTauri) return null;
  return (
    <div className="border-b border-warning/30 bg-warning/10 px-5 py-1.5 text-center text-xs text-warning">
      Browser preview: changes live in memory only and no hardware is accessed. Run{" "}
      <code className="font-semibold">npm run tauri dev</code> for the desktop app.
    </div>
  );
}
