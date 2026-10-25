// Scoped API-key / OAuth / MCP-key types (SPEC §20.1/§20.3, plan §2.3/§2.6, §3 e8).
// The UI model (`ApiKeyScope`, camelCase) mirrors `mw-oauth::Scope`; `scopeToWire`
// serializes it to the FROZEN `mw-oauth` serde JSON shape the server consumes (e3/e11).

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
  readonly scripts: boolean;
  readonly ipAllowlist: readonly string[];
  readonly expiresAt: string | null;
  readonly rateLimit: number | null;
  readonly mcpTools: readonly string[];
  readonly unattendedSend: boolean;
}

/** The safest default scope (read-only, single account, mail-only). */
export function readOnlyScope(accountId: string): ApiKeyScope {
  return {
    read: true,
    send: false,
    delete: false,
    accounts: { kind: 'subset', ids: [accountId] },
    folders: { kind: 'all' },
    mail: true,
    pim: false,
    scripts: false,
    ipAllowlist: [],
    expiresAt: null,
    rateLimit: null,
    mcpTools: [],
    unattendedSend: false,
  };
}

/** The FROZEN `mw-oauth::ScopeSelector` serde JSON (kebab-case externally-tagged enum). */
export type WireScopeSelector = 'all' | { subset: string[] };

/** The FROZEN `mw-oauth::Scope` serde JSON (snake_case fields). */
export interface WireScope {
  read: boolean;
  send: boolean;
  delete: boolean;
  accounts: WireScopeSelector;
  folders: WireScopeSelector;
  mail: boolean;
  pim: boolean;
  scripts: boolean;
  ip_allowlist: string[];
  expires_at: string | null;
  rate_limit: number | null;
  mcp_tools: string[];
  unattended_send: boolean;
}

function selectorToWire(sel: ScopeSelector): WireScopeSelector {
  return sel.kind === 'all' ? 'all' : { subset: [...sel.ids] };
}

/** Serialize the UI scope into the `mw-oauth` wire shape the server enforces. */
export function scopeToWire(scope: ApiKeyScope): WireScope {
  return {
    read: scope.read,
    send: scope.send,
    delete: scope.delete,
    accounts: selectorToWire(scope.accounts),
    folders: selectorToWire(scope.folders),
    mail: scope.mail,
    pim: scope.pim,
    scripts: scope.scripts,
    ip_allowlist: [...scope.ipAllowlist],
    expires_at: scope.expiresAt,
    rate_limit: scope.rateLimit,
    mcp_tools: [...scope.mcpTools],
    unattended_send: scope.unattendedSend,
  };
}

function selectorFromWire(sel: WireScopeSelector): ScopeSelector {
  return sel === 'all' ? { kind: 'all' } : { kind: 'subset', ids: [...sel.subset] };
}

/** Parse a server-provided `mw-oauth` wire scope back into the UI model (for consent). */
export function scopeFromWire(wire: WireScope): ApiKeyScope {
  return {
    read: wire.read,
    send: wire.send,
    delete: wire.delete,
    accounts: selectorFromWire(wire.accounts),
    folders: selectorFromWire(wire.folders),
    mail: wire.mail,
    pim: wire.pim,
    scripts: wire.scripts ?? false,
    ipAllowlist: [...wire.ip_allowlist],
    expiresAt: wire.expires_at,
    rateLimit: wire.rate_limit,
    mcpTools: [...wire.mcp_tools],
    unattendedSend: wire.unattended_send,
  };
}

/** A stored (never-secret) API-key record listed in the UI (mirrors `ApiKey`, minus hash). */
export interface ApiKeyRecord {
  readonly prefix: string;
  readonly label: string;
  readonly accountId: string;
  readonly scope: ApiKeyScope;
  readonly createdAt: string;
  readonly lastUsedAt: string | null;
  readonly revokedAt: string | null;
  /** Whether an admin countersigned this key's `unattended_send` (see §2.4). */
  readonly unattendedSendApproved: boolean;
}

/** The shown-ONCE mint result (`mwk_<prefix>.<secret>` display token). */
export interface MintedKey {
  readonly displayToken: string;
  readonly record: ApiKeyRecord;
}

/**
 * The MCP tool ids grantable per key (SPEC §20.3 / plan §2.4). The checkbox text
 * lives in apikeys.ftl as `apikeys-tool-<id>-label` / `-desc` (dots become dashes).
 */
export interface McpTool {
  readonly id: string;
  /** Whether granting this tool implies the mail-send path (Outbox-gated). */
  readonly sends: boolean;
}

/** The frozen §2.4 MCP tool set; the mail tool descriptions say results are untrusted input. */
export const MCP_TOOLS: readonly McpTool[] = [
  { id: 'mail.search', sends: false },
  { id: 'mail.read', sends: false },
  { id: 'folders.list', sends: false },
  { id: 'drafts.create', sends: false },
  { id: 'mail.send', sends: true },
  { id: 'calendar.read', sends: false },
  { id: 'calendar.propose', sends: false },
  { id: 'tasks.read', sends: false },
  { id: 'tasks.write', sends: false },
  { id: 'contacts.read', sends: false },
];

/** The honest `unattended_send` disclosure copy (plan §2.4 / R4 — safety-critical). */
export const UNATTENDED_SEND_DISCLOSURE =
  'By default a granted send lands in your Outbox and waits for you to confirm it in the app — ' +
  'automation cannot send on its own. Unattended send REMOVES that human-in-the-loop step so this ' +
  'key can send mail without confirmation. It additionally requires an administrator to countersign ' +
  'the key. Grant it only to automation you fully trust; a compromised unattended-send key can send ' +
  'mail as you with no prompt.';
