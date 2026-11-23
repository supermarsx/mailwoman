// Forced second-factor enrolment at sign-in (t47-e2).
//
// When an administrator requires two-factor authentication and the account has
// no factor yet, `/api/login` answers `twofaRequired` + `enrollmentRequired` with
// a one-shot pending token and no session. These calls enrol a factor over that
// token; the confirming call issues the session (cookie, or a native shell's
// bearer token, which `http.ts` keeps for every `/api/login/2fa*` answer) and the
// account's first recovery codes. Server: `crates/mw-server/src/twofa_routes.rs`
// (`login_enroll_totp_*`, `login_enroll_passkey_*`).
//
// Every refused call throws `ApiError`. The server answers an unknown or expired
// pending token and a wrong code with the same 401, so the caller cannot tell
// them apart from the status alone.

import { withBase } from './basePath.ts';
import { apiFetch } from './http.ts';
import { ApiError } from './errors.ts';
import { errorDetails } from './client.ts';
/** `begin` for an authenticator app: the secret to add, and its `otpauth://` URI. */
export interface LoginTotpSetup {
  readonly secret: string;
  readonly otpauthUri: string;
}

/** A passkey registration challenge: the shape `passkey_begin` answers with (the
 *  same one the settings enrolment gets, and `registerPasskey` takes). */
export interface LoginPasskeySetup {
  readonly challenge: string;
  readonly rpId: string;
  readonly userHandle: string;
  readonly userName: string;
  readonly userVerification: string;
  readonly residentKey?: 'discouraged' | 'preferred' | 'required';
  readonly authenticatorAttachment?: string | null;
  readonly excludeCredentialIds?: readonly string[];
}

/** What a completed enrolment returns: the recovery codes, shown once (may be empty). */
export interface LoginEnrolled {
  readonly recoveryCodes: readonly string[];
}

export interface LoginPasskeyFinish {
  readonly clientDataJson: string;
  readonly attestationObject: string;
  readonly transports: string;
  readonly label: string;
  readonly authenticatorAttachment?: string;
  readonly residentKey?: boolean;
}

export interface LoginEnrolmentApi {
  totpBegin(pendingToken: string): Promise<LoginTotpSetup>;
  totpConfirm(pendingToken: string, code: string): Promise<LoginEnrolled>;
  passkeyBegin(pendingToken: string): Promise<LoginPasskeySetup>;
  passkeyFinish(pendingToken: string, body: LoginPasskeyFinish): Promise<LoginEnrolled>;
}

async function post<T>(path: string, body: Record<string, unknown>): Promise<T> {
  const res = await apiFetch(withBase(path), {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new ApiError(res.status, `enrolment request failed with ${res.status}`, await errorDetails(res));
  return (await res.json()) as T;
}

function enrolled(body: { recoveryCodes?: unknown }): LoginEnrolled {
  const codes = Array.isArray(body.recoveryCodes) ? body.recoveryCodes.filter((c): c is string => typeof c === 'string') : [];
  return { recoveryCodes: codes };
}

export const loginEnrolmentApi: LoginEnrolmentApi = {
  totpBegin: (pendingToken) => post<LoginTotpSetup>('/api/login/2fa/enroll/totp/begin', { pendingToken }),
  totpConfirm: async (pendingToken, code) =>
    enrolled(await post<{ recoveryCodes?: unknown }>('/api/login/2fa/enroll/totp/confirm', { pendingToken, code })),
  passkeyBegin: (pendingToken) => post<LoginPasskeySetup>('/api/login/2fa/enroll/passkey/begin', { pendingToken }),
  passkeyFinish: async (pendingToken, body) =>
    enrolled(await post<{ recoveryCodes?: unknown }>('/api/login/2fa/enroll/passkey/finish', { pendingToken, ...body })),
};
