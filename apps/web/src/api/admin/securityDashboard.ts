// The security centre Overview (t97). Server: `crates/mw-server/src/security_dashboard.rs`.
//
//   GET {console}/security/dashboard → SecurityDashboard
//
// Each tile is present only when the signed-in administrator may open its detail
// page; a missing tile means "not for you", never "no data". Source addresses are
// network prefixes (`sourcesMasked`) unless the caller can read the
// authentication logs.

import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export interface FactorsTile {
  adminsWithoutFactor: number;
  staleApiKeys: number;
  environmentAdminLoginEnabled: boolean;
  knownAccounts: number;
  withFactor: number;
  totp: number;
  passkey: number;
  both: number;
  withoutFactorRequired: number;
  requiredGlobally: boolean;
  requiredDomains: number;
}
export interface SigninBucket { t: number; failed: number; blocked: number }
export interface SigninsTile {
  failed24h: number;
  blocked24h: number;
  failed7d: number;
  blocked7d: number;
  hourly: SigninBucket[];
  daily: SigninBucket[];
  topSources: { source: string; country: string | null; count: number }[];
  topCountries: { country: string; count: number }[];
  sourcesMasked: boolean;
  geoipReady: boolean;
}
export interface ProtectionTile {
  protectionEnforced24h: number;
  protectionObserved24h: number;
  activeLocks: number;
  activeAdminLocks: number;
  activeBans: number;
}
export interface AdminsTile {
  activeSessions: number;
  adminsSignedIn: number;
  recentSignins: { username: string; at: number; source: string }[];
  sourcesMasked: boolean;
}
export interface AuditTile { recent: { id: string; at: string; actor: string; action: string; target: string | null }[] }
export type CheckStatus = 'pass' | 'warning' | 'info' | null;
export interface PostureTile {
  checks: number;
  warnings: number;
  https: CheckStatus;
  hsts: CheckStatus;
  secureCookies: CheckStatus;
  csrf: CheckStatus;
  adminFactors: CheckStatus;
  geoipReady: boolean;
  analyticsTag: boolean | null;
  /** Present only with `security.read`. */
  consoleDefaultPath?: boolean;
}
export interface SecurityDashboard {
  generatedAt: number;
  factors?: FactorsTile;
  signins?: SigninsTile;
  protection?: ProtectionTile;
  admins?: AdminsTile;
  audit?: AuditTile;
  posture?: PostureTile;
}

export async function loadSecurityDashboard(): Promise<SecurityDashboard> {
  return (await adminRequest<SecurityDashboard>('/security/dashboard')).data;
}
