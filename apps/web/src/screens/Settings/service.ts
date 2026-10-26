import { preferenceAccount, preferenceAccountIsCurrent, observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
// Settings-surface server I/O (t16 e15). One service over an injectable `Fetcher`
// so every component unit-tests without a live server. Same-origin cookie auth,
// byte-identical to the passwd/apikeys services (SPEC §7.4/§19, W12/W13/W15).
//
// Contract sources:
//   • 2FA + sessions  → crates/mw-server/src/twofa_routes.rs (LIVE this milestone)
//   • signatures / notifications / saved-searches / identities → the `mw-store`
//     0017 + frozen-0003 rows. Those persist server-side; the HTTP surface here is
//     the account-prefs contract the web drives (see the DONE report's backend note).

import { withBase } from '../../api/basePath.ts';
import { apiFetch } from '../../api/http.ts';
import { t } from '../../i18n';
import type {
  Identity,
  NotificationConfig,
  PasskeyRegistrationChallenge,
  RecoveryCodes,
  SavedSearch,
  SessionMeta,
  Signature,
  TotpBegin,
  TwofaStatus,
} from './types.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => apiFetch(input, { credentials: 'same-origin', ...init });

/**
 * Raised on a non-2xx settings request. `message` is localised text chosen from
 * the status and the server's `code` (t32-admin §2.8: the UI never shows the
 * server's English), with the answer's request id as a reference when it sent
 * one. `detail` keeps the server's own text for logs and tests; `status` lets
 * callers special-case 409.
 */
export class SettingsError extends Error {
  readonly status: number;
  readonly code: string | null;
  readonly detail: string;
  readonly requestId: string | null;
  constructor(
    status: number,
    message: string,
    extra: { code?: string | null; detail?: string; requestId?: string | null } = {},
  ) {
    super(message);
    this.name = 'SettingsError';
    this.status = status;
    this.code = extra.code ?? null;
    this.detail = extra.detail ?? message;
    this.requestId = extra.requestId ?? null;
  }
}

/** The localised text for a refused settings request. */
export function settingsErrorText(status: number, code: string | null, requestId: string | null): string {
  let id = 'settings-policy-error-generic';
  if (code === 'setting_locked') id = 'settings-policy-error-locked';
  else if (status === 401) id = 'settings-policy-error-signed-out';
  else if (status === 400 || status === 422) id = 'settings-policy-error-invalid';
  else if (status === 403) id = 'settings-policy-error-forbidden';
  else if (status === 404) id = 'settings-policy-error-not-found';
  else if (status === 409) id = 'settings-policy-error-conflict';
  else if (status === 413) id = 'settings-policy-error-too-large';
  else if (status === 429) id = 'settings-policy-error-rate';
  const text = t(id);
  return requestId !== null ? `${text} ${t('settings-policy-reference', { id: requestId })}` : text;
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    // Read the server's code and text without throwing; show neither as such.
    let detail = `request failed with ${res.status}`;
    let code: string | null = null;
    try {
      const body = (await res.json()) as { error?: unknown; code?: unknown };
      if (typeof body.error === 'string' && body.error !== '') detail = body.error;
      if (typeof body.code === 'string' && body.code !== '') code = body.code;
    } catch {
      /* non-JSON body — no detail */
    }
    const requestId = res.headers.get('x-request-id');
    throw new SettingsError(res.status, settingsErrorText(res.status, code, requestId), {
      code,
      detail,
      requestId,
    });
  }
  return (await res.json()) as T;
}

async function okOrThrow(res: Response): Promise<void> {
  await jsonOrThrow<unknown>(res);
}

function postJson(fetcher: Fetcher, url: string, body?: unknown): Promise<Response> {
  const init: RequestInit = { method: 'POST' };
  if (body !== undefined) {
    init.headers = { 'content-type': 'application/json' };
    init.body = JSON.stringify(body);
  }
  return fetcher(url, init);
}

