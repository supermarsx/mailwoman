// Settings-surface server I/O (t16 e15). One service over an injectable `Fetcher`
// so every component unit-tests without a live server. Same-origin cookie auth,
// byte-identical to the passwd/apikeys services (SPEC §7.4/§19, W12/W13/W15).
//
// Contract sources:
//   • 2FA + sessions  → crates/mw-server/src/twofa_routes.rs (LIVE this milestone)
//   • signatures / notifications / saved-searches / identities → the `mw-store`
//     0017 + frozen-0003 rows. Those persist server-side; the HTTP surface here is
//     the account-prefs contract the web drives (see the DONE report's backend note).

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

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/** Raised on a non-2xx settings request. `status` lets callers special-case 409. */
export class SettingsError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'SettingsError';
    this.status = status;
  }
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    // Surface the server's error string when it sends one (e.g. the last-factor
    // 409), else a generic message. Never throws while reading the body.
    let detail = `request failed with ${res.status}`;
    try {
      const body = (await res.json()) as { error?: string };
      if (typeof body.error === 'string' && body.error !== '') detail = body.error;
    } catch {
      /* non-JSON body — keep the generic message */
    }
    throw new SettingsError(res.status, detail);
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
    return jsonOrThrow<TwofaStatus>(await this.fetcher('/api/account/2fa'));
  }

  async totpBegin(): Promise<TotpBegin> {
    return jsonOrThrow<TotpBegin>(await postJson(this.fetcher, '/api/account/2fa/totp/begin'));
  }

  /** Confirm TOTP enrolment; returns recovery codes to show ONCE (empty if the
   *  account already had a set). Throws on a wrong code (400). */
  async totpConfirm(code: string): Promise<RecoveryCodes> {
    return jsonOrThrow<RecoveryCodes>(await postJson(this.fetcher, '/api/account/2fa/totp/confirm', { code }));
  }

  async totpDisable(): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, '/api/account/2fa/totp/disable'));
  }

  async passkeyBegin(): Promise<PasskeyRegistrationChallenge> {
    return jsonOrThrow<PasskeyRegistrationChallenge>(
      await postJson(this.fetcher, '/api/account/2fa/passkey/begin'),
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
    return jsonOrThrow<RecoveryCodes>(await postJson(this.fetcher, '/api/account/2fa/passkey/finish', body));
  }

  async passkeyRemove(handle: string): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, '/api/account/2fa/passkey/remove', { handle }));
  }

  async recoveryRegenerate(): Promise<RecoveryCodes> {
    return jsonOrThrow<RecoveryCodes>(await postJson(this.fetcher, '/api/account/2fa/recovery/regenerate'));
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
    await okOrThrow(await postJson(this.fetcher, '/api/login/2fa', body));
  }

  // ── Sessions (S11) ───────────────────────────────────────────────────────

  async sessions(): Promise<SessionMeta[]> {
    const out = await jsonOrThrow<{ sessions: SessionMeta[] }>(await this.fetcher('/api/account/sessions'));
    return out.sessions;
  }

  /** Revoke one session by handle. */
  async revokeSession(handle: string): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, '/api/account/sessions/revoke', { handle }));
  }

  /** Sign out everywhere else (revoke every session but the current one). */
  async revokeOtherSessions(): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, '/api/account/sessions/revoke', { all: true }));
  }

  // ── Signatures (W12) ─────────────────────────────────────────────────────

  async listSignatures(): Promise<Signature[]> {
    const out = await jsonOrThrow<{ signatures: Signature[] }>(await this.fetcher('/api/account/signatures'));
    return out.signatures;
  }

  async upsertSignature(sig: Signature): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, '/api/account/signatures', sig));
  }

  async deleteSignature(name: string): Promise<void> {
    await okOrThrow(
      await this.fetcher(`/api/account/signatures/${encodeURIComponent(name)}`, { method: 'DELETE' }),
    );
  }

  // ── Identities ───────────────────────────────────────────────────────────

  async listIdentities(): Promise<Identity[]> {
    const out = await jsonOrThrow<{ identities: Identity[] }>(await this.fetcher('/api/account/identities'));
    return out.identities;
  }

  async upsertIdentity(identity: Identity): Promise<void> {
    await okOrThrow(await postJson(this.fetcher, '/api/account/identities', identity));
  }

  async deleteIdentity(id: string): Promise<void> {
    await okOrThrow(
      await this.fetcher(`/api/account/identities/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    );
  }

  // ── Notification rules + quiet hours (W15) ───────────────────────────────

  async notifications(): Promise<NotificationConfig> {
    return jsonOrThrow<NotificationConfig>(await this.fetcher('/api/account/notifications'));
  }

  async saveNotifications(config: NotificationConfig): Promise<void> {
    await okOrThrow(
      await this.fetcher('/api/account/notifications', {
        method: 'PUT',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(config),
      }),
    );
  }

  // ── Saved searches → search folders (W13) ────────────────────────────────

  async listSavedSearches(): Promise<SavedSearch[]> {
    const out = await jsonOrThrow<{ savedSearches: SavedSearch[] }>(
      await this.fetcher('/api/account/saved-searches'),
    );
    return out.savedSearches;
  }

  async upsertSavedSearch(search: SavedSearch): Promise<void> {
    await okOrThrow(
      await this.fetcher('/api/account/saved-searches', {
        method: 'PUT',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(search),
      }),
    );
  }

  async deleteSavedSearch(id: string): Promise<void> {
    await okOrThrow(
      await this.fetcher(`/api/account/saved-searches/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    );
  }
}
