// V7 password-change module (SPEC §18.3, plan §2.6 / §3 e7). SCAFFOLD stub (e0):
// inert, lazily importable, typecheck-green, NOT routed. e7 fills the in-app
// password change + policy display + the zero-access key-hierarchy re-wrap flow
// (reusing the existing crypto worker); e14 wires it to /api/password.

/** Password policy the form displays before a change (mirrors `mw_passwd::PasswordPolicy`). */
export interface PasswordPolicy {
  readonly minLength: number;
  readonly requireUpper: boolean;
  readonly requireLower: boolean;
  readonly requireDigit: boolean;
  readonly requireSymbol: boolean;
  readonly description: string;
}

export { PasswordChange } from './PasswordChange.tsx';
