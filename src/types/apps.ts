// Mirrors crates/periphos-core/src/apps.rs.

export interface AppIdentity {
  exeName: string;
  exePath: string | null;
  /** Microsoft Store / Game Pass Package Family Name. */
  packageFamily: string | null;
  title: string;
}

export type AppRuleKind = "executable" | "package";

export interface AppRule {
  id: string;
  profileId: string;
  kind: AppRuleKind;
  value: string;
  displayName: string;
  createdAt: number;
}
