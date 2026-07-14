// OAuth 2.1 consent service (SPEC §20.1, plan §2.3 / §3 e8). Talks to the authorize/
// consent endpoints e11 mounts. The untrusted client supplies the authorize params in
// the URL; the server validates the client + redirect_uri + PKCE + resource and returns
// the friendly display metadata. The resource OWNER's identity comes from the session,
// never from the client (matching `mw-oauth::AuthorizeRequest`).

import type { WireScope } from '../../modules/apikeys/index.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/** The raw OAuth 2.1 authorize params (from the redirect query). */
export interface AuthorizeParams {
  responseType: string;
  clientId: string;
  redirectUri: string;
  state?: string | undefined;
  codeChallenge: string;
  codeChallengeMethod: string;
  resource: string;
  /** The requested scope, in `mw-oauth` wire form (server-normalized). */
  scope?: WireScope | undefined;
}

/** Server-validated consent context (the client display + the exact requested scope). */
export interface ConsentContext {
  readonly clientId: string;
  readonly clientName: string;
  /** Whether this client is admin-approved in the registry (§2.3). */
  readonly approved: boolean;
  readonly redirectUri: string;
  readonly resource: string;
  readonly requestedScope: WireScope;
}

/** The result of a consent decision: where to send the user's browser next. */
export interface ConsentResult {
  readonly redirectUri: string;
}

/** Parse the authorize params from a URL query string (browser entry point). */
export function parseAuthorizeParams(search: string): AuthorizeParams {
  const q = new URLSearchParams(search);
  const scopeRaw = q.get('scope');
  let scope: WireScope | undefined;
  if (scopeRaw !== null) {
    try {
      scope = JSON.parse(scopeRaw) as WireScope;
    } catch {
      scope = undefined;
    }
  }
  return {
    responseType: q.get('response_type') ?? 'code',
    clientId: q.get('client_id') ?? '',
    redirectUri: q.get('redirect_uri') ?? '',
    state: q.get('state') ?? undefined,
    codeChallenge: q.get('code_challenge') ?? '',
    codeChallengeMethod: q.get('code_challenge_method') ?? 'S256',
    resource: q.get('resource') ?? '',
    scope,
  };
}

/**
 * The consent service.
 * Endpoints (e11 to satisfy):
 *   POST /oauth/consent  (AuthorizeParams)          → ConsentContext  (validate + display)
 *   POST /oauth/decision ({approve, params})        → ConsentResult   (302 target / deny redirect)
 */
export class ConsentService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  /** Validate the request server-side and fetch the client display + requested scope. */
  async context(params: AuthorizeParams): Promise<ConsentContext> {
    const res = await this.fetcher('/oauth/consent', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(params),
    });
    if (!res.ok) throw new Error(`consent context failed: ${res.status}`);
    return (await res.json()) as ConsentContext;
  }

  /** Record grant/deny. On grant the server mints the code and returns the redirect. */
  async decide(params: AuthorizeParams, approve: boolean): Promise<ConsentResult> {
    const res = await this.fetcher('/oauth/decision', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ approve, params }),
    });
    if (!res.ok) throw new Error(`consent decision failed: ${res.status}`);
    return (await res.json()) as ConsentResult;
  }
}
