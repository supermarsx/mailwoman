//! A first-party browser marker for informational new-device alerts only.
//! It never authenticates, remembers a factor, or authorizes a session.
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub(crate) const COOKIE: &str = "mw_browser_device";
const LIFETIME_SECS: u64 = 365 * 86400;
#[derive(Clone)]
pub(crate) struct BrowserFingerprint(pub String);

fn valid_token(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn cookie(token: &str, secure: bool) -> HeaderValue {
    let mut value = HeaderValue::from_str(&format!(
        "{COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={LIFETIME_SECS}{}",
        if secure { "; Secure" } else { "" }
    ))
    .expect("browser marker is hex");
    value.set_sensitive(true);
    value
}
pub(crate) async fn layer(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let sso = path.starts_with("/api/sso/")
        && ["/begin", "/callback", "/acs"]
            .iter()
            .any(|suffix| path.ends_with(suffix));
    if !(path == "/api/login" || path.starts_with("/api/login/") || sso) {
        return next.run(request).await;
    }
    let existing =
        crate::hardening::cookie(request.headers(), COOKIE).filter(|token| valid_token(token));
    // SameSite=Lax is deliberately withheld on a SAML cross-site POST. The
    // validated one-shot SSO flow carries its original fingerprint instead.
    let saml_post = path.ends_with("/acs") && request.method() == axum::http::Method::POST;
    let mut fresh = None;
    let token = existing.or_else(|| {
        if saml_post {
            return None;
        }
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let token = bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        fresh = Some(token.clone());
        Some(token)
    });
    if let Some(token) = token {
        request.extensions_mut().insert(BrowserFingerprint(
            Sha256::digest(token.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        ));
    }
    let mut response = next.run(request).await;
    if (response.status().is_success() || response.status().is_redirection())
        && let Some(token) = fresh
    {
        response
            .headers_mut()
            .append(header::SET_COOKIE, cookie(&token, state.cookie_secure));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_is_strictly_bounded_and_has_only_first_party_cookie_privileges() {
        let token = "a".repeat(64);
        assert!(valid_token(&token));
        for invalid in ["short".to_string(), "g".repeat(64), "a".repeat(65)] {
            assert!(!valid_token(&invalid));
        }
        let secure = cookie(&token, true);
        let text = secure.to_str().unwrap();
        assert!(
            text.contains("HttpOnly") && text.contains("SameSite=Lax") && text.contains("; Secure")
        );
        assert!(!text.contains("Domain="));
        assert!(secure.is_sensitive());
        assert!(!cookie(&token, false).to_str().unwrap().contains("; Secure"));
    }
}
