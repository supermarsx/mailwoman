// Dashboard fixtures for tests only (never imported by the app). Shapes follow
// t32-admin.md §2.2; the numbers are test inputs, not samples shown to anyone.

import type { Dashboard } from '../../../api/admin/dashboard.ts';

/** An engine-mode server where every field was readable. */
export function engineDashboard(): Dashboard {
  return {
    server: { version: '26.20.0', buildSha: 'abc1234', mode: 'engine', uptimeSecs: 93_780, basePath: '', startedAt: 1_790_000_000 },
    counts: {
      provisionedUsers: 42,
      accountsSignedIn24h: 17,
      accountsSignedIn7d: 31,
      domains: 3,
      domainsWithUpstream: 2,
      domainsDisabled: 1,
      activeSessions: 9,
      activeAdminSessions: 1,
    },
    security: {
      failedLogins24h: { webmail: 5, admin: 2 },
      activeBlocks: 1,
      allowlists: { admin: 'on', webmail: 'off' },
      trustedProxiesConfigured: true,
      forwardedFromUntrustedPeer24h: 0,
    },
    storage: { dbBytes: 5_242_880, walBytes: 1_048_576, searchIndexBytes: 2_097_152, dataVolume: { freeBytes: 10_737_418_240, totalBytes: 53_687_091_200 } },
    health: {
      dbPingMs: 0.4,
      migration: { applied: 33, expected: 33 },
      engine: { registeredAccounts: 17, accountsWithSyncError: 1 },
      outbox: { pending: 2, oldestAgeSecs: 120, failed24h: 0 },
      searchIndex: { documents: 1200, rebuilding: false },
      renderSandbox: 'available',
      pluginRuntime: 'available',
    },
    traffic: {
      windowMins: 60,
      series: [
        { t: 1_790_000_000, requests: 10, errors5xx: 0, p50Ms: 4, p95Ms: 20 },
        { t: 1_790_000_060, requests: 25, errors5xx: 1, p50Ms: 5, p95Ms: 31 },
        { t: 1_790_000_120, requests: 18, errors5xx: 0, p50Ms: 4, p95Ms: 22 },
      ],
    },
    recentAudit: [{ id: 7, ts: 1_790_000_100, actor: 'root', action: 'user.revoke_sessions', target: 'alice@example.org' }],
    blockedAccounts: { total: 0, byDomain: [] },
    warnings: [],
  };
}

/** A proxy-mode server on a platform without free-space data. */
export function proxyDashboard(): Dashboard {
  const d = engineDashboard();
  return {
    ...d,
    server: { ...d.server!, mode: 'proxy' },
    storage: { dbBytes: 1_048_576, walBytes: null, searchIndexBytes: null, dataVolume: null },
    health: { ...d.health!, engine: null, outbox: null, searchIndex: null },
  };
}
