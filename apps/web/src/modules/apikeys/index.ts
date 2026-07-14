// V6 scoped API-key / OAuth-consent / MCP-key module (SPEC §20.1/§20.3, plan §2.6 /
// §3 e8). Public surface for e11 to mount into Settings/an account screen (this module
// does NOT touch the router or Settings.tsx — ownership boundary). Mirrors the frozen
// `mw-oauth::Scope` shape (§2.3); `scopeToWire` serializes to the server's serde JSON.
//
// e11 WIRE-UP:
//   import { ApiKeys, McpKeys } from '@/modules/apikeys';
//   <ApiKeys accountId={me.accountId} />
//   <McpKeys accountId={me.accountId} />
// Endpoints these components call (e11 to satisfy):
//   GET  /api/keys                     → ApiKeyRecord[]
//   POST /api/keys      (CreateKeyBody)→ MintedKey (display token shown once)
//   POST /api/keys/:prefix/revoke
// The OAuth consent screen lives at `screens/Consent` (separate file per ownership).

export { ApiKeys } from './ApiKeys.tsx';
export { McpKeys, withTool } from './McpKeys.tsx';
export { ScopeBuilder, summarizeScope, toggleSubset } from './ScopeBuilder.tsx';
export { ApiKeyService, type Fetcher, type CreateKeyRequest } from './service.ts';
export {
  readOnlyScope,
  scopeToWire,
  scopeFromWire,
  MCP_TOOLS,
  UNATTENDED_SEND_DISCLOSURE,
  type ScopeSelector,
  type ApiKeyScope,
  type WireScope,
  type WireScopeSelector,
  type ApiKeyRecord,
  type MintedKey,
  type McpTool,
} from './types.ts';
