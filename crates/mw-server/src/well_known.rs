//! `GET /.well-known/security.txt` (RFC 9116) from the settings
//! `about.securityContact` and `about.securityPolicyUrl`.
//!
//! With no contact set the route answers `404`: RFC 9116 requires at least one
//! `Contact` field, so a file without one would be invalid. `Expires` is one
//! year from the start of the current UTC day, so the file never goes stale
//! while the server runs.

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::AppState;
use crate::newly_effective::{global, text};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/.well-known/security.txt", get(security_txt))
}

async fn security_txt(State(state): State<AppState>) -> Response {
    let contact = text(&state.store, "about.securityContact", &global()).await;
    let policy = text(&state.store, "about.securityPolicyUrl", &global()).await;
    match render(contact.as_deref(), policy.as_deref(), chrono::Utc::now()) {
        Some(body) => (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            body,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// A field value on one line: control characters (a CR or LF would start a new
/// field) are dropped.
fn one_line(value: &str) -> String {
    value.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string()
}

/// The file body; `None` without a contact. A bare e-mail address becomes a
/// `mailto:` URI, as RFC 9116 §2.5.3 requires a URI.
pub(crate) fn render(
    contact: Option<&str>,
    policy: Option<&str>,
    now: chrono::DateTime<chrono::Utc>,
) -> Option<String> {
    let contact = one_line(contact?);
    if contact.is_empty() {
        return None;
    }
    let has_scheme = contact.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    let contact = if has_scheme {
        contact
    } else {
        format!("mailto:{contact}")
    };
    let expires = now.date_naive() + chrono::Days::new(365);
    let mut body = format!(
        "Contact: {contact}\nExpires: {}T00:00:00Z\n",
        expires.format("%Y-%m-%d")
    );
    if let Some(policy) = policy.map(one_line).filter(|p| !p.is_empty()) {
        body.push_str(&format!("Policy: {policy}\n"));
    }
    Some(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.with_ymd_and_hms(2026, 11, 23, 15, 4, 5).unwrap()
    }

    #[test]
    fn no_contact_means_no_file() {
        assert_eq!(render(None, Some("https://example.org/policy"), at()), None);
        assert_eq!(render(Some("  "), None, at()), None);
    }

    #[test]
    fn contact_expires_and_policy_are_written() {
        assert_eq!(
            render(
                Some("mailto:security@example.org"),
                Some("https://example.org/security"),
                at()
            )
            .unwrap(),
            "Contact: mailto:security@example.org\nExpires: 2027-11-23T00:00:00Z\n\
             Policy: https://example.org/security\n"
        );
        assert_eq!(
            render(Some("https://example.org/report"), None, at()).unwrap(),
            "Contact: https://example.org/report\nExpires: 2027-11-23T00:00:00Z\n"
        );
    }

    #[test]
    fn a_bare_address_becomes_mailto_and_line_breaks_are_dropped() {
        assert_eq!(
            render(Some("sec@example.org\r\nPolicy: x"), None, at()).unwrap(),
            "Contact: mailto:sec@example.orgPolicy: x\nExpires: 2027-11-23T00:00:00Z\n"
        );
    }
}
