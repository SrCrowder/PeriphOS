import { Badge } from "./Badge";

/**
 * Marks a feature that is designed but not implemented yet. Every control
 * for a planned feature must be disabled and carry this tag: PeriphOS never
 * shows controls that pretend to work.
 */
export function PlannedTag({ milestone }: { milestone?: string }) {
  return <Badge tone="neutral">{milestone ? `Planned for ${milestone}` : "Planned"}</Badge>;
}
