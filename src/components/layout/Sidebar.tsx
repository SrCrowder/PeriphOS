import { NAV_ITEMS } from "@/app/navigation";
import { cn } from "@/lib/cn";
import { useAppInfo } from "@/stores/appInfo";
import { useNavigation } from "@/stores/navigation";
import { useUpdates } from "@/stores/updates";

import { Led } from "./Led";
import { Logo } from "./Logo";
import { ProfileSwitcher } from "./ProfileSwitcher";

export function Sidebar() {
  const page = useNavigation((s) => s.page);
  const navigate = useNavigation((s) => s.navigate);
  const version = useAppInfo((s) => s.info?.version);
  const availableVersion = useUpdates((s) => (s.phase === "available" ? s.result?.version : null));

  return (
    <aside className="flex w-56 shrink-0 flex-col rounded-module bg-sidebar p-3 shadow-module">
      <div className="px-3 pt-3 pb-6">
        <Logo />
      </div>

      <nav aria-label="Main" className="flex flex-1 flex-col gap-1">
        {NAV_ITEMS.map(({ id, label, planned }) => {
          const active = page === id || (page === "device" && id === "devices");
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

      {availableVersion && (
        <button
          type="button"
          onClick={() => navigate("settings")}
          className="mt-3 flex items-center gap-2 rounded-2xl border-2 border-accent px-3 py-2.5 text-left text-[13px] font-semibold text-accent transition-colors hover:bg-accent hover:text-accent-fg"
        >
          <Led on className="animate-pulse" />
          Update {availableVersion} available
        </button>
      )}

      <ProfileSwitcher />
      <p className="px-3 pt-3 pb-1 text-[11.5px] text-subtle">
        {version ? `Version ${version}` : " "}
      </p>
    </aside>
  );
}
