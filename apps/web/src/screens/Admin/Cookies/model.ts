// Admin Cookies tab: the setting groups, the response shapes of the cookie routes
// and the configuration a preview shows from the current settings.

import type { AdminSetting } from '../../../api/admin/settings.ts';
import type { Category, ConsentConfig, InventoryEntry, Rule, StorageKind } from '../../../cookies/policy.ts';

export const CATEGORY_IDS: readonly Category[] = ['necessary', 'functional', 'analytics', 'marketing'];
export const KIND_IDS: readonly StorageKind[] = ['cookie', 'localStorage', 'sessionStorage', 'indexedDB'];

/** The tab's settings, grouped; every key is also in Settings under its registry label. */
export const COOKIE_GROUPS: readonly { id: string; keys: readonly string[] }[] = [
  { id: 'general', keys: ['cookies.enabled', 'cookies.mode', 'cookies.regionMode', 'cookies.strictRegions', 'cookies.regionHeader', 'cookies.unknownRegion'] },
  { id: 'categories', keys: ['cookies.functional.enabled', 'cookies.functional.default', 'cookies.analytics.enabled', 'cookies.analytics.default', 'cookies.marketing.enabled', 'cookies.marketing.default'] },
  { id: 'display', keys: ['cookies.layout', 'cookies.position', 'cookies.rejectProminent'] },
  { id: 'policy', keys: ['cookies.policyUrl', 'cookies.privacyUrl', 'cookies.policyVersion', 'cookies.lifetimeDays', 'cookies.repromptOnVersion'] },
  { id: 'signals', keys: ['cookies.honourGpc', 'cookies.honourDnt'] },
  { id: 'log', keys: ['cookies.logConsent', 'cookies.logRetentionDays'] },
  { id: 'scan', keys: ['cookies.scanIntervalHours', 'cookies.unclassified'] },
  { id: 'texts', keys: ['cookies.text.title', 'cookies.text.body', 'cookies.label.necessary', 'cookies.description.necessary', 'cookies.label.functional', 'cookies.description.functional', 'cookies.label.analytics', 'cookies.description.analytics', 'cookies.label.marketing', 'cookies.description.marketing'] },
];

export interface StoredRule extends Rule { id: string; note: string; createdAt: number }
export interface InventoryView { categories: Category[]; kinds: StorageKind[]; builtin: InventoryEntry[]; rules: StoredRule[] }
export interface DetectedRow { id: string; kind: StorageKind; name: string; firstSeen: number; lastSeen: number; seenCount: number; status: 'unclassified' | 'ignored' }
export interface ConsentStats {
  from: number;
  to: number;
  total: number;
  truncated: boolean;
  decisions: Record<'acceptAll' | 'rejectAll' | 'custom' | 'notice', number>;
  categories: Record<'functional' | 'analytics' | 'marketing', { accepted: number; total: number }>;
  regions: { region: string; count: number }[];
  signals: Record<'none' | 'gpc' | 'dnt', number>;
  versions: { version: number; count: number }[];
  series: { t: number; decisions: ConsentStats['decisions'] }[];
}

/** The value a key has now: the saved global value, else the built-in default. */
export function valueOf(rows: readonly AdminSetting[], key: string): unknown {
  const row = rows.find((r) => r.key === key);
  return row ? row.effective.value : undefined;
}

/** A configuration for the preview, from the settings as they are saved now. */
export function configFromRows(rows: readonly AdminSetting[], inv: InventoryView | undefined): ConsentConfig {
  const v = (key: string): unknown => valueOf(rows, key);
  const map = (key: string): Record<string, string> => (v(key) && typeof v(key) === 'object' ? (v(key) as Record<string, string>) : {});
  const cats: ConsentConfig['categories'] = CATEGORY_IDS.map((id) => ({
    id,
    enabled: id === 'necessary' ? true : v(`cookies.${id}.enabled`) === true,
    default: id === 'necessary' ? true : v(`cookies.${id}.default`) === true,
    label: map(`cookies.label.${id}`),
    description: map(`cookies.description.${id}`),
  }));
  const str = (key: string): string | null => (typeof v(key) === 'string' ? (v(key) as string) : null);
  return {
    enabled: true,
    mode: (v('cookies.mode') as ConsentConfig['mode']) ?? 'optIn',
    version: Number(v('cookies.policyVersion') ?? 1),
    lifetimeDays: Number(v('cookies.lifetimeDays') ?? 180),
    reprompt: v('cookies.repromptOnVersion') !== false,
    layout: (v('cookies.layout') as ConsentConfig['layout']) ?? 'banner',
    position: (v('cookies.position') as ConsentConfig['position']) ?? 'bottom',
    rejectProminent: v('cookies.rejectProminent') !== false,
    policyUrl: str('cookies.policyUrl'),
    privacyUrl: str('cookies.privacyUrl'),
    honourGpc: v('cookies.honourGpc') === true,
    honourDnt: v('cookies.honourDnt') === true,
    unclassified: (v('cookies.unclassified') as ConsentConfig['unclassified']) ?? 'report',
    text: { title: map('cookies.text.title'), body: map('cookies.text.body') },
    categories: cats,
    inventory: inv?.builtin ?? [],
    rules: inv?.rules ?? [],
  };
}

/** Percent of `part` in `whole` with one decimal, 0 when `whole` is 0. */
export function share(part: number, whole: number): number {
  return whole <= 0 ? 0 : Math.round((part / whole) * 1000) / 10;
}
