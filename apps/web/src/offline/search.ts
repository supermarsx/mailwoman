// Reduced offline search (plan §1.1, §2.5): a client-side field / substring
// filter over the cached header slice. This is deliberately LIMITED — the full
// Tantivy operator search is engine-side and online-only. The UI labels results
// "limited (offline)"; e6/e7 own that label, this module owns the filter.

import type { Email, EmailAddress } from '../api/jmap-types.ts';

/** The offline-supported subset of the frozen `Email/query` filter (§2.1). */
export interface OfflineQuery {
  /** Free text — matches from / to / subject / preview. */
  text?: string;
  from?: string;
  to?: string;
  subject?: string;
  /** Offline `body` is best-effort: the cached preview only (full bodies may not
   *  be cached). */
  body?: string;
  hasKeyword?: string;
  notKeyword?: string;
  hasAttachment?: boolean;
}

function addressText(addrs: EmailAddress[] | null | undefined): string {
  if (!addrs) return '';
  return addrs.map((a) => `${a.name ?? ''} ${a.email}`).join(' ');
}

function includesCI(haystack: string, needle: string): boolean {
  return haystack.toLowerCase().includes(needle.toLowerCase());
}

/** Does one cached header match the offline query? All present fields must match. */
export function matchesOffline(email: Email, q: OfflineQuery): boolean {
  if (q.text !== undefined) {
    const blob = [
      addressText(email.from),
      addressText(email.to),
      email.subject ?? '',
      email.preview,
    ].join(' ');
    if (!includesCI(blob, q.text)) return false;
  }
  if (q.from !== undefined && !includesCI(addressText(email.from), q.from)) return false;
  if (q.to !== undefined && !includesCI(addressText(email.to), q.to)) return false;
  if (q.subject !== undefined && !includesCI(email.subject ?? '', q.subject)) return false;
  if (q.body !== undefined && !includesCI(email.preview, q.body)) return false;
  if (q.hasKeyword !== undefined && email.keywords?.[q.hasKeyword] !== true) return false;
  if (q.notKeyword !== undefined && email.keywords?.[q.notKeyword] === true) return false;
  if (q.hasAttachment !== undefined && (email.hasAttachment ?? false) !== q.hasAttachment) return false;
  return true;
}

/** Filter the cached header slice by the offline query (order preserved). */
export function offlineSearch(headers: readonly Email[], q: OfflineQuery): Email[] {
  return headers.filter((email) => matchesOffline(email, q));
}
