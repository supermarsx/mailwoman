//! [`WebhookHmac`] — POST a password-change to a custom webhook, HMAC-SHA256 signed.
//!
//! The JSON payload is signed with a shared secret; the hex MAC travels in an
//! `X-Signature: sha256=<hex>` header over the *exact* body bytes that are sent, so the
//! receiver can authenticate it with [`verify_signature`]. Transport is `reqwest`
//! (rustls — no openssl). `hmac`/`sha2` are the in-tree pins (hmac 0.13 ⇄ sha2 0.11).

use async_trait::async_trait;
use hmac::{Hmac, KeyInit, Mac};
use serde::Serialize;
use sha2::Sha256;

use crate::{
    BackendKind, Ctx, PasswordChangeBackend, PasswordChangeOutcome, PasswordError, PasswordPolicy,
    Result, Secret,
};

type HmacSha256 = Hmac<Sha256>;

/// Config for the HMAC-signed webhook backend.
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// The webhook URL to POST the signed payload to.
    pub url: String,
    /// The shared HMAC-SHA256 signing secret.
    pub secret: Vec<u8>,
    /// Policy shown/enforced before the change.
    pub policy: PasswordPolicy,
}

impl WebhookConfig {
    #[must_use]
    pub fn new(url: impl Into<String>, secret: impl Into<Vec<u8>>) -> Self {
        Self {
            url: url.into(),
            secret: secret.into(),
            policy: PasswordPolicy::default(),
        }
    }
}

/// The signed payload sent to the webhook.
#[derive(Debug, Serialize)]
struct Payload<'a> {
    event: &'static str,
    account_id: &'a str,
    username: &'a str,
    new_password: &'a str,
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// Compute the `X-Signature` header value (`sha256=<hex>`) for `body` under `secret`.
#[must_use]
pub fn sign(secret: &[u8], body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    format!("sha256={}", hex_encode(&mac.finalize().into_bytes()))
}

/// Constant-time verify of an `X-Signature: sha256=<hex>` header over `body`.
///
/// Returns `false` for a missing/malformed prefix, bad hex, or a MAC mismatch.
#[must_use]
pub fn verify_signature(secret: &[u8], body: &[u8], header: &str) -> bool {
    let Some(hex) = header.strip_prefix("sha256=") else {
        return false;
    };
    let Some(expected) = hex_decode(hex) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

/// Custom webhook password change with an HMAC-SHA256-signed payload (plan §2.3).
pub struct WebhookHmac {
    config: WebhookConfig,
    client: reqwest::Client,
}

impl WebhookHmac {
    #[must_use]
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Serialize the payload and compute its signature header — pure, no network I/O.
    fn signed_body(&self, ctx: &Ctx, new: &Secret) -> Result<(Vec<u8>, String)> {
        let payload = Payload {
            event: "password_change",
            account_id: &ctx.account_id,
            username: &ctx.username,
            new_password: new.expose(),
        };
        let body =
            serde_json::to_vec(&payload).map_err(|e| PasswordError::Protocol(e.to_string()))?;
        let sig = sign(&self.config.secret, &body);
        Ok((body, sig))
    }
}

#[async_trait]
impl PasswordChangeBackend for WebhookHmac {
    async fn change(&self, ctx: &Ctx, _old: Secret, new: Secret) -> Result<PasswordChangeOutcome> {
        self.config.policy.validate(&new)?;
        let (body, sig) = self.signed_body(ctx, &new)?;
        let resp = self
            .client
            .post(&self.config.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("X-Signature", sig)
            .body(body)
            .send()
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(PasswordError::Transport(format!("HTTP {}", resp.status())));
        }
        Ok(PasswordChangeOutcome::changed_from(ctx))
    }

    fn policy(&self) -> PasswordPolicy {
        self.config.policy.clone()
    }

    fn kind(&self) -> BackendKind {
        BackendKind::WebhookHmac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_round_trips() {
        let secret = b"shared-webhook-secret";
        let body = br#"{"event":"password_change"}"#;
        let sig = sign(secret, body);
        assert!(sig.starts_with("sha256="));
        assert!(verify_signature(secret, body, &sig));
    }

    #[test]
    fn verify_rejects_tampered_body_wrong_key_and_bad_header() {
        let secret = b"k1";
        let body = b"payload";
        let sig = sign(secret, body);
        assert!(!verify_signature(secret, b"payload-tampered", &sig));
        assert!(!verify_signature(b"k2", body, &sig));
        assert!(!verify_signature(secret, body, "md5=deadbeef"));
        assert!(!verify_signature(secret, body, "sha256=nothex!!"));
    }

    #[test]
    fn signature_covers_the_exact_body_sent() {
        let cfg = WebhookConfig::new("https://hook.example.com/pw", b"topsecret".to_vec());
        let backend = WebhookHmac::new(cfg);
        let ctx = Ctx::new("acct-7", "bob");
        let (body, header) = backend
            .signed_body(&ctx, &Secret::new("new-strong-password"))
            .unwrap();
        // The receiver, holding the same secret, authenticates the exact bytes.
        assert!(verify_signature(b"topsecret", &body, &header));
        // Payload carries the account context (content is intentional on the wire, not a log).
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["event"], "password_change");
        assert_eq!(v["account_id"], "acct-7");
    }

    #[tokio::test]
    async fn deny_path_policy_rejects_before_request() {
        let mut cfg = WebhookConfig::new("http://127.0.0.1:1/pw", b"k".to_vec());
        cfg.policy.min_length = 40;
        let backend = WebhookHmac::new(cfg);
        let err = backend
            .change(
                &Ctx::new("a", "u"),
                Secret::new("old"),
                Secret::new("short"),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, PasswordError::PolicyViolation(_)));
    }
}
