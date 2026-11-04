// Session security notices (country change, impossible travel, concurrent use,
// client mismatch, rotated-session reuse). Rides
// `GET/POST /api/account/security/session-alerts[/ack]`.

import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';

export type SessionAlertKind = 'countryChange' | 'impossibleTravel' | 'concurrentUse' | 'fingerprintMismatch' | 'rotationReuse';
export type SessionAlertAction = 'alerted' | 'step-up' | 'revoked';

export interface SessionNotice {
  id: string;
  ts: number;
  kind: SessionAlertKind;
  reason?: string;
  fromCountry?: string | null;
  toCountry?: string | null;
  action: SessionAlertAction;
  session?: string;
  surface?: string;
  acknowledged?: boolean;
  caveat?: string | undefined;
}

export interface SessionAlertsResponse {
  notices: SessionNotice[];
  unread: number;
  currentSession?: string | undefined;
  caveat?: string | undefined;
}

const KINDS: readonly string[] = ['countryChange', 'impossibleTravel', 'concurrentUse', 'fingerprintMismatch', 'rotationReuse'];
const ACTIONS: readonly string[] = ['alerted', 'step-up', 'revoked'];

/** Kinds whose evidence is an IP-derived country: they carry the accuracy caveat. */
export const isLocationKind = (kind: string): boolean => kind === 'countryChange' || kind === 'impossibleTravel' || kind === 'concurrentUse';

export const knownKind = (kind: string): SessionAlertKind => (KINDS.includes(kind) ? (kind as SessionAlertKind) : 'fingerprintMismatch');
export const knownAction = (action: string): SessionAlertAction => (ACTIONS.includes(action) ? (action as SessionAlertAction) : 'alerted');

/** Server timestamps are unix seconds; tolerate milliseconds. */
export const noticeDate = (ts: number): Date => new Date(ts < 1e12 ? ts * 1000 : ts);

const url = (suffix = ''): string => `${basePath()}/api/account/security/session-alerts${suffix}`;

export async function fetchSessionAlerts(): Promise<SessionAlertsResponse> {
  const response = await apiFetch(url(), { credentials: 'same-origin' });
  if (!response.ok) throw new Error(`session-alerts ${response.status}`);
  const body = (await response.json()) as Partial<SessionAlertsResponse>;
  return {
    notices: Array.isArray(body.notices) ? body.notices : [],
    unread: typeof body.unread === 'number' ? body.unread : 0,
    currentSession: body.currentSession,
    caveat: body.caveat,
  };
}

export async function acknowledgeSessionAlerts(target: { id: string } | { all: true }): Promise<void> {
  const response = await apiFetch(url('/ack'), {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(target),
  });
  if (!response.ok) throw new Error(`session-alerts ack ${response.status}`);
}
