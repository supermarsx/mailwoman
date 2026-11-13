// Session security notices (country change, impossible travel, concurrent use,
// client mismatch, rotated-session reuse). Rides
// `GET/POST /api/account/security/session-alerts[/ack]`.

import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {storageAuthorityHeaders} from '../../api/storageAuthority.ts';

export type SessionAlertKind = 'countryChange' | 'countryUnavailable' | 'impossibleTravel' | 'concurrentUse' | 'fingerprintMismatch' | 'rotationReuse' | 'unknown';
export type SessionAlertAction = 'alerted' | 'step-up' | 'revoked';
export interface SessionNoticeContext {ip?:string|null;country?:string|null;network?:string;deviceType?:string;browser?:string|null;browserMajor?:number|null;browserEngine?:string|null;os?:string|null;clientType?:string;clientVersion?:string|null;userAgent?:string|null;recognized?:boolean|null;fingerprint?:string|null;authentication?:string|null}

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
  context?:SessionNoticeContext|null;
}

export interface SessionAlertsResponse {
  notices: SessionNotice[];
  unread: number;
  currentSession?: string | undefined;
  caveat?: string | undefined;
}

const KINDS: readonly string[] = ['countryChange', 'countryUnavailable', 'impossibleTravel', 'concurrentUse', 'fingerprintMismatch', 'rotationReuse'];
const ACTIONS: readonly string[] = ['alerted', 'step-up', 'revoked'];

/** Kinds whose evidence is an IP-derived country: they carry the accuracy caveat. */
export const isLocationKind = (kind: string): boolean => kind === 'countryChange' || kind === 'impossibleTravel' || kind === 'concurrentUse';

export const knownKind = (kind: string): SessionAlertKind => (KINDS.includes(kind) ? (kind as SessionAlertKind) : 'unknown');
export const knownAction = (action: string): SessionAlertAction => (ACTIONS.includes(action) ? (action as SessionAlertAction) : 'alerted');

/** Server timestamps are unix seconds; tolerate milliseconds. */
export const noticeDate = (ts: number): Date => new Date(ts < 1e12 ? ts * 1000 : ts);

const url = (suffix = ''): string => `${basePath()}/api/account/security/session-alerts${suffix}`;
async function scopedFetch(input:string,init:RequestInit={}):Promise<Response>{
 const owner=preferenceAccount(),controller=new AbortController(),headers=new Headers(init.headers);
 if(owner)headers.set('X-Mailwoman-Account',owner.id);
 for(const [key,value] of Object.entries(storageAuthorityHeaders(owner?.storageAuthority)))headers.set(key,value);
 const off=observePreferenceAccount(next=>{if(next!==owner)controller.abort();});
 try{const response=await apiFetch(input,{...init,headers,signal:controller.signal});if(controller.signal.aborted||owner&&!preferenceAccountIsCurrent(owner))throw new DOMException('Account changed','AbortError');return response;}finally{off();}
}

export async function fetchSessionAlerts(): Promise<SessionAlertsResponse> {
  const response = await scopedFetch(url(), { credentials: 'same-origin' });
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
  const response = await scopedFetch(url('/ack'), {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(target),
  });
  if (!response.ok) throw new Error(`session-alerts ack ${response.status}`);
}
