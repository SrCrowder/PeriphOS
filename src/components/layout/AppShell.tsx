import type { ReactNode } from "react";

import { PreviewBanner } from "./PreviewBanner";
import { Sidebar } from "./Sidebar";
import { Toaster } from "./Toaster";

/** Graphite "chassis" holding the sidebar module and the scrolling content. */
export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-full flex-col">
      <PreviewBanner />
      <div className="flex min-h-0 flex-1 gap-4 p-4">
        <Sidebar />
        <main className="min-w-0 flex-1 overflow-y-auto rounded-module">
          <div className="mx-auto w-full max-w-[1120px] px-4 pb-12">{children}</div>
        </main>
      </div>
      <Toaster />
    </div>
  );
}
