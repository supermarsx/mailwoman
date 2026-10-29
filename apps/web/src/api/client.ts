import { withLoginTimeout } from './loginTimeout.ts';
// HTTP client for the mw-server contract (plan §2).
//
// BROWSER (default): same-origin, cookie-authed (`credentials: 'same-origin'`);
// the browser never sees upstream JMAP creds — this path is byte-identical to
// pre-V5 and is the hard regression gate.
//
// NATIVE (V5 thin shell, opt-in — plan §2.2): a configurable `base` URL points
// the transport at a remote Mailwoman server, and an optional `ClientAuth`
// attaches an `Authorization: Bearer <token>` header (from the OS keychain) and
// drops the cookie (`credentials: 'omit'`), sidestepping the cookie-only CSRF
// guard. The browser passes neither, so nothing about the cookie path changes.
// With a `ClientAuth`, `login` asks for a bearer session (`clientType: "native"`)
// and hands the token the server returns to `auth.setToken`; `logout` clears it.

import type { JmapRequest, JmapResponse, JmapSession } from './jmap-types.ts';
import { basePath } from './basePath.ts';
import { apiFetch,type ApiRequestInit } from './http.ts';
import {linkedMailboxEpoch,LinkedContextUnsupportedError} from './linkedMailboxSession.ts';

/** What a refused answer says about itself, besides its status. */
export interface ApiErrorDetails {
  /** The server's machine code (`accountDisabled`, `signinNotAllowed`, …). */
  code?: string;
  /** The answer's `X-Request-Id`, shown as "Reference: …" for a support request. */
  requestId?: string;
  retryAfterSecs?: number;
}

export class ApiError extends Error {
  readonly status: number;
  readonly code: string | undefined;
  readonly requestId: string | undefined;
  readonly retryAfterSecs: number | undefined;
  constructor(status: number, message: string, details: ApiErrorDetails = {}) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = details.code;
    this.requestId = details.requestId;
    this.retryAfterSecs = details.retryAfterSecs;
  }
}

/**
 * The machine code and request id of a refused answer. Reads a clone, so the
 * body stays readable; never throws (a body that is not JSON has no code).
 */
export async function errorDetails(res: Response): Promise<ApiErrorDetails> {
  const out: ApiErrorDetails = {};
  const requestId = res.headers.get('x-request-id');
  if (requestId !== null && requestId !== '') out.requestId = requestId;
  const retryAfter = res.headers.get('retry-after');
  if (retryAfter !== null && /^\d+$/.test(retryAfter)) {
    const seconds = Number(retryAfter);
    if (Number.isSafeInteger(seconds) && seconds <= 86400) out.retryAfterSecs = seconds;
  }
  try {
    const body = (await res.clone().json()) as { code?: unknown; retryAfterSecs?: unknown };
    if (typeof body.code === 'string' && body.code !== '') out.code = body.code;
    if (out.retryAfterSecs === undefined && typeof body.retryAfterSecs === 'number' && Number.isInteger(body.retryAfterSecs) && body.retryAfterSecs >= 0 && body.retryAfterSecs <= 86400) out.retryAfterSecs = body.retryAfterSecs;
  } catch {
    /* not JSON */
  }
  return out;
}

/** Raised when the request never reached the server (offline / dropped). */
export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

/**
 * What a sign-in sends: the address (or user name) and the password. The server
 * decides which mail server that address signs in to; a body that names one is
 * refused (t32-h9).
 */
export interface LoginInput {
  altcha?: string;
  username: string;
  password: string;
  /**
   * `"native"` asks the server for a bearer-token session (token in the body, no
   * cookie). A client built with a `ClientAuth` (the native shells) sets it
   * itself; the browser never sends it.
   */
  clientType?: 'native';
}

export interface Me {
  /** Opaque verified storage namespace; never used as a JMAP account ID. */
  storageAuthority?: string;
  username: string;
  accountId: string;
  /**
   * `true` while the account's admin `force_password_change` flag is set; the
   * server omits the field otherwise. Sent by both `/api/login` and `/api/me`.
   * While it is set the server refuses everything except the password-change
   * endpoints, `/api/me`, logout and session rotation, so the shell must not
   * load mail for such a session.
   */
  passwordChangeRequired?: boolean;
}

/**
 * Thrown when the server refuses a request with
 * `403 {"error":"password change required","passwordChangeRequired":true}`:
 * the session is valid, but the account must change its password first. It is
 * an `ApiError` (status 403) so status checks keep working; the body field, not
 * the status, is what identifies it — `POST /api/password` answers a plain 403
 * for a wrong current password, and the origin/CSRF guard answers 403 too.
 */
export class PasswordChangeRequired extends ApiError {
  constructor() {
    super(403, 'password change required');
    this.name = 'PasswordChangeRequired';
  }
}

