// Label and help ids for the settings registry keys (worker f01). Generated from
// the same list as locales/en/admin-settings.ftl and docs/admin/settings.md; a key
// the console does not know yet is shown under its own name.

/** Fluent id suffix of a registry key: `ui.language` -> `ui-language`. */
export function keySlug(key: string): string {
  return key
    .replace('*', 'family')
    .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
    .replace(/[._]/g, '-')
    .toLowerCase();
}

/** The search anchor id of a key's field in the Settings section. */
export function settingAnchorId(key: string): string {
  return `settings:${key}`;
}

/** Every key with a label and help sentence in admin-settings.ftl and an anchor in docs/admin/settings.md. */
export const KNOWN_KEYS: readonly string[] = [
  'ui.language',
  'appearance.theme',
  'appearance.accent',
  'appearance.density',
  'appearance.brandName',
  'login.notice',
  'mail.readReceipts',
  'notifications.enabled',
  'offline.retentionDays',
  'offline.budgetMb',
  'assistant.enabled',
  'upstream.default',
  'upstream.proxyTimeoutSecs',
  'signin.newUsers',
  'signin.domainMode',
  'signin.refusalDetail',
  'signin.ldapBindPrecheck',
  'twofa.pendingTtlSecs',
  'twofa.maxAttempts',
  'admin.session.idleSecs',
  'admin.session.absoluteSecs',
  'passwd.policy.minLength',
  'passwd.policy.requireUpper',
  'passwd.policy.requireLower',
  'passwd.policy.requireDigit',
  'passwd.policy.requireSymbol',
  'limit.storage_bytes',
  'limit.send_per_day',
  'limit.ai_calls_per_day',
  'quota.warnThresholds',
  'quota.warnEnabled',
  'usage.retentionDays',
  'usage.meterApi',
  'usage.measureIntervalHours',
  'assistant.rateLimitPerMin',
  'plugins.kv.maxKeyBytes',
  'plugins.kv.maxValueBytes',
  'plugins.kv.maxTotalBytes',
  'plugins.kv.maxKeys',
  'import.maxMessages',
  'import.maxMessageBytes',
  'webhooks.maxAttempts',
  'webhooks.timeoutSecs',
  'outbox.maxAttempts',
  'realtime.recheckSecs',
  'privacy.watermark.enabled',
  'privacy.watermark.opacity',
  'privacy.openTracking.retentionDays',
  'privacy.openTracking.unboundTtlHours',
  'maskedEmail.domain',
  'abuse.reportAddress',
  'dlp.adminAddress',
  'observability.errorForwardUrl',
  'egress.probeUrl',
  'push.vapidContact',
  'push.quietHours',
  'updates.check',
  'updates.feedUrl',
  'updates.apply',
  'backup.retentionCount',
  'about.securityContact',
  'about.securityPolicyUrl',
  'about.projectLinks',
  'config.snapshotsKept',
  'jobs.historyDays',
  'maintenance.announceMinutesDefault',
  'maintenance.defaultMessage',
  'health.diskFreeMinPercent',
  'observability.otlpMetrics',
  'metrics.drilldownTopN',
  'metrics.rollupDays',
  'diagnostics.retentionDays',
  'dataExport.linkHours',
  'serverText.*',
];

/** The registry categories, in the order the list shows them. */
export const CATEGORIES: readonly string[] = [
  'interface',
  'appearance',
  'mail',
  'notifications',
  'offline',
  'assistant',
  'upstream',
  'signin',
  'admin',
  'passwords',
  'limits',
  'plugins',
  'import',
  'webhooks',
  'realtime',
  'privacy',
  'observability',
  'egress',
  'updates',
  'backups',
  'about',
  'config',
  'jobs',
  'maintenance',
  'health',
  'diagnostics',
  'users',
  'serverText',
];
