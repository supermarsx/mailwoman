// Per-image-host rules for remote images: one state per host, kept in two user
// lists (`images.trustedHosts`, `images.blockedHosts`). A host is matched by its
// own name or any parent domain in a list (`example.com` covers
// `cdn.example.com`, never `evilexample.com`). Blocked wins over trusted.
// The server applies the same rules (crates/mw-server remote_policy); the reader
// uses this copy to decide what to sign and to show the current state.
import { normalizeTrustedLinkDomain } from './linkTrust.ts';

export type ImageHostState = 'allow' | 'ask' | 'block';
export interface ImageHostLists { trusted: readonly string[]; blocked: readonly string[] }
export const IMAGE_HOST_LIST_MAX = 100;
type Settings = Readonly<Record<string, { value?: unknown } | undefined>>;

function list(values: Settings, key: string): string[] {
  const raw = values[key]?.value;
  if (!Array.isArray(raw)) return [];
  return [...new Set(raw.filter((v): v is string => typeof v === 'string' && normalizeTrustedLinkDomain(v) === v))].slice(0, IMAGE_HOST_LIST_MAX);
}
export function imageHostLists(values: Settings): ImageHostLists {
  return { trusted: list(values, 'images.trustedHosts'), blocked: list(values, 'images.blockedHosts') };
}
/** The lowercase hostname of an http(s) URL, or `null`. */
export function imageHostOf(url: string): string | null {
  try { const u = new URL(url); return u.protocol === 'https:' || u.protocol === 'http:' ? u.hostname.toLowerCase().replace(/\.$/, '') : null; } catch { return null; }
}
const covers = (entry: string, host: string): boolean => host === entry || host.endsWith('.' + entry);
const coveringEntries = (entries: readonly string[], host: string): string[] => entries.filter(entry => covers(entry, host));
export function imageHostState(host: string, lists: ImageHostLists): ImageHostState {
  if (coveringEntries(lists.blocked, host).length) return 'block';
  return coveringEntries(lists.trusted, host).length ? 'allow' : 'ask';
}
/** The lists after choosing `state` for `host`: entries covering the host are
 * dropped from both lists, then the host itself is added to the chosen one.
 * `null` when the lists are full or the host cannot be listed. */
export function setImageHostState(host: string, state: ImageHostState, lists: ImageHostLists): ImageHostLists | null {
  const entry = normalizeTrustedLinkDomain(host);
  if (entry === null) return null;
  const trusted = lists.trusted.filter(e => !covers(e, host));
  const blocked = lists.blocked.filter(e => !covers(e, host));
  if (state === 'allow') trusted.push(entry);
  if (state === 'block') blocked.push(entry);
  return trusted.length > IMAGE_HOST_LIST_MAX || blocked.length > IMAGE_HOST_LIST_MAX ? null : { trusted, blocked };
}
