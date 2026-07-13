// V6 scoped API-key / OAuth-consent / MCP-key module (SPEC §20.1/§20.3, plan
// §2.6, §3 e8). SCAFFOLD (t6-e0): inert placeholder types — importable +
// typecheck-green, NOT wired into any route yet. e8 fills create/list/revoke of
// scoped API keys (shown once), the OAuth consent + client-approval UX, and
// MCP-key management (per-tool grants, `unattended-send` disclosure), talking to
// the e3/e11 endpoints. Mirrors the `mw-oauth::Scope` shape (§2.3).

/** Selects the accounts/folders a scope applies to (mirrors `ScopeSelector`). */
export type ScopeSelector = { readonly kind: 'all' } | { readonly kind: 'subset'; readonly ids: readonly string[] };

/** The typed capability set shown in the key-create + consent UIs (mirrors `Scope`, §2.3). */
export interface ApiKeyScope {
  readonly read: boolean;
  readonly send: boolean;
  readonly delete: boolean;
  readonly accounts: ScopeSelector;
  readonly folders: ScopeSelector;
  readonly mail: boolean;
  readonly pim: boolean;
  readonly ipAllowlist: readonly string[];
  readonly expiresAt: string | null;
  readonly rateLimit: number | null;
  readonly mcpTools: readonly string[];
  readonly unattendedSend: boolean;
}

/** The safest default scope (read-only, single account, mail-only). e8 replaces this module. */
export function readOnlyScope(accountId: string): ApiKeyScope {
  return {
    read: true,
    send: false,
    delete: false,
    accounts: { kind: 'subset', ids: [accountId] },
    folders: { kind: 'all' },
    mail: true,
    pim: false,
    ipAllowlist: [],
    expiresAt: null,
    rateLimit: null,
    mcpTools: [],
    unattendedSend: false,
  };
}
