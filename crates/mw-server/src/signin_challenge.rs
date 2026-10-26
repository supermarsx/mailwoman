//! Self-hosted ALTCHA v2 proof-of-work for initial password sign-in.
use crate::{AppState, scope_mw::rate_limit::KeyedRateLimiter, settings_registry};
use axum::{
    Json,
    extract::{Query, State},
    http::{Extensions, HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, KeyInit, Mac};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

pub(crate) struct IssuanceLimiter(Mutex<KeyedRateLimiter>, Arc<tokio::sync::Semaphore>);
impl Default for IssuanceLimiter {
    fn default() -> Self {
        Self(
            Mutex::new(KeyedRateLimiter::new(2.0, 1.0 / 3.0, 10_000)),
            Arc::new(tokio::sync::Semaphore::new(8)),
        )
    }
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Surface {
    Mailbox,
    Admin,
}
impl Surface {
    fn name(self) -> &'static str {
        match self {
            Self::Mailbox => "mailbox",
            Self::Admin => "admin",
        }
    }
}
#[derive(Deserialize)]
pub(crate) struct ChallengeQuery {
    surface: Surface,
}
pub(crate) struct Config {
    mode: String,
    difficulty: u32,
    expiry: i64,
}
impl Config {
    pub(crate) fn enabled(&self, surface: Surface) -> bool {
        self.mode == "all" || self.mode == surface.name()
    }
}
pub(crate) async fn config(state: &AppState) -> Result<Config, ()> {
    let values = settings_registry::effective_all(&state.store, &Default::default(), |def| {
        matches!(
            def.key,
            "signin.antiBotMode" | "signin.antiBotDifficulty" | "signin.antiBotExpirySecs"
        )
    })
    .await
    .map_err(|_| ())?;
    let value = |key: &str| {
        values
            .iter()
            .find(|value| value.key == key)
            .map(|value| value.value.clone())
            .unwrap_or_default()
    };
    Ok(Config {
        mode: value("signin.antiBotMode")
            .as_str()
            .unwrap_or("off")
            .to_string(),
        difficulty: value("signin.antiBotDifficulty")
            .as_u64()
            .unwrap_or(2000)
            .clamp(100, 20000) as u32,
        expiry: value("signin.antiBotExpirySecs")
            .as_i64()
            .unwrap_or(180)
            .clamp(60, 600),
    })
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn peer_key(state: &AppState, ip: IpAddr) -> String {
    let normalized = match ip {
        IpAddr::V4(v) => v.to_string(),
        IpAddr::V6(v) => match v.to_ipv4_mapped() {
            Some(v) => v.to_string(),
            None => {
                let s = v.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        },
    };
    let secret = state.store.signin_challenge_secret("peer");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("fixed HMAC key");
    mac.update(normalized.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn response(status: StatusCode, body: serde_json::Value) -> Response {
    let mut response = (status, Json(body)).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
fn unavailable() -> Response {
    response(
        StatusCode::SERVICE_UNAVAILABLE,
        json!({"code":"signinChallengeUnavailable","error":"Verification is temporarily unavailable."}),
    )
}
fn rejected(missing: bool) -> Response {
    response(
        StatusCode::FORBIDDEN,
        json!({"code":if missing{"signinChallengeRequired"}else{"signinChallengeRejected"},"error":"Complete a new sign-in verification."}),
    )
}
pub(crate) async fn issue(
    State(state): State<AppState>,
    Query(query): Query<ChallengeQuery>,
    headers: HeaderMap,
    ext: Extensions,
) -> Response {
    let Ok(policy) = config(&state).await else {
        return unavailable();
    };
    if !policy.enabled(query.surface) {
        return response(StatusCode::NOT_FOUND, json!({"code":"notFound"}));
    }
    let Some(ip) = crate::scope_mw::proxy::client_ip(&headers, &ext) else {
        return unavailable();
    };
    let peer = peer_key(&state, ip);
    let allowed = state
        .signin_challenges
        .0
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .check(&peer);
    if !allowed {
        let mut r = response(
            StatusCode::TOO_MANY_REQUESTS,
            json!({"code":"signinChallengeThrottled","retryAfterSecs":3}),
        );
        r.headers_mut()
            .insert(header::RETRY_AFTER, "3".parse().unwrap());
        return r;
    }
    let Ok(permit) = state.signin_challenges.1.clone().try_acquire_owned() else {
        return unavailable();
    };
    let issued = now();
    let options = altcha::CreateChallengeOptions {
        algorithm: "PBKDF2/SHA-256".into(),
        cost: 1000,
        counter: Some(rand::thread_rng().gen_range((policy.difficulty / 2)..=policy.difficulty)),
        expires_at: Some(issued + policy.expiry),
        hmac_signature_secret: Some(state.store.signin_challenge_secret("signature")),
        hmac_key_signature_secret: Some(state.store.signin_challenge_secret("derived-key")),
        ..Default::default()
    };
    let (challenge, _permit) = match tokio::task::spawn_blocking(move || {
        (altcha::create_challenge(options), permit)
    })
    .await
    {
        Ok((Ok(challenge), permit)) => (challenge, permit),
        _ => return unavailable(),
    };
    match state
        .store
        .issue_signin_challenge(
            &challenge.parameters.nonce,
            query.surface.name(),
            &peer,
            issued,
            issued + policy.expiry,
        )
        .await
    {
        Ok(true) => response(StatusCode::OK, json!(challenge)),
        Ok(false) => {
            let mut reply = response(
                StatusCode::TOO_MANY_REQUESTS,
                json!({"code":"signinChallengeThrottled","retryAfterSecs":policy.expiry}),
            );
            reply.headers_mut().insert(
                header::RETRY_AFTER,
                policy.expiry.to_string().parse().unwrap(),
            );
            reply
        }
        Err(_) => unavailable(),
    }
}
/// Called after cooldown and before any password, LDAP bind or upstream request.
pub(crate) async fn verify(
    state: &AppState,
    surface: Surface,
    headers: &HeaderMap,
    ext: &Extensions,
    encoded: Option<&str>,
) -> Option<Response> {
    let policy = match config(state).await {
        Ok(policy) => policy,
        Err(_) => return Some(unavailable()),
    };
    if !policy.enabled(surface) {
        return None;
    }
    let Some(encoded) = encoded.filter(|v| !v.is_empty()) else {
        return Some(rejected(true));
    };
    if encoded.len() > 8192 {
        return Some(rejected(false));
    }
    let Some(ip) = crate::scope_mw::proxy::client_ip(headers, ext) else {
        return Some(rejected(false));
    };
    let payload: altcha::Payload = match STANDARD
        .decode(encoded)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    {
        Some(payload) => payload,
        None => return Some(rejected(false)),
    };
    let p = &payload.challenge.parameters;
    // Only our fixed, bounded algorithm is accepted. No attacker-controlled KDF work.
    if p.algorithm != "PBKDF2/SHA-256"
        || p.cost != 1000
        || p.key_length != 32
        || p.key_signature.as_ref().is_none_or(|v| v.len() != 64)
        || p.nonce.len() != 32
        || p.expires_at
            .is_none_or(|expires| expires <= now() || expires > now() + 600)
        || payload.solution.derived_key.len() != 64
    {
        return Some(rejected(false));
    }
    let verified = altcha::verify_solution(altcha::VerifySolutionOptions {
        hmac_key_signature_secret: Some(state.store.signin_challenge_secret("derived-key")),
        ..altcha::VerifySolutionOptions::new(
            &payload.challenge,
            &payload.solution,
            state.store.signin_challenge_secret("signature"),
        )
    })
    .is_ok_and(|result| result.verified);
    if !verified {
        return Some(rejected(false));
    }
    match state
        .store
        .consume_signin_challenge(&p.nonce, surface.name(), &peer_key(state, ip), now())
        .await
    {
        Ok(true) => None,
        Ok(false) => Some(rejected(false)),
        Err(_) => Some(unavailable()),
    }
}
