// `GET /admin/dashboard` client (t32-admin.md §2.2, t32-ext.md §K checklist).
//
// The endpoint is t32-e13's (stage A2). Until it is mounted the server answers
// 404, and the dashboard says the figures are not available yet; nothing here
// invents a number. Every field the server cannot read in its mode or platform is
// `null` on the wire and is shown as "Not available" with the reason.
//
// Same-origin, cookie-authed under the admin session, like `state/slices/admin.ts`.

import { basePath } from '../basePath.ts';
import { adminUrl } from '../adminPath.ts';
import { apiFetch } from '../http.ts';

export interface DashboardServer {
  version: string;
  buildSha: string | null;
  mode: 'engine' | 'proxy';
  uptimeSecs: number;
  basePath: string;
  startedAt: number;
}

export interface DashboardCounts {
  provisionedUsers: number | null;
  accountsSignedIn24h: number | null;
  accountsSignedIn7d: number | null;
  domains: number | null;
  domainsWithUpstream: number | null;
  domainsDisabled: number | null;
  activeSessions: number | null;
  activeAdminSessions: number | null;
}

export interface DashboardSecurity {
  failedLogins24h: { webmail: number; admin: number };
  activeBlocks: number;
  allowlists: { admin: 'off' | 'on'; webmail: 'off' | 'on' };
  trustedProxiesConfigured: boolean;
  forwardedFromUntrustedPeer24h: number;
}

export interface DashboardStorage {
  dbBytes: number | null;
  walBytes: number | null;
  searchIndexBytes: number | null;
  dataVolume: { freeBytes: number; totalBytes: number } | null;
}

/** The Health panel's data (t103): see `./health.ts`. */
export type { DashboardHealth } from './health.ts';
import type { DashboardHealth } from './health.ts';

export interface TrafficPoint {
  /** Bucket start, Unix seconds. */
  t: number;
  requests: number;
  errors5xx: number;
  p50Ms: number;
  p95Ms: number;
}

export interface DashboardTraffic {
  windowMins: number;
  series: TrafficPoint[];
}

export interface DashboardAuditRow {
  id: string | number;
  ts: number;
  actor: string;
  actorLabel?: string;
  actorLink?: { section: 'admins' | 'users'; id: string };
  action: string;
  target: string;
}

export interface BlockedAccounts {
  total: number;
  byDomain: { domain: string; accounts: number; seenHost: string | null }[];
}

export type DashboardWarningCode =
  | 'blocked-accounts'
  | 'no-trusted-proxy-with-ip-rules'
  | 'allowlist-bypass-env'
  | 'metrics-token-unset'
  | 'pending-migrations'
  | 'upstream-cert-expiring';

export interface DashboardWarning {
  /** A string, not only {@link DashboardWarningCode}: a code this build does not
   *  know is shown by its id rather than dropped. */
  code: string;
  count: number;
}

/** First-run checklist ids (t32-ext.md §K). */
export type ChecklistId =
  | 'domain-exists'
  | 'domain-upstream'
  | 'test-connection'
  | 'preferred-domain'
  | 'second-admin'
  | 'trusted-proxies'
  | 'security-policy'
  | 'metrics-token'
  | 'branding-reviewed';

export interface ChecklistItem {
  id: string;
  done: boolean;
  /** Path under the console (`/admin/...`) of the screen that completes it. */
  href: string;
  optional?: boolean;
}

export interface Dashboard {
  server: DashboardServer | null;
  counts: DashboardCounts;
  security: DashboardSecurity | null;
  storage: DashboardStorage | null;
  health: DashboardHealth | null;
  traffic: DashboardTraffic | null;
  recentAudit: DashboardAuditRow[];
  blockedAccounts: BlockedAccounts | null;
  warnings: DashboardWarning[];
  unavailable?: Record<string, string>;
  /** Added by t32-ext.md §K; absent from a server that does not compute it. */
  checklist?: ChecklistItem[];
}

/** What a dashboard load produced. `unavailable` = the server has no such route. */
export type DashboardResult =
  | { kind: 'ok'; data: Dashboard }
  | { kind: 'unavailable' }
  | { kind: 'error'; status: number | null };

export type DashboardLoader = () => Promise<DashboardResult>;

/** The production loader: `GET <base>/admin/dashboard`. */
export function createDashboardLoader(base = basePath()): DashboardLoader {
  return async () => {
    let res: Response;
    try {
      res = await apiFetch(adminUrl('/dashboard', base), { credentials: 'same-origin' });
    } catch {
      return { kind: 'error', status: null };
    }
    if (res.status === 404) return { kind: 'unavailable' };
    if (!res.ok) return { kind: 'error', status: res.status };
    try {
      return { kind: 'ok', data: (await res.json()) as Dashboard };
    } catch {
      return { kind: 'error', status: res.status };
    }
  };
}
