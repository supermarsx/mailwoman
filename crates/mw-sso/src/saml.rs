//! SAML 2.0 SP login backend — a bounded, hand-rolled pure-Rust Service Provider
//! (plan §2/§5). No `openssl`, no `libxml`, no `-sys` C: XML parsing rides the in-tree
//! `quick-xml`, canonicalization + XML-DSig are hand-rolled over in-tree RustCrypto
//! (`rsa`/`p256`/`sha2`), and the HTTP-Redirect codec is `flate2` (miniz_oxide).
//!
//! ## Bounded profile (plan §5)
//! * **SP-initiated SSO only** — no IdP-initiated unsolicited-response acceptance
//!   (a security-positive omission).
//! * Bindings: **HTTP-Redirect** for the `AuthnRequest` ([`authnrequest`]),
//!   **HTTP-POST** for the `SAMLResponse` at the ACS ([`acs`]).
//! * Signatures: **RSA-SHA256** and **ECDSA-SHA256** over **exclusive C14N**
//!   ([`c14n`]), enveloped-signature transform, SHA-256 reference digest, verified
//!   against the IdP signing cert **pinned in config** ([`dsig`]).
//! * Assertion: `Conditions` (audience = our SP entity ID, validity window with
//!   clock skew), `InResponseTo` binding, one-time replay cache, NameID + attribute
//!   extraction into an [`SsoIdentity`] via [`ClaimMap`] ([`assertion`]).
//! * `EncryptedAssertion`: a documented, config-visible **rejection** in 26.9 — never
//!   a silent pass.
//!
//! Every failure is a uniform-401 [`SsoError`]; no assertion content is logged.

use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Utc;
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

use crate::state::{PendingState, new_state_token};
use crate::{
    BeginRedirect, Metadata, Redirect, SamlConfig, SsoBackend, SsoCallback, SsoConfig, SsoError,
    SsoIdentity, SsoLogin,
};

pub mod acs;
pub mod assertion;
pub mod authnrequest;
pub mod c14n;
pub mod dsig;
pub mod metadata;

/// How long a consumed assertion `ID` is remembered for replay defence when the
/// assertion carries no `NotOnOrAfter`.
const REPLAY_FALLBACK_TTL: Duration = Duration::from_secs(600);

/// A SAML 2.0 Service-Provider backend implementing [`SsoLogin`].
#[derive(Debug, Clone)]
pub struct SamlProvider {
    backend: SsoBackend,
    /// One-time cache of consumed assertion IDs → expiry (replay defence, plan §5).
    seen: Arc<Mutex<HashMap<String, Instant>>>,
}

