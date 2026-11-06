// What this browser knows about a mailing list: "unsubscribed on <date>" (stored per account and List-Id, never
// sent anywhere) and the senders the user chose to always keep. Both are local conveniences, not server state.
import {createSignal} from 'solid-js';

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
const memory = {unsubscribed: new Map<string, UnsubscribedList>(), keep: new Set<string>()};

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

export function senderKept(address: string): boolean {
  void version();
  const key = address.toLowerCase();
  return read<true>(KEEP_KEY)[key] === true || memory.keep.has(key);
}
export function setSenderKept(address: string, keep: boolean): void {
  const key = address.toLowerCase();
  memory.keep.delete(key);
  const all = read<true>(KEEP_KEY);
  if (keep) all[key] = true; else delete all[key];
  if (!write(KEEP_KEY, all) && keep) memory.keep.add(key);
}
