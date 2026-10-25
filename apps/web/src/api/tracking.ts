// Open tracking for a sent message (t29-e12 client of t29-e8's routes).
//
// Server contract — `crates/mw-server/src/open_tracking.rs`, session-authed,
// `Cache-Control: no-store` (writes ride the CSRF header through `apiFetch`):
//
//   POST /api/track/open            { subject, to: string[] } → { token, url }
//                                   409 { error: 'publicUrlRequired' } when the feature is off
//   PUT  /api/track/open/{token}    { emailId } → 204
//   GET  /api/track/open?emailId=   → { count, first, last }   404 when nothing is bound
//
// `GET /api/policy` says whether the feature is on (`openTracking`, read in
// `api/policy.ts`); the composer renders its checkbox only then. The image the
// URL names is added to the HTML body after the editor's schema round trip, so
// it is never part of the editable document. A token is minted for each send
// (an offline send mints its token when the queue replays it, not when it is
// queued), and bound to the created Email once the server has accepted it.

import { withBase } from './basePath.ts';
import { apiFetch } from './http.ts';
import { parseRecipients, type DraftInput } from './jmap.ts';

/** A minted token and the absolute image URL that records an open. */
export interface TrackingToken {
  token: string;
  url: string;
}

/** What the server has recorded for a sent message. */
export interface OpenCount {
  count: number;
  first: string | null;
  last: string | null;
}

/** Thrown when a token could not be minted; the send must not go out untracked. */
export class TrackingUnavailable extends Error {
  constructor(readonly reason: string) {
    super(`open tracking unavailable: ${reason}`);
    this.name = 'TrackingUnavailable';
  }
}

type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const ENDPOINT = '/api/track/open';

const json = (body: unknown): RequestInit => ({
  credentials: 'same-origin',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify(body),
});

/** Mint a token for a message with this subject and these recipient addresses. */
export async function mintTrackingToken(
  subject: string,
  to: string[],
  fetcher: Fetcher = apiFetch,
): Promise<TrackingToken> {
  let res: Response;
  try {
    res = await fetcher(withBase(ENDPOINT), { method: 'POST', ...json({ subject, to }) });
  } catch {
    throw new TrackingUnavailable('network');
  }
  if (!res.ok) {
    const body = (await res.json().catch(() => null)) as { error?: unknown } | null;
    throw new TrackingUnavailable(typeof body?.error === 'string' ? body.error : `status ${res.status}`);
  }
  const body = (await res.json()) as Partial<TrackingToken>;
  if (typeof body.token !== 'string' || typeof body.url !== 'string' || !/^https?:\/\//i.test(body.url)) {
    throw new TrackingUnavailable('malformed answer');
  }
  return { token: body.token, url: body.url };
}

/** Bind a minted token to the Email the send created. Resolves `false` when
 *  the server did not take it (the token then expires unbound after 24 h). */
export async function bindTrackingToken(token: string, emailId: string, fetcher: Fetcher = apiFetch): Promise<boolean> {
  try {
    const res = await fetcher(withBase(`${ENDPOINT}/${encodeURIComponent(token)}`), {
      method: 'PUT',
      ...json({ emailId }),
    });
    return res.ok;
  } catch {
    return false;
  }
}

/** The opens recorded for a sent Email, or `null` when none is tracked. */
export async function openCount(emailId: string, fetcher: Fetcher = apiFetch): Promise<OpenCount | null> {
  const res = await fetcher(withBase(`${ENDPOINT}?emailId=${encodeURIComponent(emailId)}`), {
    credentials: 'same-origin',
  });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`open count failed with ${res.status}`);
  const body = (await res.json()) as Partial<OpenCount>;
  return {
    count: typeof body.count === 'number' ? body.count : 0,
    first: typeof body.first === 'string' ? body.first : null,
    last: typeof body.last === 'string' ? body.last : null,
  };
}

/** The image that records an open, appended after the body. The URL comes from
 *  this server and is checked to be http(s); it is escaped as an attribute. */
export function trackingPixel(url: string): string {
  const safe = url.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  return `<img src="${safe}" width="1" height="1" alt="">`;
}

/** The draft with the image appended and the token to bind once it is sent.
 *  The token is minted for the draft's subject and every recipient address
 *  (To, Cc and Bcc: the sender's own record of who was sent the message). */
export async function withTrackingPixel(
  draft: DraftInput,
  mint: (subject: string, to: string[]) => Promise<TrackingToken> = mintTrackingToken,
): Promise<{ draft: DraftInput; token: string }> {
  const to = [draft.to, draft.cc ?? '', draft.bcc ?? ''].flatMap((f) => parseRecipients(f).map((r) => r.email));
  const minted = await mint(draft.subject, to);
  return { draft: { ...draft, htmlBody: draft.htmlBody + trackingPixel(minted.url) }, token: minted.token };
}
