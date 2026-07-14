// Public barrel for the web SSO module (t9 e4). Login + the admin SSO panel
// import from here, not the internal files.

export {
  listSsoProviders,
  ssoBeginPath,
  ssoMetadataPath,
  createHttpSsoAdminApi,
  SsoAdminError,
  type SsoAdminApi,
} from './client.ts';
export type {
  SsoKind,
  FirstLoginPolicy,
  ClaimMap,
  OidcConfig,
  SamlConfig,
  SsoConfig,
  SsoBackendRow,
  SsoBackendInput,
  SsoProviderSummary,
} from './types.ts';
