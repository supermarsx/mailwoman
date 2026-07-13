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
        return jsonOrThrow<Me>(res);
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
