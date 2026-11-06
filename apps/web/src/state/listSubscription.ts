// What this browser knows about a mailing list: "unsubscribed on <date>" (stored per account and List-Id, never
// sent anywhere) and the senders the user chose to always keep. Both are local conveniences, not server state.
import {createSignal} from 'solid-js';
import {settings, writeSetting} from '../api/settings.ts';

export type UnsubscribeMethod = 'one-click' | 'link' | 'mail';
export interface UnsubscribedList { at: number; method: UnsubscribeMethod }
const UNSUBSCRIBED_KEY = 'mw.list.unsubscribed';
const KEEP_KEY = 'mw.list.keep';
/** Lists often need a few days to process a request; later mail than this means the request is being ignored. */
export const UNSUBSCRIBE_GRACE_MS = 3 * 24 * 60 * 60 * 1000;
const MAX_ENTRIES = 500;

const [version, setVersion] = createSignal(0);

function read<T>(key: string): Record<string, T> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(key) ?? '{}');
    return parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed as Record<string, T> : {};
  } catch { return {}; }
}
/** Whether the browser kept it; when storage is unavailable the caller holds the value in memory until the page reloads. */
function write(key: string, value: Record<string, unknown>): boolean {
  let stored = true;
  try {
    const entries = Object.entries(value);
    localStorage.setItem(key, JSON.stringify(entries.length > MAX_ENTRIES ? Object.fromEntries(entries.slice(-MAX_ENTRIES)) : value));
  } catch { stored = false; }
  setVersion(v => v + 1);
  return stored;
}
const memory = {unsubscribed: new Map<string, UnsubscribedList>(), keep: new Set<string>(), pending: new Map<string, boolean>()};

export const listKey = (accountId: string | null, listId: string): string => `${accountId ?? ''}|${listId.toLowerCase()}`;

export function unsubscribedList(key: string): UnsubscribedList | null {
  void version();
  const stored = read<UnsubscribedList>(UNSUBSCRIBED_KEY)[key] ?? memory.unsubscribed.get(key);
  return stored !== undefined && typeof stored.at === 'number' && Number.isFinite(stored.at) ? stored : null;
}
export function markUnsubscribed(key: string, method: UnsubscribeMethod, now: number = Date.now()): void {
  const entry = {at: now, method};
  if (!write(UNSUBSCRIBED_KEY, {...read<UnsubscribedList>(UNSUBSCRIBED_KEY), [key]: entry})) memory.unsubscribed.set(key, entry);
}
export function clearUnsubscribed(key: string): void {
  memory.unsubscribed.delete(key);
  const all = read<UnsubscribedList>(UNSUBSCRIBED_KEY); delete all[key];
  write(UNSUBSCRIBED_KEY, all);
}
/** Mail received well after the unsubscribe: the list is ignoring the request. */
export function stillReceiving(entry: UnsubscribedList | null, latestReceivedAt: string | null | undefined): boolean {
  if (entry === null || !latestReceivedAt) return false;
  const received = Date.parse(latestReceivedAt);
  return Number.isFinite(received) && received > entry.at + UNSUBSCRIBE_GRACE_MS;
}

/** The server-side keep list (`mail.list.keep`): `sender:<address>` and `list:<List-Id>` entries. */
export const KEEP_SETTING = 'mail.list.keep';
export function keepEntries(): string[] {
  const value = settings()[KEEP_SETTING]?.value;
  return Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : [];
}
const senderEntry = (address: string): string => `sender:${address.trim().toLowerCase()}`;
const listEntry = (listId: string): string => `list:${listId.trim().replace(/^<|>$/g, '').toLowerCase()}`;

/** Whether the user keeps this sender, or the list the message came from. Browsers that kept the choice locally before it was stored on the server still count. */
export function senderKept(address: string, listId?: string | null): boolean {
  void version();
  const key = address.toLowerCase();
  // A choice being saved shows at once.
  const pending = memory.pending.get(key);
  if (pending !== undefined) return pending;
  const entries = keepEntries();
  return entries.includes(senderEntry(address)) || (!!listId && entries.includes(listEntry(listId)))
    || read<true>(KEEP_KEY)[key] === true || memory.keep.has(key);
}
/** Keep or stop keeping a sender (and, with `listId`, the list). Saved in the user's settings; the old local entry is dropped. */
export async function setSenderKept(address: string, keep: boolean, listId?: string | null): Promise<boolean> {
  const key = address.toLowerCase();
  memory.keep.delete(key);
  memory.pending.set(key, keep);
  setVersion(v => v + 1);
  const local = read<true>(KEEP_KEY);
  if (key in local) { delete local[key]; write(KEEP_KEY, local); }
  const wanted = [senderEntry(address), ...(listId ? [listEntry(listId)] : [])];
  const current = keepEntries();
  const next = keep ? [...current, ...wanted.filter(w => !current.includes(w))] : current.filter(c => !wanted.includes(c));
  try {
    await writeSetting(KEEP_SETTING, next);
    memory.pending.delete(key);
    setVersion(v => v + 1);
    return true;
  } catch {
    // Not saved (offline, or locked by an administrator): a kept sender stays kept on this page only; a failed un-keep reverts.
    memory.pending.delete(key);
    if (keep) memory.keep.add(key);
    setVersion(v => v + 1);
    return false;
  }
}
