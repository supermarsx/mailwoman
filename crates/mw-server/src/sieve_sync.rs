//! A9 (t16 26.16, e10): a runtime ManageSieve caller.
//!
//! `mw-sieve` ships a full ManageSieve (RFC 5804) client ([`mw_sieve::ManageSieveClient`])
//! and a rule→Sieve code generator ([`mw_sieve::generate`]), but nothing called them
//! at runtime — the engine's always-green path evaluates rules locally instead. This
//! wires the missing caller: on request, the account's stored GUI/Sieve rules are
//! compiled to a Sieve script and uploaded (+ activated) on the user's ManageSieve
//! server, so a backend that DOES speak Sieve runs the rules server-side.
//!
//! The connection parameters (host/port/TLS/credentials) are supplied in the
//! authenticated request — these are the user's own upstream ManageSieve
//! credentials, presented over their session; the server does not persist them. The
//! target speaks the ManageSieve line protocol (not HTTP), so it is not a generic
//! egress surface.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use mw_sieve::{Credentials, ManageSieveClient, TlsMode};

use crate::{AppState, authed};

/// The A9 ManageSieve-sync route (mounted by `lib.rs`/e10).
pub(crate) fn sieve_sync_router() -> Router<AppState> {
    Router::new().route("/api/account/sieve/sync", post(sieve_sync))
}

#[derive(Debug, Deserialize)]
struct SieveSyncReq {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    /// `"implicit"` | `"starttls"` | `"plaintext"` (default `starttls`).
    #[serde(default)]
    tls: Option<String>,
    username: String,
    password: String,
    /// The script name to upload/activate (default `mailwoman`).
    #[serde(rename = "scriptName", default)]
    script_name: Option<String>,
}

/// ManageSieve's IANA-assigned port.
fn default_port() -> u16 {
    4190
}

/// Parse the requested transport security (default STARTTLS — the common
/// ManageSieve deployment). Returns `None` for an unknown value (rejected by the
/// caller rather than silently downgraded).
fn parse_tls(mode: Option<&str>) -> Option<TlsMode> {
    match mode.unwrap_or("starttls").to_ascii_lowercase().as_str() {
        "implicit" | "tls" | "implicittls" => Some(TlsMode::Implicit),
        "starttls" | "" => Some(TlsMode::StartTls),
        "plaintext" | "none" => Some(TlsMode::Plaintext),
        _ => None,
    }
}

/// `POST /api/account/sieve/sync` — compile the account's rules to a Sieve script
/// and upload+activate it on the user's ManageSieve server.
async fn sieve_sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SieveSyncReq>,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let Some(engine) = &state.engine else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "sieve sync requires engine mode" })),
        )
            .into_response();
    };
    let Some(tls) = parse_tls(req.tls.as_deref()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "unknown tls mode (use implicit|starttls|plaintext)" })),
        )
            .into_response();
    };
    if req.host.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "host is required" })),
        )
            .into_response();
    }

    // Compile the account's stored rules to a Sieve script.
    let rules = match engine.get_rules(&session.account_id).await {
        Ok(r) => r,
        Err(e) => return upstream(&format!("could not load rules: {e}")),
    };
    let script = match mw_sieve::generate(&rules) {
        Ok(s) => s,
        Err(e) => return upstream(&format!("sieve generation failed: {e}")),
    };
    let script_name = req
        .script_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("mailwoman")
        .to_string();

    // Connect, upload, activate, and log out. Any protocol/transport error surfaces
    // as a 502 with the ManageSieve server's message (never mail content).
    let creds = Credentials::Plain {
        username: req.username,
        password: req.password,
    };
    let mut client = match ManageSieveClient::connect(req.host.trim(), req.port, tls, creds).await {
        Ok(c) => c,
        Err(e) => return upstream(&format!("connect failed: {e}")),
    };
    if let Err(e) = client.put_script(&script_name, &script).await {
        return upstream(&format!("PUTSCRIPT failed: {e}"));
    }
    if let Err(e) = client.set_active(&script_name).await {
        return upstream(&format!("SETACTIVE failed: {e}"));
    }
    let _ = client.logout().await;

    Json(json!({
        "uploaded": true,
        "scriptName": script_name,
        "rules": rules.len(),
        "scriptBytes": script.len(),
    }))
    .into_response()
}

fn upstream(msg: &str) -> Response {
    tracing::warn!("sieve sync: {msg}");
    (StatusCode::BAD_GATEWAY, Json(json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_mode_parsing() {
        assert!(matches!(parse_tls(None), Some(TlsMode::StartTls)));
        assert!(matches!(
            parse_tls(Some("implicit")),
            Some(TlsMode::Implicit)
        ));
        assert!(matches!(
            parse_tls(Some("STARTTLS")),
            Some(TlsMode::StartTls)
        ));
        assert!(matches!(
            parse_tls(Some("plaintext")),
            Some(TlsMode::Plaintext)
        ));
        assert!(parse_tls(Some("bogus")).is_none());
    }

    #[test]
    fn default_port_is_managesieve() {
        assert_eq!(default_port(), 4190);
    }
}