export type PasswordChangeRequiredListener = () => void;

export type UnauthenticatedListener = () => void;

/** Is `res` the forced-password-change refusal (see {@link PasswordChangeRequired})? */
async function isPasswordChangeGate(res: Response): Promise<boolean> {
  if (res.status !== 403) return false;
  try {
    const body = (await res.json()) as { passwordChangeRequired?: unknown } | null;
    return body?.passwordChangeRequired === true;
  } catch {
    return false; // not JSON: an ordinary 403
  }
}

/**
 * The `twofaRequired` body `/api/login` returns when the credentials are correct
 * but a second factor must be cleared before a session is issued (SPEC §7.4/§19).
 * Mirrors the server shape in `crates/mw-server/src/twofa_routes.rs::gate_login`
 * and is structurally the `LoginChallenge` the `TwoFactorChallenge` component
 * consumes — kept here (api layer) so nothing in `api/` depends on a screen.
 */
export interface LoginChallenge {
  readonly pendingToken: string;
  /** Which factors the user may present ("totp" | "webauthn" | "recovery"). */
  readonly factors: readonly string[];
  /** Present when a policy-required user has nothing enrolled yet. */
  readonly enrollmentRequired?: boolean;
  readonly webauthn?: {
    readonly challenge: string;
    readonly credentialIds: readonly string[];
    readonly rpId: string;
    readonly userVerification: string;
  };
}

/**
 * Thrown by `login()` when the server answers `twofaRequired`: the password was
 * accepted but NO session cookie was issued. Carries the challenge the caller
 * must clear (there is no password-only downgrade — the login is not complete
 * until a factor verifies). Modelled as a throw so `app.login` propagates it
 * WITHOUT running its post-login session bootstrap.
 */
export class TwoFactorRequired extends Error {
  readonly challenge: LoginChallenge;
  constructor(challenge: LoginChallenge) {
    super('second factor required');
    this.name = 'TwoFactorRequired';
    this.challenge = challenge;
  }
}

/** What `/api/login` returns: either the authenticated identity, or a 2FA gate. */
type LoginResponse = Me | ({ readonly twofaRequired: true } & LoginChallenge);

function isTwofaGate(body: LoginResponse): body is { readonly twofaRequired: true } & LoginChallenge {
  return (body as { twofaRequired?: unknown }).twofaRequired === true;
}

export type NetworkListener = (online: boolean) => void;

/** The calls that need a session: with no server known they answer 401. */
const SESSION_PATHS = new Set(['/api/me', '/jmap/session', '/jmap/api', '/api/sanitize']);

/**
 * Optional bearer-auth provider for the native shell (plan §2.2). When present
 * and it yields a token, the client attaches `Authorization: Bearer <token>` and
 * omits cookies. Absent (the browser default) → the cookie same-origin path,
 * unchanged.
 */
export interface ClientAuth {
  /** The current session bearer token, or `null` before sign-in. */
  token(): Promise<string | null>;
  /** Keep the token a native sign-in returned. */
  setToken?(token: string): Promise<void>;
  /** Forget the token (sign-out). */
  clearToken?(): Promise<void>;
}

/**
 * Where the client sends requests: a fixed prefix, or a function read at each
 * request that returns `null` while no server is known (a native shell before the
 * user has entered a server address).
 */
export type ClientBase = string | (() => string | null);

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    throw new ApiError(res.status, `request failed with ${res.status}`, await errorDetails(res));
  }
  return (await res.json()) as T;
}

/**
 * The server runs a bounded number of JMAP requests at once and answers the
 * rest 429 with the RFC 8620 `limit` problem `maxConcurrentRequests` BEFORE
 * running any of their calls (mw-server `jmap_limits.rs`). Such a refusal
 * changed nothing, so it is safe to send again; a client with several tabs open
 * reaches that ceiling in ordinary use (a send racing the open readers' loads).
 * These are the waits before each retry; a 429 for any other reason is not
 * retried.
 */
const JMAP_BUSY_RETRY_MS = [100, 250, 500, 1000, 2000];

async function isJmapBusy(res: Response): Promise<boolean> {
  if (res.status !== 429) return false;
  const body = (await res.clone().json().catch(() => null)) as { type?: unknown; limit?: unknown } | null;
  return body?.type === 'urn:ietf:params:jmap:error:limit' && body.limit === 'maxConcurrentRequests';
}

/** Resolve after `ms`, or reject with the signal's reason once it aborts. */
function pause(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) { reject(signal.reason); return; }
    const onAbort = (): void => { clearTimeout(timer); reject(signal!.reason); };
    const timer = setTimeout(() => { signal?.removeEventListener('abort', onAbort); resolve(); }, ms);
    signal?.addEventListener('abort', onAbort, { once: true });
  });
}

