// Admin sign-in second step (t32-ext.md §M, TOTP for database admin accounts).
//
// Wire contract this client expects from t32-e24 (mirrors the webmail's
// `twofaRequired` gate in `crates/mw-server/src/twofa_routes.rs::gate_login`):
//   POST /admin/login            → 200 `{ twofaRequired: true, pendingToken, factors: ["totp"] }`
//                                   when the password is right and the account has TOTP;
//                                   no session cookie is set yet.
//   POST /admin/login/totp       `{ pendingToken, code }` → 200 `{ username }` + the
//                                   `mw_admin_session` cookie; 401 for a wrong or reused code.
// Before e24 the server never sends `twofaRequired`, so this step never shows.
//
// `withTotpGate` wraps any `AdminApi` (the HTTP client or a test mock) so that a
// `twofaRequired` answer to `login` is thrown as `AdminTotpRequired` instead of
// being taken for a session by the slice.

import { basePath } from '../../../api/basePath.ts';
import { apiFetch } from '../../../api/http.ts';
import { AdminApiError, type AdminApi, type AdminSession } from '../../../state/slices/admin.ts';

export interface AdminTotpChallenge {
  pendingToken: string;
  factors: string[];
}

export class AdminTotpRequired extends Error {
  readonly challenge: AdminTotpChallenge;
  constructor(challenge: AdminTotpChallenge) {
    super('admin second factor required');
    this.name = 'AdminTotpRequired';
    this.challenge = challenge;
  }
}

/** Completes the second step; resolves when the server has set the session cookie. */
export type AdminTotpVerifier = (pendingToken: string, code: string) => Promise<AdminSession>;

export function createAdminTotpVerifier(base = basePath()): AdminTotpVerifier {
  return async (pendingToken, code) => {
    const res = await apiFetch(`${base}/admin/login/totp`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ pendingToken, code }),
    });
    if (!res.ok) throw new AdminApiError(res.status, `admin totp failed (${res.status})`);
    return (await res.json()) as AdminSession;
  };
}

function isChallenge(v: unknown): v is AdminTotpChallenge & { twofaRequired: true } {
  return (
    typeof v === 'object' &&
    v !== null &&
    (v as { twofaRequired?: unknown }).twofaRequired === true &&
    typeof (v as { pendingToken?: unknown }).pendingToken === 'string'
  );
}

/** The same client, with a TOTP challenge on `login` raised as `AdminTotpRequired`. */
export function withTotpGate(api: AdminApi): AdminApi {
  async function login(username: string, password: string): Promise<AdminSession> {
    const answer: unknown = await api.login(username, password);
    if (isChallenge(answer)) {
      throw new AdminTotpRequired({
        pendingToken: answer.pendingToken,
        factors: Array.isArray(answer.factors) ? answer.factors : ['totp'],
      });
    }
    return answer as AdminSession;
  }
  // A proxy, not a copy: the slice sets `onSessionEnded` on the object it is given
  // and the HTTP client calls it on its own object, so writes must reach `api`
  // (a proxy without a set trap forwards them).
  return new Proxy(api, {
    get(target, prop, receiver) {
      if (prop === 'login') return login;
      return Reflect.get(target, prop, receiver) as unknown;
    },
  });
}
