// Mirrors crates/periphos-core/src/apps.rs.

export interface AppIdentity {
  exeName: string;
  exePath: string | null;
  /** Microsoft Store / Game Pass Package Family Name. */
  packageFamily: string | null;
  title: string;
}

/** Microsoft Store / Game Pass package installed for the current user. */
export interface InstalledPackage {
  packageFamily: string;
  displayName: string;
  installPath: string | null;
  /** PC game (its package carries a MicrosoftGame.config). */
  isGame: boolean;
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