export class SettingsService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  // ── 2FA management (S1) ──────────────────────────────────────────────────

  async twofaStatus(): Promise<TwofaStatus> {
    return jsonOrThrow<TwofaStatus>(await this.fetcher(withBase('/api/account/2fa')));
  }

  async totpBegin(): Promise<TotpBegin> {
    return jsonOrThrow<TotpBegin>(await postJson(this.fetcher, withBase('/api/account/2fa/totp/begin')));
  }

  /** Confirm TOTP enrolment; returns recovery codes to show ONCE (empty if the
   *  account already had a set). Throws on a wrong code (400). */
  async totpConfirm(code: string): Promise<RecoveryCodes> {
    return jsonOrThrow<RecoveryCodes>(await postJson(this.fetcher, withBase('/api/account/2fa/totp/confirm'), { code }));
  }

  async totpDisable(): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, withBase('/api/account/2fa/totp/disable')));
  }

  async passkeyBegin(): Promise<PasskeyRegistrationChallenge> {
    return jsonOrThrow<PasskeyRegistrationChallenge>(
      await postJson(this.fetcher, withBase('/api/account/2fa/passkey/begin')),
    );
  }

  /** Finish passkey enrolment; returns recovery codes to show ONCE (empty if a
   *  set already existed). */
  async passkeyFinish(body: {
    clientDataJson: string;
    attestationObject: string;
    transports: string;
    label: string;
  }): Promise<RecoveryCodes> {
    return jsonOrThrow<RecoveryCodes>(await postJson(this.fetcher, withBase('/api/account/2fa/passkey/finish'), body));
  }

  async passkeyRemove(handle: string): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, withBase('/api/account/2fa/passkey/remove'), { handle }));
  }

  async recoveryRegenerate(): Promise<RecoveryCodes> {
    return jsonOrThrow<RecoveryCodes>(await postJson(this.fetcher, withBase('/api/account/2fa/recovery/regenerate')));
  }

  // ── Login-time second factor (S1 web half; pre-auth pending token) ────────

  /** Present a second factor for a pending login. Resolves on success, throws a
   *  `SettingsError(401)` on a wrong/absent factor (uniform server 401). */
  async verifyLoginFactor(body: {
    pendingToken: string;
    method: 'totp' | 'recovery' | 'webauthn';
    code?: string;
    credentialId?: string;
    clientDataJson?: string;
    authenticatorData?: string;
    signature?: string;
  }): Promise<void> {
    try {
      await okOrThrow(await postJson(this.fetcher, withBase('/api/login/2fa'), body));
    } catch (e) {
      // A refused factor gets the one uniform text: never which check failed.
      if (e instanceof SettingsError && (e.status === 401 || e.status === 410)) {
        throw new SettingsError(e.status, t('settings-2fa-challenge-failed'), { detail: e.detail });
      }
      throw e;
    }
  }

  // ── Sessions (S11) ───────────────────────────────────────────────────────

  async sessions(): Promise<SessionMeta[]> {
    const out = await jsonOrThrow<{ sessions: SessionMeta[] }>(await this.sessionFetch(withBase('/api/account/sessions')));
    return out.sessions;
  }

  /** Revoke one session by handle. */
  async revokeSession(handle: string): Promise<void> {
    await okOrThrow(await postJson(this.sessionFetch.bind(this), withBase('/api/account/sessions/revoke'), { handle }));
  }

  /** Sign out everywhere else (revoke every session but the current one). */
  async revokeOtherSessions(): Promise<void> {
    await okOrThrow(await postJson(this.sessionFetch.bind(this), withBase('/api/account/sessions/revoke'), { all: true }));
  }

  private async sessionFetch(input:string,init?:RequestInit):Promise<Response>{
    const account=preferenceAccount();const controller=new AbortController();const headers=new Headers(init?.headers);
    if(account)headers.set('X-Mailwoman-Account',account.id);
    const unsubscribe=observePreferenceAccount(value=>{if(value!==account)controller.abort();});
    try{
      const response=await this.fetcher(input,{...init,headers,signal:controller.signal});
      if(controller.signal.aborted)throw new DOMException('Authentication changed','AbortError');
      return response;
    }finally{unsubscribe();}
  }

  private async signatureFetch(input:string,init?:RequestInit):Promise<Response> {
    const account=preferenceAccount();
    const headers=new Headers(init?.headers);
    if(account)headers.set('X-Mailwoman-Account',account.id);
    const response=await this.fetcher(input,{...init,headers});
    if(account&&!preferenceAccountIsCurrent(account))throw new SettingsError(403,t('settings-sig-account-changed'));
    return response;
  }

  // ── Signatures (W12) ─────────────────────────────────────────────────────

  async listSignatures(): Promise<Signature[]> {
    const out = await jsonOrThrow<{ signatures: Signature[] }>(await this.signatureFetch.bind(this)(withBase('/api/account/signatures')));
    return out.signatures;
  }

  async upsertSignature(sig: Signature): Promise<void> {
    await okOrThrow(await postJson(this.signatureFetch.bind(this), withBase('/api/account/signatures'), sig));
  }

  async deleteSignature(name: string): Promise<void> {
    await okOrThrow(
      await this.signatureFetch.bind(this)(withBase(`/api/account/signatures/${encodeURIComponent(name)}`), { method: 'DELETE' }),
    );
  }

  // ── Identities ───────────────────────────────────────────────────────────

  async listIdentities(): Promise<Identity[]> {
    const out = await jsonOrThrow<{ identities: Identity[] }>(await this.fetcher(withBase('/api/account/identities')));
    return out.identities;
  }

  async upsertIdentity(identity: Identity): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, withBase('/api/account/identities'), identity));
  }

  async deleteIdentity(id: string): Promise<void> {
    await okOrThrow(
      await this.fetcher(withBase(`/api/account/identities/${encodeURIComponent(id)}`), { method: 'DELETE' }),
    );
  }

  // ── Notification rules + quiet hours (W15) ───────────────────────────────

  async notifications(): Promise<NotificationConfig> {
    return jsonOrThrow<NotificationConfig>(await this.fetcher(withBase('/api/account/notifications')));
  }

  async saveNotifications(config: NotificationConfig): Promise<void> {
    await okOrThrow(
      await this.fetcher(withBase('/api/account/notifications'), {
        method: 'PUT',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(config),
      }),
    );
  }

  // ── Saved searches → search folders (W13) ────────────────────────────────

  async listSavedSearches(): Promise<SavedSearch[]> {
    const out = await jsonOrThrow<{ savedSearches: SavedSearch[] }>(
      await this.fetcher(withBase('/api/account/saved-searches')),
    );
    return out.savedSearches;
  }

  async upsertSavedSearch(search: SavedSearch): Promise<void> {
    await okOrThrow(
      await this.fetcher(withBase('/api/account/saved-searches'), {
        method: 'PUT',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(search),
      }),
    );
  }

  async deleteSavedSearch(id: string): Promise<void> {
    await okOrThrow(
      await this.fetcher(withBase(`/api/account/saved-searches/${encodeURIComponent(id)}`), { method: 'DELETE' }),
    );
  }
}
