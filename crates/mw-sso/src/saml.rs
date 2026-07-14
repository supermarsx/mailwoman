//! SAML 2.0 SP login backend — **stub** (t9-e2 implements). Frozen shape + the
//! HTTP-Redirect DEFLATE primitive only.
//!
//! e2 hand-rolls the bounded pure-Rust SP (plan §5): SP metadata,
//! AuthnRequest build + DEFLATE + base64 for the HTTP-Redirect binding
//! ([`begin`](crate::SsoLogin::begin)), and HTTP-POST ACS parsing + XML-DSig
//! validation (exclusive-C14N, RSA-SHA256/ECDSA-SHA256 over `SignedInfo`) +
//! Conditions/audience/`InResponseTo` checks + NameID/attribute extraction
//! ([`complete`](crate::SsoLogin::complete)) over in-tree RustCrypto + quick-xml.
//!
//! e0 lands the DEFLATE codec the HTTP-Redirect binding needs ([`deflate`] /
//! [`inflate`]) — a real, tested use of the vetted pure-Rust `flate2`
//! (miniz_oxide) tree, not a phantom dep.

use std::io::Write;

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

use crate::{
    BeginRedirect, Metadata, Redirect, SamlConfig, SsoBackend, SsoCallback, SsoConfig, SsoError,
    SsoIdentity, SsoLogin,
};

/// A SAML 2.0 Service-Provider backend. Construction validates the config; the
/// AuthnRequest/ACS crypto lands with e2.
#[derive(Debug, Clone)]
pub struct SamlProvider {
    backend: SsoBackend,
}

impl SamlProvider {
    /// Build a provider from a resolved [`SsoBackend`]. Errors if the backend is not
    /// SAML.
    pub fn new(backend: SsoBackend) -> Result<Self, SsoError> {
        Self::saml_config(&backend)?;
        Ok(SamlProvider { backend })
    }

    /// The resolved backend.
    pub fn backend(&self) -> &SsoBackend {
        &self.backend
    }

    fn saml_config(backend: &SsoBackend) -> Result<&SamlConfig, SsoError> {
        match &backend.config {
            SsoConfig::Saml(c) => Ok(c),
            SsoConfig::Oidc(_) => Err(SsoError::Config(
                "SamlProvider built from an OIDC backend".into(),
            )),
        }
    }
}

/// Raw-DEFLATE compress (RFC 1951, no zlib header) for the SAML HTTP-Redirect
/// binding — the `SAMLRequest`/`SAMLResponse` are DEFLATEd then base64'd. Pure-Rust
/// `flate2` (miniz_oxide) backend.
pub fn deflate(xml: &[u8]) -> Result<Vec<u8>, SsoError> {
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::fast());
    enc.write_all(xml)
        .map_err(|e| SsoError::Config(format!("deflate: {e}")))?;
    enc.finish()
        .map_err(|e| SsoError::Config(format!("deflate: {e}")))
}

/// Raw-DEFLATE decompress — the inverse of [`deflate`], for decoding an inbound
/// HTTP-Redirect `SAMLResponse` (e2's SLO/redirect path).
pub fn inflate(bytes: &[u8]) -> Result<Vec<u8>, SsoError> {
    use std::io::Read;
    let mut out = Vec::new();
    DeflateDecoder::new(bytes)
        .read_to_end(&mut out)
        .map_err(|e| SsoError::TokenValidation(format!("inflate: {e}")))?;
    Ok(out)
}

#[async_trait::async_trait]
impl SsoLogin for SamlProvider {
    async fn begin(&self, _relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        unimplemented!("t9-e2: SAML AuthnRequest build + DEFLATE + HTTP-Redirect")
    }

    async fn complete(&self, _callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        unimplemented!("t9-e2: SAML ACS parse + XML-DSig validation + assertion checks")
    }

    fn metadata(&self) -> Option<Metadata> {
        // e2: emit SP metadata XML from the config (entity id, ACS, SLO, certs).
        None
    }

    fn logout(&self, _subject: &str) -> Option<Redirect> {
        // e2: SP-initiated SLO to idp_slo_url.
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClaimMap, SsoKind, SsoScope};

    fn saml_backend() -> SsoBackend {
        SsoBackend {
            id: "corp-saml".into(),
            kind: SsoKind::Saml,
            display_name: "Acme SAML".into(),
            scope: SsoScope::Domain("acme.test".into()),
            enabled: true,
            config: SsoConfig::Saml(SamlConfig {
                sp_entity_id: "https://mail.example/sp".into(),
                idp_sso_url: "https://idp.example/sso".into(),
                want_assertions_signed: true,
                ..Default::default()
            }),
            claim_map: ClaimMap::default(),
            secret: None,
        }
    }

    #[test]
    fn builds_from_valid_saml_backend() {
        let p = SamlProvider::new(saml_backend()).unwrap();
        assert_eq!(p.backend().id, "corp-saml");
    }

    #[test]
    fn rejects_oidc_backend() {
        let mut b = saml_backend();
        b.config = SsoConfig::Oidc(Default::default());
        assert!(matches!(SamlProvider::new(b), Err(SsoError::Config(_))));
    }

    #[test]
    fn deflate_inflate_round_trips() {
        let xml = b"<samlp:AuthnRequest xmlns:samlp=\"urn:oasis:names:tc:SAML:2.0:protocol\"/>";
        let compressed = deflate(xml).unwrap();
        // Raw DEFLATE has no zlib header; round-trip must reproduce the bytes.
        assert_eq!(inflate(&compressed).unwrap(), xml);
        assert_ne!(compressed, xml, "should actually compress/transform");
    }
}
