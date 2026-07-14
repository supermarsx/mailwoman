//! [`DovecotHttp`] — change a password via Dovecot's doveadm HTTP admin API.
//!
//! The doveadm HTTP API (`/doveadm/v1`) takes a JSON array of commands
//! `[[ "<command>", { params }, "<tag>" ]]` and authenticates with an
//! `Authorization: X-Dovecot-API <base64(api-key)>` header. The request *shape* is
//! built by the pure [`DovecotHttp::build_request`] (unit-tested without a socket); the
//! backend then sends it over `reqwest` (rustls — no openssl).

use async_trait::async_trait;
use base64::Engine;
use serde_json::json;

use crate::{
    BackendKind, Ctx, PasswordChangeBackend, PasswordChangeOutcome, PasswordError, PasswordPolicy,
    Result, Secret,
};

/// Config for the Dovecot doveadm HTTP backend.
#[derive(Debug, Clone)]
pub struct DovecotConfig {
    /// The doveadm HTTP API endpoint, e.g. `https://mail.example.com:8080/doveadm/v1`.
    pub api_url: String,
    /// The doveadm API key (sent base64-encoded in the `X-Dovecot-API` auth header).
    pub api_key: String,
    /// The doveadm command that sets a password (deployment-dependent; default `pw`).
    pub command: String,
    /// Policy shown/enforced before the change.
    pub policy: PasswordPolicy,
}

impl DovecotConfig {
    #[must_use]
    pub fn new(api_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            api_key: api_key.into(),
            command: "pw".into(),
            policy: PasswordPolicy::default(),
        }
    }
}

/// The parts of the doveadm HTTP request, built without touching the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DovecotRequest {
    pub url: String,
    /// The full `Authorization` header value (`X-Dovecot-API <base64>`).
    pub auth_header: String,
    /// The JSON body (a doveadm command array).
    pub body: serde_json::Value,
}

/// Dovecot doveadm HTTP admin API password change (plan §2.3).
pub struct DovecotHttp {
    config: DovecotConfig,
    client: reqwest::Client,
}

impl DovecotHttp {
    #[must_use]
    pub fn new(config: DovecotConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Build the doveadm HTTP request envelope for a password change. Pure — issues no
    /// network I/O — so the request shape is unit-testable.
    #[must_use]
    pub fn build_request(&self, ctx: &Ctx, new: &Secret) -> DovecotRequest {
        let auth = base64::engine::general_purpose::STANDARD.encode(&self.config.api_key);
        let body = json!([[
            self.config.command,
            { "user": ctx.username, "password": new.expose() },
            format!("mw-passwd-{}", ctx.account_id),
        ]]);
        DovecotRequest {
            url: self.config.api_url.clone(),
            auth_header: format!("X-Dovecot-API {auth}"),
            body,
        }
    }
}

#[async_trait]
impl PasswordChangeBackend for DovecotHttp {
    async fn change(&self, ctx: &Ctx, _old: Secret, new: Secret) -> Result<PasswordChangeOutcome> {
        self.config.policy.validate(&new)?;
        let req = self.build_request(ctx, &new);
        let resp = self
            .client
            .post(&req.url)
            .header(reqwest::header::AUTHORIZATION, &req.auth_header)
            .json(&req.body)
            .send()
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(PasswordError::Transport(format!("HTTP {}", resp.status())));
        }
        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PasswordError::Protocol(e.to_string()))?;
        // doveadm signals failure with a leading "error" command element.
        if value.get(0).and_then(|c| c.get(0)).and_then(|s| s.as_str()) == Some("error") {
            return Err(PasswordError::Protocol("doveadm error response".into()));
        }
        Ok(PasswordChangeOutcome::changed_from(ctx))
    }

    fn policy(&self) -> PasswordPolicy {
        self.config.policy.clone()
    }

    fn kind(&self) -> BackendKind {
        BackendKind::DovecotHttp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_shape_matches_doveadm_http_api() {
        let cfg = DovecotConfig::new("https://mail.example.com:8080/doveadm/v1", "s3cr3t-key");
        let backend = DovecotHttp::new(cfg);
        let ctx = Ctx::new("acct-9", "alice@example.com");
        let req = backend.build_request(&ctx, &Secret::new("new-password-value"));

        assert_eq!(req.url, "https://mail.example.com:8080/doveadm/v1");
        // Authorization: X-Dovecot-API <base64(api-key)>
        let expected_b64 = base64::engine::general_purpose::STANDARD.encode("s3cr3t-key");
        assert_eq!(req.auth_header, format!("X-Dovecot-API {expected_b64}"));

        // Body: [[ "pw", { user, password }, "mw-passwd-acct-9" ]]
        let cmd = &req.body[0];
        assert_eq!(cmd[0], "pw");
        assert_eq!(cmd[1]["user"], "alice@example.com");
        assert_eq!(cmd[1]["password"], "new-password-value");
        assert_eq!(cmd[2], "mw-passwd-acct-9");
    }

    #[tokio::test]
    async fn deny_path_policy_rejects_before_any_request() {
        let mut cfg = DovecotConfig::new("http://127.0.0.1:1/doveadm/v1", "k");
        cfg.policy.min_length = 30;
        let backend = DovecotHttp::new(cfg);
        // Too short ⇒ PolicyViolation, and crucially no HTTP call is attempted.
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
