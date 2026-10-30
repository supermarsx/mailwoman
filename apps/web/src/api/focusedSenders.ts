// t94: the user's Focused inbox overrides ("Always move to Focused / Other"),
// kept on the server per user (`/api/account/focused-senders`). Requests carry
// the signed-in mailbox and its storage authority, like the other personal
// settings, so a write made for one mailbox cannot land in another.
import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {preferenceAccount} from './accountPreferenceLifecycle.ts';
import {storageAuthorityHeaders} from './storageAuthority.ts';

export type FocusTab = 'focused' | 'other';
export interface FocusedSender {email: string; tab: FocusTab; updatedAt: number}

export class FocusedSendersError extends Error {
  constructor(readonly status: number) {super(`Focused senders request failed (${status})`);}
}

const PATH = '/api/account/focused-senders';

function headers(write: boolean): Headers {
  const out = new Headers();
  const account = preferenceAccount();
  if (account) out.set('X-Mailwoman-Account', account.id);
  if (write) for (const [name, value] of Object.entries(storageAuthorityHeaders())) out.set(name, value);
  else if (account?.storageAuthority) out.set('x-mailwoman-storage-authority', account.storageAuthority);
  if (write) out.set('Content-Type', 'application/json');
  return out;
}

function isTab(value: unknown): value is FocusTab {return value === 'focused' || value === 'other';}

/** Parse the server's list; anything malformed is an error, not a partial list. */
export function parseSenders(value: unknown): FocusedSender[] {
  if (!Array.isArray(value) || value.length > 1000) throw new Error('Invalid focused senders');
  return value.map((row: unknown) => {
    if (!row || typeof row !== 'object') throw new Error('Invalid focused sender');
    const {email, tab, updatedAt} = row as Record<string, unknown>;
    if (typeof email !== 'string' || !isTab(tab) || typeof updatedAt !== 'number') throw new Error('Invalid focused sender');
    return {email, tab, updatedAt};
  });
}

export async function loadFocusedSenders(signal?: AbortSignal): Promise<FocusedSender[]> {
  const response = await apiFetch(withBase(PATH), {credentials: 'same-origin', headers: headers(false), ...(signal ? {signal} : {})});
  if (!response.ok) throw new FocusedSendersError(response.status);
  return parseSenders(((await response.json()) as {senders?: unknown}).senders);
}

/** Set (`tab`) or remove (`null`) the override for one sender. */
export async function saveFocusedSender(email: string, tab: FocusTab | null): Promise<void> {
  const response = await apiFetch(withBase(PATH), {method: 'PUT', credentials: 'same-origin', headers: headers(true), body: JSON.stringify({email, tab})});
  if (!response.ok) throw new FocusedSendersError(response.status);
}

/** The one-time move of this browser's old local overrides; server rows win. */
export async function importFocusedSenders(senders: Record<string, FocusTab>): Promise<FocusedSender[]> {
  const response = await apiFetch(withBase(`${PATH}/import`), {method: 'POST', credentials: 'same-origin', headers: headers(true), body: JSON.stringify({senders})});
  if (!response.ok) throw new FocusedSendersError(response.status);
  return parseSenders(((await response.json()) as {senders?: unknown}).senders);
}
