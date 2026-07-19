// Thin client wrapper for the anonymizing image proxy's remote-image display
// grants (t16 §S8/S9, e14b UI ↔ e6 server). This is the ONE file that binds the
// UI to e6's wire shapes: the reader's grant bar (`RemoteContentBar.tsx`) and its
// tests import ONLY the interface + types below, never a raw request, so this
// module is the single place that tracks e6's server contract.
//
// e6 (`crates/mw-server/src/image_proxy.rs`) ships the grant surface as REST over
// the 0016 4-scope model, session-authed (the account is the session; a
// client-supplied account id is never trusted):
//   • `GET  /api/remote-images/grants`  → `{ accountId, list: RemoteImageGrant[] }`
//   • `POST /api/remote-images/grant`   ← `{ scopeKind, scopeValue }`
//   • `POST /api/remote-images/revoke`  ← `{ scopeKind, scopeValue }`
// and the anonymizing fetch as `GET /api/image-proxy?url=<original>` — the reader
// rewrites a GRANTED remote image's `src` to that same-origin URL ({@link
// rewriteGrantedImages}) so the browser only ever contacts Mailwoman itself.
//
// The grant model is the 0016 `remote_image_grants` table's 4-scope shape
// (`crates/mw-store/src/image_grants.rs`): a remote image loads only when a
// matching, non-revoked grant exists — deny-by-default (SPEC §7.2). `single`
// grants one message, `per-sender` a sender address, `per-domain` a sender
// domain, `all` the whole account.
//
// The blocked-content REPORT (how many trackers/remote resources the sanitizer
// stripped, and from which hosts — S9) is derived CLIENT-SIDE from the already-
// sanitized body string, so it needs no extra round-trip. The contract with the
// sanitizer (e6 adds the classification without changing the strip default,
// `crates/mw-sanitize/src/lib.rs`): a stripped remote resource is marked with
// `data-mw-blocked-host="<host>"`, and one the sanitizer classified as a tracker
// (1×1 beacon / known-tracker host) additionally carries `data-mw-tracker`. When
// those markers are absent (a body with nothing blocked, or a pre-e6 sanitizer)
// the report is empty and the bar stays hidden — honest by construction.

/** The 4 grant scopes (0016 `scope_kind`). */
export type GrantScopeKind = 'single' | 'all' | 'per-sender' | 'per-domain';

/** A grant scope: `kind` + the value it applies to (message id / sender / domain;
 *  `''` for the account-wide `all`). */
export interface GrantScope {
  kind: GrantScopeKind;
  value: string;
}

/** An active remote-image grant (0016 row, public fields). */
export interface RemoteImageGrant {
  scopeKind: GrantScopeKind;
  scopeValue: string;
  grantedAt: string;
}

/** What the sanitizer blocked in the open message's body (S9). */
export interface BlockedContentReport {
  /** Distinct remote hosts whose resources were stripped, deduped + sorted. */
  blockedHosts: string[];
  /** Total number of blocked remote resources (may exceed `blockedHosts.length`
   *  when several load from the same host). */
  blockedCount: number;
  /** How many of the blocked resources the sanitizer classified as trackers. */
  trackerCount: number;
}

const EMPTY_REPORT: BlockedContentReport = { blockedHosts: [], blockedCount: 0, trackerCount: 0 };

/** The remote-image grant surface the UI depends on. e6 provides the production
 *  implementation ({@link createRemoteImageApi}); tests pass a fake. */
export interface RemoteImageApi {
  /** Grant remote-image loading for a scope (idempotent; un-revokes). */
  grant(accountId: string, scope: GrantScope): Promise<void>;
  /** Soft-revoke a grant (remote images for that scope block again next load). */
  revoke(accountId: string, scope: GrantScope): Promise<void>;
  /** Active (non-revoked) grants for the account, newest first. */
  listGrants(accountId: string): Promise<RemoteImageGrant[]>;
}

/** An injectable request function so the reader unit-tests with a fake — the
 *  default is a same-origin cookie-authed `fetch`, matching the sibling settings
 *  service (`screens/Settings/service.ts`). */