impl SamlProvider {
    /// Build a provider from a resolved [`SsoBackend`]. Errors if the backend is not
    /// SAML.
    pub fn new(backend: SsoBackend) -> Result<Self, SsoError> {
        Self::saml_config(&backend)?;
        Ok(SamlProvider {
            backend,
            seen: Arc::new(Mutex::new(HashMap::new())),
        })
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

    fn config(&self) -> &SamlConfig {
        match &self.backend.config {
            SsoConfig::Saml(c) => c,
            // unreachable: `new` rejects non-SAML backends.
            SsoConfig::Oidc(_) => unreachable!("SamlProvider always holds a SAML config"),
        }
    }

    /// Record a consumed assertion `ID`; returns `Err(Replay)` if already seen.
    fn remember(&self, assertion_id: &str, expiry: Instant) -> Result<(), SsoError> {
        let mut seen = self.seen.lock().expect("replay cache poisoned");
        let now = Instant::now();
        seen.retain(|_, exp| *exp > now);
        if seen.contains_key(assertion_id) {
            return Err(SsoError::Replay);
        }
        seen.insert(assertion_id.to_string(), expiry);
        Ok(())
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
    async fn begin(&self, relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        let cfg = self.config();
        if cfg.idp_sso_url.is_empty() {
            return Err(SsoError::Config("SAML idp_sso_url not configured".into()));
        }
        let request_id = format!("_{}", new_state_token());
        let state_token = new_state_token();
        let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let xml = authnrequest::build_xml(
            &request_id,
            &issue_instant,
            &cfg.sp_entity_id,
            &cfg.acs_url,
            &cfg.idp_sso_url,
            &cfg.nameid_format,
        );
        let url = authnrequest::redirect_url(&cfg.idp_sso_url, &xml, &state_token)?;

        Ok(BeginRedirect {
            url,
            state_token,
            pending: PendingState::Saml {
                request_id,
                relay_state,
            },
        })
    }

    async fn complete(&self, callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        let cfg = self.config();
        let saml_response = callback
            .param("SAMLResponse")
            .ok_or_else(|| SsoError::TokenValidation("missing SAMLResponse".into()))?;

        let expected_in_response_to = match &callback.pending {
            PendingState::Saml { request_id, .. } => Some(request_id.as_str()),
            PendingState::Oidc { .. } => {
                return Err(SsoError::Config(
                    "SAML complete got OIDC pending state".into(),
                ));
            }
        };

        let now = Utc::now();
        let result = acs::process_response(
            saml_response,
            &acs::AcsContext {
                sp_entity_id: &cfg.sp_entity_id,
                acs_url: &cfg.acs_url,
                certs_pem: &cfg.idp_signing_certs_pem,
                want_signed: cfg.want_assertions_signed,
                expected_in_response_to,
                claim_map: &self.backend.claim_map,
                now,
            },
        )?;

        // Replay defence: an assertion ID is consumable at most once.
        let ttl = result
            .not_on_or_after
            .and_then(|exp| (exp - now).to_std().ok())
            .unwrap_or(REPLAY_FALLBACK_TTL);
        self.remember(&result.assertion_id, Instant::now() + ttl)?;

        Ok(result.identity)
    }

    fn metadata(&self) -> Option<Metadata> {
        let cfg = self.config();
        Some(Metadata {
            content_type: "application/samlmetadata+xml".to_string(),
            body: metadata::sp_metadata_xml(
                &cfg.sp_entity_id,
                &cfg.acs_url,
                &cfg.nameid_format,
                cfg.idp_slo_url.as_deref(),
            ),
        })
    }

    fn logout(&self, subject: &str) -> Option<Redirect> {
        let cfg = self.config();
        let slo_url = cfg.idp_slo_url.as_deref()?;
        let request_id = format!("_{}", new_state_token());
        let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let xml = build_logout_request(
            &request_id,
            &issue_instant,
            &cfg.sp_entity_id,
            slo_url,
            subject,
        );
        let state_token = new_state_token();
        authnrequest::redirect_url(slo_url, &xml, &state_token)
            .ok()
            .map(|url| Redirect { url })
    }
}

fn build_logout_request(
    request_id: &str,
    issue_instant: &str,
    sp_entity_id: &str,
    destination: &str,
    name_id: &str,
) -> String {
    let esc_attr = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('"', "&quot;")
    };
    let esc_text = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };
    format!(
        "<samlp:LogoutRequest xmlns:samlp=\"urn:oasis:names:tc:SAML:2.0:protocol\" \
         xmlns:saml=\"urn:oasis:names:tc:SAML:2.0:assertion\" ID=\"{id}\" Version=\"2.0\" \
         IssueInstant=\"{instant}\" Destination=\"{dest}\">\
         <saml:Issuer>{issuer}</saml:Issuer><saml:NameID>{nameid}</saml:NameID>\
         </samlp:LogoutRequest>",
        id = esc_attr(request_id),
        instant = esc_attr(issue_instant),
        dest = esc_attr(destination),
        issuer = esc_text(sp_entity_id),
        nameid = esc_text(name_id),
    )
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
                acs_url: "https://mail.example/acs".into(),
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
        assert_eq!(inflate(&compressed).unwrap(), xml);
        assert_ne!(compressed, xml, "should actually compress/transform");
    }

    #[tokio::test]
    async fn begin_produces_redirect_with_saml_request() {
        let p = SamlProvider::new(saml_backend()).unwrap();
        let begin = p.begin(Some("/inbox".into())).await.unwrap();
        assert!(
            begin
                .url
                .starts_with("https://idp.example/sso?SAMLRequest=")
        );
        assert!(begin.url.contains("RelayState="));
        match &begin.pending {
            PendingState::Saml {
                request_id,
                relay_state,
            } => {
                assert!(request_id.starts_with('_'));
                assert_eq!(relay_state.as_deref(), Some("/inbox"));
            }
            _ => panic!("expected SAML pending state"),
        }
    }

    #[test]
    fn metadata_and_logout_are_populated() {
        let mut b = saml_backend();
        if let SsoConfig::Saml(c) = &mut b.config {
            c.idp_slo_url = Some("https://idp.example/slo".into());
            c.nameid_format = "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent".into();
        }
        let p = SamlProvider::new(b).unwrap();
        let md = p.metadata().unwrap();
        assert_eq!(md.content_type, "application/samlmetadata+xml");
        assert!(md.body.contains("entityID=\"https://mail.example/sp\""));
        let lo = p.logout("user@acme.test").unwrap();
        assert!(lo.url.starts_with("https://idp.example/slo?SAMLRequest="));
    }
}
