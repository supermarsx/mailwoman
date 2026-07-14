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

/** The MCP tool ids grantable per key (SPEC §20.3 / plan §2.4). */
export interface McpTool {
  readonly id: string;
  readonly label: string;
  readonly description: string;
  /** Whether granting this tool implies the mail-send path (Outbox-gated). */
  readonly sends: boolean;
}

/** The frozen §2.4 MCP tool set, with mail marked as untrusted input at the source. */
export const MCP_TOOLS: readonly McpTool[] = [
  { id: 'mail.search', label: 'Search mail', description: 'Search messages. Results are labelled untrusted input.', sends: false },
  { id: 'mail.read', label: 'Read mail', description: 'Read a message body. Bodies are labelled untrusted input.', sends: false },
  { id: 'folders.list', label: 'List folders', description: 'List mailbox folders.', sends: false },
  { id: 'drafts.create', label: 'Create drafts', description: 'Create a draft message (never sent automatically).', sends: false },
  { id: 'mail.send', label: 'Send mail', description: 'Queue a message to the Outbox for in-app confirmation.', sends: true },
  { id: 'calendar.read', label: 'Read calendar', description: 'Read calendar events.', sends: false },
  { id: 'calendar.propose', label: 'Propose events', description: 'Propose (not commit) calendar events.', sends: false },
  { id: 'tasks.read', label: 'Read tasks', description: 'Read tasks.', sends: false },
  { id: 'tasks.write', label: 'Write tasks', description: 'Create or update tasks.', sends: false },
  { id: 'contacts.read', label: 'Read contacts', description: 'Read contacts.', sends: false },
];

/** The honest `unattended_send` disclosure copy (plan §2.4 / R4 — safety-critical). */
export const UNATTENDED_SEND_DISCLOSURE =
  'By default a granted send lands in your Outbox and waits for you to confirm it in the app — ' +
  'automation cannot send on its own. Unattended send REMOVES that human-in-the-loop step so this ' +
  'key can send mail without confirmation. It additionally requires an administrator to countersign ' +
  'the key. Grant it only to automation you fully trust; a compromised unattended-send key can send ' +
  'mail as you with no prompt.';
