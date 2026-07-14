//! OIDC login backend — **stub** (t9-e1 implements). Frozen shape only.
//!
//! e1 fills in discovery (`.well-known/openid-configuration`), the
//! authorization-code + PKCE [`begin`](crate::SsoLogin::begin), JWKS ID-token
//! validation + nonce + userinfo in [`complete`](crate::SsoLogin::complete),
//! RP-initiated logout, and the claim → [`SsoIdentity`] mapping — wiring
//! `openidconnect` to the in-tree rustls `reqwest` async client. JWS verification
//! is RustCrypto (`rsa`/`p256`/`p384`/`hmac`/`sha2`) — **no openssl**.
//!
//! The `openidconnect` link is real (see [`issuer_url`]) so e0's dependency VET
//! exercises the actual tree, not a phantom dep.

use openidconnect::IssuerUrl;

use crate::{
    BeginRedirect, Metadata, OidcConfig, Redirect, SsoBackend, SsoCallback, SsoConfig, SsoError,
    SsoIdentity, SsoLogin,
};

/// An OIDC relying-party backend. Construction validates the config into the
/// `openidconnect` types; the network-bound flow lands with e1.
#[derive(Debug, Clone)]
pub struct OidcProvider {
    backend: SsoBackend,
}

impl OidcProvider {
    /// Build a provider from a resolved [`SsoBackend`]. Errors if the backend is not
    /// OIDC or the issuer URL is malformed (the cheap validation e0 can do without
    /// the network; e1 adds discovery).
    pub fn new(backend: SsoBackend) -> Result<Self, SsoError> {
        let cfg = Self::oidc_config(&backend)?;
        // Validate the issuer eagerly so a misconfigured backend fails at build,
        // not mid-login. This also links `openidconnect` into the vetted tree.
        let _issuer = issuer_url(cfg)?;
        Ok(OidcProvider { backend })
    }

    /// The resolved backend.
    pub fn backend(&self) -> &SsoBackend {
        &self.backend
    }

    fn oidc_config(backend: &SsoBackend) -> Result<&OidcConfig, SsoError> {
        match &backend.config {
            SsoConfig::Oidc(c) => Ok(c),
            SsoConfig::Saml(_) => Err(SsoError::Config(
                "OidcProvider built from a SAML backend".into(),
            )),
        }
    }
}

/// Parse the configured issuer into an `openidconnect::IssuerUrl` (the discovery
/// anchor). A real, compiling use of the vetted `openidconnect` tree.
pub fn issuer_url(cfg: &OidcConfig) -> Result<IssuerUrl, SsoError> {
    IssuerUrl::new(cfg.issuer_url.clone())
        .map_err(|e| SsoError::Config(format!("bad issuer_url: {e}")))
}

#[async_trait::async_trait]
impl SsoLogin for OidcProvider {
    async fn begin(&self, _relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        unimplemented!("t9-e1: OIDC discovery + auth-code+PKCE begin()")
    }

    async fn complete(&self, _callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        unimplemented!("t9-e1: OIDC JWKS ID-token validation + nonce + userinfo")
    }

    fn metadata(&self) -> Option<Metadata> {
        // OIDC advertises via the IdP discovery doc, not SP metadata.
        None
    }

    fn logout(&self, _subject: &str) -> Option<Redirect> {
        // e1: RP-initiated logout (end_session_endpoint from discovery).
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClaimMap, SsoKind, SsoScope};

    fn oidc_backend(issuer: &str) -> SsoBackend {
        SsoBackend {
            id: "corp-oidc".into(),
            kind: SsoKind::Oidc,
            display_name: "Acme SSO".into(),
            scope: SsoScope::Deployment,
            enabled: true,
            config: SsoConfig::Oidc(OidcConfig {
                issuer_url: issuer.into(),
                client_id: "mailwoman".into(),
                redirect_url: "https://mail.example/api/sso/corp-oidc/callback".into(),
                scopes: vec!["openid".into(), "email".into()],
                ..Default::default()
            }),
            claim_map: ClaimMap::default(),
            secret: Some(b"client-secret".to_vec()),
        }
    }

    #[test]
    fn builds_from_valid_oidc_backend() {
        let p = OidcProvider::new(oidc_backend("https://idp.example")).unwrap();
        assert_eq!(p.backend().id, "corp-oidc");
    }

    #[test]
    fn rejects_bad_issuer_url() {
        assert!(matches!(
            OidcProvider::new(oidc_backend("not a url")),
            Err(SsoError::Config(_))
        ));
    }

    #[test]
    fn rejects_saml_backend() {
        let mut b = oidc_backend("https://idp.example");
        b.config = SsoConfig::Saml(Default::default());
        assert!(matches!(OidcProvider::new(b), Err(SsoError::Config(_))));
    }
}
