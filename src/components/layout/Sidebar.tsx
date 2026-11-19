import { NAV_ITEMS } from "@/app/navigation";
import { cn } from "@/lib/cn";
import { useAppInfo } from "@/stores/appInfo";
import { useNavigation } from "@/stores/navigation";
import { selectDefaultProfile, useProfiles } from "@/stores/profiles";

import { Logo } from "./Logo";

/** Small status light, the recurring "hardware" detail of the interface. */
export function Led({ on, className }: { on: boolean; className?: string }) {
  return (
    <span
      aria-hidden
      className={cn("size-1.5 shrink-0 rounded-full", on ? "bg-current" : "bg-led-off", className)}
    />
  );
}

export function Sidebar() {
  const page = useNavigation((s) => s.page);
  const navigate = useNavigation((s) => s.navigate);
  const activeProfile = useProfiles(selectDefaultProfile);
  const version = useAppInfo((s) => s.info?.version);

  return (
    <aside className="flex w-56 shrink-0 flex-col rounded-module bg-sidebar p-3 shadow-module">
      <div className="px-3 pt-3 pb-6">
        <Logo />
      </div>

      <nav aria-label="Main" className="flex flex-1 flex-col gap-1">
        {NAV_ITEMS.map(({ id, label, planned }) => {
          const active = page === id;
          return (
            <button
              key={id}
              type="button"
              onClick={() => navigate(id)}
              aria-current={active ? "page" : undefined}
              className={cn(
                "flex h-10 items-center gap-3 rounded-xl px-3 text-sm transition-colors duration-150",
                active
                  ? "bg-accent font-bold text-accent-fg"
                  : "font-medium text-muted hover:bg-hover hover:text-fg",
              )}
            >
              <Led on={active} />
              <span className="flex-1 text-left">{label}</span>
              {planned && (
                <span
                  className={cn("text-[11px] font-medium", active ? "text-accent-fg/70" : "text-subtle")}
                  title={`Planned for ${planned}`}
                >
                  Soon
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <button
        type="button"
        onClick={() => navigate("profiles")}
        className="mt-3 rounded-2xl bg-raised p-3 text-left shadow-module transition-colors hover:bg-hover"
      >
        <span className="flex items-center gap-2 text-xs text-subtle">
          <Led on className="text-accent" /> Active profile
        </span>
        <span className="mt-1 block truncate text-sm font-bold">{activeProfile?.name ?? "Loading…"}</span>
      </button>
      <p className="px-3 pt-3 pb-1 text-[11.5px] text-subtle">
        {version ? `Version ${version}` : " "}
      </p>
    </aside>
  );
}
