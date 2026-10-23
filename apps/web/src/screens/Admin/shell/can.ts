// Capability-aware console (t32-ext.md §M).
//
// The shell reads `GET /admin/me` once after sign-in and hides sections, and
// disables actions, that the signed-in admin's roles do not include. This is
// presentation only: every `/admin` route checks the capability on the server.
//
// Until RBAC lands (stage A2, t32-e24) the server has no `/admin/me` and answers
// 404. The only admin then is the environment credential, which may do
// everything, so a 404 means "all capabilities" and the console says nothing
// about roles. Any other failure (network, 5xx) also falls back to showing
// everything: hiding the console would lock the admin out of a screen the server
// may well allow, and the server refuses what it does not allow.

import { createSignal, type Accessor } from 'solid-js';
import { basePath } from '../../../api/basePath.ts';
import { apiFetch } from '../../../api/http.ts';
import { t } from '../../../i18n';

/** The capability catalogue (t32-ext.md §M, plus the R–V additions). */
export const CAPABILITIES = [
  'dashboard.read',
  'users.read',
  'users.write',
  'users.revoke_sessions',
  'users.flags',
  'users.quota',
  'users.capabilities',
  'domains.read',
  'domains.write',
  'domains.delete',
  'tunnels.read',
  'tunnels.write',
  'tunnels.test',
  'security.read',
  'security.write',
  'security_policy.write',
  'twofa_policy.write',
  'sso.read',
  'sso.write',
  'settings.read',
  'settings.write',
  'settings.user_values',
  'branding.read',
  'branding.manage',
  'user_css.review',
  'ai_assistant.read',
  'ai_assistant.write',
  'ai_assistant.kill',
  'integrations.read',
  'api_keys.revoke',
  'oauth_clients.write',
  'plugins.read',
  'plugins.write',
  'ui_plugins.write',
  'egress.read',
  'egress.write',
  'maintenance.run',
  'server_metadata.write',
  'diagnostics.read',
  'diagnostics.run',
  'diagnostics.bundle',
  'logs.read',
  'logs.download',
  'logs.level',
  'audit.read',
  'audit.export',
  'admins.read',
  'admins.manage',
  'roles.manage',
  'privacy.read',
  'privacy.write',
  'sieve.policy',
  'pgp.policy',
  'imap.extensions',
  'appearance.manage',
  'themes.edit',
] as const;

export type Capability = (typeof CAPABILITIES)[number];

/** `GET /admin/me` (t32-ext.md §M). */
export interface AdminMe {
  username: string;
  displayName: string | null;
  roles: string[];
  capabilities: string[];
  /** `null` = all domains. */
  domainScope: string[] | null;
  totpEnrolled: boolean;
}

/** What the console knows about the signed-in admin's grants. */
export type AdminGrants =
  | { kind: 'all'; reason: 'no-rbac' | 'unreadable' }
  | { kind: 'listed'; me: AdminMe };

export type AdminMeLoader = () => Promise<AdminGrants>;

/** The production loader: `GET <base>/admin/me`. */
export function createAdminMeLoader(base = basePath()): AdminMeLoader {
  return async () => {
    try {
      const res = await apiFetch(`${base}/admin/me`, { credentials: 'same-origin' });
      if (res.status === 404) return { kind: 'all', reason: 'no-rbac' };
      if (!res.ok) return { kind: 'all', reason: 'unreadable' };
      const me = (await res.json()) as AdminMe;
      if (!Array.isArray(me.capabilities)) return { kind: 'all', reason: 'unreadable' };
      return { kind: 'listed', me };
    } catch {
      return { kind: 'all', reason: 'unreadable' };
    }
  };
}

export interface Can {
  /** The grants as last loaded. */
  grants: Accessor<AdminGrants>;
  /** May the admin use `cap`? */
  can(cap: Capability): boolean;
  /** The sentence to show next to an action disabled for lack of `cap`, or null. */
  reason(cap: Capability): string | null;
}

/** Wrap a grants accessor in the `can` helpers. */
export function createCan(grants: Accessor<AdminGrants>): Can {
  function can(cap: Capability): boolean {
    const g = grants();
    return g.kind === 'all' || g.me.capabilities.includes(cap);
  }
  return {
    grants,
    can,
    reason: (cap) => (can(cap) ? null : t('admin-shell-cap-missing', { capability: cap })),
  };
}

/** A `Can` filled by `load()`. The shell waits for `loaded()` before drawing the
 *  navigation, so a restricted admin never sees a section flash and vanish. */
export function createLoadedCan(
  load: AdminMeLoader,
): Can & { loaded: Accessor<boolean>; reload(): Promise<void> } {
  const [grants, setGrants] = createSignal<AdminGrants>({ kind: 'all', reason: 'no-rbac' });
  const [loaded, setLoaded] = createSignal(false);
  return {
    ...createCan(grants),
    loaded,
    async reload() {
      setGrants(await load());
      setLoaded(true);
    },
  };
}
