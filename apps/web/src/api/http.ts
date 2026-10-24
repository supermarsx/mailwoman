// The one place the web client calls `fetch` (enforced by the `mw/no-global-fetch`
// lint rule in `eslint.config.js`; the service worker and tests are outside it).
//
// What it adds to `fetch`: the double-submit CSRF token. The server sets a
// readable `mw_csrf` cookie at sign-in (`/api/login`, `/admin/login`) and when a
// signed-in client asks who it is (`GET /api/me`, `GET /admin/session`). With
// `MW_CSRF_STRICT=true` — what `deploy.sh` writes for a new installation — every
// POST/PUT/PATCH/DELETE authorised by a cookie must repeat that value in
// `X-CSRF-Token`, or it is answered
// `403 {"error":"missing or invalid CSRF token"}` before any handler runs
// (`state_change_guard`, `crates/mw-server/src/lib.rs`).
//
// What it does not do: it sets no `credentials` mode, no base path and no
// content type — those stay with the caller.
//
// A request with an `Authorization` header is treated like any other. The server
// excuses a bearer request from the check only when no session cookie comes with
// it, so the token is sent whenever the cookie can be read. In a native shell the
// server is another origin, its cookies are not in `document.cookie`, and the
// bearer request goes out exactly as the caller built it.
//
// In a native shell (t31) the SPA's own origin is the shell, not the server. A
// root-relative request (`/api/...`) is sent to the configured server instead,
// with the bearer token and no cookies, through the transport `transport.ts`
// registers with `setNativeTransport`. The server's answer to a sign-in route
// carries the token for a native client; it is handed back to that transport to
// keep. An absolute URL is left alone: `client.ts` builds those and attaches the
// token itself.
//
// Imports: `basePath.ts` only, which has none of its own, so `platform/browser.ts`
// can import this module without closing the `transport.ts` cycle it avoids.

import { basePath, stripBase } from './basePath.ts';
import { consolePath } from './adminPath.ts';

/** The readable cookie holding the token (`hardening::CSRF_COOKIE`). */
const CSRF_COOKIE = 'mw_csrf';
/** The header the token is repeated in (`hardening::CSRF_HEADER`). */
const CSRF_HEADER = 'X-CSRF-Token';
/** The `error` field of the guard's 403 (`state_change_guard`). */
const CSRF_REFUSAL = 'missing or invalid CSRF token';

/** Methods the guard does not check. */
const SAFE_METHODS = new Set(['GET', 'HEAD', 'OPTIONS']);

/**
 * Where a native shell sends root-relative requests (registered by
 * `transport.ts`; `null` in a browser).
 */
export interface NativeTransport {
  /** The server's base URL without a trailing slash, or `null` when none is set. */
  base(): string | null;
  /** The bearer token, or `null` before sign-in. */
  token(): Promise<string | null>;
  /** Keep the token a sign-in route answered with. */
  setToken(token: string): Promise<void>;
}

let native: NativeTransport | null = null;

/** Install (or, with `null`, remove) the native transport. */
export function setNativeTransport(transport: NativeTransport | null): void {
  native = transport;
}

/** The sign-in routes whose answer carries a native client's token. */
function isSignInPath(path: string): boolean {
  return path === '/api/login' || path.startsWith('/api/login/2fa');
}

/** `/api/...` but not `//host/...` (a scheme-relative URL names another host). */
function isRootRelative(input: string): boolean {
  return input.startsWith('/') && !input.startsWith('//');
}

/**
 * Send a root-relative request to the native shell's server: the bearer token
 * instead of cookies, and no CSRF token (the server does not check a request
 * whose only authority is its bearer token). A sign-in answer's token is kept.
 */
async function sendNative(transport: NativeTransport, input: string, init: RequestInit | undefined): Promise<Response> {
  const token = await transport.token();
  const base = transport.base();
  if (base === null) throw new TypeError('no server address is configured');
  const path = stripBase(input, basePath());
  const headers = new Headers(init?.headers);
  if (token !== null && !headers.has('Authorization')) headers.set('Authorization', `Bearer ${token}`);
  const res = await fetch(`${base}${path}`, { ...init, headers, credentials: 'omit' });
  if (res.ok && isSignInPath(path.split('?')[0] ?? path)) {
    const body = (await res.clone().json().catch(() => null)) as { token?: unknown } | null;
    if (typeof body?.token === 'string' && body.token !== '') await transport.setToken(body.token);
  }
  return res;
}

