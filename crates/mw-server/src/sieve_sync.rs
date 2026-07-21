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

use std::net::{IpAddr, SocketAddr};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use mw_sieve::{Credentials, ManageSieveClient, TlsMode};

use crate::{AppState, authed, image_proxy};

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

/// The Sieve egress policy (t17-e6 / L5) — deliberately NARROWER than the image
/// proxy's deny-by-default. Syncing rules to your OWN, often internal (RFC1918),
/// ManageSieve server is the legitimate use case, so private ranges stay REACHABLE.
/// Only the cloud-metadata address (`169.254.169.254`), loopback, and link-local are
/// refused — closing the coarse metadata/loopback reachability oracle without
/// breaking an internal Sieve server.
fn sieve_egress_permitted(ip: &IpAddr) -> bool {
    // Fast path: anything the image proxy already treats as public unicast is fine
    // here too (public unicast is a subset of the Sieve-allowed set).
    if image_proxy::ip_allowed(ip) {
        return true;
    }
    // Otherwise the address is in some blocked-by-image-proxy range (private,
    // loopback, link-local, …). Refuse ONLY loopback / link-local (incl. the
    // `169.254.169.254` metadata address); everything else — notably RFC1918 —
    // stays reachable.
    match ip {
        IpAddr::V4(v4) => !(v4.is_loopback() || v4.is_link_local()),
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() {
                return false;
            }
            // Link-local fe80::/10.
            if (v6.segments()[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            // Unwrap IPv4-mapped/compat and apply the same narrow rule.
            if let Some(v4) = v6.to_ipv4() {
                return !(v4.is_loopback() || v4.is_link_local());
            }
            // Unwrap every transitional embedding (NAT64, 6to4, Teredo, ISATAP) and
            // refuse if ANY embedded v4 is loopback/link-local, so a metadata/
            // loopback target cannot be smuggled through a v6 embedding.
            for v4 in image_proxy::embedded_ipv4s(v6) {
                if v4.is_loopback() || v4.is_link_local() {
                    return false;
                }
            }
            true
        }
    }
}

/// Resolve `host:port` ONCE, refuse if ANY resolved address is in the blocked set
/// ([`sieve_egress_permitted`]), and return the first allowed address to PIN the
/// connect to (t18 R1). Resolving here and dialing that exact address via
/// [`ManageSieveClient::connect_pinned`] closes the DNS-rebinding TOCTOU: a name
/// cannot re-resolve to a metadata/loopback target between this validation and the
/// connect. Refusing when even one resolved address is blocked keeps the fail-safe
/// direction (a `[public, metadata]` rebind answer is rejected outright). Returns a
/// client-error [`Response`] on refusal.
async fn gate_sieve_target(host: &str, port: u16) -> Result<SocketAddr, Response> {
    let addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| upstream(&format!("could not resolve sieve host: {host}")))?;
    let mut chosen: Option<SocketAddr> = None;
    for a in addrs {
        if !sieve_egress_permitted(&a.ip()) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "target address is not permitted" })),
            )
                .into_response());
        }
        if chosen.is_none() {
            chosen = Some(a);
        }
    }
    chosen.ok_or_else(|| upstream(&format!("could not resolve sieve host: {host}")))
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

    // Egress gate (L5) + connect-pin (R1): resolve the user-supplied host ONCE and
    // refuse metadata/loopback/link-local BEFORE connecting, returning the allowed
    // address. RFC1918/private stays reachable so an internal ManageSieve server
    // keeps working — a narrower policy than the image proxy.
    let host = req.host.trim().to_string();
    let addr = match gate_sieve_target(&host, req.port).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    // Connect, upload, activate, and log out. The connect is PINNED to the gated
    // address (`connect_pinned`), so it cannot re-resolve `host` to a rebound
    // metadata/loopback target; `host` is kept for TLS SNI. Any protocol/transport
    // error surfaces as a 502 with the ManageSieve server's message (never mail
    // content).
    let creds = Credentials::Plain {
        username: req.username,
        password: req.password,
    };
    let mut client = match ManageSieveClient::connect_pinned(&host, addr, tls, creds).await {
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

    // ── L5 egress policy (narrower than the image proxy) ──────────────────────

    #[test]
    fn sieve_refuses_metadata_loopback_and_link_local() {
        for s in [
            "169.254.169.254",    // cloud metadata
            "169.254.1.1",        // link-local
            "127.0.0.1",          // loopback
            "::1",                // v6 loopback
            "::",                 // unspecified
            "fe80::1",            // v6 link-local
            "::ffff:127.0.0.1",   // IPv4-mapped loopback
            "::ffff:169.254.1.1", // IPv4-mapped link-local
            "64:ff9b::a9fe:a9fe", // NAT64-embedded metadata
            "2002:7f00:1::",      // 6to4-embedded loopback
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!sieve_egress_permitted(&ip), "{s} must be refused");
        }
    }

    // ── R1: the gate resolves once and returns the address the connect is pinned to
    #[tokio::test]
    async fn gate_returns_pinned_addr_for_allowed_and_refuses_blocked() {
        // An RFC1918 literal resolves to itself, is permitted, and is returned as the
        // pinned address the connect must dial (no re-resolution after this point).
        let addr = gate_sieve_target("10.0.0.1", 4190)
            .await
            .expect("RFC1918 target must be allowed");
        assert_eq!(addr, "10.0.0.1:4190".parse::<SocketAddr>().unwrap());
        // Metadata / loopback / link-local literals are refused BEFORE any connect,
        // so a rebind-after-validate can never reach them.
        for blocked in ["169.254.169.254", "127.0.0.1", "169.254.1.1"] {
            assert!(
                gate_sieve_target(blocked, 4190).await.is_err(),
                "{blocked} must be refused by the gate"
            );
        }
    }

    #[test]
    fn sieve_refuses_teredo_and_isatap_embedded_loopback() {
        // The narrower Sieve policy still unwraps Teredo/ISATAP embeddings and
        // refuses a smuggled loopback/metadata v4 (link-local set), while keeping
        // RFC1918 reachable (covered separately below).
        for s in [
            // Teredo client v4 127.0.0.1 (obfuscated 0x80fffffe), public server.
            "2001:0:4136:e378:8000:ffff:80ff:fffe",
            // Teredo client v4 169.254.169.254 (metadata; obfuscated 0x56015601).
            "2001:0:4136:e378:8000:ffff:5601:5601",
            // ISATAP global-prefix IID wrapping 127.0.0.1.
            "2001:470::5efe:7f00:1",
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!sieve_egress_permitted(&ip), "{s} must be refused");
        }
    }

    #[test]
    fn sieve_keeps_rfc1918_and_public_reachable() {
        // The deliberate difference from the image proxy: private ranges stay
        // reachable so an INTERNAL ManageSieve server is a legitimate target.
        for s in [
            "10.0.0.1",           // RFC1918
            "10.255.255.255",     // RFC1918
            "172.16.0.1",         // RFC1918
            "172.31.255.255",     // RFC1918
            "192.168.1.1",        // RFC1918
            "8.8.8.8",            // public
            "93.184.216.34",      // public
            "fc00::1",            // ULA (private-equivalent v6 — reachable)
            "2606:2800:220:1::1", // public v6
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(sieve_egress_permitted(&ip), "{s} must stay reachable");
        }
    }
}
