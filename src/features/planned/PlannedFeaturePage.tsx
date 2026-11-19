import type { ReactNode } from "react";

import { PageHeader } from "@/components/layout/PageHeader";
import { Panel, PlannedTag } from "@/components/ui";

interface PlannedFeaturePageProps {
  title: string;
  milestone: string;
  summary: string;
  /** What the feature will do once built. */
  scope: string[];
  /** Hardware or OS constraints users should know about up-front. */
  limitations: ReactNode;
}

/**
 * Placeholder for a section that is designed but not built. It describes the
 * plan honestly instead of rendering controls that do nothing.
 */
export function PlannedFeaturePage({ title, milestone, summary, scope, limitations }: PlannedFeaturePageProps) {
  return (
    <>
      <PageHeader title={title} description={summary} actions={<PlannedTag milestone={milestone} />} />
      <div className="grid max-w-4xl gap-4 md:grid-cols-2">
        <Panel title="What it will do">
          <ul className="flex list-disc flex-col gap-2 pl-4 text-[13px] text-muted marker:text-subtle">
            {scope.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </Panel>
        <Panel title="Good to know">
          <div className="text-[13px] text-muted">{limitations}</div>
        </Panel>
      </div>
    </>
  );
}
