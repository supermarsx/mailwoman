// Same-origin HTTP client for the mw-server contract (plan §2).
// All requests are cookie-authed (credentials: 'same-origin'); the browser
// never sees upstream JMAP creds.

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

async function req(input: string, init?: RequestInit): Promise<Response> {
  let res: Response;
  try {
    res = await fetch(input, {
      credentials: 'same-origin',
      ...init,
    });
  } catch (cause) {
    throw new NetworkError(cause instanceof Error ? cause.message : 'network request failed');
  }
  return res;
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

export function createClient(base = ''): Client {
  const listeners = new Set<NetworkListener>();

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
