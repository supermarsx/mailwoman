// Dashboard fixtures for tests only (never imported by the app). Shapes follow
// t32-admin.md §2.2; the numbers are test inputs, not samples shown to anyone.

import type { Dashboard } from '../../../api/admin/dashboard.ts';
import type { DashboardHealth, HealthComponent } from '../../../api/admin/health.ts';

const ENABLE = 'Set MW_MODE=engine and restart.';

/** Every component measured, as an engine-mode server reports it (t103). */
export function engineHealth(): DashboardHealth {
  const c = (status: HealthComponent['status'], reason: string, metrics: Record<string, unknown>, thresholds: Record<string, number> = {}): HealthComponent => ({ status, reason, metrics, thresholds, probeUs: 420 });
  return {
    dbPingMs: 0.4,
    migration: { applied: 72, expected: 72 },
    checkedAt: 1_790_000_200,
    ageSecs: 0,
    ttlSecs: 5,
    historyStepSecs: 30,
    history: [
      { t: 1_790_000_140, dbPingMs: 0.3, renderPingMs: 12, syncErrorsLastHour: 0, outboxQueued: 1, outboxDeferred: 0, searchPending: 0 },
      { t: 1_790_000_200, dbPingMs: 0.4, renderPingMs: 14, syncErrorsLastHour: 1, outboxQueued: 2, outboxDeferred: 1, searchPending: 3 },
    ],
    components: {
      database: {
        ...c('ok', 'The database answers.', { pingMs: 0.4, backend: 'sqlite', poolSize: 5, poolIdle: 4, poolInUse: 1, dbBytes: 5_242_880, walBytes: 1_048_576, serverVersion: '3.46.0', name: 'mw.db', host: null, port: null, journalMode: 'wal', slowQueriesLastHour: null }, { pingWarnMs: 50, pingFailMs: 500 }),
        notes: { slowQueriesLastHour: 'Not tracked: the store does not time individual queries.' },
      },
      migrations: c('ok', 'Up to date.', { applied: 72, expected: 72, latest: { version: 75, description: 'linked avatars' }, latestExpected: { version: 75, description: 'linked avatars' }, pending: [], failed: [], unknown: [] }),
      sync: c('warn', '1 of 17 mailbox(es) are failing to sync.', {
        accounts: 17, syncing: 1, idle: 15, erroring: 1, notWatching: 0, neverSynced: 2, lastSyncAgeP50Secs: 42, lastSyncAgeMaxSecs: 3_700, queueDepth: 1, errorsLastHour: 3,
        byKind: { imap: { accounts: 15, watchConnections: 15, syncing: 1 }, pop3: { accounts: 2, watchConnections: 2, syncing: 0 } },
        recentErrors: [{ kind: 'imap', error: 'connection refused', ageSecs: 30, consecutiveFailures: 2 }],
      }, { erroringFailRatio: 0.5 }),
      outbox: c('warn', '1 message(s) are waiting to retry; 0 send(s) failed in the last hour.', {
        queued: 2, scheduled: 1, held: 0, deferred: 1, sending: 0, failed: 0, uncertain: 0, oldestDueAgeSecs: 120, limited: false,
        sendsLastHour: { sent: 9, deferred: 1, failed: 0, uncertain: 0 },
        lastSend: { outcome: 'sent', ageSecs: 64, error: null },
        submission: { configured: 1, checked: 1, reachable: 1, unreachable: 0, notCheckable: 0, slowestHandshakeMs: 38.5, credentialsSent: false, ageSecs: 12 },
      }, { oldestDueWarnSecs: 600, oldestDueFailSecs: 3600 }),
      search: c('ok', 'Present and in step with the store.', { present: true, persistent: true, documents: 1200, indexableMessages: 1203, pending: 3, zeroAccessDocuments: 0, bytes: 2_097_152, lastCommitAgeSecs: 8, rebuilding: false, rebuildProgressPct: null, rebuildDone: 1200, rebuildTotal: 1200 }, { pendingWarn: 500 }),
      render: c('ok', 'The worker answers.', { workerConfigured: true, version: '26.19.0', pingMs: 14.2, inFlight: 0, queueDepth: 0, failuresLastHour: 0, timeoutsLastHour: 0, busyRefusalsLastHour: 0, jailRequired: false, jailDegraded: null }, { pingWarnMs: 1000, pingTimeoutMs: 1500 }),
      plugins: c('ok', '1 plugin(s) loaded.', {
        installed: 2, enabled: 1, loaded: 1, notLoaded: 0, hostPingMs: 0.002, trapsLastHour: 0, limitTripsLastHour: 0, runtimeErrorsLastHour: 0,
        plugins: [{ id: 'spam-rspamd', name: 'Rspamd', version: '1.0.0', loaded: true, notLoadedReason: null, restartRequired: false, limits: { memoryMb: 32, deadlineMs: 10_000, fuel: null } }],
      }),
    },
  };
}

/** Proxy mode: sync, outbox, search and plugins are disabled with how to enable them. */
export function proxyHealth(): DashboardHealth {
  const h = engineHealth();
  const off = (reason: string): HealthComponent => ({ status: 'disabled', reason, enable: ENABLE, metrics: {}, thresholds: {}, probeUs: 3 });
  return {
    ...h,
    history: [],
    components: {
      ...h.components,
      sync: off('Proxy mode: this server does not synchronize mailboxes.'),
      outbox: off('Proxy mode: messages are handed to the mail server at once, so there is no outbox here.'),
      search: off('Proxy mode: mail is not stored on this server, so there is no search index; searches go to the mail server.'),
      plugins: off('Proxy mode: engine plugins (account bridges, spam classifiers) run only in engine mode.'),
    },
  };
}

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
    health: engineHealth(),
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
    health: proxyHealth(),
  };
}
