// SSO wire DTOs (t9 e4 web) — the `/api/sso/*` (public, login) + `/admin/sso`
// (admin CRUD) JSON contract the web consumes, coding to e0's frozen `mw-sso`
// shapes (`.orchestration/logs/t9-e0.md` §1/§3). Field casing follows the
// house admin convention (camelCase), NOT Rust snake_case. The server side of the
// `/admin/sso` contract is `crates/mw-sso/src/admin_wire.rs`; both sides test
// against the shared fixtures in `crates/mw-sso/tests/fixtures/admin-sso`.

/** IdP protocol (e0 `SsoKind`; serde `'oidc' | 'saml'`). */
export type SsoKind = 'oidc' | 'saml';

/** First-login policy (e0 `FirstLoginPolicy`). DEFAULT = `allowlist` (deny;
 *  no open auto-registration). `autocreate` provisions an unknown subject. */
export type FirstLoginPolicy = 'allowlist' | 'autocreate';

/** IdP-claim → Mailwoman-account attribute names (e0 `ClaimMap`). */
export interface ClaimMap {
  email: string;
  username: string;
  display: string;
  /** Optional groups claim/attribute name. */
  groups?: string | null;
}

/** OIDC backend config (e0 `OidcConfig`). The `clientSecret` is WRITE-ONLY —
 *  never returned by `GET /admin/sso` (sealed at the store), only sent on save. */
export interface OidcConfig {
  kind: 'oidc';
  issuerUrl: string;
  clientId: string;
  redirectUrl: string;
  scopes: string[];
  firstLoginPolicy: FirstLoginPolicy;
  /** Refuse sign-ins whose email is not `email_verified: true` (server default true). */
  requireVerifiedEmail?: boolean;
}

/** SAML 2.0 backend config (e0 `SamlConfig`). */
export interface SamlConfig {
  kind: 'saml';
  spEntityId: string;
  acsUrl: string;
  idpMetadataUrl?: string | null;
  idpMetadataXml?: string | null;
  idpSsoUrl: string;
  idpSloUrl?: string | null;
  idpSigningCertsPem: string[];
  wantAssertionsSigned: boolean;
  wantEncrypted: boolean;
  nameidFormat: string;
  firstLoginPolicy: FirstLoginPolicy;
}

/** The kind-tagged config union (e0 `SsoConfig`, serde `tag = "kind"`). */
export type SsoConfig = OidcConfig | SamlConfig;

/**
 * A configured backend as returned by `GET /admin/sso` (e0 `SsoConfigRow`).
 * `scope` is `'deployment'` or `'domain:<d>'` (e0 `SsoScope::as_db`). No secret.
 */
export interface SsoBackendRow {
  id: string;
  displayName: string;
  scope: string;
  enabled: boolean;
  config: SsoConfig;
  claimMap: ClaimMap;
  /** `GET` only: a secret (OIDC client secret / SAML SP key) is stored. */
  hasSecret?: boolean;
  /** `GET` only (RFC 3339). */
  createdAt?: string;
  updatedAt?: string;
  /** `GET` only: what is wrong with the stored config, by wire field name
   *  (`config.issuerUrl`, ...); empty when nothing is. */
  problems?: SsoFieldProblem[];
  /** `GET` only: whether sign-in builds this provider at all. False for an unsafe
   *  or incomplete config, which is never offered. Absent from older servers. */
  signInReady?: boolean;
}

/** One problem the server found with a config (admin_wire `FieldError`). */
export interface SsoFieldProblem {
  /** Wire path, e.g. `config.issuerUrl`. */
  field: string;
  /** Stable code: `required`, `invalid_url`, `issuer_query`, `openid_scope`,
   *  `cert_required`, `unsigned_assertions`, `id_pattern`, `scope_invalid`, ... */
  code?: string;
  /** English detail from the server. */
  message: string;
}

/**
 * The upsert body for `POST /admin/sso`. Carries the optional WRITE-ONLY
 * `secret` (OIDC client secret / SAML SP private key material). Omitted/empty
 * on an edit leaves the stored secret unchanged.
 */
export interface SsoBackendInput extends SsoBackendRow {
  secret?: string | null;
}

/**
 * A public, enabled IdP as advertised pre-auth by `GET /api/sso/providers`
 * (kind + id + display name only — no config leaks before login).
 */
export interface SsoProviderSummary {
  id: string;
  kind: SsoKind;
  displayName: string;
}