export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/** The account-wide grants list the server returns. */
interface GrantsResponse {
  accountId: string;
  list: RemoteImageGrant[];
}

/**
 * Build the production {@link RemoteImageApi} over e6's REST grant endpoints (the
 * same session the reader already uses; the server derives the account from the
 * session, so the `accountId` argument is accepted for interface compatibility but
 * not sent). This mapping is the whole e6 seam.
 */
export function createRemoteImageApi(fetcher: Fetcher = defaultFetcher): RemoteImageApi {
  async function mutate(action: 'grant' | 'revoke', scope: GrantScope): Promise<void> {
    const res = await fetcher(`/api/remote-images/${action}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ scopeKind: scope.kind, scopeValue: scope.value }),
    });
    if (!res.ok) throw new Error(`remote-image ${action} failed with ${res.status}`);
  }
  return {
    grant: (_accountId, scope) => mutate('grant', scope),
    revoke: (_accountId, scope) => mutate('revoke', scope),
    async listGrants(_accountId) {
      const res = await fetcher('/api/remote-images/grants');
      if (!res.ok) throw new Error(`remote-image grants failed with ${res.status}`);
      const body = (await res.json()) as GrantsResponse;
      return body.list ?? [];
    },
  };
}

/**
 * The same-origin URL that routes an original remote image URL through e6's
 * anonymizing proxy (`GET /api/image-proxy?url=…`). Same-origin so the shell CSP's
 * `img-src 'self'` admits it inside the sandboxed body frame while the original
 * remote host stays disallowed; the proxy — not the browser — fetches the bytes.
 */
export function imageProxyUrl(originalUrl: string): string {
  return `/api/image-proxy?url=${encodeURIComponent(originalUrl)}`;
}

/** An absolute `http`/`https` URL (trimmed), else `null` — the proxy only fetches
 *  those, so a `cid:`/`data:`/relative/empty `src` is never rewritten. */
function absoluteRemoteSrc(src: string | null): string | null {
  if (src === null) return null;
  const s = src.trim();
  return /^https?:\/\//i.test(s) ? s : null;
}

/**
 * Route a message body's GRANTED remote images through the anonymizing proxy.
 *
 * The sanitizer (`crates/mw-sanitize`) strips every remote `<img src>` by default
 * and keeps NO loadable URL, so the originals are recovered from the message's RAW
 * html: the sanitizer preserves every `<img>` element in document order (it removes
 * only the remote `src`, never the element), so the k-th sanitized `<img>`
 * corresponds to the k-th raw `<img>`. For a body the caller says is `granted` (a
 * covering grant exists — see {@link coveringGrant}), each raw absolute-http(s)
 * image whose sanitized twin had its `src` stripped is repointed at
 * {@link imageProxyUrl}; everything else is left exactly as the sanitizer produced
 * it.
 *
 * Safety (deny-by-default is never weakened):
 *   • `granted === false` → the sanitized body is returned BYTE-for-byte (no parse,
 *     no reserialize), so the default reader path is unchanged.
 *   • if the raw and sanitized `<img>` lists can't be aligned 1:1, NO change is
 *     made — a granted image simply stays blocked, never an ungranted one loaded.
 *   • only images the sanitizer actually stripped are touched, and only to a
 *     same-origin proxy URL; a surviving `cid:` image is left alone.
 */
export function rewriteGrantedImages(
  sanitizedHtml: string | null,
  rawHtml: string | null,
  granted: boolean,
): string | null {
  if (sanitizedHtml === null) return null;
  if (!granted || rawHtml === null || rawHtml === '') return sanitizedHtml;

  let rawDoc: Document;
  let cleanDoc: Document;
  try {
    rawDoc = new DOMParser().parseFromString(rawHtml, 'text/html');
    cleanDoc = new DOMParser().parseFromString(sanitizedHtml, 'text/html');
  } catch {
    return sanitizedHtml;
  }
  const rawImgs = Array.from(rawDoc.querySelectorAll('img'));
  const cleanImgs = Array.from(cleanDoc.querySelectorAll('img'));
  // Can't align → make no change (fail closed: keep ungranted content blocked).
  if (rawImgs.length !== cleanImgs.length) return sanitizedHtml;

  let rewrote = false;
  for (let i = 0; i < rawImgs.length; i += 1) {
    const original = absoluteRemoteSrc(rawImgs[i]!.getAttribute('src'));
    if (original === null) continue; // cid:/data:/relative/none — not proxied.
    const img = cleanImgs[i]!;
    // Only repoint an image the sanitizer stripped; a surviving src (e.g. cid:) is
    // left untouched.
    if (img.getAttribute('src') !== null) continue;
    img.setAttribute('src', imageProxyUrl(original));
    rewrote = true;
  }
  return rewrote ? cleanDoc.body.innerHTML : sanitizedHtml;
}

/** The sender domain of an address (`a@b.example` → `b.example`), lower-cased;
 *  `''` when the address has no `@`. */
export function senderDomain(address: string): string {
  const at = address.lastIndexOf('@');
  return at >= 0 ? address.slice(at + 1).toLowerCase() : '';
}

/**
 * The grant scope a UI action maps to, for the open message. `single` uses the
 * message id, `per-sender` the sender address, `per-domain` its domain, `all` the
 * empty account-wide value — matching `image_grants.rs`.
 */
export function scopeFor(kind: GrantScopeKind, ctx: { emailId: string; sender: string }): GrantScope {
  switch (kind) {
    case 'single':
      return { kind, value: ctx.emailId };
    case 'per-sender':
      return { kind, value: ctx.sender.toLowerCase() };
    case 'per-domain':
      return { kind, value: senderDomain(ctx.sender) };
    case 'all':
      return { kind, value: '' };
  }
}

/**
 * Derive the {@link BlockedContentReport} from a sanitized body string by reading
 * the sanitizer's block markers (`data-mw-blocked-host`, `data-mw-tracker`). Pure
 * + client-side: no network, safe on any string (a parse failure yields the empty
 * report). Parsing the STRING (not the live sandboxed iframe) keeps the reader's
 * no-same-origin frame contract intact.
 */
export function analyzeBlockedContent(sanitizedHtml: string | null): BlockedContentReport {
  if (sanitizedHtml === null || sanitizedHtml === '') return EMPTY_REPORT;
  let doc: Document;
  try {
    doc = new DOMParser().parseFromString(sanitizedHtml, 'text/html');
  } catch {
    return EMPTY_REPORT;
  }
  const marked = Array.from(doc.querySelectorAll('[data-mw-blocked-host]'));
  if (marked.length === 0) return EMPTY_REPORT;
  const hosts = new Set<string>();
  let trackerCount = 0;
  for (const el of marked) {
    const host = (el.getAttribute('data-mw-blocked-host') ?? '').trim().toLowerCase();
    if (host !== '') hosts.add(host);
    if (el.hasAttribute('data-mw-tracker')) trackerCount += 1;
  }
  return {
    blockedHosts: Array.from(hosts).sort((a, b) => a.localeCompare(b)),
    blockedCount: marked.length,
    trackerCount,
  };
}

/** True when a report has anything to show (drives whether the bar renders). */
export function hasBlockedContent(report: BlockedContentReport): boolean {
  return report.blockedCount > 0;
}

/** Whether an active grant list already covers the open message's context, so the
 *  bar can offer "turn off" (revoke) instead of "load". A `single` grant matches
 *  the message id, `per-sender` the sender, `per-domain` the domain, `all` always. */
export function coveringGrant(
  grants: RemoteImageGrant[],
  ctx: { emailId: string; sender: string },
): RemoteImageGrant | null {
  const domain = senderDomain(ctx.sender);
  const sender = ctx.sender.toLowerCase();
  for (const g of grants) {
    const v = g.scopeValue.toLowerCase();
    if (g.scopeKind === 'all') return g;
    if (g.scopeKind === 'single' && g.scopeValue === ctx.emailId) return g;
    if (g.scopeKind === 'per-sender' && v === sender && sender !== '') return g;
    if (g.scopeKind === 'per-domain' && v === domain && domain !== '') return g;
  }
  return null;
}
