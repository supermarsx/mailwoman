// Password-change server I/O (SPEC §18.3, plan §3 e7). Talks to `/api/password[/policy]`
// (e9 fills, e14 mounts over `mw-passwd`). Transport injectable so the form unit-tests
// without a live server. The service ENFORCES the recovery-phrase-before-change ordering
// for zero-access accounts: `change()` refuses to POST a re-wrap unless the recovery
// phrase was acknowledged first (`rewrap` present ⇒ the caller went through the
// pre-prompt). No plaintext key ever leaves the client — only wrapped material.

import type { ZaKdfParams } from '../zeroaccess/crypto.ts';
import { t } from '../../i18n';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`password request failed: ${res.status}`);
  return (await res.json()) as T;
}

/** The password policy the backend advertises (`GET /api/password/policy`). */
export interface PasswordPolicy {
  readonly minLength: number;
  readonly requireUppercase: boolean;
  readonly requireLowercase: boolean;
  readonly requireDigit: boolean;
  readonly requireSymbol: boolean;
  /** The backend's `PasswordPolicy` Display string (human summary). */
  readonly description: string;
  /** Whether the account is under forced-change-on-next-login. */
  readonly forceChange: boolean;
}

/** The re-wrapped key material posted alongside a zero-access password change. */
export interface RewrapPayload {
  readonly saltB64: string;
  readonly kdfParams: ZaKdfParams;
  readonly wrappedDataKeyB64: string;
}

/** The change request body. `rewrap` is present only for zero-access accounts, and
 *  only after the recovery-phrase pre-prompt has been acknowledged. */
export interface PasswordChangeRequest {
  readonly oldPassword: string;
  readonly newPassword: string;
  readonly rewrap?: RewrapPayload;
}

/** The server outcome (mirrors `mw_passwd::PasswordChangeOutcome`). */
export interface PasswordChangeOutcome {
  readonly changed: boolean;
  readonly reencryptCredentials: boolean;
  readonly zeroaccessRewrapRequired: boolean;
}

export class PasswordService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  /** Fetch the backend's password policy + forced-change state. */
  async policy(): Promise<PasswordPolicy> {
    const res = await this.fetcher('/api/password/policy');
    return jsonOrThrow<PasswordPolicy>(res);
  }

  /** Apply a password change. For zero-access accounts pass the `rewrap` material. */
  async change(req: PasswordChangeRequest): Promise<PasswordChangeOutcome> {
    const res = await this.fetcher('/api/password', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(req),
    });
    return jsonOrThrow<PasswordChangeOutcome>(res);
  }
}

/** Validate a candidate password against a policy; returns the failing rules (empty ⇒ ok). */
export function policyViolations(policy: PasswordPolicy, candidate: string): string[] {
  const out: string[] = [];
  if (candidate.length < policy.minLength) out.push(t('passwd-rule-min-length', { count: policy.minLength }));
  if (policy.requireUppercase && !/[A-Z]/.test(candidate)) out.push(t('passwd-rule-uppercase'));
  if (policy.requireLowercase && !/[a-z]/.test(candidate)) out.push(t('passwd-rule-lowercase'));
  if (policy.requireDigit && !/[0-9]/.test(candidate)) out.push(t('passwd-rule-digit'));
  if (policy.requireSymbol && !/[^A-Za-z0-9]/.test(candidate)) out.push(t('passwd-rule-symbol'));
  return out;
}
