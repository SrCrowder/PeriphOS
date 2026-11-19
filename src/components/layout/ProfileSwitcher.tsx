import { Check, ChevronUp, Radar, Settings2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { cn } from "@/lib/cn";
import { errorMessage } from "@/lib/errors";
import { useNavigation } from "@/stores/navigation";
import { reportActivation, selectActiveProfile, useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";

import { Led } from "./Led";

/**
 * The sidebar's active-profile card. Opens a menu to pick a profile (manual
 * mode), return to automatic switching, or manage profiles.
 */
export function ProfileSwitcher() {
  const { profiles, activeId, mode, matchedApp, automaticSupported, activate, resumeAutomatic } = useProfiles();
  const active = useProfiles(selectActiveProfile);
  const navigate = useNavigation((s) => s.navigate);
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onPointer = (e: PointerEvent) => {
      if (!root.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    try {
      await action();
      setOpen(false);
    } catch (e) {
      toast.error(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const manual = mode === "manual";
  const subtitle = manual ? "Chosen manually" : matchedApp ? `For ${matchedApp}` : "Automatic";

  return (
    <div ref={root} className="relative mt-3">
      {open && (
        <div
          role="menu"
          aria-label="Profiles"
          className="absolute right-0 bottom-full left-0 z-30 mb-2 flex animate-pop-in flex-col gap-1 rounded-2xl border border-line bg-raised p-2 shadow-pop"
        >
          {manual && (
            <div className="mb-1 rounded-xl bg-accent-soft p-2.5">
              <p className="px-0.5 text-[11.5px] text-muted">
                {automaticSupported
                  ? "Automatic switching is paused."
                  : "Automatic switching follows games on Windows."}
              </p>
              <button
                type="button"
                role="menuitem"
                disabled={busy}
                onClick={() => void run(resumeAutomatic)}
                className="mt-2 flex w-full items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-[13px] font-bold text-accent-fg transition-[filter] hover:brightness-110 disabled:opacity-60"
              >
                <Radar className="size-4" /> Switch automatically
              </button>
            </div>
          )}
          <p className="px-2 pt-1 pb-0.5 text-[11.5px] font-semibold text-subtle">Use profile</p>
          <ul className="flex max-h-64 flex-col gap-0.5 overflow-y-auto">
            {profiles.map((profile) => {
              const current = profile.id === activeId;
              return (
                <li key={profile.id}>
                  <button
                    type="button"
                    role="menuitemradio"
                    aria-checked={current && manual}
                    disabled={busy}
                    onClick={() =>
                      void run(async () => {
                        reportActivation(await activate(profile.id));
                      })
                    }
                    className={cn(
                      "flex w-full items-center gap-2.5 rounded-lg px-2 py-2 text-left text-[13px] transition-colors hover:bg-hover",
                      current ? "font-bold" : "font-medium text-muted hover:text-fg",
                    )}
                  >
                    <Led on={current} className={current ? "text-accent" : undefined} />
                    <span className="min-w-0 flex-1 truncate">{profile.name}</span>
                    {profile.isDefault && <span className="text-[11px] font-normal text-subtle">Default</span>}
                    {current && <Check className="size-3.5 text-accent" />}
                  </button>
                </li>
              );
            })}
          </ul>
          <button
            type="button"
            role="menuitem"
            onClick={() => {
              setOpen(false);
              navigate("profiles");
            }}
            className="mt-1 flex items-center gap-2 border-t border-line px-2 pt-2.5 pb-1 text-[13px] font-medium text-muted hover:text-fg"
          >
            <Settings2 className="size-4" /> Manage profiles
          </button>
        </div>
      )}

      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((o) => !o)}
        className={cn(
          "flex w-full items-center gap-2 rounded-2xl bg-raised p-3 text-left shadow-module transition-colors hover:bg-hover",
          manual && "ring-1 ring-accent-line",
        )}
      >
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-2 text-xs text-subtle">
            <Led on className="text-accent" /> Active profile
          </span>
          <span className="mt-1 block truncate text-sm font-bold">{active?.name ?? "Loading…"}</span>
          <span className={cn("mt-0.5 block truncate text-[11.5px]", manual ? "text-accent" : "text-subtle")}>
            {subtitle}
          </span>
        </span>
        <ChevronUp className={cn("size-4 shrink-0 text-subtle transition-transform", !open && "rotate-180")} />
      </button>
    </div>
  );
}
