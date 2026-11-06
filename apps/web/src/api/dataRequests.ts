// Data requests the signed-in person files about their own data (Settings > Privacy & data).
// `apiFetch` asks for a fresh password when the server answers `step_up_required` and retries once.
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';

export const REQUEST_KINDS = ['access', 'portability', 'rectification', 'erasure', 'restriction', 'objection', 'other'] as const;
/** The short reference a person quotes to us for a request: `R-` and four characters of its id. */
export const requestReference = (id: string): string => `R-${id.replace(/[^0-9a-z]/gi, '').slice(0, 4).toUpperCase()}`;
export type RequestKind = (typeof REQUEST_KINDS)[number];

export type RequestStatus =
  | 'awaiting_confirmation' | 'awaiting_verification' | 'awaiting_approval' | 'queued' | 'processing' | 'scheduled'
  | 'in_progress' | 'blocked' | 'failed' | 'ready' | 'downloaded' | 'expired' | 'completed' | 'rejected' | 'cancelled';

export interface DataRequest {
  id: string; kind: RequestKind; status: RequestStatus;
  createdAt: number; updatedAt: number; dueAt: number;
  details: string; requestedBy: 'self' | 'admin';
  /** An objection's purposes (ids of the consent ledger); empty for other kinds. */
  purposes?: string[];
  needsConfirmation: boolean; canCancel: boolean; graceUntil: number | null;
  decisionReason: string | null; extensionDays: number; extensionReason: string | null;
  export: { bytes: number; expiresAt: number; complete: boolean } | null;
  downloadAvailable: boolean; downloads: number;
  completion: { completedAt: number; retained: { item: string; reason: string }[]; digest: string | null } | null;
  dueState: 'none' | 'ok' | 'dueSoon' | 'overdue';
  confirmationSent?: boolean;
}

export interface TrailEntry { ts: number; kind: string; by: 'you' | 'system' | 'administrator' }

export interface DataRequestPolicy {
  enabled: boolean; types: Record<RequestKind, boolean>;
  reauthRequired: boolean; confirmationRequired: boolean; identityVerificationRequired: boolean;
  approval: { export: string; erasure: string; other: string };
  graceDays: number; slaDays: number; mailAllowed: boolean; mailFormat: string;
  singleUseDownload: boolean; linkMinutes: number; exportHours: number;
  contact: { name: string | null; email: string | null; url: string | null };
  text: { intro: Record<string, string> };
}

export class DataRequestError extends Error {
  constructor(readonly status: number, readonly code: string, message: string) { super(message); }
}

async function call<T>(path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (init.body !== undefined) headers['Content-Type'] = 'application/json';
  const response = await apiFetch(withBase(`/api/privacy/requests${path}`), {
    method: init.method ?? 'GET', credentials: 'same-origin', headers,
    ...(init.body === undefined ? {} : { body: JSON.stringify(init.body) }),
  });
  const body = await response.json().catch(() => ({})) as Record<string, unknown>;
  if (!response.ok) throw new DataRequestError(response.status, typeof body['code'] === 'string' ? body['code'] : 'error', typeof body['error'] === 'string' ? body['error'] : `request failed (${response.status})`);
  return body as T;
}

export const loadOverview = (): Promise<{ policy: DataRequestPolicy }> => call('/overview');
export const listRequests = async (): Promise<DataRequest[]> => (await call<{ requests: DataRequest[] }>('')).requests;
export const loadRequest = (id: string): Promise<{ request: DataRequest; trail: TrailEntry[] }> => call(`/${encodeURIComponent(id)}`);
/** The purposes a person can object to by request; the server drops any other id. */
export const OBJECTION_PURPOSES = ['analytics', 'analyticsTag', 'assistant', 'openTracking', 'push', 'integrations'] as const;
export const submitRequest = async (input: { kind: RequestKind; details?: string; purposes?: string[]; includeMail?: boolean; locale?: string }): Promise<DataRequest> => (await call<{ request: DataRequest }>('', { method: 'POST', body: input })).request;
export const confirmRequest = async (id: string, code: string): Promise<DataRequest> => (await call<{ request: DataRequest }>(`/${encodeURIComponent(id)}/confirm`, { method: 'POST', body: { code } })).request;
export const cancelRequest = async (id: string): Promise<DataRequest> => (await call<{ request: DataRequest }>(`/${encodeURIComponent(id)}/cancel`, { method: 'POST' })).request;
export const downloadLink = (id: string): Promise<{ url: string; expiresAt: number; singleUse: boolean }> => call(`/${encodeURIComponent(id)}/download-link`, { method: 'POST' });
