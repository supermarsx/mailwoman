// Mailbox quota (RFC 9425 `Quota/get`) for the storage meter and the Settings
// detail (t39-e7, L10).
//
// The server answers with the upstream mail server's own quota (IMAP
// GETQUOTAROOT, ids `upstream-*`) and any limit the administrator set
// (`mail-storage` / `mail-count`). In proxy mode the upstream JMAP server
// answers itself. No quota capability, an `unknownMethod`, or an empty list all
// mean the same thing to the UI: there is no quota, so nothing is shown.
//
// Warnings follow the administrator's `quota.warnEnabled` / `quota.warnThresholds`
// settings (percent, default [80, 95]).

import type { Client } from './client.ts';
import { responseFor } from './jmap.ts';
import { isUnsupportedMethod } from './unsupported.ts';
import type { SettingsMap } from './settings.ts';

export const CAP_QUOTA = 'urn:ietf:params:jmap:quota';

export interface Quota {
  id: string;
  resourceType: 'octets' | 'count' | string;
  used: number;
  hardLimit: number;
  name?: string;
  description?: string | null;
}

/** The account's quotas; `[]` when the server reports none. */
export async function loadQuotas(client: Client, accountId: string): Promise<Quota[]> {
  const session = await client.session();
  if (!session.capabilities[CAP_QUOTA] || session.accounts[accountId]?.accountCapabilities[CAP_QUOTA] === undefined) {
    return [];
  }
  try {
    const response = responseFor<{ list?: Quota[] }>(
      await client.jmap({
        using: ['urn:ietf:params:jmap:core', CAP_QUOTA],
        methodCalls: [['Quota/get', { accountId, ids: null }, 'q']],
      }),
      'q',
    );
    return (response.list ?? []).filter(isUsable);
  } catch (error) {
    if (isUnsupportedMethod(error)) return [];
    throw error;
  }
}

function isUsable(q: Quota): boolean {
  return Number.isFinite(q.used) && Number.isFinite(q.hardLimit) && q.hardLimit > 0 && q.used >= 0;
}

/** How full a quota is, 0..1 (can exceed 1 when the server lets usage overrun). */
export function ratio(q: Quota): number {
  return q.used / q.hardLimit;
}

/** The fullest storage (octets) quota, which is the one the meter shows. */
export function storageQuota(list: readonly Quota[]): Quota | null {
  let best: Quota | null = null;
  for (const q of list) {
    if (q.resourceType !== 'octets') continue;
    if (best === null || ratio(q) > ratio(best)) best = q;
  }
  return best;
}

export interface WarnPolicy {
  enabled: boolean;
  /** Ascending percentages, 1..100. */
  thresholds: number[];
}

const DEFAULT_THRESHOLDS = [80, 95];

/** The administrator's warning policy, from the effective settings map. */
export function warnPolicy(settings: SettingsMap): WarnPolicy {
  const enabled = settings['quota.warnEnabled']?.value;
  const raw = settings['quota.warnThresholds']?.value;
  const thresholds = Array.isArray(raw)
    ? raw.filter((v): v is number => typeof v === 'number' && v >= 1 && v <= 100)
    : DEFAULT_THRESHOLDS;
  return { enabled: enabled !== false, thresholds: [...new Set(thresholds)].sort((a, b) => a - b) };
}

/**
 * The warning a quota has reached: `null` below the first threshold (or with
 * warnings off), `'warn'` past any threshold, `'critical'` past the highest one
 * when there are two or more. `threshold` is the highest percentage crossed.
 */
export function warnLevel(q: Quota, policy: WarnPolicy): { level: 'warn' | 'critical'; threshold: number } | null {
  if (!policy.enabled || policy.thresholds.length === 0) return null;
  const percent = ratio(q) * 100;
  const crossed = policy.thresholds.filter((t) => percent >= t);
  const threshold = crossed.at(-1);
  if (threshold === undefined) return null;
  const top = policy.thresholds.at(-1)!;
  return { level: policy.thresholds.length > 1 && threshold === top ? 'critical' : 'warn', threshold };
}
