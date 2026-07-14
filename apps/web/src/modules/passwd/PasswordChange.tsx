// V7 in-app password change (SPEC §18.3, plan §3 e7). SCAFFOLD stub (e0): inert
// placeholder. e7 builds the change form + policy display + zero-access re-wrap
// ceremony (offering the recovery-key path FIRST); e14 wires it to /api/password.

import type { JSX } from 'solid-js';

export interface PasswordChangeProps {
  accountId?: string;
}

export function PasswordChange(_props: PasswordChangeProps): JSX.Element {
  return <div data-module="passwd">Password change not yet implemented (t7 e7).</div>;
}
