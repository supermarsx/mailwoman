// SSO wire DTOs (t9 e4 web) — the `/api/sso/*` (public, login) + `/admin/sso`
// (admin CRUD) JSON contract the web consumes, coding to e0's frozen `mw-sso`
// shapes (`.orchestration/logs/t9-e0.md` §1/§3). Field casing follows the
// house admin convention (serde `rename_all = "camelCase"`, like `Domain`,
// `ApiKeyInfo`), NOT Rust snake_case — e3 serialises the rows to camelCase.

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
