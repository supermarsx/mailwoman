use super::*;
use axum::{
    extract::{Request, State},
    middleware::Next,
};

fn id_path(path: &str, prefix: &str, segments: usize) -> bool {
    path.strip_prefix(prefix).is_some_and(|tail| {
        tail.split('/').count() == segments && tail.split('/').all(|v| !v.is_empty())
    })
}
pub(crate) fn allowed(path: &str) -> bool {
    matches!(
        path,
        "/api/me"
            | "/api/policy"
            | "/jmap/session"
            | "/jmap/api"
            | "/api/settings"
            | "/api/account/preferences"
            | "/api/account/appearance"
            | "/api/account/signatures"
            | "/api/account/notifications"
            | "/api/account/saved-searches"
            | "/api/account/identities"
            | "/api/account/sender/resolve"
            | "/api/account/mdn-policy"
            | "/api/account/drafts"
            | "/api/account/drafts/resolve"
            | "/api/account/session/activity"
            | "/api/attachments/policy"
            | "/api/security/step-up"
            | "/api/security/step-up/passkey"
            | "/api/security/dlp/config"
            | "/api/security/watermark"
            | "/api/sanitize"
            | "/api/attachments/preview"
            | "/api/mail/delivery-status"
            | "/api/rc/sign"
            | "/api/rc/stats"
            | "/api/rc/policy"
    ) || id_path(path, "/api/settings/", 1)
        || id_path(path, "/api/account/signatures/", 1)
        || id_path(path, "/api/account/saved-searches/", 1)
        || id_path(path, "/api/account/identities/", 1)
        || id_path(path, "/api/account/drafts/", 1)
        || (path.ends_with("/cleanup") && id_path(path, "/api/account/drafts/", 2))
        || id_path(path, "/jmap/upload/", 1)
        || resource(path)
}
fn resource(path: &str) -> bool {
    id_path(path, "/jmap/download/", 3) || id_path(path, "/api/export/", 1)
}
pub(crate) async fn guard(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if !request.headers().contains_key(SELECTOR) && request.uri().query().is_none() {
        return next.run(request).await;
    }
    let mut query_url = reqwest::Url::parse("http://linked.invalid/").expect("fixed URL");
    query_url.set_query(request.uri().query());
    let pairs: Vec<(String, String)> = query_url.query_pairs().into_owned().collect();
    let links: Vec<_> = pairs.iter().filter(|(key, _)| key == "mwLink").collect();
    let header_count = request.headers().get_all(SELECTOR).iter().count();
    if header_count > 1 || links.len() > 1 {
        return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
    }
    if let Some((_, link)) = links.first() {
        if !resource(request.uri().path())
            || request.method() != axum::http::Method::GET
            || header_count > 0
            || request
                .headers()
                .contains_key(axum::http::header::AUTHORIZATION)
        {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        }
        let Ok(value) = axum::http::HeaderValue::from_str(link) else {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        };
        request.headers_mut().insert(SELECTOR, value);
        query_url.set_query(None);
        query_url
            .query_pairs_mut()
            .extend_pairs(pairs.iter().filter(|(key, _)| key != "mwLink"));
        let query = query_url.query().unwrap_or("");
        let uri = if query.is_empty() {
            request.uri().path().to_string()
        } else {
            format!("{}?{}", request.uri().path(), query)
        };
        let Ok(uri) = uri.parse() else {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        };
        *request.uri_mut() = uri;
    }
    if let Some(link) = request.headers().get(SELECTOR) {
        if !allowed(request.uri().path()) {
            return failure(StatusCode::FORBIDDEN, "linkedContextUnsupported");
        }
        if link
            .to_str()
            .ok()
            .and_then(|v| uuid::Uuid::parse_str(v).ok())
            .is_none()
        {
            return failure(StatusCode::BAD_REQUEST, "invalidLinkedContext");
        }
        // Scoped API keys/OAuth adapters may bypass normal session resolution.
        // They cannot select links. Establish a real primary cookie/native
        // session here before any handler-specific scoped adapter can execute.
        if request.headers().contains_key("x-api-key") {
            return failure(StatusCode::FORBIDDEN, "linkedContextUnsupported");
        }
        let mut primary = request.headers().clone();
        primary.remove(SELECTOR);
        if let Err(response) = crate::authed(&state, &primary).await {
            return response;
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linked_scope_is_explicit_and_excludes_primary_security_and_streams() {
        for path in [
            "/api/me",
            "/api/policy",
            "/jmap/api",
            "/api/account/identities/x",
            "/jmap/download/a/b/name",
            "/api/settings/mail.folderRole.sent",
        ] {
            assert!(allowed(path), "{path}");
        }
        for path in [
            "/api/login",
            "/api/accounts",
            "/api/security/passkeys",
            "/api/session/rotate",
            "/oidc/token",
            "/oauth/decision",
            "/jmap/ws",
            "/jmap/eventsource",
            "/admin/settings",
            "/api/unknown",
            "/api/account/identities/x/more",
        ] {
            assert!(!allowed(path), "{path}");
        }
    }
}
