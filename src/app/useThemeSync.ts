import { useEffect } from "react";

import { useSettings } from "@/stores/settings";

/** Reflect the theme, accent and motion settings onto <html> attributes. */
export function useThemeSync() {
  const { theme, accent, reduceMotion } = useSettings((s) => s.settings);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.accent = accent;
    root.dataset.reduceMotion = String(reduceMotion);

    if (theme !== "system") {
      root.dataset.theme = theme;
      return;
    }
    const query = window.matchMedia("(prefers-color-scheme: light)");
    const apply = () => {
      root.dataset.theme = query.matches ? "light" : "dark";
    };
    apply();
    query.addEventListener("change", apply);
    return () => query.removeEventListener("change", apply);
  }, [theme, accent, reduceMotion]);
}
