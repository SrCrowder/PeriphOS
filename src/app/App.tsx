import { useEffect, type ComponentType } from "react";

import { AppShell } from "@/components/layout/AppShell";
import { DashboardPage } from "@/features/dashboard/DashboardPage";
import { DevicesPage } from "@/features/devices/DevicesPage";
import { MacrosPage } from "@/features/planned/MacrosPage";
import { RgbStudioPage } from "@/features/planned/RgbStudioPage";
import { ProfilesPage } from "@/features/profiles/ProfilesPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { useAppInfo } from "@/stores/appInfo";
import { useNavigation, type PageId } from "@/stores/navigation";
import { useProfiles } from "@/stores/profiles";
import { useSettings } from "@/stores/settings";

import { useThemeSync } from "./useThemeSync";

const PAGES: Record<PageId, ComponentType> = {
  dashboard: DashboardPage,
  devices: DevicesPage,
  profiles: ProfilesPage,
  macros: MacrosPage,
  rgb: RgbStudioPage,
  settings: SettingsPage,
};

export function App() {
  const page = useNavigation((s) => s.page);
  useThemeSync();

  useEffect(() => {
    void useSettings.getState().load();
    void useProfiles.getState().load();
    void useAppInfo.getState().load();
  }, []);

  const Page = PAGES[page];
  return (
    <AppShell>
      <div key={page} className="animate-fade-in">
        <Page />
      </div>
    </AppShell>
  );
}
