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

import type { JmapRequest, JmapResponse, JmapSession } from './jmap-types.ts';

export class ApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

/** Raised when the request never reached the server (offline / dropped). */
export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

export interface LoginInput {
  jmapUrl: string;
  username: string;
  password: string;
}

export interface Me {
  username: string;
  accountId: string;
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

/**
 * Optional bearer-auth provider for the native shell (plan §2.2). When present
 * and it yields a token, the client attaches `Authorization: Bearer <token>` and
 * omits cookies. Absent (the browser default) → the cookie same-origin path,
 * unchanged.
 */
export interface ClientAuth {
  /** The current session bearer token, or `null` to use the cookie path. */
  token(): Promise<string | null>;
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    throw new ApiError(res.status, `request failed with ${res.status}`);
  }
  return (await res.json()) as T;
}

export interface Client {
  login(input: LoginInput): Promise<Me>;
  logout(): Promise<void>;
  me(): Promise<Me>;
  session(): Promise<JmapSession>;
  jmap(body: JmapRequest): Promise<JmapResponse>;
  sanitize(html: string): Promise<string>;
  onNetwork(listener: NetworkListener): () => void;
}

export function createClient(base = '', auth?: ClientAuth): Client {
  const listeners = new Set<NetworkListener>();

  /**
   * Perform a request. With no `auth` (the browser default) this is byte-for-byte
   * the pre-V5 behaviour: `credentials: 'same-origin'` and the caller's `init`
   * verbatim. With an `auth` that yields a token it attaches the bearer header and
   * omits cookies (native cross-origin path).
   */
  async function req(input: string, init?: RequestInit): Promise<Response> {
    const finalInit: RequestInit = { credentials: 'same-origin', ...init };
    if (auth) {
      const token = await auth.token();
      if (token !== null) {
        finalInit.headers = { ...(init?.headers as Record<string, string>), Authorization: `Bearer ${token}` };
        finalInit.credentials = 'omit'; // bearer path: no ambient cookie authority.
      }
    }
    let res: Response;
    try {
      res = await fetch(input, finalInit);
    } catch (cause) {
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
      if (err instanceof NetworkError) notify(false);
      throw err;
    }
  }

  return {
    login(input) {
      return guarded(async () => {
        const res = await req(`${base}/api/login`, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(input),
        });
        if (res.status === 401) {
          throw new ApiError(401, 'invalid credentials');
        }
        const body = await jsonOrThrow<LoginResponse>(res);
        // A 2FA-gated response is a SUCCESSFUL request that is not yet an
        // authenticated session: surface the challenge (no session cookie was
        // set) rather than mistaking the body for a `Me`.
        if (isTwofaGate(body)) {
          const { twofaRequired: _required, ...challenge } = body;
          throw new TwoFactorRequired(challenge);
        }
        return body;
      });
    },
    logout() {
      return guarded(async () => {
        await req(`${base}/api/logout`, { method: 'POST' });
      });
    },
    me() {
      return guarded(async () => {
        const res = await req(`${base}/api/me`);
        if (res.status === 401) throw new ApiError(401, 'not authenticated');
        return jsonOrThrow<Me>(res);
      });
    },
    session() {
      return guarded(async () => {
        const res = await req(`${base}/jmap/session`);
        if (res.status === 401) throw new ApiError(401, 'not authenticated');
        return jsonOrThrow<JmapSession>(res);
      });
    },
    jmap(body) {
      return guarded(async () => {
        const res = await req(`${base}/jmap/api`, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(body),
        });
        if (res.status === 401) throw new ApiError(401, 'not authenticated');
        return jsonOrThrow<JmapResponse>(res);
      });
    },
    sanitize(html) {
      return guarded(async () => {
        const res = await req(`${base}/api/sanitize`, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ html }),
        });
        const out = await jsonOrThrow<{ html: string }>(res);
        return out.html;
      });
    },
    onNetwork(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
}
