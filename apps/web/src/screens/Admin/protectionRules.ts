// The adaptive protection rules as the Security centre presents them (t97):
// grouped, each with a plain-language label, an explanation and an example
// (`admin-protect-rule-<slug>`, `-help`, `-example` in admin-security.ftl).
// The values, bounds and defaults come from the server's settings registry.

import type { AdminSetting } from '../../api/admin/settings.ts';
import { formatDurationMs } from '../../i18n/format.ts';
import type { PresetName } from '../../api/admin/requestProtection.ts';

export const PROTECTION_PREFIX = 'security.protection.';

export const RULE_GROUPS: readonly { id: string; keys: readonly string[] }[] = [
  { id: 'user-agent', keys: ['userAgentMode', 'userAgentCategories', 'userAgentWeakRequests', 'userAgentCrawlerRequests', 'userAgentWindowSecs', 'userAgentMaxAction', 'userAgentScope', 'userAgentAdaptive', 'userAgentPatterns', 'userAgentExceptions'] },
  { id: 'budget', keys: ['rateMode', 'clientRequests', 'accountRequests', 'adminRequests', 'rateWindowSecs'] },
  { id: 'unusual', keys: ['adaptiveMode', 'signals', 'adaptiveClients', 'adaptiveAccounts', 'adaptiveAdmins', 'shortWindowSecs', 'shortAbsoluteAnomalies', 'shortRatioPercent', 'shortMinimumSamples', 'longWindowSecs', 'longAbsoluteAnomalies', 'longRatioPercent', 'longMinimumSamples', 'normalCapPerBucket'] },
  { id: 'actions', keys: ['throttleSecs', 'blockSecs', 'decaySecs'] },
  { id: 'exceptions', keys: ['exemptClients', 'exemptAccounts', 'exemptAdmins', 'storageFailure'] },
];

export const SIGNALS = ['malformed', 'oversized', 'credential-rejected', 'permission-denied', 'body-deadline', 'admin-route-miss'] as const;
export const USER_AGENT_CATEGORIES = ['missing', 'uncommon', 'automation', 'search', 'ai-training', 'ai-search', 'ai-user'] as const;

export const PRESETS: readonly PresetName[] = ['relaxed', 'balanced', 'strict'];

/** `security.protection.shortWindowSecs` → `short-window-secs`. */
export function ruleSlug(key: string): string {
  return key.slice(PROTECTION_PREFIX.length).replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
}
export function fullKey(short: string): string {
  return `${PROTECTION_PREFIX}${short}`;
}
/** The value in force: the stored global value, else the default. */
export function currentValue(rule: AdminSetting): unknown {
  return rule.effective?.value ?? rule.globalValue ?? rule.default;
}
export function sameValue(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
/** Whether a rule's value is a number of seconds (shown as a duration too). */
export function isSeconds(key: string): boolean {
  return /Secs$/.test(key);
}

/** Client-side checks mirroring the registry bounds; the server decides. */
export function validateRule(rule: AdminSetting, value: unknown): string | null {
  const kind = rule.kind;
  if (kind.type === 'int') {
    if (typeof value !== 'number' || !Number.isInteger(value)) return 'whole';
    if (kind.min !== undefined && value < kind.min) return 'min';
    if (kind.max !== undefined && value > kind.max) return 'max';
  }
  if (kind.type === 'enum' && !(kind.values ?? []).includes(String(value))) return 'choice';
  if (kind.type === 'stringList') {
    if (!Array.isArray(value)) return 'list';
    if (kind.maxItems !== undefined && value.length > kind.maxItems) return 'max-items';
  }
  return null;
}

/** The rules a preset would change, with their current and preset values. */
export function presetDiff(rules: readonly AdminSetting[], preset: Record<string, unknown>): { rule: AdminSetting; from: unknown; to: unknown }[] {
  return Object.entries(preset).flatMap(([key, to]) => {
    const rule = rules.find((r) => r.key === key);
    if (!rule) return [];
    const from = currentValue(rule);
    return sameValue(from, to) ? [] : [{ rule, from, to }];
  });
}

/** A rule's seconds with the shared duration formatter (t90): "3 s", "5 min", "24 h". */
export function formatSeconds(secs: number): string {
  return formatDurationMs(secs * 1000);
}
