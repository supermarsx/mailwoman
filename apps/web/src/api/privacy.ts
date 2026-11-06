// Settings > Privacy & data > Your data: the overview of what is stored about the
// signed-in person, their consents and their restriction status
// (`crates/mw-server/src/privacy_overview.rs`, `consent_ledger.rs`, `restriction.rs`).
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';

export type ConsentState = 'granted' | 'withdrawn' | 'unset' | 'stale';
export type LegalBasis = 'consent' | 'legitimateInterest' | 'contract' | 'legalObligation';

export interface ConsentPurpose {
  id: string;
  basis: LegalBasis;
  version: number;
  /** The administrator's own text in the person's language, when set. */
  text: string | null;
  state: ConsentState;
  enforcement: 'ledger' | 'preference' | 'listed';
  managedIn: string;
  decidedAt: number | null;
  decidedVersion: number | null;
  canToggle: boolean;
  defaultOn: boolean;
}

export interface AssistantFeature { id: string; feature: string; enabled: boolean; decidedAt: number | null }
export interface ConsentReceipt { id: string; purpose: string; decision: string; version: number; basis: string; source: string; createdAt: number }

export interface ConsentView {
  enabled: boolean;
  art22: { enabled: boolean; humanReview: boolean };
  purposes: ConsentPurpose[];
  assistantFeatures: AssistantFeature[];
  receipts: ConsentReceipt[];
}

export interface RestrictionStatus {
  active: boolean;
  scopes: string[];
  since: number | null;
  expiresAt: number | null;
  mailDelivery?: string;
  source?: string;
}

export interface DataCategory {
  id: string;
  rows: number;
  bytes: number;
  retention: { days: number; source: 'setting' | 'userRule' } | null;
  processors: string[];
  purpose: string | null;
}

export interface PrivacyOverview {
  account: { address: string; domain: string };
  categories: DataCategory[];
  consents: ConsentView;
  restriction: RestrictionStatus;
  links: { policy: string; processing: string };
  shortcuts: { download: string; delete: string };
}

export class PrivacyError extends Error {
  constructor(readonly status: number, readonly code: string) { super(code); }
}

async function call<T>(path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (init.body !== undefined) headers['Content-Type'] = 'application/json';
  const response = await apiFetch(withBase(path), {
    method: init.method ?? 'GET', credentials: 'same-origin', headers,
    ...(init.body === undefined ? {} : { body: JSON.stringify(init.body) }),
  });
  const body = await response.json().catch(() => ({})) as Record<string, unknown>;
  if (!response.ok) throw new PrivacyError(response.status, typeof body['code'] === 'string' ? body['code'] : 'error');
  return body as T;
}

export const loadOverview = (): Promise<PrivacyOverview> => call('/api/privacy/overview');
export const loadRestriction = (): Promise<RestrictionStatus> => call('/api/privacy/restriction');
export const decideConsent = (purpose: string, decision: 'granted' | 'withdrawn'): Promise<ConsentView> =>
  call(`/api/privacy/consents/${encodeURIComponent(purpose)}`, { method: 'POST', body: { decision } });

export interface SessionRow { handle: string; current: boolean; createdAt?: string; lastSeen?: string; deviceLabel?: string | null; userAgent?: string | null; country?: string | null }
export interface KeyRow { prefix: string; label?: string; name?: string; createdAt?: string; lastUsedAt?: string | null; revokedAt?: string | null }

export async function listSessions(): Promise<SessionRow[]> {
  return (await call<{ sessions: SessionRow[] }>('/api/account/sessions')).sessions;
}
export async function revokeSession(handle: string): Promise<void> {
  await call('/api/account/sessions/revoke', { method: 'POST', body: { handle } });
}
export async function listKeys(): Promise<KeyRow[]> {
  const body = await call<{ keys?: KeyRow[] } | KeyRow[]>('/api/keys');
  return Array.isArray(body) ? body : (body.keys ?? []);
}
export async function revokeKey(prefix: string): Promise<void> {
  await call(`/api/keys/${encodeURIComponent(prefix)}/revoke`, { method: 'POST', body: {} });
}