export interface Client {
  login(input: LoginInput): Promise<Me>;
  logout(): Promise<void>;
  me(context?:{linkId:string|null}): Promise<Me>;
  session(context?:{linkId:string|null}): Promise<JmapSession>;
  /**
   * Post a JMAP request. `opts.signal` cancels it: the list layer supersedes its
   * own in-flight queries (`state/slices/mail.ts`), and with append-on-scroll
   * every scroll issues one, so without this a superseded page keeps its socket
   * and has its body parsed only to be discarded.
   *
   * An aborted call rejects with the abort reason — NOT a {@link NetworkError} —
   * so it does not register as an offline transition. See `req`.
   *
   * `opts.keepalive` asks the browser to finish the request even if the page is
   * closed or reloaded meanwhile (a save started just before). Browsers allow
   * {@link KEEPALIVE_BUDGET} bytes of keepalive bodies in flight and reject a
   * fetch over it, so a body that does not fit in what is left is sent as an
   * ordinary request.
   */
  jmap(body: JmapRequest, opts?: { signal?: AbortSignal; keepalive?: boolean }): Promise<JmapResponse>;
  sanitize(html: string): Promise<string>;
  onNetwork(listener: NetworkListener): () => void;
  /**
   * Subscribe to the forced-password-change refusal: called each time a request
   * made through this client is answered with `passwordChangeRequired` (an
   * admin can set the flag while a session is open). Optional so a hand-built
   * client need not provide it.
   */
  onPasswordChangeRequired?(listener: PasswordChangeRequiredListener): () => void;
  /**
   * Subscribe to 401s from the endpoints that need a session (`/api/me`,
   * `/jmap/session`, `/jmap/api`, `/api/sanitize`). `/api/login`'s 401 is a
   * refused credential and is not reported. The client cannot tell "the session
   * ended" from "there never was one" — `/api/me` answers 401 at every
   * signed-out boot — so the listener decides. Optional so a hand-built client
   * need not provide it.
   */
  onUnauthenticated?(listener: UnauthenticatedListener): () => void;
}

/**
 * The bytes of keepalive request bodies this module lets the page have in
 * flight at once: the Fetch standard's 64 KiB per page, less a margin for other
 * keepalive senders. The count is per page, so it is shared by every client.
 */
export const KEEPALIVE_BUDGET = 60 * 1024;
let keepaliveInFlight = 0;

/**
 * Build an API client. `base` defaults to the sub-path prefix (t20 B4) — `''` at
 * the origin root, so a bare `createClient()` is unchanged there, and `/mail`
 * under `MW_BASE_PATH=/mail`, so callers that construct a client directly rather
 * than through `createConfiguredClient()` are prefixed too.
 *
 * A function `base` is read at each request. While it returns `null` no request
 * is sent: the calls that need a session answer as signed out (401), the others
 * fail as unreachable.
 */
