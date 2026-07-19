// Admin two-factor-policy client (26.16, plan §3 e16 — DQ2).
//
// The require-2FA policy is a small admin-gated surface e3 exposes at
// `GET|POST /admin/2fa/policy` — SEPARATE from the frozen `/admin/*` REST contract
// in `state/slices/admin.ts`. It lives in its own file (mirroring `api/maintenance.ts`)
// rather than extending the frozen `AdminApi`, because it is a distinct endpoint
// pair, not a method on that contract.
//
// Contract (e3, twofa_routes.rs):
//   GET  /admin/2fa/policy → { policies: TwofaPolicyRow[] } (admin-session cookie)
//   POST /admin/2fa/policy   body { scopeKind: 'global'|'domain', scopeValue, require2fa }
// A `global` scope pins `scopeValue` to ''. There is no DELETE: a scope is disabled
// by upserting it with `require2fa: false`.

/** One require-2FA policy row (global, or per-domain). */
export interface TwofaPolicyRow {
  /** `'global'` (whole deployment) or `'domain'` (one mail domain). */
  scopeKind: 'global' | 'domain';
  /** The domain for a `'domain'` scope; `''` for `'global'`. */
  scopeValue: string;
  /** Whether a second factor is required for accounts in scope. */
  require2fa: boolean;
  /** The admin who last set it (server-populated), if known. */
  updatedBy?: string | null;
  /** Last-updated timestamp (server-populated), if known. */
  updatedAt?: string | null;
}

/** The body accepted by `POST /admin/2fa/policy`. */
export interface TwofaPolicyInput {
  scopeKind: 'global' | 'domain';
  scopeValue: string;
  require2fa: boolean;
}

/** Raised when a `/admin/2fa/policy` request returns a non-2xx response. */
export class TwofaPolicyApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'TwofaPolicyApiError';
    this.status = status;
  }
}

/**
 * The admin two-factor-policy client. Component tests inject a mock;
 * `createHttpTwofaPolicyApi` is the production `fetch` impl. `base` lets a native
 * shell point at a remote server (as the admin/JMAP clients do), defaulting to
 * same-origin in the browser.
 */
export interface TwofaPolicyApi {
  /** `GET /admin/2fa/policy` → the current policy rows. */
  list(): Promise<TwofaPolicyRow[]>;
  /** `POST /admin/2fa/policy` → upsert a policy row (global or per-domain). */
  set(input: TwofaPolicyInput): Promise<void>;
}

/** The production HTTP client (same-origin, admin-session cookie). */
export function createHttpTwofaPolicyApi(base = ''): TwofaPolicyApi {
  return {
    async list() {
      const res = await fetch(`${base}/admin/2fa/policy`, { credentials: 'same-origin' });
      if (!res.ok) throw new TwofaPolicyApiError(res.status, `list 2fa policy failed (${res.status})`);
      const body = (await res.json()) as { policies?: TwofaPolicyRow[] };
      return body.policies ?? [];
    },
    async set(input) {
      const res = await fetch(`${base}/admin/2fa/policy`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(input),
      });
      if (!res.ok) throw new TwofaPolicyApiError(res.status, `set 2fa policy failed (${res.status})`);
    },
  };
}