/** The current CSRF token from `document.cookie`, or `null` when there is none. */
export function csrfToken(): string | null {
  if (typeof document === 'undefined') return null;
  const prefix = `${CSRF_COOKIE}=`;
  for (const part of document.cookie.split(';')) {
    const pair = part.trim();
    if (pair.startsWith(prefix)) {
      const value = pair.slice(prefix.length);
      return value === '' ? null : value;
    }
  }
  return null;
}

/** `fetch` with the arguments the caller gave: no `init` stays no `init`. */
function send(input: string, init: RequestInit | undefined): Promise<Response> {
  return init === undefined ? fetch(input) : fetch(input, init);
}

function withToken(init: RequestInit | undefined, token: string): RequestInit {
  const headers = new Headers(init?.headers);
  headers.set(CSRF_HEADER, token);
  return { ...init, headers };
}

/** Is `res` the guard's token refusal? Reads a clone, so the body stays unread. */
async function isCsrfRefusal(res: Response): Promise<boolean> {
  if (res.status !== 403) return false;
  try {
    const body = (await res.clone().json()) as { error?: unknown } | null;
    return body?.error === CSRF_REFUSAL;
  } catch {
    return false; // not JSON: some other 403
  }
}

/**
 * The GET that makes the server issue a token for `input`: `<console>/session`
 * for an admin route (`/admin/session` unless the console is elsewhere, see
 * `adminPath.ts`), `/api/me` for everything else, under the same origin and
 * deploy prefix as `input`.
 */
function tokenSource(input: string): string {
  const base = basePath();
  const url = new URL(input, globalThis.location?.href ?? 'http://localhost');
  const path = stripBase(url.pathname, base);
  const root = consolePath();
  const source = path.startsWith(`${root}/`) ? `${root}/session` : '/api/me';
  const absolute = /^[a-z][a-z0-9+.-]*:\/\//i.test(input);
  return `${absolute ? url.origin : ''}${base}${source}`;
}

/** One request per source at a time: parallel refused writes share its answer. */
const refreshing = new Map<string, Promise<string | null>>();

/** Ask the server for a token; `null` when it issues none (not signed in). */
function refreshToken(source: string): Promise<string | null> {
  const running = refreshing.get(source);
  if (running !== undefined) return running;
  const next = (async () => {
    try {
      const res = await fetch(source, { credentials: 'same-origin' });
      if (!res.ok) return null;
      const body = (await res.json().catch(() => null)) as { csrfToken?: unknown } | null;
      // The cookie is what the server compares against, so it wins; the body
      // field covers a response whose `Set-Cookie` has not been applied yet.
      return csrfToken() ?? (typeof body?.csrfToken === 'string' ? body.csrfToken : null);
    } catch {
      return null;
    } finally {
      refreshing.delete(source);
    }
  })();
  refreshing.set(source, next);
  return next;
}

/**
 * `fetch`, plus the CSRF token on a write.
 *
 * For a method other than GET/HEAD/OPTIONS the value of the `mw_csrf` cookie is
 * sent as `X-CSRF-Token`. With no such cookie the request is sent as given.
 *
 * If the server refuses the write for its token, a token is requested once
 * (see `tokenSource`) and the write is sent once more. The guard refuses before
 * the handler runs, so the first attempt changed nothing. A body that can be
 * read only once (a stream) is not sent again; the refusal is returned.
 *
 * In a native shell a root-relative `input` goes to the configured server with
 * the bearer token instead (see `setNativeTransport`).
 */
export async function apiFetch(input: string, init?: RequestInit): Promise<Response> {
  if (native !== null && isRootRelative(input)) return sendNative(native, input, init);
  const method = (init?.method ?? 'GET').toUpperCase();
  if (SAFE_METHODS.has(method)) return send(input, init);
  const token = csrfToken();
  const res = await send(input, token === null ? init : withToken(init, token));
  if (!(await isCsrfRefusal(res))) return res;
  if (typeof ReadableStream !== 'undefined' && init?.body instanceof ReadableStream) return res;
  const fresh = await refreshToken(tokenSource(input));
  if (fresh === null) return res;
  return fetch(input, withToken(init, fresh));
}