export function createClient(base: ClientBase = basePath(), auth?: ClientAuth): Client {
  let loginPending = false;
  const listeners = new Set<NetworkListener>();
  const gateListeners = new Set<PasswordChangeRequiredListener>();
  const unauthListeners = new Set<UnauthenticatedListener>();

  /** `jsonOrThrow`, with a 401 and the forced-password-change refusal told
   *  apart first. */
  async function jsonOrGate<T>(res: Response): Promise<T> {
    if (res.status === 401) {
      for (const l of unauthListeners) l();
      throw new ApiError(401, 'not authenticated');
    }
    if (await isPasswordChangeGate(res)) {
      for (const l of gateListeners) l();
      throw new PasswordChangeRequired();
    }
    return jsonOrThrow<T>(res);
  }

  /**
   * Perform a request. With no `auth` (the browser default) this is byte-for-byte
   * the pre-V5 behaviour: `credentials: 'same-origin'` and the caller's `init`
   * verbatim. With an `auth` that yields a token it attaches the bearer header and
   * omits cookies (native cross-origin path).
   */
  async function req(path: string, init?: ApiRequestInit): Promise<Response> {
    const contextEpoch=linkedMailboxEpoch();
    const finalInit: ApiRequestInit = { credentials: 'same-origin', ...init };
    if (auth) {
      // Read before the base: the token provider waits for the platform, which is
      // what puts a stored server address in place at boot.
      const token = await auth.token();
      if(contextEpoch!==linkedMailboxEpoch())throw new DOMException('Mailbox changed','AbortError');
      finalInit.credentials = 'omit'; // bearer path: no ambient cookie authority.
      if (token !== null) {
        finalInit.headers = { ...(init?.headers as Record<string, string>), Authorization: `Bearer ${token}` };
      }
    }
    const prefix = typeof base === 'function' ? base() : base;
    if (prefix === null) {
      if (SESSION_PATHS.has(path)) return new Response(null, { status: 401 });
      throw new NetworkError('no server address is configured');
    }
    const input = `${prefix}${path}`;
    let res: Response;
    try {
      res = await apiFetch(input, finalInit);
    } catch (cause) {
      // A DELIBERATE cancellation is not a network failure, and must not be
      // reported as one: `guarded` turns a NetworkError into `notify(false)`,
      // which is what `offline.ts` replays its queue on and what flips the app
      // into offline mode. Since the list layer aborts a superseded query on
      // every scroll, mapping an abort to NetworkError would announce "offline"
      // during ordinary scrolling. Rethrow the abort reason unchanged; the
      // caller's own generation guard is what decides it no longer matters.
      if (finalInit.signal?.aborted === true||cause instanceof LinkedContextUnsupportedError||(cause instanceof DOMException&&cause.name==='AbortError')) throw cause;
      throw new NetworkError(cause instanceof Error ? cause.message : 'network request failed');
    }
    return res;
  }

  function notify(online: boolean): void {
    for (const l of listeners) l(online);
  }

  /** Run a request, emitting network up/down transitions to listeners. */
  async function guarded<T>(run: () => Promise<T>): Promise<T> {
    try {
      const out = await run();
      notify(true);
      return out;
    } catch (err) {
      // A NetworkError means the server was not reached. An ApiError carries
      // the status the server answered with (401, 403, 500, ...), so the
      // server WAS reached: report "online", or a "connection lost" notice
      // raised by an earlier failure stays up after a retry that ends in 401.
      if (err instanceof NetworkError) notify(false);
      else if (err instanceof ApiError) notify(true);
      throw err;
    }
  }

  return {
    login(input) {
      if (loginPending) return Promise.reject(new Error('Sign-in is already in progress'));
      loginPending = true;
      return guarded(() => withLoginTimeout(async signal => {
        // Only these fields: the server refuses a body that names a mail server.
        const body: LoginInput = { username: input.username, password: input.password };
        if (input.altcha) body.altcha = input.altcha;
        if (auth) body.clientType = 'native';
        const res = await req(`/api/login`, {
          signal,
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(body),
        });
        if (res.status === 401) {
          throw new ApiError(401, 'invalid credentials', await errorDetails(res));
        }
        const answer = await jsonOrThrow<LoginResponse>(res);
        signal.throwIfAborted();
        // A 2FA-gated response is a SUCCESSFUL request that is not yet an
        // authenticated session: surface the challenge (no session cookie was
        // set) rather than mistaking the body for a `Me`.
        if (isTwofaGate(answer)) {
          const { twofaRequired: _required, ...challenge } = answer;
          throw new TwoFactorRequired(challenge);
        }
        // A native sign-in returns its bearer token in the body (no cookie).
        const { token, ...me } = answer as Me & { token?: unknown };
        if (auth?.setToken && typeof token === 'string' && token !== '') await auth.setToken(token);
        return me;
      })).finally(() => { loginPending = false; });
    },
    logout() {
      return guarded(async () => {
        try {
          await req(`/api/logout`, { method: 'POST' });
        } finally {
          await auth?.clearToken?.();
        }
      });
    },
    me(context) {
      return guarded(async () => {
        const res = await req(`/api/me`,context?{mailboxContext:context.linkId}:undefined);
        return jsonOrGate<Me>(res);
      });
    },
    session(context) {
      return guarded(async () => {
        const res = await req(`/jmap/session`,context?{mailboxContext:context.linkId}:undefined);
        return jsonOrGate<JmapSession>(res);
      });
    },
    jmap(body, opts) {
      const text = JSON.stringify(body);
      const size = new TextEncoder().encode(text).length;
      const keepalive = opts?.keepalive === true && keepaliveInFlight + size <= KEEPALIVE_BUDGET;
      if (keepalive) keepaliveInFlight += size;
      return guarded(async () => {
        try {
          const send = (): Promise<Response> => req(`/jmap/api`, {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: text,
            ...(opts?.signal !== undefined ? { signal: opts.signal } : {}),
            ...(keepalive ? { keepalive: true } : {}),
          });
          let res = await send();
          for (const wait of JMAP_BUSY_RETRY_MS) {
            if (!(await isJmapBusy(res))) break;
            await pause(wait, opts?.signal);
            res = await send();
          }
          return await jsonOrGate<JmapResponse>(res);
        } finally {
          if (keepalive) keepaliveInFlight -= size;
        }
      });
    },
    sanitize(html) {
      return guarded(async () => {
        const res = await req(`/api/sanitize`, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ html }),
        });
        const out = await jsonOrGate<{ html: string }>(res);
        return out.html;
      });
    },
    onNetwork(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    onPasswordChangeRequired(listener) {
      gateListeners.add(listener);
      return () => gateListeners.delete(listener);
    },
    onUnauthenticated(listener) {
      unauthListeners.add(listener);
      return () => unauthListeners.delete(listener);
    },
  };
}
