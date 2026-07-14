// SSO HTTP clients (t9 e4 web).
//
// Two disjoint surfaces, both same-origin + cookie-authed, sharing nothing with
// the JMAP mailbox client (the mailbox path stays byte-unchanged):
//
//   • PUBLIC login surface — `GET /api/sso/providers` (pre-auth advertise) and
//     the `GET /api/sso/{id}/begin` full-redirect entry. Consumed by Login.
//   • ADMIN surface — `GET/POST/DELETE /admin/sso`, mirroring the `AdminApi`
//     pattern (a mockable interface + a thin `fetch` impl). Consumed by the
//     admin SSO sub-panel.
//
// e3 mounts these exact paths. See `.orchestration/logs/t9-e4.md` for the
// consumed-endpoint list.

import type { SsoBackendInput, SsoBackendRow, SsoProviderSummary } from './types.ts';

// ── Public login surface ───────────────────────────────────────────────────────

/** The path a "Sign in with <IdP>" control navigates to (full redirect). */
export function ssoBeginPath(id: string, base = ''): string {
  return `${base}/api/sso/${encodeURIComponent(id)}/begin`;
}

/** The SAML SP-metadata path for a backend (`GET /api/sso/{id}/metadata`). */
export function ssoMetadataPath(id: string, base = ''): string {
  return `${base}/api/sso/${encodeURIComponent(id)}/metadata`;
}

/**
 * Fetch the enabled IdPs to advertise on the login screen (`GET /api/sso/providers`).
 *
 * Best-effort + fail-soft: any error (endpoint absent, offline, non-2xx, or
 * `fetch` unavailable under jsdom) resolves to `[]`, so a deployment with no SSO
 * configured shows the login exactly as today — the additive path never breaks
 * password sign-in.
 */
export async function listSsoProviders(base = ''): Promise<SsoProviderSummary[]> {
  if (typeof fetch === 'undefined') return [];
  try {
    const res = await fetch(`${base}/api/sso/providers`, { credentials: 'same-origin' });
    if (!res.ok) return [];
    const body = (await res.json()) as unknown;
    // The server returns `{ providers: [...] }` (mw-server sso.rs GET /api/sso/providers);
    // tolerate a bare array too for forward-compat.
    const list = Array.isArray(body)
      ? body
      : ((body as { providers?: unknown })?.providers ?? []);
    return Array.isArray(list) ? (list as SsoProviderSummary[]) : [];
  } catch {
    return [];
  }
}

// ── Admin surface ──────────────────────────────────────────────────────────────

/** Raised when an `/admin/sso` request fails. */
export class SsoAdminError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'SsoAdminError';
    this.status = status;
  }
}

/**
 * The admin SSO-config client. The panel injects a mock in tests;
 * `createHttpSsoAdminApi` is the production `fetch` impl. Every method maps to
 * exactly one `/admin/sso` endpoint — the contract e3 mounts.
 */
export interface SsoAdminApi {
  /** `GET /admin/sso` → every configured backend (secrets excluded). */
  list(): Promise<SsoBackendRow[]>;
  /** `POST /admin/sso` → create or update a backend (secret write-only). */
  save(input: SsoBackendInput): Promise<void>;
  /** `DELETE /admin/sso/{id}` → remove a backend (idempotent). */
  remove(id: string): Promise<void>;
}

/** The production HTTP client (same-origin, admin-session cookie-authed). */
export function createHttpSsoAdminApi(base = ''): SsoAdminApi {
  return {
    async list() {
      const res = await fetch(`${base}/admin/sso`, { credentials: 'same-origin' });
      if (!res.ok) throw new SsoAdminError(res.status, `GET /admin/sso failed (${res.status})`);
      return (await res.json()) as SsoBackendRow[];
    },
    async save(input) {
      const res = await fetch(`${base}/admin/sso`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(input),
      });
      if (!res.ok) throw new SsoAdminError(res.status, `POST /admin/sso failed (${res.status})`);
    },
    async remove(id) {
      const res = await fetch(`${base}/admin/sso/${encodeURIComponent(id)}`, {
        method: 'DELETE',
        credentials: 'same-origin',
      });
      if (!res.ok) throw new SsoAdminError(res.status, `DELETE /admin/sso failed (${res.status})`);
    },
  };
}
